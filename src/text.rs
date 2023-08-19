use glyphon::{Edit, FontSystem, SwashCache, TextAtlas, TextArea, TextBounds};

use crate::colour::Colour;
use crate::render::RenderInformation;
use crate::vectors::Vec2;

pub struct TextRenderer {
    font_system: FontSystem,
    cache: SwashCache,
    atlas: TextAtlas,
    text_renderer: glyphon::TextRenderer,
}

impl TextRenderer {
    pub fn draw_text<'pass, 'others>(&mut self, text: &'others Text, renderer: &'others mut RenderInformation<'pass, 'others>) where 'others: 'pass {
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
                scale: text.scale,
                bounds: TextBounds {
                    left: text.bounds.x.x,
                    top: text.bounds.x.y,
                    right: text.bounds.y.x,
                    bottom: text.bounds.y.y,
                },
                default_color: glyphon::Color::rgb(255, 255, 255),
            }],
            &mut self.cache,
        );
    }
}

pub struct Text {
    pos: Vec2<f32>,
    scale: f32,
    bounds: Vec2<Vec2<i32>>,
    text_buffer: glyphon::Buffer,
}