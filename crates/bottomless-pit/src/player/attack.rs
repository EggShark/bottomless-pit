use raylib::core::{RaylibThread, RaylibHandle};
use animation::{PlayerAnimation, HitBox};

pub(super) struct Attack {
    hitbox: HitBox,
    animation: PlayerAnimation,
    base_damage: f32,
    frame_data: FrameData,
}

struct FrameData {
    startup: i16, // animation no hitbox
    active: i16,  // anumation + hitbox
    recovery: i16,// animation no hitbox
    on_block: i16,// how soon you can act
    on_hit: i16,  // does not include gat + special cancle
}

impl Attack {
    pub fn new(hitbox: HitBox, path: &str, base_damage: f32, animation_frames: u8, frame_data: FrameData, rl: &mut RaylibHandle, thread: &RaylibThread) -> Self {
        let animation = PlayerAnimation::new(path, animation_frames, rl, thread);
        Self {
            hitbox,
            animation,
            base_damage,
            frame_data,
        }
    }
}

pub(super) enum AttackType {
    Slash,
    Kick,
}

impl AttackType {
    pub fn into_uszie(&self) -> usize {
        match self {
            Self::Slash => 0,
            Self::Kick => 1,
        }
    }
}