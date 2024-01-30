use glam::Mat3;
use wgpu::util::DeviceExt;

use crate::engine_handle::{Engine, WgpuClump};
use crate::{layouts, IDENTITY_MATRIX, vec2};
use crate::render::RenderInformation;
use crate::vectors::Vec2;

pub struct Camera {
    bind_group: wgpu::BindGroup,
    buffer: wgpu::Buffer,
    pub center: Vec2<f32>,
    /// needs to be in degrees
    pub rotation: f32,
    pub scale: Vec2<f32>,
}

impl Camera {
    pub fn new(engine: &Engine) -> Self {
        let wgpu = engine.get_wgpu();

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

        Self {
            bind_group,
            buffer,
            center: Vec2{x: 0.0, y: 0.0},
            rotation: 0.0,
            scale: Vec2{x: 1.0, y: 1.0},
        }
    }

    pub fn set_scale(&mut self, scale: f32) {
        self.scale = vec2!(scale);
    }

    pub fn set_xy_scale(&mut self, scale: Vec2<f32>) {
        self.scale = scale;
    }

    pub fn set_center(&mut self, new_pos: Vec2<f32>) {
        self.center = new_pos;
    }

    pub fn move_center(&mut self, translation: Vec2<f32>) {
        self.center = self.center + translation;
    }

    pub fn set_rotation(&mut self, rotation: f32) {
        self.rotation = rotation;
    }

    pub fn get_rotation(&self) -> f32 {
        self.rotation
    }

    fn write_matrix(&self, wgpu: &WgpuClump, screen_size: Vec2<u32>) {
        let screen_size = Vec2{x: screen_size.x as f32, y: screen_size.y as f32};

        // get normalized translation and mult by scale
        let x_trans = (self.center.x / screen_size.x + 1.0) * self.scale.x;
        let y_trans = (self.center.y / screen_size.y - 1.0) * self.scale.y;

        let sin = self.rotation.to_radians().sin();
        let cos = self.rotation.to_radians().cos();

        let matrix = [
            self.scale.x * cos, self.scale.x * -sin, x_trans, 0.0,
            self.scale.y * sin, self.scale.y * cos,  y_trans, 0.0,
            0.0,                0.0,                 1.0,     0.0
        ];

        println!("{:?}", matrix);

        wgpu
            .queue
            .write_buffer(&self.buffer, 0, bytemuck::cast_slice(&matrix));
    }

    pub fn set_active<'others, 'pass>(&'others self, renderer: &mut RenderInformation<'pass, 'others>) where 'others: 'pass {
        self.write_matrix(renderer.wgpu, renderer.size);

        renderer.render_pass.set_bind_group(1, &self.bind_group, &[]);
    }
}