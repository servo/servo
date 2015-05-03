/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use dom::bindings::codegen::Bindings::KeyboardEventBinding;
use dom::bindings::codegen::Bindings::KeyboardEventBinding::{KeyboardEventMethods, KeyboardEventConstants};
use dom::bindings::codegen::Bindings::UIEventBinding::UIEventMethods;
use dom::bindings::codegen::InheritTypes::{EventCast, UIEventCast, KeyboardEventDerived};
use dom::bindings::error::Fallible;
use dom::bindings::global::GlobalRef;
use dom::bindings::js::{JSRef, Temporary, Rootable, RootedReference};
use dom::bindings::utils::{Reflectable, reflect_dom_object};
use dom::event::{Event, EventTypeId};
use dom::uievent::UIEvent;
use dom::window::Window;
use msg::constellation_msg;
use msg::constellation_msg::{Key, KeyModifiers};
use msg::constellation_msg::{SHIFT, CONTROL, ALT, SUPER};
use util::str::DOMString;

use std::borrow::ToOwned;
use std::cell::{RefCell, Cell};

no_jsmanaged_fields!(Key);

#[dom_struct]
pub struct KeyboardEvent {
    uievent: UIEvent,
    key: Cell<Option<Key>>,
    key_string: RefCell<DOMString>,
    code: RefCell<DOMString>,
    location: Cell<u32>,
    ctrl: Cell<bool>,
    alt: Cell<bool>,
    shift: Cell<bool>,
    meta: Cell<bool>,
    repeat: Cell<bool>,
    is_composing: Cell<bool>,
    char_code: Cell<Option<u32>>,
    key_code: Cell<u32>,
}

impl KeyboardEventDerived for Event {
    fn is_keyboardevent(&self) -> bool {
        *self.type_id() == EventTypeId::KeyboardEvent
    }
}

impl KeyboardEvent {
    fn new_inherited() -> KeyboardEvent {
        KeyboardEvent {
            uievent: UIEvent::new_inherited(EventTypeId::KeyboardEvent),
            key: Cell::new(None),
            key_string: RefCell::new("".to_owned()),
            code: RefCell::new("".to_owned()),
            location: Cell::new(0),
            ctrl: Cell::new(false),
            alt: Cell::new(false),
            shift: Cell::new(false),
            meta: Cell::new(false),
            repeat: Cell::new(false),
            is_composing: Cell::new(false),
            char_code: Cell::new(None),
            key_code: Cell::new(0),
        }
    }

    pub fn new_uninitialized(window: JSRef<Window>) -> Temporary<KeyboardEvent> {
        reflect_dom_object(box KeyboardEvent::new_inherited(),
                           GlobalRef::Window(window),
                           KeyboardEventBinding::Wrap)
    }

    pub fn new(window: JSRef<Window>,
               type_: DOMString,
               canBubble: bool,
               cancelable: bool,
               view: Option<JSRef<Window>>,
               _detail: i32,
               key: Option<Key>,
               key_string: DOMString,
               code: DOMString,
               location: u32,
               repeat: bool,
               isComposing: bool,
               ctrlKey: bool,
               altKey: bool,
               shiftKey: bool,
               metaKey: bool,
               char_code: Option<u32>,
               key_code: u32) -> Temporary<KeyboardEvent> {
        let ev = KeyboardEvent::new_uninitialized(window).root();
        ev.r().InitKeyboardEvent(type_, canBubble, cancelable, view, key_string, location,
                                 "".to_owned(), repeat, "".to_owned());
        // FIXME(https://github.com/rust-lang/rust/issues/23338)
        let ev = ev.r();
        ev.key.set(key);
        *ev.code.borrow_mut() = code;
        ev.ctrl.set(ctrlKey);
        ev.alt.set(altKey);
        ev.shift.set(shiftKey);
        ev.meta.set(metaKey);
        ev.char_code.set(char_code);
        ev.key_code.set(key_code);
        ev.is_composing.set(isComposing);
        Temporary::from_rooted(ev)
    }

    pub fn Constructor(global: GlobalRef,
                       type_: DOMString,
                       init: &KeyboardEventBinding::KeyboardEventInit) -> Fallible<Temporary<KeyboardEvent>> {
        let key = if let Some((key, _)) = key_from_string(init.key.as_slice()) {
            Some(key)
        } else {
            None
        };
        let event = KeyboardEvent::new(global.as_window(), type_,
                                       init.parent.parent.parent.bubbles,
                                       init.parent.parent.parent.cancelable,
                                       init.parent.parent.view.r(),
                                       init.parent.parent.detail, key,
                                       init.key.clone(), init.code.clone(), init.location,
                                       init.repeat, init.isComposing, init.parent.ctrlKey,
                                       init.parent.altKey, init.parent.shiftKey, init.parent.metaKey,
                                       None, 0);
        Ok(event)
    }

    pub fn key_properties(key: Key, mods: KeyModifiers)
        -> KeyEventProperties {
            KeyEventProperties {
                key_string: key_value(key, mods),
                code: code_value(key),
                location: key_location(key),
                char_code: key_charcode(key, mods),
                key_code: key_keycode(key),
            }
    }
}

pub trait KeyboardEventHelpers {
    fn get_key(&self) -> Option<Key>;
    fn get_key_modifiers(&self) -> KeyModifiers;
}

impl<'a> KeyboardEventHelpers for JSRef<'a, KeyboardEvent> {
    fn get_key(&self) -> Option<Key> {
        self.key.get().clone()
    }

    fn get_key_modifiers(&self) -> KeyModifiers {
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
        return result;
    }
}


// https://dvcs.w3.org/hg/dom3events/raw-file/tip/html/DOM3Events-key.html
pub fn key_value(key: Key, mods: KeyModifiers) -> &'static str {
    let shift = mods.contains(constellation_msg::SHIFT);
    match key {
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
    }
}

pub fn key_from_string(key_string: &str) -> Option<(Key, KeyModifiers)> {
    match key_string {
        " " => Some((Key::Space, KeyModifiers::empty())),
        "\"" => Some((Key::Apostrophe, SHIFT)),
        "'" => Some((Key::Apostrophe, KeyModifiers::empty())),
        "<" => Some((Key::Comma, SHIFT)),
        "," => Some((Key::Comma, KeyModifiers::empty())),
        "_" => Some((Key::Minus, SHIFT)),
        "-" => Some((Key::Minus, KeyModifiers::empty())),
        ">" => Some((Key::Period, SHIFT)),
        "." => Some((Key::Period, KeyModifiers::empty())),
        "?" => Some((Key::Slash, SHIFT)),
        "/" => Some((Key::Slash, KeyModifiers::empty())),
        "~" => Some((Key::GraveAccent, SHIFT)),
        "`" => Some((Key::GraveAccent, KeyModifiers::empty())),
        ")" => Some((Key::Num0, SHIFT)),
        "0" => Some((Key::Num0, KeyModifiers::empty())),
        "!" => Some((Key::Num1, SHIFT)),
        "1" => Some((Key::Num1, KeyModifiers::empty())),
        "@" => Some((Key::Num2, SHIFT)),
        "2" => Some((Key::Num2, KeyModifiers::empty())),
        "#" => Some((Key::Num3, SHIFT)),
        "3" => Some((Key::Num3, KeyModifiers::empty())),
        "$" => Some((Key::Num4, SHIFT)),
        "4" => Some((Key::Num4, KeyModifiers::empty())),
        "%" => Some((Key::Num5, SHIFT)),
        "5" => Some((Key::Num5, KeyModifiers::empty())),
        "^" => Some((Key::Num6, SHIFT)),
        "6" => Some((Key::Num6, KeyModifiers::empty())),
        "&" => Some((Key::Num7, SHIFT)),
        "7" => Some((Key::Num7, KeyModifiers::empty())),
        "*" => Some((Key::Num8, SHIFT)),
        "8" => Some((Key::Num8, KeyModifiers::empty())),
        "(" => Some((Key::Num9, SHIFT)),
        "9" => Some((Key::Num9, KeyModifiers::empty())),
        ":" => Some((Key::Semicolon, SHIFT)),
        ";" => Some((Key::Semicolon, KeyModifiers::empty())),
        "+" => Some((Key::Equal, SHIFT)),
        "=" => Some((Key::Equal, KeyModifiers::empty())),
        "A" => Some((Key::A, SHIFT)),
        "a" => Some((Key::A, KeyModifiers::empty())),
        "B" => Some((Key::B, SHIFT)),
        "b" => Some((Key::B, KeyModifiers::empty())),
        "C" => Some((Key::C, SHIFT)),
        "c" => Some((Key::C, KeyModifiers::empty())),
        "D" => Some((Key::D, SHIFT)),
        "d" => Some((Key::D, KeyModifiers::empty())),
        "E" => Some((Key::E, SHIFT)),
        "e" => Some((Key::E, KeyModifiers::empty())),
        "F" => Some((Key::F, SHIFT)),
        "f" => Some((Key::F, KeyModifiers::empty())),
        "G" => Some((Key::G, SHIFT)),
        "g" => Some((Key::G, KeyModifiers::empty())),
        "H" => Some((Key::H, SHIFT)),
        "h" => Some((Key::H, KeyModifiers::empty())),
        "I" => Some((Key::I, SHIFT)),
        "i" => Some((Key::I, KeyModifiers::empty())),
        "J" => Some((Key::J, SHIFT)),
        "j" => Some((Key::J, KeyModifiers::empty())),
        "K" => Some((Key::K, SHIFT)),
        "k" => Some((Key::K, KeyModifiers::empty())),
        "L" => Some((Key::L, SHIFT)),
        "l" => Some((Key::L, KeyModifiers::empty())),
        "M" => Some((Key::M, SHIFT)),
        "m" => Some((Key::M, KeyModifiers::empty())),
        "N" => Some((Key::N, SHIFT)),
        "n" => Some((Key::N, KeyModifiers::empty())),
        "O" => Some((Key::O, SHIFT)),
        "o" => Some((Key::O, KeyModifiers::empty())),
        "P" => Some((Key::P, SHIFT)),
        "p" => Some((Key::P, KeyModifiers::empty())),
        "Q" => Some((Key::Q, SHIFT)),
        "q" => Some((Key::Q, KeyModifiers::empty())),
        "R" => Some((Key::R, SHIFT)),
        "r" => Some((Key::R, KeyModifiers::empty())),
        "S" => Some((Key::S, SHIFT)),
        "s" => Some((Key::S, KeyModifiers::empty())),
        "T" => Some((Key::T, SHIFT)),
        "t" => Some((Key::T, KeyModifiers::empty())),
        "U" => Some((Key::U, SHIFT)),
        "u" => Some((Key::U, KeyModifiers::empty())),
        "V" => Some((Key::V, SHIFT)),
        "v" => Some((Key::V, KeyModifiers::empty())),
        "W" => Some((Key::W, SHIFT)),
        "w" => Some((Key::W, KeyModifiers::empty())),
        "X" => Some((Key::X, SHIFT)),
        "x" => Some((Key::X, KeyModifiers::empty())),
        "Y" => Some((Key::Y, SHIFT)),
        "y" => Some((Key::Y, KeyModifiers::empty())),
        "Z" => Some((Key::Z, SHIFT)),
        "z" => Some((Key::Z, KeyModifiers::empty())),
        "{" => Some((Key::LeftBracket, SHIFT)),
        "[" => Some((Key::LeftBracket, KeyModifiers::empty())),
        "|" => Some((Key::Backslash, SHIFT)),
        "\\" => Some((Key::Backslash, KeyModifiers::empty())),
        "}" => Some((Key::RightBracket, SHIFT)),
        "]" => Some((Key::RightBracket, KeyModifiers::empty())),
        "Unidentified" => Some((Key::World1, KeyModifiers::empty())),
        /*"Unidentified" => Some((Key::World2, KeyModifiers::empty())),*/
        "Escape" => Some((Key::Escape, KeyModifiers::empty())),
        "Enter" => Some((Key::Enter, KeyModifiers::empty())),
        "Tab" => Some((Key::Tab, KeyModifiers::empty())),
        "Backspace" => Some((Key::Backspace, KeyModifiers::empty())),
        "Insert" => Some((Key::Insert, KeyModifiers::empty())),
        "Delete" => Some((Key::Delete, KeyModifiers::empty())),
        "ArrowRight" => Some((Key::Right, KeyModifiers::empty())),
        "ArrowLeft" => Some((Key::Left, KeyModifiers::empty())),
        "ArrowDown" => Some((Key::Down, KeyModifiers::empty())),
        "ArrowUp" => Some((Key::Up, KeyModifiers::empty())),
        "PageUp" => Some((Key::PageUp, KeyModifiers::empty())),
        "PageDown" => Some((Key::PageDown, KeyModifiers::empty())),
        "Home" => Some((Key::Home, KeyModifiers::empty())),
        "End" => Some((Key::End, KeyModifiers::empty())),
        "CapsLock" => Some((Key::CapsLock, KeyModifiers::empty())),
        "ScrollLock" => Some((Key::ScrollLock, KeyModifiers::empty())),
        "NumLock" => Some((Key::NumLock, KeyModifiers::empty())),
        "PrintScreen" => Some((Key::PrintScreen, KeyModifiers::empty())),
        "Pause" => Some((Key::Pause, KeyModifiers::empty())),
        "F1" => Some((Key::F1, KeyModifiers::empty())),
        "F2" => Some((Key::F2, KeyModifiers::empty())),
        "F3" => Some((Key::F3, KeyModifiers::empty())),
        "F4" => Some((Key::F4, KeyModifiers::empty())),
        "F5" => Some((Key::F5, KeyModifiers::empty())),
        "F6" => Some((Key::F6, KeyModifiers::empty())),
        "F7" => Some((Key::F7, KeyModifiers::empty())),
        "F8" => Some((Key::F8, KeyModifiers::empty())),
        "F9" => Some((Key::F9, KeyModifiers::empty())),
        "F10" => Some((Key::F10, KeyModifiers::empty())),
        "F11" => Some((Key::F11, KeyModifiers::empty())),
        "F12" => Some((Key::F12, KeyModifiers::empty())),
        "F13" => Some((Key::F13, KeyModifiers::empty())),
        "F14" => Some((Key::F14, KeyModifiers::empty())),
        "F15" => Some((Key::F15, KeyModifiers::empty())),
        "F16" => Some((Key::F16, KeyModifiers::empty())),
        "F17" => Some((Key::F17, KeyModifiers::empty())),
        "F18" => Some((Key::F18, KeyModifiers::empty())),
        "F19" => Some((Key::F19, KeyModifiers::empty())),
        "F20" => Some((Key::F20, KeyModifiers::empty())),
        "F21" => Some((Key::F21, KeyModifiers::empty())),
        "F22" => Some((Key::F22, KeyModifiers::empty())),
        "F23" => Some((Key::F23, KeyModifiers::empty())),
        "F24" => Some((Key::F24, KeyModifiers::empty())),
        "F25" => Some((Key::F25, KeyModifiers::empty())),
        /*"0" => Some((Key::Kp0, KeyModifiers::empty())),
        "1" => Some((Key::Kp1, KeyModifiers::empty())),
        "2" => Some((Key::Kp2, KeyModifiers::empty())),
        "3" => Some((Key::Kp3, KeyModifiers::empty())),
        "4" => Some((Key::Kp4, KeyModifiers::empty())),
        "5" => Some((Key::Kp5, KeyModifiers::empty())),
        "6" => Some((Key::Kp6, KeyModifiers::empty())),
        "7" => Some((Key::Kp7, KeyModifiers::empty())),
        "8" => Some((Key::Kp8, KeyModifiers::empty())),
        "9" => Some((Key::Kp9, KeyModifiers::empty())),
        "." => Some((Key::KpDecimal, KeyModifiers::empty())),
        "/" => Some((Key::KpDivide, KeyModifiers::empty())),
        "*" => Some((Key::KpMultiply, KeyModifiers::empty())),
        "-" => Some((Key::KpSubtract, KeyModifiers::empty())),
        "+" => Some((Key::KpAdd, KeyModifiers::empty())),
        "Enter" => Some((Key::KpEnter, KeyModifiers::empty())),
        "=" => Some((Key::KpEqual, KeyModifiers::empty())),*/
        "Shift" => Some((Key::LeftShift, SHIFT)),
        "Control" => Some((Key::LeftControl, CONTROL)),
        "Alt" => Some((Key::LeftAlt, ALT)),
        "Super" => Some((Key::LeftSuper, SUPER)),
        /*"Shift" => Some((Key::RightShift, SHIFT)),
        "Control" => Some((Key::RightControl, CONTROL)),
        "Alt" => Some((Key::RightAlt, ALT)),
        "Super" => Some((Key::RightSuper, SUPER)),*/
        "ContextMenu" => Some((Key::Menu, KeyModifiers::empty())),
        _ => None
    }
}

// https://dvcs.w3.org/hg/dom3events/raw-file/tip/html/DOM3Events-code.html
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
        Key::Menu => "Menu",
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

// https://dvcs.w3.org/hg/dom3events/raw-file/tip/html/DOM3-Events.html#widl-KeyboardEvent-charCode
fn key_charcode(key: Key, mods: KeyModifiers) -> Option<u32> {
    let key_string = key_value(key, mods);
    if key_string.len() == 1 {
        Some(key_string.chars().next().unwrap() as u32)
    } else {
        None
    }
}

// https://dvcs.w3.org/hg/dom3events/raw-file/tip/html/DOM3-Events.html#legacy-key-models
fn key_keycode(key: Key) -> u32 {
    match key {
        // https://dvcs.w3.org/hg/dom3events/raw-file/tip/html/DOM3-Events.html#legacy-key-models
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

        // https://dvcs.w3.org/hg/dom3events/raw-file/tip/html/DOM3-Events.html#optionally-fixed-virtual-key-codes
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

pub struct KeyEventProperties {
    pub key_string: &'static str,
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

impl<'a> KeyboardEventMethods for JSRef<'a, KeyboardEvent> {
    fn InitKeyboardEvent(self,
                         typeArg: DOMString,
                         canBubbleArg: bool,
                         cancelableArg: bool,
                         viewArg: Option<JSRef<Window>>,
                         keyArg: DOMString,
                         locationArg: u32,
                         _modifiersListArg: DOMString,
                         repeat: bool,
                         _locale: DOMString) {
        let event: JSRef<Event> = EventCast::from_ref(self);
        if event.dispatching() {
            return;
        }

        let uievent: JSRef<UIEvent> = UIEventCast::from_ref(self);
        uievent.InitUIEvent(typeArg, canBubbleArg, cancelableArg, viewArg, 0);
        *self.key_string.borrow_mut() = keyArg;
        self.location.set(locationArg);
        self.repeat.set(repeat);
    }

    fn Key(self) -> DOMString {
        // FIXME(https://github.com/rust-lang/rust/issues/23338)
        let key_string = self.key_string.borrow();
        key_string.clone()
    }

    fn Code(self) -> DOMString {
        // FIXME(https://github.com/rust-lang/rust/issues/23338)
        let code = self.code.borrow();
        code.clone()
    }

    fn Location(self) -> u32 {
        self.location.get()
    }

    fn CtrlKey(self) -> bool {
        self.ctrl.get()
    }

    fn ShiftKey(self) -> bool {
        self.shift.get()
    }

    fn AltKey(self) -> bool {
        self.alt.get()
    }

    fn MetaKey(self) -> bool {
        self.meta.get()
    }

    fn Repeat(self) -> bool {
        self.repeat.get()
    }

    fn IsComposing(self) -> bool {
        self.is_composing.get()
    }

    // https://dvcs.w3.org/hg/dom3events/raw-file/tip/html/DOM3-Events.html#widl-KeyboardEvent-getModifierState
    fn GetModifierState(self, keyArg: DOMString) -> bool {
        match &*keyArg {
            "Ctrl" => self.CtrlKey(),
            "Alt" => self.AltKey(),
            "Shift" => self.ShiftKey(),
            "Meta" => self.MetaKey(),
            "AltGraph" | "CapsLock" | "NumLock" | "ScrollLock" | "Accel" |
            "Fn" | "FnLock" | "Hyper" | "OS" | "Symbol" | "SymbolLock" => false, //FIXME
            _ => false,
        }
    }

    fn CharCode(self) -> u32 {
        self.char_code.get().unwrap_or(0)
    }

    fn KeyCode(self) -> u32 {
        self.key_code.get()
    }

    fn Which(self) -> u32 {
        self.char_code.get().unwrap_or(self.KeyCode())
    }
}
