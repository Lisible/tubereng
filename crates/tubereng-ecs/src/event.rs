use std::{
    any::{Any, TypeId},
    cell::{Ref, RefCell, RefMut},
    marker::PhantomData,
    rc::Rc,
};

pub struct EventQueue {
    pending_events: Rc<RefCell<Vec<Box<dyn Any>>>>,
    next_events: Rc<RefCell<Vec<Box<dyn Any>>>>,
}

impl EventQueue {
    #[must_use]
    pub fn new() -> Self {
        Self {
            pending_events: Rc::new(RefCell::new(Vec::new())),
            next_events: Rc::new(RefCell::new(Vec::new())),
        }
    }

    pub fn push<E>(&self, event: E)
    where
        E: 'static + Any,
    {
        self.next_events.borrow_mut().push(Box::new(event));
    }

    pub fn swap_and_clear(&mut self) {
        std::mem::swap(&mut self.pending_events, &mut self.next_events);
        self.next_events.borrow_mut().clear();
    }

    #[must_use]
    pub fn pending_events(&self) -> Ref<Vec<Box<dyn Any>>> {
        self.pending_events.borrow()
    }

    #[must_use]
    pub fn pending_events_mut(&mut self) -> RefMut<Vec<Box<dyn Any>>> {
        self.pending_events.borrow_mut()
    }
    
    #[must_use]
    pub fn len(&self) -> usize {
        self.pending_events.borrow().len()
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
    E: 'static,
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
    queue: Ref<'q, Vec<Box<dyn Any>>>,
    _marker: PhantomData<E>,
}

impl<'q, E> EventReader<'q, E>
where
    E: 'static,
{
    #[must_use]
    pub fn new(queue: Ref<'q, Vec<Box<dyn Any>>>) -> Self {
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

