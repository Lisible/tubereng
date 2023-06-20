use std::fmt::{Debug, Display, Formatter};
use std::ops::{Add, Index, IndexMut, Mul, MulAssign};

use crate::number_traits::{Float, IsZero, NumericOps, One, Zero};
use crate::vector::{Vector3, Vector4};

pub type Matrix4f = Matrix4<f32>;

#[derive(Clone, Copy)]
pub struct Matrix4<T = f32> {
    values: [T; 16],
}

impl<T> Debug for Matrix4<T>
where
    T: Display,
{
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "[")?;
        for i in 0..Self::ROWS {
            write!(f, "\t")?;
            for j in 0..Self::COLS {
                write!(f, "{}, ", self.values[i * Self::COLS + j])?;
            }
            writeln!(f)?;
        }
        writeln!(f, "]")
    }
}

impl<T> Matrix4<T> {
    const COLS: usize = 4;
    const ROWS: usize = 4;

    pub fn with_values(values: [T; 16]) -> Self {
        Self { values }
    }

    //noinspection RsBorrowChecker
    #[rustfmt::skip]
    pub fn new_orthographic<U>(
        left: U,
        right: U,
        bottom: U,
        top: U,
        near: U,
        far: U,
    ) -> Matrix4<U>
        where U: Copy + Float {
        Matrix4 {
            values: [
                U::two() / (right - left), U::zero(), U::zero(), -((right + left) / (right - left)),
                U::zero(), U::two() / (top - bottom), U::zero(), -((top + bottom) / (top - bottom)),
                U::zero(), U::zero(), -U::two() / (far - near), -((far + near) / (far - near)),
                U::zero(), U::zero(), U::zero(), U::one()
            ]
        }
    }

    #[rustfmt::skip]
    pub fn new_translation<U>(translation: &Vector3<U>) -> Matrix4<U>
        where U: Copy + Zero + One {
        Matrix4 {
            values: [
                U::one(), U::zero(), U::zero(), translation.x,
                U::zero(), U::one(), U::zero(), translation.y,
                U::zero(), U::zero(), U::one(), translation.z,
                U::zero(), U::zero(), U::zero(), U::one()
            ]
        }
    }

    #[rustfmt::skip]
    pub fn new_scale_uniform<U>(scale: U) -> Matrix4<U>
        where U: Copy + Zero + One {
        Self::new_scale(&Vector3::new(scale, scale, scale))
    }

    #[rustfmt::skip]
    pub fn new_scale<U>(scale: &Vector3<U>) -> Matrix4<U>
        where U: Copy + Zero + One {
        Matrix4 {
            values: [
                scale.x, U::zero(), U::zero(), U::zero(),
                U::zero(), scale.y, U::zero(), U::zero(),
                U::zero(), U::zero(), scale.z, U::zero(),
                U::zero(), U::zero(), U::zero(), U::one(),
            ]
        }
    }
}

impl<T> Matrix4<T>
where
    T: Copy + NumericOps + Zero + One + IsZero,
{
    #[rustfmt::skip]
    pub fn try_inverse(&self) -> Option<Matrix4<T>> {
        let a2323 = self[2][2] * self[3][3] - self[2][3] * self[3][2];
        let a1323 = self[2][1] * self[3][3] - self[2][3] * self[3][1];
        let a1223 = self[2][1] * self[3][2] - self[2][2] * self[3][1];
        let a0323 = self[2][0] * self[3][3] - self[2][3] * self[3][0];
        let a0223 = self[2][0] * self[3][2] - self[2][2] * self[3][0];
        let a0123 = self[2][0] * self[3][1] - self[2][1] * self[3][0];
        let a2313 = self[1][2] * self[3][3] - self[1][3] * self[3][2];
        let a1313 = self[1][1] * self[3][3] - self[1][3] * self[3][1];
        let a1213 = self[1][1] * self[3][2] - self[1][2] * self[3][1];
        let a2312 = self[1][2] * self[2][3] - self[1][3] * self[2][2];
        let a1312 = self[1][1] * self[2][3] - self[1][3] * self[2][1];
        let a1212 = self[1][1] * self[2][2] - self[1][2] * self[2][1];
        let a0313 = self[1][0] * self[3][3] - self[1][3] * self[3][0];
        let a0213 = self[1][0] * self[3][2] - self[1][2] * self[3][0];
        let a0312 = self[1][0] * self[2][3] - self[1][3] * self[2][0];
        let a0212 = self[1][0] * self[2][2] - self[1][2] * self[2][0];
        let a0113 = self[1][0] * self[3][1] - self[1][1] * self[3][0];
        let a0112 = self[1][0] * self[2][1] - self[1][1] * self[2][0];

        let det = self[0][0] * (self[1][1] * a2323 - self[1][2] * a1323 + self[1][3] * a1223)
            - self[0][1] * (self[1][0] * a2323 - self[1][2] * a0323 + self[1][3] * a0223)
            + self[0][2] * (self[1][0] * a1323 - self[1][1] * a0323 + self[1][3] * a0123)
            - self[0][3] * (self[1][0] * a1223 - self[1][1] * a0223 + self[1][2] * a0123);

        if det.is_zero() {
            return None;
        }

        let inv_det = T::one() / det;

        Some(Matrix4 {
            values: [
                inv_det * (self[1][1] * a2323 - self[1][2] * a1323 + self[1][3] * a1223),
                inv_det * -(self[0][1] * a2323 - self[0][2] * a1323 + self[0][3] * a1223),
                inv_det * (self[0][1] * a2313 - self[0][2] * a1313 + self[0][3] * a1213),
                inv_det * -(self[0][1] * a2312 - self[0][2] * a1312 + self[0][3] * a1212),
                inv_det * -(self[1][0] * a2323 - self[1][2] * a0323 + self[1][3] * a0223),
                inv_det * (self[0][0] * a2323 - self[0][2] * a0323 + self[0][3] * a0223),
                inv_det * -(self[0][0] * a2313 - self[0][2] * a0313 + self[0][3] * a0213),
                inv_det * (self[0][0] * a2312 - self[0][2] * a0312 + self[0][3] * a0212),
                inv_det * (self[1][0] * a1323 - self[1][1] * a0323 + self[1][3] * a0123),
                inv_det * -(self[0][0] * a1323 - self[0][1] * a0323 + self[0][3] * a0123),
                inv_det * (self[0][0] * a1313 - self[0][1] * a0313 + self[0][3] * a0113),
                inv_det * -(self[0][0] * a1312 - self[0][1] * a0312 + self[0][3] * a0112),
                inv_det * -(self[1][0] * a1223 - self[1][1] * a0223 + self[1][2] * a0123),
                inv_det * (self[0][0] * a1223 - self[0][1] * a0223 + self[0][2] * a0123),
                inv_det * -(self[0][0] * a1213 - self[0][1] * a0213 + self[0][2] * a0113),
                inv_det * (self[0][0] * a1212 - self[0][1] * a0212 + self[0][2] * a0112),
            ]
        })
    }

    #[must_use]
    pub fn add_translation(&self, translation: &Vector3<T>) -> Self {
        *self * Matrix4f::new_translation(translation)
    }

    pub fn append_translation(&mut self, translation: &Vector3<T>) {
        *self *= Matrix4f::new_translation(translation);
    }

    pub fn transform_vec(&self, vec: &Vector4<T>) -> Vector4<T> {
        let x = self[0][0] * vec.x + self[0][1] * vec.y + self[0][2] * vec.z + self[0][3] * vec.w;
        let y = self[1][0] * vec.x + self[1][1] * vec.y + self[1][2] * vec.z + self[1][3] * vec.w;
        let z = self[2][0] * vec.x + self[2][1] * vec.y + self[2][2] * vec.z + self[2][3] * vec.w;
        let w = self[3][0] * vec.x + self[3][1] * vec.y + self[3][2] * vec.z + self[3][3] * vec.w;
        Vector4::new(x, y, z, w)
    }

    pub fn transform_vec3(&self, vec: &Vector3<T>) -> Vector3<T> {
        let transformed = self.transform_vec(&Vector4::<T>::new(vec.x, vec.y, vec.z, T::one()));
        Vector3::new(transformed.x, transformed.y, transformed.z)
    }
}

impl<T> Mul<Self> for Matrix4<T>
where
    T: Copy + Zero + Add<Output = T> + Mul<Output = T>,
{
    type Output = Self;

    fn mul(self, rhs: Self) -> Self::Output {
        let mut values = [T::zero(); 16];

        for j in 0..4 {
            for i in 0..4 {
                values[j * Self::COLS + i] = self.values[j * Self::COLS] * rhs.values[i]
                    + self.values[j * Self::COLS + 1] * rhs.values[i + Self::COLS]
                    + self.values[j * Self::COLS + 2] * rhs.values[i + Self::COLS * 2]
                    + self.values[j * Self::COLS + 3] * rhs.values[i + Self::COLS * 3];
            }
        }

        Self { values }
    }
}

impl<T> MulAssign<Self> for Matrix4<T>
where
    T: Copy + Zero + Add<Output = T> + Mul<Output = T>,
{
    fn mul_assign(&mut self, rhs: Self) {
        self.values = (*self * rhs).values;
    }
}

impl<T> Index<usize> for Matrix4<T> {
    type Output = [T];

    fn index(&self, index: usize) -> &Self::Output {
        &self.values[index * Self::ROWS..index * Self::ROWS + Self::COLS]
    }
}

impl<T> IndexMut<usize> for Matrix4<T> {
    fn index_mut(&mut self, index: usize) -> &mut Self::Output {
        &mut self.values[index * Self::ROWS..index * Self::ROWS + Self::COLS]
    }
}

impl<T> From<Matrix4<T>> for [[T; 4]; 4]
where
    T: Copy,
{
    fn from(matrix: Matrix4<T>) -> Self {
        [
            [
                matrix.values[0],
                matrix.values[4],
                matrix.values[8],
                matrix.values[12],
            ],
            [
                matrix.values[1],
                matrix.values[5],
                matrix.values[9],
                matrix.values[13],
            ],
            [
                matrix.values[2],
                matrix.values[6],
                matrix.values[10],
                matrix.values[14],
            ],
            [
                matrix.values[3],
                matrix.values[7],
                matrix.values[11],
                matrix.values[15],
            ],
        ]
    }
}

pub trait Identity {
    fn identity() -> Self;
}

#[rustfmt::skip]
impl<T> Identity for Matrix4<T>
    where T: One + Zero {
    fn identity() -> Self {
        Self {
            values: [
                T::one(), T::zero(), T::zero(), T::zero(),
                T::zero(), T::one(), T::zero(), T::zero(),
                T::zero(), T::zero(), T::one(), T::zero(),
                T::zero(), T::zero(), T::zero(), T::one()
            ]
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn identity() {
        let m = Matrix4::<i32>::identity();

        for i in 0..4 {
            for j in 0..4 {
                if i == j {
                    assert_eq!(m[i][j], 1);
                } else {
                    assert_eq!(m[i][j], 0);
                }
            }
        }
    }

    #[test]
    fn index_mut() {
        let mut m = Matrix4::<i32>::identity();
        m[3][2] = 5;

        assert_eq!(m[3][2], 5);
    }

    #[test]
    fn index() {
        let m = Matrix4::<i32>::identity();

        assert_eq!(m[0][0], 1);
        assert_eq!(m[0][1], 0);
    }

    #[rustfmt::skip]
    #[test]
    fn mul() {
        let a = Matrix4::<i32>::with_values([
            1, 2, 3, 4,
            5, 6, 7, 8,
            9, 39, 11, 12,
            13, 14, 15, 16
        ]);
        let b = Matrix4::<i32>::with_values([
            17, 18, 19, 20,
            21, 22, 23, 24,
            25, 26, 27, 28,
            29, 30, 31, 32
        ]);

        let result = a * b;

        assert_eq!(result[0][0], 250);
        assert_eq!(result[0][1], 260);
        assert_eq!(result[0][2], 270);
        assert_eq!(result[0][3], 280);
        assert_eq!(result[1][0], 618);
        assert_eq!(result[1][1], 644);
        assert_eq!(result[1][2], 670);
        assert_eq!(result[1][3], 696);
        assert_eq!(result[2][0], 1595);
        assert_eq!(result[2][1], 1666);
        assert_eq!(result[2][2], 1737);
        assert_eq!(result[2][3], 1808);
        assert_eq!(result[3][0], 1354);
        assert_eq!(result[3][1], 1412);
        assert_eq!(result[3][2], 1470);
        assert_eq!(result[3][3], 1528);
    }

    #[rustfmt::skip]
    #[test]
    fn mul_assign() {
        let mut a = Matrix4::<i32>::with_values([
            1, 2, 3, 4,
            5, 6, 7, 8,
            9, 39, 11, 12,
            13, 14, 15, 16
        ]);
        let b = Matrix4::<i32>::with_values([
            17, 18, 19, 20,
            21, 22, 23, 24,
            25, 26, 27, 28,
            29, 30, 31, 32
        ]);

        a *= b;

        assert_eq!(a[0][0], 250);
        assert_eq!(a[0][1], 260);
        assert_eq!(a[0][2], 270);
        assert_eq!(a[0][3], 280);
        assert_eq!(a[1][0], 618);
        assert_eq!(a[1][1], 644);
        assert_eq!(a[1][2], 670);
        assert_eq!(a[1][3], 696);
        assert_eq!(a[2][0], 1595);
        assert_eq!(a[2][1], 1666);
        assert_eq!(a[2][2], 1737);
        assert_eq!(a[2][3], 1808);
        assert_eq!(a[3][0], 1354);
        assert_eq!(a[3][1], 1412);
        assert_eq!(a[3][2], 1470);
        assert_eq!(a[3][3], 1528);
    }

    #[rustfmt::skip]
    #[test]
    fn try_inverse() {
        let a = Matrix4f::with_values([
            1.0, 0.0, 0.0, 1.0,
            0.0, 2.0, 1.0, 2.0,
            2.0, 1.0, 0.0, 1.0,
            2.0, 0.0, 1.0, 4.0,
        ]);

        let inverse = a.try_inverse().unwrap();

        assert_float_absolute_eq!(inverse[0][0], -2.0, 0.1);
        assert_float_absolute_eq!(inverse[0][1], -0.5, 0.1);
        assert_float_absolute_eq!(inverse[0][2], 1.0, 0.1);
        assert_float_absolute_eq!(inverse[0][3], 0.5, 0.1);
        assert_float_absolute_eq!(inverse[1][0], 1.0, 0.1);
        assert_float_absolute_eq!(inverse[1][1], 0.5, 0.1);
        assert_float_absolute_eq!(inverse[1][2], 0.0, 0.1);
        assert_float_absolute_eq!(inverse[1][3], -0.5, 0.1);
        assert_float_absolute_eq!(inverse[2][0], -8.0, 0.1);
        assert_float_absolute_eq!(inverse[2][1], -1.0, 0.1);
        assert_float_absolute_eq!(inverse[2][2], 2.0, 0.1);
        assert_float_absolute_eq!(inverse[2][3], 2.0, 0.1);
        assert_float_absolute_eq!(inverse[3][0], 3.0, 0.1);
        assert_float_absolute_eq!(inverse[3][1], 0.5, 0.1);
        assert_float_absolute_eq!(inverse[3][2], -1.0, 0.1);
        assert_float_absolute_eq!(inverse[3][3], -0.5, 0.1);
    }
}
