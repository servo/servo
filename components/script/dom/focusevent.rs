/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use dom::bindings::codegen::Bindings::FocusEventBinding;
use dom::bindings::codegen::Bindings::FocusEventBinding::FocusEventMethods;
use dom::bindings::codegen::Bindings::UIEventBinding::UIEventMethods;
use dom::bindings::codegen::InheritTypes::{EventCast, FocusEventDerived, UIEventCast};
use dom::bindings::error::Fallible;
use dom::bindings::global::GlobalRef;
use dom::bindings::js::{JS, MutNullableHeap, Root, RootedReference};
use dom::bindings::utils::reflect_dom_object;
use dom::event::{Event, EventBubbles, EventCancelable, EventTypeId};
use dom::eventtarget::EventTarget;
use dom::uievent::{UIEvent, UIEventTypeId};
use dom::window::Window;
use std::default::Default;
use util::str::DOMString;

#[dom_struct]
pub struct FocusEvent {
    uievent: UIEvent,
    related_target: MutNullableHeap<JS<EventTarget>>,
}

impl FocusEventDerived for Event {
    fn is_focusevent(&self) -> bool {
        *self.type_id() == EventTypeId::UIEvent(UIEventTypeId::FocusEvent)
    }
}

impl FocusEvent {
    fn new_inherited() -> FocusEvent {
        FocusEvent {
            uievent: UIEvent::new_inherited(UIEventTypeId::FocusEvent),
            related_target: Default::default(),
        }
    }

    pub fn new_uninitialized(window: &Window) -> Root<FocusEvent> {
        reflect_dom_object(box FocusEvent::new_inherited(),
                           GlobalRef::Window(window),
                           FocusEventBinding::Wrap)
    }

    pub fn new(window: &Window,
               type_: DOMString,
               canBubble: EventBubbles,
               cancelable: EventCancelable,
               view: Option<&Window>,
               detail: i32,
               relatedTarget: Option<&EventTarget>) -> Root<FocusEvent> {
        let ev = FocusEvent::new_uninitialized(window);
        ev.r().InitFocusEvent(type_, canBubble == EventBubbles::Bubbles, cancelable == EventCancelable::Cancelable,
                              view, detail,
                              relatedTarget);
        ev
    }

    pub fn Constructor(global: GlobalRef,
                       type_: DOMString,
                       init: &FocusEventBinding::FocusEventInit) -> Fallible<Root<FocusEvent>> {
        let bubbles = if init.parent.parent.bubbles {
            EventBubbles::Bubbles
        } else {
            EventBubbles::DoesNotBubble
        };
        let cancelable = if init.parent.parent.cancelable {
            EventCancelable::Cancelable
        } else {
            EventCancelable::NotCancelable
        };
        let event = FocusEvent::new(global.as_window(), type_,
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
        self.related_target.get_rooted()
    }

    // https://w3c.github.io/uievents/#widl-FocusEvent-initFocusEvent
    fn InitFocusEvent(&self,
                      typeArg: DOMString,
                      canBubbleArg: bool,
                      cancelableArg: bool,
                      viewArg: Option<&Window>,
                      detailArg: i32,
                      relatedTargetArg: Option<&EventTarget>) {
        let event: &Event = EventCast::from_ref(self);
        if event.dispatching() {
            return;
        }

        let uievent: &UIEvent = UIEventCast::from_ref(self);
        uievent.InitUIEvent(typeArg, canBubbleArg, cancelableArg, viewArg, detailArg);
        self.related_target.set(relatedTargetArg.map(JS::from_ref));
    }
}
