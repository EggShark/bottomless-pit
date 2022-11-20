use raylib::consts::KeyboardKey;
use raylib::core::RaylibHandle;
use utils::Point;
use crate::buffer::inputs_to_point;

#[derive(Clone, Copy, PartialEq)]
pub enum MovmentKeys {
    None,
    LeftKey,
    RightKey,
    UpKey,
    DownKey,
}

pub enum AttackKeys {
    SlashKey,
    KickKey,
    HeavySlashKey,
}

// wrapping an array for easy implmentations and custom features
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Inputs {
    inputs: [KeyboardKey; 8],
} 
// as of 10/20/22 0 = up 1 = left 2 = down 3 = right 4 = slash 5 = heavy slash 6 = kick 
// 7 = sprint

impl Inputs {
    pub fn new(inputs: [KeyboardKey; 8]) -> Self {
        Self {
            inputs,
        }
    }

    pub fn is_sprint_down(&self, rl: &RaylibHandle) -> bool {
        rl.is_key_down(self.inputs[7])
    }

    pub fn is_movment_key_down(&self, key: MovmentKeys, rl: &RaylibHandle) -> bool {
        match key {
            MovmentKeys::UpKey => rl.is_key_down(self.inputs[0]),
            MovmentKeys::LeftKey => rl.is_key_down(self.inputs[1]),
            MovmentKeys::DownKey => rl.is_key_down(self.inputs[2]),
            MovmentKeys::RightKey => rl.is_key_down(self.inputs[3]),
            MovmentKeys::None => false, // not sure why id ever need to check this just gonna return false
        }
    }

    pub fn is_attack_key_pressed(&self, key: AttackKeys, rl: &RaylibHandle) -> bool {
        match key {
            AttackKeys::SlashKey => rl.is_key_pressed(self.inputs[4]),
            AttackKeys::HeavySlashKey => rl.is_key_pressed(self.inputs[5]),
            AttackKeys::KickKey => rl.is_key_pressed(self.inputs[6]),
        }
    }

    pub fn point_sum(&self, rl: &RaylibHandle) -> Point {
        let mut key_combo: Vec<MovmentKeys> = Vec::new();
        
        if self.is_movment_key_down(MovmentKeys::UpKey, rl) {
            key_combo.push(MovmentKeys::UpKey);
        }
        if self.is_movment_key_down(MovmentKeys::DownKey, rl) {
            key_combo.push(MovmentKeys::DownKey);
        }
        if self.is_movment_key_down(MovmentKeys::LeftKey, rl) {
            key_combo.push(MovmentKeys::LeftKey);
        }
        if self.is_movment_key_down(MovmentKeys::RightKey, rl) {
            key_combo.push(MovmentKeys::RightKey);
        }

        inputs_to_point(&key_combo)
    } 

    // just something to unwrap the array mainly used for settings
    pub fn get_raw(&self) -> [KeyboardKey; 8] {
        self.inputs
    }
}