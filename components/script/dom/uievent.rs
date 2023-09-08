/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use std::cell::Cell;
use std::default::Default;

use dom_struct::dom_struct;
use js::rust::HandleObject;
use servo_atoms::Atom;

use crate::dom::bindings::codegen::Bindings::EventBinding::EventMethods;
use crate::dom::bindings::codegen::Bindings::UIEventBinding;
use crate::dom::bindings::codegen::Bindings::UIEventBinding::UIEventMethods;
use crate::dom::bindings::error::Fallible;
use crate::dom::bindings::inheritance::Castable;
use crate::dom::bindings::reflector::reflect_dom_object_with_proto;
use crate::dom::bindings::root::{DomRoot, MutNullableDom};
use crate::dom::bindings::str::DOMString;
use crate::dom::event::{Event, EventBubbles, EventCancelable};
use crate::dom::window::Window;

// https://w3c.github.io/uievents/#interface-uievent
#[dom_struct]
pub struct UIEvent {
    event: Event,
    view: MutNullableDom<Window>,
    detail: Cell<i32>,
}

impl UIEvent {
    pub fn new_inherited() -> UIEvent {
        UIEvent {
            event: Event::new_inherited(),
            view: Default::default(),
            detail: Cell::new(0),
        }
    }

    pub fn new_uninitialized(window: &Window) -> DomRoot<UIEvent> {
        Self::new_uninitialized_with_proto(window, None)
    }

    fn new_uninitialized_with_proto(
        window: &Window,
        proto: Option<HandleObject>,
    ) -> DomRoot<UIEvent> {
        reflect_dom_object_with_proto(Box::new(UIEvent::new_inherited()), window, proto)
    }

    pub fn new(
        window: &Window,
        type_: DOMString,
        can_bubble: EventBubbles,
        cancelable: EventCancelable,
        view: Option<&Window>,
        detail: i32,
    ) -> DomRoot<UIEvent> {
        Self::new_with_proto(window, None, type_, can_bubble, cancelable, view, detail)
    }

    fn new_with_proto(
        window: &Window,
        proto: Option<HandleObject>,
        type_: DOMString,
        can_bubble: EventBubbles,
        cancelable: EventCancelable,
        view: Option<&Window>,
        detail: i32,
    ) -> DomRoot<UIEvent> {
        let ev = UIEvent::new_uninitialized_with_proto(window, proto);
        ev.InitUIEvent(
            type_,
            bool::from(can_bubble),
            bool::from(cancelable),
            view,
            detail,
        );
        ev
    }

    #[allow(non_snake_case)]
    pub fn Constructor(
        window: &Window,
        proto: Option<HandleObject>,
        type_: DOMString,
        init: &UIEventBinding::UIEventInit,
    ) -> Fallible<DomRoot<UIEvent>> {
        let bubbles = EventBubbles::from(init.parent.bubbles);
        let cancelable = EventCancelable::from(init.parent.cancelable);
        let event = UIEvent::new_with_proto(
            window,
            proto,
            type_,
            bubbles,
            cancelable,
            init.view.as_deref(),
            init.detail,
        );
        Ok(event)
    }
}

impl UIEventMethods for UIEvent {
    // https://w3c.github.io/uievents/#widl-UIEvent-view
    fn GetView(&self) -> Option<DomRoot<Window>> {
        self.view.get()
    }

    // https://w3c.github.io/uievents/#widl-UIEvent-detail
    fn Detail(&self) -> i32 {
        self.detail.get()
    }

    // https://w3c.github.io/uievents/#widl-UIEvent-initUIEvent
    fn InitUIEvent(
        &self,
        type_: DOMString,
        can_bubble: bool,
        cancelable: bool,
        view: Option<&Window>,
        detail: i32,
    ) {
        let event = self.upcast::<Event>();
        if event.dispatching() {
            return;
        }

        event.init_event(Atom::from(type_), can_bubble, cancelable);
        self.view.set(view);
        self.detail.set(detail);
    }

    // https://dom.spec.whatwg.org/#dom-event-istrusted
    fn IsTrusted(&self) -> bool {
        self.event.IsTrusted()
    }
}
