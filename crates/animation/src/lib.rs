use raylib::core::texture::Texture2D;

pub struct PlayerAnimation {
    sprite: Texture2D,
    animation_length: u8,
    curr_frame: i8,
}

pub enum PlayerAnimations {
    Walking,
    Running,
}