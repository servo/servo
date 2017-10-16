/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use dom::bindings::cell::DomRefCell;
use dom::bindings::codegen::Bindings::KeyboardEventBinding;
use dom::bindings::codegen::Bindings::KeyboardEventBinding::{KeyboardEventConstants, KeyboardEventMethods};
use dom::bindings::codegen::Bindings::UIEventBinding::UIEventMethods;
use dom::bindings::error::Fallible;
use dom::bindings::inheritance::Castable;
use dom::bindings::reflector::reflect_dom_object;
use dom::bindings::root::{DomRoot, RootedReference};
use dom::bindings::str::DOMString;
use dom::event::Event;
use dom::uievent::UIEvent;
use dom::window::Window;
use dom_struct::dom_struct;
use msg::constellation_msg;
use msg::constellation_msg::{Key, KeyModifiers};
use std::borrow::Cow;
use std::cell::Cell;

unsafe_no_jsmanaged_fields!(Key);

#[dom_struct]
pub struct KeyboardEvent {
    uievent: UIEvent,
    key: Cell<Option<Key>>,
    key_string: DomRefCell<DOMString>,
    code: DomRefCell<DOMString>,
    location: Cell<u32>,
    ctrl: Cell<bool>,
    alt: Cell<bool>,
    shift: Cell<bool>,
    meta: Cell<bool>,
    repeat: Cell<bool>,
    is_composing: Cell<bool>,
    char_code: Cell<Option<u32>>,
    key_code: Cell<u32>,
    printable: Cell<Option<char>>,
}

impl KeyboardEvent {
    fn new_inherited() -> KeyboardEvent {
        KeyboardEvent {
            uievent: UIEvent::new_inherited(),
            key: Cell::new(None),
            key_string: DomRefCell::new(DOMString::new()),
            code: DomRefCell::new(DOMString::new()),
            location: Cell::new(0),
            ctrl: Cell::new(false),
            alt: Cell::new(false),
            shift: Cell::new(false),
            meta: Cell::new(false),
            repeat: Cell::new(false),
            is_composing: Cell::new(false),
            char_code: Cell::new(None),
            key_code: Cell::new(0),
            printable: Cell::new(None),
        }
    }

    pub fn new_uninitialized(window: &Window) -> DomRoot<KeyboardEvent> {
        reflect_dom_object(Box::new(KeyboardEvent::new_inherited()),
                           window,
                           KeyboardEventBinding::Wrap)
    }

    pub fn new(window: &Window,
               type_: DOMString,
               can_bubble: bool,
               cancelable: bool,
               view: Option<&Window>,
               _detail: i32,
               ch: Option<char>,
               key: Option<Key>,
               key_string: DOMString,
               code: DOMString,
               location: u32,
               repeat: bool,
               is_composing: bool,
               ctrl_key: bool,
               alt_key: bool,
               shift_key: bool,
               meta_key: bool,
               char_code: Option<u32>,
               key_code: u32) -> DomRoot<KeyboardEvent> {
        let ev = KeyboardEvent::new_uninitialized(window);
        ev.InitKeyboardEvent(type_, can_bubble, cancelable, view, key_string, location,
                             DOMString::new(), repeat, DOMString::new());
        ev.key.set(key);
        *ev.code.borrow_mut() = code;
        ev.ctrl.set(ctrl_key);
        ev.alt.set(alt_key);
        ev.shift.set(shift_key);
        ev.meta.set(meta_key);
        ev.char_code.set(char_code);
        ev.printable.set(ch);
        ev.key_code.set(key_code);
        ev.is_composing.set(is_composing);
        ev
    }

    pub fn Constructor(window: &Window,
                       type_: DOMString,
                       init: &KeyboardEventBinding::KeyboardEventInit) -> Fallible<DomRoot<KeyboardEvent>> {
        let event = KeyboardEvent::new(window,
                                       type_,
                                       init.parent.parent.parent.bubbles,
                                       init.parent.parent.parent.cancelable,
                                       init.parent.parent.view.r(),
                                       init.parent.parent.detail,
                                       None,
                                       key_from_string(&init.key, init.location),
                                       init.key.clone(), init.code.clone(), init.location,
                                       init.repeat, init.isComposing, init.parent.ctrlKey,
                                       init.parent.altKey, init.parent.shiftKey, init.parent.metaKey,
                                       None, 0);
        Ok(event)
    }

    pub fn key_properties(ch: Option<char>, key: Key, mods: KeyModifiers)
        -> KeyEventProperties {
            KeyEventProperties {
                key_string: key_value(ch, key, mods),
                code: code_value(key),
                location: key_location(key),
                char_code: ch.map(|ch| ch as u32),
                key_code: key_keycode(key),
            }
    }
}


impl KeyboardEvent {
    pub fn printable(&self) -> Option<char> {
        self.printable.get()
    }

    pub fn get_key(&self) -> Option<Key> {
        self.key.get().clone()
    }

    pub fn get_key_modifiers(&self) -> KeyModifiers {
        let mut result = KeyModifiers::empty();
        if self.shift.get() {
            result = result | constellation_msg::SHIFT;
        }
        if self.ctrl.get() {
            result = result | constellation_msg::CONTROL;
        }
        if self.alt.get() {
            result = result | constellation_msg::ALT;
        }
        if self.meta.get() {
            result = result | constellation_msg::SUPER;
        }
        result
    }
}

// https://w3c.github.io/uievents-key/#key-value-tables
pub fn key_value(ch: Option<char>, key: Key, mods: KeyModifiers) -> Cow<'static, str> {
    if let Some(ch) = ch {
        return Cow::from(format!("{}", ch));
    }

    let shift = mods.contains(constellation_msg::SHIFT);
    Cow::from(match key {
        Key::Space => " ",
        Key::Apostrophe if shift => "\"",
        Key::Apostrophe => "'",
        Key::Comma if shift => "<",
        Key::Comma => ",",
        Key::Minus if shift => "_",
        Key::Minus => "-",
        Key::Period if shift => ">",
        Key::Period => ".",
        Key::Slash if shift => "?",
        Key::Slash => "/",
        Key::GraveAccent if shift => "~",
        Key::GraveAccent => "`",
        Key::Num0 if shift => ")",
        Key::Num0 => "0",
        Key::Num1 if shift => "!",
        Key::Num1 => "1",
        Key::Num2 if shift => "@",
        Key::Num2 => "2",
        Key::Num3 if shift => "#",
        Key::Num3 => "3",
        Key::Num4 if shift => "$",
        Key::Num4 => "4",
        Key::Num5 if shift => "%",
        Key::Num5 => "5",
        Key::Num6 if shift => "^",
        Key::Num6 => "6",
        Key::Num7 if shift => "&",
        Key::Num7 => "7",
        Key::Num8 if shift => "*",
        Key::Num8 => "8",
        Key::Num9 if shift => "(",
        Key::Num9 => "9",
        Key::Semicolon if shift => ":",
        Key::Semicolon => ";",
        Key::Equal if shift => "+",
        Key::Equal => "=",
        Key::A if shift => "A",
        Key::A => "a",
        Key::B if shift => "B",
        Key::B => "b",
        Key::C if shift => "C",
        Key::C => "c",
        Key::D if shift => "D",
        Key::D => "d",
        Key::E if shift => "E",
        Key::E => "e",
        Key::F if shift => "F",
        Key::F => "f",
        Key::G if shift => "G",
        Key::G => "g",
        Key::H if shift => "H",
        Key::H => "h",
        Key::I if shift => "I",
        Key::I => "i",
        Key::J if shift => "J",
        Key::J => "j",
        Key::K if shift => "K",
        Key::K => "k",
        Key::L if shift => "L",
        Key::L => "l",
        Key::M if shift => "M",
        Key::M => "m",
        Key::N if shift => "N",
        Key::N => "n",
        Key::O if shift => "O",
        Key::O => "o",
        Key::P if shift => "P",
        Key::P => "p",
        Key::Q if shift => "Q",
        Key::Q => "q",
        Key::R if shift => "R",
        Key::R => "r",
        Key::S if shift => "S",
        Key::S => "s",
        Key::T if shift => "T",
        Key::T => "t",
        Key::U if shift => "U",
        Key::U => "u",
        Key::V if shift => "V",
        Key::V => "v",
        Key::W if shift => "W",
        Key::W => "w",
        Key::X if shift => "X",
        Key::X => "x",
        Key::Y if shift => "Y",
        Key::Y => "y",
        Key::Z if shift => "Z",
        Key::Z => "z",
        Key::LeftBracket if shift => "{",
        Key::LeftBracket => "[",
        Key::Backslash if shift => "|",
        Key::Backslash => "\\",
        Key::RightBracket if shift => "}",
        Key::RightBracket => "]",
        Key::World1 => "Unidentified",
        Key::World2 => "Unidentified",
        Key::Escape => "Escape",
        Key::Enter => "Enter",
        Key::Tab => "Tab",
        Key::Backspace => "Backspace",
        Key::Insert => "Insert",
        Key::Delete => "Delete",
        Key::Right => "ArrowRight",
        Key::Left => "ArrowLeft",
        Key::Down => "ArrowDown",
        Key::Up => "ArrowUp",
        Key::PageUp => "PageUp",
        Key::PageDown => "PageDown",
        Key::Home => "Home",
        Key::End => "End",
        Key::CapsLock => "CapsLock",
        Key::ScrollLock => "ScrollLock",
        Key::NumLock => "NumLock",
        Key::PrintScreen => "PrintScreen",
        Key::Pause => "Pause",
        Key::F1 => "F1",
        Key::F2 => "F2",
        Key::F3 => "F3",
        Key::F4 => "F4",
        Key::F5 => "F5",
        Key::F6 => "F6",
        Key::F7 => "F7",
        Key::F8 => "F8",
        Key::F9 => "F9",
        Key::F10 => "F10",
        Key::F11 => "F11",
        Key::F12 => "F12",
        Key::F13 => "F13",
        Key::F14 => "F14",
        Key::F15 => "F15",
        Key::F16 => "F16",
        Key::F17 => "F17",
        Key::F18 => "F18",
        Key::F19 => "F19",
        Key::F20 => "F20",
        Key::F21 => "F21",
        Key::F22 => "F22",
        Key::F23 => "F23",
        Key::F24 => "F24",
        Key::F25 => "F25",
        Key::Kp0 => "0",
        Key::Kp1 => "1",
        Key::Kp2 => "2",
        Key::Kp3 => "3",
        Key::Kp4 => "4",
        Key::Kp5 => "5",
        Key::Kp6 => "6",
        Key::Kp7 => "7",
        Key::Kp8 => "8",
        Key::Kp9 => "9",
        Key::KpDecimal => ".",
        Key::KpDivide => "/",
        Key::KpMultiply => "*",
        Key::KpSubtract => "-",
        Key::KpAdd => "+",
        Key::KpEnter => "Enter",
        Key::KpEqual => "=",
        Key::LeftShift => "Shift",
        Key::LeftControl => "Control",
        Key::LeftAlt => "Alt",
        Key::LeftSuper => "Super",
        Key::RightShift => "Shift",
        Key::RightControl => "Control",
        Key::RightAlt => "Alt",
        Key::RightSuper => "Super",
        Key::Menu => "ContextMenu",
        Key::NavigateForward => "BrowserForward",
        Key::NavigateBackward => "BrowserBack",
    })
}

fn key_from_string(key_string: &str, location: u32) -> Option<Key> {
    match key_string {
        " " => Some(Key::Space),
        "\"" => Some(Key::Apostrophe),
        "'" => Some(Key::Apostrophe),
        "<" => Some(Key::Comma),
        "," => Some(Key::Comma),
        "_" => Some(Key::Minus),
        "-" if location == KeyboardEventConstants::DOM_KEY_LOCATION_STANDARD => Some(Key::Minus),
        ">" => Some(Key::Period),
        "." if location == KeyboardEventConstants::DOM_KEY_LOCATION_STANDARD => Some(Key::Period),
        "?" => Some(Key::Slash),
        "/" if location == KeyboardEventConstants::DOM_KEY_LOCATION_STANDARD => Some(Key::Slash),
        "~" => Some(Key::GraveAccent),
        "`" => Some(Key::GraveAccent),
        ")" => Some(Key::Num0),
        "0" if location == KeyboardEventConstants::DOM_KEY_LOCATION_STANDARD => Some(Key::Num0),
        "!" => Some(Key::Num1),
        "1" if location == KeyboardEventConstants::DOM_KEY_LOCATION_STANDARD => Some(Key::Num1),
        "@" => Some(Key::Num2),
        "2" if location == KeyboardEventConstants::DOM_KEY_LOCATION_STANDARD => Some(Key::Num2),
        "#" => Some(Key::Num3),
        "3" if location == KeyboardEventConstants::DOM_KEY_LOCATION_STANDARD => Some(Key::Num3),
        "$" => Some(Key::Num4),
        "4" if location == KeyboardEventConstants::DOM_KEY_LOCATION_STANDARD => Some(Key::Num4),
        "%" => Some(Key::Num5),
        "5" if location == KeyboardEventConstants::DOM_KEY_LOCATION_STANDARD => Some(Key::Num5),
        "^" => Some(Key::Num6),
        "6" if location == KeyboardEventConstants::DOM_KEY_LOCATION_STANDARD => Some(Key::Num6),
        "&" => Some(Key::Num7),
        "7" if location == KeyboardEventConstants::DOM_KEY_LOCATION_STANDARD => Some(Key::Num7),
        "*" if location == KeyboardEventConstants::DOM_KEY_LOCATION_STANDARD => Some(Key::Num8),
        "8" if location == KeyboardEventConstants::DOM_KEY_LOCATION_STANDARD => Some(Key::Num8),
        "(" => Some(Key::Num9),
        "9" if location == KeyboardEventConstants::DOM_KEY_LOCATION_STANDARD => Some(Key::Num9),
        ":" => Some(Key::Semicolon),
        ";" => Some(Key::Semicolon),
        "+" if location == KeyboardEventConstants::DOM_KEY_LOCATION_STANDARD => Some(Key::Equal),
        "=" if location == KeyboardEventConstants::DOM_KEY_LOCATION_STANDARD => Some(Key::Equal),
        "A" => Some(Key::A),
        "a" => Some(Key::A),
        "B" => Some(Key::B),
        "b" => Some(Key::B),
        "C" => Some(Key::C),
        "c" => Some(Key::C),
        "D" => Some(Key::D),
        "d" => Some(Key::D),
        "E" => Some(Key::E),
        "e" => Some(Key::E),
        "F" => Some(Key::F),
        "f" => Some(Key::F),
        "G" => Some(Key::G),
        "g" => Some(Key::G),
        "H" => Some(Key::H),
        "h" => Some(Key::H),
        "I" => Some(Key::I),
        "i" => Some(Key::I),
        "J" => Some(Key::J),
        "j" => Some(Key::J),
        "K" => Some(Key::K),
        "k" => Some(Key::K),
        "L" => Some(Key::L),
        "l" => Some(Key::L),
        "M" => Some(Key::M),
        "m" => Some(Key::M),
        "N" => Some(Key::N),
        "n" => Some(Key::N),
        "O" => Some(Key::O),
        "o" => Some(Key::O),
        "P" => Some(Key::P),
        "p" => Some(Key::P),
        "Q" => Some(Key::Q),
        "q" => Some(Key::Q),
        "R" => Some(Key::R),
        "r" => Some(Key::R),
        "S" => Some(Key::S),
        "s" => Some(Key::S),
        "T" => Some(Key::T),
        "t" => Some(Key::T),
        "U" => Some(Key::U),
        "u" => Some(Key::U),
        "V" => Some(Key::V),
        "v" => Some(Key::V),
        "W" => Some(Key::W),
        "w" => Some(Key::W),
        "X" => Some(Key::X),
        "x" => Some(Key::X),
        "Y" => Some(Key::Y),
        "y" => Some(Key::Y),
        "Z" => Some(Key::Z),
        "z" => Some(Key::Z),
        "{" => Some(Key::LeftBracket),
        "[" => Some(Key::LeftBracket),
        "|" => Some(Key::Backslash),
        "\\" => Some(Key::Backslash),
        "}" => Some(Key::RightBracket),
        "]" => Some(Key::RightBracket),
        "Escape" => Some(Key::Escape),
        "Enter" if location == KeyboardEventConstants::DOM_KEY_LOCATION_STANDARD => Some(Key::Enter),
        "Tab" => Some(Key::Tab),
        "Backspace" => Some(Key::Backspace),
        "Insert" => Some(Key::Insert),
        "Delete" => Some(Key::Delete),
        "ArrowRight" => Some(Key::Right),
        "ArrowLeft" => Some(Key::Left),
        "ArrowDown" => Some(Key::Down),
        "ArrowUp" => Some(Key::Up),
        "PageUp" => Some(Key::PageUp),
        "PageDown" => Some(Key::PageDown),
        "Home" => Some(Key::Home),
        "End" => Some(Key::End),
        "CapsLock" => Some(Key::CapsLock),
        "ScrollLock" => Some(Key::ScrollLock),
        "NumLock" => Some(Key::NumLock),
        "PrintScreen" => Some(Key::PrintScreen),
        "Pause" => Some(Key::Pause),
        "F1" => Some(Key::F1),
        "F2" => Some(Key::F2),
        "F3" => Some(Key::F3),
        "F4" => Some(Key::F4),
        "F5" => Some(Key::F5),
        "F6" => Some(Key::F6),
        "F7" => Some(Key::F7),
        "F8" => Some(Key::F8),
        "F9" => Some(Key::F9),
        "F10" => Some(Key::F10),
        "F11" => Some(Key::F11),
        "F12" => Some(Key::F12),
        "F13" => Some(Key::F13),
        "F14" => Some(Key::F14),
        "F15" => Some(Key::F15),
        "F16" => Some(Key::F16),
        "F17" => Some(Key::F17),
        "F18" => Some(Key::F18),
        "F19" => Some(Key::F19),
        "F20" => Some(Key::F20),
        "F21" => Some(Key::F21),
        "F22" => Some(Key::F22),
        "F23" => Some(Key::F23),
        "F24" => Some(Key::F24),
        "F25" => Some(Key::F25),
        "0" if location == KeyboardEventConstants::DOM_KEY_LOCATION_NUMPAD => Some(Key::Kp0),
        "1" if location == KeyboardEventConstants::DOM_KEY_LOCATION_NUMPAD => Some(Key::Kp1),
        "2" if location == KeyboardEventConstants::DOM_KEY_LOCATION_NUMPAD => Some(Key::Kp2),
        "3" if location == KeyboardEventConstants::DOM_KEY_LOCATION_NUMPAD => Some(Key::Kp3),
        "4" if location == KeyboardEventConstants::DOM_KEY_LOCATION_NUMPAD => Some(Key::Kp4),
        "5" if location == KeyboardEventConstants::DOM_KEY_LOCATION_NUMPAD => Some(Key::Kp5),
        "6" if location == KeyboardEventConstants::DOM_KEY_LOCATION_NUMPAD => Some(Key::Kp6),
        "7" if location == KeyboardEventConstants::DOM_KEY_LOCATION_NUMPAD => Some(Key::Kp7),
        "8" if location == KeyboardEventConstants::DOM_KEY_LOCATION_NUMPAD => Some(Key::Kp8),
        "9" if location == KeyboardEventConstants::DOM_KEY_LOCATION_NUMPAD => Some(Key::Kp9),
        "." if location == KeyboardEventConstants::DOM_KEY_LOCATION_NUMPAD => Some(Key::KpDecimal),
        "/" if location == KeyboardEventConstants::DOM_KEY_LOCATION_NUMPAD => Some(Key::KpDivide),
        "*" if location == KeyboardEventConstants::DOM_KEY_LOCATION_NUMPAD => Some(Key::KpMultiply),
        "-" if location == KeyboardEventConstants::DOM_KEY_LOCATION_NUMPAD => Some(Key::KpSubtract),
        "+" if location == KeyboardEventConstants::DOM_KEY_LOCATION_NUMPAD => Some(Key::KpAdd),
        "Enter" if location == KeyboardEventConstants::DOM_KEY_LOCATION_NUMPAD => Some(Key::KpEnter),
        "=" if location == KeyboardEventConstants::DOM_KEY_LOCATION_NUMPAD => Some(Key::KpEqual),
        "Shift" if location == KeyboardEventConstants::DOM_KEY_LOCATION_LEFT => Some(Key::LeftShift),
        "Control" if location == KeyboardEventConstants::DOM_KEY_LOCATION_LEFT => Some(Key::LeftControl),
        "Alt" if location == KeyboardEventConstants::DOM_KEY_LOCATION_LEFT => Some(Key::LeftAlt),
        "Super" if location == KeyboardEventConstants::DOM_KEY_LOCATION_LEFT => Some(Key::LeftSuper),
        "Shift" if location == KeyboardEventConstants::DOM_KEY_LOCATION_RIGHT => Some(Key::RightShift),
        "Control" if location == KeyboardEventConstants::DOM_KEY_LOCATION_RIGHT => Some(Key::RightControl),
        "Alt" if location == KeyboardEventConstants::DOM_KEY_LOCATION_RIGHT => Some(Key::RightAlt),
        "Super" if location == KeyboardEventConstants::DOM_KEY_LOCATION_RIGHT => Some(Key::RightSuper),
        "ContextMenu" => Some(Key::Menu),
        "BrowserForward" => Some(Key::NavigateForward),
        "BrowserBack" => Some(Key::NavigateBackward),
        _ => None
    }
}

// https://w3c.github.io/uievents-code/#code-value-tables
fn code_value(key: Key) -> &'static str {
    match key {
        Key::Space => "Space",
        Key::Apostrophe => "Quote",
        Key::Comma => "Comma",
        Key::Minus => "Minus",
        Key::Period => "Period",
        Key::Slash => "Slash",
        Key::GraveAccent => "Backquote",
        Key::Num0 => "Digit0",
        Key::Num1 => "Digit1",
        Key::Num2 => "Digit2",
        Key::Num3 => "Digit3",
        Key::Num4 => "Digit4",
        Key::Num5 => "Digit5",
        Key::Num6 => "Digit6",
        Key::Num7 => "Digit7",
        Key::Num8 => "Digit8",
        Key::Num9 => "Digit9",
        Key::Semicolon => "Semicolon",
        Key::Equal => "Equal",
        Key::A => "KeyA",
        Key::B => "KeyB",
        Key::C => "KeyC",
        Key::D => "KeyD",
        Key::E => "KeyE",
        Key::F => "KeyF",
        Key::G => "KeyG",
        Key::H => "KeyH",
        Key::I => "KeyI",
        Key::J => "KeyJ",
        Key::K => "KeyK",
        Key::L => "KeyL",
        Key::M => "KeyM",
        Key::N => "KeyN",
        Key::O => "KeyO",
        Key::P => "KeyP",
        Key::Q => "KeyQ",
        Key::R => "KeyR",
        Key::S => "KeyS",
        Key::T => "KeyT",
        Key::U => "KeyU",
        Key::V => "KeyV",
        Key::W => "KeyW",
        Key::X => "KeyX",
        Key::Y => "KeyY",
        Key::Z => "KeyZ",
        Key::LeftBracket => "BracketLeft",
        Key::Backslash => "Backslash",
        Key::RightBracket => "BracketRight",

        Key::World1 |
        Key::World2 => panic!("unknown char code for {:?}", key),

        Key::Escape => "Escape",
        Key::Enter => "Enter",
        Key::Tab => "Tab",
        Key::Backspace => "Backspace",
        Key::Insert => "Insert",
        Key::Delete => "Delete",
        Key::Right => "ArrowRight",
        Key::Left => "ArrowLeft",
        Key::Down => "ArrowDown",
        Key::Up => "ArrowUp",
        Key::PageUp => "PageUp",
        Key::PageDown => "PageDown",
        Key::Home => "Home",
        Key::End => "End",
        Key::CapsLock => "CapsLock",
        Key::ScrollLock => "ScrollLock",
        Key::NumLock => "NumLock",
        Key::PrintScreen => "PrintScreen",
        Key::Pause => "Pause",
        Key::F1 => "F1",
        Key::F2 => "F2",
        Key::F3 => "F3",
        Key::F4 => "F4",
        Key::F5 => "F5",
        Key::F6 => "F6",
        Key::F7 => "F7",
        Key::F8 => "F8",
        Key::F9 => "F9",
        Key::F10 => "F10",
        Key::F11 => "F11",
        Key::F12 => "F12",
        Key::F13 => "F13",
        Key::F14 => "F14",
        Key::F15 => "F15",
        Key::F16 => "F16",
        Key::F17 => "F17",
        Key::F18 => "F18",
        Key::F19 => "F19",
        Key::F20 => "F20",
        Key::F21 => "F21",
        Key::F22 => "F22",
        Key::F23 => "F23",
        Key::F24 => "F24",
        Key::F25 => "F25",
        Key::Kp0 => "Numpad0",
        Key::Kp1 => "Numpad1",
        Key::Kp2 => "Numpad2",
        Key::Kp3 => "Numpad3",
        Key::Kp4 => "Numpad4",
        Key::Kp5 => "Numpad5",
        Key::Kp6 => "Numpad6",
        Key::Kp7 => "Numpad7",
        Key::Kp8 => "Numpad8",
        Key::Kp9 => "Numpad9",
        Key::KpDecimal => "NumpadDecimal",
        Key::KpDivide => "NumpadDivide",
        Key::KpMultiply => "NumpadMultiply",
        Key::KpSubtract => "NumpadSubtract",
        Key::KpAdd => "NumpadAdd",
        Key::KpEnter => "NumpadEnter",
        Key::KpEqual => "NumpadEqual",
        Key::LeftShift | Key::RightShift => "Shift",
        Key::LeftControl | Key::RightControl => "Control",
        Key::LeftAlt | Key::RightAlt => "Alt",
        Key::LeftSuper | Key::RightSuper => "Super",
        Key::Menu => "ContextMenu",

        Key::NavigateForward => "BrowserForward",
        Key::NavigateBackward => "BrowserBackward",
    }
}

fn key_location(key: Key) -> u32 {
    match key {
        Key::Kp0 | Key::Kp1 | Key::Kp2 |
        Key::Kp3 | Key::Kp4 | Key::Kp5 |
        Key::Kp6 | Key::Kp7 | Key::Kp8 |
        Key::Kp9 | Key::KpDecimal |
        Key::KpDivide | Key::KpMultiply |
        Key::KpSubtract | Key::KpAdd |
        Key::KpEnter | Key::KpEqual =>
            KeyboardEventConstants::DOM_KEY_LOCATION_NUMPAD,

        Key::LeftShift | Key::LeftAlt |
        Key::LeftControl | Key::LeftSuper =>
            KeyboardEventConstants::DOM_KEY_LOCATION_LEFT,

        Key::RightShift | Key::RightAlt |
        Key::RightControl | Key::RightSuper =>
            KeyboardEventConstants::DOM_KEY_LOCATION_RIGHT,

        _ => KeyboardEventConstants::DOM_KEY_LOCATION_STANDARD,
    }
}

// https://w3c.github.io/uievents/#legacy-key-models
fn key_keycode(key: Key) -> u32 {
    match key {
        // https://w3c.github.io/uievents/#legacy-key-models
        Key::Backspace => 8,
        Key::Tab => 9,
        Key::Enter => 13,
        Key::LeftShift | Key::RightShift => 16,
        Key::LeftControl | Key::RightControl => 17,
        Key::LeftAlt | Key::RightAlt => 18,
        Key::CapsLock => 20,
        Key::Escape => 27,
        Key::Space => 32,
        Key::PageUp => 33,
        Key::PageDown => 34,
        Key::End => 35,
        Key::Home => 36,
        Key::Left => 37,
        Key::Up => 38,
        Key::Right => 39,
        Key::Down => 40,
        Key::Delete => 46,

        // https://w3c.github.io/uievents/#optionally-fixed-virtual-key-codes
        Key::Semicolon => 186,
        Key::Equal => 187,
        Key::Comma => 188,
        Key::Minus => 189,
        Key::Period => 190,
        Key::Slash => 191,
        Key::LeftBracket => 219,
        Key::Backslash => 220,
        Key::RightBracket => 221,
        Key::Apostrophe => 222,

        //§ B.2.1.3
        Key::Num0 |
        Key::Num1 |
        Key::Num2 |
        Key::Num3 |
        Key::Num4 |
        Key::Num5 |
        Key::Num6 |
        Key::Num7 |
        Key::Num8 |
        Key::Num9 => key as u32 - Key::Num0 as u32 + '0' as u32,

        //§ B.2.1.4
        Key::A |
        Key::B |
        Key::C |
        Key::D |
        Key::E |
        Key::F |
        Key::G |
        Key::H |
        Key::I |
        Key::J |
        Key::K |
        Key::L |
        Key::M |
        Key::N |
        Key::O |
        Key::P |
        Key::Q |
        Key::R |
        Key::S |
        Key::T |
        Key::U |
        Key::V |
        Key::W |
        Key::X |
        Key::Y |
        Key::Z => key as u32 - Key::A as u32 + 'A' as u32,

        //§ B.2.1.8
        _ => 0
    }
}

#[derive(HeapSizeOf)]
pub struct KeyEventProperties {
    pub key_string: Cow<'static, str>,
    pub code: &'static str,
    pub location: u32,
    pub char_code: Option<u32>,
    pub key_code: u32,
}

impl KeyEventProperties {
    pub fn is_printable(&self) -> bool {
        self.char_code.is_some()
    }
}

impl KeyboardEventMethods for KeyboardEvent {
    // https://w3c.github.io/uievents/#widl-KeyboardEvent-initKeyboardEvent
    fn InitKeyboardEvent(&self,
                         type_arg: DOMString,
                         can_bubble_arg: bool,
                         cancelable_arg: bool,
                         view_arg: Option<&Window>,
                         key_arg: DOMString,
                         location_arg: u32,
                         _modifiers_list_arg: DOMString,
                         repeat: bool,
                         _locale: DOMString) {
        if self.upcast::<Event>().dispatching() {
            return;
        }

        self.upcast::<UIEvent>()
            .InitUIEvent(type_arg, can_bubble_arg, cancelable_arg, view_arg, 0);
        *self.key_string.borrow_mut() = key_arg;
        self.location.set(location_arg);
        self.repeat.set(repeat);
    }

    // https://w3c.github.io/uievents/#widl-KeyboardEvent-key
    fn Key(&self) -> DOMString {
        self.key_string.borrow().clone()
    }

    // https://w3c.github.io/uievents/#widl-KeyboardEvent-code
    fn Code(&self) -> DOMString {
        self.code.borrow().clone()
    }

    // https://w3c.github.io/uievents/#widl-KeyboardEvent-location
    fn Location(&self) -> u32 {
        self.location.get()
    }

    // https://w3c.github.io/uievents/#widl-KeyboardEvent-ctrlKey
    fn CtrlKey(&self) -> bool {
        self.ctrl.get()
    }

    // https://w3c.github.io/uievents/#widl-KeyboardEvent-shiftKey
    fn ShiftKey(&self) -> bool {
        self.shift.get()
    }

    // https://w3c.github.io/uievents/#widl-KeyboardEvent-altKey
    fn AltKey(&self) -> bool {
        self.alt.get()
    }

    // https://w3c.github.io/uievents/#widl-KeyboardEvent-metaKey
    fn MetaKey(&self) -> bool {
        self.meta.get()
    }

    // https://w3c.github.io/uievents/#widl-KeyboardEvent-repeat
    fn Repeat(&self) -> bool {
        self.repeat.get()
    }

    // https://w3c.github.io/uievents/#widl-KeyboardEvent-isComposing
    fn IsComposing(&self) -> bool {
        self.is_composing.get()
    }

    // https://w3c.github.io/uievents/#dom-keyboardevent-getmodifierstate
    fn GetModifierState(&self, key_arg: DOMString) -> bool {
        match &*key_arg {
            "Ctrl" => self.CtrlKey(),
            "Alt" => self.AltKey(),
            "Shift" => self.ShiftKey(),
            "Meta" => self.MetaKey(),
            "AltGraph" | "CapsLock" | "NumLock" | "ScrollLock" | "Accel" |
            "Fn" | "FnLock" | "Hyper" | "OS" | "Symbol" | "SymbolLock" => false, //FIXME
            _ => false,
        }
    }

    // https://w3c.github.io/uievents/#widl-KeyboardEvent-charCode
    fn CharCode(&self) -> u32 {
        self.char_code.get().unwrap_or(0)
    }

    // https://w3c.github.io/uievents/#widl-KeyboardEvent-keyCode
    fn KeyCode(&self) -> u32 {
        self.key_code.get()
    }

    // https://w3c.github.io/uievents/#widl-KeyboardEvent-which
    fn Which(&self) -> u32 {
        self.char_code.get().unwrap_or(self.KeyCode())
    }

    // https://dom.spec.whatwg.org/#dom-event-istrusted
    fn IsTrusted(&self) -> bool {
        self.uievent.IsTrusted()
    }
}
