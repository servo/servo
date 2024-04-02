/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use std::default::Default;

use dom_struct::dom_struct;
use js::rust::HandleObject;

use crate::dom::bindings::codegen::Bindings::FocusEventBinding;
use crate::dom::bindings::codegen::Bindings::FocusEventBinding::FocusEventMethods;
use crate::dom::bindings::codegen::Bindings::UIEventBinding::UIEventMethods;
use crate::dom::bindings::error::Fallible;
use crate::dom::bindings::inheritance::Castable;
use crate::dom::bindings::reflector::reflect_dom_object_with_proto;
use crate::dom::bindings::root::{DomRoot, MutNullableDom};
use crate::dom::bindings::str::DOMString;
use crate::dom::event::{EventBubbles, EventCancelable};
use crate::dom::eventtarget::EventTarget;
use crate::dom::uievent::UIEvent;
use crate::dom::window::Window;

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
        Self::new_uninitialized_with_proto(window, None)
    }

    pub fn new_uninitialized_with_proto(
        window: &Window,
        proto: Option<HandleObject>,
    ) -> DomRoot<FocusEvent> {
        reflect_dom_object_with_proto(Box::new(FocusEvent::new_inherited()), window, proto)
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
        Self::new_with_proto(
            window,
            None,
            type_,
            can_bubble,
            cancelable,
            view,
            detail,
            related_target,
        )
    }

    #[allow(clippy::too_many_arguments)]
    fn new_with_proto(
        window: &Window,
        proto: Option<HandleObject>,
        type_: DOMString,
        can_bubble: EventBubbles,
        cancelable: EventCancelable,
        view: Option<&Window>,
        detail: i32,
        related_target: Option<&EventTarget>,
    ) -> DomRoot<FocusEvent> {
        let ev = FocusEvent::new_uninitialized_with_proto(window, proto);
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
        proto: Option<HandleObject>,
        type_: DOMString,
        init: &FocusEventBinding::FocusEventInit,
    ) -> Fallible<DomRoot<FocusEvent>> {
        let bubbles = EventBubbles::from(init.parent.parent.bubbles);
        let cancelable = EventCancelable::from(init.parent.parent.cancelable);
        let event = FocusEvent::new_with_proto(
            window,
            proto,
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
