use std::{any::Any, cell::RefCell, marker::PhantomData, vec::IntoIter};

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

    pub fn insert_resource<R>(&self, resource: R)
    where
        R: 'static,
    {
        self.push_command(InsertResource::new(resource));
    }

    pub fn register_system<S, F, A>(&self, system: F)
    where
        S: 'static,
        F: system::Into<A>,
    {
        self.push_command(RegisterSystem::<S>::new::<S, _, _>(system));
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

pub struct InsertResource {
    resource: Option<Box<dyn Any>>,
}

impl InsertResource {
    pub fn new<R>(resource: R) -> Self
    where
        R: 'static,
    {
        Self {
            resource: Some(Box::new(resource)),
        }
    }
}

impl Command for InsertResource {
    fn apply(&mut self, ecs: &mut Ecs) {
        ecs.insert_resource(self.resource.take().unwrap());
    }
}

pub struct RegisterSystem<S> {
    system: Option<System>,
    _marker: PhantomData<S>,
}

impl<S> RegisterSystem<S> {
    pub fn new<SS, F, A>(system: F) -> RegisterSystem<SS>
    where
        F: system::Into<A>,
    {
        RegisterSystem {
            system: Some(system.into_system()),
            _marker: PhantomData,
        }
    }
}

impl<S> Command for RegisterSystem<S>
where
    S: 'static,
{
    fn apply(&mut self, ecs: &mut Ecs) {
        ecs.insert_system::<S>(self.system.take().unwrap());
    }
}
