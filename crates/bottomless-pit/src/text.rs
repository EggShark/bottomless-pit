use crate::colour::Colour;
use crate::vectors::Vec2;
use wgpu_glyph::GlyphCruncher;
#[derive(Debug)]
pub(crate) struct Text {
    pub(crate) text: String,
    pub(crate) scale: f32,
    pub(crate) position: Vec2<f32>,
    pub(crate) colour: Colour,
}

#[derive(Debug)]
pub(crate) struct TransformedText {
    pub(crate) text: String,
    pub(crate) scale: f32,
    pub(crate) position: Vec2<f32>,
    pub(crate) colour: Colour,
    pub(crate) bounds: (f32, f32),
    pub(crate) transformation: [f32; 16],
}

pub(crate) fn measure_text(
    text: &str,
    brush: &mut wgpu_glyph::GlyphBrush<()>,
    scale: f32,
) -> Vec2<f32> {
    let section = wgpu_glyph::Section {
        text: vec![wgpu_glyph::Text::new(text).with_scale(scale)],
        screen_position: (1.0, 1.0),
        bounds: (f32::MAX, f32::MAX),
        ..Default::default()
    };

    let rect = brush
        .glyph_bounds(section)
        .unwrap_or(wgpu_glyph::ab_glyph::Rect {
            max: wgpu_glyph::ab_glyph::point(0.0, 0.0),
            min: wgpu_glyph::ab_glyph::point(0.0, 0.0),
        });

    let width = rect.width();
    let height = rect.height();
    Vec2 {
        x: width,
        y: height,
    }
}
