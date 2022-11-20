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
    frame_delay: i16,
    frame_counter: i16,
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
    pub fn new(path: &str, frames: i16, frame_delay: i16, rl: &mut RaylibHandle, thread: &RaylibThread) -> Self {
        let sprite = rl.load_texture(thread, path).unwrap();
        Self {
            sprite,
            frames,
            frame_delay,
            curr_frame: 0,
            frame_counter: 0,
            direction: 1,
        }
    }

    fn advance(&mut self) {
        self.frame_counter = 0;
        
        if self.curr_frame + 1 > self.frames {
            self.curr_frame = 0;
        }
        else {
            self.curr_frame += 1;
        }
    }

    pub fn draw(&self, d_handle: &mut RaylibDrawHandle, pos: Point) {
        let frame_width = self.sprite.width()/self.frames as i32;
        let frame_rec = Rectangle::new((self.curr_frame as i32 * frame_width )as f32, 0.0, (frame_width as f32) * self.direction as f32, self.sprite.height() as f32);
        // gets the x (0,0) is top left by multiplying the frame by (witdh/num of frames), y is always 0 as our spritesheets are 1D ish  width is just how big each frame should be having a negative value flips the sprite
        let destination_rec = Rectangle::new(pos.x as f32, pos.y as f32, (frame_width / 2) as f32, 200.0);
        d_handle.draw_texture_pro(&self.sprite, frame_rec, destination_rec, Vector2::new(0.0, 0.0), 0.0, Color::WHITE);
    }

    pub fn update(&mut self, dir: i16) {
        self.frame_counter += 1;

        if self.frame_counter >= self.frame_delay {
            self.advance();

        }

        self.direction = dir;
    }

    pub fn set_frame(&mut self, new_frame: i16) {
        self.curr_frame = new_frame;
    }
}