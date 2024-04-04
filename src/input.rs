//! Contains Both the MouseKey and Key Enums for input
//! ```rust,no_run
//! impl Game For Struct {
//!     fn update(&mut self, engine_handle: &mut Engine) {
//!         if engine_handle.is_key_down(Key::W) {
//!             // do something
//!         }
//!         if engine_handle.is_mouse_key_down(MouseKey::Left) {
//!             // do more things
//!         }
//!     }
//! }

use winit::event::{ElementState, KeyEvent, MouseButton, WindowEvent};
use winit::keyboard::{PhysicalKey, KeyCode};

use crate::vectors::Vec2;

pub(crate) struct InputHandle {
    previous_keyboard_state: [bool; 111],
    current_keyboard_state: [bool; 111],
    previous_mouse_state: [bool; 6],
    current_mouse_state: [bool; 6],
    mouse_position: Vec2<f32>,
}

impl InputHandle {
    pub(crate) fn new() -> Self {
        Self {
            previous_keyboard_state: [false; 111],
            current_keyboard_state: [false; 111],
            previous_mouse_state: [false; 6],
            current_mouse_state: [false; 6],
            mouse_position: Vec2 { x: 0.0, y: 0.0 },
        }
    }

    pub(crate) fn end_of_frame_refresh(&mut self) {
        self.previous_keyboard_state = self.current_keyboard_state;
        self.previous_mouse_state = self.current_mouse_state;
    }

    pub(crate) fn process_input(&mut self, event: &WindowEvent) -> bool {
        match event {
            WindowEvent::KeyboardInput {
                event: key_event,
                ..
            } => self.process_keyboard_input(key_event),
            WindowEvent::MouseInput { state, button, .. } => {
                self.process_mouse_input(*state, *button)
            }
            WindowEvent::CursorMoved { position, .. } => {
                let pos = Vec2 {
                    x: position.x as f32,
                    y: position.y as f32,
                };
                self.mouse_position = pos;
                true
            }
            _ => false,
        }
    }

    fn process_mouse_input(&mut self, state: ElementState, button: MouseButton) -> bool {
        let key_bool = state == ElementState::Pressed;
        let key: MouseKey = button.into();
        let idx: usize = key as usize;

        self.current_mouse_state[idx] = key_bool;
        true
    }

    fn process_keyboard_input(
        &mut self,
        event: &KeyEvent,
    ) -> bool {
        let key_bool = event.state == ElementState::Pressed;
        let key_code = match event.physical_key {
            PhysicalKey::Code(c) => c,
            PhysicalKey::Unidentified(_) => return false,
        };

        let key: Key = key_code.into(); 

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

    pub(crate) fn is_mouse_key_down(&self, key: MouseKey) -> bool {
        let index = key as usize;
        self.current_mouse_state[index]
    }

    pub(crate) fn is_mouse_key_up(&self, key: MouseKey) -> bool {
        let index = key as usize;
        !self.current_keyboard_state[index]
    }

    pub(crate) fn is_mouse_key_pressed(&self, key: MouseKey) -> bool {
        let index = key as usize;
        !self.previous_mouse_state[index] && self.current_mouse_state[index]
    }

    pub(crate) fn is_mouse_key_released(&self, key: MouseKey) -> bool {
        let index = key as usize;
        self.previous_mouse_state[index] && !self.current_mouse_state[index]
    }

    pub(crate) fn get_mouse_position(&self) -> Vec2<f32> {
        self.mouse_position
    }
}

/// Representation of mouse buttons
#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum MouseKey {
    Left,
    Right,
    Middle,
    Back,
    Fowrawrds,
    Other,
}

impl From<MouseButton> for MouseKey {
    fn from(val: MouseButton) -> Self {
        match val {
            MouseButton::Left => MouseKey::Left,
            MouseButton::Right => MouseKey::Right,
            MouseButton::Middle => MouseKey::Middle,
            MouseButton::Forward => MouseKey::Fowrawrds,
            MouseButton::Back => MouseKey::Back,
            MouseButton::Other(_) => MouseKey::Other,
        }
    }
}

/// Representation of keyboard keys
#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
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
    BackSlash,
    CapsLock,
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
    Unrecognized,
}

impl From<KeyCode> for Key {
    fn from(val: KeyCode) -> Self {
        match val {
            KeyCode::Digit0 => Key::Key0,
            KeyCode::Digit1 => Key::Key1,
            KeyCode::Digit2 => Key::Key2,
            KeyCode::Digit3 => Key::Key3,
            KeyCode::Digit4 => Key::Key4,
            KeyCode::Digit5 => Key::Key5,
            KeyCode::Digit6 => Key::Key6,
            KeyCode::Digit7 => Key::Key7,
            KeyCode::Digit8 => Key::Key8,
            KeyCode::Digit9 => Key::Key9,
            KeyCode::Equal => Key::Equals,
            KeyCode::KeyA => Key::A,
            KeyCode::KeyB => Key::B,
            KeyCode::KeyC => Key::C,
            KeyCode::KeyD => Key::D,
            KeyCode::KeyE => Key::E,
            KeyCode::KeyF => Key::F,
            KeyCode::KeyG => Key::G,
            KeyCode::KeyH => Key::H,
            KeyCode::KeyI => Key::I,
            KeyCode::KeyJ => Key::J,
            KeyCode::KeyK => Key::K,
            KeyCode::KeyL => Key::L,
            KeyCode::KeyM => Key::M,
            KeyCode::KeyN => Key::N,
            KeyCode::KeyO => Key::O,
            KeyCode::KeyP => Key::P,
            KeyCode::KeyQ => Key::Q,
            KeyCode::KeyR => Key::R,
            KeyCode::KeyS => Key::S,
            KeyCode::KeyT => Key::T,
            KeyCode::KeyU => Key::U,
            KeyCode::KeyV => Key::V,
            KeyCode::KeyW => Key::W,
            KeyCode::KeyX => Key::X,
            KeyCode::KeyY => Key::Y,
            KeyCode::KeyZ => Key::Z,
            KeyCode::Escape => Key::Esc,
            KeyCode::F1 => Key::F1,
            KeyCode::F2 => Key::F2,
            KeyCode::F3 => Key::F3,
            KeyCode::F4 => Key::F4,
            KeyCode::F5 => Key::F5,
            KeyCode::F6 => Key::F6,
            KeyCode::F7 => Key::F7,
            KeyCode::F8 => Key::F8,
            KeyCode::F9 => Key::F9,
            KeyCode::F10 => Key::F10,
            KeyCode::F11 => Key::F11,
            KeyCode::F12 => Key::F12,
            KeyCode::F13 => Key::F13,
            KeyCode::F14 => Key::F14,
            KeyCode::F15 => Key::F15,
            KeyCode::F16 => Key::F16,
            KeyCode::F17 => Key::F17,
            KeyCode::F18 => Key::F18,
            KeyCode::F19 => Key::F19,
            KeyCode::F20 => Key::F20,
            KeyCode::F21 => Key::F21,
            KeyCode::F22 => Key::F22,
            KeyCode::F23 => Key::F23,
            KeyCode::F24 => Key::F24,
            KeyCode::ScrollLock => Key::ScrollLock,
            KeyCode::Home => Key::Home,
            KeyCode::Delete => Key::Delete,
            KeyCode::End => Key::End,
            KeyCode::PageUp => Key::PageUp,
            KeyCode::PageDown => Key::PageDown,
            KeyCode::ArrowLeft => Key::Left,
            KeyCode::ArrowUp => Key::Up,
            KeyCode::ArrowRight => Key::Right,
            KeyCode::ArrowDown => Key::Down,
            KeyCode::Backspace => Key::BackSpace,
            KeyCode::Enter => Key::Enter,
            KeyCode::Space => Key::Space,
            KeyCode::NumLock => Key::Numlock,
            KeyCode::Numpad0 => Key::Numpad0,
            KeyCode::Numpad1 => Key::Numpad1,
            KeyCode::Numpad2 => Key::Numpad2,
            KeyCode::Numpad3 => Key::Numpad3,
            KeyCode::Numpad4 => Key::Numpad4,
            KeyCode::Numpad5 => Key::Numpad5,
            KeyCode::Numpad6 => Key::Numpad6,
            KeyCode::Numpad7 => Key::Numpad7,
            KeyCode::Numpad8 => Key::Numpad8,
            KeyCode::Numpad9 => Key::Numpad9,
            KeyCode::NumpadAdd => Key::NumpadAdd,
            KeyCode::NumpadDivide => Key::NumpadDivide,
            KeyCode::NumpadDecimal => Key::NumpadDecimial,
            KeyCode::NumpadComma => Key::NumpadComma,
            KeyCode::NumpadEnter => Key::NumpadEnter,
            KeyCode::NumpadEqual => Key::NumpadEquals,
            KeyCode::NumpadMultiply => Key::NumpadMultiply,
            KeyCode::NumpadSubtract => Key::NumpadSubtract,
            KeyCode::Backslash => Key::BackSlash,
            KeyCode::CapsLock => Key::CapsLock,
            KeyCode::Comma => Key::Comma,
            KeyCode::AltLeft => Key::LeftAlt,
            KeyCode::BracketLeft => Key::LeftBracket,
            KeyCode::ControlLeft => Key::LeftControl,
            KeyCode::ShiftLeft => Key::LeftShift,
            KeyCode::Minus => Key::Minus,
            KeyCode::Period => Key::Period,
            KeyCode::AltRight => Key::RightAlt,
            KeyCode::BracketRight => Key::RightBracket,
            KeyCode::ControlRight => Key::RightControl,
            KeyCode::ShiftRight => Key::RightShift,
            KeyCode::Semicolon => Key::SemiColon,
            KeyCode::Slash => Key::Slash,
            KeyCode::Tab => Key::Tab,
            _ => Key::Unrecognized,
        }
    }
}
