use crate::{Vertex, cache::TextureIndex};
use crate::vectors::Vec2;

pub const RECT_INDICIES: &[u16] = &[
    0, 1, 2,
    3, 0, 2,
];

pub struct Rectangle {
    points: [Vertex; 4],
}

impl Rectangle {
    pub fn new(pos: Vec2<f32>, size: [f32; 2], colour: [f32; 4]) -> Self {
        let pos = pos.to_raw();
        let points = [
            Vertex::from_2d(pos, [0.0, 0.0], colour), 
            Vertex::from_2d([pos[0] + size[0], pos[1]], [1.0, 0.0], colour),
            Vertex::from_2d([pos[0] + size[0], pos[1] - size[1]], [1.0, 1.0], colour),
            Vertex::from_2d([pos[0], pos[1] - size[1]], [0.0, 1.0], colour),
        ];

        Self {
            points,
        }
    }

    pub fn from_points(points: [[f32; 2]; 4], colour: [f32; 4]) -> Self {
        let points = [
            Vertex::from_2d(points[0], [0.0, 0.0], colour),
            Vertex::from_2d(points[1], [1.0, 0.0], colour),
            Vertex::from_2d(points[2], [1.0, 1.0], colour),
            Vertex::from_2d(points[3], [0.0, 1.0], colour),
        ]; 

        Self {
            points,
        }
    }

    pub fn from_corners(corners: [[f32; 2]; 2], colour: [f32; 4]) -> Self {
        let width = corners[0][0] - corners[1][0];
        let height = corners[1][1] - corners[0][1];
        let pos = Vec2{x: corners[0][0], y: corners[0][1]};
        
        Self::new(pos, [width, height], colour)
    }

    pub fn get_width(&self) -> f32 {
        let left = self.points[0].get_pos_2d()[0];
        let right = self.points[1].get_pos_2d()[0];

        if left < 0.0 && right > 0.0 {
            left.abs() + right
        } else {
            (right - left).abs()
        }
    }

    pub fn get_height(&self) -> f32 {
        let top = self.points[0].get_pos_2d()[1];
        let bottom = self.points[2].get_pos_2d()[1];

        if top < 0.0 && bottom > 0.0 {
            top.abs() + bottom
        } else {
            (bottom - top).abs()
        }
    }

    pub fn get_vertices(&self) -> [Vertex; 4] {
        self.points
    }
}