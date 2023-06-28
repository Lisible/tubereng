use tubereng_math::matrix::{Identity, Matrix4f};

#[rustfmt::skip]
pub const OPENGL_TO_WGPU_MATRIX: Matrix4f = Matrix4f::with_values([
    1.0, 0.0, 0.0, 0.0,
    0.0, 1.0, 0.0, 0.0,
    0.0, 0.0, 0.5, 0.0,
    0.0, 0.0, 0.5, 1.0
]);

#[derive(Debug)]
pub struct Camera {
    projection_matrix: Matrix4f,
}

impl Camera {
    #[must_use]
    pub fn new_perspective(fov_y: f32, aspect: f32, near: f32, far: f32) -> Self {
        Self {
            projection_matrix: Matrix4f::new_perspective(fov_y, aspect, near, far),
        }
    }

    #[must_use]
    pub fn new_orthographic(
        left: f32,
        right: f32,
        bottom: f32,
        top: f32,
        near: f32,
        far: f32,
    ) -> Self {
        Self {
            projection_matrix: Matrix4f::new_orthographic(left, right, bottom, top, near, far),
        }
    }

    #[must_use]
    pub fn projection_matrix(&self) -> &Matrix4f {
        &self.projection_matrix
    }
}

/// Component to define a camera as the active one
#[allow(clippy::module_name_repetitions)]
#[derive(Debug)]
pub struct ActiveCamera;

#[repr(C)]
#[derive(Debug, Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
pub struct CameraUniform {
    view_projection_matrix: [[f32; 4]; 4],
}

impl CameraUniform {
    #[must_use]
    pub fn new() -> Self {
        Self {
            view_projection_matrix: Matrix4f::identity().into(),
        }
    }

    pub fn set_view_projection_matrix(&mut self, view_projection_matrix: Matrix4f) {
        self.view_projection_matrix = view_projection_matrix.into();
    }
}

impl Default for CameraUniform {
    fn default() -> Self {
        Self {
            view_projection_matrix: Matrix4f::identity().into(),
        }
    }
}
