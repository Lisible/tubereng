use tubereng_math::{matrix::Matrix4f, quaternion::Quaternion, vector::Vector3f};

#[derive(Debug, Clone)]
pub struct Transform {
    pub translation: Vector3f,
    pub scale: Vector3f,
    pub rotation: Quaternion,
}

impl Transform {
    pub fn as_matrix4(&self) -> Matrix4f {
        Matrix4f::new_scale(&self.scale)
            * Matrix4f::new_translation(&self.translation)
            * self.rotation.rotation_matrix()
    }
}

impl Default for Transform {
    fn default() -> Self {
        Self {
            translation: Vector3f::new(0.0, 0.0, 0.0),
            scale: Vector3f::new(1.0, 1.0, 1.0),
            rotation: Quaternion::new(1.0, Vector3f::new(0.0, 0.0, 0.0)),
        }
    }
}
