use std::{cell::RefCell, rc::Rc};

use crate::{
    system::{Into, System},
    Ecs, EntityDefinition,
};

pub struct CommandBuffer {
    commands: Rc<RefCell<Vec<Box<dyn Command>>>>,
}

impl CommandBuffer {
    #[must_use]
    pub fn new() -> Self {
        Self {
            commands: Rc::new(RefCell::new(Vec::new())),
        }
    }

    pub fn clear(&mut self) {
        self.commands.borrow_mut().clear();
    }

    pub fn insert<ED>(&self, entity: ED)
    where
        ED: 'static + EntityDefinition,
    {
        self.commands
            .borrow_mut()
            .push(Box::new(InsertEntity::new(entity)));
    }

    pub fn register_system<S, M, ST>(&self, system: S)
    where
        S: Into<M, SystemType = ST>,
        ST: System + 'static,
    {
        self.commands
            .borrow_mut()
            .push(Box::new(RegisterSystem::new(system)));
    }

    pub fn flush_commands(&mut self) -> Vec<Box<dyn Command>> {
        self.commands.borrow_mut().drain(..).collect::<Vec<_>>()
    }

    #[must_use]
    pub fn len(&self) -> usize {
        self.commands.borrow().len()
    }

    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }
}

impl Default for CommandBuffer {
    fn default() -> Self {
        Self::new()
    }
}

pub trait Command {
    fn apply(&mut self, ecs: &mut Ecs);
}

pub struct InsertEntity<ED>
where
    ED: EntityDefinition,
{
    entity: Option<ED>,
}

impl<ED> InsertEntity<ED>
where
    ED: EntityDefinition,
{
    pub fn new(entity: ED) -> Self {
        Self {
            entity: Some(entity),
        }
    }
}

impl<ED> Command for InsertEntity<ED>
where
    ED: EntityDefinition,
{
    fn apply(&mut self, ecs: &mut Ecs) {
        ecs.insert(self.entity.take().unwrap());
    }
}

pub struct RegisterSystem {
    system: Option<Box<dyn System>>,
}

impl RegisterSystem {
    pub fn new<S, ST, M>(system: S) -> Self
    where
        S: Into<M, SystemType = ST>,
        ST: System + 'static,
    {
        Self {
            system: Some(Box::new(Into::into(system))),
        }
    }
}

impl Command for RegisterSystem {
    fn apply(&mut self, ecs: &mut Ecs) {
        ecs.register_system(self.system.take().unwrap());
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Debug)]
    struct Player;
    #[derive(Debug)]
    struct Health(usize);

    #[test]
    fn apply_insert_entity_command() {
        let mut ecs = Ecs::new();
        assert_eq!(ecs.entity_count(), 0);
        let mut insert_entity_command = InsertEntity::new((Player, Health(10)));
        insert_entity_command.apply(&mut ecs);
        assert_eq!(ecs.entity_count(), 1);
        let mut insert_entity_command = InsertEntity::new((Player, Health(10)));
        insert_entity_command.apply(&mut ecs);
        let mut insert_entity_command = InsertEntity::new((Player, Health(10)));
        insert_entity_command.apply(&mut ecs);
        assert_eq!(ecs.entity_count(), 3);
    }
}
