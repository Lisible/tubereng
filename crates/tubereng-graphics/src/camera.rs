use tubereng_core::Transform;
use tubereng_ecs::{
    query::Q,
    system::{Res, SystemSet},
};
use tubereng_input::{keyboard::Key, InputState};
use tubereng_math::{
    matrix::{Identity, Matrix4f},
    quaternion::Quaternion,
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

#[derive(Debug)]
pub struct FlyCamera;

impl FlyCamera {
    pub fn system_bundle() -> SystemSet {
        let mut system_set = SystemSet::new();
        system_set.add_system(move_camera);
        system_set.add_system(rotate_camera);
        system_set
    }
}

fn rotate_camera(
    camera_query: Q<(&ActiveCamera, &FlyCamera, &mut Transform)>,
    input: Res<InputState>,
) {
    let input = input.0;
    let mouse_motion = input.mouse.motion();

    let (_, _, mut transform) = camera_query
        .iter()
        .next()
        .expect("No active camera found in the scene");
    #[allow(clippy::cast_possible_truncation)]
    {
        transform.rotation = Quaternion::from_axis_angle(
            &Vector3f::new(0.0, 1.0, 0.0),
            -mouse_motion.0 as f32 * 0.01,
        ) * transform.rotation.clone()
            * Quaternion::from_axis_angle(
                &Vector3f::new(1.0, 0.0, 0.0),
                -mouse_motion.1 as f32 * 0.01,
            );
    }
}

fn move_camera(
    camera_query: Q<(&ActiveCamera, &FlyCamera, &mut Transform)>,
    input: Res<InputState>,
) {
    let input = input.0;

    let mut camera_speed = 0.01;
    if input.keyboard.is_key_down(Key::LShift) {
        camera_speed = 0.1;
    }

    let (_, _, mut transform) = camera_query
        .iter()
        .next()
        .expect("No active camera found in the scene");
    let forward = transform
        .rotation
        .apply_to_vector(&Vector3f::new(0.0, 0.0, -1.0));

    let up = transform
        .rotation
        .apply_to_vector(&Vector3f::new(0.0, 1.0, 0.0));

    let right = transform
        .rotation
        .apply_to_vector(&Vector3f::new(1.0, 0.0, 0.0));

    if input.keyboard.is_key_down(Key::W) {
        transform.translation += forward * camera_speed;
    }
    if input.keyboard.is_key_down(Key::S) {
        transform.translation -= forward * camera_speed;
    }
    if input.keyboard.is_key_down(Key::D) {
        transform.translation += right * camera_speed;
    }
    if input.keyboard.is_key_down(Key::A) {
        transform.translation -= right * camera_speed;
    }
    if input.keyboard.is_key_down(Key::Space) {
        transform.translation += up * camera_speed;
    }
    if input.keyboard.is_key_down(Key::LControl) {
        transform.translation -= up * camera_speed;
    }
}

#[repr(C)]
#[derive(Debug, Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
pub struct CameraUniform {
    position: [f32; 3],
    _padding: u32,
    view_projection_matrix: [[f32; 4]; 4],
}

impl CameraUniform {
    #[must_use]
    pub fn new() -> Self {
        Self {
            view_projection_matrix: Matrix4f::identity().into(),
            _padding: 0,
            position: [0.0, 0.0, 0.0],
        }
    }

    pub fn set_view_projection_matrix(&mut self, view_projection_matrix: Matrix4f) {
        self.view_projection_matrix = view_projection_matrix.into();
    }

    pub fn set_position(&mut self, position: Vector3f) {
        self.position[0] = position.x;
        self.position[1] = position.y;
        self.position[2] = position.z;
    }
}

impl Default for CameraUniform {
    fn default() -> Self {
        Self::new()
    }
}
