use crate::Vertex;
use crate::vertex::vert_pixels_to_screenspace;
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

    pub fn from_pixels(pos: Vec2<f32>, size: [f32; 2], colour: [f32; 4], screen_size: Vec2<u32>) -> Self {
        let pos = pos.to_raw();
        let points = [
            vert_pixels_to_screenspace(Vertex::from_2d(pos, [0.0, 0.0], colour), screen_size), 
            vert_pixels_to_screenspace(Vertex::from_2d([pos[0] + size[0], pos[1]], [1.0, 0.0], colour), screen_size),
            vert_pixels_to_screenspace(Vertex::from_2d([pos[0] + size[0], pos[1] + size[1]], [1.0, 1.0], colour), screen_size),
            vert_pixels_to_screenspace(Vertex::from_2d([pos[0], pos[1] + size[1]], [0.0, 1.0], colour), screen_size),
        ];

        Self { 
            points
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

    pub fn get_vertices(&self) -> [Vertex; 4] {
        self.points
    }
}