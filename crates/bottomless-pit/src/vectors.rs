use std::ops::{Add, Sub};

pub struct Vec2<T> {
    pub x: T,
    pub y: T,
}

pub struct Vec3<T> {
    pub x: T,
    pub y: T,
    pub z: T,
}

impl<T> Into<Vec2<T>> for (T, T) {
    fn into(self) -> Vec2<T> {
        Vec2{x: self.0, y: self.1}
    }
}

impl<T> Into<Vec2<T>> for winit::dpi::PhysicalSize<T> {
    fn into(self) -> Vec2<T> {
        Vec2{x: self.width , y: self.height}
    }
}

impl<T> Into<Vec2<T>> for cgmath::Vector2<T> {
    fn into(self) -> Vec2<T> {
        Vec2{x: self.x, y: self.y}
    }
}

impl<T> Into<Vec3<T>> for cgmath::Vector3<T> {
    fn into(self) -> Vec3<T> {
        Vec3{x: self.x, y: self.y, z: self.z}
    }
}

impl<T: Add::<Output = T>> Add for Vec2<T> {
    type Output = Vec2<T>;
    fn add(self, rhs: Self) -> Self::Output {
        Self{x: self.x + rhs.x, y: self.y + rhs.y}
    }
}

impl<T: Sub::<Output = T>> Sub for Vec2<T> {
    type Output = Vec2<T>;
    fn sub(self, rhs: Self) -> Self::Output {
        Self{x: self.x - rhs.x, y: self.y - rhs.y}
    }
}

impl<T: Add::<Output = T>> Add for Vec3<T> {
    type Output = Vec3<T>;
    fn add(self, rhs: Self) -> Self::Output {
        Self{x: self.x + rhs.x, y: self.y + rhs.y, z: self.z + rhs.z}
    }
}

impl<T: Sub::<Output = T>> Sub for Vec3<T> {
    type Output = Vec3<T>;
    fn sub(self, rhs: Self) -> Self::Output {
        Self{x: self.x - rhs.x, y: self.y - rhs.y, z: self.z - rhs.z}
    }
}

impl<T: std::fmt::Debug> std::fmt::Debug for Vec2<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Vec2{{x: {:?}, y: {:?}}}", self.x, self.y)
    }
}

impl<T: std::fmt::Debug> std::fmt::Debug for Vec3<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Vec3{{x: {:?}, y: {:?}, z: {:?}}}", self.x, self.y, self.z)
    }
}