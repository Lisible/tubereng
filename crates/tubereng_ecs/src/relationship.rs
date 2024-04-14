use std::{
    any::TypeId,
    collections::{HashMap, HashSet},
};

use crate::EntityId;

pub struct ChildOf;

pub(crate) struct Relationships {
    relationships: HashMap<TypeId, Relationship>,
}

impl Relationships {
    pub fn new() -> Self {
        Self {
            relationships: HashMap::new(),
        }
    }

    pub fn insert<R: 'static>(&mut self, source: EntityId, target: EntityId) {
        let relationship = self.relationships.entry(TypeId::of::<R>()).or_default();
        relationship.add(source, target);
    }

    pub fn get<R: 'static>(&self) -> Option<&Relationship> {
        self.relationships.get(&TypeId::of::<R>())
    }
}

#[derive(Default)]
pub struct Relationship {
    sources_for_entity: HashMap<EntityId, HashSet<EntityId>>,
    targets_for_entity: HashMap<EntityId, HashSet<EntityId>>,
}

impl Relationship {
    pub fn add(&mut self, source: EntityId, target: EntityId) {
        self.sources_for_entity
            .entry(target)
            .or_default()
            .insert(source);
        self.targets_for_entity
            .entry(source)
            .or_default()
            .insert(target);
    }

    #[must_use]
    pub fn sources(&self, target: EntityId) -> Option<&HashSet<EntityId>> {
        self.sources_for_entity.get(&target)
    }

    #[must_use]
    pub fn targets(&self, source: EntityId) -> Option<&HashSet<EntityId>> {
        self.targets_for_entity.get(&source)
    }

    #[must_use]
    pub fn ancestors(&self, target: EntityId) -> Vec<EntityId> {
        let mut ancestors = vec![];
        let mut to_process = vec![target];
        while let Some(current_entity) = to_process.pop() {
            if let Some(anc) = self.sources_for_entity.get(&current_entity) {
                ancestors.extend(anc.iter());
                to_process.extend(anc.iter());
            }
        }

        ancestors
    }

    #[must_use]
    pub fn successors(&self, source: EntityId) -> Vec<EntityId> {
        let mut successors = vec![];
        let mut to_process = vec![source];
        while let Some(current_entity) = to_process.pop() {
            if let Some(suc) = self.targets_for_entity.get(&current_entity) {
                successors.extend(suc.iter());
                to_process.extend(suc.iter());
            }
        }

        successors
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ancestors() {
        let mut relationship = Relationship::default();
        relationship.add(4, 3);
        relationship.add(3, 2);
        relationship.add(2, 1);
        relationship.add(1, 0);

        let ancestors = relationship.ancestors(0);
        assert!(&ancestors.contains(&1));
        assert!(&ancestors.contains(&2));
        assert!(&ancestors.contains(&3));
        assert!(&ancestors.contains(&4));
    }

    #[test]
    fn successors() {
        let mut relationship = Relationship::default();
        relationship.add(4, 3);
        relationship.add(3, 2);
        relationship.add(2, 1);
        relationship.add(1, 0);

        let successors = relationship.successors(4);
        assert!(successors.contains(&0));
        assert!(successors.contains(&1));
        assert!(successors.contains(&2));
        assert!(successors.contains(&3));
    }

    #[test]
    fn successors_tree() {
        let mut relationship = Relationship::default();
        relationship.add(4, 3);
        relationship.add(3, 2);
        relationship.add(2, 1);
        relationship.add(2, 5);
        relationship.add(5, 6);
        relationship.add(1, 0);

        let successors = relationship.successors(4);
        assert!(successors.contains(&0));
        assert!(successors.contains(&1));
        assert!(successors.contains(&2));
        assert!(successors.contains(&3));
        assert!(successors.contains(&5));
        assert!(successors.contains(&6));
    }
}
