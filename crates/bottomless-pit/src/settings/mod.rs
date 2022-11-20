use std::io;
use std::io::{BufReader, Read, Write};
use std::fs::{File, OpenOptions};
use std::convert::{From, Into};
use raylib::consts::KeyboardKey;
use input_handler::Inputs;

#[derive(Copy, Clone, Debug, PartialEq)]
pub enum Resolutions {
    X1920x1080,
    X1280x720,
    X854x360,
}

impl Resolutions {
    pub fn len_width(&self) -> (u16, u16) {
        match self {
            Self::X1920x1080 => {
                (1920, 1080)
            }
            Self::X1280x720 => {
                (1280, 720)
            }
            Self::X854x360 => {
                (854, 360)
            }
        }
    }

    pub fn defualt() -> Self {
        Self::X1280x720
    }
}

impl Into<u8> for Resolutions {
    fn into(self) -> u8{
        match self {
            Self::X1920x1080 => 0,
            Self::X1280x720 => 1,
            Self::X854x360 => 2,
        }
    }
}

impl From<u8> for Resolutions {
    fn from(num: u8) -> Self{
        match num {
            0 => Self::X1920x1080,
            1 => Self::X1280x720,
            2 => Self::X854x360,
            _ => Self::defualt(),
        }
    }
}

#[derive(Debug, PartialEq)]
pub struct Settings {
    pub resolution: Resolutions,
    pub keys: Inputs,
    volume: u8,
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            resolution: Resolutions::defualt(),
            volume: 10,
            keys: Inputs::new([KeyboardKey::KEY_W, KeyboardKey::KEY_A, KeyboardKey::KEY_S, KeyboardKey::KEY_D, KeyboardKey::KEY_I, KeyboardKey::KEY_K, KeyboardKey::KEY_J, KeyboardKey::KEY_LEFT_SHIFT]),
            // as of 9/18/22 0 = up 1 = left 2 = down 3 = right 4 = slash 5 = heavy slash 6 = kick
            // 7 = sprint
        }
    }
}

impl Settings {
    pub fn get_resoultion(&self) -> (u16, u16) {
        self.resolution.len_width()
    }

    pub fn load_from_file() -> Result<Self, std::io::Error> {
        let settings = File::open("settings.bps")?;
        let mut reader = BufReader::new(settings);

        let mut resolution: [u8; 1] = [0; 1];
        reader.read_exact(&mut resolution[..])?;
        let resolution = Resolutions::from(u8::from_le_bytes(resolution));

        let mut volume: [u8; 1] = [0; 1];
        reader.read_exact(&mut volume[..])?;
        let volume = u8::from_le_bytes(volume);

        let keys = Self::load_keys(&mut reader)?;

        Ok(Self {
            resolution,
            volume,
            keys,
        })
    }

    fn load_keys(reader: &mut BufReader<File>) -> Result<Inputs, std::io::Error> {
        let mut buffer = [KeyboardKey::KEY_NULL; 8];

        for i in 0..buffer.len() {
            let mut key_byte: [u8; 1] = [0; 1];
            reader.read_exact(&mut key_byte)?;
            let read_key = u8_to_rl_key(key_byte[0]);
            buffer[i] = read_key;
        }

        Ok(Inputs::new(buffer))
    }

    pub fn update_settings(&mut self, resolution: Resolutions, volume: u8) {
        self.resolution = resolution;
        self.volume = volume;

        self.write_to_file().unwrap();
    }

    pub fn update_bindings(&mut self, keys: Inputs) {
        self.keys = keys;

        self.write_to_file().unwrap();
    }

    fn write_to_file(&self) -> io::Result<()>{
        let mut settings = OpenOptions::new()
            .write(true)
            .create(true)
            .open("settings.bps")
            .unwrap();
        let resolution: u8 = self.resolution.into();
        let resolution = resolution.to_le_bytes();
        let volume = self.volume.to_le_bytes();

        settings.write(&resolution)?;
        settings.write(&volume)?;
        self.write_keys_to_file(&mut settings)?;

        Ok(())
    }

    fn write_keys_to_file(&self, settings: &mut File) -> io::Result<()>{
        let mut buf: [u8; 8] = [0; 8];
        let keys = self.keys.get_raw();
        for i in 0..keys.len() {
            buf[i] = rl_key_to_u8(&keys[i]);
        }

        settings.write(&buf)?;

        Ok(())
    }
}

// they have this represented as a u32??? could be a u8 doing this for settings size
fn rl_key_to_u8(key: &KeyboardKey) -> u8 {
    match key {
        KeyboardKey::KEY_NULL => 0,
        KeyboardKey::KEY_APOSTROPHE => 1,
        KeyboardKey::KEY_COMMA => 2,
        KeyboardKey::KEY_MINUS => 3,
        KeyboardKey::KEY_PERIOD => 4,
        KeyboardKey::KEY_SLASH => 5,
        KeyboardKey::KEY_ZERO => 6,
        KeyboardKey::KEY_ONE => 7,
        KeyboardKey::KEY_TWO => 8,
        KeyboardKey::KEY_THREE => 9,
        KeyboardKey::KEY_FOUR => 10,
        KeyboardKey::KEY_FIVE => 11,
        KeyboardKey::KEY_SIX => 12,
        KeyboardKey::KEY_SEVEN => 13,
        KeyboardKey::KEY_EIGHT => 14,
        KeyboardKey::KEY_NINE => 15,
        KeyboardKey::KEY_SEMICOLON => 16,
        KeyboardKey::KEY_EQUAL => 17,
        KeyboardKey::KEY_A => 18,
        KeyboardKey::KEY_B => 19,
        KeyboardKey::KEY_C => 20,
        KeyboardKey::KEY_D => 21,
        KeyboardKey::KEY_E => 22,
        KeyboardKey::KEY_F => 23,
        KeyboardKey::KEY_G => 24,
        KeyboardKey::KEY_H => 25,
        KeyboardKey::KEY_I => 26,
        KeyboardKey::KEY_J => 27,
        KeyboardKey::KEY_K => 28,
        KeyboardKey::KEY_L => 29,
        KeyboardKey::KEY_M => 30,
        KeyboardKey::KEY_N => 31,
        KeyboardKey::KEY_O => 32,
        KeyboardKey::KEY_P => 33,
        KeyboardKey::KEY_Q => 34,
        KeyboardKey::KEY_R => 35,
        KeyboardKey::KEY_S => 36,
        KeyboardKey::KEY_T => 37,
        KeyboardKey::KEY_U => 38,
        KeyboardKey::KEY_V => 39,
        KeyboardKey::KEY_W => 40,
        KeyboardKey::KEY_X => 41,
        KeyboardKey::KEY_Y => 42,
        KeyboardKey::KEY_Z => 43,
        KeyboardKey::KEY_SPACE => 44,
        KeyboardKey::KEY_ESCAPE => 45,
        KeyboardKey::KEY_ENTER => 46,
        KeyboardKey::KEY_TAB => 47,
        KeyboardKey::KEY_BACKSPACE => 48,
        KeyboardKey::KEY_INSERT => 49,
        KeyboardKey::KEY_DELETE => 50,
        KeyboardKey::KEY_RIGHT => 51,
        KeyboardKey::KEY_LEFT => 52,
        KeyboardKey::KEY_DOWN => 53,
        KeyboardKey::KEY_UP => 54,
        KeyboardKey::KEY_PAGE_UP => 55,
        KeyboardKey::KEY_PAGE_DOWN => 56,
        KeyboardKey::KEY_HOME => 57,
        KeyboardKey::KEY_END => 58,
        KeyboardKey::KEY_CAPS_LOCK => 59,
        KeyboardKey::KEY_SCROLL_LOCK => 60,
        KeyboardKey::KEY_NUM_LOCK => 61,
        KeyboardKey::KEY_PRINT_SCREEN => 62,
        KeyboardKey::KEY_PAUSE => 63,
        KeyboardKey::KEY_F1 => 64,
        KeyboardKey::KEY_F2 => 65,
        KeyboardKey::KEY_F3 => 66,
        KeyboardKey::KEY_F4 => 67,
        KeyboardKey::KEY_F5 => 68,
        KeyboardKey::KEY_F6 => 69,
        KeyboardKey::KEY_F7 => 70,
        KeyboardKey::KEY_F8 => 71,
        KeyboardKey::KEY_F9 => 72,
        KeyboardKey::KEY_F10 => 73,
        KeyboardKey::KEY_F11 => 74,
        KeyboardKey::KEY_F12 => 75,
        KeyboardKey::KEY_LEFT_SHIFT => 76,
        KeyboardKey::KEY_LEFT_CONTROL => 77,
        KeyboardKey::KEY_LEFT_ALT => 78,
        KeyboardKey::KEY_LEFT_SUPER => 79,
        KeyboardKey::KEY_RIGHT_SHIFT => 80,
        KeyboardKey::KEY_RIGHT_CONTROL => 81,
        KeyboardKey::KEY_RIGHT_ALT => 82,
        KeyboardKey::KEY_RIGHT_SUPER => 83,
        KeyboardKey::KEY_KB_MENU => 84,
        KeyboardKey::KEY_LEFT_BRACKET => 85,
        KeyboardKey::KEY_BACKSLASH => 86,
        KeyboardKey::KEY_RIGHT_BRACKET => 87,
        KeyboardKey::KEY_GRAVE => 88,
        KeyboardKey::KEY_KP_0 => 89,
        KeyboardKey::KEY_KP_1 => 90,
        KeyboardKey::KEY_KP_2 => 91,
        KeyboardKey::KEY_KP_3 => 92,
        KeyboardKey::KEY_KP_4 => 93,
        KeyboardKey::KEY_KP_5 => 94,
        KeyboardKey::KEY_KP_6 => 95,
        KeyboardKey::KEY_KP_7 => 96,
        KeyboardKey::KEY_KP_8 => 97,
        KeyboardKey::KEY_KP_9 => 98,
        KeyboardKey::KEY_KP_DECIMAL => 99,
        KeyboardKey::KEY_KP_DIVIDE => 100,
        KeyboardKey::KEY_KP_MULTIPLY => 101,
        KeyboardKey::KEY_KP_SUBTRACT => 102,
        KeyboardKey::KEY_KP_ADD => 103,
        KeyboardKey::KEY_KP_ENTER => 104,
        KeyboardKey::KEY_KP_EQUAL => 105,
        KeyboardKey::KEY_BACK => 106,
        KeyboardKey::KEY_VOLUME_UP => 107,
        KeyboardKey::KEY_VOLUME_DOWN => 108,
    }
}

fn u8_to_rl_key(num: u8) -> KeyboardKey {
    match num {
        0 => KeyboardKey::KEY_NULL,
        1 => KeyboardKey::KEY_APOSTROPHE,
        2 => KeyboardKey::KEY_COMMA,
        3 => KeyboardKey::KEY_MINUS,
        4 => KeyboardKey::KEY_PERIOD,
        5 => KeyboardKey::KEY_SLASH,
        6 => KeyboardKey::KEY_ZERO,
        7 => KeyboardKey::KEY_ONE,
        8 => KeyboardKey::KEY_TWO,
        9 => KeyboardKey::KEY_THREE,
        10 => KeyboardKey::KEY_FOUR,
        11 => KeyboardKey::KEY_FIVE,
        12 => KeyboardKey::KEY_SIX,
        13 => KeyboardKey::KEY_SEVEN,
        14 => KeyboardKey::KEY_EIGHT,
        15 => KeyboardKey::KEY_NINE,
        16 => KeyboardKey::KEY_SEMICOLON,
        17 => KeyboardKey::KEY_EQUAL,
        18 => KeyboardKey::KEY_A,
        19 => KeyboardKey::KEY_B,
        20 => KeyboardKey::KEY_C,
        21 => KeyboardKey::KEY_D,
        22 => KeyboardKey::KEY_E,
        23 => KeyboardKey::KEY_F,
        24 => KeyboardKey::KEY_G,
        25 => KeyboardKey::KEY_H,
        26 => KeyboardKey::KEY_I,
        27 => KeyboardKey::KEY_J,
        28 => KeyboardKey::KEY_K,
        29 => KeyboardKey::KEY_L,
        30 => KeyboardKey::KEY_M,
        31 => KeyboardKey::KEY_N,
        32 => KeyboardKey::KEY_O,
        33 => KeyboardKey::KEY_P,
        34 => KeyboardKey::KEY_Q,
        35 => KeyboardKey::KEY_R,
        36 => KeyboardKey::KEY_S,
        37 => KeyboardKey::KEY_T,
        38 => KeyboardKey::KEY_U,
        39 => KeyboardKey::KEY_V,
        40 => KeyboardKey::KEY_W,
        41 => KeyboardKey::KEY_X,
        42 => KeyboardKey::KEY_Y,
        43 => KeyboardKey::KEY_Z,
        44 => KeyboardKey::KEY_SPACE,
        45 => KeyboardKey::KEY_ESCAPE,
        46 => KeyboardKey::KEY_ENTER,
        47 => KeyboardKey::KEY_TAB,
        48 => KeyboardKey::KEY_BACKSPACE,
        49 => KeyboardKey::KEY_INSERT,
        50 => KeyboardKey::KEY_DELETE,
        51 => KeyboardKey::KEY_RIGHT,
        52 => KeyboardKey::KEY_LEFT,
        53 => KeyboardKey::KEY_DOWN,
        54 => KeyboardKey::KEY_UP,
        55 => KeyboardKey::KEY_PAGE_UP,
        56 => KeyboardKey::KEY_PAGE_DOWN,
        57 => KeyboardKey::KEY_HOME,
        58 => KeyboardKey::KEY_END,
        59 => KeyboardKey::KEY_CAPS_LOCK,
        60 => KeyboardKey::KEY_SCROLL_LOCK,
        61 => KeyboardKey::KEY_NUM_LOCK,
        62 => KeyboardKey::KEY_PRINT_SCREEN,
        63 => KeyboardKey::KEY_PAUSE,
        64 => KeyboardKey::KEY_F1,
        65 => KeyboardKey::KEY_F2,
        66 => KeyboardKey::KEY_F3,
        67 => KeyboardKey::KEY_F4,
        68 => KeyboardKey::KEY_F5,
        69 => KeyboardKey::KEY_F6,
        70 => KeyboardKey::KEY_F7,
        71 => KeyboardKey::KEY_F8,
        72 => KeyboardKey::KEY_F9,
        73 => KeyboardKey::KEY_F10,
        74 => KeyboardKey::KEY_F11,
        75 => KeyboardKey::KEY_F12,
        76 => KeyboardKey::KEY_LEFT_SHIFT,
        77 => KeyboardKey::KEY_LEFT_CONTROL,
        78 => KeyboardKey::KEY_LEFT_ALT,
        79 => KeyboardKey::KEY_LEFT_SUPER,
        80 => KeyboardKey::KEY_RIGHT_SHIFT,
        81 => KeyboardKey::KEY_RIGHT_CONTROL,
        82 => KeyboardKey::KEY_RIGHT_ALT,
        83 => KeyboardKey::KEY_RIGHT_SUPER,
        84 => KeyboardKey::KEY_KB_MENU,
        85 => KeyboardKey::KEY_LEFT_BRACKET,
        86 => KeyboardKey::KEY_BACKSLASH,
        87 => KeyboardKey::KEY_RIGHT_BRACKET,
        88 => KeyboardKey::KEY_GRAVE,
        89 => KeyboardKey::KEY_KP_0,
        90 => KeyboardKey::KEY_KP_1,
        91 => KeyboardKey::KEY_KP_2,
        92 => KeyboardKey::KEY_KP_3,
        93 => KeyboardKey::KEY_KP_4,
        94 => KeyboardKey::KEY_KP_5,
        95 => KeyboardKey::KEY_KP_6,
        96 => KeyboardKey::KEY_KP_7,
        97 => KeyboardKey::KEY_KP_8,
        98 => KeyboardKey::KEY_KP_9,
        99 => KeyboardKey::KEY_KP_DECIMAL,
        100 => KeyboardKey::KEY_KP_DIVIDE,
        101 => KeyboardKey::KEY_KP_MULTIPLY,
        102 => KeyboardKey::KEY_KP_SUBTRACT,
        103 => KeyboardKey::KEY_KP_ADD,
        104 => KeyboardKey::KEY_KP_ENTER,
        105 => KeyboardKey::KEY_KP_EQUAL,
        106 => KeyboardKey::KEY_BACK,
        107 => KeyboardKey::KEY_VOLUME_UP,
        108 => KeyboardKey::KEY_VOLUME_DOWN,
        _ => KeyboardKey::KEY_NULL,
    }
}

#[cfg(test)] 
mod tests {
    use super::*;
    #[test]
    fn resolution_to_u8() {
        let zero = Resolutions::X1920x1080;
        let one = Resolutions::X1280x720;
        let two = Resolutions::X854x360;

        assert_eq!(u8::from(0), zero.into());
        assert_eq!(u8::from(1), one.into());
        assert_eq!(u8::from(2), two.into());
    }

    #[test]
    fn u8_to_resolution() {
        let x1920_1080 = Resolutions::from(0);
        let x1280_720 = Resolutions::from(1);
        let x854x360 = Resolutions::from(2);

        assert_eq!(x1920_1080, Resolutions::X1920x1080);
        assert_eq!(x1280_720, Resolutions::X1280x720);
        assert_eq!(x854x360, Resolutions::X854x360);
    }
}