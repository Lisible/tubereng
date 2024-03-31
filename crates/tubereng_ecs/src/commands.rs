use std::{cell::RefCell, vec::IntoIter};

use crate::{Ecs, EntityDefinition};

pub struct CommandQueue(RefCell<Vec<Box<dyn Command>>>);
impl CommandQueue {
    #[must_use]
    pub fn new() -> Self {
        Self(RefCell::new(vec![]))
    }

    pub fn push_command<C>(&self, command: C)
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
