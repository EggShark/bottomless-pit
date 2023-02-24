pub enum Colour {
    White,
    Black,
    Red,
    Green,
    Blue,
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
            Self::Rgba(colour) => *colour,
            Self::Rgb(rgb) => [rgb[0], rgb[1], rgb[2], 1.0],
        }
    }
}