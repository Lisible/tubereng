use crate::{Ecs, EntityDefinition};

pub trait EcsCommand {
    fn apply(self, ecs: &mut Ecs);
}

pub struct InsertEntity<ED>
where
    ED: EntityDefinition,
{
    entity: ED,
}

impl<ED> InsertEntity<ED>
where
    ED: EntityDefinition,
{
    pub fn new(entity: ED) -> Self {
        Self { entity }
    }
}

impl<ED> EcsCommand for InsertEntity<ED>
where
    ED: EntityDefinition,
{
    fn apply(self, ecs: &mut Ecs) {
        ecs.insert(self.entity);
    }
}

pub struct EcsCommandBuffer {
    commands: Vec<Box<dyn EcsCommand>>,
}

impl EcsCommandBuffer {
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
}

impl Default for EcsCommandBuffer {
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
        let insert_entity_command = InsertEntity::new((Player, Health(10)));
        insert_entity_command.apply(&mut ecs);
        assert_eq!(ecs.entity_count(), 1);
        let insert_entity_command = InsertEntity::new((Player, Health(10)));
        insert_entity_command.apply(&mut ecs);
        let insert_entity_command = InsertEntity::new((Player, Health(10)));
        insert_entity_command.apply(&mut ecs);
        assert_eq!(ecs.entity_count(), 3);
    }
}
