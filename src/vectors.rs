//! Generic implmentation of 2D Vectors
use std::ops::{Add, AddAssign, Div, DivAssign, Mul, MulAssign, Sub, SubAssign};

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
        Vec2 { x: $x, y: $y }
    };

    ($x:expr) => {
        Vec2 { x: $x, y: $x }
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

macro_rules! vec2_math_impl {
    ($type_one:ident) => {
        impl Add<Vec2<$type_one>> for $type_one {
            type Output = Vec2<$type_one>;
            fn add(self, rhs: Vec2<$type_one>) -> Self::Output {
                Vec2 {
                    x: self + rhs.x,
                    y: self + rhs.y,
                }
            }
        }
        impl Add<$type_one> for Vec2<$type_one> {
            type Output = Vec2<$type_one>;
            fn add(self, rhs: $type_one) -> Self::Output {
                Vec2 {
                    x: self.x + rhs,
                    y: self.y + rhs,
                }
            }
        }
        impl Sub<Vec2<$type_one>> for $type_one {
            type Output = Vec2<$type_one>;
            fn sub(self, rhs: Vec2<$type_one>) -> Self::Output {
                Vec2 {
                    x: self - rhs.x,
                    y: self - rhs.y,
                }
            }
        }
        impl Sub<$type_one> for Vec2<$type_one> {
            type Output = Vec2<$type_one>;
            fn sub(self, rhs: $type_one) -> Self::Output {
                Vec2 {
                    x: self.x - rhs,
                    y: self.y - rhs,
                }
            }
        }
        impl Mul<Vec2<$type_one>> for $type_one {
            type Output = Vec2<$type_one>;
            fn mul(self, rhs: Vec2<$type_one>) -> Self::Output {
                Vec2 {
                    x: self * rhs.x,
                    y: self * rhs.y,
                }
            }
        }
        impl Mul<$type_one> for Vec2<$type_one> {
            type Output = Vec2<$type_one>;
            fn mul(self, rhs: $type_one) -> Self::Output {
                Vec2 {
                    x: self.x * rhs,
                    y: self.y * rhs,
                }
            }
        }
        impl Div<Vec2<$type_one>> for $type_one {
            type Output = Vec2<$type_one>;
            fn div(self, rhs: Vec2<$type_one>) -> Self::Output {
                Vec2 {
                    x: self / rhs.x,
                    y: self / rhs.y,
                }
            }
        }
        impl Div<$type_one> for Vec2<$type_one> {
            type Output = Vec2<$type_one>;
            fn div(self, rhs: $type_one) -> Self::Output {
                Vec2 {
                    x: self.x / rhs,
                    y: self.y / rhs,
                }
            }
        }
    };
}

vec2_math_impl!(f32);
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
    pub fn scale(self, number: T) -> Vec2<T> {
        Vec2 {
            x: self.x * number,
            y: self.y * number,
        }
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

impl From<Vec2<f32>> for glam::Vec2 {
    fn from(val: Vec2<f32>) -> Self {
        Self { x: val.x, y: val.y }
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

impl<T: Mul<Output = T>> Mul for Vec2<T> {
    type Output = Vec2<T>;
    fn mul(self, rhs: Self) -> Self::Output {
        Self {
            x: self.x * rhs.x,
            y: self.y * rhs.y,
        }
    }
}

impl<T: Div<Output = T>> Div for Vec2<T> {
    type Output = Vec2<T>;
    fn div(self, rhs: Self) -> Self::Output {
        Self {
            x: self.x / rhs.x,
            y: self.y / rhs.y,
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

impl<T: DivAssign> DivAssign for Vec2<T> {
    fn div_assign(&mut self, rhs: Self) {
        self.x /= rhs.x;
        self.y /= rhs.y;
    }
}

impl<T: MulAssign> MulAssign for Vec2<T> {
    fn mul_assign(&mut self, rhs: Self) {
        self.x *= rhs.x;
        self.y *= rhs.y;
    }
}

#[cfg(feature = "mint")]
impl<T> From<mint::Vector2<T>> for Vec2<T>{
    fn from(v: mint::Vector2<T>) -> Self {
        Self::new(v.x, v.y)
    }
}

#[cfg(feature = "mint")]
impl<T> From<Vec2<T>> for mint::Vector2<T> {
    fn from(v: Vec2<T>) -> Self {
        mint::Vector2::<T> {
            x: v.x,
            y: v.y,
        }
    }
}
