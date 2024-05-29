//! contains several functions that help with doing matrix arithmetic
use crate::vectors::Vec2;

/// Helper function to normalize 2D points
pub fn normalize_points<T: std::ops::Div<Output = T>>(point: Vec2<T>, size: Vec2<T>) -> Vec2<T> {
    let x = point.x / size.x;
    let y = point.y / size.y;
    Vec2 { x, y }
}

/// Helper function that turns pixels into wgsl screen space
pub fn pixels_to_screenspace(mut point: Vec2<f32>, screen_size: Vec2<u32>) -> Vec2<f32> {
    let width = screen_size.x as f32;
    let height = screen_size.y as f32;
    point.x = (2.0 * point.x / width) - 1.0;
    point.y = ((2.0 * point.y / height) - 1.0) * -1.0;
    
    point
}