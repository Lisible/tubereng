use crate::material;

#[derive(Debug, Default)]
pub struct Sprite {
    pub material: Option<material::Id>,
}
