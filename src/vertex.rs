use crate::vectors::Vec2;

#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
pub struct Vertex {
    pub position: [f32; 2],
    pub tex_coords: [f32; 2],
    pub colour: [f32; 4],
}

impl Vertex {
    pub fn desc<'a>() -> wgpu::VertexBufferLayout<'a> {
        wgpu::VertexBufferLayout {
            array_stride: std::mem::size_of::<Vertex>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: &[
                wgpu::VertexAttribute {
                    offset: 0,
                    shader_location: 0,
                    format: wgpu::VertexFormat::Float32x2,
                },
                wgpu::VertexAttribute {
                    offset: std::mem::size_of::<[f32; 2]>() as wgpu::BufferAddress,
                    shader_location: 1,
                    format: wgpu::VertexFormat::Float32x2,
                },
                wgpu::VertexAttribute {
                    offset: std::mem::size_of::<[f32; 4]>() as wgpu::BufferAddress,
                    shader_location: 2,
                    format: wgpu::VertexFormat::Float32x4,
                },
            ],
        }
    }

    pub fn from_2d(position: [f32; 2], tex_coords: [f32; 2], colour: [f32; 4]) -> Self {
        Self {
            position,
            tex_coords,
            colour,
        }
    }

    // pub(crate) fn pixels_to_screenspace(mut self, screen_size: Vec2<u32>) -> Self {
    //     let width = screen_size.x as f32;
    //     let height = screen_size.y as f32;
    //     self.position[0] = 2.0 * self.position[0] / width - 1.0;
    //     self.position[1] = ((2.0 * self.position[1] / height) - 1.0) * -1.0;
    //     self
    // }

    pub(crate) fn screenspace_to_pixels(mut self, screen_size: Vec2<u32>) -> Self {
        let width = screen_size.x as f32;
        let height = screen_size.y as f32;

        self.position[0] = ((self.position[0] + 1.0) * width) / 2.0;
        self.position[1] = (((self.position[1] * -1.0) + 1.0) * height) / 2.0;

        self
    }

    pub(crate) fn rotate(mut self, rotation: f32, center: Vec2<f32>) -> Self {
        let rotaion_matrix = glam::Mat3::from_angle(rotation.to_radians());
        let translation_matrix = glam::Mat3::from_translation(center.into());
        let inverse_translation = translation_matrix.inverse();
        let combined = translation_matrix * rotaion_matrix * inverse_translation;

        let glam_2d = glam::vec2(self.position[0], self.position[1]);
        let out = combined.transform_point2(glam_2d);

        self.position[0] = out.x;
        self.position[1] = out.y;

        self
    }
}

#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
pub(crate) struct LineVertex {
    pub pos: [f32; 2],
    pub colour: [f32; 4],
}

impl LineVertex {
    pub fn desc<'a>() -> wgpu::VertexBufferLayout<'a> {
        wgpu::VertexBufferLayout {
            array_stride: std::mem::size_of::<LineVertex>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: &[
                wgpu::VertexAttribute {
                    offset: 0,
                    shader_location: 0,
                    format: wgpu::VertexFormat::Float32x2,
                },
                wgpu::VertexAttribute {
                    offset: std::mem::size_of::<[f32; 2]>() as wgpu::BufferAddress,
                    shader_location: 1,
                    format: wgpu::VertexFormat::Float32x4,
                },
            ],
        }
    }

    pub fn new(pos: [f32; 2], colour: [f32; 4]) -> Self {
        Self { pos, colour }
    }

    pub fn screenspace_to_pixels(mut self, screen_size: Vec2<u32>) -> Self {
        let width = screen_size.x as f32;
        let height = screen_size.y as f32;

        self.pos[0] = ((self.pos[0] + 1.0) * width) / 2.0;
        self.pos[1] = (((self.pos[1] * -1.0) + 1.0) * height) / 2.0;

        self
    }
}

pub(crate) fn new(pos: Vec2<f32>, size: Vec2<f32>, colour: [f32; 4], screen_size: Vec2<u32>) -> [Vertex; 4] {
    let pos = pos.to_raw();
    let size = size.to_raw();
    [
        Vertex::from_2d(pos, [0.0, 0.0], colour)
            .screenspace_to_pixels(screen_size),
        Vertex::from_2d([pos[0] + size[0], pos[1]], [1.0, 0.0], colour)
            .screenspace_to_pixels(screen_size),
        Vertex::from_2d([pos[0] + size[0], pos[1] - size[1]], [1.0, 1.0], colour)
            .screenspace_to_pixels(screen_size),
        Vertex::from_2d([pos[0], pos[1] - size[1]], [0.0, 1.0], colour)
            .screenspace_to_pixels(screen_size),
    ]
}

pub fn from_pixels(
    pos: Vec2<f32>,
    size: Vec2<f32>,
    colour: [f32; 4],
) -> [Vertex; 4] {
    let pos = pos.to_raw();
    let size = size.to_raw();

    [
        Vertex::from_2d(pos, [0.0, 0.0], colour),
        Vertex::from_2d([pos[0] + size[0], pos[1]], [1.0, 0.0], colour),
        Vertex::from_2d([pos[0] + size[0], pos[1] + size[1]], [1.0, 1.0], colour),
        Vertex::from_2d([pos[0], pos[1] + size[1]], [0.0, 1.0], colour)
    ]
}

pub fn from_pixels_with_uv(
    pos: Vec2<f32>,
    size: Vec2<f32>,
    colour: [f32; 4],
    uv_pos: Vec2<f32>,
    uv_size: Vec2<f32>,
) -> [Vertex; 4] {
    let pos = pos.to_raw();
    let size = size.to_raw();
    let uv_pos = uv_pos.to_raw();

    [
        Vertex::from_2d(pos, uv_pos, colour),
        Vertex::from_2d(
            [pos[0] + size[0], pos[1]],
            [uv_pos[0] + uv_size.x, uv_pos[1]],
            colour,
        ),
        Vertex::from_2d(
            [pos[0] + size[0], pos[1] + size[1]],
            [uv_pos[0] + uv_size.x, uv_pos[1] + uv_size.y],
            colour,
        ),
        Vertex::from_2d(
            [pos[0], pos[1] + size[1]],
            [uv_pos[0], uv_pos[1] + uv_size.y],
            colour,
        ),
    ]
}

pub(crate) fn from_pixels_with_rotation(
    pos: Vec2<f32>,
    size: Vec2<f32>,
    colour: [f32; 4],
    rotation: f32,
) -> [Vertex; 4] {
    let pos = pos.to_raw();
    let size = size.to_raw();

    let center = Vec2 {
        x: pos[0] + size[0] / 2.0,
        y: pos[1] + size[1] / 2.0,
    };

    [
        Vertex::from_2d(pos, [0.0, 0.0], colour)
            .rotate(rotation, center),
        Vertex::from_2d([pos[0] + size[0], pos[1]], [1.0, 0.0], colour)
            .rotate(rotation, center),
        Vertex::from_2d([pos[0] + size[0], pos[1] + size[1]], [1.0, 1.0], colour)
            .rotate(rotation, center),
        Vertex::from_2d([pos[0], pos[1] + size[1]], [0.0, 1.0], colour)
            .rotate(rotation, center),
    ]
}

pub(crate) fn from_pixels_ex(
    pos: Vec2<f32>,
    size: Vec2<f32>,
    colour: [f32; 4],
    rotation: f32,
    uv_pos: Vec2<f32>,
    uv_size: Vec2<f32>,
) -> [Vertex; 4] {
    let pos = pos.to_raw();
    let size = size.to_raw();
    let uv_pos = uv_pos.to_raw();

    let center = Vec2 {
        x: pos[0] + size[0] / 2.0,
        y: pos[1] + size[1] / 2.0,
    };
    [
        Vertex::from_2d(pos, uv_pos, colour)
            .rotate(rotation, center),
        Vertex::from_2d(
            [pos[0] + size[0], pos[1]],
            [uv_pos[0] + uv_size.x, uv_pos[1]],
            colour,
        )
        .rotate(rotation, center),
        Vertex::from_2d(
            [pos[0] + size[0], pos[1] + size[1]],
            [uv_pos[0] + uv_size.x, uv_pos[1] + uv_size.y],
            colour,
        )
        .rotate(rotation, center),
        Vertex::from_2d(
            [pos[0], pos[1] + size[1]],
            [uv_pos[0], uv_pos[1] + uv_size.y],
            colour,
        )
        .rotate(rotation, center),
    ]
}

// TODO: Calculate Screenspace to PIXELS
pub(crate) fn new_ex(
    pos: Vec2<f32>,
    size: Vec2<f32>,
    colour: [f32; 4],
    rotation: f32,
    uv_pos: Vec2<f32>,
    uv_size: Vec2<f32>,
) -> [Vertex; 4] {
    let pos = pos.to_raw();
    let size = size.to_raw();
    let uv_pos = uv_pos.to_raw();

    let center = Vec2 {
        x: pos[0] + size[0] / 2.0,
        y: pos[1] + size[1] / 2.0,
    };

    [
        Vertex::from_2d(pos, uv_pos, colour).rotate(rotation, center),
        Vertex::from_2d(
            [pos[0] + size[0], pos[1]],
            [uv_pos[0] + uv_size.x, uv_pos[1]],
            colour,
        )
        .rotate(rotation, center),
        Vertex::from_2d(
            [pos[0] + size[0], pos[1] + size[1]],
            [uv_pos[0] + uv_size.x, uv_pos[1] + uv_size.y],
            colour,
        )
        .rotate(rotation, center),
        Vertex::from_2d(
            [pos[0], pos[1] + size[1]],
            [uv_pos[0], uv_pos[1] + uv_size.y],
            colour,
        )
        .rotate(rotation, center),
    ]
}

pub(crate) fn from_pixels_custom(
    points: [Vec2<f32>; 4],
    uvs: [Vec2<f32>; 4],
    rotation: f32,
    colour: [f32; 4],
) -> [Vertex; 4] {
    let center = get_center_of_four_points(points);

    [
        Vertex::from_2d(points[0].to_raw(), uvs[0].to_raw(), colour)
            .rotate(rotation, center),
        Vertex::from_2d(points[1].to_raw(), uvs[1].to_raw(), colour)
            .rotate(rotation, center),
        Vertex::from_2d(points[2].to_raw(), uvs[2].to_raw(), colour)
            .rotate(rotation, center),
        Vertex::from_2d(points[3].to_raw(), uvs[3].to_raw(), colour)
            .rotate(rotation, center),
    ]
}

fn get_center_of_four_points(points: [Vec2<f32>; 4]) -> Vec2<f32> {
    let tri1_centroid_x = (points[0].x + points[1].x + points[3].x) / 3.0;
    let tri1_centroid_y = (points[0].y + points[1].y + points[3].y) / 3.0;

    let tri2_centriod_x = (points[1].x + points[2].x + points[3].x) / 3.0;
    let tri2_centroid_y = (points[1].y + points[2].y + points[3].y) / 3.0;

    let mid_point_x = (tri1_centroid_x + tri2_centriod_x) / 2.0;
    let mid_point_y = (tri1_centroid_y + tri2_centroid_y) / 2.0;

    Vec2 {
        x: mid_point_x,
        y: mid_point_y,
    }
}
