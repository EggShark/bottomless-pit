use crate::vectors::Vec2;
use crate::vertex::Vertex;

pub struct Rectangle {
    points: [Vertex; 4],
}

impl Rectangle {
    pub fn new(pos: Vec2<f32>, size: Vec2<f32>, colour: [f32; 4]) -> Self {
        let pos = pos.to_raw();
        let size = size.to_raw();
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
        size: Vec2<f32>,
        colour: [f32; 4],
        screen_size: Vec2<u32>,
    ) -> Self {
        let pos = pos.to_raw();
        let size = size.to_raw();

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
        size: Vec2<f32>,
        colour: [f32; 4],
        screen_size: Vec2<u32>,
        uv_pos: Vec2<f32>,
        uv_size: Vec2<f32>,
    ) -> Self {
        let pos = pos.to_raw();
        let size = size.to_raw();
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
        size: Vec2<f32>,
        colour: [f32; 4],
        screen_size: Vec2<u32>,
        rotation: f32,
    ) -> Self {
        let pos = pos.to_raw();
        let size = size.to_raw();

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
        size: Vec2<f32>,
        colour: [f32; 4],
        screen_size: Vec2<u32>,
        rotation: f32,
        uv_pos: Vec2<f32>,
        uv_size: Vec2<f32>,
    ) -> Self {
        let pos = pos.to_raw();
        let size = size.to_raw();
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

    pub fn new_ex(
        pos: Vec2<f32>,
        size: Vec2<f32>,
        colour: [f32; 4],
        rotation: f32,
        uv_pos: Vec2<f32>,
        uv_size: Vec2<f32>,
    ) -> Self {
        let pos = pos.to_raw();
        let size = size.to_raw();
        let uv_pos = uv_pos.to_raw();

        let center = Vec2{x: pos[0] + size[0]/2.0, y: pos[1] + size[1]/2.0};
        let points = [
            Vertex::from_2d(pos, uv_pos, colour)
                .rotate(rotation, center),
            Vertex::from_2d(
                [pos[0] + size[0], pos[1]],
                [uv_pos[0] + uv_size.x, uv_pos[1]],
                colour,
            ).rotate(rotation, center),
            Vertex::from_2d(
                [pos[0] + size[0], pos[1] + size[1]],
                [uv_pos[0] + uv_size.x, uv_pos[1] + uv_size.y],
                colour,
            ).rotate(rotation, center),
            Vertex::from_2d(
                [pos[0], pos[1] + size[1]],
                [uv_pos[0], uv_pos[1] + uv_size.y],
                colour,
            ).rotate(rotation, center),
        ];
        
        Self { points }
    }

    pub fn from_pixels_custom(points: [Vec2<f32>; 4], uvs: [Vec2<f32>; 4], rotation: f32, colour: [f32; 4], screen_size: Vec2<u32>) -> Self {
        let center = get_center_of_four_points(points);

        let points = [
            Vertex::from_2d(points[0].to_raw(), uvs[0].to_raw(), colour)
                .rotate(rotation, center)
                .pixels_to_screenspace(screen_size),
            Vertex::from_2d(points[1].to_raw(), uvs[1].to_raw(), colour)
                .rotate(rotation, center)
                .pixels_to_screenspace(screen_size),
            Vertex::from_2d(points[2].to_raw(), uvs[2].to_raw(), colour)
                .rotate(rotation, center)
                .pixels_to_screenspace(screen_size),
            Vertex::from_2d(points[3].to_raw(), uvs[3].to_raw(), colour)
                .rotate(rotation, center)
                .pixels_to_screenspace(screen_size),
        ];
        
        Self { points }
    }

    pub fn into_vertices(self) -> [Vertex; 4] {
        self.points
    }
}

fn get_center_of_four_points(points: [Vec2<f32>; 4]) -> Vec2<f32> {
    let tri1_centroid_x = (points[0].x + points[1].x + points[3].x) / 3.0;
    let tri1_centroid_y = (points[0].y  + points[1].y + points[3].y) / 3.0;

    let tri2_centriod_x = (points[1].x + points[2].x  + points[3].x) / 3.0;
    let tri2_centroid_y = (points[1].y + points[2].y + points[3].y) / 3.0;

    let mid_point_x = (tri1_centroid_x + tri2_centriod_x) / 2.0;
    let mid_point_y = (tri1_centroid_y + tri2_centroid_y) / 2.0;

    Vec2{x: mid_point_x, y: mid_point_y}
}