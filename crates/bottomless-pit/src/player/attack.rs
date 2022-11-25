use raylib::{core::{RaylibThread, RaylibHandle}, prelude::RaylibDrawHandle};
use animation::{PlayerAnimation, HitBox};
use utils::Point;

#[derive(Debug)]
pub struct Attack {
    base_hitbox: HitBox, // all hitboxes should originate at 0,0
    actual: HitBox,      // then we shift this when the attack starts,
    animation: PlayerAnimation,
    hit_data: OnHitData,
    frame_data: FrameData,
    state: AttackState,
    frame_count: i16,
}

#[derive(Debug, Clone, Copy)]
pub struct OnHitData {
    base_damage: f32,
    knock_down: bool,
    guard: AttackGuard,
    knock_back: Point,
}

impl OnHitData {
    pub fn new(base_damage: f32, knock_down: bool, guard: AttackGuard, knock_back: Point) -> Self {
        Self {
            base_damage,
            knock_down,
            guard,
            knock_back,
        }
    }

    pub fn get_base_damage(&self) -> f32 {
        self.base_damage
    }

    pub fn get_knock_down(&self) -> bool {
        self.knock_down
    }

    pub fn get_guard(&self) -> AttackGuard {
        self.guard
    }

    pub fn get_knock_back_v(&self) -> Point {
        self.knock_back
    }
}

#[derive(Debug)]
enum AttackState {
    Startup,
    Active,
    Recovery,
}

#[derive(Debug, Clone, Copy)]
pub enum AttackGuard {
    Low,
    All,
    High,
}

// frame data is no based on animation frames
// we account for animation delay when creating the attack
#[derive(Debug)]
pub struct FrameData {
    startup: i16, // animation no hitbox
    active: i16,  // anumation + hitbox
    recovery: i16,// animation no hitbox
    on_block: i16,// how soon you can act
    on_hit: i16,  // does not include gat + special cancel
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

    pub fn add_delay(&mut self, frame_delay: i16) {
        self.startup *= frame_delay;
        self.active *= frame_delay;
        self.recovery *= frame_delay;
    }
}

impl Attack {
    pub fn new(base_hitbox: HitBox, guard: AttackGuard, path: &str, base_damage: f32, animation_frames: i16, frame_delay: i16, mut frame_data: FrameData, rl: &mut RaylibHandle, thread: &RaylibThread) -> Self {
        let animation = PlayerAnimation::new(path, animation_frames, frame_delay, rl, thread);
        let hit_data = OnHitData::new(base_damage, false, guard, Point{x: 30, y: 10});
        frame_data.add_delay(frame_delay); // accounts for the animation delay

        Self {
            actual: base_hitbox.copy(),
            base_hitbox,
            animation,
            hit_data,
            frame_data,
            state: AttackState::Startup,
            frame_count: 0,
        }
    }

    pub fn shift_actual(&mut self, shift_x: i32, shift_y: i32) {
        // used to have a hitbox based off where the player actually is
        self.actual.shift_x(shift_x);
        self.actual.shift_y(shift_y);
    }

    pub fn reset_actual(&mut self) {
        // resets the hitbox allowing us to change it for the next attack
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

    // reutrns true if the attack is over I liked this more
    // compared to a is_over functions
    pub fn update(&mut self) -> bool {
        self.frame_count += 1;
        self.animation.update(1);

        // sets the state to the approraite state based off the frame data
        if self.frame_count > self.frame_data.active + self.frame_data.startup {
            self.state = AttackState::Recovery
        } else if self.frame_count > self.frame_data.startup {
            self.state = AttackState::Active
        }


        // checks to see if the attack is "over"
        if self.frame_count == (self.frame_data.active + self.frame_data.recovery + self.frame_data.startup) {
            self.reset_actual();
            self.frame_count = 0;
            self.animation.set_frame(0);
            return true;
        }

        false
    }

    pub fn get_hit_data(&self) -> OnHitData {
        self.hit_data
    }

    pub fn get_curr_hitbox(&self) -> &HitBox {
        &self.actual
    }
}

#[derive(Debug)]
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