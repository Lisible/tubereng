use crate::commands::EcsCommandBuffer;

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
