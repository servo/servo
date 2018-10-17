/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use keyboard_types::{Key, KeyboardEvent};

// spec: https://w3c.github.io/webdriver/#keyboard-actions
// normalised (sic) as in british spelling
fn get_normalised_key_value(key: char) -> Key {
    match key {
        '\u{E000}' => Key::Unidentified,
        '\u{E001}' => Key::Cancel,
        '\u{E002}' => Key::Help,
        '\u{E003}' => Key::Backspace,
        '\u{E004}' => Key::Tab,
        '\u{E005}' => Key::Clear,
        // FIXME(pyfisch): spec says "Return"
        '\u{E006}' => Key::Enter,
        '\u{E007}' => Key::Enter,
        '\u{E008}' => Key::Shift,
        '\u{E009}' => Key::Control,
        '\u{E00A}' => Key::Alt,
        '\u{E00B}' => Key::Pause,
        '\u{E00C}' => Key::Escape,
        '\u{E00D}' => Key::Character(" ".to_string()),
        '\u{E00E}' => Key::PageUp,
        '\u{E00F}' => Key::PageDown,
        '\u{E010}' => Key::End,
        '\u{E011}' => Key::Home,
        '\u{E012}' => Key::ArrowLeft,
        '\u{E013}' => Key::ArrowUp,
        '\u{E014}' => Key::ArrowRight,
        '\u{E015}' => Key::ArrowDown,
        '\u{E016}' => Key::Insert,
        '\u{E017}' => Key::Delete,
        '\u{E018}' => Key::Character(";".to_string()),
        '\u{E019}' => Key::Character("=".to_string()),
        '\u{E01A}' => Key::Character("0".to_string()),
        '\u{E01B}' => Key::Character("1".to_string()),
        '\u{E01C}' => Key::Character("2".to_string()),
        '\u{E01D}' => Key::Character("3".to_string()),
        '\u{E01E}' => Key::Character("4".to_string()),
        '\u{E01F}' => Key::Character("5".to_string()),
        '\u{E020}' => Key::Character("6".to_string()),
        '\u{E021}' => Key::Character("7".to_string()),
        '\u{E022}' => Key::Character("8".to_string()),
        '\u{E023}' => Key::Character("9".to_string()),
        '\u{E024}' => Key::Character("*".to_string()),
        '\u{E025}' => Key::Character("+".to_string()),
        '\u{E026}' => Key::Character(",".to_string()),
        '\u{E027}' => Key::Character("-".to_string()),
        '\u{E028}' => Key::Character(".".to_string()),
        '\u{E029}' => Key::Character("/".to_string()),
        '\u{E031}' => Key::F1,
        '\u{E032}' => Key::F2,
        '\u{E033}' => Key::F3,
        '\u{E034}' => Key::F4,
        '\u{E035}' => Key::F5,
        '\u{E036}' => Key::F6,
        '\u{E037}' => Key::F7,
        '\u{E038}' => Key::F8,
        '\u{E039}' => Key::F9,
        '\u{E03A}' => Key::F10,
        '\u{E03B}' => Key::F11,
        '\u{E03C}' => Key::F12,
        '\u{E03D}' => Key::Meta,
        '\u{E040}' => Key::ZenkakuHankaku,
        '\u{E050}' => Key::Shift,
        '\u{E051}' => Key::Control,
        '\u{E052}' => Key::Alt,
        '\u{E053}' => Key::Meta,
        '\u{E054}' => Key::PageUp,
        '\u{E055}' => Key::PageDown,
        '\u{E056}' => Key::End,
        '\u{E057}' => Key::Home,
        '\u{E058}' => Key::ArrowLeft,
        '\u{E059}' => Key::ArrowUp,
        '\u{E05A}' => Key::ArrowRight,
        '\u{E05B}' => Key::ArrowDown,
        '\u{E05C}' => Key::Insert,
        '\u{E05D}' => Key::Delete,
        _ => Key::Character(key.to_string()),
    }
}

pub fn keycodes_to_keys(key_codes: &str) -> Vec<KeyboardEvent> {
    let mut rv = vec![];

    for char_code in key_codes.chars() {
        // TODO(pyfisch): compute code, location, modifiers according to spec
        let key = get_normalised_key_value(char_code);
        let mut event = KeyboardEvent {
            state: ::keyboard_types::KeyState::Down,
            key,
            code: ::keyboard_types::Code::Unidentified,
            location: ::keyboard_types::Location::Standard,
            modifiers: ::keyboard_types::Modifiers::empty(),
            repeat: false,
            is_composing: false,
        };
        rv.push(event.clone());
        event.state = ::keyboard_types::KeyState::Up;
        rv.push(event);
    }
    rv
}
