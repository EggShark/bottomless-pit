//! contains several functions that help with doing matrix arithmetic
use crate::vectors::Vec2;

/// Helper function to normalize 2D points
pub fn normalize_points<T: std::ops::Div<Output = T>>(
    point: Vec2<T>,
    width: T,
    height: T,
) -> Vec2<T> {
    let x = point.x / width;
    let y = point.y / height;
    Vec2 { x, y }
}

#[rustfmt::skip]
/// Helper function to make a 2d rotation matrix
pub fn calculate_rotation_matrix(degree: f32) -> cgmath::Matrix4<f32> {
    let degree = degree.to_radians();
    cgmath::Matrix4::new(
        degree.cos(), -degree.sin(), 0.0, 0.0,
        degree.sin(), degree.cos(), 0.0, 0.0,
        0.0, 0.0, 1.0, 0.0,
        0.0, 0.0, 0.0, 1.0,
    )
}
