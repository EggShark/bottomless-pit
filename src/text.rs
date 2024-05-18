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
//!     fn render<'pass, 'others>(&'others mut self, mut render_handle: Renderer<'pass, 'others>) where 'others: 'pass {
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

use glyphon::fontdb::Source;
use glyphon::{
    self, Attrs, Family, FontSystem, Metrics, Shaping, SwashCache, TextArea, TextAtlas, TextBounds,
};

use crate::colour::Colour;
use crate::engine_handle::{Engine, WgpuClump};
use crate::layouts;
use crate::material::{self, Material};
use crate::matrix_math::normalize_points;
use crate::render::Renderer;
use crate::resource::{self, InProgressResource, ResourceId, ResourceManager, ResourceType};
use crate::vectors::Vec2;
use crate::vertex::{self, Vertex};

/// Stores Important information nessicary to rendering text. Only on of these should
/// be created per application.
pub(crate) struct TextRenderer {
    pub(crate) font_system: FontSystem,
    cache: SwashCache,
    atlas: TextAtlas,
    text_renderer: glyphon::TextRenderer,
    defualt_font_name: String,
}

impl TextRenderer {
    pub fn new(wgpu: &WgpuClump) -> Self {
        let mut font_system = FontSystem::new();
        let defualt_font_id = font_system
            .db_mut()
            .load_font_source(Source::Binary(Arc::new(include_bytes!("Monocraft.ttf"))))[0];

        let defualt_font_name = font_system.db().face(defualt_font_id).unwrap().families[0]
            .0
            .clone();
        // its expensive but whatever

        let cache = SwashCache::new();
        let mut atlas = TextAtlas::new(
            &wgpu.device,
            &wgpu.queue,
            wgpu::TextureFormat::Rgba8UnormSrgb,
        );
        let text_renderer = glyphon::TextRenderer::new(
            &mut atlas,
            &wgpu.device,
            wgpu::MultisampleState::default(),
            None,
        );

        Self {
            font_system,
            cache,
            atlas,
            text_renderer,
            defualt_font_name,
        }
    }

    pub fn get_defualt_font_name(&self) -> &str {
        &self.defualt_font_name
    }

    /// Loads in a font from a byte array
    pub fn load_font_from_bytes(&mut self, data: &[u8]) -> Font {
        let id = self
            .font_system
            .db_mut()
            .load_font_source(Source::Binary(Arc::new(data.to_vec())))[0];
        // I am lying to the user here

        let name = self.font_system.db().face(id).unwrap().families[0]
            .0
            .clone();

        Font { name }
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
    text: String,
    vertex_buffer: wgpu::Buffer,
    index_buffer: wgpu::Buffer,
    texture: wgpu::Texture,
    bind_group: wgpu::BindGroup,
    vertex_count: u64,
    index_count: u64,
}

impl TextMaterial {
    pub fn new(
        text: &str,
        colour: Colour,
        font_size: f32,
        line_height: f32,
        engine: &mut Engine,
    ) -> Self {
        let font_info = FontInformation::new(engine);

        let mut text_buffer = glyphon::Buffer::new(
            &mut font_info.text_handle.font_system,
            Metrics::new(font_size, line_height),
        );
        
        text_buffer.set_size(
            &mut font_info.text_handle.font_system,
            f32::MAX,
            f32::MAX,
        );

        text_buffer.set_text(
            &mut font_info.text_handle.font_system,
            text,
            Attrs::new()
                .color(colour.into())
                .family(Family::Name(&font_info.text_handle.defualt_font_name)),
            Shaping::Advanced,
        );

        let hieght = text_buffer.lines.len() as f32 * text_buffer.metrics().line_height;
        let run_width = text_buffer
            .layout_runs()
            .map(|run| run.line_w)
            .max_by(f32::total_cmp)
            .unwrap_or(0.0);

        let texture = font_info
            .wgpu
            .device
            .create_texture(&wgpu::TextureDescriptor {
                label: Some("Text Texture"),
                size: wgpu::Extent3d {
                    width: run_width.ceil() as u32,
                    height: hieght.ceil() as u32,
                    depth_or_array_layers: 1,
                },
                mip_level_count: 1,
                sample_count: 1,
                dimension: wgpu::TextureDimension::D2,
                format: wgpu::TextureFormat::Rgba8UnormSrgb,
                usage: wgpu::TextureUsages::RENDER_ATTACHMENT
                    | wgpu::TextureUsages::TEXTURE_BINDING,
                view_formats: &[],
            });

        let bounds = Vec2 {
            x: Vec2 { x: 0, y: 0 },
            y: Vec2 {
                x: i32::MAX,
                y: i32::MAX,
            },
        };

        let texture_size = Vec2 {
            x: run_width.ceil() as u32,
            y: hieght.ceil() as u32,
        };

        let texture_view = texture.create_view(&Default::default());

        render_text_to_texture(
            font_info.text_handle,
            &text_buffer,
            bounds,
            texture_size,
            &texture_view,
            font_info.wgpu,
        );

        let vertex_size = std::mem::size_of::<Vertex>() as u64;

        let (vertex_buffer, index_buffer) =
            Material::create_buffers(&engine.wgpu_clump.device, vertex_size, 16, 2, 32);

        let bind_group = engine
            .wgpu_clump
            .device
            .create_bind_group(&wgpu::BindGroupDescriptor {
                label: Some("Text Widget BindGroup"),
                layout: &layouts::create_texture_layout(&engine.wgpu_clump.device),
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
            text: text.into(),
            vertex_buffer,
            index_buffer,
            texture,
            bind_group,
            vertex_count: 0,
            index_count: 0,
        }
    }

    /// Sets the text for the widget, using the defualt font.
    /// This only needs to be done once, not every frame like Materials.
    pub fn set_text(&mut self, text: &str, colour: Colour, engine: &mut Engine) {
        let font_info = FontInformation::new(engine);
        self.text_buffer.set_text(
            &mut font_info.text_handle.font_system,
            text,
            Attrs::new()
                .color(colour.into())
                .family(Family::Name(&font_info.text_handle.defualt_font_name)),
            Shaping::Basic,
        );

        self.update_measurements(font_info);
    }

    /// Sets the text for the widget, but with a font of your choosing.
    /// This only needs to be done once, not every frame like Materials
    pub fn set_text_with_font(
        &mut self,
        text: &str,
        colour: Colour,
        font: &ResourceId<Font>,
        engine: &mut Engine,
    ) {
        let font_info = FontInformation::new(engine);

        let backup = &String::new();
        let name = font_info
            .resources
            .get_font(font)
            .map(|f| &f.name)
            .unwrap_or(backup);

        self.text_buffer.set_text(
            &mut font_info.text_handle.font_system,
            text,
            Attrs::new().color(colour.into()).family(Family::Name(name)),
            Shaping::Basic,
        );

        self.update_measurements(font_info);
    }

    /// Sets bounds for the text. Any text drawn outside of the bounds will be cropped
    pub fn set_bounds(&mut self, position: Vec2<i32>, size: Vec2<i32>) {
        self.bounds = Vec2 {
            x: position,
            y: size,
        };
    }

    /// Sets the font size of the text
    pub fn set_font_size(&mut self, new_size: f32, engine: &mut Engine) {
        self.font_size = new_size;
        let font_info = FontInformation::new(engine);
        let metrics = Metrics::new(self.font_size, self.line_height);
        self.text_buffer
            .set_metrics(&mut font_info.text_handle.font_system, metrics);

        self.update_measurements(font_info);
    }

    /// Sets the line hieght of the tex
    pub fn set_line_height(&mut self, new_height: f32, engine: &mut Engine) {
        self.line_height = new_height;
        let font_info = FontInformation::new(engine);
        let metrics = Metrics::new(self.font_size, self.line_height);
        self.text_buffer
            .set_metrics(&mut font_info.text_handle.font_system, metrics);

        self.update_measurements(font_info);
    }

    /// Measuers the text contained within the widget
    pub fn get_measurements(&self) -> Vec2<u32> {
        self.size
    }

    pub fn get_text(&self) -> &str {
        &self.text
    }

    fn update_measurements(&mut self, font_info: FontInformation) {
        let hieght = self.text_buffer.lines.len() as f32 * self.text_buffer.metrics().line_height;
        let run_width = self
            .text_buffer
            .layout_runs()
            .map(|run| run.line_w)
            .max_by(f32::total_cmp)
            .unwrap_or(0.0);

        self.size = Vec2 {
            x: run_width.ceil() as u32,
            y: hieght.ceil() as u32,
        };

        self.texture = font_info.wgpu.device.create_texture(&wgpu::TextureDescriptor {
            label: Some("Text Texture"),
            size: wgpu::Extent3d {
                width: self.size.x,
                height: self.size.y,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Rgba8UnormSrgb,
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::TEXTURE_BINDING,
            view_formats: &[],
        });
    }

    /// Queues a peice of text at the specified position. Its size will be the size of the entire
    /// text.
    pub fn add_instance(&mut self, position: Vec2<f32>, tint: Colour, render: &Renderer) {
        let rect_size = Vec2 {
            x: self.size.x as f32,
            y: self.size.y as f32,
        };
        let wgpu = render.wgpu;
        let verts = vertex::from_pixels(position, rect_size, tint.as_raw());

        self.push_rectangle(wgpu, verts);
    }

    /// Queues a piece of text at the specified postion, with rotation. Its size will be the size of the
    /// text.
    pub fn add_instance_with_rotation(
        &mut self,
        position: Vec2<f32>,
        tint: Colour,
        degrees: f32,
        render: &Renderer,
    ) {
        let rect_size = Vec2 {
            x: self.size.x as f32,
            y: self.size.y as f32,
        };
        let wgpu = render.wgpu;
        let verts = vertex::from_pixels_with_rotation(
            position,
            rect_size,
            tint.as_raw(),
            degrees,
        );

        self.push_rectangle(wgpu, verts);
    }

    /// Queues a piece of text at the specified postion. This also allows you to control the uv coordinates to the texture
    /// that the text has been rendered to.
    pub fn add_instance_with_uv(
        &mut self,
        position: Vec2<f32>,
        size: Vec2<f32>,
        uv_pos: Vec2<f32>,
        uv_size: Vec2<f32>,
        tint: Colour,
        render: &Renderer,
    ) {
        let wgpu = render.wgpu;

        let uv_pos = normalize_points(
            uv_pos,
            Vec2 {
                x: self.size.x as f32,
                y: self.size.y as f32,
            },
        );
        let uv_size = normalize_points(
            uv_size,
            Vec2 {
                x: self.size.x as f32,
                y: self.size.y as f32,
            },
        );

        let verts = vertex::from_pixels_with_uv(
            position,
            size,
            tint.as_raw(),
            uv_pos,
            uv_size,
        );

        self.push_rectangle(wgpu, verts);
    }

    #[allow(clippy::too_many_arguments)]
    /// Queues a piece of text at the position with, uv controll, and rotation.
    pub fn add_instace_ex(
        &mut self,
        position: Vec2<f32>,
        size: Vec2<f32>,
        uv_pos: Vec2<f32>,
        uv_size: Vec2<f32>,
        degrees: f32,
        tint: Colour,
        render: &Renderer,
    ) {
        let wgpu = render.wgpu;

        let uv_pos = normalize_points(
            uv_pos,
            Vec2 {
                x: self.size.x as f32,
                y: self.size.y as f32,
            },
        );
        let uv_size = normalize_points(
            uv_size,
            Vec2 {
                x: self.size.x as f32,
                y: self.size.y as f32,
            },
        );

        let verts = vertex::from_pixels_ex(
            position,
            size,
            tint.as_raw(),
            degrees,
            uv_pos,
            uv_size,
        );

        self.push_rectangle(wgpu, verts);
    }

    /// Queues a peice with complete controll over the points, rotation, and uv coordinates. This can allow for non rectanglular shapes
    /// and the points must be in top left, top right, bottom right, bottom left order otherwise it will not draw properly.
    pub fn add_instance_custom(
        &mut self,
        points: [Vec2<f32>; 4],
        uv_points: [Vec2<f32>; 4],
        degrees: f32,
        tint: Colour,
        render: &Renderer,
    ) {
        let wgpu = render.wgpu;

        let uv_points = [
            normalize_points(
                uv_points[0],
                Vec2 {
                    x: self.size.x as f32,
                    y: self.size.y as f32,
                },
            ),
            normalize_points(
                uv_points[1],
                Vec2 {
                    x: self.size.x as f32,
                    y: self.size.y as f32,
                },
            ),
            normalize_points(
                uv_points[2],
                Vec2 {
                    x: self.size.x as f32,
                    y: self.size.y as f32,
                },
            ),
            normalize_points(
                uv_points[3],
                Vec2 {
                    x: self.size.x as f32,
                    y: self.size.y as f32,
                },
            ),
        ];

        let verts =
            vertex::from_pixels_custom(points, uv_points, degrees, tint.as_raw());

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
            num_verts,
            1 + num_verts,
            2 + num_verts,
            3 + num_verts,
            num_verts,
            2 + num_verts,
        ];

        let max_indicies = self.index_buffer.size();
        if self.index_count + (6 * index_size) > max_indicies {
            material::grow_buffer(&mut self.index_buffer, wgpu, 1, wgpu::BufferUsages::INDEX);
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

    pub fn prepare(&mut self, engine: &mut Engine) {
        let font_info = FontInformation::new(engine);
        let texture_view = self.texture.create_view(&wgpu::TextureViewDescriptor::default());
        render_text_to_texture(
            font_info.text_handle,
            &self.text_buffer,
            self.bounds,
            self.size,
            &texture_view,
            font_info.wgpu,
        );

        drop(font_info);

        self.bind_group = engine.wgpu_clump.device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("Text Widget BindGroup"),
            layout: &layouts::create_texture_layout(&engine.wgpu_clump.device),
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
    }

    /// Draws all queued text instances to the screen
    pub fn draw<'pass, 'others>(
        &'others mut self,
        information: &mut Renderer<'pass, 'others>,
    ) {
        let pipeline = &information
            .resources
            .get_pipeline(&information.defualt_id)
            .unwrap()
            .pipeline;

        information.pass.set_pipeline(pipeline);
        information
            .pass
            .set_bind_group(0, &self.bind_group, &[]);

        information
            .pass
            .set_vertex_buffer(0, self.vertex_buffer.slice(0..self.vertex_count));
        information.pass.set_index_buffer(
            self.index_buffer.slice(0..self.index_count),
            wgpu::IndexFormat::Uint16,
        );

        information
            .pass
            .draw_indexed(0..self.get_index_number() as u32, 0, 0..1);

        self.vertex_count = 0;
        self.index_count = 0;
    }
}

// fun hacky thing to get around BC
pub(crate) struct FontInformation<'a> {
    text_handle: &'a mut TextRenderer,
    wgpu: &'a WgpuClump,
    resources: &'a ResourceManager,
}

impl<'a> FontInformation<'a> {
    pub fn new(engine: &'a mut Engine) -> Self {
        Self {
            text_handle: &mut engine.text_renderer,
            wgpu: &engine.wgpu_clump,
            resources: &engine.resource_manager,
        }
    }
}

/// A struct to ensure Font Families are created properly.
pub struct Font {
    name: String,
}

impl Font {
    /// Attempts to load in a Shader from file. This will halt the engine for more information on
    /// this halting behavior check the [resource module](crate::resource).
    pub fn new<P: AsRef<Path>>(path: P, engine: &mut Engine) -> ResourceId<Font> {
        let typed_id = resource::generate_id::<Font>();
        let id = typed_id.get_id();
        let path = path.as_ref();
        let ip_resource = InProgressResource::new(path, id, ResourceType::Font);

        resource::start_load(engine, path, ip_resource);

        engine.add_in_progress_resource();
        typed_id
    }

    /// Attempts to load in a Shader from a byte array. This will not halt the
    /// engine, please check the [resource module](crate::resource) for more information.
    pub fn from_bytes(data: &[u8], engine: &mut Engine) -> ResourceId<Font> {
        let font = engine.text_renderer.load_font_from_bytes(data);
        let typed_id = resource::generate_id::<Font>();

        engine.resource_manager.insert_font(typed_id, font);

        typed_id
    }

    pub(crate) fn from_str(name: &str) -> Self {
        Self {
            name: name.to_string(),
        }
    }
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
        buffer: text,
        left: 0.0,
        top: 0.0,
        scale: 1.0,
        bounds: TextBounds {
            left: bounds.x.x,
            top: bounds.x.y,
            right: bounds.y.x,
            bottom: bounds.y.y,
        },
        default_color: glyphon::Color::rgb(255, 255, 255),
    };

    text_handle
        .text_renderer
        .prepare(
            &wgpu.device,
            &wgpu.queue,
            &mut text_handle.font_system,
            &mut text_handle.atlas,
            texture_size.into(),
            [text_area],
            &mut text_handle.cache,
        )
        .unwrap_or(());

    let mut text_encoder = wgpu
        .device
        .create_command_encoder(&wgpu::CommandEncoderDescriptor {
            label: Some("text encoder"),
        });

    let mut text_pass = text_encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
        label: Some("Text to Texture Pass"),
        color_attachments: &[Some(wgpu::RenderPassColorAttachment {
            view: texture_view,
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
        depth_stencil_attachment: None,
    });

    text_handle
        .text_renderer
        .render(&text_handle.atlas, &mut text_pass)
        .unwrap();

    drop(text_pass);
    text_handle.atlas.trim();

    wgpu.queue.submit(std::iter::once(text_encoder.finish()));
}
