#![warn(clippy::pedantic)]

pub struct Engine;
impl Engine {
    pub fn update(&mut self) {
        println!("update");
    }
    pub fn render(&mut self) {
        println!("render");
    }
}
