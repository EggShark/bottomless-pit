use crate::vectors::Vec2;
use crate::vertex::Vertex;

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

        Self { points }
    }

    pub fn from_pixels(
        pos: Vec2<f32>,
        size: [f32; 2],
        colour: [f32; 4],
        screen_size: Vec2<u32>,
    ) -> Self {
        let pos = pos.to_raw();
        let points = [
            Vertex::from_2d(pos, [0.0, 0.0], colour).to_owned()
                .pixels_to_screenspace(screen_size),
            Vertex::from_2d([pos[0] + size[0], pos[1]], [1.0, 0.0], colour)
                .pixels_to_screenspace(screen_size),
            Vertex::from_2d([pos[0] + size[0], pos[1] + size[1]], [1.0, 1.0], colour)
                .pixels_to_screenspace(screen_size),
            Vertex::from_2d([pos[0], pos[1] + size[1]], [0.0, 1.0], colour)
                .pixels_to_screenspace(screen_size),
        ];

        Self { points }
    }

    pub fn from_pixels_with_uv(
        pos: Vec2<f32>,
        size: [f32; 2],
        colour: [f32; 4],
        screen_size: Vec2<u32>,
        uv_pos: Vec2<f32>,
        uv_size: Vec2<f32>,
    ) -> Self {
        let pos = pos.to_raw();
        let uv_pos = uv_pos.to_raw();
        let points = [
            Vertex::from_2d(pos, uv_pos, colour)
                .pixels_to_screenspace(screen_size),
            Vertex::from_2d(
                [pos[0] + size[0], pos[1]],
                [uv_pos[0] + uv_size.x, uv_pos[1]],
                colour,
            ).pixels_to_screenspace(screen_size),
            Vertex::from_2d(
                [pos[0] + size[0], pos[1] + size[1]],
                [uv_pos[0] + uv_size.x, uv_pos[1] + uv_size.y],
                colour,
            ).pixels_to_screenspace(screen_size),
            Vertex::from_2d(
                [pos[0], pos[1] + size[1]],
                [uv_pos[0], uv_pos[1] + uv_size.y],
                colour,
            ).pixels_to_screenspace(screen_size),
        ];
        Self { points }
    }

    pub fn from_pixels_with_rotation(
        pos: Vec2<f32>,
        size: [f32; 2],
        colour: [f32; 4],
        screen_size: Vec2<u32>,
        rotation: f32,
    ) -> Self {
        let pos = pos.to_raw();

        let center = Vec2{x: pos[0] + size[0]/2.0, y: pos[1] + size[1]/2.0};

        let points = [
            Vertex::from_2d(pos, [0.0, 0.0], colour)
                .rotate(rotation, center)
                .pixels_to_screenspace(screen_size),
            Vertex::from_2d([pos[0] + size[0], pos[1]], [1.0, 0.0], colour)
                .rotate(rotation, center)
                .pixels_to_screenspace(screen_size),
            Vertex::from_2d([pos[0] + size[0], pos[1] + size[1]], [1.0, 1.0], colour)
                .rotate(rotation, center)
                .pixels_to_screenspace(screen_size),
            Vertex::from_2d([pos[0], pos[1] + size[1]], [0.0, 1.0], colour)
                .rotate(rotation, center)
                .pixels_to_screenspace(screen_size),
        ];

        Self { points }
    }

    pub fn from_pixels_ex(
        pos: Vec2<f32>,
        size: [f32; 2],
        colour: [f32; 4],
        screen_size: Vec2<u32>,
        rotation: f32,
        uv_pos: Vec2<f32>,
        uv_size: Vec2<f32>,
    ) -> Self {
        let pos = pos.to_raw();
        let uv_pos = uv_pos.to_raw();

        let center = Vec2{x: pos[0] + size[0]/2.0, y: pos[1] + size[1]/2.0};
        let points = [
            Vertex::from_2d(pos, uv_pos, colour)
                .rotate(rotation, center)
                .pixels_to_screenspace(screen_size),
            Vertex::from_2d(
                [pos[0] + size[0], pos[1]],
                [uv_pos[0] + uv_size.x, uv_pos[1]],
                colour,
            ).rotate(rotation, center)
                .pixels_to_screenspace(screen_size),
            Vertex::from_2d(
                [pos[0] + size[0], pos[1] + size[1]],
                [uv_pos[0] + uv_size.x, uv_pos[1] + uv_size.y],
                colour,
            ).rotate(rotation, center)
                .pixels_to_screenspace(screen_size),
            Vertex::from_2d(
                [pos[0], pos[1] + size[1]],
                [uv_pos[0], uv_pos[1] + uv_size.y],
                colour,
            ).rotate(rotation, center)
                .pixels_to_screenspace(screen_size),
        ];
        
        Self { points }
    }

    pub fn get_vertices(&self) -> [Vertex; 4] {
        self.points
    }
}
