use std::fmt::{Display, Formatter};
use std::ops::{Add, AddAssign, Div, DivAssign, Mul, MulAssign, Neg, Sub, SubAssign};

use crate::number_traits::{Float, Zero};

pub type Vector3f = Vector3<f32>;
pub type Vector4f = Vector4<f32>;

macro_rules! struct_vec {
    ($name:ident : $display_fmt:literal, ($($dim:ident : $TY:ty => $idx:tt,)*)) => {
        #[must_use]
        #[derive(Clone, Copy, PartialEq, Eq, Debug)]
        pub struct $name<T = f32> {
            $(pub $dim: T,)*
        }

        impl<T> $name<T> {
            pub fn new($($dim: T),*) -> Self {
                Self {
                    $($dim),*
                }
            }
        }

        impl<T> $name<T>
        where T: Zero + Float {
            pub fn norm(&self) -> T {
                let mut norm = T::zero();
                $(norm += self.$dim * self.$dim;)*
                norm.sqrt()
            }

            pub fn normalize(&mut self) {
                let norm = self.norm();
                $(self.$dim /= norm;)*
            }

            pub fn normalized(&self) -> Self {
                let mut normalized = self.clone();
                normalized.normalize();
                normalized
            }
        }

        impl<T> Default for $name<T>
        where T: Zero {
            fn default() -> Self {
                Self {
                    $($dim: T::zero(),)*
                }
            }
        }

        impl<T> Add for $name<T>
        where
            T: Copy + Add<Output = T>, {
            type Output = Self;

            fn add(self, rhs: Self) -> Self::Output {
                Self {
                    $($dim: self.$dim + rhs.$dim),*
                }
            }
        }

        impl<T> AddAssign for $name<T>
        where
            T: Copy + Add<Output = T>, {
            fn add_assign(&mut self, rhs: Self) {
                *self = Self {
                    $($dim: self.$dim + rhs.$dim),*
                }
            }
        }

        impl<T> Sub for $name<T>
        where
            T: Copy + Sub<Output = T>, {
            type Output = Self;

            fn sub(self, rhs: Self) -> Self::Output {
                Self {
                    $($dim: self.$dim - rhs.$dim),*
                }
            }
        }

        impl<T> SubAssign for $name<T>
        where
            T: Copy + Sub<Output = T>, {
            fn sub_assign(&mut self, rhs: Self) {
                *self = Self {
                    $($dim: self.$dim - rhs.$dim),*
                }
            }
        }

        impl<T> Mul<T> for $name<T>
        where
            T: Copy + Mul<Output = T>, {
            type Output = Self;

            fn mul(self, rhs: T) -> Self::Output {
                Self {
                    $($dim: self.$dim * rhs),*
                }
            }
        }

        impl<T> MulAssign<T> for $name<T>
        where
            T: Copy + Mul<Output = T>, {
            fn mul_assign(&mut self, rhs: T) {
                *self = Self {
                    $($dim: self.$dim * rhs),*
                }
            }
        }

        impl<T> Div<T> for $name<T>
        where
            T: Copy + Div<Output = T>, {
            type Output = Self;

            fn div(self, rhs: T) -> Self::Output {
                Self {
                    $($dim: self.$dim / rhs),*
                }
            }
        }

        impl<T> DivAssign<T> for $name<T>
        where
            T: Copy + Div<Output = T>, {
            fn div_assign(&mut self, rhs: T) {
                *self = Self {
                    $($dim: self.$dim / rhs),*
                }
            }
        }

        impl<T> Neg for $name<T>
        where
            T: Copy + Neg<Output = T>,
        {
            type Output = Self;

            fn neg(self) -> Self::Output {
                Self {
                    $($dim: -self.$dim),*
                }
            }
        }

        impl<T> Display for $name<T>
        where
            T: Display,
        {
            fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
                write!(f, $display_fmt, $(self.$dim),*)
            }
        }

        impl<T> From<($($TY),*)> for $name<T>
        where
            T: Copy {
            fn from(tuple: ($($TY),*)) -> Self {
                Self {
                    $($dim: tuple.$idx),*
                }
            }
        }

        impl<T> From<$name<T>> for ($($TY),*)
        where
            T: Copy,
        {
            fn from(vector: $name<T>) -> Self {
                ($(vector.$dim),*)
            }
        }
    };
}

struct_vec!(Vector2: "({}, {})", (x: T => 0, y: T => 1,));
struct_vec!(Vector3: "({}, {}, {})", (x: T => 0, y: T => 1, z: T => 2,));
struct_vec!(Vector4: "({}, {}, {}, {})", (x: T => 0, y: T => 1, z: T => 2, w: T => 3,));

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn vector3_new() {
        let v = Vector3::new(1, 2, 3);

        assert_eq!(v.x, 1);
        assert_eq!(v.y, 2);
        assert_eq!(v.z, 3);
    }

    #[test]
    fn add() {
        let a = Vector3::new(1, 2, 3);
        let b = Vector3::new(4, 5, 6);

        let result = a + b;

        assert_eq!(result.x, 5);
        assert_eq!(result.y, 7);
        assert_eq!(result.z, 9);
    }

    #[test]
    fn add_assign() {
        let mut a = Vector3::new(1, 2, 3);
        let b = Vector3::new(4, 5, 6);

        a += b;

        assert_eq!(a.x, 5);
        assert_eq!(a.y, 7);
        assert_eq!(a.z, 9);
    }

    #[test]
    fn sub() {
        let a = Vector3::new(1, 2, 3);
        let b = Vector3::new(4, 3, 2);

        let result = a - b;

        assert_eq!(result.x, -3);
        assert_eq!(result.y, -1);
        assert_eq!(result.z, 1);
    }

    #[test]
    fn sub_assign() {
        let mut a = Vector3::new(1, 2, 3);
        let b = Vector3::new(4, 3, 2);

        a -= b;

        assert_eq!(a.x, -3);
        assert_eq!(a.y, -1);
        assert_eq!(a.z, 1);
    }

    #[test]
    fn mul_scalar() {
        let a = Vector3::new(1, 2, 3);
        let b = 5;

        let result = a * b;

        assert_eq!(result.x, 5);
        assert_eq!(result.y, 10);
        assert_eq!(result.z, 15);
    }

    #[test]
    fn mul_assign_scalar() {
        let mut vec = Vector3::new(1, 2, 3);
        let scalar = 5;

        vec *= scalar;

        assert_eq!(vec.x, 5);
        assert_eq!(vec.y, 10);
        assert_eq!(vec.z, 15);
    }

    #[test]
    fn div_scalar() {
        let a = Vector3::new(5, 10, 15);
        let b = 5;

        let result = a / b;

        assert_eq!(result.x, 1);
        assert_eq!(result.y, 2);
        assert_eq!(result.z, 3);
    }

    #[test]
    fn div_assign_scalar() {
        let mut vec = Vector3::new(5, 10, 15);
        let scalar = 5;

        vec /= scalar;

        assert_eq!(vec.x, 1);
        assert_eq!(vec.y, 2);
        assert_eq!(vec.z, 3);
    }

    #[test]
    fn neg() {
        let a = Vector3::new(1, 2, 3);

        let result = -a;

        assert_eq!(result.x, -1);
        assert_eq!(result.y, -2);
        assert_eq!(result.z, -3);
    }

    #[test]
    fn display() {
        let result = format!("{}", Vector3::new(1, 2, 3));
        assert_eq!("(1, 2, 3)", &result);
    }

    #[test]
    fn norm() {
        let vector = Vector3::new(1.0, 2.0, 3.0);
        assert_float_absolute_eq!(vector.norm(), 3.74, 0.01);
    }

    #[test]
    fn normalize() {
        let mut vector = Vector3::new(1.0, 2.0, 3.0);

        vector.normalize();

        assert_float_absolute_eq!(vector.x, 0.26, 0.01);
        assert_float_absolute_eq!(vector.y, 0.53, 0.01);
        assert_float_absolute_eq!(vector.z, 0.80, 0.01);
    }

    #[test]
    fn normalized() {
        let vector = Vector3::new(1.0, 2.0, 3.0);

        let normalized = vector.normalized();

        assert_float_absolute_eq!(normalized.x, 0.26, 0.01);
        assert_float_absolute_eq!(normalized.y, 0.53, 0.01);
        assert_float_absolute_eq!(normalized.z, 0.80, 0.01);
    }

    #[test]
    fn default() {
        let vector = Vector4::<f32>::default();

        assert_float_absolute_eq!(vector.x, 0.0, 0.0);
        assert_float_absolute_eq!(vector.y, 0.0, 0.0);
        assert_float_absolute_eq!(vector.z, 0.0, 0.0);
        assert_float_absolute_eq!(vector.w, 0.0, 0.0);
    }

    #[test]
    fn from_tuple() {
        let tuple = (0, 1, 2, 3);
        let v = Vector4::from(tuple);

        assert_eq!(v.x, 0);
        assert_eq!(v.y, 1);
        assert_eq!(v.z, 2);
        assert_eq!(v.w, 3);
    }

    #[test]
    fn into_tuple() {
        let v = Vector4::new(0, 1, 2, 3);
        let tuple: (i32, i32, i32, i32) = v.into();

        assert_eq!(tuple.0, 0);
        assert_eq!(tuple.1, 1);
        assert_eq!(tuple.2, 2);
        assert_eq!(tuple.3, 3);
    }
}
