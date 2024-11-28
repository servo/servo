/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use keyboard_types::{Code, Key, KeyState, KeyboardEvent, Location, Modifiers};
use log::info;
use winit::event::{ElementState, KeyEvent};
use winit::keyboard::{Key as LogicalKey, KeyCode, ModifiersState, NamedKey, PhysicalKey};

/// Some shortcuts use Cmd on Mac and Control on other systems.
#[cfg(macos)]
pub const CMD_OR_CONTROL: Modifiers = Modifiers::META;
/// Some shortcuts use Cmd on Mac and Control on other systems.
#[cfg(not(macos))]
pub const CMD_OR_CONTROL: Modifiers = Modifiers::CONTROL;

/// Some shortcuts use Cmd on Mac and Alt on other systems.
#[cfg(macos)]
pub const CMD_OR_ALT: Modifiers = Modifiers::META;
/// Some shortcuts use Cmd on Mac and Alt on other systems.
#[cfg(not(macos))]
pub const CMD_OR_ALT: Modifiers = Modifiers::ALT;

/// Maps [`LogicalKey`] to [`Key`].
///
/// Example:
/// 1. `One-one mappings`:
/// ```
///     logical_to_winit_key!(a, Escape, F1, F2,...)
/// ```
/// matches [`NamedKey::Escape`] => [`Key::Escape`],
/// [`NamedKey::F1`] => [`Key::F1`],
/// [`NamedKey::F2`] => [`Key::F2`],...
///
/// 2. `Custom mappings`:
/// ```
///    logical_to_winit_key!(a, Escape, F1 => F2, F3)
/// ```
/// matches [`NamedKey::Escape`] => [`Key::Escape`],
/// [`NamedKey::F1`] => [`Key::F2`],
/// [`NamedKey::F3`] => [`Key::F3`],...
macro_rules! logical_to_winit_key {
    // Matches an optional token
    (@opt $_: ident, $optional: ident) => {
        Key::$optional
    };

    (@opt $variant: ident) => {
        Key::$variant
    };

    ($key: ident $(,$variant: ident $(=> $matchto: ident)?)+) => {
        match $key {
            LogicalKey::Character(c) => Key::Character(c.to_string()),
            $(LogicalKey::Named(NamedKey::$variant) => logical_to_winit_key!(@opt $variant $(, $matchto)?),)+
            _ => Key::Unidentified,
        }
    };
}

fn get_servo_key_from_winit_key(key: &LogicalKey) -> Key {
    // TODO: figure out how to map NavigateForward, NavigateBackward
    // TODO: map the remaining keys if possible
    logical_to_winit_key! {
        key,
        // printable: Key1 to Key0
        // printable: A to Z
        Escape, F1, F2, F3, F4, F5, F6, F7, F8, F9, F10, F11, F12,
        // F13 to F15 are not mapped
        PrintScreen,
        // Scroll not mapped
        Pause, Insert, Home, Delete, End, PageDown, PageUp,
        ArrowLeft, ArrowUp, ArrowRight, ArrowDown,
        Backspace, Enter,
        // printable: Space
        Compose,
        // Caret not mapped
        NumLock,
        // printable: Numpad0 to Numpad9
        // AbntC1 and AbntC2 not mapped
        // printable: Add, Apostrophe,
        // Apps, At, Ax not mapped
        // printable: Backslash,
        LaunchApplication2, CapsLock,
        // printable: Colon, Comma,
        Convert,
        // not mapped: Decimal,
        // printable: Divide, Equals, Grave,
        KanaMode, KanjiMode, Alt,
        // printable: LBracket,
        Control, Shift, Meta, LaunchMail,
        // not mapped: MediaSelect,
        MediaStop,
        // printable: Minus, Multiply,
        AudioVolumeMute, LaunchApplication1,
        // not mapped: NavigateForward, NavigateBackward
        MediaTrackNext, NonConvert,
        // printable: NumpadComma, NumpadEnter, NumpadEquals,
        // not mapped: OEM102,
        // printable: Period,
        MediaPlayPause, Power, MediaTrackPrevious,
        // printable RBracket
        // printable Semicolon, Slash
        Standby,
        // not mapped: Stop,
        // printable Subtract,
        // not mapped: Sysrq,
        Tab,
        // printable: Underline,
        // not mapped: Unlabeled,
        AudioVolumeDown,
        AudioVolumeUp,
        WakeUp,
        BrowserBack,
        BrowserFavorites,
        BrowserForward,
        BrowserHome,
        BrowserRefresh,
        BrowserSearch,
        BrowserStop,
        // printable Yen,
        Copy,
        Paste,
        Cut
    }
}

/// Maps [`KeyCode`] to [`Location`].
macro_rules! map_key_location {
    ($key_code: ident $(, $location: ident : $k1: ident $(| $kn: ident)*)+) => {
        match $key_code {
            $(KeyCode::$k1 $(| KeyCode::$kn)* => Location::$location,)+
            _ => Location::Standard,
        }
    };
}

fn get_servo_location_from_physical_key(physical_key: PhysicalKey) -> Location {
    let key_code = if let PhysicalKey::Code(key_code) = physical_key {
        key_code
    } else {
        return Location::Standard;
    };

    // TODO: Map more locations
    map_key_location!(
        key_code,
        Left: ShiftLeft | ControlLeft | AltLeft | SuperLeft,
        Right: ShiftRight | ControlRight | AltRight | SuperRight,
        Numpad: Numpad0 | Numpad1 | Numpad2 | Numpad3 | Numpad4 | Numpad5 | Numpad6 | Numpad7 | Numpad8 | Numpad9
            | NumpadComma | NumpadEnter | NumpadEqual | NumpadAdd | NumpadSubtract | NumpadMultiply | NumpadDivide
            | NumpadDecimal | NumpadBackspace | NumpadStar
    )
}

/// Maps [`PhysicalKey`] to [`Code`].
///
/// Example:
/// 1. `One-one mappings`:
/// ```
///     physical_key_to_code!(a, Escape, F1, F2,...)
/// ```
/// matches [`KeyCode::Escape`] => [`Code::Escape`],
/// [`KeyCode::F1`] => [`Code::F1`],
/// [`KeyCode::F2`] => [`Code::F2`],...
///
/// 2. `Custom mappings`:
/// ```
///    physical_key_to_code!(a, Escape, F1 => F2, F3)
/// ```
/// matches [`KeyCode::Escape`] => [`Code::Escape`],
/// [`KeyCode::F1`] => [`Code::F2`],
/// [`KeyCode::F3`] => [`Code::F3`],...
macro_rules! physical_key_to_code {
    // Matches an optional token
    (@opt $_: ident, $optional: ident) => {
        Code::$optional
    };

    (@opt $variant: ident) => {
        Code::$variant
    };

    ($key_code: ident $(, $pk: ident $(=> $matchto: ident)?)+) => {
        match $key_code {
            $(KeyCode::$pk => physical_key_to_code!(@opt $pk $(, $matchto)?),)+
            _ => Code::Unidentified,
        }
    };
}

fn get_servo_code_from_physical_key(physical_key: PhysicalKey) -> Code {
    let key_code = if let PhysicalKey::Code(key_code) = physical_key {
        key_code
    } else {
        return Code::Unidentified;
    };

    // TODO: Map more codes
    physical_key_to_code! {key_code,
        Escape, F1, F2, F3, F4, F5, F6, F7, F8, F9, F10, F11, F12,
        Backquote, Digit1, Digit2, Digit3, Digit4, Digit5, Digit6, Digit7, Digit8, Digit9, Digit0,
        Minus, Equal, Backspace,
        Tab, KeyQ, KeyW, KeyE, KeyR, KeyT, KeyY, KeyU, KeyI, KeyO, KeyP, BracketLeft, BracketRight, Backslash,
        CapsLock, KeyA, KeyS, KeyD, KeyF, KeyG, KeyH, KeyJ, KeyK, KeyL, Semicolon, Quote, Enter,
        ShiftLeft, KeyZ, KeyX, KeyC, KeyV, KeyB, KeyN, KeyM, Comma, Period, Slash, ShiftRight,
        ControlLeft, AltLeft, Space, AltRight, ControlRight,
        PrintScreen, ScrollLock, Pause,
        Insert, Home, PageUp,
        Delete, End, PageDown,
        ArrowUp, ArrowLeft, ArrowDown, ArrowRight,
        NumLock, NumpadDivide, NumpadMultiply, NumpadStar, NumpadSubtract,
        Numpad7, Numpad8, Numpad9, NumpadAdd,
        Numpad4, Numpad5, Numpad6,
        Numpad1, Numpad2, Numpad3, NumpadEnter,
        Numpad0, NumpadDecimal,
        NumpadParenLeft, NumpadParenRight, NumpadComma, NumpadHash, NumpadBackspace
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

/// Convert Winit's KeyEvent to Servo's KeyboardEvent
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
