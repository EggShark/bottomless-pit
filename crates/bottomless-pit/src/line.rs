pub struct Line {
    pub start: LineVertex,
    pub end: LineVertex,
}

impl Line {
    pub fn new(start: [f32; 2], end: [f32; 2], colour: [f32; 4]) -> Self {
        let start = LineVertex::new(start, colour);
        let end = LineVertex::new(end, colour);

        Self {
            start,
            end,
        }
    }
}

#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
pub struct LineVertex {
    pub pos: [f32; 3],
    pub colour: [f32; 4],
}

impl LineVertex {
    pub fn desc<'a>() -> wgpu::VertexBufferLayout<'a> {
        wgpu::VertexBufferLayout{
            array_stride: std::mem::size_of::<LineVertex>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: &[
                wgpu::VertexAttribute {
                    offset: 0,
                    shader_location: 0,
                    format: wgpu::VertexFormat::Float32x3,
                },
                wgpu::VertexAttribute {
                    offset: std::mem::size_of::<[f32; 3]>() as wgpu::BufferAddress,
                    shader_location: 1,
                    format: wgpu::VertexFormat::Float32x4,
                },
            ]
        }
    }

    fn new(point: [f32; 2], colour: [f32; 4]) -> Self {
        let pos =  [point[0], point[1], 0.0];
        Self {
            pos,
            colour,
        }
    }
}