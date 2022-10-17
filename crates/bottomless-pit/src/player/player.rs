use raylib::core::drawing::RaylibDrawHandle;
use raylib::core::{RaylibThread, RaylibHandle};
use raylib::consts::KeyboardKey;
use utils::Point;
use animation::{PlayerAnimation, PlayerAnimations, HitBox};
use super::attack::{Attack, AttackType};


pub struct Player {
    pos: Point,
    state: PlayerState,
    animation_state: PlayerAnimations,
    animations: [PlayerAnimation; 2],
    attacks: [Option<Attack>; 2],
    player_type: PlayerTypes,
    hurtbox: HitBox,
}

pub enum PlayerTypes {
    BaseBaller,
    TestOne
}

#[derive(PartialEq)]
pub enum PlayerState {
    Attacking,
    Normal,
    Hurt,
    Inactionable,
}

impl Player {
    pub fn make_baller(rl: &mut RaylibHandle, thread: &RaylibThread, pos: Point) -> Self {
        let walk_anim = PlayerAnimation::new("assets/walk_forwards.png", 2, rl, thread);
        let idle = PlayerAnimation::new("assets/idle.png", 2, rl, thread);

        let animations: [PlayerAnimation; 2] = [idle, walk_anim];
        let poly = vec![Point{x: pos.x, y: pos.y}, Point{x: pos.x + 64, y: pos.y}, Point{x:pos.x + 64, y: pos.y + 64}, Point{x: pos.x, y: pos.y + 64}];
        let hurtbox = HitBox::new(poly, animation::HitboxType::DamageAble);

        let slash = Attack::new(hitbox, path, base_damage, animation_frames, frame_data, rl, thread);

        Self {
            pos,
            state: PlayerState::Normal,
            animation_state: PlayerAnimations::Idle,
            animations,
            attacks: [None, None],
            player_type: PlayerTypes::BaseBaller,
            hurtbox,
        }
    }

    pub fn draw(&self, d_handle: &mut RaylibDrawHandle) {
        let animation_pos: usize = self.animation_state.into_usize(); 
        self.animations[animation_pos].draw(d_handle, self.pos);
        self.hurtbox.draw_hibox(d_handle);
    }

    pub fn update(&mut self, rl: &RaylibHandle) {
        let mut dir: i8 = 1;
        self.animation_state = PlayerAnimations::Idle;

        self.attack(rl);

        if self.state == PlayerState::Normal {
            if rl.is_key_down(KeyboardKey::KEY_D) {
                dir = 1;
                self.animation_state = PlayerAnimations::Walking;
                self.pos.x += 1;
                self.hurtbox.shift_x(1);
            }
    
            if rl.is_key_down(KeyboardKey::KEY_A) {
                dir = -1;
                self.animation_state = PlayerAnimations::Walking;
                self.pos.x -= 1;
                self.hurtbox.shift_x(-1);
            }    
        }

        let animation_pos: usize = self.animation_state.into_usize(); 
        self.animations[animation_pos].update(dir);
    }

    pub fn attack(&mut self, rl: &RaylibHandle) {
        // each input = attack type  check array and see if its some or none
        if self.state == PlayerState::Normal {
            let mut attack: Option<&Attack> = None;
            if rl.is_key_pressed(KeyboardKey::KEY_I) {
                attack = self.attacks[AttackType::Slash.into_uszie()].as_ref()
            }
    
            match attack {
                Some(attack) => {
                    self.state = PlayerState::Attacking;
                    todo!()
                },
                None => {}
            }
        }
    }

    fn change_state(&mut self, new_state: PlayerState) {
        // do some fancy stuff later
        self.state = new_state;
    }
}