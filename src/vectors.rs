//! Generic implmentation of both 2D and 3D vectors

use std::ops::{Add, Sub};

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

impl<T> From<Vec2<T>> for (T, T) {
    fn from(value: Vec2<T>) -> Self {
        (value.x, value.y)
    }
}

impl From<Vec2<u32>> for glyphon::Resolution {
    fn from(value: Vec2<u32>) -> Self {
        Self {width: value.x, height: value.y}
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

impl<T> From<cgmath::Vector2<T>> for Vec2<T> {
    fn from(value: cgmath::Vector2<T>) -> Vec2<T> {
        Vec2 {
            x: value.x,
            y: value.y,
        }
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