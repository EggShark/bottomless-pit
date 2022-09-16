use std::io;
use std::io::{BufReader, Read, Write};
use std::fs::{File, OpenOptions};
use std::convert::{From, Into};

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
            _ => panic!("Invalid num expected 0-2 found {}", num)
        }
    }
}

#[derive(Debug, PartialEq)]
pub struct Settings {
    pub resolution: Resolutions,
    volume: u8,
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            resolution: Resolutions::defualt(),
            volume: 10,
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

        Ok(Self {
            resolution,
            volume,
        })
    }

    pub fn update_settings(&mut self, resolution: Resolutions, volume: u8) {
        self.resolution = resolution;
        self.volume = volume;

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

        Ok(())
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