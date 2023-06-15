use crate::{Ecs, EntityDefinition};

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

    pub fn iter_mut(&mut self) -> std::slice::IterMut<Box<dyn Command>> {
        self.commands.iter_mut()
    }

    pub fn len(&self) -> usize {
        self.commands.len()
    }
}

impl Default for CommandBuffer {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    struct Player;
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
