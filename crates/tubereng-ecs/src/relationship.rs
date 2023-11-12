use std::{
    any::TypeId,
    collections::{HashMap, HashSet},
};

use crate::entity::EntityId;

/// The `ChildOf` relationship
pub struct ChildOf;

pub struct RelationshipStore {
    relationships: HashMap<RelationshipId, RelationshipData>,
}

impl Default for RelationshipStore {
    fn default() -> Self {
        Self::new()
    }
}

impl RelationshipStore {
    #[must_use]
    pub fn new() -> Self {
        Self {
            relationships: HashMap::new(),
        }
    }

    pub fn insert(&mut self, relationship_id: RelationshipId, source: EntityId, target: EntityId) {
        let relationship_data = self.relationships.entry(relationship_id).or_default();

        relationship_data.insert_source(target, source);
    }

    #[must_use]
    pub fn has(&self, relationship_id: RelationshipId, source: EntityId, target: EntityId) -> bool {
        let Some(relationship) = self.relationships.get(&relationship_id) else {
            return false;
        };

        relationship.has_source(target, source)
    }

    #[must_use]
    pub fn all_sources_of<R>(&self) -> HashSet<EntityId>
    where
        R: Relationship,
    {
        let Some(relationship) = self.relationships.get(&R::relationship_id()) else {
            return HashSet::new();
        };

        relationship
            .targets_by_source
            .keys()
            .copied()
            .collect::<HashSet<_>>()
    }

    #[must_use]
    pub fn sources_of<R>(&self, target: EntityId) -> Option<&HashSet<EntityId>>
    where
        R: Relationship,
    {
        let relationship = self.relationships.get(&R::relationship_id())?;
        relationship.sources_by_target.get(&target)
    }
    #[must_use]
    pub fn targets_of<R>(&self, source: EntityId) -> Vec<EntityId>
    where
        R: Relationship,
    {
        let Some(relationship) = self.relationships.get(&R::relationship_id()) else {
            return vec![];
        };

        relationship.targets_by_source[&source]
            .iter()
            .copied()
            .collect()
    }
}

pub struct RelationshipData {
    sources_by_target: HashMap<EntityId, HashSet<EntityId>>,
    targets_by_source: HashMap<EntityId, HashSet<EntityId>>,
}

impl RelationshipData {
    #[must_use]
    pub fn new() -> Self {
        Self {
            sources_by_target: HashMap::new(),
            targets_by_source: HashMap::new(),
        }
    }

    fn has_source(&self, target: EntityId, source: EntityId) -> bool {
        let Some(sources) = self.sources_by_target.get(&target) else {
            return false;
        };

        sources.contains(&source)
    }

    pub fn insert_source(&mut self, target: EntityId, source: EntityId) {
        self.sources_by_target
            .entry(target)
            .or_default()
            .insert(source);
        self.targets_by_source
            .entry(source)
            .or_default()
            .insert(target);
    }
}

impl Default for RelationshipData {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Clone, Copy, Hash, PartialEq, Eq)]
pub struct RelationshipId(pub(crate) TypeId);

pub trait Relationship: 'static {
    #[must_use]
    fn relationship_id() -> RelationshipId {
        RelationshipId(TypeId::of::<Self>())
    }
}

impl<T> Relationship for T where T: 'static {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn add_relationship() {
        let mut relationship_store = RelationshipStore::new();
        relationship_store.insert(ChildOf::relationship_id(), 1, 0);
        assert!(relationship_store.has(ChildOf::relationship_id(), 1, 0));
        let children_of_0 = relationship_store.sources_of::<ChildOf>(0).unwrap();
        assert_eq!(children_of_0.len(), 1);
        assert_eq!(*children_of_0.iter().next().unwrap(), 1);
        let all_children_of_any = relationship_store.all_sources_of::<ChildOf>();
        assert_eq!(all_children_of_any.len(), 1);
        assert_eq!(*all_children_of_any.iter().next().unwrap(), 1);
    }

    #[test]
    fn add_relationships() {
        let mut relationship_store = RelationshipStore::new();
        relationship_store.insert(ChildOf::relationship_id(), 1, 0);
        relationship_store.insert(ChildOf::relationship_id(), 2, 1);
        assert!(relationship_store.has(ChildOf::relationship_id(), 1, 0));
        let children_of_0 = relationship_store.sources_of::<ChildOf>(0).unwrap();
        assert_eq!(children_of_0.len(), 1);
        assert_eq!(*children_of_0.iter().next().unwrap(), 1);
        let all_children_of_any = relationship_store.all_sources_of::<ChildOf>();
        assert_eq!(all_children_of_any.len(), 2);
    }
}
