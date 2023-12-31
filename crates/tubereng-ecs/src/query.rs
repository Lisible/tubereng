use std::marker::PhantomData;

use parking_lot::{MappedRwLockReadGuard, MappedRwLockWriteGuard};

use crate::{
    entity::EntityStore,
    relationship::{Relationship, RelationshipId, RelationshipStore},
    EntityId,
};

pub type ComponentRef<'a, T> = MappedRwLockReadGuard<'a, T>;
pub type ComponentRefMut<'a, T> = MappedRwLockWriteGuard<'a, T>;

pub struct Q<'q, QD>
where
    QD: Query<'q>,
{
    entity_store: &'q EntityStore,
    relationship_store: &'q RelationshipStore,
    relationship_filters: Vec<Box<dyn RelationshipFilter>>,
    _marker: PhantomData<QD>,
}

trait RelationshipFilter {
    fn is_valid(&self, relationship_store: &RelationshipStore, index: EntityId) -> bool;
}

struct WithRelationshipFilter {
    relationship_id: RelationshipId,
    target_entity: EntityId,
}

impl RelationshipFilter for WithRelationshipFilter {
    fn is_valid(&self, relationship_store: &RelationshipStore, index: EntityId) -> bool {
        relationship_store.has(self.relationship_id, index, self.target_entity)
    }
}

impl<'q, QD> Q<'q, QD>
where
    QD: Query<'q>,
{
    #[must_use]
    pub fn new(entity_store: &'q EntityStore, relationship_store: &'q RelationshipStore) -> Self {
        Self {
            entity_store,
            relationship_store,
            relationship_filters: vec![],
            _marker: PhantomData,
        }
    }

    #[must_use]
    pub fn with_id(self, entity_id: EntityId) -> Option<QD::ResultType> {
        if self
            .relationship_filters
            .iter()
            .any(|filter| !filter.is_valid(self.relationship_store, entity_id))
        {
            return None;
        }

        QD::fetch(self.entity_store, entity_id)
    }

    #[must_use]
    pub fn with_relationship<R>(mut self, entity_id: EntityId) -> Self
    where
        R: 'static + Relationship,
    {
        self.relationship_filters
            .push(Box::new(WithRelationshipFilter {
                relationship_id: R::relationship_id(),
                target_entity: entity_id,
            }));
        self
    }

    #[must_use]
    pub fn iter_with_indices(self) -> IterWithIndices<'q, QD> {
        IterWithIndices {
            current_index: 0,
            relationship_filters: self.relationship_filters,
            entity_store: self.entity_store,
            _marker: PhantomData,
            relationship_store: self.relationship_store,
        }
    }

    #[must_use]
    pub fn iter(self) -> Iter<'q, QD> {
        Iter(IterWithIndices {
            current_index: 0,
            relationship_filters: self.relationship_filters,
            entity_store: self.entity_store,
            _marker: PhantomData,
            relationship_store: self.relationship_store,
        })
    }

    pub fn for_each_parallel<'a, F>(self, chunk_size: usize, f: F)
    where
        F: Fn(QD::ResultType) + Send + Sync + 'a,
    {
        let entity_ids = (0..self.entity_store.entity_count()).collect::<Vec<usize>>();
        let entity_chunks = entity_ids.chunks(chunk_size);
        std::thread::scope(|s| {
            let f = &f;
            for chunk in entity_chunks {
                s.spawn(move || {
                    for e in chunk {
                        match QD::fetch(self.entity_store, *e) {
                            Some(result) => f(result),
                            None => continue,
                        }
                    }
                });
            }
        });
    }
}

pub struct IterWithIndices<'q, QD>
where
    QD: Query<'q>,
{
    current_index: usize,
    entity_store: &'q EntityStore,
    relationship_store: &'q RelationshipStore,
    _marker: PhantomData<&'q QD>,
    relationship_filters: Vec<Box<dyn RelationshipFilter>>,
}

impl<'q, QD> Iterator for IterWithIndices<'q, QD>
where
    QD: Query<'q>,
{
    type Item = (EntityId, QD::ResultType);

    fn next(&mut self) -> Option<Self::Item> {
        if self.current_index >= self.entity_store.entity_count() {
            return None;
        }

        let mut result = QD::fetch(self.entity_store, self.current_index);
        while result.is_none() {
            self.current_index += 1;
            if self.current_index >= self.entity_store.entity_count() {
                return None;
            }

            if !self
                .relationship_filters
                .iter()
                .all(|filter| filter.is_valid(self.relationship_store, self.current_index))
            {
                continue;
            }

            result = QD::fetch(self.entity_store, self.current_index);
        }

        let index = self.current_index;
        self.current_index += 1;
        Some((index, result.unwrap()))
    }
}

pub struct Iter<'q, QD>(IterWithIndices<'q, QD>)
where
    QD: Query<'q>;

impl<'q, QD> Iterator for Iter<'q, QD>
where
    QD: Query<'q>,
{
    type Item = QD::ResultType;

    fn next(&mut self) -> Option<Self::Item> {
        self.0.next().map(|res| res.1)
    }
}

pub trait Query<'q> {
    type ResultType: 'q;

    fn fetch(entity_store: &'q EntityStore, index: usize) -> Option<Self::ResultType>;
}

macro_rules! impl_query_for_tuples {
    ($head:ident, $($tail:ident,)*) => {
        impl<'q, $head, $($tail,)*> Query<'q> for ($head, $($tail,)*)
        where
            $head: Argument<'q>,
            $($tail: Argument<'q>,)*
        {
            type ResultType = ($head::Type, $($tail::Type,)*);

            fn fetch(entity_store: &'q EntityStore, index: usize) -> Option<Self::ResultType> {
                Some(($head::fetch(entity_store, index)?, $($tail::fetch(entity_store, index)?,)*))
            }
        }

        impl_query_for_tuples!($($tail,)*);
    };

    () => {};
}

impl_query_for_tuples!(A, B, C, D, E,);

pub trait Argument<'a>: Sized {
    type Type: 'a;
    fn fetch(entity_store: &'a EntityStore, index: usize) -> Option<Self::Type>;
}

impl<'a, T: 'static> Argument<'a> for &T
where
    T: Send,
{
    type Type = ComponentRef<'a, T>;
    fn fetch(entity_store: &'a EntityStore, index: usize) -> Option<Self::Type> {
        entity_store.query_component(index)
    }
}

impl<'a, T: 'static> Argument<'a> for &mut T
where
    T: Send,
{
    type Type = ComponentRefMut<'a, T>;
    fn fetch(entity_store: &'a EntityStore, index: usize) -> Option<Self::Type> {
        entity_store.query_component_mut(index)
    }
}

impl<'a, T: 'static> Argument<'a> for Option<&T>
where
    T: Send,
{
    type Type = Option<ComponentRef<'a, T>>;

    fn fetch(entity_store: &'a EntityStore, index: usize) -> Option<Self::Type> {
        match entity_store.query_component(index) {
            Some(component) => Some(Some(component)),
            None => Some(None),
        }
    }
}

impl<'a, T: 'static> Argument<'a> for Option<&mut T>
where
    T: Send,
{
    type Type = Option<ComponentRefMut<'a, T>>;
    fn fetch(entity_store: &'a EntityStore, index: usize) -> Option<Self::Type> {
        match entity_store.query_component_mut(index) {
            Some(component) => Some(Some(component)),
            None => Some(None),
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::Ecs;

    use super::*;

    #[derive(Debug)]
    struct Player;
    #[derive(Debug)]
    struct Enemy;
    #[derive(Debug)]
    struct Health(i32);

    #[test]
    fn query() {
        let mut ecs = Ecs::new();
        ecs.insert((Player, Health(10)));
        ecs.insert((Player, Health(8)));

        let query = Q::<(&Player, &Health)>::new(&ecs.entity_store, &ecs.relationship_store);
        assert_eq!(query.iter().count(), 2);
        let query = Q::<(&Player, &Health)>::new(&ecs.entity_store, &ecs.relationship_store);
        for (_player, health) in query.iter() {
            assert!(health.0 >= 8);
        }
    }

    #[test]
    fn query_optional() {
        let mut ecs = Ecs::new();
        ecs.insert((Player, Health(10)));
        ecs.insert((Health(8),));

        let query =
            Q::<(Option<&Player>, &Health)>::new(&ecs.entity_store, &ecs.relationship_store);
        assert_eq!(query.iter().count(), 2);
    }

    #[test]
    fn query_optional_mut() {
        let mut ecs = Ecs::new();
        ecs.insert((Player, Health(10)));
        ecs.insert((Health(8),));

        let query =
            Q::<(Option<&mut Player>, &Health)>::new(&ecs.entity_store, &ecs.relationship_store);
        assert_eq!(query.iter().count(), 2);
    }

    #[test]
    fn query_mut() {
        let mut ecs = Ecs::new();
        ecs.insert((Player, Health(10)));
        ecs.insert((Player, Health(8)));

        let query_mutate_health =
            Q::<(&Player, &mut Health)>::new(&ecs.entity_store, &ecs.relationship_store);
        assert_eq!(query_mutate_health.iter().count(), 2);
        let query_mutate_health =
            Q::<(&Player, &mut Health)>::new(&ecs.entity_store, &ecs.relationship_store);
        for (_player, mut health) in query_mutate_health.iter() {
            health.0 = 0;
        }

        let query_health = Q::<(&Player, &Health)>::new(&ecs.entity_store, &ecs.relationship_store);
        assert_eq!(query_health.iter().count(), 2);
        let query_health = Q::<(&Player, &Health)>::new(&ecs.entity_store, &ecs.relationship_store);
        for (_player, health) in query_health.iter() {
            assert_eq!(health.0, 0);
        }
    }
}
