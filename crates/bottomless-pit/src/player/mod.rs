use raylib::core::drawing::RaylibDrawHandle;
use raylib::core::{RaylibThread, RaylibHandle};
use raylib::consts::KeyboardKey;
use utils::Point;
use animation::{PlayerAnimation, PlayerAnimations};

pub struct Player {
    pos: Point,
    animation_state: PlayerAnimations,
    animations: [PlayerAnimation; 2],
    player_type: PlayerTypes,
}

pub enum PlayerTypes {
    BaseBaller,
    TestOne
}

impl Player {
    pub fn make_baller(rl: &mut RaylibHandle, thread: &RaylibThread, pos: Point) -> Self {
        let walk_anim = PlayerAnimation::new("assets/walk_forwards.png", 2, rl, thread);
        let idle = PlayerAnimation::new("assets/idle.png", 2, rl, thread);

        let animations: [PlayerAnimation; 2] = [idle, walk_anim];

        Self {
            pos,
            animation_state: PlayerAnimations::Idle,
            animations,
            player_type: PlayerTypes::BaseBaller,
        }
    }

    pub fn draw(&self, d_handle: &mut RaylibDrawHandle) {
        let animation_pos: usize = self.animation_state.into_uszie(); 
        self.animations[animation_pos].draw(d_handle, self.pos);
    }

    pub fn update(&mut self, rl: &RaylibHandle) {
        let mut dir: i8 = 1;
        self.animation_state = PlayerAnimations::Idle;

        if rl.is_key_down(KeyboardKey::KEY_D) {
            dir = 1;
            self.animation_state = PlayerAnimations::Walking;
            self.pos.x += 1;
        }

        if rl.is_key_down(KeyboardKey::KEY_A) {
            dir = -1;
            self.animation_state = PlayerAnimations::Walking;
            self.pos.x -= 1;
        }
        
        let animation_pos: usize = self.animation_state.into_uszie(); 
        self.animations[animation_pos].update(dir);
    }
}
