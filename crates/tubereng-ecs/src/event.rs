use std::{
    any::{Any, TypeId},
    marker::PhantomData, sync::Arc,
};

use parking_lot::{Mutex, MutexGuard};


pub struct EventQueue {
    pending_events: Arc<Mutex<Vec<Box<dyn Any + Send>>>>,
    next_events: Arc<Mutex<Vec<Box<dyn Any + Send>>>>,
}

impl EventQueue {
    #[must_use]
    pub fn new() -> Self {
        Self {
            pending_events: Arc::new(Mutex::new(Vec::new())),
            next_events: Arc::new(Mutex::new(Vec::new())),
        }
    }

    pub fn push<E>(&self, event: E)
    where
        E: 'static + Any + Send,
    {
        self.next_events.lock().push(Box::new(event));
    }

    pub fn swap_and_clear(&mut self) {
        std::mem::swap(&mut self.pending_events, &mut self.next_events);
        self.next_events.lock().clear();
    }

    pub fn pending_events(&self) -> MutexGuard<Vec<Box<dyn Any + Send>>> {
        self.pending_events.lock()
    }

    pub fn pending_events_mut(&mut self) -> MutexGuard<Vec<Box<dyn Any + Send>>> {
        self.pending_events.lock()
    }
    
    #[must_use]
    pub fn len(&self) -> usize {
        self.pending_events.lock().len()
    }

    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }
}

impl Default for EventQueue {
    fn default() -> Self {
        Self::new()
    }
}

pub struct EventWriter<'q, E> {
    queue: &'q EventQueue,
    _marker: PhantomData<E>,
}

impl<'q, E> EventWriter<'q, E>
where
    E: 'static + Send,
{
    #[must_use]
    pub fn new(queue: &'q EventQueue) -> Self {
        Self {
            queue,
            _marker: PhantomData,
        }
    }

    pub fn write(&self, event: E) {
        self.queue.push(event);
    }
}

pub struct EventReader<'q, E> {
    queue: MutexGuard<'q, Vec<Box<dyn Any + Send>>>,
    _marker: PhantomData<E>,
}

impl<'q, E> EventReader<'q, E>
where
    E: 'static,
{
    #[must_use]
    pub fn new(queue: MutexGuard<'q, Vec<Box<dyn Any + Send>>>) -> Self {
        Self {
            queue,
            _marker: PhantomData,
        }
    }

    pub fn iter(&self) -> impl Iterator<Item = &E> {
        self.queue
            .iter()
            .filter(|e| (***e).type_id() == TypeId::of::<E>())
            .map(|e| 
                // SAFETY: We filtered items with the type id of E
                // so they can only be E instances
                unsafe { e.downcast_ref::<E>().unwrap_unchecked() })
    }
}

