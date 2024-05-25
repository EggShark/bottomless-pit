//! The colour struct module

use encase::ShaderType;
/// A struct containing RGBA Colours (spelled properly) with some predefind colour consts
#[derive(Copy, Clone, Debug, PartialEq, ShaderType)]
pub struct Colour {
    r: f32,
    g: f32,
    b: f32,
    a: f32,
}

impl Colour {
    pub const WHITE: Self = Self {
        r: 1.0,
        g: 1.0,
        b: 1.0,
        a: 1.0,
    };

    pub const BLACK: Self = Self {
        r: 0.0,
        g: 0.0,
        b: 0.0,
        a: 1.0,
    };

    pub const RED: Self = Self {
        r: 1.0,
        g: 0.0,
        b: 0.0,
        a: 1.0,
    };

    pub const GREEN: Self = Self {
        r: 0.0,
        g: 1.0,
        b: 0.0,
        a: 1.0,
    };

    pub const BLUE: Self = Self {
        r: 0.0,
        g: 0.0,
        b: 1.0,
        a: 1.0,
    };

    pub const YELLOW: Self = Self {
        r: 1.0,
        g: 1.0,
        b: 0.0,
        a: 1.0,
    };

    pub const ORANGE: Self = Self {
        r: 1.0,
        g: 0.64705884,
        b: 0.0,
        a: 1.0,
    };

    pub const PINK: Self = Self {
        r: 1.0,
        g: 0.7529412,
        b: 0.79607844,
        a: 1.0,
    };

    pub const BROWN: Self = Self {
        r: 0.63529414,
        g: 0.16470589,
        b: 0.16470589,
        a: 1.0,
    };

    /// Takes a hex string like `#805E4E` and turns into a colour. Can fail if an
    /// invaild string is provided
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

    /// Creates a colour from seperate values beteen 0.0 and 255.0
    /// and an alpha value of 0 to 1.
    pub fn from_rgba(r: f32, g: f32, b: f32, a: f32) -> Self {
        Self {
            r: r / 255.0,
            g: g / 255.0,
            b: b / 255.0,
            a: a,
        }
    }

    /// Interpolates between two colours by the specified fraction which should be between 1.0
    /// and 0.0
    pub fn linear_interpolation(start: Colour, end: Colour, fraction: f32) -> Self {
        Self {
            r: (end.r - start.r) * fraction + start.r,
            g: (end.g - start.g) * fraction + start.g,
            b: (end.b - start.b) * fraction + start.b,
            a: (end.a - start.a) * fraction + start.a,
        }
    }

    pub(crate) fn as_raw(&self) -> [f32; 4] {
        [self.r, self.g, self.b, self.a]
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

impl From<Colour> for glyphon::Color {
    fn from(value: Colour) -> Self {
        // should work as it is always a number betweeon 1-0
        let r = (value.r * 255.0) as u8;
        let g = (value.g * 255.0) as u8;
        let b = (value.b * 255.0) as u8;
        let a = (value.a * 255.0) as u8;
        Self::rgba(r, g, b, a)
    }
}

impl From<Colour> for [f32; 4] {
    fn from(value: Colour) -> Self {
        [value.r, value.g, value.b, value.a]
    }
}
