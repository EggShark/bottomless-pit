use wgpu::util::DeviceExt;

pub struct Line {
    start: LineVertex,
    end: LineVertex,
    pub(crate) buffer: wgpu::Buffer,
}

impl Line {
    pub fn new(start: [f32; 2], end: [f32; 2], colour: [f32; 4], device: &wgpu::Device) -> Self {
        let start = LineVertex::new(start, colour);
        let end = LineVertex::new(end, colour);
        let buffer = Self::create_vertex_buffer(device, &[start, end]);

        Self {
            start,
            end,
            buffer,
        }
    }

    fn create_vertex_buffer(device: &wgpu::Device, points: &[LineVertex]) -> wgpu::Buffer {
        let vertex_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor{
            label: Some("Vertex Buffer"),
            contents: bytemuck::cast_slice(&points),
            usage: wgpu::BufferUsages::VERTEX,
        });

        vertex_buffer
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

pub trait DrawLines<'a> {
    fn draw_line(&mut self, rect: &'a Line, camera_bind_group: &'a wgpu::BindGroup);
}

impl<'a, 'b> DrawLines<'b> for wgpu::RenderPass<'a> where 'b: 'a, {
    fn draw_line(&mut self, line: &'a Line, camera_bind_group: &'a wgpu::BindGroup) {
        self.set_vertex_buffer(0, line.buffer.slice(..));
        self.set_bind_group(0, camera_bind_group, &[]);
        self.draw(0..2, 0..1);
    }
}