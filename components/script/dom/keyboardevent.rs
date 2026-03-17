/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use std::cell::Cell;
use std::str::FromStr;

use dom_struct::dom_struct;
use js::rust::HandleObject;
use keyboard_types::{Code, Key, Modifiers, NamedKey};
use style::Atom;

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
use crate::script_runtime::CanGc;

#[dom_struct]
pub(crate) struct KeyboardEvent {
    uievent: UIEvent,
    key: DomRefCell<DOMString>,
    #[no_trace]
    typed_key: DomRefCell<Key>,
    code: DomRefCell<DOMString>,
    #[no_trace]
    original_code: DomRefCell<Option<Code>>,
    location: Cell<u32>,
    #[no_trace]
    modifiers: Cell<Modifiers>,
    repeat: Cell<bool>,
    is_composing: Cell<bool>,
    char_code: Cell<u32>,
    key_code: Cell<u32>,
}

impl KeyboardEvent {
    fn new_inherited() -> Self {
        Self {
            uievent: UIEvent::new_inherited(),
            key: Default::default(),
            typed_key: Default::default(),
            code: Default::default(),
            original_code: Default::default(),
            location: Default::default(),
            modifiers: Default::default(),
            repeat: Default::default(),
            is_composing: Default::default(),
            char_code: Default::default(),
            key_code: Default::default(),
        }
    }

    pub(crate) fn new_uninitialized(window: &Window, can_gc: CanGc) -> DomRoot<KeyboardEvent> {
        Self::new_uninitialized_with_proto(window, None, can_gc)
    }

    fn new_uninitialized_with_proto(
        window: &Window,
        proto: Option<HandleObject>,
        can_gc: CanGc,
    ) -> DomRoot<KeyboardEvent> {
        reflect_dom_object_with_proto(
            Box::new(KeyboardEvent::new_inherited()),
            window,
            proto,
            can_gc,
        )
    }

    pub(crate) fn new_with_platform_keyboard_event(
        window: &Window,
        event_type: Atom,
        keyboard_event: &keyboard_types::KeyboardEvent,
        can_gc: CanGc,
    ) -> DomRoot<KeyboardEvent> {
        Self::new_with_proto(
            window,
            None,
            event_type,
            true,         /* can_bubble */
            true,         /* cancelable */
            Some(window), /* view */
            0,            /* detail */
            keyboard_event.key.clone(),
            DOMString::from(keyboard_event.code.to_string()),
            Some(keyboard_event.code),
            keyboard_event.location as u32,
            keyboard_event.repeat,
            keyboard_event.is_composing,
            keyboard_event.modifiers,
            0, /* char_code */
            keyboard_event.key.legacy_keycode(),
            can_gc,
        )
    }

    #[expect(clippy::too_many_arguments)]
    fn new_with_proto(
        window: &Window,
        proto: Option<HandleObject>,
        event_type: Atom,
        can_bubble: bool,
        cancelable: bool,
        view: Option<&Window>,
        _detail: i32,
        key: Key,
        code: DOMString,
        original_code: Option<Code>,
        location: u32,
        repeat: bool,
        is_composing: bool,
        modifiers: Modifiers,
        char_code: u32,
        key_code: u32,
        can_gc: CanGc,
    ) -> DomRoot<KeyboardEvent> {
        let event = KeyboardEvent::new_uninitialized_with_proto(window, proto, can_gc);
        event.init_event(
            event_type,
            can_bubble,
            cancelable,
            view,
            DOMString::from(key.to_string()),
            location,
            repeat,
        );
        *event.typed_key.borrow_mut() = key;
        *event.code.borrow_mut() = code;
        *event.original_code.borrow_mut() = original_code;
        event.modifiers.set(modifiers);
        event.is_composing.set(is_composing);
        event.char_code.set(char_code);
        event.key_code.set(key_code);
        event.uievent.set_which(key_code);
        event
    }

    pub(crate) fn key(&self) -> Key {
        self.typed_key.borrow().clone()
    }

    pub(crate) fn original_code(&self) -> Option<Code> {
        *self.original_code.borrow()
    }

    pub(crate) fn modifiers(&self) -> Modifiers {
        self.modifiers.get()
    }

    /// <https://w3c.github.io/uievents/#widl-KeyboardEvent-initKeyboardEvent>
    #[expect(clippy::too_many_arguments)]
    pub(crate) fn init_event(
        &self,
        event_type: Atom,
        can_bubble_arg: bool,
        cancelable_arg: bool,
        view_arg: Option<&Window>,
        key_arg: DOMString,
        location_arg: u32,
        repeat: bool,
    ) {
        if self.upcast::<Event>().dispatching() {
            return;
        }

        self.upcast::<UIEvent>().init_event(
            event_type,
            can_bubble_arg,
            cancelable_arg,
            view_arg,
            0,
        );
        *self.key.borrow_mut() = key_arg;
        self.location.set(location_arg);
        self.repeat.set(repeat);
    }
}

impl KeyboardEventMethods<crate::DomTypeHolder> for KeyboardEvent {
    /// <https://w3c.github.io/uievents/#dom-keyboardevent-keyboardevent>
    fn Constructor(
        window: &Window,
        proto: Option<HandleObject>,
        can_gc: CanGc,
        event_type: DOMString,
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
            event_type.into(),
            init.parent.parent.parent.bubbles,
            init.parent.parent.parent.cancelable,
            init.parent.parent.view.as_deref(),
            init.parent.parent.detail,
            Key::Named(NamedKey::Unidentified),
            init.code.clone(),
            Code::from_str(&init.code.str()).ok(),
            init.location,
            init.repeat,
            init.isComposing,
            modifiers,
            init.charCode,
            init.keyCode,
            can_gc,
        );
        *event.key.borrow_mut() = init.key.clone();
        Ok(event)
    }

    /// <https://w3c.github.io/uievents/#widl-KeyboardEvent-initKeyboardEvent>
    fn InitKeyboardEvent(
        &self,
        event_type: DOMString,
        can_bubble_arg: bool,
        cancelable_arg: bool,
        view_arg: Option<&Window>,
        key_arg: DOMString,
        location_arg: u32,
        _modifiers_list_arg: DOMString,
        repeat: bool,
        _locale: DOMString,
    ) {
        self.init_event(
            event_type.into(),
            can_bubble_arg,
            cancelable_arg,
            view_arg,
            key_arg,
            location_arg,
            repeat,
        );
    }

    /// <https://w3c.github.io/uievents/#dom-keyboardevent-initkeyboardevent>
    fn Key(&self) -> DOMString {
        self.key.borrow().clone()
    }

    /// <https://w3c.github.io/uievents/#dom-keyboardevent-code>
    fn Code(&self) -> DOMString {
        self.code.borrow().clone()
    }

    /// <https://w3c.github.io/uievents/#dom-keyboardevent-location>
    fn Location(&self) -> u32 {
        self.location.get()
    }

    /// <https://w3c.github.io/uievents/#dom-keyboardevent-ctrlkey>
    fn CtrlKey(&self) -> bool {
        self.modifiers.get().contains(Modifiers::CONTROL)
    }

    /// <https://w3c.github.io/uievents/#dom-keyboardevent-shiftkey>
    fn ShiftKey(&self) -> bool {
        self.modifiers.get().contains(Modifiers::SHIFT)
    }

    /// <https://w3c.github.io/uievents/#dom-keyboardevent-altkey>
    fn AltKey(&self) -> bool {
        self.modifiers.get().contains(Modifiers::ALT)
    }

    /// <https://w3c.github.io/uievents/#dom-keyboardevent-metakey>
    fn MetaKey(&self) -> bool {
        self.modifiers.get().contains(Modifiers::META)
    }

    /// <https://w3c.github.io/uievents/#dom-keyboardevent-repeat>
    fn Repeat(&self) -> bool {
        self.repeat.get()
    }

    /// <https://w3c.github.io/uievents/#dom-keyboardevent-iscomposing>
    fn IsComposing(&self) -> bool {
        self.is_composing.get()
    }

    /// <https://w3c.github.io/uievents/#dom-keyboardevent-getmodifierstate>
    fn GetModifierState(&self, key_arg: DOMString) -> bool {
        self.modifiers.get().contains(match &*key_arg.str() {
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

    /// <https://w3c.github.io/uievents/#dom-keyboardevent-charcode>
    fn CharCode(&self) -> u32 {
        self.char_code.get()
    }

    /// <https://w3c.github.io/uievents/#dom-keyboardevent-keycode>
    fn KeyCode(&self) -> u32 {
        self.key_code.get()
    }

    /// <https://dom.spec.whatwg.org/#dom-event-istrusted>
    fn IsTrusted(&self) -> bool {
        self.uievent.IsTrusted()
    }
}
