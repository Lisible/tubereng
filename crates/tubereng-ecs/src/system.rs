use crate::EntityDefinition;

pub enum EcsCommand {
    InsertEntity(Box<dyn EntityDefinition>),
}

pub struct EcsCommandBuffer {
    commands: Vec<EcsCommand>,
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
        self.commands
            .push(EcsCommand::InsertEntity(Box::new(entity)));
    }
}

impl Default for EcsCommandBuffer {
    fn default() -> Self {
        Self::new()
    }
}

pub struct System {
    command_buffer: EcsCommandBuffer,
    system_fn: Box<dyn FnMut(&mut EcsCommandBuffer)>,
}

impl System {
    pub fn run(&mut self) {
        self.command_buffer.clear();
        (self.system_fn)(&mut self.command_buffer);
    }

    pub fn command_buffer_mut(&mut self) -> &mut EcsCommandBuffer {
        &mut self.command_buffer
    }
}

pub trait Into<T> {
    fn into_system(self) -> System;
}

impl<F> Into<F> for F
where
    F: 'static + FnMut(),
{
    fn into_system(mut self) -> System {
        System {
            system_fn: Box::new(move |_| (self)()),
            command_buffer: EcsCommandBuffer::new(),
        }
    }
}

impl<F> Into<(F,)> for F
where
    F: 'static + FnMut(&mut EcsCommandBuffer),
{
    fn into_system(self) -> System {
        System {
            system_fn: Box::new(self),
            command_buffer: EcsCommandBuffer::new(),
        }
    }
}
