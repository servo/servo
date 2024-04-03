/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use std::cell::Cell;

use dom_struct::dom_struct;
use js::rust::HandleObject;
use keyboard_types::{Key, Modifiers};

use crate::dom::bindings::cell::DomRefCell;
use crate::dom::bindings::codegen::Bindings::KeyboardEventBinding;
use crate::dom::bindings::codegen::Bindings::KeyboardEventBinding::KeyboardEventMethods;
use crate::dom::bindings::codegen::Bindings::UIEventBinding::UIEventMethods;
use crate::dom::bindings::error::Fallible;
use crate::dom::bindings::inheritance::Castable;
use crate::dom::bindings::reflector::reflect_dom_object_with_proto;
use crate::dom::bindings::root::DomRoot;
use crate::dom::bindings::str::DOMString;
use crate::dom::event::Event;
use crate::dom::uievent::UIEvent;
use crate::dom::window::Window;

#[dom_struct]
pub struct KeyboardEvent {
    uievent: UIEvent,
    key: DomRefCell<DOMString>,
    #[no_trace]
    typed_key: DomRefCell<Key>,
    code: DomRefCell<DOMString>,
    location: Cell<u32>,
    #[no_trace]
    modifiers: Cell<Modifiers>,
    repeat: Cell<bool>,
    is_composing: Cell<bool>,
    char_code: Cell<u32>,
    key_code: Cell<u32>,
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
            char_code: Cell::new(0),
            key_code: Cell::new(0),
        }
    }

    pub fn new_uninitialized(window: &Window) -> DomRoot<KeyboardEvent> {
        Self::new_uninitialized_with_proto(window, None)
    }

    fn new_uninitialized_with_proto(
        window: &Window,
        proto: Option<HandleObject>,
    ) -> DomRoot<KeyboardEvent> {
        reflect_dom_object_with_proto(Box::new(KeyboardEvent::new_inherited()), window, proto)
    }

    #[allow(clippy::too_many_arguments)]
    pub fn new(
        window: &Window,
        type_: DOMString,
        can_bubble: bool,
        cancelable: bool,
        view: Option<&Window>,
        detail: i32,
        key: Key,
        code: DOMString,
        location: u32,
        repeat: bool,
        is_composing: bool,
        modifiers: Modifiers,
        char_code: u32,
        key_code: u32,
    ) -> DomRoot<KeyboardEvent> {
        Self::new_with_proto(
            window,
            None,
            type_,
            can_bubble,
            cancelable,
            view,
            detail,
            key,
            code,
            location,
            repeat,
            is_composing,
            modifiers,
            char_code,
            key_code,
        )
    }

    #[allow(clippy::too_many_arguments)]
    fn new_with_proto(
        window: &Window,
        proto: Option<HandleObject>,
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
        char_code: u32,
        key_code: u32,
    ) -> DomRoot<KeyboardEvent> {
        let ev = KeyboardEvent::new_uninitialized_with_proto(window, proto);
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

    #[allow(non_snake_case)]
    pub fn Constructor(
        window: &Window,
        proto: Option<HandleObject>,
        type_: DOMString,
        init: &KeyboardEventBinding::KeyboardEventInit,
    ) -> Fallible<DomRoot<KeyboardEvent>> {
        let mut modifiers = Modifiers::empty();
        modifiers.set(Modifiers::CONTROL, init.parent.ctrlKey);
        modifiers.set(Modifiers::ALT, init.parent.altKey);
        modifiers.set(Modifiers::SHIFT, init.parent.shiftKey);
        modifiers.set(Modifiers::META, init.parent.metaKey);
        let event = KeyboardEvent::new_with_proto(
            window,
            proto,
            type_,
            init.parent.parent.parent.bubbles,
            init.parent.parent.parent.cancelable,
            init.parent.parent.view.as_deref(),
            init.parent.parent.detail,
            Key::Unidentified,
            init.code.clone(),
            init.location,
            init.repeat,
            init.isComposing,
            modifiers,
            0,
            0,
        );
        *event.key.borrow_mut() = init.key.clone();
        Ok(event)
    }
}

impl KeyboardEvent {
    pub fn key(&self) -> Key {
        self.typed_key.borrow().clone()
    }

    pub fn modifiers(&self) -> Modifiers {
        self.modifiers.get()
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
        self.char_code.get()
    }

    // https://w3c.github.io/uievents/#widl-KeyboardEvent-keyCode
    fn KeyCode(&self) -> u32 {
        self.key_code.get()
    }

    // https://w3c.github.io/uievents/#widl-KeyboardEvent-which
    fn Which(&self) -> u32 {
        if self.char_code.get() != 0 {
            self.char_code.get()
        } else {
            self.key_code.get()
        }
    }

    // https://dom.spec.whatwg.org/#dom-event-istrusted
    fn IsTrusted(&self) -> bool {
        self.uievent.IsTrusted()
    }
}
