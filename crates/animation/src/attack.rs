use utils::Point;
use raylib::core::texture::Texture2D;

pub struct AttackAnimation {
    texture: Texture2D,
    pos: Point,
    baseDamage: f32,
}

pub enum AttackTypes {
    Punch,
    Slash,
    HeavySlash,
}