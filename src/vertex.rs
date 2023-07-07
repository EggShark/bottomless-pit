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

    pub(crate) fn pixels_to_screenspace(mut self, screen_size: Vec2<u32>) -> Self {
        let width = screen_size.x as f32;
        let height = screen_size.y as f32;
        self.position[0] = (2.0 * self.position[0] / width) - 1.0;
        self.position[1] = ((2.0 * self.position[1] / height) - 1.0) * -1.0;
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

    pub(crate) fn new(pos: [f32; 2], colour: [f32; 4]) -> Self {
        Self { pos, colour }
    }

    pub(crate) fn pixels_to_screenspace(mut self, screen_size: Vec2<u32>) -> Self {
        let width = screen_size.x as f32;
        let height = screen_size.y as f32;
        self.pos[0] = (2.0 * self.pos[0] / width) - 1.0;
        self.pos[1] = ((2.0 * self.pos[1] / height) - 1.0) * -1.0;
        self
    }
}