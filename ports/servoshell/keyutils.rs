/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use keyboard_types::{Code, Key, KeyState, KeyboardEvent, Location, Modifiers};
use log::info;
use winit::event::{ElementState, KeyEvent};
use winit::keyboard::{Key as LogicalKey, KeyCode, ModifiersState, NamedKey, PhysicalKey};

// Some shortcuts use Cmd on Mac and Control on other systems.
#[cfg(target_os = "macos")]
pub const CMD_OR_CONTROL: Modifiers = Modifiers::META;
#[cfg(not(target_os = "macos"))]
pub const CMD_OR_CONTROL: Modifiers = Modifiers::CONTROL;

// Some shortcuts use Cmd on Mac and Alt on other systems.
#[cfg(target_os = "macos")]
pub const CMD_OR_ALT: Modifiers = Modifiers::META;
#[cfg(not(target_os = "macos"))]
pub const CMD_OR_ALT: Modifiers = Modifiers::ALT;

fn get_servo_key_from_winit_key(key: &LogicalKey) -> Key {
    // TODO: figure out how to map NavigateForward, NavigateBackward
    // TODO: map the remaining keys if possible
    match key {
        // printable: Key1 to Key0
        // printable: A to Z
        LogicalKey::Named(NamedKey::Escape) => Key::Escape,
        LogicalKey::Named(NamedKey::F1) => Key::F1,
        LogicalKey::Named(NamedKey::F2) => Key::F2,
        LogicalKey::Named(NamedKey::F3) => Key::F3,
        LogicalKey::Named(NamedKey::F4) => Key::F4,
        LogicalKey::Named(NamedKey::F5) => Key::F5,
        LogicalKey::Named(NamedKey::F6) => Key::F6,
        LogicalKey::Named(NamedKey::F7) => Key::F7,
        LogicalKey::Named(NamedKey::F8) => Key::F8,
        LogicalKey::Named(NamedKey::F9) => Key::F9,
        LogicalKey::Named(NamedKey::F10) => Key::F10,
        LogicalKey::Named(NamedKey::F11) => Key::F11,
        LogicalKey::Named(NamedKey::F12) => Key::F12,
        // F13 to F15 are not mapped
        LogicalKey::Named(NamedKey::PrintScreen) => Key::PrintScreen,
        // Scroll not mapped
        LogicalKey::Named(NamedKey::Pause) => Key::Pause,
        LogicalKey::Named(NamedKey::Insert) => Key::Insert,
        LogicalKey::Named(NamedKey::Home) => Key::Home,
        LogicalKey::Named(NamedKey::Delete) => Key::Delete,
        LogicalKey::Named(NamedKey::End) => Key::End,
        LogicalKey::Named(NamedKey::PageDown) => Key::PageDown,
        LogicalKey::Named(NamedKey::PageUp) => Key::PageUp,
        LogicalKey::Named(NamedKey::ArrowLeft) => Key::ArrowLeft,
        LogicalKey::Named(NamedKey::ArrowUp) => Key::ArrowUp,
        LogicalKey::Named(NamedKey::ArrowRight) => Key::ArrowRight,
        LogicalKey::Named(NamedKey::ArrowDown) => Key::ArrowDown,
        LogicalKey::Named(NamedKey::Backspace) => Key::Backspace,
        LogicalKey::Named(NamedKey::Enter) => Key::Enter,
        // printable: Space
        LogicalKey::Named(NamedKey::Compose) => Key::Compose,
        // Caret not mapped
        LogicalKey::Named(NamedKey::NumLock) => Key::NumLock,
        // printable: Numpad0 to Numpad9
        // AbntC1 and AbntC2 not mapped
        // printable: Add, Apostrophe,
        // Apps, At, Ax not mapped
        // printable: Backslash,
        LogicalKey::Named(NamedKey::LaunchApplication2) => Key::LaunchApplication2,
        LogicalKey::Named(NamedKey::CapsLock) => Key::CapsLock,
        // printable: Colon, Comma,
        LogicalKey::Named(NamedKey::Convert) => Key::Convert,
        // not mapped: Decimal,
        // printable: Divide, Equals, Grave,
        LogicalKey::Named(NamedKey::KanaMode) => Key::KanaMode,
        LogicalKey::Named(NamedKey::KanjiMode) => Key::KanjiMode,
        LogicalKey::Named(NamedKey::Alt) => Key::Alt,
        // printable: LBracket,
        LogicalKey::Named(NamedKey::Control) => Key::Control,
        LogicalKey::Named(NamedKey::Shift) => Key::Shift,
        LogicalKey::Named(NamedKey::Meta) => Key::Meta,
        LogicalKey::Named(NamedKey::LaunchMail) => Key::LaunchMail,
        // not mapped: MediaSelect,
        LogicalKey::Named(NamedKey::MediaStop) => Key::MediaStop,
        // printable: Minus, Multiply,
        LogicalKey::Named(NamedKey::AudioVolumeMute) => Key::AudioVolumeMute,
        LogicalKey::Named(NamedKey::LaunchApplication1) => Key::LaunchApplication1,
        // not mapped: NavigateForward, NavigateBackward
        LogicalKey::Named(NamedKey::MediaTrackNext) => Key::MediaTrackNext,
        LogicalKey::Named(NamedKey::NonConvert) => Key::NonConvert,
        // printable: NumpadComma, NumpadEnter, NumpadEquals,
        // not mapped: OEM102,
        // printable: Period,
        LogicalKey::Named(NamedKey::MediaPlayPause) => Key::MediaPlayPause,
        LogicalKey::Named(NamedKey::Power) => Key::Power,
        LogicalKey::Named(NamedKey::MediaTrackPrevious) => Key::MediaTrackPrevious,
        // printable RBracket
        // printable Semicolon, Slash
        LogicalKey::Named(NamedKey::Standby) => Key::Standby,
        // not mapped: Stop,
        // printable Subtract,
        // not mapped: Sysrq,
        LogicalKey::Named(NamedKey::Tab) => Key::Tab,
        // printable: Underline,
        // not mapped: Unlabeled,
        LogicalKey::Named(NamedKey::AudioVolumeDown) => Key::AudioVolumeDown,
        LogicalKey::Named(NamedKey::AudioVolumeUp) => Key::AudioVolumeUp,
        LogicalKey::Named(NamedKey::WakeUp) => Key::WakeUp,
        LogicalKey::Named(NamedKey::BrowserBack) => Key::BrowserBack,
        LogicalKey::Named(NamedKey::BrowserFavorites) => Key::BrowserFavorites,
        LogicalKey::Named(NamedKey::BrowserForward) => Key::BrowserForward,
        LogicalKey::Named(NamedKey::BrowserHome) => Key::BrowserHome,
        LogicalKey::Named(NamedKey::BrowserRefresh) => Key::BrowserRefresh,
        LogicalKey::Named(NamedKey::BrowserSearch) => Key::BrowserSearch,
        LogicalKey::Named(NamedKey::BrowserStop) => Key::BrowserStop,
        // printable Yen,
        LogicalKey::Named(NamedKey::Copy) => Key::Copy,
        LogicalKey::Named(NamedKey::Paste) => Key::Paste,
        LogicalKey::Named(NamedKey::Cut) => Key::Cut,
        _ => Key::Unidentified,
    }
}

fn get_servo_location_from_physical_key(physical_key: PhysicalKey) -> Location {
    let key_code = if let PhysicalKey::Code(key_code) = physical_key {
        key_code
    } else {
        return Location::Standard;
    };

    // TODO: add more numpad keys
    match key_code {
        KeyCode::ShiftLeft | KeyCode::ControlLeft | KeyCode::AltLeft | KeyCode::SuperLeft => {
            Location::Left
        },
        KeyCode::ShiftRight | KeyCode::ControlRight | KeyCode::AltRight | KeyCode::SuperRight => {
            Location::Right
        },
        KeyCode::Numpad0 |
        KeyCode::Numpad1 |
        KeyCode::Numpad2 |
        KeyCode::Numpad3 |
        KeyCode::Numpad4 |
        KeyCode::Numpad5 |
        KeyCode::Numpad6 |
        KeyCode::Numpad7 |
        KeyCode::Numpad8 |
        KeyCode::Numpad9 => Location::Numpad,
        KeyCode::NumpadComma | KeyCode::NumpadEnter | KeyCode::NumpadEqual => Location::Numpad,
        _ => Location::Standard,
    }
}

fn get_servo_code_from_physical_key(physical_key: PhysicalKey) -> Code {
    let key_code = if let PhysicalKey::Code(key_code) = physical_key {
        key_code
    } else {
        return Code::Unidentified;
    };

    // TODO: Map more codes
    match key_code {
        KeyCode::Escape => Code::Escape,
        KeyCode::Digit1 => Code::Digit1,
        KeyCode::Digit2 => Code::Digit2,
        KeyCode::Digit3 => Code::Digit3,
        KeyCode::Digit4 => Code::Digit4,
        KeyCode::Digit5 => Code::Digit5,
        KeyCode::Digit6 => Code::Digit6,
        KeyCode::Digit7 => Code::Digit7,
        KeyCode::Digit8 => Code::Digit8,
        KeyCode::Digit9 => Code::Digit9,
        KeyCode::Digit0 => Code::Digit0,

        KeyCode::Backspace => Code::Backspace,
        KeyCode::Tab => Code::Tab,
        KeyCode::KeyQ => Code::KeyQ,
        KeyCode::KeyW => Code::KeyW,
        KeyCode::KeyE => Code::KeyE,
        KeyCode::KeyR => Code::KeyR,
        KeyCode::KeyT => Code::KeyT,
        KeyCode::KeyY => Code::KeyY,
        KeyCode::KeyU => Code::KeyU,
        KeyCode::KeyI => Code::KeyI,
        KeyCode::KeyO => Code::KeyO,
        KeyCode::KeyP => Code::KeyP,
        KeyCode::BracketLeft => Code::BracketLeft,
        KeyCode::BracketRight => Code::BracketRight,
        KeyCode::Enter => Code::Enter,

        KeyCode::KeyA => Code::KeyA,
        KeyCode::KeyS => Code::KeyS,
        KeyCode::KeyD => Code::KeyD,
        KeyCode::KeyF => Code::KeyF,
        KeyCode::KeyG => Code::KeyG,
        KeyCode::KeyH => Code::KeyH,
        KeyCode::KeyJ => Code::KeyJ,
        KeyCode::KeyK => Code::KeyK,
        KeyCode::KeyL => Code::KeyL,
        KeyCode::Semicolon => Code::Semicolon,
        KeyCode::Quote => Code::Quote,

        KeyCode::ShiftLeft => Code::ShiftLeft,
        KeyCode::Backslash => Code::Backslash,
        KeyCode::KeyZ => Code::KeyZ,
        KeyCode::KeyX => Code::KeyX,
        KeyCode::KeyC => Code::KeyC,
        KeyCode::KeyV => Code::KeyV,
        KeyCode::KeyB => Code::KeyB,
        KeyCode::KeyN => Code::KeyN,
        KeyCode::KeyM => Code::KeyM,
        KeyCode::Comma => Code::Comma,
        KeyCode::Period => Code::Period,
        KeyCode::Slash => Code::Slash,
        KeyCode::ShiftRight => Code::ShiftRight,

        KeyCode::Space => Code::Space,

        KeyCode::F1 => Code::F1,
        KeyCode::F2 => Code::F2,
        KeyCode::F3 => Code::F3,
        KeyCode::F4 => Code::F4,
        KeyCode::F5 => Code::F5,
        KeyCode::F6 => Code::F6,
        KeyCode::F7 => Code::F7,
        KeyCode::F8 => Code::F8,
        KeyCode::F9 => Code::F9,
        KeyCode::F10 => Code::F10,

        KeyCode::F11 => Code::F11,
        KeyCode::F12 => Code::F12,

        KeyCode::ArrowUp => Code::ArrowUp,
        KeyCode::PageUp => Code::PageUp,
        KeyCode::ArrowLeft => Code::ArrowLeft,
        KeyCode::ArrowRight => Code::ArrowRight,

        KeyCode::Home => Code::Home,
        KeyCode::End => Code::End,
        KeyCode::ArrowDown => Code::ArrowDown,
        KeyCode::PageDown => Code::PageDown,
        KeyCode::Insert => Code::Insert,
        KeyCode::Delete => Code::Delete,

        _ => Code::Unidentified,
    }
}

fn get_modifiers(mods: ModifiersState) -> Modifiers {
    let mut modifiers = Modifiers::empty();
    modifiers.set(Modifiers::CONTROL, mods.control_key());
    modifiers.set(Modifiers::SHIFT, mods.shift_key());
    modifiers.set(Modifiers::ALT, mods.alt_key());
    modifiers.set(Modifiers::META, mods.super_key());
    modifiers
}

pub fn keyboard_event_from_winit(input: &KeyEvent, state: ModifiersState) -> KeyboardEvent {
    info!("winit keyboard input: {:?}", input);
    KeyboardEvent {
        state: match input.state {
            ElementState::Pressed => KeyState::Down,
            ElementState::Released => KeyState::Up,
        },
        key: get_servo_key_from_winit_key(&input.logical_key),
        code: get_servo_code_from_physical_key(input.physical_key),
        location: get_servo_location_from_physical_key(input.physical_key),
        modifiers: get_modifiers(state),
        repeat: false,
        is_composing: false,
    }
}
