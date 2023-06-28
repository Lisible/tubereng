use std::{
    any::{Any, TypeId},
    cell::{Ref, RefCell, RefMut},
    collections::HashMap,
    rc::Rc,
};

pub struct Resources {
    resources: HashMap<TypeId, Rc<RefCell<Box<dyn Any>>>>,
}

impl Resources {
    #[must_use]
    pub fn new() -> Self {
        Self {
            resources: HashMap::new(),
        }
    }

    pub fn insert<R>(&mut self, resource: R)
    where
        R: 'static + Any,
    {
        self.resources
            .insert(TypeId::of::<R>(), Rc::new(RefCell::new(Box::new(resource))));
    }

    #[must_use]
    pub fn resource<R>(&self) -> Option<Ref<R>>
    where
        R: 'static + Any,
    {
        Some(Ref::map(self.resources[&TypeId::of::<R>()].borrow(), |r| {
            r.downcast_ref().expect("Couldn't downcast resource ref")
        }))
    }

    #[must_use]
    pub fn resource_mut<R>(&self) -> Option<RefMut<R>>
    where
        R: 'static + Any,
    {
        Some(RefMut::map(
            self.resources[&TypeId::of::<R>()].borrow_mut(),
            |r| {
                r.downcast_mut()
                    .expect("Couldn't downcast resource mut ref")
            },
        ))
    }
}

impl Default for Resources {
    fn default() -> Self {
        Self::new()
    }
}
