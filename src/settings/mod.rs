mod reader;
mod writer;

use std::io;
use std::io::{BufReader, BufWriter, Read, Write};
use std::fs::{File, OpenOptions};

#[derive(Debug)]
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
    fn load_from_file() -> Self {
        let settings = File::open("settings.bps").unwrap();
        let mut reader = BufReader::new(settings);

        let mut height: [u8; 2] = [0; 2];
        reader.read_exact(&mut height[..]).unwrap();
        let height = u16::from_le_bytes(height);

        let mut length: [u8; 2] = [0; 2];
        reader.read_exact(&mut length[..]).unwrap();
        let length = u16::from_le_bytes(length);

        let mut volume: [u8; 1] = [0; 1];
        reader.read_exact(&mut volume[..]).unwrap();
        let volume = u8::from_le_bytes(volume);

        Self {
            height,
            length,
            volume,
        }
    }

    pub fn write_to_file(&self) -> io::Result<()>{
        let mut settings = OpenOptions::new()
            .write(true)
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