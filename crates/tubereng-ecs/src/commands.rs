use crate::{
    system::{self, System},
    Ecs, EntityDefinition,
};

pub struct CommandBuffer {
    commands: Vec<Box<dyn Command>>,
}

impl CommandBuffer {
    #[must_use]
    pub fn new() -> Self {
        Self {
            commands: Vec::new(),
        }
    }

    pub fn clear(&mut self) {
        self.commands.clear();
    }

    pub fn insert<ED>(&mut self, entity: ED)
    where
        ED: 'static + EntityDefinition,
    {
        self.commands.push(Box::new(InsertEntity::new(entity)));
    }

    pub fn register_system<S, T>(&mut self, system: S)
    where
        S: system::Into<T>,
    {
        self.commands.push(Box::new(RegisterSystem::new(system)));
    }

    pub fn iter_mut(&mut self) -> std::slice::IterMut<Box<dyn Command>> {
        self.commands.iter_mut()
    }

    #[must_use]
    pub fn len(&self) -> usize {
        self.commands.len()
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
    system: Option<System>,
}

impl RegisterSystem {
    pub fn new<S, T>(system: S) -> Self
    where
        S: system::Into<T>,
    {
        Self {
            system: Some(system.into_system()),
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
