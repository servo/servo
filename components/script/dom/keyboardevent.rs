/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use dom::bindings::codegen::Bindings::KeyboardEventBinding;
use dom::bindings::codegen::Bindings::KeyboardEventBinding::{KeyboardEventMethods, KeyboardEventConstants};
use dom::bindings::codegen::Bindings::UIEventBinding::UIEventMethods;
use dom::bindings::codegen::InheritTypes::{EventCast, UIEventCast, KeyboardEventDerived};
use dom::bindings::error::Fallible;
use dom::bindings::global::GlobalRef;
use dom::bindings::global;
use dom::bindings::js::{JSRef, Temporary, RootedReference};
use dom::bindings::utils::{Reflectable, Reflector, reflect_dom_object};
use dom::event::{Event, KeyboardEventTypeId};
use dom::uievent::UIEvent;
use dom::window::Window;
use servo_msg::constellation_msg;
use servo_util::str::DOMString;
use std::cell::{RefCell, Cell};

#[jstraceable]
#[must_root]
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
        *self.type_id() == KeyboardEventTypeId
    }
}

impl KeyboardEvent {
    fn new_inherited() -> KeyboardEvent {
        KeyboardEvent {
            uievent: UIEvent::new_inherited(KeyboardEventTypeId),
            key: RefCell::new("".to_string()),
            code: RefCell::new("".to_string()),
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
                           &global::Window(window),
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
        ev.deref().InitKeyboardEvent(type_, canBubble, cancelable, view, key, location,
                                     "".to_string(), repeat, "".to_string());
        *ev.code.borrow_mut() = code;
        ev.ctrl.set(ctrlKey);
        ev.alt.set(altKey);
        ev.shift.set(shiftKey);
        ev.meta.set(metaKey);
        ev.char_code.set(char_code);
        ev.key_code.set(key_code);
        ev.is_composing.set(isComposing);
        Temporary::from_rooted(*ev)
    }

    pub fn Constructor(global: &GlobalRef,
                       type_: DOMString,
                       init: &KeyboardEventBinding::KeyboardEventInit) -> Fallible<Temporary<KeyboardEvent>> {
        let event = KeyboardEvent::new(global.as_window(), type_,
                                       init.parent.parent.parent.bubbles,
                                       init.parent.parent.parent.cancelable,
                                       init.parent.parent.view.root_ref(),
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
        constellation_msg::KeySpace => " ",
        constellation_msg::KeyApostrophe if shift => "\"",
        constellation_msg::KeyApostrophe => "'",
        constellation_msg::KeyComma if shift => "<",
        constellation_msg::KeyComma => ",",
        constellation_msg::KeyMinus if shift => "_",
        constellation_msg::KeyMinus => "-",
        constellation_msg::KeyPeriod if shift => ">",
        constellation_msg::KeyPeriod => ".",
        constellation_msg::KeySlash if shift => "?",
        constellation_msg::KeySlash => "/",
        constellation_msg::Key0 if shift => ")",
        constellation_msg::Key0 => "0",
        constellation_msg::Key1 if shift => "!",
        constellation_msg::Key1 => "1",
        constellation_msg::Key2 if shift => "@",
        constellation_msg::Key2 => "2",
        constellation_msg::Key3 if shift => "#",
        constellation_msg::Key3 => "3",
        constellation_msg::Key4 if shift => "$",
        constellation_msg::Key4 => "4",
        constellation_msg::Key5 if shift => "%",
        constellation_msg::Key5 => "5",
        constellation_msg::Key6 if shift => "^",
        constellation_msg::Key6 => "6",
        constellation_msg::Key7 if shift => "&",
        constellation_msg::Key7 => "7",
        constellation_msg::Key8 if shift => "*",
        constellation_msg::Key8 => "8",
        constellation_msg::Key9 if shift => "(",
        constellation_msg::Key9 => "9",
        constellation_msg::KeySemicolon if shift => ":",
        constellation_msg::KeySemicolon => ";",
        constellation_msg::KeyEqual if shift => "+",
        constellation_msg::KeyEqual => "=",
        constellation_msg::KeyA if shift => "A",
        constellation_msg::KeyA => "a",
        constellation_msg::KeyB if shift => "B",
        constellation_msg::KeyB => "b",
        constellation_msg::KeyC if shift => "C",
        constellation_msg::KeyC => "c",
        constellation_msg::KeyD if shift => "D",
        constellation_msg::KeyD => "d",
        constellation_msg::KeyE if shift => "E",
        constellation_msg::KeyE => "e",
        constellation_msg::KeyF if shift => "F",
        constellation_msg::KeyF => "f",
        constellation_msg::KeyG if shift => "G",
        constellation_msg::KeyG => "g",
        constellation_msg::KeyH if shift => "H",
        constellation_msg::KeyH => "h",
        constellation_msg::KeyI if shift => "I",
        constellation_msg::KeyI => "i",
        constellation_msg::KeyJ if shift => "J",
        constellation_msg::KeyJ => "j",
        constellation_msg::KeyK if shift => "K",
        constellation_msg::KeyK => "k",
        constellation_msg::KeyL if shift => "L",
        constellation_msg::KeyL => "l",
        constellation_msg::KeyM if shift => "M",
        constellation_msg::KeyM => "m",
        constellation_msg::KeyN if shift => "N",
        constellation_msg::KeyN => "n",
        constellation_msg::KeyO if shift => "O",
        constellation_msg::KeyO => "o",
        constellation_msg::KeyP if shift => "P",
        constellation_msg::KeyP => "p",
        constellation_msg::KeyQ if shift => "Q",
        constellation_msg::KeyQ => "q",
        constellation_msg::KeyR if shift => "R",
        constellation_msg::KeyR => "r",
        constellation_msg::KeyS if shift => "S",
        constellation_msg::KeyS => "s",
        constellation_msg::KeyT if shift => "T",
        constellation_msg::KeyT => "t",
        constellation_msg::KeyU if shift => "U",
        constellation_msg::KeyU => "u",
        constellation_msg::KeyV if shift => "V",
        constellation_msg::KeyV => "v",
        constellation_msg::KeyW if shift => "W",
        constellation_msg::KeyW => "w",
        constellation_msg::KeyX if shift => "X",
        constellation_msg::KeyX => "x",
        constellation_msg::KeyY if shift => "Y",
        constellation_msg::KeyY => "y",
        constellation_msg::KeyZ if shift => "Z",
        constellation_msg::KeyZ => "z",
        constellation_msg::KeyLeftBracket if shift => "{",
        constellation_msg::KeyLeftBracket => "[",
        constellation_msg::KeyBackslash if shift => "|",
        constellation_msg::KeyBackslash => "\\",
        constellation_msg::KeyRightBracket if shift => "}",
        constellation_msg::KeyRightBracket => "]",
        constellation_msg::KeyGraveAccent => "Dead",
        constellation_msg::KeyWorld1 => "Unidentified",
        constellation_msg::KeyWorld2 => "Unidentified",
        constellation_msg::KeyEscape => "Escape",
        constellation_msg::KeyEnter => "Enter",
        constellation_msg::KeyTab => "Tab",
        constellation_msg::KeyBackspace => "Backspace",
        constellation_msg::KeyInsert => "Insert",
        constellation_msg::KeyDelete => "Delete",
        constellation_msg::KeyRight => "ArrowRight",
        constellation_msg::KeyLeft => "ArrowLeft",
        constellation_msg::KeyDown => "ArrowDown",
        constellation_msg::KeyUp => "ArrowUp",
        constellation_msg::KeyPageUp => "PageUp",
        constellation_msg::KeyPageDown => "PageDown",
        constellation_msg::KeyHome => "Home",
        constellation_msg::KeyEnd => "End",
        constellation_msg::KeyCapsLock => "CapsLock",
        constellation_msg::KeyScrollLock => "ScrollLock",
        constellation_msg::KeyNumLock => "NumLock",
        constellation_msg::KeyPrintScreen => "PrintScreen",
        constellation_msg::KeyPause => "Pause",
        constellation_msg::KeyF1 => "F1",
        constellation_msg::KeyF2 => "F2",
        constellation_msg::KeyF3 => "F3",
        constellation_msg::KeyF4 => "F4",
        constellation_msg::KeyF5 => "F5",
        constellation_msg::KeyF6 => "F6",
        constellation_msg::KeyF7 => "F7",
        constellation_msg::KeyF8 => "F8",
        constellation_msg::KeyF9 => "F9",
        constellation_msg::KeyF10 => "F10",
        constellation_msg::KeyF11 => "F11",
        constellation_msg::KeyF12 => "F12",
        constellation_msg::KeyF13 => "F13",
        constellation_msg::KeyF14 => "F14",
        constellation_msg::KeyF15 => "F15",
        constellation_msg::KeyF16 => "F16",
        constellation_msg::KeyF17 => "F17",
        constellation_msg::KeyF18 => "F18",
        constellation_msg::KeyF19 => "F19",
        constellation_msg::KeyF20 => "F20",
        constellation_msg::KeyF21 => "F21",
        constellation_msg::KeyF22 => "F22",
        constellation_msg::KeyF23 => "F23",
        constellation_msg::KeyF24 => "F24",
        constellation_msg::KeyF25 => "F25",
        constellation_msg::KeyKp0 => "0",
        constellation_msg::KeyKp1 => "1",
        constellation_msg::KeyKp2 => "2",
        constellation_msg::KeyKp3 => "3",
        constellation_msg::KeyKp4 => "4",
        constellation_msg::KeyKp5 => "5",
        constellation_msg::KeyKp6 => "6",
        constellation_msg::KeyKp7 => "7",
        constellation_msg::KeyKp8 => "8",
        constellation_msg::KeyKp9 => "9",
        constellation_msg::KeyKpDecimal => ".",
        constellation_msg::KeyKpDivide => "/",
        constellation_msg::KeyKpMultiply => "*",
        constellation_msg::KeyKpSubtract => "-",
        constellation_msg::KeyKpAdd => "+",
        constellation_msg::KeyKpEnter => "Enter",
        constellation_msg::KeyKpEqual => "=",
        constellation_msg::KeyLeftShift => "Shift",
        constellation_msg::KeyLeftControl => "Control",
        constellation_msg::KeyLeftAlt => "Alt",
        constellation_msg::KeyLeftSuper => "Super",
        constellation_msg::KeyRightShift => "Shift",
        constellation_msg::KeyRightControl => "Control",
        constellation_msg::KeyRightAlt => "Alt",
        constellation_msg::KeyRightSuper => "Super",
        constellation_msg::KeyMenu => "ContextMenu",
    }
}

// https://dvcs.w3.org/hg/dom3events/raw-file/tip/html/DOM3Events-code.html
fn code_value(key: constellation_msg::Key) -> &'static str {
    match key {
        constellation_msg::KeySpace => "Space",
        constellation_msg::KeyApostrophe => "Quote",
        constellation_msg::KeyComma => "Comma",
        constellation_msg::KeyMinus => "Minus",
        constellation_msg::KeyPeriod => "Period",
        constellation_msg::KeySlash => "Slash",
        constellation_msg::Key0 => "Digit0",
        constellation_msg::Key1 => "Digit1",
        constellation_msg::Key2 => "Digit2",
        constellation_msg::Key3 => "Digit3",
        constellation_msg::Key4 => "Digit4",
        constellation_msg::Key5 => "Digit5",
        constellation_msg::Key6 => "Digit6",
        constellation_msg::Key7 => "Digit7",
        constellation_msg::Key8 => "Digit8",
        constellation_msg::Key9 => "Digit9",
        constellation_msg::KeySemicolon => "Semicolon",
        constellation_msg::KeyEqual => "Equals",
        constellation_msg::KeyA => "KeyA",
        constellation_msg::KeyB => "KeyB",
        constellation_msg::KeyC => "KeyC",
        constellation_msg::KeyD => "KeyD",
        constellation_msg::KeyE => "KeyE",
        constellation_msg::KeyF => "KeyF",
        constellation_msg::KeyG => "KeyG",
        constellation_msg::KeyH => "KeyH",
        constellation_msg::KeyI => "KeyI",
        constellation_msg::KeyJ => "KeyJ",
        constellation_msg::KeyK => "KeyK",
        constellation_msg::KeyL => "KeyL",
        constellation_msg::KeyM => "KeyM",
        constellation_msg::KeyN => "KeyN",
        constellation_msg::KeyO => "KeyO",
        constellation_msg::KeyP => "KeyP",
        constellation_msg::KeyQ => "KeyQ",
        constellation_msg::KeyR => "KeyR",
        constellation_msg::KeyS => "KeyS",
        constellation_msg::KeyT => "KeyT",
        constellation_msg::KeyU => "KeyU",
        constellation_msg::KeyV => "KeyV",
        constellation_msg::KeyW => "KeyW",
        constellation_msg::KeyX => "KeyX",
        constellation_msg::KeyY => "KeyY",
        constellation_msg::KeyZ => "KeyZ",
        constellation_msg::KeyLeftBracket => "BracketLeft",
        constellation_msg::KeyBackslash => "Backslash",
        constellation_msg::KeyRightBracket => "BracketRight",

        constellation_msg::KeyGraveAccent |
        constellation_msg::KeyWorld1 |
        constellation_msg::KeyWorld2 => panic!("unknown char code for {}", key),

        constellation_msg::KeyEscape => "Escape",
        constellation_msg::KeyEnter => "Enter",
        constellation_msg::KeyTab => "Tab",
        constellation_msg::KeyBackspace => "Backspace",
        constellation_msg::KeyInsert => "Insert",
        constellation_msg::KeyDelete => "Delete",
        constellation_msg::KeyRight => "ArrowRight",
        constellation_msg::KeyLeft => "ArrowLeft",
        constellation_msg::KeyDown => "ArrowDown",
        constellation_msg::KeyUp => "ArrowUp",
        constellation_msg::KeyPageUp => "PageUp",
        constellation_msg::KeyPageDown => "PageDown",
        constellation_msg::KeyHome => "Home",
        constellation_msg::KeyEnd => "End",
        constellation_msg::KeyCapsLock => "CapsLock",
        constellation_msg::KeyScrollLock => "ScrollLock",
        constellation_msg::KeyNumLock => "NumLock",
        constellation_msg::KeyPrintScreen => "PrintScreen",
        constellation_msg::KeyPause => "Pause",
        constellation_msg::KeyF1 => "F1",
        constellation_msg::KeyF2 => "F2",
        constellation_msg::KeyF3 => "F3",
        constellation_msg::KeyF4 => "F4",
        constellation_msg::KeyF5 => "F5",
        constellation_msg::KeyF6 => "F6",
        constellation_msg::KeyF7 => "F7",
        constellation_msg::KeyF8 => "F8",
        constellation_msg::KeyF9 => "F9",
        constellation_msg::KeyF10 => "F10",
        constellation_msg::KeyF11 => "F11",
        constellation_msg::KeyF12 => "F12",
        constellation_msg::KeyF13 => "F13",
        constellation_msg::KeyF14 => "F14",
        constellation_msg::KeyF15 => "F15",
        constellation_msg::KeyF16 => "F16",
        constellation_msg::KeyF17 => "F17",
        constellation_msg::KeyF18 => "F18",
        constellation_msg::KeyF19 => "F19",
        constellation_msg::KeyF20 => "F20",
        constellation_msg::KeyF21 => "F21",
        constellation_msg::KeyF22 => "F22",
        constellation_msg::KeyF23 => "F23",
        constellation_msg::KeyF24 => "F24",
        constellation_msg::KeyF25 => "F25",
        constellation_msg::KeyKp0 => "Numpad0",
        constellation_msg::KeyKp1 => "Numpad1",
        constellation_msg::KeyKp2 => "Numpad2",
        constellation_msg::KeyKp3 => "Numpad3",
        constellation_msg::KeyKp4 => "Numpad4",
        constellation_msg::KeyKp5 => "Numpad5",
        constellation_msg::KeyKp6 => "Numpad6",
        constellation_msg::KeyKp7 => "Numpad7",
        constellation_msg::KeyKp8 => "Numpad8",
        constellation_msg::KeyKp9 => "Numpad9",
        constellation_msg::KeyKpDecimal => "NumpadDecimal",
        constellation_msg::KeyKpDivide => "NumpadDivide",
        constellation_msg::KeyKpMultiply => "NumpadMultiply",
        constellation_msg::KeyKpSubtract => "NumpadSubtract",
        constellation_msg::KeyKpAdd => "NumpadAdd",
        constellation_msg::KeyKpEnter => "NumpadEnter",
        constellation_msg::KeyKpEqual => "NumpadEquals",
        constellation_msg::KeyLeftShift | constellation_msg::KeyRightShift => "Shift",
        constellation_msg::KeyLeftControl | constellation_msg::KeyRightControl => "Control",
        constellation_msg::KeyLeftAlt | constellation_msg::KeyRightAlt => "Alt",
        constellation_msg::KeyLeftSuper | constellation_msg::KeyRightSuper => "Super",
        constellation_msg::KeyMenu => "Menu",
    }
}

fn key_location(key: constellation_msg::Key) -> u32 {
    match key {
        constellation_msg::KeyKp0 | constellation_msg::KeyKp1 | constellation_msg::KeyKp2 |
        constellation_msg::KeyKp3 | constellation_msg::KeyKp4 | constellation_msg::KeyKp5 |
        constellation_msg::KeyKp6 | constellation_msg::KeyKp7 | constellation_msg::KeyKp8 |
        constellation_msg::KeyKp9 | constellation_msg::KeyKpDecimal |
        constellation_msg::KeyKpDivide | constellation_msg::KeyKpMultiply |
        constellation_msg::KeyKpSubtract | constellation_msg::KeyKpAdd |
        constellation_msg::KeyKpEnter | constellation_msg::KeyKpEqual =>
            KeyboardEventConstants::DOM_KEY_LOCATION_NUMPAD,

        constellation_msg::KeyLeftShift | constellation_msg::KeyLeftAlt |
        constellation_msg::KeyLeftControl | constellation_msg::KeyLeftSuper =>
            KeyboardEventConstants::DOM_KEY_LOCATION_LEFT,

        constellation_msg::KeyRightShift | constellation_msg::KeyRightAlt |
        constellation_msg::KeyRightControl | constellation_msg::KeyRightSuper =>
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
        constellation_msg::KeyBackspace => 8,
        constellation_msg::KeyTab => 9,
        constellation_msg::KeyEnter => 13,
        constellation_msg::KeyLeftShift | constellation_msg::KeyRightShift => 16,
        constellation_msg::KeyLeftControl | constellation_msg::KeyRightControl => 17,
        constellation_msg::KeyLeftAlt | constellation_msg::KeyRightAlt => 18,
        constellation_msg::KeyCapsLock => 20,
        constellation_msg::KeyEscape => 27,
        constellation_msg::KeySpace => 32,
        constellation_msg::KeyPageUp => 33,
        constellation_msg::KeyPageDown => 34,
        constellation_msg::KeyEnd => 35,
        constellation_msg::KeyHome => 36,
        constellation_msg::KeyLeft => 37,
        constellation_msg::KeyUp => 38,
        constellation_msg::KeyRight => 39,
        constellation_msg::KeyDown => 40,
        constellation_msg::KeyDelete => 46,

        // https://dvcs.w3.org/hg/dom3events/raw-file/tip/html/DOM3-Events.html#optionally-fixed-virtual-key-codes
        constellation_msg::KeySemicolon => 186,
        constellation_msg::KeyEqual => 187,
        constellation_msg::KeyComma => 188,
        constellation_msg::KeyMinus => 189,
        constellation_msg::KeyPeriod => 190,
        constellation_msg::KeySlash => 191,
        constellation_msg::KeyLeftBracket => 219,
        constellation_msg::KeyBackslash => 220,
        constellation_msg::KeyRightBracket => 221,
        constellation_msg::KeyApostrophe => 222,

        //§ B.2.1.3
        constellation_msg::Key0 |
        constellation_msg::Key1 |
        constellation_msg::Key2 |
        constellation_msg::Key3 |
        constellation_msg::Key4 |
        constellation_msg::Key5 |
        constellation_msg::Key6 |
        constellation_msg::Key7 |
        constellation_msg::Key8 |
        constellation_msg::Key9 => key as u32 - constellation_msg::Key0 as u32 + '0' as u32,

        //§ B.2.1.4
        constellation_msg::KeyA |
        constellation_msg::KeyB |
        constellation_msg::KeyC |
        constellation_msg::KeyD |
        constellation_msg::KeyE |
        constellation_msg::KeyF |
        constellation_msg::KeyG |
        constellation_msg::KeyH |
        constellation_msg::KeyI |
        constellation_msg::KeyJ |
        constellation_msg::KeyK |
        constellation_msg::KeyL |
        constellation_msg::KeyM |
        constellation_msg::KeyN |
        constellation_msg::KeyO |
        constellation_msg::KeyP |
        constellation_msg::KeyQ |
        constellation_msg::KeyR |
        constellation_msg::KeyS |
        constellation_msg::KeyT |
        constellation_msg::KeyU |
        constellation_msg::KeyV |
        constellation_msg::KeyW |
        constellation_msg::KeyX |
        constellation_msg::KeyY |
        constellation_msg::KeyZ => key as u32 - constellation_msg::KeyA as u32 + 'A' as u32,

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

impl Reflectable for KeyboardEvent {
    fn reflector<'a>(&'a self) -> &'a Reflector {
        self.uievent.reflector()
    }
}
