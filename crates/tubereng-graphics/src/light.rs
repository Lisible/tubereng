use tubereng_math::vector::Vector3f;

#[derive(Debug)]
pub struct DirectionalLight {
    pub direction: Vector3f,
    pub color: Vector3f,
}

impl Default for DirectionalLight {
    fn default() -> Self {
        Self {
            direction: Vector3f::new(0.0, -0.3, 1.0),
            color: Vector3f::new(1.0, 1.0, 1.0),
        }
    }
}

#[derive(Debug)]
pub struct PointLight {
    pub color: Vector3f,
    pub constant: f32,
    pub linear: f32,
    pub quadratic: f32,
}

impl Default for PointLight {
    fn default() -> Self {
        Self {
            color: Vector3f::new(1.0, 1.0, 1.0),
            constant: 1.0,
            linear: 0.14,
            quadratic: 0.07,
        }
    }
}
