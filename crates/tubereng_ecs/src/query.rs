use std::{any::TypeId, marker::PhantomData};

use crate::{
    component_store::{ComponentRef, ComponentRefMut},
    ComponentStores, EntityId,
};

pub struct State<'w, QD>
where
    QD: Definition,
{
    component_stores: &'w ComponentStores,
    deleted_entities: &'w [EntityId],
    max_entity_index: usize,
    _marker: PhantomData<QD>,
}

impl<'w, QD> State<'w, QD>
where
    QD: Definition,
{
    #[must_use]
    pub fn new(
        component_stores: &'w ComponentStores,
        deleted_entities: &'w [EntityId],
        max_entity_index: usize,
    ) -> Self {
        Self {
            component_stores,
            max_entity_index,
            _marker: PhantomData,
            deleted_entities,
        }
    }

    pub fn iter<'s>(&'s mut self) -> Iter<'w, 's, QD> {
        Iter::new(
            self,
            self.deleted_entities,
            self.max_entity_index,
            self.component_stores,
        )
    }

    pub fn iter_with_ids<'s>(&'s mut self) -> IterWithIds<'w, 's, QD> {
        IterWithIds::new(
            self,
            self.deleted_entities,
            self.max_entity_index,
            self.component_stores,
        )
    }
}

pub struct IterWithIds<'w, 's, QD>
where
    QD: Definition,
{
    _query_state: &'s State<'w, QD>,
    max_entity_index: usize,
    deleted_entities: &'w [EntityId],
    component_stores: &'w ComponentStores,
    current_entity_index: usize,
}

impl<'w, 's, QD> IterWithIds<'w, 's, QD>
where
    QD: Definition,
{
    #[must_use]
    pub fn new(
        query_state: &'s State<'w, QD>,
        deleted_entities: &'w [EntityId],
        max_entity_index: usize,
        component_stores: &'w ComponentStores,
    ) -> Self {
        Self {
            _query_state: query_state,
            max_entity_index,
            component_stores,
            current_entity_index: 0,
            deleted_entities,
        }
    }
}

impl<'w, 's, QD> Iterator for IterWithIds<'w, 's, QD>
where
    QD: Definition,
{
    type Item = (EntityId, QD::Item<'w>);

    fn next(&mut self) -> Option<Self::Item> {
        if self.current_entity_index > self.max_entity_index {
            return None;
        }

        let mut fetched = None;
        if !self.deleted_entities.contains(&self.current_entity_index) {
            fetched = QD::fetch(self.component_stores, self.current_entity_index);
        }

        while fetched.is_none() {
            self.current_entity_index += 1;
            if self.current_entity_index > self.max_entity_index {
                return None;
            }

            if self.deleted_entities.contains(&self.current_entity_index) {
                continue;
            }

            fetched = QD::fetch(self.component_stores, self.current_entity_index);
        }

        let entity_id = self.current_entity_index;
        self.current_entity_index += 1;
        Some((entity_id, fetched?))
    }
}

pub struct Iter<'w, 's, QD>
where
    QD: Definition,
{
    inner: IterWithIds<'w, 's, QD>,
}

impl<'w, 's, QD> Iter<'w, 's, QD>
where
    QD: Definition,
{
    #[must_use]
    pub fn new(
        query_state: &'s State<'w, QD>,
        deleted_entities: &'w [EntityId],
        entity_count: usize,
        component_stores: &'w ComponentStores,
    ) -> Self {
        Self {
            inner: IterWithIds::new(
                query_state,
                deleted_entities,
                entity_count,
                component_stores,
            ),
        }
    }
}

impl<'w, 's, QD> Iterator for Iter<'w, 's, QD>
where
    QD: Definition,
{
    type Item = QD::Item<'w>;

    fn next(&mut self) -> Option<Self::Item> {
        self.inner.next().map(|item| item.1)
    }
}

pub trait Definition {
    type Item<'a>;
    fn fetch(component_stores: &ComponentStores, entity_id: usize) -> Option<Self::Item<'_>>
    where
        Self: Sized;
}

macro_rules! impl_definition_for_tuples {
    ($head:tt, $($tail:tt,)*) => {
        impl<$head: Definition, $($tail: Definition,)*> Definition for ($head, $($tail,)*) {
            type Item<'a> = ($head::Item<'a>, $($tail::Item<'a>,)*);

            fn fetch(component_stores: &ComponentStores, entity_id: usize) -> Option<Self::Item<'_>> {
                Some((
                    $head::fetch(component_stores, entity_id)?,
                    $($tail::fetch(component_stores, entity_id)?,)*
                ))
            }
        }

        impl_definition_for_tuples!($($tail,)*);
    };
    () => {};
}

impl_definition_for_tuples!(A, B, C, D, E, F,);

pub struct DirtyState<C>(PhantomData<C>);
impl<C: 'static> Definition for DirtyState<C> {
    type Item<'a> = bool;

    fn fetch(component_stores: &ComponentStores, entity_id: usize) -> Option<Self::Item<'_>>
    where
        Self: Sized,
    {
        Some(component_stores.get(&TypeId::of::<C>())?.dirty(entity_id))
    }
}

impl<T: 'static> Definition for &T {
    type Item<'a> = ComponentRef<T>;

    fn fetch(component_stores: &ComponentStores, entity_id: usize) -> Option<Self::Item<'_>> {
        component_stores.get(&TypeId::of::<T>())?.get(entity_id)
    }
}

impl<T: 'static> Definition for &mut T {
    type Item<'a> = ComponentRefMut<T>;

    fn fetch(component_stores: &ComponentStores, entity_id: usize) -> Option<Self::Item<'_>> {
        component_stores.get(&TypeId::of::<T>())?.get_mut(entity_id)
    }
}

#[cfg(test)]
mod tests {
    use crate::Ecs;

    use super::*;

    #[derive(Debug)]
    struct Name(&'static str);

    #[test]
    fn set_component_dirty_flag() {
        let mut ecs = Ecs::new();
        let entity = ecs.insert((Name("Some name"),));

        ecs.storage.clear_dirty_flags();
        for (name, dirty) in ecs.query::<(&mut Name, DirtyState<Name>)>().iter() {
            assert_eq!("Some name", name.0);
            assert!(!dirty);
        }
        assert!(!ecs.storage.component_stores[&TypeId::of::<Name>()].dirty(entity));

        for (mut name, dirty) in ecs.query::<(&mut Name, DirtyState<Name>)>().iter() {
            name.0 = "Some other name";
            assert!(!dirty);
        }

        assert!(ecs.query::<DirtyState<Name>>().iter().next().unwrap());
        assert!(ecs.storage.component_stores[&TypeId::of::<Name>()].dirty(entity));
    }
}
