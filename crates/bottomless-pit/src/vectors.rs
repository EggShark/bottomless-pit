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