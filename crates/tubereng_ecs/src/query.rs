use std::{any::TypeId, cell::RefCell, collections::HashSet, marker::PhantomData};

use crate::Ecs;

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

pub struct State<QD>
where
    QD: Definition,
{
    _accesses: ComponentAccesses,
    _marker: PhantomData<QD>,
}

impl<QD> State<QD>
where
    QD: Definition,
{
    #[must_use]
    pub fn new() -> Self {
        let accesses = ComponentAccesses::new();
        QD::register_component_accesses(&accesses);
        Self {
            _accesses: accesses,
            _marker: PhantomData,
        }
    }

    pub fn iter<'s, 'w>(&'s mut self, ecs: &'w Ecs) -> Iter<'w, 's, QD> {
        Iter::new(self, ecs)
    }
}

impl<QD> Default for State<QD>
where
    QD: Definition,
{
    fn default() -> Self {
        Self::new()
    }
}

pub struct Iter<'w, 's, QD>
where
    QD: Definition,
{
    _query_state: &'s State<QD>,
    ecs: &'w Ecs,
    current_entity_index: usize,
}

impl<'w, 's, QD> Iter<'w, 's, QD>
where
    QD: Definition,
{
    #[must_use]
    pub fn new(query_state: &'s State<QD>, ecs: &'w Ecs) -> Self {
        Self {
            _query_state: query_state,
            ecs,
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
        if self.current_entity_index >= self.ecs.entity_count() {
            return None;
        }

        let mut fetched = QD::fetch(self.ecs, self.current_entity_index);
        while fetched.is_none() {
            self.current_entity_index += 1;
            if self.current_entity_index >= self.ecs.entity_count() {
                return None;
            }

            fetched = QD::fetch(self.ecs, self.current_entity_index);
        }

        self.current_entity_index += 1;
        fetched
    }
}

pub trait Definition {
    type Item<'a>;
    fn register_component_accesses(accesses: &ComponentAccesses);

    fn fetch(ecs: &Ecs, entity_id: usize) -> Option<Self::Item<'_>>
    where
        Self: Sized;
}

impl<A: Definition, B: Definition> Definition for (A, B) {
    type Item<'a> = (A::Item<'a>, B::Item<'a>);

    fn register_component_accesses(accesses: &ComponentAccesses) {
        A::register_component_accesses(accesses);
        B::register_component_accesses(accesses);
    }

    fn fetch(ecs: &Ecs, entity_id: usize) -> Option<Self::Item<'_>> {
        Some((A::fetch(ecs, entity_id)?, B::fetch(ecs, entity_id)?))
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
    fn fetch(ecs: &Ecs, entity_id: usize) -> Option<Self::Item<'_>> {
        ecs.component::<T>(entity_id)
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
    fn fetch(ecs: &Ecs, entity_id: usize) -> Option<Self::Item<'_>> {
        ecs.component_mut::<T>(entity_id)
    }
}
