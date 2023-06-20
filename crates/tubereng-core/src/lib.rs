use tubereng_math::{quaternion::Quaternion, vector::Vector3f};

#[derive(Debug, Clone)]
pub struct Tranform {
    pub translation: Vector3f,
    pub scale: Vector3f,
    pub rotation: Quaternion,
}
