use log::trace;
use std::{
    collections::HashMap,
    hash::{DefaultHasher, Hash, Hasher},
    ops::DerefMut,
};
use tubereng_ecs::system::Res;
use tubereng_ecs::system::ResMut;
use tubereng_input::{mouse, Input};
use tubereng_renderer::GraphicsState;

pub type ComponentId = u64;
pub struct Context {
    last_cursor_position: (f32, f32),
    components: HashMap<ComponentId, Box<dyn Component>>,
}

impl Context {
    pub fn new() -> Self {
        Self {
            last_cursor_position: (0.0, 0.0),
            components: HashMap::new(),
        }
    }

    pub fn on_input(&mut self, input: &Input) {
        match &input {
            Input::CursorMoved(position) => {
                self.on_cursor_moved((position.0 as f32, position.1 as f32))
            }
            Input::MouseButtonDown(button) => {
                self.on_mouse_button_down(*button);
            }
            Input::MouseButtonUp(button) => {
                self.on_mouse_button_up(*button);
            }
            _ => {}
        }
    }

    fn on_cursor_moved(&mut self, position: (f32, f32)) {
        for component in &mut self.components.values_mut() {
            let is_cursor_in_component = component.rect().contains((position.0, position.1));
            if !component.is_hovering() && is_cursor_in_component {
                component.on_hover();
            } else if component.is_hovering() && !is_cursor_in_component {
                component.off_hover();
            }

            if component.is_grabbing() {
                let delta_pos = (
                    position.0 - self.last_cursor_position.0,
                    position.1 - self.last_cursor_position.1,
                );
                component.move_rel(delta_pos);
            }
        }

        self.last_cursor_position = position;
    }

    fn on_mouse_button_down(&mut self, button: mouse::Button) {
        for component in &mut self.components.values_mut() {
            if component.is_hovering() && button == mouse::Button::Left {
                component.start_grabbing();
            }
        }
    }
    fn on_mouse_button_up(&mut self, button: mouse::Button) {
        for component in &mut self.components.values_mut() {
            if component.is_hovering() && button == mouse::Button::Left {
                component.stop_grabbing();
            }
        }
    }

    pub fn component_count(&self) -> usize {
        self.components.len()
    }
}

impl Default for Context {
    fn default() -> Self {
        Self::new()
    }
}

pub struct Rect {
    x: f32,
    y: f32,
    width: f32,
    height: f32,
}

impl Rect {
    pub fn contains(&self, position: (f32, f32)) -> bool {
        position.0 >= self.x
            && position.0 <= self.x + self.width
            && position.1 >= self.y
            && position.1 <= self.y + self.height
    }
}

pub struct Window {
    title: String,
    rect: Rect,
    hovering: bool,
    grabbing: bool,
}

impl Window {
    pub fn new<S>(title: S) -> Self
    where
        S: ToString,
    {
        Self {
            title: title.to_string(),
            rect: Rect {
                x: 0.0,
                y: 0.0,
                width: 300.0,
                height: 200.0,
            },
            hovering: false,
            grabbing: false,
        }
    }

    pub fn init_position(mut self, x: f32, y: f32) -> Self {
        self.rect.x = x;
        self.rect.y = y;
        self
    }

    pub fn init_size(mut self, width: f32, height: f32) -> Self {
        self.rect.width = width;
        self.rect.height = height;
        self
    }
}

pub trait Component: Renderable {
    fn show(self, ctx: &mut Context);
    fn rect(&self) -> &Rect;

    fn on_hover(&mut self);
    fn off_hover(&mut self);
    fn is_hovering(&self) -> bool;
    fn start_grabbing(&mut self);
    fn stop_grabbing(&mut self);
    fn is_grabbing(&self) -> bool;
    fn move_rel(&mut self, delta_pos: (f32, f32));
}

impl Component for Window {
    fn show(self, ctx: &mut Context) {
        let mut hasher = DefaultHasher::new();
        self.title.hash(&mut hasher);
        let id = hasher.finish();
        ctx.components.entry(id).or_insert_with(|| Box::new(self));
    }

    fn rect(&self) -> &Rect {
        &self.rect
    }

    fn on_hover(&mut self) {
        trace!("Hovering window {}", self.title);
        self.hovering = true;
    }
    fn off_hover(&mut self) {
        trace!("Stopped hovering window {}", self.title);
        self.hovering = false;
    }
    fn is_hovering(&self) -> bool {
        self.hovering
    }
    fn start_grabbing(&mut self) {
        trace!("Grabbing window {}", self.title);
        self.grabbing = true;
    }
    fn stop_grabbing(&mut self) {
        trace!("Stopped grabbing window {}", self.title);
        self.grabbing = false;
    }
    fn is_grabbing(&self) -> bool {
        self.grabbing
    }
    fn move_rel(&mut self, delta_pos: (f32, f32)) {
        self.rect.x += delta_pos.0;
        self.rect.y += delta_pos.1;
    }
}

pub trait Renderable {
    fn render(&self, gfx: &mut GraphicsState);
}

impl Renderable for Window {
    fn render(&self, gfx: &mut GraphicsState) {
        gfx.draw_ui_quad(self.rect.x, self.rect.y, self.rect.width, self.rect.height);
    }
}

pub fn emit_draw_commands_system(ctx: Res<Context>, mut gfx: ResMut<GraphicsState>) {
    let gfx = gfx.deref_mut();
    for component in ctx.components.values() {
        component.render(gfx);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn window() {
        let mut ctx = Context::new();
        assert_eq!(ctx.component_count(), 0);
        Window::new("some_window").show(&mut ctx);
        assert_eq!(ctx.component_count(), 1);
    }
}
