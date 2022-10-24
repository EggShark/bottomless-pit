use raylib::core::drawing::RaylibDrawHandle;
use raylib::core::{RaylibThread, RaylibHandle};
use raylib::consts::KeyboardKey;
use utils::Point;
use animation::{PlayerAnimation, PlayerAnimations, HitBox, HitboxType};
use super::attack::{Attack, AttackType, FrameData};
use super::Inputs;

#[derive(Debug)]
pub struct Player {
    pos: Point,
    state: PlayerState,
    animation_state: PlayerAnimations,
    animations: [PlayerAnimation; 2],
    attacks: [Option<Attack>; 2],
    attack_type: AttackType,
    player_type: PlayerTypes,
    hurtbox: HitBox,
}

#[derive(Debug)]
pub enum PlayerTypes {
    BaseBaller,
    TestOne
}

#[derive(Debug, PartialEq)]
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

        let slash_hitbox_poly: Vec<Point> = vec![Point{x:0, y:0}, Point{x:10, y:0}, Point{x:10, y:20}, Point{x:0, y:20}];
        let slash_hitbox: HitBox = HitBox::new(slash_hitbox_poly, HitboxType::DamageDealing);
        let slash_frame_data = FrameData::new(1, 3, 3, -1, 2);
        let slash = Attack::new(slash_hitbox, "assets/slash_test.png", 10.0, 7, slash_frame_data, rl, thread);

        Self {
            pos,
            state: PlayerState::Normal,
            animation_state: PlayerAnimations::Idle,
            animations,
            attacks: [Some(slash), None],
            attack_type: AttackType::Kick,
            player_type: PlayerTypes::BaseBaller,
            hurtbox,
        }
    }

    pub fn draw(&self, d_handle: &mut RaylibDrawHandle) {
        match self.state {
            PlayerState::Normal => {
                self.draw_normal(d_handle);
            }
            PlayerState::Attacking => {
                self.draw_attack(d_handle);
            }
            _ => todo!()
        }
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
        self.attack(rl);

        match self.state {
            PlayerState::Normal => {
                self.normal_update(rl);
            },
            PlayerState::Attacking => {
                self.update_attacking();
            },
            _ => todo!(),
        }
    }

    fn normal_update(&mut self, rl: &RaylibHandle) {
        let mut dir = 1;

        // handles the movment we plan to have a 
        // more complex character controller
        // plus a input buffer for fg reasons
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

        let animation_pos: usize = self.animation_state.into_usize(); 
        self.animations[animation_pos].update(dir);
    }

    fn update_attacking(&mut self) {
        let attack = self.attacks[self.attack_type.into_uszie()].as_mut().unwrap(); 
        // shouldn't fail as the state will only be this way if there is an attack there

        // if it returns true the attack is 'over'
        if attack.update() {
            self.state = PlayerState::Normal;
        }
    }

    pub fn attack(&mut self, rl: &RaylibHandle) {
        // each input = attack type  check array and see if its some or none
        if self.state == PlayerState::Normal {
            let mut attack: Option<&mut Attack> = None;

            // checks for an attack input and
            // looks into the array of attacks
            if rl.is_key_pressed(KeyboardKey::KEY_I) {
                self.attack_type = AttackType::Slash;
                attack = self.attacks[AttackType::Slash.into_uszie()].as_mut()
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

    fn change_state(&mut self, new_state: PlayerState) {
        // do some fancy stuff later
        self.state = new_state;
    }
}