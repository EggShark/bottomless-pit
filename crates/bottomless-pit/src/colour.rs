#[derive(Copy, Clone, Debug)]
pub enum Colour {
    White,
    Black,
    Red,
    Green,
    Blue,
    Yellow,
    Orange,
    Pink,
    Purple,
    Brown,
    Rgba([f32; 4]),
    Rgb([f32; 3]),
}

impl Colour {
    pub fn to_raw(&self) -> [f32; 4] {
        match self {
            Self::White => [1.0, 1.0, 1.0, 1.0],
            Self::Black => [0.0, 0.0, 0.0, 1.0],
            Self::Red => [1.0, 0.0, 0.0, 1.0],
            Self::Green => [0.0, 1.0, 0.0, 1.0],
            Self::Blue => [0.0, 0.0, 1.0, 1.0],
            Self::Yellow => [1.0, 1.0, 0.0, 1.0],
            Self::Orange => [1.0, 0.647058824, 0.0, 1.0],
            Self::Pink => [1.0, 0.752941176, 0.796078431, 1.0],
            Self::Purple => [0.501960784, 0.0, 0.501960784, 1.0],
            Self::Brown => [0.635294118, 0.164705882, 0.164705882, 1.0],
            Self::Rgba(colour) => *colour,
            Self::Rgb(rgb) => [rgb[0], rgb[1], rgb[2], 1.0],
        }
    }

    pub fn from_hex(hex_str: &str) -> Result<Colour, std::num::ParseIntError> {
        let colour_values = i32::from_str_radix(hex_str, 16)?;
        let b = colour_values % 0x100;
        let g = (colour_values - b) / 0x100 % 0x100;
        let r = (colour_values - g) / 0x10000;
        Ok(Self::Rgba([r as f32 / 255.0, g as f32 / 255.0, b as f32/ 255.0, 1.0]))
    }
}

impl Into<wgpu::Color> for Colour {
    fn into(self) -> wgpu::Color {
        let raw = self.to_raw();
        wgpu::Color {
            r: raw[0] as f64, 
            g: raw[1] as f64, 
            b: raw[2] as f64, 
            a: raw[3] as f64,
        }
    }
}

pub fn linear_interpolation(start: Colour, end: Colour, fraction: f32) -> Colour {
    let start = start.to_raw();
    let end = end.to_raw();
    let end_colour = [(end[0] - start[0]) * fraction + start[0], (end[1] - start[1]) * fraction + start[1], (end[2] - start[2]) * fraction + start[2], (end[3] - start[3]) * fraction + start[3]];
    Colour::Rgba(end_colour)
}