/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use crate::dom::bindings::codegen::Bindings::FocusEventBinding;
use crate::dom::bindings::codegen::Bindings::FocusEventBinding::FocusEventMethods;
use crate::dom::bindings::codegen::Bindings::UIEventBinding::UIEventMethods;
use crate::dom::bindings::error::Fallible;
use crate::dom::bindings::inheritance::Castable;
use crate::dom::bindings::reflector::reflect_dom_object;
use crate::dom::bindings::root::{DomRoot, MutNullableDom};
use crate::dom::bindings::str::DOMString;
use crate::dom::event::{EventBubbles, EventCancelable};
use crate::dom::eventtarget::EventTarget;
use crate::dom::uievent::UIEvent;
use crate::dom::window::Window;
use dom_struct::dom_struct;
use std::default::Default;

#[dom_struct]
pub struct FocusEvent {
    uievent: UIEvent,
    related_target: MutNullableDom<EventTarget>,
}

impl FocusEvent {
    fn new_inherited() -> FocusEvent {
        FocusEvent {
            uievent: UIEvent::new_inherited(),
            related_target: Default::default(),
        }
    }

    pub fn new_uninitialized(window: &Window) -> DomRoot<FocusEvent> {
        reflect_dom_object(Box::new(FocusEvent::new_inherited()), window)
    }

    pub fn new(
        window: &Window,
        type_: DOMString,
        can_bubble: EventBubbles,
        cancelable: EventCancelable,
        view: Option<&Window>,
        detail: i32,
        related_target: Option<&EventTarget>,
    ) -> DomRoot<FocusEvent> {
        let ev = FocusEvent::new_uninitialized(window);
        ev.upcast::<UIEvent>().InitUIEvent(
            type_,
            bool::from(can_bubble),
            bool::from(cancelable),
            view,
            detail,
        );
        ev.related_target.set(related_target);
        ev
    }

    #[allow(non_snake_case)]
    pub fn Constructor(
        window: &Window,
        type_: DOMString,
        init: &FocusEventBinding::FocusEventInit,
    ) -> Fallible<DomRoot<FocusEvent>> {
        let bubbles = EventBubbles::from(init.parent.parent.bubbles);
        let cancelable = EventCancelable::from(init.parent.parent.cancelable);
        let event = FocusEvent::new(
            window,
            type_,
            bubbles,
            cancelable,
            init.parent.view.as_deref(),
            init.parent.detail,
            init.relatedTarget.as_deref(),
        );
        Ok(event)
    }
}

impl FocusEventMethods for FocusEvent {
    // https://w3c.github.io/uievents/#widl-FocusEvent-relatedTarget
    fn GetRelatedTarget(&self) -> Option<DomRoot<EventTarget>> {
        self.related_target.get()
    }

    // https://dom.spec.whatwg.org/#dom-event-istrusted
    fn IsTrusted(&self) -> bool {
        self.uievent.IsTrusted()
    }
}
