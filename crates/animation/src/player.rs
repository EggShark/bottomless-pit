use raylib::core::texture::Texture2D;
use raylib::core::{RaylibHandle, RaylibThread};
use raylib::core::drawing::RaylibDrawHandle;
use raylib::core::drawing::RaylibDraw;
use raylib::core::math::{Vector2, Rectangle};
use raylib::core::color::Color;
use raylib::texture::RaylibTexture2D;
use utils::Point;

#[derive(Debug)]
pub struct PlayerAnimation {
    sprite: Texture2D,
    frames: i16,
    curr_frame: i16,
    direction: i16, //1 is facing right
}

#[derive(Debug)]
pub enum PlayerAnimations {
    Walking,
    Running,
    Idle,
}

impl PlayerAnimations {
    pub fn into_usize(&self) -> usize {
        match self {
            Self::Idle => 0,
            Self::Walking => 1,
            Self::Running => 2,
        }
    }
}

impl PlayerAnimation {
    pub fn new(path: &str, frames: i16, rl: &mut RaylibHandle, thread: &RaylibThread) -> Self {
        let sprite = rl.load_texture(thread, path).unwrap();
        Self {
            sprite,
            frames,
            curr_frame: 0,
            direction: 1,
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
        let frame_rec = Rectangle::new((self.curr_frame as i32 * (self.sprite.width()/self.frames as i32))as f32, 0.0, (self.sprite.width() as f32 /self.frames as f32) * self.direction as f32, self.sprite.height() as f32);
        // gets the x (0,0) is top left by multiplying the frame by (witdh/num of frames), y is always 0 as our spritesheets are 1D ish  width is just how big each frame should be having a negative value flips the sprite
        d_handle.draw_texture_rec(&self.sprite, frame_rec, <Point as Into<Vector2>>::into(pos), Color::WHITE);
    }

    pub fn update(&mut self, dir: i16) {
        self.direction = dir;
        self.advance();
    }

    pub fn set_frame(&mut self, new_frame: i16) {
        self.curr_frame = new_frame;
    }
}