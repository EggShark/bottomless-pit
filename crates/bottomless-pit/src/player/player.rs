use raylib::core::drawing::RaylibDrawHandle;
use raylib::core::{RaylibThread, RaylibHandle};
use utils::Point;
use animation::{PlayerAnimation, PlayerAnimations, HitBox, HitboxType};
use input_handler::{AttackKeys, InputBuffer, Inputs, point_to_numpad};
use super::attack::{Attack, AttackType, AttackGuard, FrameData, OnHitData};

const TURN_AROUND: i32 = 640;

#[derive(Debug)]
pub struct Player {
    pos: Point,
    state: PlayerState,
    stun_timer: i16,
    animation_state: PlayerAnimations,
    animations: [PlayerAnimation; 3],
    attacks: [Option<Attack>; 2],
    attack_type: AttackType,
    player_type: PlayerTypes,
    hurtbox: HitBox,
    input_buffer: InputBuffer,
    health: f32,
    vel: Point,
}

#[derive(Debug)]
pub enum PlayerTypes {
    BaseBaller,
}

#[derive(Debug, PartialEq)]
pub enum PlayerState {
    Attacking,
    Normal,
    BlockStun,
    Hurt,
    Inactionable,
}

#[derive(Debug, PartialEq)]
pub enum AttackOutcome {
    Blocked,
    Hit,
}

impl Player {
    pub fn make_baller(rl: &mut RaylibHandle, thread: &RaylibThread, pos: Point) -> Self {
        let walk_anim = PlayerAnimation::new("assets/walk_forwards.png", 5, 10,rl, thread);
        let idle = PlayerAnimation::new("assets/idle.png", 4, 10, rl, thread);
        let run = PlayerAnimation::new("assets/walk_forwards.png", 5, 2, rl, thread);

        let animations: [PlayerAnimation; 3] = [idle, walk_anim, run];
        let poly = vec![Point{x: pos.x + 60, y: pos.y}, Point{x: pos.x + 200, y: pos.y}, Point{x:pos.x + 200, y: pos.y + 200}, Point{x: pos.x + 60, y: pos.y + 200}];
        let hurtbox = HitBox::new(poly, animation::HitboxType::DamageAble);

        let slash_hitbox_poly: Vec<Point> = vec![Point{x:75, y:75}, Point{x:200, y:75}, Point{x:200, y:200}, Point{x:75, y:200}];
        let slash_hitbox: HitBox = HitBox::new(slash_hitbox_poly, HitboxType::DamageDealing);
        let slash_frame_data = FrameData::new(1, 3, 3, -1, 2);
        let slash = Attack::new(slash_hitbox, AttackGuard::All, "assets/slash_test.png", 10.0, 7, 2, slash_frame_data, rl, thread);

        let kick_hitbox_poly: Vec<Point> = vec![Point{x:75, y:75}, Point{x:200, y:75}, Point{x:200, y:200}, Point{x:75, y:200}];
        let kick_hitbox: HitBox = HitBox::new(kick_hitbox_poly, HitboxType::DamageDealing);
        let kick_frame_data = FrameData::new(2, 2, 2, -1, 2);
        let kick = Attack::new(kick_hitbox, AttackGuard::All, "assets/kick.png", 2.0, 6, 4, kick_frame_data, rl, thread);

        Self {
            pos,
            state: PlayerState::Normal,
            stun_timer: 0,
            animation_state: PlayerAnimations::Idle,
            animations,
            attacks: [Some(slash), Some(kick)],
            attack_type: AttackType::Kick,
            player_type: PlayerTypes::BaseBaller,
            hurtbox,
            input_buffer: InputBuffer::new(),
            health: 100.0,
            vel: Point{x: 0, y: 0},
        }
    }

    pub fn draw(&self, d_handle: &mut RaylibDrawHandle) {
        match self.state {
            PlayerState::Normal => {
                self.draw_normal(d_handle);
            },
            PlayerState::BlockStun => {
                self.draw_normal(d_handle);
            },
            PlayerState::Attacking => {
                self.draw_attack(d_handle);
            },
            _ => todo!()
        }
    }

    pub fn get_health(&self) -> f32 {
        self.health
    }

    pub fn get_hurtbox(&self) -> &HitBox {
        &self.hurtbox
    }

    pub fn get_active_attack(&self) -> Option<&Attack> {
        match self.state {
            PlayerState::Attacking => {
                self.attacks[self.attack_type.into_uszie()].as_ref()
            },
            _ => None,
        }
    }
    // 🐢
    pub fn is_blocking_n(&self) -> bool {
        // turtle and beyond 🐢
        let dir = {
            if self.pos.x > TURN_AROUND {
                true // right of point
            } else {
                false // left of point
            }
        };
        if dir {
            is_rightwards(self.input_buffer.get(0)) && is_neutral(self.input_buffer.get(0))
        } else {
            is_leftwards(self.input_buffer.get(0)) && is_neutral(self.input_buffer.get(0))
        }
    }

    pub fn is_blocking(&self) -> bool {
        let dir = {
            if self.pos.x > TURN_AROUND {
                true // right of point
            } else {
                false // left of point
            }
        };
        if dir {
            is_rightwards(self.input_buffer.get(0))
        } else {
            is_leftwards(self.input_buffer.get(0))
        }
    }

    pub fn is_blocking_down(&self) -> bool {
        let dir = {
            if self.pos.x > TURN_AROUND {
                true // right of point
            } else {
                false // left of point
            }
        };
        if dir {
            is_rightwards(self.input_buffer.get(0)) && is_downwards(self.input_buffer.get(0))
        } else {
            is_leftwards(self.input_buffer.get(0)) && is_downwards(self.input_buffer.get(0))
        }
    }

    pub fn is_grounded(&self) -> bool {
        self.pos.y == 520
    }

    fn draw_normal(&self, d_handle: &mut RaylibDrawHandle) {
        let animation_pos: usize = self.animation_state.into_usize(); 
        self.animations[animation_pos].draw(d_handle, self.pos);
        self.hurtbox.draw_hitbox(d_handle);
    }

    fn draw_attack(&self, d_handle: &mut RaylibDrawHandle) {
        let attack = self.attacks[self.attack_type.into_uszie()].as_ref().unwrap(); 
        attack.draw(self.pos, d_handle);
        self.hurtbox.draw_hitbox(d_handle);
    }

    pub fn update(&mut self, rl: &RaylibHandle, keys: &Inputs) {
        // resets to default animation state when needed
        // however almost any action in the other checks will
        // change the state so it is rarley kept at idle
        self.animation_state = PlayerAnimations::Idle;
        self.attack(rl, keys);

        // updates the input buffer I dont do anything about it rn
        // input buffer will always buffer the inputs :)
        let mb_point = keys.point_sum(rl);
        let numpad_notation = point_to_numpad(mb_point);
        self.input_buffer.new_input(numpad_notation);

        match self.state {
            PlayerState::Normal => {
                self.normal_update(rl, keys);
            },
            PlayerState::Attacking => {
                self.update_attacking();
            },
            PlayerState::BlockStun => {
                self.stun_timer -= 1;
                if self.stun_timer == 0 {
                    self.state = PlayerState::Normal;
                }
            },
            _ => todo!(),
        }
        self.apply_velocity();
    }

    fn normal_update(&mut self, rl: &RaylibHandle, keys: &Inputs) {
        // just checking stuff for now
        let dir = {
            if self.pos.x > TURN_AROUND {
                -1
            } else {
                1
            }
        };

        // handles the movment we plan to have a 
        // more complex character controller
        // plus a input buffer for fg reasons
        if self.is_grounded() {
            self.vel.y = 0;
        }

        if is_rightwards(self.input_buffer.get(0)) && !is_downwards(self.input_buffer.get(0)) {
            self.animation_state = PlayerAnimations::Walking;
            self.vel.x += 3;

            if keys.is_sprint_down(rl) {
                self.vel.x *= 2;
                self.animation_state = PlayerAnimations::Running;
            }
        }

        if is_leftwards(self.input_buffer.get(0)) && !is_downwards(self.input_buffer.get(0)) {
            self.animation_state = PlayerAnimations::Walking;
            self.vel.x -= 3;

            if keys.is_sprint_down(rl) {
                self.vel.x *= 2;
                self.animation_state = PlayerAnimations::Running;
            }
        }

        // most recent input
        if is_upwards(self.input_buffer.get(0)) && self.is_grounded() {
            println!("Jump!");
            self.vel.y = -25;
        }

        // clamps the player to the "floor"
        if self.pos.y > 520 {
            self.pos.y = 520;
            self.vel.y = 0;
            let dif = self.pos.y - self.get_hurtbox().get_poly()[0].y;
            self.hurtbox.shift_y(dif);
        }

        self.apply_gravity();

        let animation_pos: usize = self.animation_state.into_usize(); 
        self.animations[animation_pos].update(dir);
    }

    fn update_attacking(&mut self) {
        // the player shouldnt fall as they attack
        self.vel.y = 0;

        let attack = self.attacks[self.attack_type.into_uszie()].as_mut().unwrap(); 
        // shouldn't fail as the state will only be this way if there is an attack there
        // if it returns true the attack is 'over'
        if attack.update() {
            self.state = PlayerState::Normal;
        }
    }

    pub fn attack(&mut self, rl: &RaylibHandle, keys: &Inputs) {
        // each input = attack type  check array and see if its some or none
        if self.state == PlayerState::Normal {
            let mut attack: Option<&mut Attack> = None;

            // checks for an attack input and
            // looks into the array of attacks
            if keys.is_attack_key_pressed(AttackKeys::SlashKey, rl) {
                self.attack_type = AttackType::Slash;
                attack = self.attacks[AttackType::Slash.into_uszie()].as_mut()
            }

            if keys.is_attack_key_pressed(AttackKeys::KickKey, rl) {
                self.attack_type = AttackType::Kick;
                attack = self.attacks[AttackType::Kick.into_uszie()].as_mut()
            }
    
            match attack {
                Some(attack) => {
                    // updates the player as well as the attacks
                    // hitbox to the appropriate space
                    self.state = PlayerState::Attacking;
                    attack.shift_actual(self.pos.x, self.pos.y);
                },
                None => {}
            }
        }
    }

    fn apply_velocity(&mut self) {
        self.pos += self.vel;
        self.hurtbox.shift_point(self.vel);
        self.vel.x = 0; // dont change y as gravity will do it for us
    }

    fn apply_gravity(&mut self) {
        if !self.is_grounded() {
            self.vel.y += 2;
        } 
    }

    pub fn on_hit(&mut self, data: OnHitData, on_block: i16, on_hit: i16) -> AttackOutcome {
        match data.get_guard() {
            AttackGuard::All => {
                if self.is_blocking() {
                    // add stun
                    self.state = PlayerState::BlockStun;
                    self.stun_timer += on_block;
                    return AttackOutcome::Blocked;
                }
            },
            AttackGuard::Low => {
                if self.is_blocking_down() {
                    self.state = PlayerState::BlockStun;
                    self.stun_timer += on_block;
                    return AttackOutcome::Blocked;
                }
            },
            AttackGuard::High => {
                if self.is_blocking_n() {
                    self.state = PlayerState::BlockStun;
                    self.stun_timer += on_block;
                    return AttackOutcome::Blocked;
                }
            },
        }
        // do more idk,
        self.health -= data.get_base_damage();
        self.vel += data.get_knock_back_v();



        AttackOutcome::Hit
    }

    fn change_state(&mut self, new_state: PlayerState) {
        // do some fancy stuff later
        self.state = new_state;
    }
}

// checks the numpad notation to see if its an "upward motion"
fn is_upwards(num_pad: i32) -> bool {
    num_pad >= 7 && num_pad <= 9
}

fn is_neutral(num_pad: i32) -> bool {
    num_pad >= 4 && num_pad <= 7
}

fn is_downwards(num_pad: i32) -> bool {
    num_pad >= 1 && num_pad <= 3
}

fn is_rightwards(num_pad: i32) -> bool {
    num_pad % 3 == 0
}

fn is_leftwards(num_pad: i32) -> bool {
    num_pad % 3 == 1
}