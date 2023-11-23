//! Contains the Text Struct used for text widgets and the TextRender which is needed for rendering 
//! Text widgets
//! 
//! How Text Rendering Works
//! ---
//! Text rendering works diffrently compared to standard rendering. One of the main diffrences is that
//! you do not need to set the text buffer every single frame unlike [Material](../material/struct.Material.html)
//! buffers.
//!  
//! Example:
//! ```rust,no_run
//! fn main() {
//!     // assuming you've made a engine handle already
//!     let mut text_handle = TextRenderer::new(&engine);
//!     let text_mat = TextMaterial::new("Hello World", Colour::RED, 100.0, 100.0, &mut text_render, &engine);
//!     
//!     // construct any struct that implements Game
//!     let game = UserStruct::new(text, text_buffer);
//! 
//!     egnine.run(game);
//! }
//! 
//! impl Game for UserStruct {
//!     fn render<'pass, 'others>(&'others mut self, mut render_handle: RenderInformation<'pass, 'others>) where 'others: 'pass {
//!         // notice how you dont need to set the text every frame, and works the same as a Material
//!         self.text_mat.add_instance(Vec2{x: 0.0, y: 0.0}, Colour::WHITE, &render_handle);
//! 
//!         self.text_mat.draw(&mut self.text_handle, &mut render_handle);
//!     }
//! 
//!     fn update(&mut self, engine_handle: &mut Engine) {
//!         // nothing, you dont need to update the text buffer every frame
//!     }
//! }
//! 
//! ```
use std::path::Path;
use std::sync::Arc;

use crate::glyphon::fontdb::Source;
use crate::glyphon::{self, FontSystem, SwashCache, TextAtlas, TextArea, TextBounds, Metrics, Attrs, Shaping, Family};

use crate::colour::Colour;
use crate::engine_handle::{Engine, WgpuClump};
use crate::layouts;
use crate::material::{self, Material};
use crate::matrix_math::normalize_points;
use crate::render::RenderInformation;
use crate::vectors::Vec2;
use crate::vertex::{self, Vertex};

/// Stores Important information nessicary to rendering text. Only on of these should
/// be created per application.
pub struct TextRenderer {
    font_system: FontSystem,
    cache: SwashCache,
    atlas: TextAtlas,
    text_renderer: glyphon::TextRenderer,
    defualt_font_name: String,
}

impl TextRenderer {
    pub fn new(engine: &Engine) -> Self {
        let wgpu = engine.get_wgpu();
        let mut font_system = FontSystem::new();
        let defualt_font_id = font_system
            .db_mut()
            .load_font_source(Source::Binary(Arc::new(include_bytes!("Monocraft.ttf"))))[0];

        let defualt_font_name = font_system
            .db()
            .face(defualt_font_id)
            .unwrap()
            .families[0]
            .0
            .clone();
        // its expensive but whatever

        let cache = SwashCache::new();
        let mut atlas = TextAtlas::new(&wgpu.device, &wgpu.queue, wgpu::TextureFormat::Rgba8UnormSrgb);
        let text_renderer = glyphon::TextRenderer::new(&mut atlas, &wgpu.device, wgpu::MultisampleState::default(), None);

        Self {
            font_system,
            cache,
            atlas,
            text_renderer,
            defualt_font_name,
        }
    }

    /// Loads in a new font from a file
    pub fn load_font_file<P: AsRef<Path>>(&mut self, path: P) -> Result<Font, std::io::Error> {
        let data = std::fs::read(path)?;
        let id = self
            .font_system
            .db_mut()
            .load_font_source(Source::Binary(Arc::new(data)))[0];

        let name = self.font_system
            .db()
            .face(id)
            .unwrap()
            .families[0]
            .0
            .clone();

        Ok(Font {
            name,
        })
    }

    /// Loads in a font from a byte array
    pub fn load_font_from_bytes(&mut self, data: &[u8]) -> Font {
        let id = self
            .font_system
            .db_mut()
            .load_font_source(Source::Binary(Arc::new(data.to_vec())))[0];
        // I am lying to the user here 

        let name = self.font_system
            .db()
            .face(id)
            .unwrap()
            .families[0]
            .0
            .clone();

        Font {
            name
        }
    }

    /// Takes an &str with measurments and gives out the width and hieght of the possible text
    pub fn measure_str(&mut self, text: &str, font_size: f32, line_height: f32, engine: &Engine) -> Vec2<f32> {
        let mut buffer = glyphon::Buffer::new(&mut self.font_system, Metrics::new(font_size, line_height));
        let size = engine.get_window_size();
        let scale_factor = engine.get_window_scale_factor();
        let phyisical_width = (size.x as f64 * scale_factor) as f32;
        let phyiscal_hieght = (size.y as f64 * scale_factor) as f32;

        buffer.set_size(&mut self.font_system, phyisical_width, phyiscal_hieght);
        buffer.set_text(&mut self.font_system, text, Attrs::new(), Shaping::Basic);

        let hieght = buffer.lines.len() as f32 * buffer.metrics().line_height;
        let run_width = buffer.layout_runs().map(|run| run.line_w).max_by(f32::total_cmp).unwrap_or(0.0);

        Vec2{x: run_width, y: hieght}
    }
}

/// This struct represents a piece of text. You only need to create 
/// one peice of text per string you would like to draw
/// as you can draw multpiple instances easily.
pub struct TextMaterial {
    size: Vec2<u32>,
    font_size: f32,
    line_height: f32,
    bounds: Vec2<Vec2<i32>>,
    text_buffer: glyphon::Buffer,
    vertex_buffer: wgpu::Buffer,
    index_buffer: wgpu::Buffer,
    texture: wgpu::Texture,
    texture_view: wgpu::TextureView,
    bind_group: wgpu::BindGroup,
    vertex_count: u64,
    index_count: u64,
    change_flag: bool,
}

impl TextMaterial {
    pub fn new(
        text: &str,
        colour: Colour,
        font_size: f32,
        line_height: f32,
        text_handle: &mut TextRenderer,
        engine: &Engine
    ) -> Self {
        let mut text_buffer = glyphon::Buffer::new(&mut text_handle.font_system, Metrics::new(font_size, line_height));
        let window_size = engine.get_window_size();
        let scale_factor = engine.get_window_scale_factor();
        let wgpu = engine.get_wgpu();
        let phyisical_width = (window_size.x as f64 * scale_factor) as f32;
        let phyiscal_hieght = (window_size.y as f64 * scale_factor) as f32;

        text_buffer.set_size(&mut text_handle.font_system, phyisical_width, phyiscal_hieght);
        text_buffer.set_text(
            &mut text_handle.font_system,
            text,
            Attrs::new().color(colour.into()).family(Family::Name(&text_handle.defualt_font_name)),
            Shaping::Basic
        );

        let hieght = text_buffer.lines.len() as f32 * text_buffer.metrics().line_height;
        let run_width = text_buffer
            .layout_runs()
            .map(|run| run.line_w)
            .max_by(f32::total_cmp).unwrap_or(0.0);


        let texture = wgpu.device.create_texture(&wgpu::TextureDescriptor {
            label: Some("Text Texture"),
            size: wgpu::Extent3d {
                width: hieght.ceil() as u32,
                height: run_width.ceil() as u32,
                depth_or_array_layers: 1
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Rgba8UnormSrgb,
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::TEXTURE_BINDING,
            view_formats: &[],
        });

        let bounds = Vec2{x: Vec2{x: 0, y: 0}, y: Vec2{x: i32::MAX, y: i32::MAX}};
        let texture_size = Vec2{x: run_width.ceil() as u32, y: hieght.ceil() as u32};
        let texture_view = texture.create_view(&Default::default());
    
        render_text_to_texture(text_handle, &text_buffer, bounds, texture_size, &texture_view, wgpu);
        let vertex_size = std::mem::size_of::<Vertex>() as u64;

        let (vertex_buffer, index_buffer) = Material::create_buffers(&wgpu.device, vertex_size, 8, 2, 12);

        let bind_group = wgpu.device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("Text Widget BindGroup"),
            layout: &layouts::create_texture_layout(&wgpu.device),
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::TextureView(&texture_view),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::Sampler(engine.get_texture_sampler()),
                },
            ],
        });

        Self {
            size: texture_size,
            font_size,
            line_height,
            bounds,
            text_buffer,
            vertex_buffer,
            index_buffer,
            texture_view,
            texture: texture,
            bind_group,
            vertex_count: 0,
            index_count: 0,
            change_flag: false,
        }
    }

    /// Sets the text for the widget, using the defualt font.
    /// This only needs to be done once, not every frame like Materials.
    pub fn set_text(&mut self, text: &str, colour: Colour, text_handle: &mut TextRenderer, engine: &Engine) {
        self.text_buffer.set_text(
            &mut text_handle.font_system,
            text,
            Attrs::new().color(colour.into()).family(Family::Name(&text_handle.defualt_font_name)),
            Shaping::Basic
        );

        self.update_measurements(engine.get_wgpu());
    }

    /// Sets the text for the widget, but with a font of your choosing.
    /// This only needs to be done once, not every frame like Materials
    pub fn set_text_with_font(&mut self, text: &str, colour: Colour, font: &Font, text_handle: &mut TextRenderer, engine: &Engine) {
        self.text_buffer.set_text(
            &mut text_handle.font_system,
            text,
            Attrs::new().color(colour.into()).family(Family::Name(&font.name)),
            Shaping::Basic
        );

        self.update_measurements(engine.get_wgpu());
    }

    /// Sets bounds for the text. Any text drawn outside of the bounds will be cropped
    pub fn set_bounds(&mut self, position: Vec2<i32>, size: Vec2<i32>) {
        self.bounds = Vec2{x: position, y: size};
    }

    /// Sets the font size of the text
    pub fn set_font_size(&mut self, new_size: f32, text_handle: &mut TextRenderer, engine: &Engine) {
        self.font_size = new_size;
        let metrics = Metrics::new(self.font_size, self.line_height);
        self.text_buffer.set_metrics(&mut text_handle.font_system, metrics);

        self.update_measurements(engine.get_wgpu());
    }

    /// Sets the line hieght of the tex
    pub fn set_line_height(&mut self, new_height: f32, text_handle: &mut TextRenderer, engine: &Engine) {
        self.line_height = new_height;
        let metrics = Metrics::new(self.font_size, self.line_height);
        self.text_buffer.set_metrics(&mut text_handle.font_system, metrics);

        self.update_measurements(engine.get_wgpu());
    }

    /// Measuers the text contained within the widget
    pub fn get_measurements(&self) -> Vec2<u32> {
        self.size
    }

    fn update_measurements(&mut self, wgpu: &WgpuClump) {
        let hieght = self.text_buffer.lines.len() as f32 * self.text_buffer.metrics().line_height;
        let run_width = self.text_buffer
            .layout_runs()
            .map(|run| run.line_w)
            .max_by(f32::total_cmp)
            .unwrap_or(0.0);

        self.size = Vec2{x: run_width.ceil() as u32, y: hieght.ceil() as u32};

        self.texture = wgpu.device.create_texture(&wgpu::TextureDescriptor {
            label: Some("Text Texture"),
            size: wgpu::Extent3d {
                width: self.size.x,
                height: self.size.y,
                depth_or_array_layers: 1
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Rgba8UnormSrgb,
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::TEXTURE_BINDING,
            view_formats: &[],
        });

        self.change_flag = true;
    }

    /// Queues a peice of text at the specified position. Its size will be the size of the entire
    /// text.
    pub fn add_instance(&mut self, position: Vec2<f32>, tint: Colour, render: &RenderInformation) {
        let window_size = render.size;
        let rect_size = Vec2{x: self.size.x as f32, y: self.size.y as f32};
        let wgpu = render.wgpu;
        let verts =
            vertex::from_pixels(position, rect_size, tint.to_raw(), window_size);

        self.push_rectangle(wgpu, verts);
    }

    /// Queues a piece of text at the specified postion, with rotation. Its size will be the size of the
    /// text.
    pub fn add_instance_with_rotation(&mut self, position: Vec2<f32>, tint: Colour, degrees: f32, render: &RenderInformation) {
        let window_size = render.size;
        let rect_size = Vec2{x: self.size.x as f32, y: self.size.y as f32};
        let wgpu = render.wgpu;
        let verts =
            vertex::from_pixels_with_rotation(position, rect_size, tint.to_raw(), window_size, degrees);

        self.push_rectangle(wgpu, verts);
    }

    /// Queues a piece of text at the specified postion. This also allows you to control the uv coordinates to the texture
    /// that the text has been rendered to.
    pub fn add_instance_with_uv(&mut self, position: Vec2<f32>, size: Vec2<f32>, uv_pos: Vec2<f32>, uv_size: Vec2<f32>, tint: Colour, render: &RenderInformation) {
        let window_size = render.size;
        let wgpu = render.wgpu;

        let uv_pos = normalize_points(uv_pos, Vec2{x: self.size.x as f32, y: self.size.y as f32});
        let uv_size = normalize_points(uv_size, Vec2{x: self.size.x as f32, y: self.size.y as f32});

        let verts =
            vertex::from_pixels_with_uv(position, size, tint.to_raw(), window_size, uv_pos, uv_size);

        self.push_rectangle(wgpu, verts);
    }

    /// Queues a piece of text at the position with, uv controll, and rotation.
    pub fn add_instace_ex(
        &mut self,
        position: Vec2<f32>,
        size: Vec2<f32>,
        uv_pos: Vec2<f32>,
        uv_size: Vec2<f32>,
        degrees: f32,
        tint: Colour,
        render: &RenderInformation
    ) {
        let window_size = render.size;
        let wgpu = render.wgpu;

        let uv_pos = normalize_points(uv_pos, Vec2{x: self.size.x as f32, y: self.size.y as f32});
        let uv_size = normalize_points(uv_size, Vec2{x: self.size.x as f32, y: self.size.y as f32});


        let verts = 
            vertex::from_pixels_ex(position, size, tint.to_raw(), window_size, degrees, uv_pos, uv_size);

        self.push_rectangle(wgpu, verts);
    }

    /// Queues a peice with complete controll over the points, rotation, and uv coordinates. This can allow for non rectanglular shapes
    /// and the points must be in top left, top right, bottom right, bottom left order otherwise it will not draw properly.
    pub fn add_instance_custom(&mut self, points: [Vec2<f32>; 4], uv_points: [Vec2<f32>; 4], degrees: f32, tint: Colour, render: &RenderInformation) {
        let window_size = render.size;
        let wgpu = render.wgpu;

        let uv_points = [
            normalize_points(uv_points[0], Vec2{x: self.size.x as f32, y: self.size.y as f32}),
            normalize_points(uv_points[1], Vec2{x: self.size.x as f32, y: self.size.y as f32}),
            normalize_points(uv_points[2], Vec2{x: self.size.x as f32, y: self.size.y as f32}),
            normalize_points(uv_points[3], Vec2{x: self.size.x as f32, y: self.size.y as f32}),
        ];

        let verts = 
            vertex::from_pixels_custom(points, uv_points, degrees, tint.to_raw(), window_size);

        self.push_rectangle(wgpu, verts);
    }

    fn push_rectangle(&mut self, wgpu: &WgpuClump, verts: [Vertex; 4]) {
        let vertex_size = std::mem::size_of::<Vertex>() as u64;
        let index_size = 2;

        let max_verts = self.vertex_buffer.size();
        if self.vertex_count + (4 * vertex_size) > max_verts {
            material::grow_buffer(&mut self.vertex_buffer, wgpu, 1, wgpu::BufferUsages::VERTEX);
        }

        let num_verts = self.get_vertex_number() as u16;
        let indicies = [
            num_verts, 1 + num_verts, 2 + num_verts,
            3 + num_verts, num_verts, 2 + num_verts,
        ];

        let max_indicies = self.index_buffer.size();
        if self.index_count + (6 * index_size) > max_indicies {
            material::grow_buffer(&mut self.vertex_buffer, wgpu, 1, wgpu::BufferUsages::INDEX);
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

        self.vertex_count += 4 * vertex_size;
        self.index_count += 6 * index_size;
    }

    fn get_vertex_number(&self) -> u64 {
        self.vertex_count / std::mem::size_of::<Vertex>() as u64
    }

    fn get_index_number(&self) -> u64 {
        self.index_count / 2
    }

    /// Draws all queued text instances to the screen
    pub fn draw<'pass, 'others>(&'others mut self, text_handle: &mut TextRenderer, information: &mut RenderInformation<'pass, 'others>) where 'others: 'pass, {
        if self.change_flag {
            render_text_to_texture(text_handle, &self.text_buffer, self.bounds, self.size, &self.texture_view, information.wgpu);
            self.change_flag = false;
        }
        
        let pipeline = &information
            .resources
            .get_pipeline(&information.defualt_id)
            .unwrap()
            .pipeline;

        information.render_pass.set_pipeline(pipeline);
        information.render_pass.set_bind_group(0, &self.bind_group, &[]);

        information.render_pass.set_vertex_buffer(0, self.vertex_buffer.slice(0..self.get_vertex_number()));
        information.render_pass.set_index_buffer(
            self.index_buffer.slice(0..self.index_count),
            wgpu::IndexFormat::Uint16
        );

        information.render_pass.draw_indexed(0..self.get_index_number() as u32, 0, 0..1);

        self.vertex_count = 0;
        self.index_count = 0;
    }
}

/// A struct to ensure Font Families are created properly
/// can be created by [load_font_file](struct.TextRenderer.html#method.load_font_file)
/// or [load_font_data](struct.TextRenderer.html#method.load_font_file)
pub struct Font {
    name: String
}

fn render_text_to_texture(
    text_handle: &mut TextRenderer,
    text: &glyphon::Buffer,
    bounds: Vec2<Vec2<i32>>,
    texture_size: Vec2<u32>,
    texture_view: &wgpu::TextureView,
    wgpu: &WgpuClump,
) {
    let text_area = TextArea {
        buffer: &text,
        left: 0.0,
        top: 0.0,
        scale: 1.0,
        bounds: TextBounds {
            left: bounds.x.x,
            top: bounds.x.y,
            right: bounds.y.x,
            bottom: bounds.y.y
        },
        default_color: glyphon::Color::rgb(255, 255, 255),
    };


    text_handle.text_renderer.prepare(
        &wgpu.device,
        &wgpu.queue,
        &mut text_handle.font_system,
        &mut text_handle.atlas,
        texture_size.into(),
        [text_area],
        &mut text_handle.cache,
    ).unwrap_or(());

    let mut text_encoder = wgpu.device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
        label: Some("text encoder"),
    });

    let mut text_pass = text_encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
        label: Some("Text to Texture Pass"),
        color_attachments: &[Some(wgpu::RenderPassColorAttachment {
            view: &texture_view,
            resolve_target: None,
            ops: wgpu::Operations {
                load: wgpu::LoadOp::Clear(wgpu::Color {
                    r: 0.0,
                    g: 0.0,
                    b: 0.0,
                    a: 0.0,
                }),
                store: wgpu::StoreOp::Store,
            },
        })],
        timestamp_writes: None,
        occlusion_query_set: None,
        depth_stencil_attachment: None
    });

    text_handle.text_renderer.render(&text_handle.atlas, &mut text_pass).unwrap();

    drop(text_pass);
    text_handle.atlas.trim();
    
    wgpu.queue.submit(std::iter::once(text_encoder.finish()));
}