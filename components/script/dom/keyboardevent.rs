/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use dom::bindings::cell::DomRefCell;
use dom::bindings::codegen::Bindings::KeyboardEventBinding;
use dom::bindings::codegen::Bindings::KeyboardEventBinding::KeyboardEventMethods;
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
use keyboard_types::{Key, Modifiers};
use std::cell::Cell;

unsafe_no_jsmanaged_fields!(Key);
unsafe_no_jsmanaged_fields!(Modifiers);

#[dom_struct]
pub struct KeyboardEvent {
    uievent: UIEvent,
    key: DomRefCell<DOMString>,
    typed_key: DomRefCell<Key>,
    code: DomRefCell<DOMString>,
    location: Cell<u32>,
    modifiers: Cell<Modifiers>,
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
            key: DomRefCell::new(DOMString::new()),
            typed_key: DomRefCell::new(Key::Unidentified),
            code: DomRefCell::new(DOMString::new()),
            location: Cell::new(0),
            modifiers: Cell::new(Modifiers::empty()),
            repeat: Cell::new(false),
            is_composing: Cell::new(false),
            char_code: Cell::new(None),
            key_code: Cell::new(0),
            printable: Cell::new(None),
        }
    }

    pub fn new_uninitialized(window: &Window) -> DomRoot<KeyboardEvent> {
        reflect_dom_object(
            Box::new(KeyboardEvent::new_inherited()),
            window,
            KeyboardEventBinding::Wrap,
        )
    }

    pub fn new(
        window: &Window,
        type_: DOMString,
        can_bubble: bool,
        cancelable: bool,
        view: Option<&Window>,
        _detail: i32,
        key: Key,
        code: DOMString,
        location: u32,
        repeat: bool,
        is_composing: bool,
        modifiers: Modifiers,
        char_code: Option<u32>,
        key_code: u32,
    ) -> DomRoot<KeyboardEvent> {
        let ev = KeyboardEvent::new_uninitialized(window);
        ev.InitKeyboardEvent(
            type_,
            can_bubble,
            cancelable,
            view,
            DOMString::from(key.to_string()),
            location,
            DOMString::new(),
            repeat,
            DOMString::new(),
        );
        *ev.typed_key.borrow_mut() = key;
        *ev.code.borrow_mut() = code;
        ev.modifiers.set(modifiers);
        ev.is_composing.set(is_composing);
        ev.char_code.set(char_code);
        ev.key_code.set(key_code);
        ev
    }

    pub fn Constructor(
        window: &Window,
        type_: DOMString,
        init: &KeyboardEventBinding::KeyboardEventInit,
    ) -> Fallible<DomRoot<KeyboardEvent>> {
        let mut modifiers = Modifiers::empty();
        modifiers.set(Modifiers::CONTROL, init.parent.ctrlKey);
        modifiers.set(Modifiers::ALT, init.parent.altKey);
        modifiers.set(Modifiers::SHIFT, init.parent.shiftKey);
        modifiers.set(Modifiers::META, init.parent.metaKey);
        let event = KeyboardEvent::new(
            window,
            type_,
            init.parent.parent.parent.bubbles,
            init.parent.parent.parent.cancelable,
            init.parent.parent.view.r(),
            init.parent.parent.detail,
            Key::Unidentified,
            init.code.clone(),
            init.location,
            init.repeat,
            init.isComposing,
            modifiers,
            None,
            0,
        );
        *event.key.borrow_mut() = init.key.clone();
        Ok(event)
    }
}

impl KeyboardEvent {
    pub fn printable(&self) -> Option<char> {
        self.printable.get()
    }

    pub fn key(&self) -> Key {
        self.typed_key.borrow().clone()
    }

    pub fn modifiers(&self) -> Modifiers {
        self.modifiers.get()
    }
}

// https://w3c.github.io/uievents/#legacy-key-models
pub fn key_keycode(key: &Key) -> u32 {
    match key {
        // https://w3c.github.io/uievents/#legacy-key-models
        Key::Backspace => 8,
        Key::Tab => 9,
        Key::Enter => 13,
        Key::Shift => 16,
        Key::Control => 17,
        Key::Alt => 18,
        Key::CapsLock => 20,
        Key::Escape => 27,
        Key::PageUp => 33,
        Key::PageDown => 34,
        Key::End => 35,
        Key::Home => 36,
        Key::ArrowLeft => 37,
        Key::ArrowUp => 38,
        Key::ArrowRight => 39,
        Key::ArrowDown => 40,
        Key::Delete => 46,
        Key::Character(ref c) if c.len() == 1 => match c.chars().next().unwrap() {
            ' ' => 32,
            //§ B.2.1.3
            '0'...'9' => c.as_bytes()[0] as u32,
            //§ B.2.1.4
            'a'...'z' => c.to_ascii_uppercase().as_bytes()[0] as u32,
            'A'...'Z' => c.as_bytes()[0] as u32,
            // https://w3c.github.io/uievents/#optionally-fixed-virtual-key-codes
            ';' | ':' => 186,
            '=' | '+' => 187,
            ',' | '<' => 188,
            '-' | '_' => 189,
            '.' | '>' => 190,
            '/' | '?' => 191,
            '`' | '~' => 192,
            '[' | '{' => 219,
            '\\' | '|' => 220,
            ']' | '}' => 221,
            '\'' | '\"' => 222,
            _ => 0,
        },

        //§ B.2.1.8
        _ => 0,
    }
}

impl KeyboardEventMethods for KeyboardEvent {
    // https://w3c.github.io/uievents/#widl-KeyboardEvent-initKeyboardEvent
    fn InitKeyboardEvent(
        &self,
        type_arg: DOMString,
        can_bubble_arg: bool,
        cancelable_arg: bool,
        view_arg: Option<&Window>,
        key_arg: DOMString,
        location_arg: u32,
        _modifiers_list_arg: DOMString,
        repeat: bool,
        _locale: DOMString,
    ) {
        if self.upcast::<Event>().dispatching() {
            return;
        }

        self.upcast::<UIEvent>()
            .InitUIEvent(type_arg, can_bubble_arg, cancelable_arg, view_arg, 0);
        *self.key.borrow_mut() = key_arg;
        self.location.set(location_arg);
        self.repeat.set(repeat);
    }

    // https://w3c.github.io/uievents/#widl-KeyboardEvent-key
    fn Key(&self) -> DOMString {
        self.key.borrow().clone()
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
        self.modifiers.get().contains(Modifiers::CONTROL)
    }

    // https://w3c.github.io/uievents/#widl-KeyboardEvent-shiftKey
    fn ShiftKey(&self) -> bool {
        self.modifiers.get().contains(Modifiers::SHIFT)
    }

    // https://w3c.github.io/uievents/#widl-KeyboardEvent-altKey
    fn AltKey(&self) -> bool {
        self.modifiers.get().contains(Modifiers::ALT)
    }

    // https://w3c.github.io/uievents/#widl-KeyboardEvent-metaKey
    fn MetaKey(&self) -> bool {
        self.modifiers.get().contains(Modifiers::META)
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
        self.modifiers.get().contains(match &*key_arg {
            "Alt" => Modifiers::ALT,
            "AltGraph" => Modifiers::ALT_GRAPH,
            "CapsLock" => Modifiers::CAPS_LOCK,
            "Control" => Modifiers::CONTROL,
            "Fn" => Modifiers::FN,
            "FnLock" => Modifiers::FN_LOCK,
            "Meta" => Modifiers::META,
            "NumLock" => Modifiers::NUM_LOCK,
            "ScrollLock" => Modifiers::SCROLL_LOCK,
            "Shift" => Modifiers::SHIFT,
            "Symbol" => Modifiers::SYMBOL,
            "SymbolLock" => Modifiers::SYMBOL_LOCK,
            _ => return false,
        })
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
