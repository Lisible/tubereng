use crate::commands::CommandBuffer;

pub struct System {
    system_fn: Box<dyn Fn(&mut CommandBuffer)>,
}

impl System {
    pub fn run(&self, command_buffer: &mut CommandBuffer) {
        (self.system_fn)(command_buffer);
    }
}

pub trait Into<T> {
    fn into_system(self) -> System;
}

impl<F> Into<()> for F
where
    F: 'static + Fn(),
{
    fn into_system(self) -> System {
        System {
            system_fn: Box::new(move |_| (self)()),
        }
    }
}

impl<F> Into<(&mut CommandBuffer,)> for F
where
    F: 'static + Fn(&mut CommandBuffer),
{
    fn into_system(self) -> System {
        System {
            system_fn: Box::new(self),
        }
    }
}
