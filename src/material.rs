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
use std::f32::consts::PI;

use crate::matrix_math::normalize_points;
use crate::texture::RegisteredTexture;
use crate::vertex::{self, Vertex, LineVertex};
use crate::engine_handle::{WgpuClump, Engine};
use crate::vectors::Vec2;
use crate::colour::Colour;
use crate::render::RenderInformation;
use crate::shader::{UniformData, Shader};

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
    uniform_bindgroup: Option<wgpu::BindGroup>,
}

impl Material {
    /// Takes a MaterialBuilder and turns it into a Material
    fn from_builder(builder: MaterialBuilder, engine: &mut Engine) -> Self {
        let pipeline_id = match builder.shader_change {
            Some(rs) => rs.pipeline_id,
            None => engine.defualt_pipe_id(),
        };

        let (texture_id, texture_size) = match builder.texture_change {
            Some(rt) => (rt.bindgroup_id, rt.texture_size),
            // should just be the size of the white pixel
            None => (engine.defualt_material_bg_id(), Vec2{x: 1.0, y: 1.0})
        };

        let wgpu = engine.get_wgpu();

        let uniform_bindgroup = builder.uniform_data
            .and_then(|data| Some(data.create_bind_group(engine.get_texture_sampler(), &wgpu)));

        let vertex_size = std::mem::size_of::<Vertex>() as u64;
        let index_size = std::mem::size_of::<u16>() as u64;
        let (vertex_buffer, index_buffer) = Self::create_buffers(&wgpu.device, vertex_size, 50, index_size, 50);

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
            uniform_bindgroup,
        }
    }

    /// Will queue a Rectangle to be draw.
    pub fn add_rectangle(&mut self, position: Vec2<f32>, size: Vec2<f32>, colour: Colour, render: &RenderInformation) {
        let window_size = render.size;
        let wgpu = render.wgpu;
        let verts =
            vertex::from_pixels(position, size, colour.to_raw(), window_size);

        self.push_rectangle(wgpu, verts);
    }

    /// Queues a rectangle using WGSL cordinate space. (0, 0) is the center of the screen and (-1, 1) is the top left corner
    pub fn add_screenspace_rectangle(&mut self, position: Vec2<f32>, size: Vec2<f32>, colour: Colour, render: &RenderInformation) {
        let wgpu = render.wgpu;

        let verts = vertex::new(position, size, colour.to_raw());
        self.push_rectangle(wgpu, verts);
    }

    /// Queues a rectagnle with UV coordniates. The position and size of the UV cordniates are the same as the pixels in the 
    /// actaul image.
    pub fn add_rectangle_with_uv(&mut self, position: Vec2<f32>, size: Vec2<f32>, uv_position: Vec2<f32>, uv_size: Vec2<f32>, colour: Colour, render: &RenderInformation) {
        let wgpu = render.wgpu;
        let window_size = render.size;
        let uv_size = normalize_points(uv_size, self.texture_size);
        let uv_position = normalize_points(uv_position, self.texture_size);

        let verts = 
            vertex::from_pixels_with_uv(position, size, colour.to_raw(), window_size, uv_position, uv_size);

        self.push_rectangle(wgpu, verts);
    }

    /// Queues a rectangle that will be rotated around its centerpoint. Rotation is in degrees
    pub fn add_rectangle_with_rotation(&mut self, position: Vec2<f32>, size: Vec2<f32>, colour: Colour, rotation: f32, render: &RenderInformation) {
        let wgpu = render.wgpu;
        let window_size = render.size;

        let verts =
            vertex::from_pixels_with_rotation(position, size, colour.to_raw(), window_size, rotation);

        self.push_rectangle(wgpu, verts);
    }

    /// Queues a rectangle with both UV, and Rotation,
    pub fn add_rectangle_ex(&mut self, position: Vec2<f32>, size: Vec2<f32>, colour: Colour, rotation: f32, uv_position: Vec2<f32>, uv_size: Vec2<f32>, render: &RenderInformation) {
        let wgpu = render.wgpu;
        let window_size = render.size;
        let uv_size = normalize_points(uv_size, self.texture_size);
        let uv_position = normalize_points(uv_position, self.texture_size);

        let verts = 
            vertex::from_pixels_ex(position, size, colour.to_raw(), window_size, rotation, uv_position, uv_size);

        self.push_rectangle(wgpu, verts);
    }
    
    /// Queues a rectangle with both UV, and Rotation, but will draw the rectangle in WGSL screenspace
    pub fn add_screenspace_rectangle_ex(&mut self, position: Vec2<f32>, size: Vec2<f32>, colour: Colour, rotation: f32, uv_position: Vec2<f32>, uv_size: Vec2<f32>, render: &RenderInformation) {
        let wgpu = render.wgpu;
        
        let verts = 
            vertex::new_ex(position, size, colour.to_raw(), rotation, uv_position, uv_size);

        self.push_rectangle(wgpu, verts);
    }

    /// Queues a 4 pointed polygon with complete control over uv coordinates and rotation. The points need to be in top left, right
    /// bottom right and bottom left order as it will not render porperly otherwise.
    pub fn add_custom(&mut self, points: [Vec2<f32>; 4], uv_points: [Vec2<f32>; 4], rotation: f32, colour: Colour, render: &RenderInformation) {
        let wgpu = render.wgpu;
        let uv_points = [
            normalize_points(uv_points[0], self.texture_size),
            normalize_points(uv_points[1], self.texture_size),
            normalize_points(uv_points[2], self.texture_size),
            normalize_points(uv_points[3], self.texture_size),
        ];

        let verts =
            vertex::from_pixels_custom(points, uv_points, rotation, colour.to_raw(), render.size);

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

    /// Queues a polygon with the specified number of sides at a position with size and colour.
    /// This will not play nicely with texture as all the UV coords will be at [0, 0]. 
    pub fn add_regular_n_gon(&mut self, number_of_sides: usize, radius: f32, center: Vec2<f32>, colour: Colour, render: &RenderInformation) {
        if number_of_sides < 4 {
            return;
        }

        let wgpu = render.wgpu;
        let screen_size = render.size;

        let vertices = (0..number_of_sides)
            .map(|num| {
                Vec2 {
                    x: radius * (2.0 * PI * num as f32 / number_of_sides as f32).cos() + center.x,
                    y: radius * (2.0 * PI * num as f32 / number_of_sides as f32).sin() + center.y,
                }
            })
            .map(|point| {
                Vertex::from_2d([point.x, point.y], [0.0, 0.0], colour.to_raw())
                .pixels_to_screenspace(screen_size)   
            })
            .collect::<Vec<Vertex>>();
    
        let number_of_vertices = self.get_vertex_number() as u16;
        let number_of_triangles = (number_of_sides - 2) as u16;

        let mut indicies = (1..number_of_triangles + 1)
            .flat_map(|i| {
                [
                    number_of_vertices,
                    i + number_of_vertices,
                    i + 1 + number_of_vertices,
                ]
            })
            .collect::<Vec<u16>>();

        // ensures we follow copy buffer alignment
        let num_indicies = indicies.len();
        let triangles_to_add = if num_indicies < 12 {
            (12 % num_indicies) / 3
        } else {
            (num_indicies % 12) / 3
        };

        for _ in 0..triangles_to_add {
            indicies.extend_from_slice(&[indicies[num_indicies-3], indicies[num_indicies-2], indicies[num_indicies-1]]);
        }

        let max_verts = self.vertex_buffer.size();
        if self.vertex_count + (vertices.len() as u64 * self.vertex_size) > max_verts {
            grow_buffer(&mut self.vertex_buffer, wgpu, self.vertex_count + (vertices.len() as u64 * self.vertex_size), wgpu::BufferUsages::VERTEX);
        }

        let max_indicies = self.index_buffer.size();
        if self.index_count + (indicies.len() as u64 * self.index_size) > max_indicies {
            grow_buffer(&mut self.index_buffer, wgpu, self.index_count + (indicies.len() as u64 * self.index_size), wgpu::BufferUsages::INDEX);
        }

        wgpu.queue.write_buffer(
            &self.vertex_buffer,
            self.vertex_count,
            bytemuck::cast_slice(&vertices),
        );
        wgpu.queue.write_buffer(
            &self.index_buffer,
            self.index_count,
            bytemuck::cast_slice(&indicies),
        );

        self.vertex_count += vertices.len() as u64 * self.vertex_size;
        self.index_count += indicies.len() as u64 * self.index_size;
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
            grow_buffer(&mut self.vertex_buffer, wgpu, 1, wgpu::BufferUsages::VERTEX);
        }

        let num_verts = self.get_vertex_number() as u16;
        let indicies = [
            num_verts, 1 + num_verts, 2 + num_verts,
            3 + num_verts, num_verts, 2 + num_verts,
        ];

        let max_indicies = self.index_buffer.size();
        if self.index_count + (6 * self.index_size) > max_indicies {
            grow_buffer(&mut self.index_buffer, wgpu, 1, wgpu::BufferUsages::INDEX);
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
            grow_buffer(&mut self.vertex_buffer, wgpu, 1, wgpu::BufferUsages::VERTEX);
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
            grow_buffer(&mut self.index_buffer, wgpu, 1, wgpu::BufferUsages::INDEX);
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

    // there where 'others: 'pass notation says that 'others lives longer than 'pass
    /// Draws all queued shapes to the screen.
    pub fn draw<'pass, 'others>(&'others mut self, information: &mut RenderInformation<'pass, 'others>) where 'others: 'pass, {
        if self.vertex_count == 0 {
            return;
        }

        let pipeline = information.pipelines.get(&self.pipeline_id).unwrap();
        let texture = information.bind_groups.get(&self.texture_id).unwrap();

        information.render_pass.set_pipeline(pipeline);
        information.render_pass.set_bind_group(0, texture, &[]);
        information.render_pass.set_bind_group(1, information.camera_bindgroup, &[]);

        match &self.uniform_bindgroup {
            Some(bg) => information.render_pass.set_bind_group(2, bg, &[]),
            None => {},
        }

        information.render_pass.set_vertex_buffer(0, self.vertex_buffer.slice(0..self.vertex_count));
        information.render_pass.set_index_buffer(
            self.index_buffer.slice(0..self.index_count),
            wgpu::IndexFormat::Uint16,
        );

        information.render_pass.draw_indexed(0..self.get_index_number() as u32, 0, 0..1);

        self.vertex_count = 0;
        self.index_count = 0;
    }

    pub(crate) fn create_buffers(device: &wgpu::Device, vertex_size: u64, vert_count: u64, index_size: u64, index_count: u64) -> (wgpu::Buffer, wgpu::Buffer) {
        let vertex_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Vertex_Buffer"),
            size: vertex_size * vert_count,
            usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::COPY_SRC,
            mapped_at_creation: false,
        });

        // this is just 200 bytes pretty small
        let index_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Index_Buffer"),
            size: index_size * index_count,
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
    texture_change: Option<RegisteredTexture>,
    shader_change: Option<Shader>,
    uniform_data: Option<&'a UniformData>,
}

impl<'a> MaterialBuilder<'a> {
    /// Creates a new MaterialBuilder, that contains no texture, custom shaders, or
    /// uniforms 
    pub fn new() -> Self {
        Self {
            texture_change: None,
            shader_change: None,
            uniform_data: None,
        }
    }

    /// Adds a Texture to the Material
    pub fn add_texture(self, texture: RegisteredTexture) -> Self {
        Self {
            texture_change: Some(texture),
            shader_change: self.shader_change,
            uniform_data: self.uniform_data,
        }
    }


    /// Sets the initial Uniform data for the shader
    pub fn set_uniform(self, data: &'a UniformData) -> Self {
        Self {
            texture_change: self.texture_change,
            shader_change: self.shader_change,
            uniform_data: Some(data),
        }
    }

    /// Sets the shader for the Material
    pub fn set_shader(self, shader: Shader) -> Self {
        Self {
            texture_change: self.texture_change,
            shader_change: Some(shader),
            uniform_data: self.uniform_data,
        }
    }

    /// Turns the builder into a Material
    pub fn build(self, engine_handle: &mut Engine) -> Material {
        Material::from_builder(self, engine_handle)
    }
}

/// A diffrent type of material used to draw WebGPU debug
/// lines. These lines will allways be 1px wide and only one
/// instance of this material should ever be made per programm.
pub struct LineMaterial {
    pipe_id: wgpu::Id<wgpu::RenderPipeline>,
    vertex_buffer: wgpu::Buffer,
    vertex_count: u64,
    vertex_size: u64,
}

impl LineMaterial {
    /// Creates a new LineMaterial
    pub fn new(engine: &Engine) -> Self {
        let wgpu = engine.get_wgpu();
        let vertex_size = std::mem::size_of::<LineVertex>() as u64;

        let vertex_buffer = wgpu.device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Line material vertex buffer"),
            usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::COPY_SRC,
            size: vertex_size * 100,
            mapped_at_creation: false,
        });

        Self {
            pipe_id: engine.line_pipe_id(),
            vertex_buffer,
            vertex_count: 0,
            vertex_size,
        }
    }

    /// Queues a line from the two points.
    pub fn add_line(&mut self, start: Vec2<f32>, end: Vec2<f32>, colour: Colour, renderer: &RenderInformation) {
        let screen_size = renderer.size;
        let wgpu = renderer.wgpu;

        let verts = [
            LineVertex::new(start.to_raw(), colour.to_raw())
                .pixels_to_screenspace(screen_size),
            LineVertex::new(end.to_raw(), colour.to_raw())
                .pixels_to_screenspace(screen_size),
        ];

        let max_verts = self.vertex_buffer.size();
        let vert_size = std::mem::size_of::<LineVertex>() as u64;
        if self.vertex_count + (2 * vert_size) > max_verts {
            grow_buffer(&mut self.vertex_buffer, wgpu, 1, wgpu::BufferUsages::VERTEX);
        }

        wgpu.queue.write_buffer(
            &self.vertex_buffer,
            self.vertex_count,
            bytemuck::cast_slice(&verts),
        );

        self.vertex_count += 2 * self.vertex_size;
    }
    
    /// Draws all queued lines to the screen.
    pub fn draw<'pass, 'others>(&'others mut self, information: &mut RenderInformation<'pass, 'others>) where 'others: 'pass, {
        let pipeline = information.pipelines.get(&self.pipe_id).unwrap();
        
        information.render_pass.set_pipeline(pipeline);
        information.render_pass.set_bind_group(0, information.camera_bindgroup, &[]);
        information.render_pass.set_vertex_buffer(0, self.vertex_buffer.slice(0..self.vertex_count));

        information.render_pass.draw(0..self.get_vertex_count() as u32, 0..1);

        self.vertex_count = 0;
    }
    
    pub fn get_vertex_count(&self) -> u64 {
        self.vertex_count/self.vertex_size
    }

}

pub(crate) fn grow_buffer(buffer: &mut wgpu::Buffer, wgpu: &WgpuClump, size_needed: u64, vert_or_index: wgpu::BufferUsages) {
    let mut encoder = wgpu.device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
        label: Some("Material Buffer Grower"),
    });

    let size_needed = size_needed + (4 - (size_needed % wgpu::COPY_BUFFER_ALIGNMENT));

    let new_size = std::cmp::max(buffer.size() * 2, size_needed);
    let new_buffer = wgpu.device.create_buffer(&wgpu::BufferDescriptor {
        label: Some("Vertex_Buffer"),
        size: new_size,
        usage: vert_or_index | wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::COPY_SRC,
        mapped_at_creation: false,
    });

    encoder.copy_buffer_to_buffer(
        &buffer,
        0,
        &new_buffer,
        0,
        buffer.size(),
    );

    wgpu.
        queue
        .submit(std::iter::once(encoder.finish()));

    *buffer = new_buffer;
}