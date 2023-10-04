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
//!     let mut text = Text::new(Vec2{x: 0.0, y: 0.0}, 30.0, 28.0, &mut text_handle, &engine);
//!     text.set_text("Hello World", Color::WHITE, &mut text_handle);
//!     
//!     // construct any struct that implements Game
//!     let game = UserStruct::new(text, text_buffer);
//! 
//!     egnine.run(game);
//! }
//! 
//! impl Game for UserStruct {
//!     fn render<'pass, 'others>(&'others mut self, mut render_handle: RenderInformation<'pass, 'others>) where 'others: 'pass {
//!         // notice how you dont need to set the text every frame
//!         self.text_handle.draw_text(&self.text, &mut render_handle);
//!     }
//! 
//!     fn update(&mut self, engine_handle: &mut Engine) {
//!         // nothing, you dont need to update the text buffer
//!     }
//! }
//! 
//! ```

use std::fmt::format;
use std::path::Path;
use std::sync::Arc;

use glyphon::fontdb::Source;
use glyphon::{FontSystem, SwashCache, TextAtlas, TextArea, TextBounds, Metrics, Attrs, Shaping, Family};

use crate::colour::Colour;
use crate::engine_handle::Engine;
use crate::render::RenderInformation;
use crate::vectors::Vec2;
use crate::vertex::Vertex;

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
        let mut atlas = TextAtlas::new(&wgpu.device, &wgpu.queue, wgpu::TextureFormat::Bgra8UnormSrgb);
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

    /// Loads in a font from a byte vector
    pub fn load_font_from_bytes(&mut self, data: Vec<u8>) -> Font {
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

    /// Prepares [Text](struct.Text.html) widgets for rendering, this MUST be called if you want the text to 
    /// appear on screen when calling [render_text](#method.render_text).
    pub fn prepare_texts(&mut self, texts: &[&Text], renderer: &RenderInformation<'_, '_>) {
        let device = &renderer.wgpu.device;
        let queue = &renderer.wgpu.queue;
        self.text_renderer.prepare(
            device,
            queue,
            &mut self.font_system,
            &mut self.atlas,
            renderer.size.into(),
            texts.iter().map(|t| t.into_text_area()),
            &mut self.cache,
        ).unwrap();
    }

    /// Renders prepared text to the srceen pelase see at the [prepare](#method.prepare_texts)
    /// function.
    pub fn render_text<'pass, 'others>(&'others mut self, renderer: &mut RenderInformation<'pass, 'others>) where 'others: 'pass {
        self.text_renderer.render(&self.atlas, &mut renderer.render_pass).unwrap();
    }

    pub async fn render_texts_to_image(&mut self, texts: &[&Text], renderer: &RenderInformation<'_, '_>) {
        let wgpu = renderer.wgpu;

        let mut text_encoder = wgpu.device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
            label: Some("text encoder"),
        });

        let textures = texts
            .iter()
            .map(|t| t.get_measurements())
            .map(|size| {
                wgpu.device.create_texture(&wgpu::TextureDescriptor {
                    label: Some("Text Texture"),
                    size: wgpu::Extent3d {
                        width: size.x.ceil() as u32,
                        height: size.y.ceil() as u32,
                        depth_or_array_layers: 1,
                    },
                    mip_level_count: 1,
                    sample_count: 1,
                    dimension: wgpu::TextureDimension::D2,
                    format: wgpu::TextureFormat::Bgra8UnormSrgb,
                    usage: wgpu::TextureUsages::COPY_SRC | wgpu::TextureUsages::RENDER_ATTACHMENT,
                    view_formats: &[],
                })
            })
            .collect::<Vec<wgpu::Texture>>();

        let buffers = texts
            .iter()
            .map(|t| t.size)
            .map(|size| {
                let width = size.x.ceil() as u64;
                let buffer_size = size.y.ceil() as u64 * ((4*width) + (256 - (4*width % 256)));
                wgpu.device.create_buffer(&wgpu::BufferDescriptor {
                    label: Some("text texture buffer"),
                    size: buffer_size,
                    usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::MAP_READ,
                    mapped_at_creation: false,
                })
            })
            .collect::<Vec<wgpu::Buffer>>();

        for (texture, text) in textures.iter().zip(texts) {
            self.text_renderer.prepare(
                &wgpu.device,
                &wgpu.queue,
                &mut self.font_system,
                &mut self.atlas,
                renderer.size.into(),
                [text.into_text_area()],
                &mut self.cache,
            ).unwrap();

            let view = texture.create_view(&Default::default());

            let mut render_pass = text_encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("Text texture render"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color {
                            r: 0.0,
                            g: 0.0,
                            b: 0.0,
                            a: 0.0,
                        }),
                        store: true
                    },
                })],
                depth_stencil_attachment: None}
            );

            self.text_renderer.render(&self.atlas, &mut render_pass).unwrap();
        }

        textures
            .iter()
            .zip(buffers.iter())
            .for_each(|(texture, buffer)| {
                let bytes_per_row = (4*texture.width()) + (256 - (4*texture.width() % 256));
                text_encoder.copy_texture_to_buffer(
                    wgpu::ImageCopyTextureBase {
                        texture,
                        mip_level: 0,
                        origin: wgpu::Origin3d::ZERO,
                        aspect: wgpu::TextureAspect::All
                    },
                    wgpu::ImageCopyBufferBase {
                        buffer,
                        layout: wgpu::ImageDataLayout {
                            offset: 0,
                            bytes_per_row: Some(bytes_per_row),
                            rows_per_image: Some(texture.height()),
                        }
                    },
                    texture.size(),
                );
            });

        wgpu.queue.submit(std::iter::once(text_encoder.finish()));

        for ((idx, buffer), texture) in buffers.iter().enumerate().zip(textures.iter()) {
            {
                let buffer_slice = buffer.slice(..);
    
                // NOTE: We have to create the mapping THEN device.poll() before await
                // the future. Otherwise the application will freeze.
                let (tx, rx) = futures_intrusive::channel::shared::oneshot_channel();
                buffer_slice.map_async(wgpu::MapMode::Read, move |result| {
                    tx.send(result).unwrap();
                });
                wgpu.device.poll(wgpu::Maintain::Wait);
                rx.receive().await.unwrap().unwrap();
            
                let data = buffer_slice.get_mapped_range();
            
                use image::{ImageBuffer, Rgba};
                let image_buffer =
                    ImageBuffer::<Rgba<u8>, _>::from_raw(texture.width(), texture.height(), data).unwrap();
                image_buffer.save(format!("{idx}.png")).unwrap();
            }
            buffer.unmap();
        }

        panic!("IT SHOULD PANIC T HATS GOOD!");
    }
}

/// Represents a section of Text. One should be created per widget of text
pub struct Text {
    pos: Vec2<f32>,
    size: Vec2<f32>,
    font_size: f32,
    line_height: f32,
    bounds: Vec2<Vec2<i32>>,
    text_buffer: glyphon::Buffer,
}

impl Text {
    pub fn new(position: Vec2<f32>, font_size: f32, line_height: f32, text_handle: &mut TextRenderer, engine: &Engine) -> Self {
        let mut text_buffer = glyphon::Buffer::new(&mut text_handle.font_system, Metrics::new(font_size, line_height));
        let size = engine.get_window_size();
        let scale_factor = engine.get_window_scale_factor();
        let phyisical_width = (size.x as f64 * scale_factor) as f32;
        let phyiscal_hieght = (size.y as f64 * scale_factor) as f32;

        text_buffer.set_size(&mut text_handle.font_system, phyisical_width, phyiscal_hieght);

        let hieght = text_buffer.lines.len() as f32 * text_buffer.metrics().line_height;
        let run_width = text_buffer
            .layout_runs()
            .map(|run| run.line_w)
            .max_by(f32::total_cmp).unwrap_or(0.0);


        Self {
            pos: position,
            size: Vec2{x: run_width, y: hieght},
            font_size,
            line_height,
            bounds: Vec2{x: Vec2{x: 0, y: 0}, y: Vec2{x: i32::MAX, y: i32::MAX}},
            text_buffer
        }
    }

    /// Sets the text for the widget, using the defualt font.
    /// This only needs to be done once, not every frame like Materials.
    pub fn set_text(&mut self, text: &str, colour: Colour, text_handle: &mut TextRenderer) {
        self.text_buffer.set_text(
            &mut text_handle.font_system,
            text,
            Attrs::new().color(colour.into()).family(Family::Name(&text_handle.defualt_font_name)),
            Shaping::Basic
        );

        self.update_measurements();
    }

    /// Sets the text for the widget, but with a font of your choosing.
    /// This only needs to be done once, not every frame like Materials
    pub fn set_text_with_font(&mut self, text: &str, colour: Colour, font: &Font, text_handle: &mut TextRenderer) {
        self.text_buffer.set_text(
            &mut text_handle.font_system,
            text,
            Attrs::new().color(colour.into()).family(Family::Name(&font.name)),
            Shaping::Basic
        );

        self.update_measurements();
    }

    /// Sets bounds for the text. Any text drawn outside of the bounds will be cropped
    pub fn set_bounds(&mut self, position: Vec2<i32>, size: Vec2<i32>) {
        self.bounds = Vec2{x: position, y: size};
    }

    /// Sets the font size of the text
    pub fn set_font_size(&mut self, new_size: f32, text_handle: &mut TextRenderer) {
        self.font_size = new_size;
        let metrics = Metrics::new(self.font_size, self.line_height);
        self.text_buffer.set_metrics(&mut text_handle.font_system, metrics);

        self.update_measurements();
    }

    /// Sets the line hieght of the tex
    pub fn set_line_height(&mut self, new_height: f32, text_handle: &mut TextRenderer) {
        self.line_height = new_height;
        let metrics = Metrics::new(self.font_size, self.line_height);
        self.text_buffer.set_metrics(&mut text_handle.font_system, metrics);

        self.update_measurements();
    }

    /// Measuers the text contained within the widget
    pub fn get_measurements(&self) -> Vec2<f32> {
        self.size
    }

    fn update_measurements(&mut self) {
        let hieght = self.text_buffer.lines.len() as f32 * self.text_buffer.metrics().line_height;
        let run_width = self.text_buffer
            .layout_runs()
            .map(|run| run.line_w)
            .max_by(f32::total_cmp)
            .unwrap_or(0.0);

        self.size = Vec2{x: run_width, y: hieght}
    }

    fn into_text_area(&self) -> TextArea {
        TextArea {
            buffer: &self.text_buffer,
            left: self.pos.x,
            top: self.pos.y,
            scale: 1.0,
            bounds: TextBounds {
                left: self.bounds.x.x,
                top: self.bounds.x.y,
                right: self.bounds.y.x,
                bottom: self.bounds.y.y
            },
            default_color: glyphon::Color::rgb(255, 255, 255),
        }
    }
}

/// A struct to ensure Font Families are created properly
/// can be created by [load_font_file](struct.TextRenderer.html#method.load_font_file)
/// or [load_font_data](struct.TextRenderer.html#method.load_font_file)
pub struct Font {
    name: String
}