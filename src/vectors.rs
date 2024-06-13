//! Generic implmentation of 2D Vectors
use std::ops::{Add, AddAssign, Mul, Sub, SubAssign};

use winit::dpi::Size;

/// A generic representation of 2D data
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct Vec2<T> {
    pub x: T,
    pub y: T,
}

impl<T> Vec2<T> {
    pub fn new(x: T, y: T) -> Self {
        Self { x, y }
    }

    pub fn to_raw(self) -> [T; 2] {
        [self.x, self.y]
    }
}

#[macro_export]
macro_rules! vec2 {
    ($x:expr, $y:expr) => {
        Vec2{x: $x, y: $y}
    };

    ($x:expr) => {
        Vec2{x: $x, y: $x}
    };
}

macro_rules! from_vec2_impl {
    ($type_one:ident, $type_two:ident) => {
        impl From<Vec2<$type_two>> for Vec2<$type_one> {
            fn from(value: Vec2<$type_two>) -> Self {
                Vec2 {
                    x: $type_one::from(value.x),
                    y: $type_one::from(value.y),
                }
            }
        }
    };
}

from_vec2_impl!(i128, i8);
from_vec2_impl!(i128, i16);
from_vec2_impl!(i128, i32);
from_vec2_impl!(i128, i64);
from_vec2_impl!(i128, u8);
from_vec2_impl!(i128, u16);
from_vec2_impl!(i128, u32);
from_vec2_impl!(i128, u64);
from_vec2_impl!(i64, i8);
from_vec2_impl!(i64, i16);
from_vec2_impl!(i64, i32);
from_vec2_impl!(i64, u8);
from_vec2_impl!(i64, u16);
from_vec2_impl!(i64, u32);
from_vec2_impl!(i32, i8);
from_vec2_impl!(i32, i16);
from_vec2_impl!(i32, u8);
from_vec2_impl!(i32, u16);
from_vec2_impl!(i16, i8);
from_vec2_impl!(i16, u8);
from_vec2_impl!(u128, u8);
from_vec2_impl!(u128, u16);
from_vec2_impl!(u128, u32);
from_vec2_impl!(u128, u64);
from_vec2_impl!(u64, u8);
from_vec2_impl!(u64, u16);
from_vec2_impl!(u64, u32);
from_vec2_impl!(u32, u8);
from_vec2_impl!(u32, u16);
from_vec2_impl!(u16, u8);
from_vec2_impl!(f64, f32);
from_vec2_impl!(f64, i32);
from_vec2_impl!(f64, i16);
from_vec2_impl!(f64, i8);
from_vec2_impl!(f64, u32);
from_vec2_impl!(f64, u16);
from_vec2_impl!(f64, u8);
from_vec2_impl!(f32, i16);
from_vec2_impl!(f32, i8);
from_vec2_impl!(f32, u16);
from_vec2_impl!(f32, u8);

impl<T: Mul<Output = T> + Copy> Vec2<T> {
    pub fn scale(self, number: T) -> Vec2<T>{
        Vec2{x: self.x * number, y: self.y * number}
    }
}

impl<T> From<Vec2<T>> for (T, T) {
    fn from(value: Vec2<T>) -> Self {
        (value.x, value.y)
    }
}

impl From<Vec2<u32>> for glyphon::Resolution {
    fn from(value: Vec2<u32>) -> Self {
        Self {
            width: value.x,
            height: value.y,
        }
    }
}

impl From<Size> for Vec2<u32> {
    fn from(value: Size) -> Self {
        match value {
            Size::Physical(s) => vec2!(s.width, s.height),
            Size::Logical(s) => vec2!(s.width as u32, s.height as u32),
        }
    }
}

impl<T> From<(T, T)> for Vec2<T> {
    fn from(value: (T, T)) -> Self {
        Vec2 {
            x: value.0,
            y: value.1,
        }
    }
}

impl<T> From<winit::dpi::PhysicalSize<T>> for Vec2<T> {
    fn from(value: winit::dpi::PhysicalSize<T>) -> Self {
        Vec2 {
            x: value.width,
            y: value.height,
        }
    }
}

impl From<glam::Vec2> for Vec2<f32> {
    fn from(value: glam::Vec2) -> Self {
        Self {
            x: value.x,
            y: value.y,
        }
    }
}

impl Into<glam::Vec2> for Vec2<f32> {
    fn into(self) -> glam::Vec2 {
        glam::Vec2 { x: self.x, y: self.y }
    }
}

impl<T: Add<Output = T>> Add for Vec2<T> {
    type Output = Vec2<T>;
    fn add(self, rhs: Self) -> Self::Output {
        Self {
            x: self.x + rhs.x,
            y: self.y + rhs.y,
        }
    }
}

impl<T: Sub<Output = T>> Sub for Vec2<T> {
    type Output = Vec2<T>;
    fn sub(self, rhs: Self) -> Self::Output {
        Self {
            x: self.x - rhs.x,
            y: self.y - rhs.y,
        }
    }
}

impl<T: AddAssign> AddAssign for Vec2<T> {
    fn add_assign(&mut self, rhs: Self) {
        self.x += rhs.x;
        self.y += rhs.y;
    }
}

impl<T: SubAssign> SubAssign for Vec2<T> {
    fn sub_assign(&mut self, rhs: Self) {
        self.x -= rhs.x;
        self.y -= rhs.y;
    }
}