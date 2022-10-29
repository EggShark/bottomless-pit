use raylib::RaylibHandle;
use utils::{Point, Text, Collide};
use raylib::consts::{KeyboardKey, MouseButton};
use raylib::prelude::RaylibDraw;
use raylib::drawing::RaylibDrawHandle;
use raylib::color::Color;
use raylib::core::math::Vector2;

use crate::ui_utils::Selectable;

#[derive(Debug, PartialEq)]
pub struct KeyChanger {
    pos: Point,
    size: Point,
    selected: bool,
    text: String,
    key_selcted: KeyboardKey,
    awaiting_input: bool,
}

impl Selectable for KeyChanger {
    fn deslect(&mut self) {
        self.selected = false;
    }
    fn get_pos(&self) -> Point {
        self.pos
    }
    fn select(&mut self) {
        self.selected = true;
    }
}

impl KeyChanger {
    pub fn new(pos: Point, size: Point, text: String, key_selcted: KeyboardKey) -> Self {
        Self {
            pos,
            size,
            selected: false,
            text,
            key_selcted,
            awaiting_input: false,
        }
    }

    pub fn draw(&self, d_handle: &mut RaylibDrawHandle) {
        let color = if self.selected {
            Color::RED
        } else {
            Color::WHITE
        };
        let text_pos = Text::center_text_x_pos(&self.text, self.pos.x, self.size.x, 20);

        let key_text = rl_key_str(self.key_selcted);
        let (key_text_x, key_text_y) = Text::center_text(key_text, &self.pos, &self.size, 20, d_handle.get_font_default());

        d_handle.draw_rectangle(self.pos.x, self.pos.y, self.size.x, self.size.y, color);
        d_handle.draw_text(&self.text, text_pos, self.pos.y, 20, Color::BLACK);
        d_handle.draw_text(key_text, key_text_x, key_text_y, 20, Color::BLACK);
    }

    pub fn update(&mut self, rl: &mut RaylibHandle) {
        if Collide::point_in_rect(&self.size, &self.pos, &rl.get_mouse_position()) && rl.is_mouse_button_released(MouseButton::MOUSE_LEFT_BUTTON) {
            self.awaiting_input = true;
        }

        if self.awaiting_input {
            let key = rl.get_key_pressed();
            match key {
                Some(k) => {
                    self.awaiting_input = false;
                    self.key_selcted = k;
                },
                None => {},
            }
        }
    }

    pub fn get_key(&self) -> KeyboardKey {
        self.key_selcted
    }
}

fn rl_key_str<'a>(key: KeyboardKey) -> &'a str {
    match key {
        KeyboardKey::KEY_NULL => "",
        KeyboardKey::KEY_APOSTROPHE => "'",
        KeyboardKey::KEY_COMMA => ",",
        KeyboardKey::KEY_MINUS => "-",
        KeyboardKey::KEY_PERIOD => ".",
        KeyboardKey::KEY_SLASH => "/",
        KeyboardKey::KEY_ZERO => "0",
        KeyboardKey::KEY_ONE => "1",
        KeyboardKey::KEY_TWO => "2",
        KeyboardKey::KEY_THREE => "3",
        KeyboardKey::KEY_FOUR => "4",
        KeyboardKey::KEY_FIVE => "5",
        KeyboardKey::KEY_SIX => "6",
        KeyboardKey::KEY_SEVEN => "7",
        KeyboardKey::KEY_EIGHT => "8",
        KeyboardKey::KEY_NINE => "9",
        KeyboardKey::KEY_SEMICOLON => ";",
        KeyboardKey::KEY_EQUAL => "=",
        KeyboardKey::KEY_A => "A",
        KeyboardKey::KEY_B => "B",
        KeyboardKey::KEY_C => "C",
        KeyboardKey::KEY_D => "D",
        KeyboardKey::KEY_E => "E",
        KeyboardKey::KEY_F => "F",
        KeyboardKey::KEY_G => "G",
        KeyboardKey::KEY_H => "H",
        KeyboardKey::KEY_I => "I",
        KeyboardKey::KEY_J => "J",
        KeyboardKey::KEY_K => "K",
        KeyboardKey::KEY_L => "L",
        KeyboardKey::KEY_M => "M",
        KeyboardKey::KEY_N => "N",
        KeyboardKey::KEY_O => "O",
        KeyboardKey::KEY_P => "P",
        KeyboardKey::KEY_Q => "Q",
        KeyboardKey::KEY_R => "R",
        KeyboardKey::KEY_S => "S",
        KeyboardKey::KEY_T => "T",
        KeyboardKey::KEY_U => "U",
        KeyboardKey::KEY_V => "V",
        KeyboardKey::KEY_W => "W",
        KeyboardKey::KEY_X => "X",
        KeyboardKey::KEY_Y => "Y",
        KeyboardKey::KEY_Z => "Z",
        KeyboardKey::KEY_SPACE => "Space",
        KeyboardKey::KEY_ESCAPE => "Esc",
        KeyboardKey::KEY_ENTER => "Enter",
        KeyboardKey::KEY_TAB => "Tab",
        KeyboardKey::KEY_BACKSPACE => "Backspc",
        KeyboardKey::KEY_INSERT => "Insert",
        KeyboardKey::KEY_DELETE => "Del",
        KeyboardKey::KEY_RIGHT => "Right",
        KeyboardKey::KEY_LEFT => "Left",
        KeyboardKey::KEY_UP => "Up",
        KeyboardKey::KEY_DOWN => "Down",
        KeyboardKey::KEY_PAGE_DOWN => "Pg Down",
        KeyboardKey::KEY_PAGE_UP => "Pg Up",
        KeyboardKey::KEY_HOME => "Home",
        KeyboardKey::KEY_END => "End",
        KeyboardKey::KEY_CAPS_LOCK => "Caps Lock",
        KeyboardKey::KEY_SCROLL_LOCK => "Scrl Lock",
        KeyboardKey::KEY_NUM_LOCK => "Num Lock",
        KeyboardKey::KEY_PRINT_SCREEN => "Prnt Scrn", // should be uncreachable or just wack tbh
        KeyboardKey::KEY_PAUSE => "Pause",
        KeyboardKey::KEY_F1 => "F1",
        KeyboardKey::KEY_F2 => "F2",
        KeyboardKey::KEY_F3 => "F3",
        KeyboardKey::KEY_F4 => "F4",
        KeyboardKey::KEY_F5 => "F5",
        KeyboardKey::KEY_F6 => "F6",
        KeyboardKey::KEY_F7 => "F7",
        KeyboardKey::KEY_F8 => "F8",
        KeyboardKey::KEY_F9 => "F9",
        KeyboardKey::KEY_F10 => "F10",
        KeyboardKey::KEY_F11 => "F11",
        KeyboardKey::KEY_F12 => "F12",
        KeyboardKey::KEY_LEFT_SHIFT => "Shift",
        KeyboardKey::KEY_LEFT_CONTROL => "Ctrl",
        KeyboardKey::KEY_LEFT_ALT => "Alt",
        KeyboardKey::KEY_LEFT_SUPER => "Super",
        KeyboardKey::KEY_RIGHT_SHIFT => "Shift",
        KeyboardKey::KEY_RIGHT_CONTROL => "Ctrl",
        KeyboardKey::KEY_RIGHT_ALT => "Alt",
        KeyboardKey::KEY_RIGHT_SUPER => "Super",
        KeyboardKey::KEY_KB_MENU => "Menu",
        KeyboardKey::KEY_LEFT_BRACKET => "[",
        KeyboardKey::KEY_BACKSLASH => "\\",
        KeyboardKey::KEY_RIGHT_BRACKET => "]",
        KeyboardKey::KEY_GRAVE => "Guh??",
        KeyboardKey::KEY_KP_0 => "0",
        KeyboardKey::KEY_KP_1 => "1",
        KeyboardKey::KEY_KP_2 => "2",
        KeyboardKey::KEY_KP_3 => "3",
        KeyboardKey::KEY_KP_4 => "4",
        KeyboardKey::KEY_KP_5 => "5",
        KeyboardKey::KEY_KP_6 => "6",
        KeyboardKey::KEY_KP_7 => "7",
        KeyboardKey::KEY_KP_8 => "8",
        KeyboardKey::KEY_KP_9 => "9",
        KeyboardKey::KEY_KP_DECIMAL => ".",
        KeyboardKey::KEY_KP_DIVIDE => "/",
        KeyboardKey::KEY_KP_MULTIPLY => "*",
        KeyboardKey::KEY_KP_SUBTRACT => "-",
        KeyboardKey::KEY_KP_ADD => "+",
        KeyboardKey::KEY_KP_ENTER => "Enter",
        KeyboardKey::KEY_KP_EQUAL => "=",
        KeyboardKey::KEY_BACK => "Back",
        KeyboardKey::KEY_VOLUME_DOWN => "Vol-",
        KeyboardKey::KEY_VOLUME_UP => "Vol+",
    }
}