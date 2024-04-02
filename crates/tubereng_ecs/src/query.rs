use std::{any::TypeId, cell::RefCell, collections::HashSet, marker::PhantomData};

use crate::ComponentStores;

pub struct ComponentAccesses {
    // TODO: Consider using bitsets instead of HashSet, but we would need to
    // have ecs-managed ComponentIds instead of TypeIds
    read: RefCell<HashSet<TypeId>>,
    write: RefCell<HashSet<TypeId>>,
}

impl Default for ComponentAccesses {
    fn default() -> Self {
        Self::new()
    }
}

impl ComponentAccesses {
    #[must_use]
    pub fn new() -> Self {
        Self {
            read: RefCell::new(HashSet::new()),
            write: RefCell::new(HashSet::new()),
        }
    }
}

pub struct State<'w, QD>
where
    QD: Definition,
{
    component_stores: &'w ComponentStores,
    entity_count: usize,
    _accesses: ComponentAccesses,
    _marker: PhantomData<QD>,
}

impl<'w, QD> State<'w, QD>
where
    QD: Definition,
{
    #[must_use]
    pub fn new(component_stores: &'w ComponentStores, entity_count: usize) -> Self {
        let accesses = ComponentAccesses::new();
        QD::register_component_accesses(&accesses);
        Self {
            component_stores,
            entity_count,
            _accesses: accesses,
            _marker: PhantomData,
        }
    }

    pub fn iter<'s>(&'s mut self) -> Iter<'w, 's, QD> {
        Iter::new(self, self.entity_count, self.component_stores)
    }
}

pub struct Iter<'w, 's, QD>
where
    QD: Definition,
{
    _query_state: &'s State<'w, QD>,
    entity_count: usize,
    component_stores: &'w ComponentStores,
    current_entity_index: usize,
}

impl<'w, 's, QD> Iter<'w, 's, QD>
where
    QD: Definition,
{
    #[must_use]
    pub fn new(
        query_state: &'s State<'w, QD>,
        entity_count: usize,
        component_stores: &'w ComponentStores,
    ) -> Self {
        Self {
            _query_state: query_state,
            entity_count,
            component_stores,
            current_entity_index: 0,
        }
    }
}

impl<'w, 's, QD> Iterator for Iter<'w, 's, QD>
where
    QD: Definition,
{
    type Item = QD::Item<'w>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.current_entity_index >= self.entity_count {
            return None;
        }

        let mut fetched = QD::fetch(self.component_stores, self.current_entity_index);
        while fetched.is_none() {
            self.current_entity_index += 1;
            if self.current_entity_index >= self.entity_count {
                return None;
            }

            fetched = QD::fetch(self.component_stores, self.current_entity_index);
        }

        self.current_entity_index += 1;
        fetched
    }
}

pub trait Definition {
    type Item<'a>;
    fn register_component_accesses(accesses: &ComponentAccesses);

    fn fetch(component_stores: &ComponentStores, entity_id: usize) -> Option<Self::Item<'_>>
    where
        Self: Sized;
}

impl<A: Definition, B: Definition> Definition for (A, B) {
    type Item<'a> = (A::Item<'a>, B::Item<'a>);

    fn register_component_accesses(accesses: &ComponentAccesses) {
        A::register_component_accesses(accesses);
        B::register_component_accesses(accesses);
    }

    fn fetch(component_stores: &ComponentStores, entity_id: usize) -> Option<Self::Item<'_>> {
        Some((
            A::fetch(component_stores, entity_id)?,
            B::fetch(component_stores, entity_id)?,
        ))
    }
}

impl<T: 'static> Definition for &T {
    type Item<'a> = &'a T;
    fn register_component_accesses(accesses: &ComponentAccesses) {
        let component_type_id = TypeId::of::<T>();
        assert!(
            !accesses.write.borrow().contains(&component_type_id),
            "Component already has write access"
        );

        accesses.read.borrow_mut().insert(component_type_id);
    }
    fn fetch(component_stores: &ComponentStores, entity_id: usize) -> Option<Self::Item<'_>> {
        component_stores.get(&TypeId::of::<T>())?.get(entity_id)
    }
}

impl<T: 'static> Definition for &mut T {
    type Item<'a> = &'a mut T;
    fn register_component_accesses(accesses: &ComponentAccesses) {
        let component_type_id = TypeId::of::<T>();
        assert!(
            !accesses.read.borrow().contains(&component_type_id),
            "Component already has read access"
        );
        assert!(
            !accesses.write.borrow().contains(&component_type_id),
            "Component already has write access"
        );

        accesses.write.borrow_mut().insert(component_type_id);
    }
    fn fetch(component_stores: &ComponentStores, entity_id: usize) -> Option<Self::Item<'_>> {
        component_stores.get(&TypeId::of::<T>())?.get_mut(entity_id)
    }
}
