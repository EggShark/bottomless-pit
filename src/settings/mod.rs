use std::io;
use std::io::{BufReader, Read, Write};
use std::fs::{File, OpenOptions};

#[derive(Debug, PartialEq)]
pub struct Settings {
    pub height: u16,
    pub length: u16,
    pub volume: u8,
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            height: 450,
            length: 800,
            volume: 100,
        }
    }
}

impl Settings {
    pub fn load_from_file() -> Result<Self, std::io::Error> {
        let settings = File::open("settings.bps")?;
        let mut reader = BufReader::new(settings);

        let mut height: [u8; 2] = [0; 2];
        reader.read_exact(&mut height[..])?;
        let height = u16::from_le_bytes(height);

        let mut length: [u8; 2] = [0; 2];
        reader.read_exact(&mut length[..])?;
        let length = u16::from_le_bytes(length);

        let mut volume: [u8; 1] = [0; 1];
        reader.read_exact(&mut volume[..])?;
        let volume = u8::from_le_bytes(volume);

        Ok(Self {
            height,
            length,
            volume,
        })
    }

    pub fn update_settings(&mut self, width: u16, height: u16, volume: u8) {
        self.length = width;
        self.height = height;
        self.volume = volume;

        self.write_to_file().unwrap();
    }

    fn write_to_file(&self) -> io::Result<()>{
        let mut settings = OpenOptions::new()
            .write(true)
            .create(true)
            .open("settings.bps")
            .unwrap();
        let height = self.height.to_le_bytes();
        let length = self.length.to_le_bytes();
        let volume = self.volume.to_le_bytes();

        settings.write(&height)?;
        settings.write(&length)?;
        settings.write(&volume)?;

        Ok(())
    }
}