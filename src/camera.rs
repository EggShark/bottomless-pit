//! This contains a simple 2D camera that can be used to transform 
//! the world.
//! ```rust
//!     fn render<'pass, 'others>(
//!         &'others mut self,
//!         mut render_handle: RenderInformation<'pass, 'others>,
//!     ) where
//!     'others: 'pass,
//!     {
//!         self.material.add_rectangle(Vec2 { x: 0.0, y: 0.0 }, Vec2{x: 300.0, y: 300.0}, Colour::WHITE, &render_handle);
//!         // Draws objects with a cameras transform
//!         self.camera.set_active(&mut render_handle);
//!         self.material.draw(&mut render_handle);
//!         // Resets back to the default camera great for static elements like a HUD or UI
//!         render_handle.reset_camera();
//!         self.text.add_instance(vec2!(0.0), Colour::WHITE, &render_handle);
//!         self.text.draw(&mut render_handle);
//! }
//! ```

use glam::Mat3;
use wgpu::util::DeviceExt;

use crate::context::WgpuClump;
use crate::layouts;
use crate::render::Renderer;
use crate::vectors::Vec2;

/// A simple 2D camera that can translate, rotate, and scale everything on the screen.
pub struct Camera {
    inner: Option<InternalCamera>,
    /// The center of the camera becomes the center of the screen
    pub center: Vec2<f32>,
    /// needs to be in degrees
    pub rotation: f32,
    /// This controls the size of every object in view
    pub scale: Vec2<f32>,
}

impl Camera {
    /// creates a new camera with center point, rotation and scale.
    pub fn new(center: Vec2<f32>, rotation: f32, scale: Vec2<f32>) -> Self {
        Self {
            inner: None,
            center,
            rotation,
            scale,
        }
    }

    /// This will transform a point in screen space to camera space.
    /// You can get the screen size from [crate::Engine::get_window_size]
    pub fn transform_point(&self, point: Vec2<f32>, screen_size: Vec2<u32>) -> Vec2<f32> {
        let screen_size = Vec2{x: screen_size.x as f32, y: screen_size.y as f32};

        let scale_x = self.scale.x;
        let scale_y = self.scale.y;

        let rot_x = screen_size.x / 2.0;
        let rot_y = screen_size.y / 2.0;

        let x_trans = rot_x - self.center.x;
        let y_trans = rot_y - self.center.y;

        let sin = self.rotation.to_radians().sin();
        let cos = self.rotation.to_radians().cos();

        let mat: Mat3 = Mat3::from_cols_array(&[
            scale_x * cos, scale_y * sin, 0.0,
            //c2
            -scale_x * sin, scale_y * cos, 0.0,
            //c3
            scale_x * x_trans * cos - scale_x * y_trans * sin - rot_x * scale_x * cos + rot_y * scale_x * sin + rot_x, scale_y * x_trans * sin + scale_y * y_trans * cos - rot_x * scale_y * sin - rot_y * scale_y * cos + rot_y, 1.0,
        ]).inverse();

        mat.transform_point2(point.into()).into()
    }

    fn write_matrix(&mut self, wgpu: &WgpuClump, screen_size: Vec2<u32>) {
        let screen_size = Vec2{x: screen_size.x as f32, y: screen_size.y as f32};

        let scale_x = self.scale.x;
        let scale_y = self.scale.y;

        let rot_x = screen_size.x / 2.0;
        let rot_y = screen_size.y / 2.0;

        let x_trans = rot_x - self.center.x;
        let y_trans = rot_y - self.center.y;

        let sin = self.rotation.to_radians().sin();
        let cos = self.rotation.to_radians().cos();

        //THIS IS T(rot)S(x_scale, y_scale)R(d)T(-rot)T(x_trans, y_trans)
        let matrix: [f32; 16] = [
            //c1
            scale_x * cos, scale_y * sin, 0.0, 0.0,
            //c2
            -scale_x * sin, scale_y * cos, 0.0, 0.0,
            //c3
            scale_x * x_trans * cos - scale_x * y_trans * sin - rot_x * scale_x * cos + rot_y * scale_x * sin + rot_x, scale_y * x_trans * sin + scale_y * y_trans * cos - rot_x * scale_y * sin - rot_y * scale_y * cos + rot_y, 1.0, 0.0,
            //screen size
            screen_size.x, screen_size.y, 0.0, 0.0,
        ];

        if self.inner.is_none() {
            self.inner = Some(InternalCamera::new(wgpu, &matrix));
        }

        wgpu
            .queue
            .write_buffer(&self.inner.as_ref().unwrap().buffer, 0, bytemuck::cast_slice(&matrix));
    }

    /// Sets this camera to the active camera transforming all objects with this camera.
    pub fn set_active<'others, 'pass>(&'others mut self, renderer: &mut Renderer<'pass, 'others>) {
        self.write_matrix(renderer.wgpu, renderer.size);

        renderer.pass.set_bind_group(1, &self.inner.as_ref().unwrap().bind_group, &[]);
    }
}

impl Default for Camera {
    /// creates a new camera with these values:
    /// ```rust
    /// Camera {
    ///     center: Vec2{x : 0.0, y: 0.0},
    ///     rotation: 0.0,
    ///     scale: Vec2{x: 1.0, y: 1.0},
    /// }
    /// ```
    fn default() -> Self {
        Self {
            inner: None,
            center: Vec2{x: 0.0, y: 0.0},
            rotation: 0.0,
            scale: Vec2{x: 1.0, y: 1.0},
        }
    }
}

struct InternalCamera {
    bind_group: wgpu::BindGroup,
    buffer: wgpu::Buffer,
}

impl InternalCamera {
    fn new(wgpu: &WgpuClump, matrix: &[f32; 16]) -> Self {
        let buffer = wgpu.device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Camera Buffer"),
            contents: bytemuck::cast_slice(matrix),
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
        }
    }
}