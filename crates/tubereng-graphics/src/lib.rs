#[derive(Clone, Copy)]
pub struct WindowSize {
    pub width: u32,
    pub height: u32,
}

impl WindowSize {
    pub fn new(width: u32, height: u32) -> Self {
        Self { width, height }
    }
}

pub trait Renderer {
    fn render(&mut self);
    fn resize(&mut self, new_size: WindowSize);
}
