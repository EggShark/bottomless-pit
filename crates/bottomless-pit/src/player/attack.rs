use raylib::{core::{RaylibThread, RaylibHandle}, prelude::RaylibDrawHandle};
use animation::{PlayerAnimation, HitBox};
use utils::Point;

pub(super) struct Attack {
    base_hitbox: HitBox, // all hitboxes should originate at 0,0
    actual: HitBox,      // then we shift this when the attack starts,
    animation: PlayerAnimation,
    base_damage: f32,
    frame_data: FrameData,
    state: AttackState,
    frame_count: i16,
}

enum AttackState {
    Startup,
    Active,
    Recovery,
}

pub(super) struct FrameData {
    startup: i16, // animation no hitbox
    active: i16,  // anumation + hitbox
    recovery: i16,// animation no hitbox
    on_block: i16,// how soon you can act
    on_hit: i16,  // does not include gat + special cancle
}

impl FrameData {
    pub fn new(startup: i16, active: i16, recovery: i16, on_block: i16, on_hit: i16) -> Self {
        Self {
            startup,
            active,
            recovery,
            on_block,
            on_hit,
        }
    }
}

impl Attack {
    pub fn new(base_hitbox: HitBox, path: &str, base_damage: f32, animation_frames: i16, frame_data: FrameData, rl: &mut RaylibHandle, thread: &RaylibThread) -> Self {
        let animation = PlayerAnimation::new(path, animation_frames, rl, thread);
        Self {
            actual: base_hitbox.copy(),
            base_hitbox,
            animation,
            base_damage,
            frame_data,
            state: AttackState::Startup,
            frame_count: 0,
        }
    }

    pub fn shift_actual(&mut self, shift_x: i32, shift_y: i32) {
        self.actual.shift_x(shift_x);
        self.actual.shift_y(shift_y);
    }

    pub fn reset_actual(&mut self) {
        self.actual = self.base_hitbox.copy();
    }

    pub fn draw(&self, pos: Point, d_handle: &mut RaylibDrawHandle) {
        self.animation.draw(d_handle, pos);
        match self.state {
            AttackState::Active => {
                self.actual.draw_hitbox(d_handle);
            },
            _ => {},
        }
    }

    pub fn update(&mut self) -> bool {
        self.frame_count += 1;
        self.animation.update(1);

        if self.frame_count > self.frame_data.active {
            self.state = AttackState::Recovery
        } else if self.frame_count > self.frame_data.startup {
            self.state = AttackState::Active
        }

        if self.frame_count == (self.frame_data.active + self.frame_data.recovery + self.frame_data.startup) {
            self.reset_actual();
            self.frame_count = 0;
            self.animation.set_frame(0);
            return true;
        }

        false
    }
}

pub(super) enum AttackType {
    Slash,
    Kick,
    Not,
}

impl AttackType {
    pub fn into_uszie(&self) -> usize {
        match self {
            Self::Slash => 0,
            Self::Kick => 1,
            Self::Not => 99999999999999,
        }
    }
}