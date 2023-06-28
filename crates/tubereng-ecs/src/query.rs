use std::{
    cell::{Ref, RefMut},
    marker::PhantomData,
};

use crate::entity::EntityStore;

pub struct Q<'q, QD>
where
    QD: Query<'q>,
{
    entity_store: &'q EntityStore,
    _marker: PhantomData<QD>,
}

impl<'q, QD> Q<'q, QD>
where
    QD: Query<'q>,
{
    #[must_use]
    pub fn new(entity_store: &'q EntityStore) -> Self {
        Self {
            entity_store,
            _marker: PhantomData,
        }
    }

    #[must_use]
    pub fn iter(&'q self) -> Iter<QD> {
        Iter {
            current_index: 0,
            ecs: self.entity_store,
            _marker: PhantomData,
        }
    }
}

pub struct Iter<'q, QD>
where
    QD: Query<'q>,
{
    current_index: usize,
    ecs: &'q EntityStore,
    _marker: PhantomData<&'q QD>,
}

impl<'q, QD> Iterator for Iter<'q, QD>
where
    QD: Query<'q>,
{
    type Item = QD::ResultType;

    fn next(&mut self) -> Option<Self::Item> {
        if self.current_index >= self.ecs.entity_count() {
            return None;
        }

        let mut result = QD::fetch(self.ecs, self.current_index);
        while result.is_none() {
            self.current_index += 1;
            if self.current_index >= self.ecs.entity_count() {
                return None;
            }

            result = QD::fetch(self.ecs, self.current_index);
        }

        self.current_index += 1;
        result
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
impl<'a, T: 'static> Argument<'a> for &T {
    type Type = Ref<'a, T>;
    fn fetch(entity_store: &'a EntityStore, index: usize) -> Option<Self::Type> {
        entity_store.query_component(index)
    }
}

impl<'a, T: 'static> Argument<'a> for &mut T {
    type Type = RefMut<'a, T>;
    fn fetch(entity_store: &'a EntityStore, index: usize) -> Option<Self::Type> {
        entity_store.query_component_mut(index)
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

        let query = Q::<(&Player, &Health)>::new(&ecs.entity_store);
        assert_eq!(query.iter().count(), 2);
        for (_player, health) in query.iter() {
            assert!(health.0 >= 8);
        }
    }

    #[test]
    fn query_mut() {
        let mut ecs = Ecs::new();
        ecs.insert((Player, Health(10)));
        ecs.insert((Player, Health(8)));

        let query_mutate_health = Q::<(&Player, &mut Health)>::new(&ecs.entity_store);
        assert_eq!(query_mutate_health.iter().count(), 2);
        for (_player, mut health) in query_mutate_health.iter() {
            health.0 = 0;
        }

        let query_health = Q::<(&Player, &Health)>::new(&ecs.entity_store);
        assert_eq!(query_health.iter().count(), 2);
        for (_player, health) in query_health.iter() {
            assert_eq!(health.0, 0);
        }
    }
}
