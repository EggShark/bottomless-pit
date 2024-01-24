use glam::Mat3;
use wgpu::util::DeviceExt;

use crate::engine_handle::Engine;
use crate::{layouts, IDENTITY_MATRIX};
use crate::render::RenderInformation;
use crate::vectors::Vec2;

// const OPENGL_TO_WGPU_MATRIX: Mat4 = Mat4::from_cols_array(&[
//     1.0, 0.0, 0.0, 0.0,
//     0.0, 1.0, 0.0, 0.0,
//     0.0, 0.0, 0.5, 0.5,
//     0.0, 0.0, 0.0, 1.0, 
// ]);

pub struct Camera {
    bind_group: wgpu::BindGroup,
    buffer: wgpu::Buffer,
    translation: Vec2<f32>,
    rotaion: f32,
    scale: Vec2<f32>,
}

impl Camera {
    pub fn new(engine: &Engine) -> Self {
        let wgpu = engine.get_wgpu();

        let matrix = Mat3::IDENTITY;

        let buffer = wgpu.device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Camera Buffer"),
            contents: bytemuck::cast_slice(&IDENTITY_MATRIX),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST
        });

        let camera_bind_group_layout = layouts::create_camera_layout(&wgpu.device);

        let bind_group = wgpu
            .device
            .create_bind_group(&wgpu::BindGroupDescriptor {
                layout: &camera_bind_group_layout,
                entries: &[wgpu::BindGroupEntry {
                    binding: 0,
                    resource: buffer.as_entire_binding(),
                }],
                label: Some("camera_bind_group"),
            });

        println!("{:?}", matrix.as_ref());

        Self {
            bind_group,
            buffer,
            translation: Vec2{x: 0.0, y: 0.0},
            rotaion: 0.0,
            scale: Vec2{x: 1.0, y: 1.0},
        }
    }

    pub fn set_scale(&mut self, scale: f32) {
        todo!()
    }

    pub fn set_xy_scale(&mut self, scale: Vec2<f32>) {
        todo!()
    }

    pub fn set_center(&mut self, new_pos: Vec2<f32>) {
        todo!()
    }

    pub fn move_center(&mut self, translation: Vec2<f32>) {
        todo!()
    }
}