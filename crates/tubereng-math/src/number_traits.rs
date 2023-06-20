use std::fmt::Display;
use std::ops::{Add, AddAssign, Div, DivAssign, Mul, MulAssign, Neg, Sub, SubAssign};

pub trait Two {
    fn two() -> Self;
}

impl Two for i32 {
    fn two() -> Self {
        2
    }
}

impl Two for f32 {
    fn two() -> Self {
        2.0
    }
}

impl Two for f64 {
    fn two() -> Self {
        2.0
    }
}

pub trait One {
    fn one() -> Self;
}

impl One for i32 {
    fn one() -> Self {
        1
    }
}

impl One for f32 {
    fn one() -> Self {
        1.0
    }
}

impl One for f64 {
    fn one() -> Self {
        1.0
    }
}

pub trait Zero {
    fn zero() -> Self;
}

impl Zero for i32 {
    fn zero() -> Self {
        0
    }
}

impl Zero for f32 {
    fn zero() -> Self {
        0.0
    }
}

impl Zero for f64 {
    fn zero() -> Self {
        0.0
    }
}

pub trait IsZero {
    fn is_zero(&self) -> bool;
}

impl IsZero for i32 {
    fn is_zero(&self) -> bool {
        *self == 0
    }
}

impl IsZero for f32 {
    fn is_zero(&self) -> bool {
        self.abs() < 0.000_000_01
    }
}

impl IsZero for f64 {
    fn is_zero(&self) -> bool {
        self.abs() < 0.000_000_01
    }
}

pub trait Pi {
    fn pi() -> Self;
}

impl Pi for f32 {
    fn pi() -> Self {
        std::f32::consts::PI
    }
}

impl Pi for f64 {
    fn pi() -> Self {
        std::f64::consts::PI
    }
}

pub trait NumericOps:
    Sized
    + Add<Output = Self>
    + AddAssign
    + Sub<Output = Self>
    + SubAssign
    + Mul<Output = Self>
    + MulAssign
    + Div<Output = Self>
    + DivAssign
    + Neg<Output = Self>
{
}

impl NumericOps for i32 {}

impl NumericOps for f32 {}

impl NumericOps for f64 {}

pub trait Float: Display + Copy + Zero + One + Two + Pi + NumericOps {
    fn sin(self) -> Self;
    fn cos(self) -> Self;
    fn half(self) -> Self;
    fn squared(self) -> Self;
    fn sqrt(self) -> Self;
}

impl Float for f32 {
    fn sin(self) -> Self {
        self.sin()
    }

    fn cos(self) -> Self {
        self.cos()
    }
    fn half(self) -> Self {
        self * 0.5
    }

    fn squared(self) -> Self {
        self * self
    }

    fn sqrt(self) -> Self {
        self.sqrt()
    }
}

impl Float for f64 {
    fn sin(self) -> Self {
        self.sin()
    }

    fn cos(self) -> Self {
        self.cos()
    }
    fn half(self) -> Self {
        self * 0.5
    }

    fn squared(self) -> Self {
        self * self
    }

    fn sqrt(self) -> Self {
        self.sqrt()
    }
}
