mod builder;
mod pipeline;

use core::hash::BuildHasher;
use glyph_brush::ab_glyph;
use glyph_brush::{DefaultSectionHasher, Extra};
use ab_glyph::Font;
use pipeline::{Instance, Pipeline};

pub struct GlyphBrush<F = ab_glyph::FontArc, H = DefaultSectionHasher> {
    pipeline: Pipeline, 
    glyph_brush: glyph_brush::GlyphBrush<Instance, Extra, F, H>,
}

impl<F: Font + Sync, H: BuildHasher> GlyphBrush<F, H> {
    fn new(device: &wgpu::Device, filter_mode: wgpu::FilterMode, multisample: wgpu::MultisampleState, render_format: wgpu::TextureFormat, raw_buider: glyph_brush::GlyphBrushBuilder<F, H>) -> Self {
        let glyph_brush = raw_buider.build();
        let (cache_width, cache_height) = glyph_brush.texture_dimensions();
        Self {
            pipeline: Pipeline::new(device, filter_mode, multisample, render_format, cache_width, cache_height),
            glyph_brush,
        }
    }
}