/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use dom::bindings::codegen::Bindings::KeyboardEventBinding;
use dom::bindings::codegen::Bindings::KeyboardEventBinding::{KeyboardEventMethods, KeyboardEventConstants};
use dom::bindings::codegen::Bindings::UIEventBinding::UIEventMethods;
use dom::bindings::codegen::InheritTypes::{EventCast, UIEventCast, KeyboardEventDerived};
use dom::bindings::error::Fallible;
use dom::bindings::global::GlobalRef;
use dom::bindings::js::{JSRef, Temporary, RootedReference};
use dom::bindings::utils::{Reflectable, reflect_dom_object};
use dom::event::{Event, EventTypeId};
use dom::uievent::UIEvent;
use dom::window::Window;
use msg::constellation_msg;
use util::str::DOMString;

use std::borrow::ToOwned;
use std::cell::{RefCell, Cell};

#[dom_struct]
pub struct KeyboardEvent {
    uievent: UIEvent,
    key: RefCell<DOMString>,
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
            key: RefCell::new("".to_owned()),
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
               key: DOMString,
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
        ev.r().InitKeyboardEvent(type_, canBubble, cancelable, view, key, location,
                                 "".to_owned(), repeat, "".to_owned());
        *ev.r().code.borrow_mut() = code;
        ev.r().ctrl.set(ctrlKey);
        ev.r().alt.set(altKey);
        ev.r().shift.set(shiftKey);
        ev.r().meta.set(metaKey);
        ev.r().char_code.set(char_code);
        ev.r().key_code.set(key_code);
        ev.r().is_composing.set(isComposing);
        Temporary::from_rooted(ev.r())
    }

    pub fn Constructor(global: GlobalRef,
                       type_: DOMString,
                       init: &KeyboardEventBinding::KeyboardEventInit) -> Fallible<Temporary<KeyboardEvent>> {
        let event = KeyboardEvent::new(global.as_window(), type_,
                                       init.parent.parent.parent.bubbles,
                                       init.parent.parent.parent.cancelable,
                                       init.parent.parent.view.r(),
                                       init.parent.parent.detail,
                                       init.key.clone(), init.code.clone(), init.location,
                                       init.repeat, init.isComposing, init.parent.ctrlKey,
                                       init.parent.altKey, init.parent.shiftKey, init.parent.metaKey,
                                       None, 0);
        Ok(event)
    }

    pub fn key_properties(key: constellation_msg::Key, mods: constellation_msg::KeyModifiers)
        -> KeyEventProperties {
            KeyEventProperties {
                key: key_value(key, mods),
                code: code_value(key),
                location: key_location(key),
                char_code: key_charcode(key, mods),
                key_code: key_keycode(key),
            }
    }
}

// https://dvcs.w3.org/hg/dom3events/raw-file/tip/html/DOM3Events-key.html
fn key_value(key: constellation_msg::Key, mods: constellation_msg::KeyModifiers) -> &'static str {
    let shift = mods.contains(constellation_msg::SHIFT);
    match key {
        constellation_msg::Key::Space => " ",
        constellation_msg::Key::Apostrophe if shift => "\"",
        constellation_msg::Key::Apostrophe => "'",
        constellation_msg::Key::Comma if shift => "<",
        constellation_msg::Key::Comma => ",",
        constellation_msg::Key::Minus if shift => "_",
        constellation_msg::Key::Minus => "-",
        constellation_msg::Key::Period if shift => ">",
        constellation_msg::Key::Period => ".",
        constellation_msg::Key::Slash if shift => "?",
        constellation_msg::Key::Slash => "/",
        constellation_msg::Key::Num0 if shift => ")",
        constellation_msg::Key::Num0 => "0",
        constellation_msg::Key::Num1 if shift => "!",
        constellation_msg::Key::Num1 => "1",
        constellation_msg::Key::Num2 if shift => "@",
        constellation_msg::Key::Num2 => "2",
        constellation_msg::Key::Num3 if shift => "#",
        constellation_msg::Key::Num3 => "3",
        constellation_msg::Key::Num4 if shift => "$",
        constellation_msg::Key::Num4 => "4",
        constellation_msg::Key::Num5 if shift => "%",
        constellation_msg::Key::Num5 => "5",
        constellation_msg::Key::Num6 if shift => "^",
        constellation_msg::Key::Num6 => "6",
        constellation_msg::Key::Num7 if shift => "&",
        constellation_msg::Key::Num7 => "7",
        constellation_msg::Key::Num8 if shift => "*",
        constellation_msg::Key::Num8 => "8",
        constellation_msg::Key::Num9 if shift => "(",
        constellation_msg::Key::Num9 => "9",
        constellation_msg::Key::Semicolon if shift => ":",
        constellation_msg::Key::Semicolon => ";",
        constellation_msg::Key::Equal if shift => "+",
        constellation_msg::Key::Equal => "=",
        constellation_msg::Key::A if shift => "A",
        constellation_msg::Key::A => "a",
        constellation_msg::Key::B if shift => "B",
        constellation_msg::Key::B => "b",
        constellation_msg::Key::C if shift => "C",
        constellation_msg::Key::C => "c",
        constellation_msg::Key::D if shift => "D",
        constellation_msg::Key::D => "d",
        constellation_msg::Key::E if shift => "E",
        constellation_msg::Key::E => "e",
        constellation_msg::Key::F if shift => "F",
        constellation_msg::Key::F => "f",
        constellation_msg::Key::G if shift => "G",
        constellation_msg::Key::G => "g",
        constellation_msg::Key::H if shift => "H",
        constellation_msg::Key::H => "h",
        constellation_msg::Key::I if shift => "I",
        constellation_msg::Key::I => "i",
        constellation_msg::Key::J if shift => "J",
        constellation_msg::Key::J => "j",
        constellation_msg::Key::K if shift => "K",
        constellation_msg::Key::K => "k",
        constellation_msg::Key::L if shift => "L",
        constellation_msg::Key::L => "l",
        constellation_msg::Key::M if shift => "M",
        constellation_msg::Key::M => "m",
        constellation_msg::Key::N if shift => "N",
        constellation_msg::Key::N => "n",
        constellation_msg::Key::O if shift => "O",
        constellation_msg::Key::O => "o",
        constellation_msg::Key::P if shift => "P",
        constellation_msg::Key::P => "p",
        constellation_msg::Key::Q if shift => "Q",
        constellation_msg::Key::Q => "q",
        constellation_msg::Key::R if shift => "R",
        constellation_msg::Key::R => "r",
        constellation_msg::Key::S if shift => "S",
        constellation_msg::Key::S => "s",
        constellation_msg::Key::T if shift => "T",
        constellation_msg::Key::T => "t",
        constellation_msg::Key::U if shift => "U",
        constellation_msg::Key::U => "u",
        constellation_msg::Key::V if shift => "V",
        constellation_msg::Key::V => "v",
        constellation_msg::Key::W if shift => "W",
        constellation_msg::Key::W => "w",
        constellation_msg::Key::X if shift => "X",
        constellation_msg::Key::X => "x",
        constellation_msg::Key::Y if shift => "Y",
        constellation_msg::Key::Y => "y",
        constellation_msg::Key::Z if shift => "Z",
        constellation_msg::Key::Z => "z",
        constellation_msg::Key::LeftBracket if shift => "{",
        constellation_msg::Key::LeftBracket => "[",
        constellation_msg::Key::Backslash if shift => "|",
        constellation_msg::Key::Backslash => "\\",
        constellation_msg::Key::RightBracket if shift => "}",
        constellation_msg::Key::RightBracket => "]",
        constellation_msg::Key::GraveAccent => "Dead",
        constellation_msg::Key::World1 => "Unidentified",
        constellation_msg::Key::World2 => "Unidentified",
        constellation_msg::Key::Escape => "Escape",
        constellation_msg::Key::Enter => "Enter",
        constellation_msg::Key::Tab => "Tab",
        constellation_msg::Key::Backspace => "Backspace",
        constellation_msg::Key::Insert => "Insert",
        constellation_msg::Key::Delete => "Delete",
        constellation_msg::Key::Right => "ArrowRight",
        constellation_msg::Key::Left => "ArrowLeft",
        constellation_msg::Key::Down => "ArrowDown",
        constellation_msg::Key::Up => "ArrowUp",
        constellation_msg::Key::PageUp => "PageUp",
        constellation_msg::Key::PageDown => "PageDown",
        constellation_msg::Key::Home => "Home",
        constellation_msg::Key::End => "End",
        constellation_msg::Key::CapsLock => "CapsLock",
        constellation_msg::Key::ScrollLock => "ScrollLock",
        constellation_msg::Key::NumLock => "NumLock",
        constellation_msg::Key::PrintScreen => "PrintScreen",
        constellation_msg::Key::Pause => "Pause",
        constellation_msg::Key::F1 => "F1",
        constellation_msg::Key::F2 => "F2",
        constellation_msg::Key::F3 => "F3",
        constellation_msg::Key::F4 => "F4",
        constellation_msg::Key::F5 => "F5",
        constellation_msg::Key::F6 => "F6",
        constellation_msg::Key::F7 => "F7",
        constellation_msg::Key::F8 => "F8",
        constellation_msg::Key::F9 => "F9",
        constellation_msg::Key::F10 => "F10",
        constellation_msg::Key::F11 => "F11",
        constellation_msg::Key::F12 => "F12",
        constellation_msg::Key::F13 => "F13",
        constellation_msg::Key::F14 => "F14",
        constellation_msg::Key::F15 => "F15",
        constellation_msg::Key::F16 => "F16",
        constellation_msg::Key::F17 => "F17",
        constellation_msg::Key::F18 => "F18",
        constellation_msg::Key::F19 => "F19",
        constellation_msg::Key::F20 => "F20",
        constellation_msg::Key::F21 => "F21",
        constellation_msg::Key::F22 => "F22",
        constellation_msg::Key::F23 => "F23",
        constellation_msg::Key::F24 => "F24",
        constellation_msg::Key::F25 => "F25",
        constellation_msg::Key::Kp0 => "0",
        constellation_msg::Key::Kp1 => "1",
        constellation_msg::Key::Kp2 => "2",
        constellation_msg::Key::Kp3 => "3",
        constellation_msg::Key::Kp4 => "4",
        constellation_msg::Key::Kp5 => "5",
        constellation_msg::Key::Kp6 => "6",
        constellation_msg::Key::Kp7 => "7",
        constellation_msg::Key::Kp8 => "8",
        constellation_msg::Key::Kp9 => "9",
        constellation_msg::Key::KpDecimal => ".",
        constellation_msg::Key::KpDivide => "/",
        constellation_msg::Key::KpMultiply => "*",
        constellation_msg::Key::KpSubtract => "-",
        constellation_msg::Key::KpAdd => "+",
        constellation_msg::Key::KpEnter => "Enter",
        constellation_msg::Key::KpEqual => "=",
        constellation_msg::Key::LeftShift => "Shift",
        constellation_msg::Key::LeftControl => "Control",
        constellation_msg::Key::LeftAlt => "Alt",
        constellation_msg::Key::LeftSuper => "Super",
        constellation_msg::Key::RightShift => "Shift",
        constellation_msg::Key::RightControl => "Control",
        constellation_msg::Key::RightAlt => "Alt",
        constellation_msg::Key::RightSuper => "Super",
        constellation_msg::Key::Menu => "ContextMenu",
    }
}

// https://dvcs.w3.org/hg/dom3events/raw-file/tip/html/DOM3Events-code.html
fn code_value(key: constellation_msg::Key) -> &'static str {
    match key {
        constellation_msg::Key::Space => "Space",
        constellation_msg::Key::Apostrophe => "Quote",
        constellation_msg::Key::Comma => "Comma",
        constellation_msg::Key::Minus => "Minus",
        constellation_msg::Key::Period => "Period",
        constellation_msg::Key::Slash => "Slash",
        constellation_msg::Key::Num0 => "Digit0",
        constellation_msg::Key::Num1 => "Digit1",
        constellation_msg::Key::Num2 => "Digit2",
        constellation_msg::Key::Num3 => "Digit3",
        constellation_msg::Key::Num4 => "Digit4",
        constellation_msg::Key::Num5 => "Digit5",
        constellation_msg::Key::Num6 => "Digit6",
        constellation_msg::Key::Num7 => "Digit7",
        constellation_msg::Key::Num8 => "Digit8",
        constellation_msg::Key::Num9 => "Digit9",
        constellation_msg::Key::Semicolon => "Semicolon",
        constellation_msg::Key::Equal => "Equals",
        constellation_msg::Key::A => "Key::A",
        constellation_msg::Key::B => "Key::B",
        constellation_msg::Key::C => "Key::C",
        constellation_msg::Key::D => "Key::D",
        constellation_msg::Key::E => "Key::E",
        constellation_msg::Key::F => "Key::F",
        constellation_msg::Key::G => "Key::G",
        constellation_msg::Key::H => "Key::H",
        constellation_msg::Key::I => "Key::I",
        constellation_msg::Key::J => "Key::J",
        constellation_msg::Key::K => "Key::K",
        constellation_msg::Key::L => "Key::L",
        constellation_msg::Key::M => "Key::M",
        constellation_msg::Key::N => "Key::N",
        constellation_msg::Key::O => "Key::O",
        constellation_msg::Key::P => "Key::P",
        constellation_msg::Key::Q => "Key::Q",
        constellation_msg::Key::R => "Key::R",
        constellation_msg::Key::S => "Key::S",
        constellation_msg::Key::T => "Key::T",
        constellation_msg::Key::U => "Key::U",
        constellation_msg::Key::V => "Key::V",
        constellation_msg::Key::W => "Key::W",
        constellation_msg::Key::X => "Key::X",
        constellation_msg::Key::Y => "Key::Y",
        constellation_msg::Key::Z => "Key::Z",
        constellation_msg::Key::LeftBracket => "BracketLeft",
        constellation_msg::Key::Backslash => "Backslash",
        constellation_msg::Key::RightBracket => "BracketRight",

        constellation_msg::Key::GraveAccent |
        constellation_msg::Key::World1 |
        constellation_msg::Key::World2 => panic!("unknown char code for {:?}", key),

        constellation_msg::Key::Escape => "Escape",
        constellation_msg::Key::Enter => "Enter",
        constellation_msg::Key::Tab => "Tab",
        constellation_msg::Key::Backspace => "Backspace",
        constellation_msg::Key::Insert => "Insert",
        constellation_msg::Key::Delete => "Delete",
        constellation_msg::Key::Right => "ArrowRight",
        constellation_msg::Key::Left => "ArrowLeft",
        constellation_msg::Key::Down => "ArrowDown",
        constellation_msg::Key::Up => "ArrowUp",
        constellation_msg::Key::PageUp => "PageUp",
        constellation_msg::Key::PageDown => "PageDown",
        constellation_msg::Key::Home => "Home",
        constellation_msg::Key::End => "End",
        constellation_msg::Key::CapsLock => "CapsLock",
        constellation_msg::Key::ScrollLock => "ScrollLock",
        constellation_msg::Key::NumLock => "NumLock",
        constellation_msg::Key::PrintScreen => "PrintScreen",
        constellation_msg::Key::Pause => "Pause",
        constellation_msg::Key::F1 => "F1",
        constellation_msg::Key::F2 => "F2",
        constellation_msg::Key::F3 => "F3",
        constellation_msg::Key::F4 => "F4",
        constellation_msg::Key::F5 => "F5",
        constellation_msg::Key::F6 => "F6",
        constellation_msg::Key::F7 => "F7",
        constellation_msg::Key::F8 => "F8",
        constellation_msg::Key::F9 => "F9",
        constellation_msg::Key::F10 => "F10",
        constellation_msg::Key::F11 => "F11",
        constellation_msg::Key::F12 => "F12",
        constellation_msg::Key::F13 => "F13",
        constellation_msg::Key::F14 => "F14",
        constellation_msg::Key::F15 => "F15",
        constellation_msg::Key::F16 => "F16",
        constellation_msg::Key::F17 => "F17",
        constellation_msg::Key::F18 => "F18",
        constellation_msg::Key::F19 => "F19",
        constellation_msg::Key::F20 => "F20",
        constellation_msg::Key::F21 => "F21",
        constellation_msg::Key::F22 => "F22",
        constellation_msg::Key::F23 => "F23",
        constellation_msg::Key::F24 => "F24",
        constellation_msg::Key::F25 => "F25",
        constellation_msg::Key::Kp0 => "Numpad0",
        constellation_msg::Key::Kp1 => "Numpad1",
        constellation_msg::Key::Kp2 => "Numpad2",
        constellation_msg::Key::Kp3 => "Numpad3",
        constellation_msg::Key::Kp4 => "Numpad4",
        constellation_msg::Key::Kp5 => "Numpad5",
        constellation_msg::Key::Kp6 => "Numpad6",
        constellation_msg::Key::Kp7 => "Numpad7",
        constellation_msg::Key::Kp8 => "Numpad8",
        constellation_msg::Key::Kp9 => "Numpad9",
        constellation_msg::Key::KpDecimal => "NumpadDecimal",
        constellation_msg::Key::KpDivide => "NumpadDivide",
        constellation_msg::Key::KpMultiply => "NumpadMultiply",
        constellation_msg::Key::KpSubtract => "NumpadSubtract",
        constellation_msg::Key::KpAdd => "NumpadAdd",
        constellation_msg::Key::KpEnter => "NumpadEnter",
        constellation_msg::Key::KpEqual => "NumpadEquals",
        constellation_msg::Key::LeftShift | constellation_msg::Key::RightShift => "Shift",
        constellation_msg::Key::LeftControl | constellation_msg::Key::RightControl => "Control",
        constellation_msg::Key::LeftAlt | constellation_msg::Key::RightAlt => "Alt",
        constellation_msg::Key::LeftSuper | constellation_msg::Key::RightSuper => "Super",
        constellation_msg::Key::Menu => "Menu",
    }
}

fn key_location(key: constellation_msg::Key) -> u32 {
    match key {
        constellation_msg::Key::Kp0 | constellation_msg::Key::Kp1 | constellation_msg::Key::Kp2 |
        constellation_msg::Key::Kp3 | constellation_msg::Key::Kp4 | constellation_msg::Key::Kp5 |
        constellation_msg::Key::Kp6 | constellation_msg::Key::Kp7 | constellation_msg::Key::Kp8 |
        constellation_msg::Key::Kp9 | constellation_msg::Key::KpDecimal |
        constellation_msg::Key::KpDivide | constellation_msg::Key::KpMultiply |
        constellation_msg::Key::KpSubtract | constellation_msg::Key::KpAdd |
        constellation_msg::Key::KpEnter | constellation_msg::Key::KpEqual =>
            KeyboardEventConstants::DOM_KEY_LOCATION_NUMPAD,

        constellation_msg::Key::LeftShift | constellation_msg::Key::LeftAlt |
        constellation_msg::Key::LeftControl | constellation_msg::Key::LeftSuper =>
            KeyboardEventConstants::DOM_KEY_LOCATION_LEFT,

        constellation_msg::Key::RightShift | constellation_msg::Key::RightAlt |
        constellation_msg::Key::RightControl | constellation_msg::Key::RightSuper =>
            KeyboardEventConstants::DOM_KEY_LOCATION_RIGHT,

        _ => KeyboardEventConstants::DOM_KEY_LOCATION_STANDARD,
    }
}

// https://dvcs.w3.org/hg/dom3events/raw-file/tip/html/DOM3-Events.html#widl-KeyboardEvent-charCode
fn key_charcode(key: constellation_msg::Key, mods: constellation_msg::KeyModifiers) -> Option<u32> {
    let key = key_value(key, mods);
    if key.len() == 1 {
        Some(key.char_at(0) as u32)
    } else {
        None
    }
}

// https://dvcs.w3.org/hg/dom3events/raw-file/tip/html/DOM3-Events.html#legacy-key-models
fn key_keycode(key: constellation_msg::Key) -> u32 {
    match key {
        // https://dvcs.w3.org/hg/dom3events/raw-file/tip/html/DOM3-Events.html#legacy-key-models
        constellation_msg::Key::Backspace => 8,
        constellation_msg::Key::Tab => 9,
        constellation_msg::Key::Enter => 13,
        constellation_msg::Key::LeftShift | constellation_msg::Key::RightShift => 16,
        constellation_msg::Key::LeftControl | constellation_msg::Key::RightControl => 17,
        constellation_msg::Key::LeftAlt | constellation_msg::Key::RightAlt => 18,
        constellation_msg::Key::CapsLock => 20,
        constellation_msg::Key::Escape => 27,
        constellation_msg::Key::Space => 32,
        constellation_msg::Key::PageUp => 33,
        constellation_msg::Key::PageDown => 34,
        constellation_msg::Key::End => 35,
        constellation_msg::Key::Home => 36,
        constellation_msg::Key::Left => 37,
        constellation_msg::Key::Up => 38,
        constellation_msg::Key::Right => 39,
        constellation_msg::Key::Down => 40,
        constellation_msg::Key::Delete => 46,

        // https://dvcs.w3.org/hg/dom3events/raw-file/tip/html/DOM3-Events.html#optionally-fixed-virtual-key-codes
        constellation_msg::Key::Semicolon => 186,
        constellation_msg::Key::Equal => 187,
        constellation_msg::Key::Comma => 188,
        constellation_msg::Key::Minus => 189,
        constellation_msg::Key::Period => 190,
        constellation_msg::Key::Slash => 191,
        constellation_msg::Key::LeftBracket => 219,
        constellation_msg::Key::Backslash => 220,
        constellation_msg::Key::RightBracket => 221,
        constellation_msg::Key::Apostrophe => 222,

        //§ B.2.1.3
        constellation_msg::Key::Num0 |
        constellation_msg::Key::Num1 |
        constellation_msg::Key::Num2 |
        constellation_msg::Key::Num3 |
        constellation_msg::Key::Num4 |
        constellation_msg::Key::Num5 |
        constellation_msg::Key::Num6 |
        constellation_msg::Key::Num7 |
        constellation_msg::Key::Num8 |
        constellation_msg::Key::Num9 => key as u32 - constellation_msg::Key::Num0 as u32 + '0' as u32,

        //§ B.2.1.4
        constellation_msg::Key::A |
        constellation_msg::Key::B |
        constellation_msg::Key::C |
        constellation_msg::Key::D |
        constellation_msg::Key::E |
        constellation_msg::Key::F |
        constellation_msg::Key::G |
        constellation_msg::Key::H |
        constellation_msg::Key::I |
        constellation_msg::Key::J |
        constellation_msg::Key::K |
        constellation_msg::Key::L |
        constellation_msg::Key::M |
        constellation_msg::Key::N |
        constellation_msg::Key::O |
        constellation_msg::Key::P |
        constellation_msg::Key::Q |
        constellation_msg::Key::R |
        constellation_msg::Key::S |
        constellation_msg::Key::T |
        constellation_msg::Key::U |
        constellation_msg::Key::V |
        constellation_msg::Key::W |
        constellation_msg::Key::X |
        constellation_msg::Key::Y |
        constellation_msg::Key::Z => key as u32 - constellation_msg::Key::A as u32 + 'A' as u32,

        //§ B.2.1.8
        _ => 0
    }
}

pub struct KeyEventProperties {
    pub key: &'static str,
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
        *self.key.borrow_mut() = keyArg;
        self.location.set(locationArg);
        self.repeat.set(repeat);
    }

    fn Key(self) -> DOMString {
        self.key.borrow().clone()
    }

    fn Code(self) -> DOMString {
        self.code.borrow().clone()
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
        match keyArg.as_slice() {
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
