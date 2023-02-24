pub(crate) struct Text {
    text: String,
    scale: f32,
    position: Point2<f32>,
    colour: Colour,
}

pub(crate) struct TransformedText {
    text: String,
    scale: f32,
    position: Point2<f32>,
    colour: Colour,
    bounds: (f32, f32),
    transformatoin: [f32; 16]
}

pub(crate) fn measure_text(text: &str, brush: &mut wgpu_glyph::GlyphBrush<()>, scale: f32) -> wgpu_glyph::ab_glyph::Rect {
    let section = wgpu_glyph::Section {
        text: vec![wgpu_glyph::Text::new(text).with_scale(scale)],
        screen_position: (1.0, 1.0),
        bounds: (f32::MAX, f32::MAX),
        ..Default::default()
    };
    brush.glyph_bounds(section).unwrap_or(wgpu_glyph::ab_glyph::Rect{
        max: wgpu_glyph::ab_glyph::point(0.0,0.0),
        min: wgpu_glyph::ab_glyph::point(0.0,0.0),
    })
}