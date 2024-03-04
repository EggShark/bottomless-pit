use wgpu::util::DeviceExt;

use crate::engine_handle::{Engine, WgpuClump};
use crate::{layouts, vec2};
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
        let size = engine.get_window_size();

        let starting = [
            1.0, 0.0, 0.0, 0.0,
            0.0, 1.0, 0.0, 0.0,
            0.0, 0.0, 1.0, 0.0,
            size.x as f32, size.y as f32, 0.0, 0.0,
        ];

        let buffer = wgpu.device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Camera Buffer"),
            contents: bytemuck::cast_slice(&starting),
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

        let scale_x = self.scale.x;
        let scale_y = self.scale.y;

        let rot_x = screen_size.x / 2.0;
        let rot_y = screen_size.y / 2.0;

        let x_trans = rot_x - self.center.x;
        let y_trans = rot_y - self.center.y;

        let sin = self.rotation.to_radians().sin();
        let cos = self.rotation.to_radians().cos();

        // SRT matrix in collum major order I belive
        // row major order withouth padding looks like:                                                               //padding byte
        // [
        //   scale_x * cos, -scale_x * sin, scale_x * x_trans * cos - scale_x * y_trans * sin,
        //   scale_y * sin, scale_y * cos,  scale_y * x_trans * sin + scale_y * y_trans * sim,
        //   0.0,           0.0,            1.0,
        // ]
        // let matrix = [
        //     //c1:
        //     scale_x * cos,                                     scale_y * sin,                                     0.0, 0.0,
        //     //c2:
        //     scale_x * -sin,                                    scale_y * cos,                                     0.0, 0.0,
        //     //c3:
        //     scale_x * x_trans * cos - scale_x * y_trans * sin, scale_x * x_trans * sin + scale_y * y_trans * cos, 1.0, 0.0,
        // ];

        // DONT ROTATE AROUND X_TRANS OR Y_TRANS ROTATE AROUND SCREEEN / 2
        let matrix: [f32; 16] = [
            //c1
            scale_x * cos, scale_y * sin, 0.0, 0.0,
            //c2
            scale_x * -sin, scale_y * cos, 0.0, 0.0,
            //c3
            scale_x * x_trans * cos - scale_x * y_trans * sin - scale_x * rot_x * cos + scale_x * rot_y * sin + scale_x * rot_x, scale_y * x_trans * sin + scale_y * y_trans * cos - scale_y * rot_x * scale_y * sin - scale_y * rot_y * scale_y * cos + rot_y, 1.0, 0.0,
            //screen size
            screen_size.x, screen_size.y, 0.0, 0.0,
        ];

        wgpu
            .queue
            .write_buffer(&self.buffer, 0, bytemuck::cast_slice(&matrix));
    }

    pub fn set_active<'others, 'pass>(&'others self, renderer: &mut RenderInformation<'pass, 'others>) where 'others: 'pass {
        self.write_matrix(renderer.wgpu, renderer.size);

        renderer.render_pass.set_bind_group(1, &self.bind_group, &[]);
    }
}