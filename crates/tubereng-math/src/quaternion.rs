use std::fmt::{Debug, Display, Formatter};
use std::ops::Mul;

use crate::matrix::Matrix4;
use crate::number_traits::Float;
use crate::vector::Vector3;

#[derive(Debug, Clone)]
pub struct Quaternion<T = f32>
where
    T: Debug,
{
    scalar_part: T,
    vector_part: Vector3<T>,
}

impl<T> Quaternion<T>
where
    T: Debug + Float,
{
    pub fn new(scalar_part: T, vector_part: Vector3<T>) -> Self {
        Self {
            scalar_part,
            vector_part,
        }
    }

    pub fn from_axis_angle(axis: &Vector3<T>, angle: T) -> Self {
        let half_angle = angle.half();
        let half_angle_sin = half_angle.sin();
        let x = axis.x * half_angle_sin;
        let y = axis.y * half_angle_sin;
        let z = axis.z * half_angle_sin;
        let w = half_angle.cos();

        Self::new(w, Vector3::new(x, y, z))
    }

    pub fn from_euler(angles: &Vector3<T>) -> Self {
        let roll = angles.x;
        let pitch = angles.y;
        let yaw = angles.z;
        let half_roll = roll.half();
        let half_pitch = pitch.half();
        let half_yaw = yaw.half();
        let cy = half_yaw.cos();
        let sy = half_yaw.sin();
        let cp = half_pitch.cos();
        let sp = half_pitch.sin();
        let cr = half_roll.cos();
        let sr = half_roll.sin();

        let w = cr * cp * cy + sr * sp * sy;
        let x = sr * cp * cy - cr * sp * sy;
        let y = cr * sp * cy + sr * cp * sy;
        let z = cr * cp * sy - sr * sp * cy;

        Quaternion::new(w, Vector3::new(x, y, z))
    }

    #[rustfmt::skip]
    #[allow(clippy::similar_names)]
    pub fn rotation_matrix(&self) -> Matrix4<T> {
        let (w, x, y, z) = (
            self.scalar_part,
            self.vector_part.x,
            self.vector_part.y,
            self.vector_part.z,
        );
        let x2 = x + x;
        let y2 = y + y;
        let z2 = z + z;
        let w2 = w + w;
        let xx2 = x2 * x;
        let xy2 = x2 * y;
        let xz2 = x2 * z;
        let yy2 = y2 * y;
        let yz2 = y2 * z;
        let zz2 = z2 * z;
        let wx2 = w2 * x;
        let wy2 = w2 * y;
        let wz2 = w2 * z;

        Matrix4::with_values([
            T::one() - yy2 - zz2, xy2 - wz2, xz2 + wy2, T::zero(),
            xy2 + wz2, T::one() - xx2 - zz2, yz2 - wx2, T::zero(),
            xz2 - wy2, yz2 + wx2, T::one() - xx2 - yy2, T::zero(),
            T::zero(), T::zero(), T::zero(), T::one()
        ])
    }

    pub fn normalize(&mut self) {
        let norm = self.norm();
        self.vector_part /= norm;
        self.scalar_part /= norm;
    }

    #[must_use]
    pub fn normalized(&self) -> Self {
        let mut normalized_quaternion = self.clone();
        normalized_quaternion.normalize();
        normalized_quaternion
    }

    pub fn norm(&self) -> T {
        let x = self.vector_part.x;
        let y = self.vector_part.y;
        let z = self.vector_part.z;
        let w = self.scalar_part;
        let xx = x * x;
        let yy = y * y;
        let zz = z * z;
        let ww = w * w;

        (ww + xx + yy + zz).sqrt()
    }
}

impl<T> Display for Quaternion<T>
where
    T: Debug + Float,
{
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        writeln!(
            f,
            "({} + {} i + {} j + {} k)",
            self.scalar_part, self.vector_part.x, self.vector_part.y, self.vector_part.z
        )
    }
}

impl<T> Mul for Quaternion<T>
where
    T: Debug + Float,
{
    type Output = Self;

    fn mul(self, rhs: Self) -> Self::Output {
        let x1 = self.vector_part.x;
        let y1 = self.vector_part.y;
        let z1 = self.vector_part.z;
        let w1 = self.scalar_part;

        let x2 = rhs.vector_part.x;
        let y2 = rhs.vector_part.y;
        let z2 = rhs.vector_part.z;
        let w2 = rhs.scalar_part;

        let scalar_part = w1 * w2 - x1 * x2 - y1 * y2 - z1 * z2;
        let vector_part = Vector3::new(
            x1 * w2 + y1 * z2 - z1 * y2 + w1 * x2,
            y1 * w2 + z1 * x2 + w1 * y2 - x1 * z2,
            z1 * w2 + w1 * z2 + x1 * y2 - y1 * x2,
        );

        Quaternion::new(scalar_part, vector_part)
    }
}

#[cfg(test)]
mod tests {
    use assert_float_eq::assert_float_absolute_eq;

    use super::*;

    #[test]
    fn mul() {
        let q1 = Quaternion::new(12.4, Vector3::new(1.1, 2.0, 4.4));
        let q2 = Quaternion::new(4.0, Vector3::new(0.3, 45.0, 5.0));

        let result = q1 * q2;

        assert_float_absolute_eq!(result.scalar_part, -62.73, 0.01);
        assert_float_absolute_eq!(result.vector_part.x, -179.88, 0.01);
        assert_float_absolute_eq!(result.vector_part.y, 561.82, 0.01);
        assert_float_absolute_eq!(result.vector_part.z, 128.5, 0.01);
    }

    #[test]
    fn rotation_matrix() {
        let q = Quaternion::new(0.56, Vector3::new(0.77, -0.31, 0.0));

        let matrix = q.rotation_matrix();

        assert_float_absolute_eq!(matrix[0][0], 0.80, 0.02);
        assert_float_absolute_eq!(matrix[0][1], -0.47, 0.02);
        assert_float_absolute_eq!(matrix[0][2], -0.34, 0.02);
        assert_float_absolute_eq!(matrix[0][3], 0.0, 0.02);
        assert_float_absolute_eq!(matrix[1][0], -0.47, 0.02);
        assert_float_absolute_eq!(matrix[1][1], -0.18, 0.02);
        assert_float_absolute_eq!(matrix[1][2], -0.86, 0.02);
        assert_float_absolute_eq!(matrix[1][3], 0.0, 0.02);
        assert_float_absolute_eq!(matrix[2][0], 0.34, 0.02);
        assert_float_absolute_eq!(matrix[2][1], 0.86, 0.02);
        assert_float_absolute_eq!(matrix[2][2], -0.37, 0.02);
        assert_float_absolute_eq!(matrix[2][3], 0.0, 0.02);
        assert_float_absolute_eq!(matrix[3][0], 0.0, 0.02);
        assert_float_absolute_eq!(matrix[3][1], 0.0, 0.02);
        assert_float_absolute_eq!(matrix[3][2], 0.0, 0.02);
        assert_float_absolute_eq!(matrix[3][3], 1.0, 0.02);
    }

    #[test]
    fn norm() {
        let quaternion = Quaternion::new(23.0, Vector3::new(12.0, 34.0, 56.0));

        let norm = quaternion.norm();

        assert_float_absolute_eq!(norm, 70.46, 0.01);
    }

    #[test]
    fn normalize() {
        let mut quaternion = Quaternion::new(23.0, Vector3::new(12.0, 34.0, 56.0));

        quaternion.normalize();

        assert_float_absolute_eq!(quaternion.scalar_part, 0.32, 0.01);
        assert_float_absolute_eq!(quaternion.vector_part.x, 0.17, 0.01);
        assert_float_absolute_eq!(quaternion.vector_part.y, 0.48, 0.01);
        assert_float_absolute_eq!(quaternion.vector_part.z, 0.79, 0.01);
    }

    #[test]
    fn normalized() {
        let quaternion = Quaternion::new(23.0, Vector3::new(12.0, 34.0, 56.0));

        let normalized = quaternion.normalized();

        assert_float_absolute_eq!(normalized.scalar_part, 0.32, 0.01);
        assert_float_absolute_eq!(normalized.vector_part.x, 0.17, 0.01);
        assert_float_absolute_eq!(normalized.vector_part.y, 0.48, 0.01);
        assert_float_absolute_eq!(normalized.vector_part.z, 0.79, 0.01);
    }

    #[test]
    fn from_axis_angle() {
        let axis = Vector3::new(1.0, 2.0, 3.0).normalized();
        let angle = 0.74;

        let quaternion = Quaternion::from_axis_angle(&axis, angle);

        assert_float_absolute_eq!(quaternion.scalar_part, 0.93, 0.01);
        assert_float_absolute_eq!(quaternion.vector_part.x, 0.09, 0.01);
        assert_float_absolute_eq!(quaternion.vector_part.y, 0.19, 0.01);
        assert_float_absolute_eq!(quaternion.vector_part.z, 0.28, 0.01);
    }

    #[test]
    fn from_euler() {
        let angles = Vector3::new(0.4, 1.3, 5.0);

        let quaternion = Quaternion::from_euler(&angles);

        assert_float_absolute_eq!(quaternion.scalar_part, -0.55, 0.01);
        assert_float_absolute_eq!(quaternion.vector_part.x, -0.48, 0.01);
        assert_float_absolute_eq!(quaternion.vector_part.y, -0.38, 0.01);
        assert_float_absolute_eq!(quaternion.vector_part.z, 0.56, 0.01);
    }
}
