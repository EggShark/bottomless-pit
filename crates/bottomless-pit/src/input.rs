pub(crate) struct InputHandle {
    previous_keyboard_state: [bool; 115],
    current_keyboard_state: [bool; 115],
}

impl InputHandle {
    pub(crate) fn new() -> Self {
        Self {
            previous_keyboard_state: [false; 115],
            current_keyboard_state: [false; 115],
        }
    }

    pub(crate) fn end_of_frame_refresh(&mut self) {
        self.previous_keyboard_state = self.current_keyboard_state;
    }

    pub(crate) fn process_input(&mut self, key_code: &Option<VirtualKeyCode>, state: winit::event::ElementState) -> bool {
        let key_bool = state == ElementState::Pressed;
        let key: Key = match key_code {
            Some(virtual_code) => {
                let c = *virtual_code;
                c.into()
            },
            None => return false,
        };
        
        if key == Key::Unrecognized {
            return false;
        }

        let index = key as usize;
        self.current_keyboard_state[index] = key_bool;
        true
    }

    pub(crate) fn is_key_down(&self, key: Key) -> bool {
        let index = key as usize;
        self.current_keyboard_state[index]
    }

    pub(crate) fn is_key_up(&self, key: Key) -> bool {
        let index = key as usize;
        !self.current_keyboard_state[index]
    }

    pub(crate) fn is_key_pressed(&self, key: Key) -> bool {
        let index = key as usize;
        !self.previous_keyboard_state[index] && self.current_keyboard_state[index]
    }

    pub(crate) fn is_key_released(&self, key: Key) -> bool {
        let index = key as usize;
        self.previous_keyboard_state[index] && !self.current_keyboard_state[index]
    }
}

#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Key {
    Key1,
    Key2,
    Key3,
    Key4,
    Key5,
    Key6,
    Key7,
    Key8,
    Key9,
    Key0,
    A,
    B,
    C,
    D,
    E,
    F,
    G,
    H,
    I,
    J,
    K,
    L,
    M,
    N,
    O,
    P,
    Q,
    R,
    S,
    T,
    U,
    V,
    W,
    X,
    Y,
    Z,
    Esc,
    F1,
    F2,
    F3,
    F4,
    F5,
    F6,
    F7,
    F8,
    F9,
    F10,
    F11,
    F12,
    F13,
    F14,
    F15,
    F16,
    F17,
    F18,
    F19,
    F20,
    F21,
    F22,
    F23,
    F24,
    ScrollLock,
    Home,
    Delete,
    End,
    PageDown,
    PageUp,
    Left,
    Up,
    Right,
    Down,
    BackSpace,
    Enter,
    Space,
    Numlock,
    Numpad0,
    Numpad1,
    Numpad2,
    Numpad3,
    Numpad4,
    Numpad5,
    Numpad6,
    Numpad7,
    Numpad8,
    Numpad9,
    NumpadAdd,
    NumpadDivide,
    NumpadDecimial,
    NumpadComma,
    NumpadEnter,
    NumpadEquals,
    NumpadMultiply,
    NumpadSubtract,
    Apostrophe,
    Asterisk,
    BackSlash,
    CapsLock,
    Colon,
    Comma,
    Equals,
    LeftAlt,
    LeftBracket,
    LeftControl,
    LeftShift,
    Minus,
    Period,
    Plus,
    RightAlt,
    RightBracket,
    RightControl,
    RightShift,
    SemiColon,
    Slash,
    Tab,
    UnderLine,
    Unrecognized,
}

use winit::event::{VirtualKeyCode, ElementState};
impl Into<Key> for VirtualKeyCode {
    fn into(self) -> Key {
        match self {
            VirtualKeyCode::Key0 => Key::Key0,
            VirtualKeyCode::Key1 => Key::Key1,
            VirtualKeyCode::Key2 => Key::Key2,
            VirtualKeyCode::Key3 => Key::Key3,
            VirtualKeyCode::Key4 => Key::Key4,
            VirtualKeyCode::Key5 => Key::Key5,
            VirtualKeyCode::Key6 => Key::Key6,
            VirtualKeyCode::Key7 => Key::Key7,
            VirtualKeyCode::Key8 => Key::Key8,
            VirtualKeyCode::Key9 => Key::Key9,
            VirtualKeyCode::A => Key::A,
            VirtualKeyCode::B => Key::B,
            VirtualKeyCode::C => Key::C,
            VirtualKeyCode::D => Key::D,
            VirtualKeyCode::E => Key::E,
            VirtualKeyCode::F => Key::F,
            VirtualKeyCode::G => Key::G,
            VirtualKeyCode::H => Key::H,
            VirtualKeyCode::I => Key::I,
            VirtualKeyCode::J => Key::J,
            VirtualKeyCode::K => Key::K,
            VirtualKeyCode::L => Key::L,
            VirtualKeyCode::M => Key::M,
            VirtualKeyCode::N => Key::N,
            VirtualKeyCode::O => Key::O,
            VirtualKeyCode::P => Key::P,
            VirtualKeyCode::Q => Key::Q,
            VirtualKeyCode::R => Key::R,
            VirtualKeyCode::S => Key::S,
            VirtualKeyCode::T => Key::T,
            VirtualKeyCode::U => Key::U,
            VirtualKeyCode::V => Key::V,
            VirtualKeyCode::W => Key::W,
            VirtualKeyCode::X => Key::X,
            VirtualKeyCode::Y => Key::Y,
            VirtualKeyCode::Z => Key::Z,
            VirtualKeyCode::Escape => Key::Esc,
            VirtualKeyCode::F1 => Key::F1,
            VirtualKeyCode::F2 => Key::F2,
            VirtualKeyCode::F3 => Key::F3,
            VirtualKeyCode::F4 => Key::F4,
            VirtualKeyCode::F5 => Key::F5,
            VirtualKeyCode::F6 => Key::F6,
            VirtualKeyCode::F7 => Key::F7,
            VirtualKeyCode::F8 => Key::F8,
            VirtualKeyCode::F9 => Key::F9,
            VirtualKeyCode::F10 => Key::F10,
            VirtualKeyCode::F11 => Key::F11,
            VirtualKeyCode::F12 => Key::F12,
            VirtualKeyCode::F13 => Key::F13,
            VirtualKeyCode::F14 => Key::F14,
            VirtualKeyCode::F15 => Key::F15,
            VirtualKeyCode::F16 => Key::F16,
            VirtualKeyCode::F17 => Key::F17,
            VirtualKeyCode::F18 => Key::F18,
            VirtualKeyCode::F19 => Key::F19,
            VirtualKeyCode::F20 => Key::F20,
            VirtualKeyCode::F21 => Key::F21,
            VirtualKeyCode::F22 => Key::F22,
            VirtualKeyCode::F23 => Key::F23,
            VirtualKeyCode::F24 => Key::F24,
            VirtualKeyCode::Scroll => Key::ScrollLock,
            VirtualKeyCode::Home => Key::Home,
            VirtualKeyCode::Delete => Key::Delete,
            VirtualKeyCode::End => Key::End,
            VirtualKeyCode::PageUp => Key::PageUp,
            VirtualKeyCode::PageDown => Key::PageDown,
            VirtualKeyCode::Left => Key::Left,
            VirtualKeyCode::Up => Key::Up,
            VirtualKeyCode::Right => Key::Right,
            VirtualKeyCode::Down => Key::Down,
            VirtualKeyCode::Back => Key::BackSpace,
            VirtualKeyCode::Return => Key::Enter,
            VirtualKeyCode::Space => Key::Space,
            VirtualKeyCode::Numlock => Key::Numlock,
            VirtualKeyCode::Numpad0 => Key::Numpad0,
            VirtualKeyCode::Numpad1 => Key::Numpad1,
            VirtualKeyCode::Numpad2 => Key::Numpad2,
            VirtualKeyCode::Numpad3 => Key::Numpad3,
            VirtualKeyCode::Numpad4 => Key::Numpad4,
            VirtualKeyCode::Numpad5 => Key::Numpad5,
            VirtualKeyCode::Numpad6 => Key::Numpad6,
            VirtualKeyCode::Numpad7 => Key::Numpad7,
            VirtualKeyCode::Numpad8 => Key::Numpad8,
            VirtualKeyCode::Numpad9 => Key::Numpad9,
            VirtualKeyCode::NumpadAdd => Key::NumpadAdd,
            VirtualKeyCode::NumpadDivide => Key::NumpadDivide,
            VirtualKeyCode::NumpadDecimal => Key::NumpadDecimial,
            VirtualKeyCode::NumpadComma => Key::NumpadComma,
            VirtualKeyCode::NumpadEnter => Key::NumpadEnter,
            VirtualKeyCode::NumpadEquals => Key::NumpadEquals,
            VirtualKeyCode::NumpadMultiply => Key::NumpadMultiply,
            VirtualKeyCode::NumpadSubtract => Key::NumpadSubtract,
            VirtualKeyCode::Apostrophe => Key::Apostrophe,
            VirtualKeyCode::Asterisk => Key::Asterisk,
            VirtualKeyCode::Backslash => Key::BackSlash,
            VirtualKeyCode::Capital => Key::CapsLock,
            VirtualKeyCode::Colon => Key::Colon,
            VirtualKeyCode::Comma => Key::Comma,
            VirtualKeyCode::Equals => Key::Equals,
            VirtualKeyCode::LAlt => Key::LeftAlt,
            VirtualKeyCode::LBracket => Key::LeftBracket,
            VirtualKeyCode::LControl => Key::LeftControl,
            VirtualKeyCode::LShift => Key::LeftShift,
            VirtualKeyCode::Minus => Key::Minus,
            VirtualKeyCode::Period => Key::Period,
            VirtualKeyCode::Plus => Key::Plus,
            VirtualKeyCode::RAlt => Key::RightAlt,
            VirtualKeyCode::RBracket => Key::RightBracket,
            VirtualKeyCode::RControl => Key::RightControl,
            VirtualKeyCode::RShift => Key::RightShift,
            VirtualKeyCode::Semicolon => Key::SemiColon,
            VirtualKeyCode::Slash => Key::Slash,
            VirtualKeyCode::Tab => Key::Tab,
            VirtualKeyCode::Underline => Key::UnderLine,
            _ => Key::Unrecognized,
        }
    }
}