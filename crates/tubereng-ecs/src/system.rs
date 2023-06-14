pub trait System {
    fn run(&mut self);
}

impl<F> System for F
where
    F: FnMut(),
{
    fn run(&mut self) {
        (self)();
    }
}
