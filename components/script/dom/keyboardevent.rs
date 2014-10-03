/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use dom::bindings::codegen::Bindings::KeyboardEventBinding;
use dom::bindings::codegen::Bindings::KeyboardEventBinding::KeyboardEventMethods;
use dom::bindings::codegen::InheritTypes::KeyboardEventDerived;
use dom::bindings::error::Fallible;
use dom::bindings::global::GlobalRef;
use dom::bindings::js::{JSRef, Temporary};
use dom::bindings::utils::{Reflectable, Reflector/*, reflect_dom_object*/};
use dom::event::{Event, KeyboardEventTypeId};
use dom::uievent::UIEvent;
use servo_util::str::DOMString;

#[jstraceable]
#[must_root]
pub struct KeyboardEvent {
    uievent: UIEvent,
}

impl KeyboardEventDerived for Event {
    fn is_keyboardevent(&self) -> bool {
        *self.type_id() == KeyboardEventTypeId
    }
}

impl KeyboardEvent {
    pub fn Constructor(_global: &GlobalRef,
                       _type_: DOMString,
                       _init: &KeyboardEventBinding::KeyboardEventInit) -> Fallible<Temporary<KeyboardEvent>> {
        fail!()
    }
}

impl<'a> KeyboardEventMethods for JSRef<'a, KeyboardEvent> {
}

impl Reflectable for KeyboardEvent {
    fn reflector<'a>(&'a self) -> &'a Reflector {
        self.uievent.reflector()
    }
}
