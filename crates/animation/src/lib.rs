use raylib::core::texture::Texture2D;
use raylib::core::{RaylibHandle, RaylibThread};
use raylib::core::drawing::RaylibDrawHandle;
use raylib::core::drawing::RaylibDraw;
use raylib::core::math::{Vector2, Rectangle};
use raylib::core::color::Color;
use raylib::texture::RaylibTexture2D;
use utils::Point;

pub struct PlayerAnimation {
    sprite: Texture2D,
    frames: u8,
    curr_frame: u8,
    framecounter: u8,
    dirrection: i8, //true is facing right 
}

pub enum PlayerAnimations {
    Walking,
    Running,
    Idle,
}

impl PlayerAnimations {
    pub fn into_uszie(&self) -> usize {
        match self {
            Self::Idle => 0,
            Self::Walking => 1,
            Self::Running => 2,
        }
    }
}

impl PlayerAnimation {
    pub fn new(path: &str, frames: u8, rl: &mut RaylibHandle, thread: &RaylibThread) -> Self {
        let sprite = rl.load_texture(thread, path).unwrap();
        Self {
            sprite,
            frames,
            curr_frame: 0,
            framecounter: 0,
            dirrection: 0,
        }
    }

    fn advance(&mut self) {
        if self.curr_frame + 1 > self.frames {
            self.curr_frame = 0;
        }
        else {
            self.curr_frame += 1;
        }
    }

    pub fn draw(&self, d_handle: &mut RaylibDrawHandle, pos: Point) {
        let frame_rec = Rectangle::new((self.curr_frame as i32 * (self.sprite.width()/self.frames as i32))as f32, 0.0, (self.sprite.width() as f32 /self.frames as f32) * self.dirrection as f32, self.sprite.height() as f32);
        d_handle.draw_texture_rec(&self.sprite, frame_rec, <Point as Into<Vector2>>::into(pos), Color::WHITE);
    }

    pub fn update(&mut self, dir: i8) {
        self.framecounter += 1;
        self.dirrection = dir;
        if self.framecounter >= 10 {
            self.framecounter = 0;
            self.advance();
        }
    }
}