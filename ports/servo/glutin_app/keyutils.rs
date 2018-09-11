/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use servo::msg::constellation_msg::{self, Key, KeyModifiers};
use winit::VirtualKeyCode;

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

pub fn winit_key_to_script_key(key: VirtualKeyCode) -> Result<constellation_msg::Key, ()> {
    use winit::VirtualKeyCode::*;
    // TODO(negge): add more key mappings
    Ok(match key {
        A => Key::A,
        B => Key::B,
        C => Key::C,
        D => Key::D,
        E => Key::E,
        F => Key::F,
        G => Key::G,
        H => Key::H,
        I => Key::I,
        J => Key::J,
        K => Key::K,
        L => Key::L,
        M => Key::M,
        N => Key::N,
        O => Key::O,
        P => Key::P,
        Q => Key::Q,
        R => Key::R,
        S => Key::S,
        T => Key::T,
        U => Key::U,
        V => Key::V,
        W => Key::W,
        X => Key::X,
        Y => Key::Y,
        Z => Key::Z,

        Numpad0 => Key::Kp0,
        Numpad1 => Key::Kp1,
        Numpad2 => Key::Kp2,
        Numpad3 => Key::Kp3,
        Numpad4 => Key::Kp4,
        Numpad5 => Key::Kp5,
        Numpad6 => Key::Kp6,
        Numpad7 => Key::Kp7,
        Numpad8 => Key::Kp8,
        Numpad9 => Key::Kp9,

        Key0 => Key::Num0,
        Key1 => Key::Num1,
        Key2 => Key::Num2,
        Key3 => Key::Num3,
        Key4 => Key::Num4,
        Key5 => Key::Num5,
        Key6 => Key::Num6,
        Key7 => Key::Num7,
        Key8 => Key::Num8,
        Key9 => Key::Num9,

        Return => Key::Enter,
        Space => Key::Space,
        Escape => Key::Escape,
        Equals => Key::Equal,
        Minus => Key::Minus,
        Back => Key::Backspace,
        PageDown => Key::PageDown,
        PageUp => Key::PageUp,

        Insert => Key::Insert,
        Home => Key::Home,
        Delete => Key::Delete,
        End => Key::End,

        Left => Key::Left,
        Up => Key::Up,
        Right => Key::Right,
        Down => Key::Down,

        LShift => Key::LeftShift,
        LControl => Key::LeftControl,
        LAlt => Key::LeftAlt,
        LWin => Key::LeftSuper,
        RShift => Key::RightShift,
        RControl => Key::RightControl,
        RAlt => Key::RightAlt,
        RWin => Key::RightSuper,

        Apostrophe => Key::Apostrophe,
        Backslash => Key::Backslash,
        Comma => Key::Comma,
        Grave => Key::GraveAccent,
        LBracket => Key::LeftBracket,
        Period => Key::Period,
        RBracket => Key::RightBracket,
        Semicolon => Key::Semicolon,
        Slash => Key::Slash,
        Tab => Key::Tab,
        Subtract => Key::Minus,

        F1 => Key::F1,
        F2 => Key::F2,
        F3 => Key::F3,
        F4 => Key::F4,
        F5 => Key::F5,
        F6 => Key::F6,
        F7 => Key::F7,
        F8 => Key::F8,
        F9 => Key::F9,
        F10 => Key::F10,
        F11 => Key::F11,
        F12 => Key::F12,

        NavigateBackward => Key::NavigateBackward,
        NavigateForward => Key::NavigateForward,
        _ => return Err(()),
    })
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
        Return |
        LAlt |
        LControl |
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

