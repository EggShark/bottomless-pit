use glyphon::{FontSystem, SwashCache, TextAtlas, TextArea, TextBounds, Metrics, Attrs, Shaping};

use crate::colour::Colour;
use crate::engine_handle::Engine;
use crate::render::RenderInformation;
use crate::vectors::Vec2;

pub struct TextRenderer {
    font_system: FontSystem,
    cache: SwashCache,
    atlas: TextAtlas,
    text_renderer: glyphon::TextRenderer,
}

impl TextRenderer {
    pub fn new(engine: &Engine) -> Self {
        let wgpu = engine.get_wgpu();
        let font_system = FontSystem::new();
        let cache = SwashCache::new();
        let mut atlas = TextAtlas::new(&wgpu.device, &wgpu.queue, wgpu::TextureFormat::Bgra8UnormSrgb);
        let text_renderer = glyphon::TextRenderer::new(&mut atlas, &wgpu.device, wgpu::MultisampleState::default(), None);
        
        Self {
            font_system,
            cache,
            atlas,
            text_renderer,
        }
    }

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

    pub fn draw_text<'pass, 'others>(&'others mut self, text: &'others Text, renderer: &mut RenderInformation<'pass, 'others>) where 'others: 'pass {
        let device = &renderer.wgpu.device;
        let queue = &renderer.wgpu.queue;
        self.text_renderer.prepare(
            device,
            queue,
            &mut self.font_system,
            &mut self.atlas,
            renderer.size.into(),
            [TextArea {
                buffer: &text.text_buffer,
                left: text.pos.x,
                top: text.pos.y,
                scale: 1.0,
                bounds: TextBounds {
                    left: text.bounds.x.x,
                    top: text.bounds.x.y,
                    right: text.bounds.y.x,
                    bottom: text.bounds.y.y,
                },
                default_color: glyphon::Color::rgb(255, 255, 255),
            }],
            &mut self.cache,
        ).unwrap();

        self.text_renderer.render(&self.atlas, &mut renderer.render_pass).unwrap();
    }
}

pub struct Text {
    pos: Vec2<f32>,
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

        Self {
            pos: position,
            font_size,
            line_height,
            bounds: Vec2{x: Vec2{x: 0, y: 0}, y: Vec2{x: i32::MAX, y: i32::MAX}},
            text_buffer
        }
    }

    pub fn set_text(&mut self, text: &str, colour: Colour, text_handle: &mut TextRenderer) {
        self.text_buffer.set_text(&mut text_handle.font_system, text, Attrs::new().color(colour.into()), Shaping::Basic);
    }

    pub fn set_bounds(&mut self, position: Vec2<i32>, size: Vec2<i32>) {
        self.bounds = Vec2{x: position, y: position + size};
    }

    pub fn set_font_size(&mut self, new_size: f32, text_handle: &mut TextRenderer) {
        self.font_size = new_size;
        let metrics = Metrics::new(self.font_size, self.line_height);
        self.text_buffer.set_metrics(&mut text_handle.font_system, metrics)
    }

    pub fn set_line_height(&mut self, new_height: f32, text_handle: &mut TextRenderer) {
        self.line_height = new_height;
        let metrics = Metrics::new(self.font_size, self.line_height);
        self.text_buffer.set_metrics(&mut text_handle.font_system, metrics);
    }

    pub fn get_measurements(&self) -> Vec2<f32> {
        let hieght = self.text_buffer.lines.len() as f32 * self.text_buffer.metrics().line_height;
        let run_width = self.text_buffer.layout_runs().map(|run| run.line_w).max_by(f32::total_cmp).unwrap_or(0.0);

        Vec2{x: run_width, y: hieght}
    }
}