use std::{cell::RefCell, vec::IntoIter};

use crate::{
    system::{self, System},
    Ecs, EntityDefinition,
};

pub struct CommandQueue(RefCell<Vec<Box<dyn Command>>>);
impl CommandQueue {
    #[must_use]
    pub fn new() -> Self {
        Self(RefCell::new(vec![]))
    }

    pub fn insert<ED>(&self, entity_definition: ED)
    where
        ED: 'static + EntityDefinition,
    {
        self.push_command(InsertEntity::new(entity_definition));
    }

    pub fn register_system<F, A>(&self, system: F)
    where
        F: system::Into<A>,
    {
        self.push_command(RegisterSystem::new(system));
    }

    fn push_command<C>(&self, command: C)
    where
        C: 'static + Command,
    {
        self.0.borrow_mut().push(Box::new(command));
    }
}

impl Default for CommandQueue {
    fn default() -> Self {
        Self::new()
    }
}

impl IntoIterator for CommandQueue {
    type Item = Box<dyn Command>;
    type IntoIter = IntoIter<Box<dyn Command>>;

    fn into_iter(self) -> Self::IntoIter {
        self.0.into_inner().into_iter()
    }
}

pub trait Command {
    fn apply(&mut self, ecs: &mut Ecs);
}

pub struct InsertEntity {
    entity_definition: Option<Box<dyn EntityDefinition>>,
}

pub struct RegisterSystem {
    system: Option<System>,
}

impl RegisterSystem {
    pub fn new<F, A>(system: F) -> Self
    where
        F: system::Into<A>,
    {
        Self {
            system: Some(system.into_system()),
        }
    }
}

impl Command for RegisterSystem {
    fn apply(&mut self, ecs: &mut Ecs) {
        ecs.insert_system(self.system.take().unwrap());
    }
}

impl InsertEntity {
    pub fn new<ED>(entity_definition: ED) -> Self
    where
        ED: 'static + EntityDefinition,
    {
        Self {
            entity_definition: Some(Box::new(entity_definition)),
        }
    }
}

impl Command for InsertEntity {
    fn apply(&mut self, ecs: &mut Ecs) {
        let boxed_ed = self.entity_definition.take().unwrap();
        ecs.insert(boxed_ed);
    }
}