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

    pub fn insert<R>(&mut self, source: EntityId, target: EntityId)
    where
        R: Relationship,
    {
        let relationship_data = self
            .relationships
            .entry(R::relationship_id())
            .or_insert_with(RelationshipData::new);

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
            return HashSet::new()
        };

        relationship
            .sources_by_target
            .values()
            .flat_map(|v| v.iter().copied())
            .clone()
            .collect::<HashSet<_>>()
    }

    #[must_use]
    pub fn sources_of<R>(&self, target: EntityId) -> Vec<EntityId>
    where
        R: Relationship,
    {
        let Some(relationship) = self.relationships.get(&R::relationship_id()) else {
            return vec![];
        };

        relationship
            .sources_by_target
            .get(&target)
            .unwrap_or(&HashSet::new())
            .iter()
            .copied()
            .collect::<Vec<_>>()
    }
}

pub struct RelationshipData {
    sources_by_target: HashMap<EntityId, HashSet<EntityId>>,
}

impl RelationshipData {
    #[must_use]
    pub fn new() -> Self {
        Self {
            sources_by_target: HashMap::new(),
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
            .or_insert_with(HashSet::new)
            .insert(source);
    }
}

impl Default for RelationshipData {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Clone, Copy, Hash, PartialEq, Eq)]
pub struct RelationshipId(TypeId);

pub trait Relationship: 'static {
    #[must_use]
    fn relationship_id() -> RelationshipId {
        RelationshipId(TypeId::of::<Self>())
    }
}

impl<T> Relationship for T where T: 'static {}
