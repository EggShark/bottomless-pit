//! Contains the Material and MaterialBuilder struct which are needed
//! for anything to be rendered
//! ```rust,no_run
//! // Simple code to draw a 100x100 red rectangle to the screen
//! let defualt_material = MaterialBuilder::new().build();
//! 
//! impl Game for Struct {
//!     fn render<'pass, 'others>(&mut Self, mut renderer: RenderInformation<'pass, 'others>) where 'others: 'pass {
//!         self.defualt_material.add_rectangle(Vec2{x: 0.0, y: 0.0}, Vec2{x: 100.0, y: 100.0}, Colour::RED, &renderer);
//!         self.default_material.draw(&mut renderer);
//!     }
//! }

use crate::matrix_math::normalize_points;
use crate::texture::Texture;
use crate::vertex::Vertex;
use crate::engine_handle::{WgpuClump, Engine};
use crate::vectors::Vec2;
use crate::colour::Colour;
use crate::rect::Rectangle;
use crate::render::RenderInformation;

/// A material represents a unique combination of a Texture
/// and RenderPipeline, while also containing all nessicary buffers
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
    texture_size: Vec2<f32>,
}

impl Material {
    /// Takes a MaterialBuilder and turns it into a Material
    fn from_builder(builder: MaterialBuilder, engine: &mut Engine) -> Self {
        let pipeline_id = engine.defualt_pipe_id();
        let (texture_id, texture_size) = match builder.texture_change {
            Some(bg) => {
                let id = bg.bind_group.global_id();
                engine.add_to_bind_group_cache(bg.bind_group, id);
                (id, bg.size)
            },
            // should just be the size of the white pixel
            None => (engine.defualt_material_bg_id(), Vec2{x: 1.0, y: 1.0})
        };

        let wgpu = engine.get_wgpu();

        let vertex_size = std::mem::size_of::<Vertex>() as u64;
        let index_size = std::mem::size_of::<u16>() as u64;
        let (vertex_buffer, index_buffer) = Self::create_buffers(&wgpu.device, vertex_size, index_size);

        Self {
            pipeline_id,
            vertex_buffer,
            vertex_count: 0,
            vertex_size,
            index_buffer,
            index_count: 0,
            index_size,
            texture_id,
            texture_size,
        }
    }

    /// Will queue a Rectangle to be draw.
    pub fn add_rectangle(&mut self, position: Vec2<f32>, size: Vec2<f32>, colour: Colour, render: &RenderInformation) {
        let window_size = render.size;
        let wgpu = render.wgpu;
        let verts =
            Rectangle::from_pixels(position, size, colour.to_raw(), window_size).into_vertices();

        self.push_rectangle(wgpu, verts);
    }

    /// Queues a rectangle using WGSL cordinate space. (0, 0) is the center of the screen and (-1, 1) is the top left corner
    pub fn add_screenspace_rectangle(&mut self, position: Vec2<f32>, size: Vec2<f32>, colour: Colour, render: &RenderInformation) {
        let wgpu = render.wgpu;

        let verts = Rectangle::new(position, size, colour.to_raw());
        self.push_rectangle(wgpu, verts.into_vertices());
    }

    /// Queues a rectagnle with UV coordniates. The position and size of the UV cordniates are the same as the pixels in the 
    /// actaul image.
    pub fn add_rectangle_with_uv(&mut self, position: Vec2<f32>, size: Vec2<f32>, uv_position: Vec2<f32>, uv_size: Vec2<f32>, colour: Colour, render: &RenderInformation) {
        let wgpu = render.wgpu;
        let window_size = render.size;
        let uv_size = normalize_points(uv_size, self.texture_size.x, self.texture_size.y);
        let uv_position = normalize_points(uv_position, self.texture_size.x, self.texture_size.y);

        let verts = 
            Rectangle::from_pixels_with_uv(position, size, colour.to_raw(), window_size, uv_position, uv_size)
            .into_vertices();

        self.push_rectangle(wgpu, verts);
    }

    /// Queues a rectangle that will be rotated around its centerpoint. Rotation is in degrees
    pub fn add_rectangle_with_rotation(&mut self, position: Vec2<f32>, size: Vec2<f32>, colour: Colour, rotation: f32, render: &RenderInformation) {
        let wgpu = render.wgpu;
        let window_size = render.size;

        let verts = Rectangle::from_pixels_with_rotation(position, size, colour.to_raw(), window_size, rotation)
            .into_vertices();

        self.push_rectangle(wgpu, verts);
    }

    /// Queues a rectangle with both UV, and Rotation,
    pub fn add_rectangle_ex(&mut self, position: Vec2<f32>, size: Vec2<f32>, colour: Colour, rotation: f32, uv_position: Vec2<f32>, uv_size: Vec2<f32>, render: &RenderInformation) {
        let wgpu = render.wgpu;
        let window_size = render.size;
        let uv_size = normalize_points(uv_size, self.texture_size.x, self.texture_size.y);
        let uv_position = normalize_points(uv_position, self.texture_size.x, self.texture_size.y);

        let verts = 
            Rectangle::from_pixels_ex(position, size, colour.to_raw(), window_size, rotation, uv_position, uv_size)
            .into_vertices();

        self.push_rectangle(wgpu, verts);
    }
    
    /// Queues a rectangle with both UV, and Rotation, but will draw the rectangle in WGSL screenspace
    pub fn add_screenspace_rectangle_ex(&mut self, position: Vec2<f32>, size: Vec2<f32>, colour: Colour, rotation: f32, uv_position: Vec2<f32>, uv_size: Vec2<f32>, render: &RenderInformation) {
        let wgpu = render.wgpu;
        
        let verts = 
            Rectangle::new_ex(position, size, colour.to_raw(), rotation, uv_position, uv_size)
            .into_vertices();

        self.push_rectangle(wgpu, verts);
    }

    /// Queues a traingle, the points must be provided in clockwise order
    pub fn add_triangle(&mut self, p1: Vec2<f32>, p2: Vec2<f32>, p3: Vec2<f32>, colour: Colour, render: &RenderInformation) {
        let window_size = render.size;
        let wgpu = render.wgpu;

        let colour = colour.to_raw();
        let tex_coords = [0.0, 0.0];

        let verts = [
            Vertex::from_2d([p1.x, p1.y], tex_coords, colour)
                .pixels_to_screenspace(window_size),
            Vertex::from_2d([p2.x, p2.y], tex_coords, colour)
                .pixels_to_screenspace(window_size),
            Vertex::from_2d([p3.x, p3.y], tex_coords, colour)
                .pixels_to_screenspace(window_size),
        ];

        self.push_triangle(wgpu, verts);
    }

    /// Queues a triangle where each vertex is given its own colour. Points must be given
    /// in clockwise order
    pub fn add_triangle_with_coloured_verticies(
        &mut self,
        p1: Vec2<f32>,
        p2: Vec2<f32>,
        p3: Vec2<f32>,
        c1: Colour,
        c2: Colour,
        c3: Colour,
        render: &RenderInformation,
    ) {
        let window_size = render.size;
        let wgpu = render.wgpu;

        let tex_coords = [0.0, 0.0];
        let verts = [
            Vertex::from_2d([p1.x, p1.y], tex_coords, c1.to_raw())
                .pixels_to_screenspace(window_size),
            Vertex::from_2d([p2.x, p2.y], tex_coords, c2.to_raw())
                .pixels_to_screenspace(window_size),
            Vertex::from_2d([p3.x, p3.y], tex_coords, c3.to_raw())
                .pixels_to_screenspace(window_size)
        ];

        self.push_triangle(wgpu, verts);
    }

    /// Returns the number of verticies in the buffer
    pub fn get_vertex_number(&self) -> u64 {
        self.vertex_count / self.vertex_size
    }

    /// Returns the number if indincies in the buffer
    pub fn get_index_number(&self) -> u64 {
        self.index_count / self.index_size
    }

    // Returns the size of the texture in pixels
    pub fn get_texture_size(&self) -> Vec2<f32> {
        self.texture_size
    }

    fn push_rectangle(&mut self, wgpu: &WgpuClump, verts: [Vertex; 4]) {
        let max_verts = self.vertex_buffer.size();
        if self.vertex_count + (4 * self.vertex_size) > max_verts {
            self.grow_vertex_buffer(wgpu);
        }

        let num_verts = self.get_vertex_number() as u16;
        let indicies = [
            num_verts, 1 + num_verts, 2 + num_verts,
            3 + num_verts, num_verts, 2 + num_verts,
        ];

        let max_indicies = self.index_buffer.size();
        if self.index_count + (6 * self.index_size) > max_indicies {
            self.grow_index_buffer(wgpu);
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

    fn push_triangle(&mut self, wgpu: &WgpuClump, verts: [Vertex; 3]) {
        let max_verts = self.vertex_buffer.size();
        if self.vertex_count + (3 * self.vertex_size) > max_verts {
            self.grow_vertex_buffer(wgpu);
        }

        let num_verts = self.get_vertex_number() as u16;
        // yes its wastefull to do this but this is the only way to not have
        // it mess up other drawings while also allowing triangles
        let indicies = [
            num_verts, 1 + num_verts, 2 + num_verts,
            num_verts, 1 + num_verts, 2 + num_verts,
        ];

        let max_indicies = self.index_buffer.size();
        if self.index_count + (6 * self.index_size) > max_indicies {
            self.grow_index_buffer(wgpu);
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

        self.vertex_count += 3 * self.vertex_size;
        self.index_count += 6 * self.index_size;
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

    // there where 'others: 'pass notation says that 'others lives longer than 'pass
    pub fn draw<'pass, 'others>(&'others mut self, information: &mut RenderInformation<'pass, 'others>) where 'others: 'pass, {
        let pipeline = information.pipelines.get(&self.pipeline_id).unwrap();
        let texture = information.bind_groups.get(&self.texture_id).unwrap();

        information.render_pass.set_pipeline(pipeline);
        information.render_pass.set_bind_group(0, texture, &[]);

        information.render_pass.set_vertex_buffer(0, self.vertex_buffer.slice(0..self.vertex_count));
        information.render_pass.set_index_buffer(
            self.index_buffer.slice(0..self.vertex_count),
            wgpu::IndexFormat::Uint16,
        );

        information.render_pass.draw_indexed(0..self.get_index_number() as u32, 0, 0..1);

        self.vertex_count = 0;
        self.index_count = 0;
    }

    fn create_buffers(device: &wgpu::Device, vertex_size: u64, index_size: u64) -> (wgpu::Buffer, wgpu::Buffer) {
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

        (vertex_buffer, index_buffer)
    }
}

/// A builder struct used to create Materials
pub struct MaterialBuilder<'a> {
    // using options to denote a change from the default
    // in the case of a texture the defualt is just the White_Pixel
    texture_change: Option<Texture>,
    pipeline_layouts: &'a [wgpu::BindGroupLayout]
}

impl<'a> MaterialBuilder<'a> {
    /// Creates a new MaterialBuilder
    pub fn new() -> Self {
        Self {
            texture_change: None,
            pipeline_layouts: &[],
        }
    }

    /// Adds a Texture to the Material
    pub fn add_texture(self, texture: Texture) -> Self {
        Self {
            texture_change: Some(texture),
            pipeline_layouts: self.pipeline_layouts,
        }
    }

    /// Sets the bindgroup layouts for the pipeline, used if you want to create
    /// your own shader
    pub fn set_layouts(self, layouts: &'a [wgpu::BindGroupLayout]) -> Self {
        Self {
            texture_change: self.texture_change,
            pipeline_layouts: layouts,
        }
    }

    /// Turns the builder into a Material
    pub fn build(self, engine_handle: &mut Engine) -> Material {
        Material::from_builder(self, engine_handle)
    }
}