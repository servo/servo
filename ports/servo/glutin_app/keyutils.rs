/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use msg::constellation_msg::{self, Key, KeyModifiers};
use winit::{self, VirtualKeyCode};

bitflags! {
    pub struct GlutinKeyModifiers: u8 {
        const LEFT_CONTROL = 1;
        const RIGHT_CONTROL = 2;
        const LEFT_SHIFT = 4;
        const RIGHT_SHIFT = 8;
        const LEFT_ALT = 16;
        const RIGHT_ALT = 32;
        const LEFT_SUPER = 64;
        const RIGHT_SUPER = 128;
    }
}

// Some shortcuts use Cmd on Mac and Control on other systems.
#[cfg(target_os = "macos")]
pub const CMD_OR_CONTROL: KeyModifiers = KeyModifiers::SUPER;
#[cfg(not(target_os = "macos"))]
pub const CMD_OR_CONTROL: KeyModifiers = KeyModifiers::CONTROL;

// Some shortcuts use Cmd on Mac and Alt on other systems.
#[cfg(target_os = "macos")]
pub const CMD_OR_ALT: KeyModifiers = KeyModifiers::SUPER;
#[cfg(not(target_os = "macos"))]
pub const CMD_OR_ALT: KeyModifiers = KeyModifiers::ALT;

pub fn char_to_script_key(c: char) -> Option<constellation_msg::Key> {
    match c {
        ' ' => Some(Key::Space),
        '"' => Some(Key::Apostrophe),
        '\'' => Some(Key::Apostrophe),
        '<' => Some(Key::Comma),
        ',' => Some(Key::Comma),
        '_' => Some(Key::Minus),
        '-' => Some(Key::Minus),
        '>' => Some(Key::Period),
        '.' => Some(Key::Period),
        '?' => Some(Key::Slash),
        '/' => Some(Key::Slash),
        '~' => Some(Key::GraveAccent),
        '`' => Some(Key::GraveAccent),
        ')' => Some(Key::Num0),
        '0' => Some(Key::Num0),
        '!' => Some(Key::Num1),
        '1' => Some(Key::Num1),
        '@' => Some(Key::Num2),
        '2' => Some(Key::Num2),
        '#' => Some(Key::Num3),
        '3' => Some(Key::Num3),
        '$' => Some(Key::Num4),
        '4' => Some(Key::Num4),
        '%' => Some(Key::Num5),
        '5' => Some(Key::Num5),
        '^' => Some(Key::Num6),
        '6' => Some(Key::Num6),
        '&' => Some(Key::Num7),
        '7' => Some(Key::Num7),
        '*' => Some(Key::Num8),
        '8' => Some(Key::Num8),
        '(' => Some(Key::Num9),
        '9' => Some(Key::Num9),
        ':' => Some(Key::Semicolon),
        ';' => Some(Key::Semicolon),
        '+' => Some(Key::Equal),
        '=' => Some(Key::Equal),
        'A' => Some(Key::A),
        'a' => Some(Key::A),
        'B' => Some(Key::B),
        'b' => Some(Key::B),
        'C' => Some(Key::C),
        'c' => Some(Key::C),
        'D' => Some(Key::D),
        'd' => Some(Key::D),
        'E' => Some(Key::E),
        'e' => Some(Key::E),
        'F' => Some(Key::F),
        'f' => Some(Key::F),
        'G' => Some(Key::G),
        'g' => Some(Key::G),
        'H' => Some(Key::H),
        'h' => Some(Key::H),
        'I' => Some(Key::I),
        'i' => Some(Key::I),
        'J' => Some(Key::J),
        'j' => Some(Key::J),
        'K' => Some(Key::K),
        'k' => Some(Key::K),
        'L' => Some(Key::L),
        'l' => Some(Key::L),
        'M' => Some(Key::M),
        'm' => Some(Key::M),
        'N' => Some(Key::N),
        'n' => Some(Key::N),
        'O' => Some(Key::O),
        'o' => Some(Key::O),
        'P' => Some(Key::P),
        'p' => Some(Key::P),
        'Q' => Some(Key::Q),
        'q' => Some(Key::Q),
        'R' => Some(Key::R),
        'r' => Some(Key::R),
        'S' => Some(Key::S),
        's' => Some(Key::S),
        'T' => Some(Key::T),
        't' => Some(Key::T),
        'U' => Some(Key::U),
        'u' => Some(Key::U),
        'V' => Some(Key::V),
        'v' => Some(Key::V),
        'W' => Some(Key::W),
        'w' => Some(Key::W),
        'X' => Some(Key::X),
        'x' => Some(Key::X),
        'Y' => Some(Key::Y),
        'y' => Some(Key::Y),
        'Z' => Some(Key::Z),
        'z' => Some(Key::Z),
        '{' => Some(Key::LeftBracket),
        '[' => Some(Key::LeftBracket),
        '|' => Some(Key::Backslash),
        '\\' => Some(Key::Backslash),
        '}' => Some(Key::RightBracket),
        ']' => Some(Key::RightBracket),
        _ => None
    }
}

pub fn glutin_key_to_script_key(key: winit::VirtualKeyCode) -> Result<constellation_msg::Key, ()> {
    // TODO(negge): add more key mappings
    match key {
        VirtualKeyCode::A => Ok(Key::A),
        VirtualKeyCode::B => Ok(Key::B),
        VirtualKeyCode::C => Ok(Key::C),
        VirtualKeyCode::D => Ok(Key::D),
        VirtualKeyCode::E => Ok(Key::E),
        VirtualKeyCode::F => Ok(Key::F),
        VirtualKeyCode::G => Ok(Key::G),
        VirtualKeyCode::H => Ok(Key::H),
        VirtualKeyCode::I => Ok(Key::I),
        VirtualKeyCode::J => Ok(Key::J),
        VirtualKeyCode::K => Ok(Key::K),
        VirtualKeyCode::L => Ok(Key::L),
        VirtualKeyCode::M => Ok(Key::M),
        VirtualKeyCode::N => Ok(Key::N),
        VirtualKeyCode::O => Ok(Key::O),
        VirtualKeyCode::P => Ok(Key::P),
        VirtualKeyCode::Q => Ok(Key::Q),
        VirtualKeyCode::R => Ok(Key::R),
        VirtualKeyCode::S => Ok(Key::S),
        VirtualKeyCode::T => Ok(Key::T),
        VirtualKeyCode::U => Ok(Key::U),
        VirtualKeyCode::V => Ok(Key::V),
        VirtualKeyCode::W => Ok(Key::W),
        VirtualKeyCode::X => Ok(Key::X),
        VirtualKeyCode::Y => Ok(Key::Y),
        VirtualKeyCode::Z => Ok(Key::Z),

        VirtualKeyCode::Numpad0 => Ok(Key::Kp0),
        VirtualKeyCode::Numpad1 => Ok(Key::Kp1),
        VirtualKeyCode::Numpad2 => Ok(Key::Kp2),
        VirtualKeyCode::Numpad3 => Ok(Key::Kp3),
        VirtualKeyCode::Numpad4 => Ok(Key::Kp4),
        VirtualKeyCode::Numpad5 => Ok(Key::Kp5),
        VirtualKeyCode::Numpad6 => Ok(Key::Kp6),
        VirtualKeyCode::Numpad7 => Ok(Key::Kp7),
        VirtualKeyCode::Numpad8 => Ok(Key::Kp8),
        VirtualKeyCode::Numpad9 => Ok(Key::Kp9),

        VirtualKeyCode::Key0 => Ok(Key::Num0),
        VirtualKeyCode::Key1 => Ok(Key::Num1),
        VirtualKeyCode::Key2 => Ok(Key::Num2),
        VirtualKeyCode::Key3 => Ok(Key::Num3),
        VirtualKeyCode::Key4 => Ok(Key::Num4),
        VirtualKeyCode::Key5 => Ok(Key::Num5),
        VirtualKeyCode::Key6 => Ok(Key::Num6),
        VirtualKeyCode::Key7 => Ok(Key::Num7),
        VirtualKeyCode::Key8 => Ok(Key::Num8),
        VirtualKeyCode::Key9 => Ok(Key::Num9),

        VirtualKeyCode::Return => Ok(Key::Enter),
        VirtualKeyCode::Space => Ok(Key::Space),
        VirtualKeyCode::Escape => Ok(Key::Escape),
        VirtualKeyCode::Equals => Ok(Key::Equal),
        VirtualKeyCode::Minus => Ok(Key::Minus),
        VirtualKeyCode::Back => Ok(Key::Backspace),
        VirtualKeyCode::PageDown => Ok(Key::PageDown),
        VirtualKeyCode::PageUp => Ok(Key::PageUp),

        VirtualKeyCode::Insert => Ok(Key::Insert),
        VirtualKeyCode::Home => Ok(Key::Home),
        VirtualKeyCode::Delete => Ok(Key::Delete),
        VirtualKeyCode::End => Ok(Key::End),

        VirtualKeyCode::Left => Ok(Key::Left),
        VirtualKeyCode::Up => Ok(Key::Up),
        VirtualKeyCode::Right => Ok(Key::Right),
        VirtualKeyCode::Down => Ok(Key::Down),

        VirtualKeyCode::LShift => Ok(Key::LeftShift),
        VirtualKeyCode::LControl => Ok(Key::LeftControl),
        VirtualKeyCode::LAlt => Ok(Key::LeftAlt),
        VirtualKeyCode::LWin => Ok(Key::LeftSuper),
        VirtualKeyCode::RShift => Ok(Key::RightShift),
        VirtualKeyCode::RControl => Ok(Key::RightControl),
        VirtualKeyCode::RAlt => Ok(Key::RightAlt),
        VirtualKeyCode::RWin => Ok(Key::RightSuper),

        VirtualKeyCode::Apostrophe => Ok(Key::Apostrophe),
        VirtualKeyCode::Backslash => Ok(Key::Backslash),
        VirtualKeyCode::Comma => Ok(Key::Comma),
        VirtualKeyCode::Grave => Ok(Key::GraveAccent),
        VirtualKeyCode::LBracket => Ok(Key::LeftBracket),
        VirtualKeyCode::Period => Ok(Key::Period),
        VirtualKeyCode::RBracket => Ok(Key::RightBracket),
        VirtualKeyCode::Semicolon => Ok(Key::Semicolon),
        VirtualKeyCode::Slash => Ok(Key::Slash),
        VirtualKeyCode::Tab => Ok(Key::Tab),
        VirtualKeyCode::Subtract => Ok(Key::Minus),

        VirtualKeyCode::F1 => Ok(Key::F1),
        VirtualKeyCode::F2 => Ok(Key::F2),
        VirtualKeyCode::F3 => Ok(Key::F3),
        VirtualKeyCode::F4 => Ok(Key::F4),
        VirtualKeyCode::F5 => Ok(Key::F5),
        VirtualKeyCode::F6 => Ok(Key::F6),
        VirtualKeyCode::F7 => Ok(Key::F7),
        VirtualKeyCode::F8 => Ok(Key::F8),
        VirtualKeyCode::F9 => Ok(Key::F9),
        VirtualKeyCode::F10 => Ok(Key::F10),
        VirtualKeyCode::F11 => Ok(Key::F11),
        VirtualKeyCode::F12 => Ok(Key::F12),

        VirtualKeyCode::NavigateBackward => Ok(Key::NavigateBackward),
        VirtualKeyCode::NavigateForward => Ok(Key::NavigateForward),
        _ => Err(()),
    }
}

pub fn glutin_mods_to_script_mods(modifiers: GlutinKeyModifiers) -> constellation_msg::KeyModifiers {
    let mut result = constellation_msg::KeyModifiers::empty();
    if modifiers.intersects(GlutinKeyModifiers::LEFT_SHIFT | GlutinKeyModifiers::RIGHT_SHIFT) {
        result.insert(KeyModifiers::SHIFT);
    }
    if modifiers.intersects(GlutinKeyModifiers::LEFT_CONTROL | GlutinKeyModifiers::RIGHT_CONTROL) {
        result.insert(KeyModifiers::CONTROL);
    }
    if modifiers.intersects(GlutinKeyModifiers::LEFT_ALT | GlutinKeyModifiers::RIGHT_ALT) {
        result.insert(KeyModifiers::ALT);
    }
    if modifiers.intersects(GlutinKeyModifiers::LEFT_SUPER | GlutinKeyModifiers::RIGHT_SUPER) {
        result.insert(KeyModifiers::SUPER);
    }
    result
}

pub fn is_printable(key_code: VirtualKeyCode) -> bool {
    use winit::VirtualKeyCode::*;
    match key_code {
        Escape |
        F1 |
        F2 |
        F3 |
        F4 |
        F5 |
        F6 |
        F7 |
        F8 |
        F9 |
        F10 |
        F11 |
        F12 |
        F13 |
        F14 |
        F15 |
        Snapshot |
        Scroll |
        Pause |
        Insert |
        Home |
        Delete |
        End |
        PageDown |
        PageUp |
        Left |
        Up |
        Right |
        Down |
        Back |
        LAlt |
        LControl |
        LMenu |
        LShift |
        LWin |
        Mail |
        MediaSelect |
        MediaStop |
        Mute |
        MyComputer |
        NavigateForward |
        NavigateBackward |
        NextTrack |
        NoConvert |
        PlayPause |
        Power |
        PrevTrack |
        RAlt |
        RControl |
        RMenu |
        RShift |
        RWin |
        Sleep |
        Stop |
        VolumeDown |
        VolumeUp |
        Wake |
        WebBack |
        WebFavorites |
        WebForward |
        WebHome |
        WebRefresh |
        WebSearch |
        WebStop => false,
        _ => true,
    }
}

/// Detect if given char is default ignorable in unicode
/// http://www.unicode.org/L2/L2002/02368-default-ignorable.pdf
pub fn is_identifier_ignorable(ch: &char) -> bool {
    match *ch {
        '\u{0000}'...'\u{0008}' | '\u{000E}'...'\u{001F}' |
            '\u{007F}'...'\u{0084}' | '\u{0086}'...'\u{009F}' |
            '\u{06DD}' | '\u{070F}' |
            '\u{180B}'...'\u{180D}' | '\u{180E}' |
            '\u{200C}'...'\u{200F}' |
            '\u{202A}'...'\u{202E}' | '\u{2060}'...'\u{2063}' |
            '\u{2064}'...'\u{2069}' | '\u{206A}'...'\u{206F}' |
            '\u{FE00}'...'\u{FE0F}' | '\u{FEFF}' |
            '\u{FFF0}'...'\u{FFF8}' | '\u{FFF9}'...'\u{FFFB}' |
            '\u{1D173}'...'\u{1D17A}' | '\u{E0000}' |
            '\u{E0001}' |
            '\u{E0002}'...'\u{E001F}' | '\u{E0020}'...'\u{E007F}' |
            '\u{E0080}'...'\u{E0FFF}' => true,
        _ => false
    }
}
