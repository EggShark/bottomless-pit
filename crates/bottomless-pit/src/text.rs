use crate::{Colour, IDENTITY_MATRIX, matrix_math, Vec2};
use cgmath::{Transform};
use wgpu_glyph::GlyphCruncher;
use crate::matrix_math::*;

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
    pub(crate) transformation: [f32; 16]
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

pub(crate) fn get_text_rotation_matrix(text: &str, position: (f32, f32), scale: f32, degree: f32, brush: &mut wgpu_glyph::GlyphBrush<()>) -> cgmath::Matrix4<f32> {
    let section = wgpu_glyph::Section {
        bounds: (f32::MAX, f32::MAX),
        screen_position: position,
        text: vec![wgpu_glyph::Text::new(text).with_scale(scale)],
        ..Default::default()
    };
    let measurement = brush.glyph_bounds(section).unwrap();
    let mid = get_mid_point(measurement);
    let rotation_matrix = unflatten_matrix(calculate_rotation_matrix(degree));
    let translation_matrix = cgmath::Matrix4::from_translation(cgmath::vec3(mid.x, mid.y, 0.0));
    let inverse_translation = translation_matrix.inverse_transform().unwrap_or(unflatten_matrix(IDENTITY_MATRIX));
    // Creates a matrix like
    // 1 0 0 0
    // 0 1 0 0
    // 0 0 1 0
    // x y z 1
    let out = translation_matrix * rotation_matrix * inverse_translation;
    out
}

pub(crate) fn make_flat_text_rotation_matrix(text: &str, position: (f32, f32), scale: f32, degree: f32, brush: &mut wgpu_glyph::GlyphBrush<()>) -> [f32; 16] {
    matrix_math::flatten_matrix(get_text_rotation_matrix(text, position, scale, degree, brush))
}