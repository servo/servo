/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use glutin::{ElementState, KeyboardInput, ModifiersState, VirtualKeyCode};
use keyboard_types::{Code, Key, KeyState, KeyboardEvent, Location, Modifiers};

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

fn get_servo_key_from_winit_key(key: Option<VirtualKeyCode>) -> Key {
    use glutin::VirtualKeyCode::*;
    // TODO: figure out how to map NavigateForward, NavigateBackward
    // TODO: map the remaining keys if possible
    let key = if let Some(key) = key {
        key
    } else {
        return Key::Unidentified;
    };
    match key {
        // printable: Key1 to Key0
        // printable: A to Z
        Escape => Key::Escape,
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
        // F13 to F15 are not mapped
        Snapshot => Key::PrintScreen,
        // Scroll not mapped
        Pause => Key::Pause,
        Insert => Key::Insert,
        Home => Key::Home,
        Delete => Key::Delete,
        End => Key::End,
        PageDown => Key::PageDown,
        PageUp => Key::PageUp,
        Left => Key::ArrowLeft,
        Up => Key::ArrowUp,
        Right => Key::ArrowRight,
        Down => Key::ArrowDown,
        Back => Key::Backspace,
        Return => Key::Enter,
        // printable: Space
        Compose => Key::Compose,
        // Caret not mapped
        Numlock => Key::NumLock,
        // printable: Numpad0 to Numpad9
        // AbntC1 and AbntC2 not mapped
        // printable: Add, Apostrophe,
        // Apps, At, Ax not mapped
        // printable: Backslash,
        Calculator => Key::LaunchApplication2,
        Capital => Key::CapsLock,
        // printable: Colon, Comma,
        Convert => Key::Convert,
        // not mapped: Decimal,
        // printable: Divide, Equals, Grave,
        Kana => Key::KanaMode,
        Kanji => Key::KanjiMode,
        LAlt => Key::Alt,
        // printable: LBracket,
        LControl => Key::Control,
        LShift => Key::Shift,
        LWin => Key::Meta,
        Mail => Key::LaunchMail,
        // not mapped: MediaSelect,
        MediaStop => Key::MediaStop,
        // printable: Minus, Multiply,
        Mute => Key::AudioVolumeMute,
        MyComputer => Key::LaunchApplication1,
        // not mapped: NavigateForward, NavigateBackward
        NextTrack => Key::MediaTrackNext,
        NoConvert => Key::NonConvert,
        // printable: NumpadComma, NumpadEnter, NumpadEquals,
        // not mapped: OEM102,
        // printable: Period,
        PlayPause => Key::MediaPlayPause,
        Power => Key::Power,
        PrevTrack => Key::MediaTrackPrevious,
        RAlt => Key::Alt,
        // printable RBracket
        RControl => Key::Control,
        RShift => Key::Shift,
        RWin => Key::Meta,
        // printable Semicolon, Slash
        Sleep => Key::Standby,
        // not mapped: Stop,
        // printable Subtract,
        // not mapped: Sysrq,
        Tab => Key::Tab,
        // printable: Underline,
        // not mapped: Unlabeled,
        VolumeDown => Key::AudioVolumeDown,
        VolumeUp => Key::AudioVolumeUp,
        Wake => Key::WakeUp,
        WebBack => Key::BrowserBack,
        WebFavorites => Key::BrowserFavorites,
        WebForward => Key::BrowserForward,
        WebHome => Key::BrowserHome,
        WebRefresh => Key::BrowserRefresh,
        WebSearch => Key::BrowserSearch,
        WebStop => Key::BrowserStop,
        // printable Yen,
        Copy => Key::Copy,
        Paste => Key::Paste,
        Cut => Key::Cut,
        _ => Key::Unidentified,
    }
}

fn get_servo_location_from_winit_key(key: Option<VirtualKeyCode>) -> Location {
    use glutin::VirtualKeyCode::*;
    // TODO: add more numpad keys
    let key = if let Some(key) = key {
        key
    } else {
        return Location::Standard;
    };
    match key {
        LShift | LControl | LAlt | LWin => Location::Left,
        RShift | RControl | RAlt | RWin => Location::Right,
        Numpad0 | Numpad1 | Numpad2 | Numpad3 | Numpad4 | Numpad5 | Numpad6 | Numpad7 |
        Numpad8 | Numpad9 => Location::Numpad,
        NumpadComma | NumpadEnter | NumpadEquals => Location::Numpad,
        _ => Location::Standard,
    }
}

#[cfg(target_os = "linux")]
fn get_servo_code_from_scancode(scancode: u32) -> Code {
    // TODO: Map more codes
    use keyboard_types::Code::*;
    match scancode {
        1 => Escape,
        2 => Digit1,
        3 => Digit2,
        4 => Digit3,
        5 => Digit4,
        6 => Digit5,
        7 => Digit6,
        8 => Digit7,
        9 => Digit8,
        10 => Digit9,
        11 => Digit0,

        14 => Backspace,
        15 => Tab,
        16 => KeyQ,
        17 => KeyW,
        18 => KeyE,
        19 => KeyR,
        20 => KeyT,
        21 => KeyY,
        22 => KeyU,
        23 => KeyI,
        24 => KeyO,
        25 => KeyP,
        26 => BracketLeft,
        27 => BracketRight,
        28 => Enter,

        30 => KeyA,
        31 => KeyS,
        32 => KeyD,
        33 => KeyF,
        34 => KeyG,
        35 => KeyH,
        36 => KeyJ,
        37 => KeyK,
        38 => KeyL,
        39 => Semicolon,
        40 => Quote,

        42 => ShiftLeft,
        43 => Backslash,
        44 => KeyZ,
        45 => KeyX,
        46 => KeyC,
        47 => KeyV,
        48 => KeyB,
        49 => KeyN,
        50 => KeyM,
        51 => Comma,
        52 => Period,
        53 => Slash,
        54 => ShiftRight,

        57 => Space,

        59 => F1,
        60 => F2,
        61 => F3,
        62 => F4,
        63 => F5,
        64 => F6,
        65 => F7,
        66 => F8,
        67 => F9,
        68 => F10,

        87 => F11,
        88 => F12,

        103 => ArrowUp,
        104 => PageUp,
        105 => ArrowLeft,
        106 => ArrowRight,

        102 => Home,
        107 => End,
        108 => ArrowDown,
        109 => PageDown,
        110 => Insert,
        111 => Delete,

        _ => Unidentified,
    }
}

#[cfg(not(target_os = "linux"))]
fn get_servo_code_from_scancode(_scancode: u32) -> Code {
    // TODO: Implement for Windows and Mac OS
    Code::Unidentified
}

fn get_modifiers(mods: ModifiersState) -> Modifiers {
    let mut modifiers = Modifiers::empty();
    modifiers.set(Modifiers::CONTROL, mods.ctrl);
    modifiers.set(Modifiers::SHIFT, mods.shift);
    modifiers.set(Modifiers::ALT, mods.alt);
    modifiers.set(Modifiers::META, mods.logo);
    modifiers
}

pub fn keyboard_event_from_winit(input: KeyboardInput) -> KeyboardEvent {
    info!("winit keyboard input: {:?}", input);
    KeyboardEvent {
        state: match input.state {
            ElementState::Pressed => KeyState::Down,
            ElementState::Released => KeyState::Up,
        },
        key: get_servo_key_from_winit_key(input.virtual_keycode),
        code: get_servo_code_from_scancode(input.scancode),
        location: get_servo_location_from_winit_key(input.virtual_keycode),
        modifiers: get_modifiers(input.modifiers),
        repeat: false,
        is_composing: false,
    }
}
