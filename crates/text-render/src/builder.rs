use core::hash::BuildHasher;

use glyph_brush::ab_glyph::Font;
use glyph_brush::delegate_glyph_brush_builder_fns;
use glyph_brush::DefaultSectionHasher;
use super::GlyphBrush;

/// Builder for a [`GlyphBrush`](struct.GlyphBrush.html).
pub struct GlyphBrushBuilder<F, H = DefaultSectionHasher> {
    inner: glyph_brush::GlyphBrushBuilder<F, H>,
    texture_filter_method: wgpu::FilterMode,
    multisample_state: wgpu::MultisampleState,
}

impl<F, H> From<glyph_brush::GlyphBrushBuilder<F, H>>
    for GlyphBrushBuilder<F, H>
{
    fn from(inner: glyph_brush::GlyphBrushBuilder<F, H>) -> Self {
        GlyphBrushBuilder {
            inner,
            texture_filter_method: wgpu::FilterMode::Linear,
            multisample_state: wgpu::MultisampleState::default(),
        }
    }
}

impl GlyphBrushBuilder<(), ()> {
    /// Specifies the default font used to render glyphs.
    /// Referenced with `FontId(0)`, which is default.
    #[inline]
    pub fn using_font<F: Font>(font: F) -> GlyphBrushBuilder<F> {
        Self::using_fonts(vec![font])
    }

    pub fn using_fonts<F: Font>(fonts: Vec<F>) -> GlyphBrushBuilder<F> {
        GlyphBrushBuilder {
            inner: glyph_brush::GlyphBrushBuilder::using_fonts(fonts),
            texture_filter_method: wgpu::FilterMode::Linear,
            multisample_state: wgpu::MultisampleState::default(),
        }
    }
}

impl<F: Font + Sync, H: BuildHasher> GlyphBrushBuilder<F, H> {
    pub fn build(self, device: &wgpu::Device, render_format: wgpu::TextureFormat) -> GlyphBrush<F, H> {
        GlyphBrush::new(device, self.texture_filter_method, self.multisample_state, render_format, self.inner)
    }
}