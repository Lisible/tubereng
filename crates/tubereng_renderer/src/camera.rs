use tubereng_math::matrix::Matrix4f;

#[derive(Debug)]
pub struct Active;

#[derive(Debug)]
pub struct D2 {
    projection: Matrix4f,
}

impl D2 {
    #[must_use]
    pub fn new(viewport_width: f32, viewport_height: f32) -> Self {
        Self {
            projection: Matrix4f::new_orthographic(
                0.0,
                viewport_width,
                viewport_height,
                0.0,
                -1000.0,
                1000.0,
            ),
        }
    }

    pub(crate) fn projection(&self) -> &Matrix4f {
        &self.projection
    }
}
