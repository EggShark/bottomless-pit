use crate::render::Renderer;
use crate::vertex::Vertex;
use crate::engine_handle::WgpuClump;
use crate::vectors::Vec2;
use crate::colour::Colour;
use crate::rect::Rectangle;

// potentially just store ids to a hashmap to 
// avoid reproductions
pub struct Material {
    pipeline_id: wgpu::Id<wgpu::RenderPipeline>,
    vertex_buffer: wgpu::Buffer,
    /// counts the bytes of vertex not the actual number
    pub(crate) vertex_size: u64,
    pub(crate) vertex_count: u64,
    index_buffer: wgpu::Buffer,
    /// counts the bytes of the index no the actual number
    pub(crate) index_count: u64,
    pub(crate) index_size: u64,
    texture_id: wgpu::Id<wgpu::BindGroup>,
}

impl Material {
    pub(crate) fn new(device: &wgpu::Device, pipe_id: wgpu::Id<wgpu::RenderPipeline>, texture_id: wgpu::Id<wgpu::BindGroup>) -> Self {
        let vertex_size = std::mem::size_of::<Vertex>() as u64;
        let index_size = std::mem::size_of::<u16>() as u64;
        // draw 100 verticies and indicies before you need to re-allocate I think this is reasonable
        // buffer is 3.2 killobytes with current config
        let vertex_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Vertex_Buffer"),
            size: vertex_size * 100,
            usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::COPY_SRC,
            mapped_at_creation: false,
        });

        // this is just 200 bytes pretty small
        let index_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Index_Buffer"),
            size: index_size * 100,
            usage: wgpu::BufferUsages::INDEX | wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::COPY_SRC,
            mapped_at_creation: false,
        });

        Self {
            pipeline_id: pipe_id,
            vertex_buffer,
            vertex_count: 0,
            vertex_size,
            index_buffer,
            index_count: 0,
            index_size,
            texture_id,
        }
    }

    pub(crate) fn add_rectangle(&mut self, position: Vec2<f32>, width: f32, hieght: f32, colour: Colour, window_size: Vec2<u32>, wgpu: &WgpuClump) {
        let verts =
            Rectangle::from_pixels(position, [width, hieght], colour.to_raw(), window_size).into_vertices();

        let max_verts = self.vertex_buffer.size();

        if self.vertex_count + (4 * self.vertex_size) > max_verts {
            self.grow_vertex_buffer(&wgpu);
        }

        let num_verts = self.get_vertex_number() as u16;
        let indicies = [
            num_verts, 1 + num_verts, 2 + num_verts,
            3 + num_verts, num_verts, 2 + num_verts,
        ];

        let max_indicies = self.index_buffer.size();
        if self.index_count + (6 * self.index_size) > max_indicies {
            self.grow_index_buffer(&wgpu);
        }

        wgpu.queue.write_buffer(
            &self.vertex_buffer,
            self.vertex_count,
            bytemuck::cast_slice(&verts),
        );
        wgpu.queue.write_buffer(
            &self.index_buffer,
            self.index_count,
            bytemuck::cast_slice(&indicies),
        );

        self.vertex_count += 4 * self.vertex_size;
        self.index_count += 6 * self.index_size;
    }

    pub(crate) fn get_ids(&self) -> (wgpu::Id<wgpu::RenderPipeline>, wgpu::Id<wgpu::BindGroup>) {
        (self.pipeline_id, self.texture_id)
    }

    /// Returns a refrence to the vertex and index buffer in that order.
    pub(crate) fn buffers(&self) -> (&wgpu::Buffer, &wgpu::Buffer) {
        (&self.vertex_buffer, &self.index_buffer)
    }

    pub fn get_vertex_number(&self) -> u64 {
        self.vertex_count / self.vertex_size
    }

    pub fn get_index_number(&self) -> u64 {
        self.index_count / self.index_size
    }

    fn grow_vertex_buffer(&mut self, wgpu: &WgpuClump) {
        let mut encoder = wgpu.device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
            label: Some("Material Buffer Grower"),
        });

        let new_size = self.vertex_buffer.size() * 2;
        println!("Growing vertex buffer to: {}", new_size);

        let new_buffer = wgpu.device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Vertex_Buffer"),
            size: new_size,
            usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::COPY_SRC,
            mapped_at_creation: false,
        });

        encoder.copy_buffer_to_buffer(
            &self.vertex_buffer,
            0,
            &new_buffer,
            0,
            self.vertex_buffer.size(),
        );

        wgpu.
            queue
            .submit(std::iter::once(encoder.finish()));

        self.vertex_buffer = new_buffer;
    }

    fn grow_index_buffer(&mut self, wgpu: &WgpuClump) {
        let mut encoder = wgpu.device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
            label: Some("Material Buffer Grower"),
        });

        let new_size = self.index_buffer.size() * 2;
        println!("growing index buffer to: {}", new_size);
        let new_buffer = wgpu.device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Vertex_Buffer"),
            size: new_size,
            usage: wgpu::BufferUsages::INDEX | wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::COPY_SRC,
            mapped_at_creation: false,
        });

        encoder.copy_buffer_to_buffer(
            &self.index_buffer,
            0,
            &new_buffer,
            0,
            self.index_buffer.size(),
        );

        wgpu.
            queue
            .submit(std::iter::once(encoder.finish()));

        self.index_buffer = new_buffer;
    }
}

struct MaterialBuilder {

}