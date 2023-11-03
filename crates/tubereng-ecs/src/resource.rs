use std::{
    any::{Any, TypeId},
    collections::HashMap,
    sync::Arc,
};

use parking_lot::{
    MappedRwLockReadGuard, MappedRwLockWriteGuard, RwLock, RwLockReadGuard, RwLockWriteGuard,
};

pub struct Resources {
    resources: HashMap<TypeId, Arc<RwLock<dyn Any + Send + Sync>>>,
}

pub type ResourceRef<'a, R> = MappedRwLockReadGuard<'a, R>;
pub type ResourceRefMut<'a, R> = MappedRwLockWriteGuard<'a, R>;

impl Resources {
    #[must_use]
    pub fn new() -> Self {
        Self {
            resources: HashMap::new(),
        }
    }

    pub fn insert<R>(&mut self, resource: R)
    where
        R: 'static + Any + Send + Sync,
    {
        self.resources
            .insert(TypeId::of::<R>(), Arc::new(RwLock::new(resource)));
    }

    #[must_use]
    pub fn resource<R>(&self) -> Option<ResourceRef<R>>
    where
        R: 'static + Any + Send,
    {
        Some(RwLockReadGuard::map(
            self.resources.get(&TypeId::of::<R>())?.as_ref().read(),
            // SAFETY: We know that the type of e is R as it has been retrieved from
            // `self.resources[&TypeId::of::<R>()]`
            |e| unsafe { e.downcast_ref::<R>().unwrap_unchecked() },
        ))
    }

    #[must_use]
    pub fn resource_mut<R>(&self) -> Option<ResourceRefMut<R>>
    where
        R: Any + Send,
    {
        Some(RwLockWriteGuard::map(
            self.resources.get(&TypeId::of::<R>())?.as_ref().write(),
            // SAFETY: We know that the type of e is R as it has been retrieved from
            // `self.resources[&TypeId::of::<R>()]`
            |e| unsafe { e.downcast_mut::<R>().unwrap_unchecked() },
        ))
    }
}

impl Default for Resources {
    fn default() -> Self {
        Self::new()
    }
}
