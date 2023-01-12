use wgpu::util::DeviceExt;

use crate::{Vertex, Texture};

const RECT_INDICIES: &[u16] = &[
    0, 1, 2,
    3, 0, 2,
];

pub struct Rectangle {
    points: [Vertex; 4],
    vertex_buffer: wgpu::Buffer,
    index_buffer: wgpu::Buffer,
}

impl Rectangle {
    pub fn new(pos: [f32; 2], size: [f32; 2], colour: [f32; 4],device: &wgpu::Device) -> Self {
        let points = [
            Vertex::from_2d(pos, [0.0, 0.0], colour), 
            Vertex::from_2d([pos[0] + size[0], pos[1]], [1.0, 0.0], colour),
            Vertex::from_2d([pos[0] + size[0], pos[1] - size[1]], [1.0, 1.0], colour),
            Vertex::from_2d([pos[0], pos[1] - size[1]], [0.0, 1.0], colour),
        ];

        let vertex_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor{
            label: Some("Vertex Buffer"),
            contents: bytemuck::cast_slice(&points),
            usage: wgpu::BufferUsages::VERTEX,
        });

        let index_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor{
            label: Some("Vertex Buffer"),
            contents: bytemuck::cast_slice(RECT_INDICIES),
            usage: wgpu::BufferUsages::INDEX,
        });

        Self {
            points,
            vertex_buffer,
            index_buffer,
        }
    }
}

pub struct TexturedRect {
    texture: Texture,
    points: [Vertex; 4],
    vertex_buffer: wgpu::Buffer,
    index_buffer: wgpu::Buffer,
}

impl TexturedRect {
    pub fn new(texture: Texture, pos: [f32; 2], size: [f32; 2], device: &wgpu::Device) -> Self {
        let colour = [1.0, 1.0, 1.0, 1.0];
        let points = [
            Vertex::from_2d(pos, [0.0, 0.0], colour), 
            Vertex::from_2d([pos[0] + size[0], pos[1]], [1.0, 0.0], colour),
            Vertex::from_2d([pos[0] + size[0], pos[1] - size[1]], [1.0, 1.0], colour),
            Vertex::from_2d([pos[0], pos[1] - size[1]], [0.0, 1.0], colour),
        ];
        let vertex_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor{
            label: Some("Vertex Buffer"),
            contents: bytemuck::cast_slice(&points),
            usage: wgpu::BufferUsages::VERTEX,
        });

        let index_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor{
            label: Some("Vertex Buffer"),
            contents: bytemuck::cast_slice(RECT_INDICIES),
            usage: wgpu::BufferUsages::INDEX,
        });

        Self {
            texture,
            points,
            vertex_buffer,
            index_buffer,
        }
    }

    pub fn get_bind_group(&self) -> &wgpu::BindGroup {
        &self.texture.bind_group
    }

    pub fn get_bind_group_layout(&self) -> &wgpu::BindGroupLayout {
        &self.texture.bind_group_layout
    }
}

pub trait DrawRectangles<'a> {
    fn draw_rectangle(&mut self, rect: &'a Rectangle, white_pixel: &'a wgpu::BindGroup, camera_bind_group: &'a wgpu::BindGroup);
    fn draw_textured_rect(&mut self, rect: &'a TexturedRect, camera_bind_group: &'a wgpu::BindGroup);
}

impl<'a, 'b> DrawRectangles<'b> for wgpu::RenderPass<'a> where 'b: 'a, {
    fn draw_rectangle(&mut self, rect: &'a Rectangle, white_pixel: &'a wgpu::BindGroup, camera_bind_group: &'a wgpu::BindGroup) {
        self.set_vertex_buffer(0, rect.vertex_buffer.slice(..));
        self.set_index_buffer(rect.index_buffer.slice(..), wgpu::IndexFormat::Uint16);
        self.set_bind_group(0, white_pixel, &[]);
        self.set_bind_group(1, camera_bind_group, &[]);
        self.draw_indexed(0..6, 0, 0..1);
    }

    fn draw_textured_rect(&mut self, rect: &'b TexturedRect, camera_bind_group: &'b wgpu::BindGroup) {
        self.set_vertex_buffer(0, rect.vertex_buffer.slice(..));
        self.set_index_buffer(rect.index_buffer.slice(..), wgpu::IndexFormat::Uint16);
        self.set_bind_group(0, &rect.texture.bind_group, &[]);
        self.set_bind_group(1, camera_bind_group, &[]);
        self.draw_indexed(0..6, 0, 0..1);
    }
}