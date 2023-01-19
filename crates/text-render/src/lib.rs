mod builder;
mod pipeline;

use core::hash::BuildHasher;
use glyph_brush::ab_glyph;
use glyph_brush::{DefaultSectionHasher, Extra};
use ab_glyph::Font;
use pipeline::Instance;

pub struct GlyphBrush<F = ab_glyph::FontArc, H = DefaultSectionHasher> {
    pipeline: bool, // just junk for now
    glyph_brush: glyph_brush::GlyphBrush<Instance, Extra, F, H>,
}

impl<F: Font + Sync, H: BuildHasher> GlyphBrush<F, H> {
    fn new(device: &wgpu::Device, filter_mode: wgpu::FilterMode, multisample: wgpu::MultisampleState, render_format: wgpu::TextureFormat, raw_buider: glyph_brush::GlyphBrushBuilder<F, H>) -> Self {
        let glyph_brush = raw_buider.build();
        // let (cache_width, cache_height = glyph_brush.texture_dimensions);
        Self {
            pipeline: false,
            glyph_brush,
        }
    }
}