use std::{
    alloc::Layout,
    any::TypeId,
    cell::{RefCell, UnsafeCell},
    ops::{Deref, DerefMut},
    ptr::NonNull,
    rc::Rc,
};

use crate::{bitset::BitSet, EntityId, MAX_ENTITY_COUNT};

type EntityBitSet = [u8; MAX_ENTITY_COUNT / 8];

pub struct ComponentStore {
    component_layout: Layout,
    data: UnsafeCell<NonNull<u8>>,
    cap: usize,
    entities_bitset: EntityBitSet,
    dirty_bitset: Rc<RefCell<EntityBitSet>>,
    read_access_bitset: Rc<RefCell<EntityBitSet>>,
    write_access_bitset: Rc<RefCell<EntityBitSet>>,
    drop_fn: unsafe fn(*mut u8),
}

impl ComponentStore {
    pub fn new(component_layout: Layout, drop_fn: unsafe fn(*mut u8)) -> Self {
        // In the case of ZSTs we don't want to allocate any data. To avoid any
        // allocation, we consider the component store to have the maximum
        // capacity
        let cap = if component_layout.size() == 0 {
            usize::MAX
        } else {
            0usize
        };

        Self {
            component_layout,
            data: UnsafeCell::new(NonNull::dangling()),
            cap,
            entities_bitset: [0u8; MAX_ENTITY_COUNT / 8],
            dirty_bitset: Rc::new(RefCell::new([0u8; MAX_ENTITY_COUNT / 8])),
            read_access_bitset: Rc::new(RefCell::new([0u8; MAX_ENTITY_COUNT / 8])),
            write_access_bitset: Rc::new(RefCell::new([0u8; MAX_ENTITY_COUNT / 8])),
            drop_fn,
        }
    }

    pub fn clear_dirty_bitset(&mut self) {
        self.dirty_bitset.borrow_mut().clear_bits();
    }

    pub fn set_dirty(&self, entity_id: EntityId) {
        self.dirty_bitset.borrow_mut().set_bit(entity_id);
    }

    pub fn dirty(&self, entity_id: EntityId) -> bool {
        self.dirty_bitset.borrow_mut().bit(entity_id)
    }

    pub fn store<C>(&mut self, entity_id: EntityId, mut component: C) {
        assert!(entity_id < MAX_ENTITY_COUNT, "The component store is full");
        self.entities_bitset.set_bit(entity_id);
        self.dirty_bitset.borrow_mut().set_bit(entity_id);
        self.ensure_capacity(entity_id + 1);

        if self.component_layout.size() > 0 {
            // SAFETY:
            // We ensured the capacity to be entity_id + 1 so we can write at
            // entity_id
            unsafe {
                self.write(entity_id, std::ptr::addr_of_mut!(component).cast());
            }
            std::mem::forget(component);
        }
    }

    pub fn delete(&mut self, entity_id: EntityId) {
        if entity_id >= self.cap || !self.entities_bitset.bit(entity_id) {
            return;
        }

        self.entities_bitset.unset_bit(entity_id);
        unsafe {
            (self.drop_fn)(self.ptr_at(entity_id));
        }
    }

    pub fn get<C>(&self, entity_id: EntityId) -> Option<ComponentRef<C>>
    where
        C: 'static,
    {
        if entity_id >= MAX_ENTITY_COUNT {
            return None;
        }

        if entity_id >= self.cap {
            return None;
        }

        if !self.entities_bitset.bit(entity_id) {
            return None;
        }

        assert!(
            !self.write_access_bitset.borrow().bit(entity_id),
            "Component {:?} of entity {entity_id} is already accessed mutably",
            TypeId::of::<C>()
        );

        self.read_access_bitset.borrow_mut().set_bit(entity_id);

        // SAFETY:
        // We checked that entity_id is smaller than self.cap, so it must be
        // in bound
        let ptr = unsafe { self.ptr_at(entity_id) };

        Some(ComponentRef {
            inner: ptr.cast::<C>(),
            access: self.read_access_bitset.clone(),
            entity_id,
        })
    }

    pub fn get_mut<C>(&self, entity_id: EntityId) -> Option<ComponentRefMut<C>>
    where
        C: 'static,
    {
        if entity_id >= MAX_ENTITY_COUNT {
            return None;
        }

        if entity_id >= self.cap {
            return None;
        }

        if !self.entities_bitset.bit(entity_id) {
            return None;
        }

        assert!(
            !self.write_access_bitset.borrow().bit(entity_id),
            "Component {:?} of entity {entity_id} is already accessed mutably",
            TypeId::of::<C>()
        );
        assert!(
            !self.read_access_bitset.borrow().bit(entity_id),
            "Component {:?} of entity {entity_id} is already accessed immutably",
            TypeId::of::<C>()
        );
        self.write_access_bitset.borrow_mut().set_bit(entity_id);

        // SAFETY:
        // We checked that entity_id is smaller than self.cap, so it must be
        // in bound
        let ptr = unsafe { self.ptr_at(entity_id) };

        Some(ComponentRefMut {
            inner: ptr.cast::<C>(),
            access: self.write_access_bitset.clone(),
            dirty: self.dirty_bitset.clone(),
            entity_id,
        })
    }

    /// # Safety
    /// The index must be in bound of the allocated data chunk
    unsafe fn write(&mut self, index: usize, data_ptr: *const u8) {
        let ptr_at = self.ptr_at(index);
        std::ptr::copy_nonoverlapping(data_ptr, ptr_at, self.component_layout.size());
    }

    /// # Safety
    /// The index must be in bound of the allocated data chunk
    unsafe fn ptr_at(&self, index: usize) -> *mut u8 {
        (*self.data.get())
            .as_ptr()
            .add(index * self.component_layout.size())
    }

    fn ensure_capacity(&mut self, capacity_to_ensure: usize) {
        if self.cap >= capacity_to_ensure {
            return;
        }

        let component_size = self.component_layout.size();
        let new_capacity = capacity_to_ensure;
        let array_size = new_capacity * component_size;
        assert_ne!(array_size, 0);

        let array_alignment = self.component_layout.align();
        assert!(array_size <= Self::max_size_for_align(array_alignment));

        let new_layout = Layout::from_size_align(array_size, array_alignment)
            .expect("Invalid layout when allocating component store data");

        let new_data = if self.cap == 0 {
            // SAFETY: We checked that the size of the array is non-zero
            unsafe { std::alloc::alloc(new_layout) }
        } else {
            let previous_array_size = self.cap * component_size;
            assert_ne!(previous_array_size, 0);

            let previous_layout = Layout::from_size_align(previous_array_size, array_alignment)
                .expect("Invalid layout when reallocating component store data");
            // SAFETY:
            // - self.data has been allocated with the same allocator
            // - previous_layout matches the layout used to create the array
            //   (using the old size)
            // - array_size is non-zero
            // - array_size when rounded up to the nearest multiple of the
            //   alignment, doesn't overflow isize
            unsafe {
                std::alloc::realloc(self.data.get_mut().as_ptr(), previous_layout, array_size)
            }
        };

        self.cap = new_capacity;
        self.data =
            UnsafeCell::new(NonNull::new(new_data).expect("ComponentStore data allocation failed"));
    }

    pub fn clear(&mut self) {
        for i in 0..self.cap {
            if self.entities_bitset.bit(i) {
                self.delete(i);
            }
        }
    }

    const fn max_size_for_align(align: usize) -> usize {
        isize::MAX as usize - (align - 1)
    }
}

impl Drop for ComponentStore {
    fn drop(&mut self) {
        if self.cap == 0 || self.component_layout.size() == 0 {
            return;
        }

        self.clear();
        let array_size = self.cap * self.component_layout.size();
        let layout = Layout::from_size_align(array_size, self.component_layout.align()).unwrap();

        // Safety
        // - The data was allocated with the same allocator
        // - The given layout is the same as that one that's been used to
        //   allocate the memory chunk
        unsafe {
            std::alloc::dealloc(self.data.get_mut().as_ptr(), layout);
        }
    }
}

pub unsafe fn drop_fn_of<T>(ptr: *mut u8) {
    ptr.cast::<T>().drop_in_place();
}

#[derive(Debug)]
pub struct ComponentRef<T> {
    inner: *const T,
    access: Rc<RefCell<EntityBitSet>>,
    entity_id: EntityId,
}

impl<T> Deref for ComponentRef<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        // SAFETY: If we have a component ref, it means we checked that there
        // was no mutable reference on that component
        unsafe { &*self.inner }
    }
}

impl<T> Drop for ComponentRef<T> {
    fn drop(&mut self) {
        self.access.borrow_mut().unset_bit(self.entity_id);
    }
}

#[derive(Debug)]
pub struct ComponentRefMut<T> {
    inner: *mut T,
    access: Rc<RefCell<EntityBitSet>>,
    dirty: Rc<RefCell<EntityBitSet>>,
    entity_id: EntityId,
}

impl<T> Deref for ComponentRefMut<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        // SAFETY: If we have a component ref, it means we checked that there
        // was no mutable reference on that component
        unsafe { &*self.inner }
    }
}

impl<T> DerefMut for ComponentRefMut<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.dirty.borrow_mut().set_bit(self.entity_id);
        unsafe { &mut *self.inner }
    }
}

impl<T> Drop for ComponentRefMut<T> {
    fn drop(&mut self) {
        self.access.borrow_mut().unset_bit(self.entity_id);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    struct Position {
        x: i32,
        y: i32,
    }

    #[test]
    fn component_store_store() {
        let mut store = ComponentStore::new(Layout::new::<Position>(), drop_fn_of::<Position>);
        store.store(5, Position { x: 23, y: 12 });
        assert_eq!(store.cap, 6);
        store.store(2, Position { x: 43, y: 45 });
        assert_eq!(store.cap, 6);
    }

    #[test]
    fn component_store_get() {
        let mut store = ComponentStore::new(Layout::new::<Position>(), drop_fn_of::<Position>);
        store.store(5, Position { x: 23, y: 12 });
        store.store(2, Position { x: 11, y: 33 });

        let position = store.get::<Position>(2).unwrap();
        assert_eq!(position.x, 11);
        assert_eq!(position.y, 33);

        let position = store.get::<Position>(5).unwrap();
        assert_eq!(position.x, 23);
        assert_eq!(position.y, 12);
    }

    #[test]
    fn component_store_get_mut() {
        let mut store = ComponentStore::new(Layout::new::<Position>(), drop_fn_of::<Position>);
        store.store(5, Position { x: 23, y: 12 });
        store.store(2, Position { x: 11, y: 33 });

        {
            let mut position = store.get_mut::<Position>(2).unwrap();
            position.x = 83;
            position.y = 92;
        }

        let position = store.get::<Position>(5).unwrap();
        assert_eq!(position.x, 23);
        assert_eq!(position.y, 12);

        let position = store.get::<Position>(2).unwrap();
        assert_eq!(position.x, 83);
        assert_eq!(position.y, 92);
    }
}
