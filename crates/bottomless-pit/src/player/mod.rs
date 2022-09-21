use raylib::core::texture::Texture2D;
use utils::Point;
use animation::{PlayerAnimation, PlayerAnimations};

pub struct Player {
    texture: Texture2D,
    pos: Point,
    animation_state: PlayerAnimations,
    animations: [PlayerAnimation; 2],
    player_type: PlayerTypes,
}

pub enum PlayerTypes {
    BaseBaller,
    TestOne
}