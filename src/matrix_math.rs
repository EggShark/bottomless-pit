//! contains several functions that help with doing matrix arithmetic
use crate::vectors::Vec2;

/// Helper function to normalize 2D points
pub fn normalize_points<T: std::ops::Div<Output = T>>(point: Vec2<T>, size: Vec2<T>) -> Vec2<T> {
    let x = point.x / size.x;
    let y = point.y / size.y;
    Vec2 { x, y }
}