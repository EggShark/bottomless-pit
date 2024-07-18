//! Contains the Material and MaterialBuilder struct which are needed
//! for anything to be rendered
//! ```rust,no_run
//! // Simple code to draw a 100x100 red rectangle to the screen
//! let defualt_material: Material<()> = MaterialBuilder::new().build();
//!
//! impl Game for Struct {
//!     fn render<'pass, 'others>(&mut Self, mut renderer: RenderInformation<'pass, 'others>) where 'others: 'pass {
//!         self.defualt_material.add_rectangle(Vec2{x: 0.0, y: 0.0}, Vec2{x: 100.0, y: 100.0}, Colour::RED, &renderer);
//!         self.default_material.draw(&mut renderer);
//!     }
//! }
use std::f32::consts::PI;
use std::marker::PhantomData;

use encase::private::WriteInto;
use encase::ShaderType;

use crate::colour::Colour;
use crate::context::WgpuClump;
use crate::engine_handle::Engine;
use crate::matrix_math::normalize_points;
use crate::render::Renderer;
use crate::resource::ResourceId;
use crate::shader::{Shader, UniformData, UniformError};
use crate::texture::{Texture, UniformTexture};
use crate::vectors::Vec2;
use crate::vertex::{self, LineVertex, Vertex};

/// A material represents a unique combination of a Texture
/// and Shader, while also containing all nessicary buffers.
/// If you don't have any uniform data attached to the shader
/// utilized by the material use the unit `()` type.
#[derive(Debug)]
pub struct Material<T> {
    pipeline_id: ResourceId<Shader>,
    /// counts the bytes of vertex not the actual number
    pub(crate) vertex_size: u64,
    pub(crate) vertex_count: u64,
    /// counts the bytes of the index not the actual number
    pub(crate) index_count: u64,
    pub(crate) index_size: u64,
    inner: Option<InnerBuffer>,
    texture_id: ResourceId<Texture>,
    _marker: PhantomData<T>,
}

impl<T> Material<T> {
    /// Takes a MaterialBuilder and turns it into a Material
    fn from_builder(builder: MaterialBuilder<T>, engine: &mut Engine) -> Self {
        let pipeline_id = match builder.shader_change {
            Some(rs) => rs,
            None => engine.defualt_pipe_id(),
        };

        let texture_id = builder
            .texture_change
            .unwrap_or(engine.defualt_material_bg_id());

        let vertex_size = std::mem::size_of::<Vertex>() as u64;
        let index_size = std::mem::size_of::<u16>() as u64;

        Self {
            pipeline_id,
            vertex_count: 0,
            vertex_size,
            index_count: 0,
            index_size,
            inner: None,
            texture_id,
            _marker: PhantomData,
        }
    }

    pub fn change_texture(&mut self, texture: ResourceId<Texture>) {
        self.texture_id = texture
    }

    /// Will queue a Rectangle to be draw.
    pub fn add_rectangle(
        &mut self,
        position: Vec2<f32>,
        size: Vec2<f32>,
        colour: Colour,
        render: &Renderer,
    ) {
        let wgpu = render.wgpu;
        let verts = vertex::from_pixels(position, size, colour.as_raw());

        self.push_rectangle(wgpu, verts);
    }

    /// Queues a rectangle using WGSL cordinate space. (0, 0) is the center of the screen and (-1, 1) is the top left corner
    pub fn add_screenspace_rectangle(
        &mut self,
        position: Vec2<f32>,
        size: Vec2<f32>,
        colour: Colour,
        render: &Renderer,
    ) {
        let wgpu = render.wgpu;
        let screen_size = render.size;

        let verts = vertex::new(position, size, colour.as_raw(), screen_size);
        self.push_rectangle(wgpu, verts);
    }

    /// Queues a rectagnle with UV coordniates. The position and size of the UV cordniates are the same as the pixels in the
    /// actaul image.
    pub fn add_rectangle_with_uv(
        &mut self,
        position: Vec2<f32>,
        size: Vec2<f32>,
        uv_position: Vec2<f32>,
        uv_size: Vec2<f32>,
        colour: Colour,
        render: &Renderer,
    ) {
        let wgpu = render.wgpu;

        let texture_size = render
            .resources
            .get_texture(&self.texture_id)
            .map(|t| t.size)
            .unwrap_or(Vec2{x: 1.0, y: 1.0});
            // doesnt matter what i put here bc the texture isnt loaded regardless

        let uv_size = normalize_points(uv_size, texture_size);
        let uv_position = normalize_points(uv_position, texture_size);

        let verts = vertex::from_pixels_with_uv(
            position,
            size,
            colour.as_raw(),
            uv_position,
            uv_size,
        );

        self.push_rectangle(wgpu, verts);
    }

    /// Queues a rectangle that will be rotated around its centerpoint. Rotation is in degrees
    pub fn add_rectangle_with_rotation(
        &mut self,
        position: Vec2<f32>,
        size: Vec2<f32>,
        colour: Colour,
        rotation: f32,
        render: &Renderer,
    ) {
        let wgpu = render.wgpu;

        let verts = vertex::from_pixels_with_rotation(
            position,
            size,
            colour.as_raw(),
            rotation,
        );

        self.push_rectangle(wgpu, verts);
    }

    #[allow(clippy::too_many_arguments)]
    /// Queues a rectangle with both UV, and Rotation,
    pub fn add_rectangle_ex(
        &mut self,
        position: Vec2<f32>,
        size: Vec2<f32>,
        colour: Colour,
        rotation: f32,
        uv_position: Vec2<f32>,
        uv_size: Vec2<f32>,
        render: &Renderer,
    ) {
        let wgpu = render.wgpu;

        let texture_size = render.resources.get_texture(&self.texture_id).unwrap().size;

        let uv_size = normalize_points(uv_size, texture_size);
        let uv_position = normalize_points(uv_position, texture_size);

        let verts = vertex::from_pixels_ex(
            position,
            size,
            colour.as_raw(),
            rotation,
            uv_position,
            uv_size,
        );

        self.push_rectangle(wgpu, verts);
    }

    #[allow(clippy::too_many_arguments)]
    /// Queues a rectangle with both UV, and Rotation, but will draw the rectangle in WGSL screenspace
    pub fn add_screenspace_rectangle_ex(
        &mut self,
        position: Vec2<f32>,
        size: Vec2<f32>,
        colour: Colour,
        rotation: f32,
        uv_position: Vec2<f32>,
        uv_size: Vec2<f32>,
        render: &Renderer,
    ) {
        let wgpu = render.wgpu;

        let verts = vertex::new_ex(
            position,
            size,
            colour.as_raw(),
            rotation,
            uv_position,
            uv_size,
        );

        self.push_rectangle(wgpu, verts);
    }

    /// Queues a 4 pointed polygon with complete control over uv coordinates and rotation. The points need to be in top left, right
    /// bottom right and bottom left order as it will not render porperly otherwise.
    pub fn add_custom(
        &mut self,
        points: [Vec2<f32>; 4],
        uv_points: [Vec2<f32>; 4],
        rotation: f32,
        colour: Colour,
        render: &Renderer,
    ) {
        let wgpu = render.wgpu;
        let texture_size = render.resources.get_texture(&self.texture_id).unwrap().size;
        let uv_points = [
            normalize_points(uv_points[0], texture_size),
            normalize_points(uv_points[1], texture_size),
            normalize_points(uv_points[2], texture_size),
            normalize_points(uv_points[3], texture_size),
        ];

        let verts =
            vertex::from_pixels_custom(points, uv_points, rotation, colour.as_raw());

        self.push_rectangle(wgpu, verts);
    }

    /// Queues a traingle, the points must be provided in clockwise order
    pub fn add_triangle(
        &mut self,
        p1: Vec2<f32>,
        p2: Vec2<f32>,
        p3: Vec2<f32>,
        colour: Colour,
        render: &Renderer,
    ) {
        let wgpu = render.wgpu;

        let colour = colour.as_raw();
        let tex_coords = [0.0, 0.0];

        let verts = [
            Vertex::from_2d([p1.x, p1.y], tex_coords, colour),
            Vertex::from_2d([p2.x, p2.y], tex_coords, colour),
            Vertex::from_2d([p3.x, p3.y], tex_coords, colour),
        ];

        self.push_triangle(wgpu, verts);
    }

    /// Queues a triangle where each vertex is given its own colour. Points must be given
    /// in clockwise order
    pub fn add_triangle_with_coloured_verticies(
        &mut self,
        points: [Vec2<f32>; 3],
        colours: [Colour; 3],
        render: &Renderer,
    ) {
        let wgpu = render.wgpu;

        let tex_coords = [0.0, 0.0];
        let verts = [
            Vertex::from_2d([points[0].x, points[0].y], tex_coords, colours[0].as_raw()),
            Vertex::from_2d([points[1].x, points[1].y], tex_coords, colours[1].as_raw()),
            Vertex::from_2d([points[2].x, points[2].y], tex_coords, colours[2].as_raw()),
        ];

        self.push_triangle(wgpu, verts);
    }

    /// Queues a polygon with the specified number of sides at a position with size and colour.
    /// This will not play nicely with texture as all the UV coords will be at [0, 0].
    pub fn add_regular_n_gon(
        &mut self,
        number_of_sides: usize,
        radius: f32,
        center: Vec2<f32>,
        colour: Colour,
        render: &Renderer,
    ) {
        if number_of_sides < 4 {
            return;
        }

        if self.inner.is_none() {
            let (vert, ind) =
                Self::create_buffers(&render.wgpu.device, self.vertex_size, 50, self.index_size, 50);

            self.inner = Some(InnerBuffer {
                vertex_buffer: vert,
                index_buffer: ind,
            });
        }

        let wgpu = render.wgpu;

        let vertices = (0..number_of_sides)
            .map(|num| Vec2 {
                x: radius * (2.0 * PI * num as f32 / number_of_sides as f32).cos() + center.x,
                y: radius * (2.0 * PI * num as f32 / number_of_sides as f32).sin() + center.y,
            })
            .map(|point| {
                Vertex::from_2d([point.x, point.y], [0.0, 0.0], colour.as_raw())
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
            indicies.extend_from_slice(&[
                indicies[num_indicies - 3],
                indicies[num_indicies - 2],
                indicies[num_indicies - 1],
            ]);
        }

        let buffers = self.inner.as_mut().unwrap();

        let max_verts = buffers.vertex_buffer.size();
        if self.vertex_count + (vertices.len() as u64 * self.vertex_size) > max_verts {
            grow_buffer(
                &mut buffers.vertex_buffer,
                wgpu,
                self.vertex_count + (vertices.len() as u64 * self.vertex_size),
                wgpu::BufferUsages::VERTEX,
            );
        }

        let max_indicies = buffers.index_buffer.size();
        if self.index_count + (indicies.len() as u64 * self.index_size) > max_indicies {
            grow_buffer(
                &mut buffers.index_buffer,
                wgpu,
                self.index_count + (indicies.len() as u64 * self.index_size),
                wgpu::BufferUsages::INDEX,
            );
        }

        wgpu.queue.write_buffer(
            &buffers.vertex_buffer,
            self.vertex_count,
            bytemuck::cast_slice(&vertices),
        );
        wgpu.queue.write_buffer(
            &buffers.index_buffer,
            self.index_count,
            bytemuck::cast_slice(&indicies),
        );

        self.vertex_count += vertices.len() as u64 * self.vertex_size;
        self.index_count += indicies.len() as u64 * self.index_size;
    }

    /// This will attempt to resize the texture stored within the shader.
    /// This will fail in the event that the shader has not loaded yet or
    /// if the shader used to create the material never had an UniformTexture.
    pub fn resize_uniform_texture(&mut self, texture: &mut UniformTexture, size: Vec2<u32>, engine: &mut Engine) -> Result<(), UniformError> {
        let context = match &engine.context {
            Some(c) => c,
            None => return Ok(()),
            // the context hasnt been created yet this also means there
            // is no reason for you to resize unless ur being silly for no
            // reason
        };
        
        let texture_format = context.get_texture_format();
        
        let options = match engine.resource_manager.get_mut_shader(&self.pipeline_id) {
            Some(shader) => shader,
            None => Err(UniformError::NotLoadedYet)?,
        };

        texture.resize(size, &context.wgpu, texture_format);

        options.update_uniform_texture(texture, &context.wgpu, texture_format)?;

        Ok(())
    }

    pub fn update_uniform_texture(&mut self, texture: &mut UniformTexture, engine: &mut Engine) -> Result<(), UniformError> {
        let context = match &engine.context {
            Some(c) => c,
            None => return Ok(()),
            // the context hasnt been created yet this also means there
            // is no reason for you to resize unless ur being silly for no
            // reason
        };

        let texture_format = context.get_texture_format();
        
        let options = match engine.resource_manager.get_mut_shader(&self.pipeline_id) {
            Some(shader) => shader,
            None => Err(UniformError::NotLoadedYet)?,
        };

        options.update_uniform_texture(texture, &context.wgpu, texture_format)?;
        
        Ok(())
    }

    /// Returns the number of verticies in the buffer
    pub fn get_vertex_number(&self) -> u64 {
        self.vertex_count / self.vertex_size
    }

    /// Returns the number if indincies in the buffer
    pub fn get_index_number(&self) -> u64 {
        self.index_count / self.index_size
    }

    /// Returns the size of the texture in pixels.
    /// Returns None when the texture is not loaded yet
    pub fn get_texture_size(&self, engine: &Engine) -> Option<Vec2<f32>> {
        engine.resource_manager.get_texture(&self.texture_id).and_then(|t| Some(t.size))
    }

    fn push_rectangle(&mut self, wgpu: &WgpuClump, verts: [Vertex; 4]) {
        if self.inner.is_none() {
            let (vert, ind) = Self::create_buffers(&wgpu.device, self.vertex_size, 50, self.index_size, 50);
            self.inner = Some(InnerBuffer {
                vertex_buffer: vert,
                index_buffer: ind,
            });
        }

        let num_verts = self.get_vertex_number() as u16;
        let buffers = self.inner.as_mut().unwrap();

        let max_verts = buffers.vertex_buffer.size();
        if self.vertex_count + (4 * self.vertex_size) > max_verts {
            grow_buffer(&mut buffers.vertex_buffer, wgpu, 1, wgpu::BufferUsages::VERTEX);
        }

        
        let indicies = [
            num_verts,
            1 + num_verts,
            2 + num_verts,
            3 + num_verts,
            num_verts,
            2 + num_verts,
        ];

        let max_indicies = buffers.index_buffer.size();
        if self.index_count + (6 * self.index_size) > max_indicies {
            grow_buffer(&mut buffers.index_buffer, wgpu, 1, wgpu::BufferUsages::INDEX);
        }

        wgpu.queue.write_buffer(
            &buffers.vertex_buffer,
            self.vertex_count,
            bytemuck::cast_slice(&verts),
        );
        wgpu.queue.write_buffer(
            &buffers.index_buffer,
            self.index_count,
            bytemuck::cast_slice(&indicies),
        );

        self.vertex_count += 4 * self.vertex_size;
        self.index_count += 6 * self.index_size;
    }

    fn push_triangle(&mut self, wgpu: &WgpuClump, verts: [Vertex; 3]) {
        if self.inner.is_none() {
            let (vert, ind) = Self::create_buffers(&wgpu.device, self.vertex_size, 50, self.index_size, 50);
            self.inner = Some(InnerBuffer {
                vertex_buffer: vert,
                index_buffer: ind,
            });
        }

        let num_verts = self.get_vertex_number() as u16;
        let buffers = self.inner.as_mut().unwrap();

        let max_verts = buffers.vertex_buffer.size();
        if self.vertex_count + (3 * self.vertex_size) > max_verts {
            grow_buffer(&mut buffers.vertex_buffer, wgpu, 1, wgpu::BufferUsages::VERTEX);
        }

        // yes its wastefull to do this but this is the only way to not have
        // it mess up other drawings while also allowing triangles
        let indicies = [
            num_verts,
            1 + num_verts,
            2 + num_verts,
            num_verts,
            1 + num_verts,
            2 + num_verts,
        ];

        let max_indicies = buffers.index_buffer.size();
        if self.index_count + (6 * self.index_size) > max_indicies {
            grow_buffer(&mut buffers.index_buffer, wgpu, 1, wgpu::BufferUsages::INDEX);
        }

        wgpu.queue.write_buffer(
            &buffers.vertex_buffer,
            self.vertex_count,
            bytemuck::cast_slice(&verts),
        );
        wgpu.queue.write_buffer(
            &buffers.index_buffer,
            self.index_count,
            bytemuck::cast_slice(&indicies),
        );

        self.vertex_count += 3 * self.vertex_size;
        self.index_count += 6 * self.index_size;
    }

    // there where 'others: 'pass notation says that 'others lives longer than 'pass
    /// Draws all queued shapes to the screen.
    pub fn draw<'pass, 'others>(
        &'others mut self,
        information: &mut Renderer<'pass, 'others>,
    ) {
        if self.vertex_count == 0 {
            return;
        }

        // returns early bc stuff inst loaded so we just ignore it ! :3
        let Some(shader) = information
            .resources
            .get_pipeline(&self.pipeline_id) else {
                return;
            };

        let Some(texture) = information
            .resources
            .get_texture(&self.texture_id)
            .map(|t| &t.bind_group) else {
                return;
            };
        // should never panic as the vertex == 0 means that there has been
        // some data put in which means this should be Some(T)
        let buffers = self.inner.as_ref().unwrap();

        shader.set_active(information);

        information.pass.set_bind_group(0, texture, &[]);

        information
            .pass
            .set_vertex_buffer(0, buffers.vertex_buffer.slice(0..self.vertex_count));
        information.pass.set_index_buffer(
            buffers.index_buffer.slice(0..self.index_count),
            wgpu::IndexFormat::Uint16,
        );

        information
            .pass
            .draw_indexed(0..self.get_index_number() as u32, 0, 0..1);

        self.vertex_count = 0;
        self.index_count = 0;
    }

    pub(crate) fn create_buffers(
        device: &wgpu::Device,
        vertex_size: u64,
        vert_count: u64,
        index_size: u64,
        index_count: u64,
    ) -> (wgpu::Buffer, wgpu::Buffer) {
        let vertex_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Vertex_Buffer"),
            size: vertex_size * vert_count,
            usage: wgpu::BufferUsages::VERTEX
                | wgpu::BufferUsages::COPY_DST
                | wgpu::BufferUsages::COPY_SRC,
            mapped_at_creation: false,
        });

        // this is just 200 bytes pretty small
        let index_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Index_Buffer"),
            size: index_size * index_count,
            usage: wgpu::BufferUsages::INDEX
                | wgpu::BufferUsages::COPY_DST
                | wgpu::BufferUsages::COPY_SRC,
            mapped_at_creation: false,
        });

        (vertex_buffer, index_buffer)
    }
}

impl<T: ShaderType + WriteInto> Material<T> {
    /// Attempts to update the uniform data held within in the shader.
    /// This will fail in the event that the shader has not loaded yet or
    /// if the shader used to create the material never had any UniformData,
    pub fn update_uniform_data(&self, data: &T, engine: &Engine) -> Result<(), UniformError> {
        let options = match engine.resource_manager.get_pipeline(&self.pipeline_id) {
            Some(shader) => shader,
            None => return Err(UniformError::NotLoadedYet),
        };

        options.update_uniform_data(data, engine)?;

        Ok(())
    }
}

/// A builder struct used to create Materials
pub struct MaterialBuilder<T> {
    // using options to denote a change from the default
    // in the case of a texture the defualt is just the White_Pixel
    texture_change: Option<ResourceId<Texture>>,
    shader_change: Option<ResourceId<Shader>>,
    _marker: PhantomData<T>
}

impl<T> MaterialBuilder<T> {
    /// Creates a new MaterialBuilder, that contains no texture, custom shaders, or
    /// uniforms
    pub fn new() -> Self {
        Self {
            texture_change: None,
            shader_change: None,
            _marker: PhantomData
        }
    }

    /// Adds a Texture to the Material
    pub fn add_texture(self, texture: ResourceId<Texture>) -> Self {
        Self {
            texture_change: Some(texture),
            shader_change: self.shader_change,
            _marker: PhantomData
        }
    }

    /// Sets the shader for the Material
    pub fn set_shader(self, shader: ResourceId<Shader>) -> Self {
        Self {
            texture_change: self.texture_change,
            shader_change: Some(shader),
            _marker: PhantomData
        }
    }

    /// This is used to set the type of data used for the materials Uniform ensuring type saftey across the GPU
    pub fn add_uniform_data<H: ShaderType + WriteInto>(self, _data: &UniformData<H>) -> MaterialBuilder<H> {
        MaterialBuilder {
            texture_change: self.texture_change,
            shader_change: self.shader_change,
            _marker: PhantomData,
        }
    }

    /// Turns the builder into a Material
    pub fn build(self, engine_handle: &mut Engine) -> Material<T> {
        Material::from_builder(self, engine_handle)
    }
}

impl<T> Default for MaterialBuilder<T> {
    fn default() -> Self {
        Self::new()
    }
}

/// A diffrent type of material used to draw WebGPU debug
/// lines. These lines will allways be 1px wide and only one
/// instance of this material should ever be made per programm.
pub struct LineMaterial {
    pipe_id: ResourceId<Shader>,
    vertex_buffer: Option<wgpu::Buffer>,
    vertex_count: u64,
    vertex_size: u64,
}

impl LineMaterial {
    /// Creates a new LineMaterial
    pub fn new(engine: &Engine) -> Self {
        let vertex_size = std::mem::size_of::<LineVertex>() as u64;

        Self {
            pipe_id: engine.line_pipe_id(),
            vertex_buffer: None,
            vertex_count: 0,
            vertex_size,
        }
    }

    /// Queues a line from the two points.
    pub fn add_line(
        &mut self,
        start: Vec2<f32>,
        end: Vec2<f32>,
        colour: Colour,
        renderer: &Renderer,
    ) {
        let wgpu = renderer.wgpu;
        if self.vertex_buffer.is_none() {
            self.vertex_buffer = Some(wgpu.device.create_buffer(&wgpu::BufferDescriptor {
                label: Some("Line material vertex buffer"),
                usage: wgpu::BufferUsages::VERTEX
                    | wgpu::BufferUsages::COPY_DST
                    | wgpu::BufferUsages::COPY_SRC,
                size: self.vertex_size * 100,
                mapped_at_creation: false,
            }));
        }

        
        let verts = [
            LineVertex::new(start.to_raw(), colour.as_raw()),
            LineVertex::new(end.to_raw(), colour.as_raw()),
        ];
            
        let vertex_buffer = self.vertex_buffer.as_mut().unwrap();

        let max_verts = vertex_buffer.size();
        let vert_size = std::mem::size_of::<LineVertex>() as u64;
        if self.vertex_count + (2 * vert_size) > max_verts {
            grow_buffer(vertex_buffer, wgpu, 1, wgpu::BufferUsages::VERTEX);
        }

        wgpu.queue.write_buffer(
            &vertex_buffer,
            self.vertex_count,
            bytemuck::cast_slice(&verts),
        );

        self.vertex_count += 2 * self.vertex_size;
    }

    pub fn add_screenspace_line(
        &mut self,
        start: Vec2<f32>,
        end: Vec2<f32>,
        colour: Colour,
        renderer: &Renderer,
    ) {
        let wgpu = renderer.wgpu;
        let size = renderer.size;

        if self.vertex_buffer.is_none() {
            self.vertex_buffer = Some(wgpu.device.create_buffer(&wgpu::BufferDescriptor {
                label: Some("Line material vertex buffer"),
                usage: wgpu::BufferUsages::VERTEX
                    | wgpu::BufferUsages::COPY_DST
                    | wgpu::BufferUsages::COPY_SRC,
                size: self.vertex_size * 100,
                mapped_at_creation: false,
            }));
        }

        let verts = [
            LineVertex::new(start.to_raw(), colour.as_raw()).screenspace_to_pixels(size),
            LineVertex::new(end.to_raw(), colour.as_raw()).screenspace_to_pixels(size),
        ];

        let vertex_buffer = self.vertex_buffer.as_mut().unwrap();

        let max_verts = vertex_buffer.size();
        let vert_size = std::mem::size_of::<LineVertex>() as u64;
        if self.vertex_count + (2 * vert_size) > max_verts {
            grow_buffer(vertex_buffer, wgpu, 1, wgpu::BufferUsages::VERTEX);
        }

        wgpu.queue.write_buffer(
            vertex_buffer,
            self.vertex_count,
            bytemuck::cast_slice(&verts),
        );

        self.vertex_count += 2 * self.vertex_size;
    }

    /// Draws all queued lines to the screen.
    pub fn draw<'pass, 'others>(
        &'others mut self,
        information: &mut Renderer<'pass, 'others>,
    ) {
        if self.vertex_count == 0 {
            return;
        }

        let Some(pipeline) = information
        .resources
        .get_pipeline(&self.pipe_id)
        .map(|p| &p.pipeline) else {
            return;
        };

        let buffer = self.vertex_buffer.as_ref().unwrap();

        information.pass.set_pipeline(pipeline);
        information
            .pass
            .set_bind_group(0, information.camera_bindgroup, &[]);
        information
            .pass
            .set_vertex_buffer(0, buffer.slice(0..self.vertex_count));

        information
            .pass
            .draw(0..self.get_vertex_count() as u32, 0..1);

        self.vertex_count = 0;
    }

    pub fn get_vertex_count(&self) -> u64 {
        self.vertex_count / self.vertex_size
    }
}

pub(crate) fn grow_buffer(
    buffer: &mut wgpu::Buffer,
    wgpu: &WgpuClump,
    size_needed: u64,
    vert_or_index: wgpu::BufferUsages,
) {
    let mut encoder = wgpu
        .device
        .create_command_encoder(&wgpu::CommandEncoderDescriptor {
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

    encoder.copy_buffer_to_buffer(buffer, 0, &new_buffer, 0, buffer.size());

    wgpu.queue.submit(std::iter::once(encoder.finish()));

    *buffer = new_buffer;
}

#[derive(Debug)]
struct InnerBuffer {
    vertex_buffer: wgpu::Buffer,
    index_buffer: wgpu::Buffer,
}