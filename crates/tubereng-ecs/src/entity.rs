use crate::relationship::Relationship;
use std::{
    any::{Any, TypeId},
    cell::{Ref, RefCell, RefMut},
    collections::{HashMap, HashSet},
    rc::Rc,
};

use log::trace;

use crate::{relationship::RelationshipId, EntityDefinition};

pub type EntityId = usize;

type ComponentStore = Vec<Option<Rc<RefCell<dyn Any>>>>;

pub struct EntityBundle {
    pub(crate) entities: Vec<Box<dyn EntityDefinition>>,
    pub(crate) relationships: Vec<HashMap<RelationshipId, Vec<usize>>>,
}

impl EntityBundle {
    #[must_use]
    pub fn new() -> Self {
        Self {
            entities: vec![],
            relationships: vec![],
        }
    }

    pub fn add_entity<ED>(&mut self, entity_definition: ED) -> usize
    where
        ED: 'static + EntityDefinition,
    {
        self.entities.push(Box::new(entity_definition));
        self.relationships.push(HashMap::new());
        self.entities.len() - 1
    }

    pub fn add_relationship<R>(&mut self, source: usize, target: usize)
    where
        R: 'static,
    {
        self.relationships[source]
            .entry(R::relationship_id())
            .or_insert_with(Vec::new)
            .push(target);
    }
}

impl Default for EntityBundle {
    fn default() -> Self {
        Self::new()
    }
}

pub struct EntityStore {
    components: HashMap<TypeId, ComponentStore>,
    next_entity_id: EntityId,
}

impl EntityStore {
    #[must_use]
    pub fn new() -> Self {
        Self {
            components: HashMap::new(),
            next_entity_id: 0,
        }
    }

    #[must_use]
    pub fn entity_count(&self) -> usize {
        self.next_entity_id
    }

    #[must_use]
    pub fn entity_ids(&self) -> HashSet<EntityId> {
        (0..self.next_entity_id).collect()
    }

    fn allocate_entity(&mut self) -> EntityId {
        let id = self.next_entity_id;
        self.next_entity_id += 1;
        trace!("Allocated entity {}", id);
        id
    }

    pub fn insert<ED>(&mut self, entity: ED) -> EntityId
    where
        ED: EntityDefinition,
    {
        trace!("Inserting entity {:?}", &entity);
        let entity_id = self.allocate_entity();
        Box::new(entity).write_into_entity_store(self, entity_id);
        entity_id
    }

    pub(crate) fn write_component<C: 'static>(&mut self, entity_id: EntityId, component: C) {
        assert!(
            entity_id < self.next_entity_id,
            "Tried to write a component in a unallocated entity"
        );

        let component_store = self
            .components
            .entry(TypeId::of::<C>())
            .or_insert_with(Vec::new);

        component_store.resize_with(self.next_entity_id, || None);
        component_store[entity_id] = Some(Rc::new(RefCell::new(component)));
    }

    #[must_use]
    pub(crate) fn query_component<T: 'static>(&self, index: usize) -> Option<Ref<T>> {
        Some(Ref::map(
            self.components
                .get(&TypeId::of::<T>())?
                .get(index)?
                .as_ref()?
                .borrow(),
            |r| r.downcast_ref().unwrap(),
        ))
    }

    #[must_use]
    pub(crate) fn query_component_mut<T: 'static>(&self, index: usize) -> Option<RefMut<T>> {
        Some(RefMut::map(
            self.components
                .get(&TypeId::of::<T>())?
                .get(index)?
                .as_ref()?
                .borrow_mut(),
            |r| r.downcast_mut().unwrap(),
        ))
    }
}

impl Default for EntityStore {
    fn default() -> Self {
        Self::new()
    }
}
