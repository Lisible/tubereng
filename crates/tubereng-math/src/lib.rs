#![warn(clippy::pedantic)]
#![allow(clippy::module_name_repetitions)]

#[macro_use]
extern crate assert_float_eq;

pub mod matrix;
mod number_traits;
pub mod quaternion;
pub mod vector;
