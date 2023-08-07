//! The colour struct module
/// A struct containing RGBA Colours (spelled properly) with some predefind colour consts
#[derive(Copy, Clone, Debug, PartialEq)]
pub struct Colour {
    r: f32,
    g: f32,
    b: f32,
    a: f32,
}

impl Colour {
    const WHITE: Self = Self {
        r: 1.0,
        g: 1.0,
        b: 1.0,
        a: 1.0,
    };

    const BLACK: Self = Self {
        r: 1.0,
        g: 1.0,
        b: 1.0,
        a: 1.0,
    };

    const RED: Self = Self {
        r: 1.0,
        g: 0.0,
        b: 0.0,
        a: 1.0,
    };

    const GREEN: Self = Self {
        r: 0.0,
        g: 1.0,
        b: 0.0,
        a: 1.0,
    };

    const BLUE: Self = Self {
        r: 0.0,
        g: 0.0,
        b: 1.0,
        a: 1.0,
    };

    const YELLOW: Self = Self {
        r: 1.0,
        g: 1.0,
        b: 0.0,
        a: 1.0,
    };

    const ORANGE: Self = Self {
        r: 1.0,
        g: 0.64705884,
        b: 0.0,
        a: 1.0,
    };

    const PINK: Self = Self {
        r: 1.0,
        g: 0.7529412,
        b: 0.79607844,
        a: 1.0,
    };

    const BROWN: Self = Self {
        r: 0.63529414,
        g: 0.16470589,
        b: 0.16470589,
        a: 1.0
    };

    pub fn from_hex(hex_str: &str) -> Result<Self, std::num::ParseIntError> {
        let colour_values = i32::from_str_radix(hex_str, 16)?;
        let b = colour_values % 0x100;
        let g = (colour_values - b) / 0x100 % 0x100;
        let r = (colour_values - g) / 0x10000;
        Ok(Self {
            r: r as f32 / 255.0,
            g: g as f32 / 255.0,
            b: b as f32 / 255.0,
            a: 1.0,
        })
    }

    pub fn from_rgba(r: f32, g: f32, b: f32, a: f32) -> Self {
        Self {
            r,
            g,
            b,
            a,
        }
    }

    pub fn linear_interpolation(start: Colour, end: Colour, fraction: f32) -> Self {
        Self {
            r: (end.r - start.r) * fraction + start.r,
            g: (end.g - start.g) * fraction + start.g,
            b: (end.b - start.b) * fraction + start.b,
            a: (end.a - start.a) * fraction + start.a,
        }
    }
}

impl From<Colour> for wgpu::Color {
    fn from(value: Colour) -> Self {
        wgpu::Color {
            r: value.r as f64,
            g: value.g as f64, 
            b: value.b as f64,
            a: value.a as f64,
        }
    }
}