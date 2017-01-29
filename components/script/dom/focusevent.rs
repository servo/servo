/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use dom::bindings::codegen::Bindings::FocusEventBinding;
use dom::bindings::codegen::Bindings::FocusEventBinding::FocusEventMethods;
use dom::bindings::codegen::Bindings::UIEventBinding::UIEventMethods;
use dom::bindings::error::Fallible;
use dom::bindings::inheritance::Castable;
use dom::bindings::js::{MutNullableJS, Root, RootedReference};
use dom::bindings::reflector::reflect_dom_object;
use dom::bindings::str::DOMString;
use dom::event::{EventBubbles, EventCancelable};
use dom::eventtarget::EventTarget;
use dom::uievent::UIEvent;
use dom::window::Window;
use std::default::Default;

#[dom_struct]
pub struct FocusEvent {
    uievent: UIEvent,
    related_target: MutNullableJS<EventTarget>,
}

impl FocusEvent {
    fn new_inherited() -> FocusEvent {
        FocusEvent {
            uievent: UIEvent::new_inherited(),
            related_target: Default::default(),
        }
    }

    pub fn new_uninitialized(window: &Window) -> Root<FocusEvent> {
        reflect_dom_object(box FocusEvent::new_inherited(),
                           window,
                           FocusEventBinding::Wrap)
    }

    pub fn new(window: &Window,
               type_: DOMString,
               can_bubble: EventBubbles,
               cancelable: EventCancelable,
               view: Option<&Window>,
               detail: i32,
               related_target: Option<&EventTarget>) -> Root<FocusEvent> {
        let ev = FocusEvent::new_uninitialized(window);
        ev.upcast::<UIEvent>().InitUIEvent(type_,
                                           bool::from(can_bubble),
                                           bool::from(cancelable),
                                           view, detail);
        ev.related_target.set(related_target);
        ev
    }

    pub fn Constructor(window: &Window,
                       type_: DOMString,
                       init: &FocusEventBinding::FocusEventInit) -> Fallible<Root<FocusEvent>> {
        let bubbles = EventBubbles::from(init.parent.parent.bubbles);
        let cancelable = EventCancelable::from(init.parent.parent.cancelable);
        let event = FocusEvent::new(window,
                                    type_,
                                    bubbles,
                                    cancelable,
                                    init.parent.view.r(),
                                    init.parent.detail,
                                    init.relatedTarget.r());
        Ok(event)
    }
}

impl FocusEventMethods for FocusEvent {
    // https://w3c.github.io/uievents/#widl-FocusEvent-relatedTarget
    fn GetRelatedTarget(&self) -> Option<Root<EventTarget>> {
        self.related_target.get()
    }

    // https://dom.spec.whatwg.org/#dom-event-istrusted
    fn IsTrusted(&self) -> bool {
        self.uievent.IsTrusted()
    }
}
