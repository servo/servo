/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use dom::bindings::codegen::Bindings::KeyboardEventBinding;
use dom::bindings::codegen::Bindings::KeyboardEventBinding::KeyboardEventMethods;
use dom::bindings::codegen::Bindings::UIEventBinding::UIEventMethods;
use dom::bindings::codegen::InheritTypes::{UIEventCast, KeyboardEventDerived};
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

    fn new_uninitialized(window: JSRef<Window>) -> Temporary<KeyboardEvent> {
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

    pub fn key_properties(key: constellation_msg::Key) -> KeyEventProperties {
        match key {
            _ => KeyEventProperties {
                key: "".to_string(),
                code: "".to_string(),
                location: 0,
                char_code: None,
                key_code: 0,
            }
        }
    }
}

pub struct KeyEventProperties {
    pub key: DOMString,
    pub code: DOMString,
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

    fn GetModifierState(self, keyArg: DOMString) -> bool {
        match keyArg.as_slice() {
            "Ctrl" => self.CtrlKey(),
            "Alt" => self.AltKey(),
            "Shift" => self.ShiftKey(),
            "Meta" => self.MetaKey(),
            "AltGraph" | "CapsLock" | "NumLock" | "ScrollLock" => false, //FIXME
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
