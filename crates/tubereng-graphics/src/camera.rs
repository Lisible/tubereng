use tubereng_math::{
    matrix::{Identity, Matrix4f},
    vector::Vector3f,
};

#[rustfmt::skip]
pub const OPENGL_TO_WGPU_MATRIX: Matrix4f = Matrix4f::with_values([
    1.0, 0.0, 0.0, 0.0,
    0.0, 1.0, 0.0, 0.0,
    0.0, 0.0, 0.5, 0.0,
    0.0, 0.0, 0.5, 1.0
]);

#[derive(Debug)]
pub struct Camera {
    eye: Vector3f,
    target: Vector3f,
    up: Vector3f,
    aspect: f32,
    fovy: f32,
    znear: f32,
    zfar: f32,
}

impl Camera {
    pub fn new(
        eye: Vector3f,
        target: Vector3f,
        up: Vector3f,
        aspect: f32,
        fovy: f32,
        znear: f32,
        zfar: f32,
    ) -> Self {
        Self {
            eye,
            target,
            up,
            aspect,
            fovy,
            znear,
            zfar,
        }
    }

    #[must_use]
    pub fn projection_matrix(&self) -> Matrix4f {
        let view = Matrix4f::new_look_at(self.eye, self.target, self.up);
        let projection = Matrix4f::new_perspective(self.fovy, self.aspect, self.znear, self.zfar);
        OPENGL_TO_WGPU_MATRIX * projection * view
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
