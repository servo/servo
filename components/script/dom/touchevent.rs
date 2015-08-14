/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use dom::bindings::codegen::Bindings::TouchEventBinding;
use dom::bindings::codegen::Bindings::TouchEventBinding::TouchEventMethods;
use dom::bindings::codegen::Bindings::UIEventBinding::UIEventMethods;
use dom::bindings::codegen::InheritTypes::{UIEventCast, TouchEventDerived};
use dom::bindings::global::GlobalRef;
use dom::bindings::js::{JS, MutHeap, Root};
use dom::bindings::utils::reflect_dom_object;
use dom::event::{Event, EventTypeId, EventBubbles, EventCancelable};
use dom::touchlist::TouchList;
use dom::uievent::{UIEvent, UIEventTypeId};
use dom::window::Window;
use std::cell::Cell;
use util::str::DOMString;

#[dom_struct]
pub struct TouchEvent {
    uievent: UIEvent,
    touches: MutHeap<JS<TouchList>>,
    target_touches: MutHeap<JS<TouchList>>,
    changed_touches: MutHeap<JS<TouchList>>,
    alt_key: Cell<bool>,
    meta_key: Cell<bool>,
    ctrl_key: Cell<bool>,
    shift_key: Cell<bool>,
}

impl TouchEventDerived for Event {
    fn is_touchevent(&self) -> bool {
        *self.type_id() == EventTypeId::UIEvent(UIEventTypeId::TouchEvent)
    }
}

impl TouchEvent {
    fn new_inherited(touches: &TouchList,
                     changed_touches: &TouchList,
                     target_touches: &TouchList) -> TouchEvent {
        TouchEvent {
            uievent: UIEvent::new_inherited(UIEventTypeId::TouchEvent),
            touches: MutHeap::new(JS::from_ref(touches)),
            target_touches: MutHeap::new(JS::from_ref(target_touches)),
            changed_touches: MutHeap::new(JS::from_ref(changed_touches)),
            ctrl_key: Cell::new(false),
            shift_key: Cell::new(false),
            alt_key: Cell::new(false),
            meta_key: Cell::new(false),
        }
    }

    pub fn new_uninitialized(window: &Window,
                     touches: &TouchList,
                     changed_touches: &TouchList,
                     target_touches: &TouchList) -> Root<TouchEvent> {
        reflect_dom_object(box TouchEvent::new_inherited(touches, changed_touches, target_touches),
                           GlobalRef::Window(window),
                           TouchEventBinding::Wrap)
    }

    pub fn new(window: &Window,
               type_: DOMString,
               canBubble: EventBubbles,
               cancelable: EventCancelable,
               view: Option<&Window>,
               detail: i32,
               touches: &TouchList,
               changed_touches: &TouchList,
               target_touches: &TouchList,
               ctrlKey: bool,
               altKey: bool,
               shiftKey: bool,
               metaKey: bool) -> Root<TouchEvent> {
        let ev = TouchEvent::new_uninitialized(window, touches, changed_touches, target_touches);
        {
            let uievent: &UIEvent = UIEventCast::from_ref(ev.r());
            uievent.InitUIEvent(type_,
                                canBubble == EventBubbles::Bubbles,
                                cancelable == EventCancelable::Cancelable,
                                view, detail);
        }

        ev.ctrl_key.set(ctrlKey);
        ev.alt_key.set(altKey);
        ev.shift_key.set(shiftKey);
        ev.meta_key.set(metaKey);

        ev
    }
}

impl<'a> TouchEventMethods for &'a TouchEvent {
    /// https://w3c.github.io/touch-events/#widl-TouchEvent-ctrlKey
    fn CtrlKey(&self) -> bool {
        self.ctrl_key.get()
    }

    /// https://w3c.github.io/touch-events/#widl-TouchEvent-shiftKey
    fn ShiftKey(&self) -> bool {
        self.shift_key.get()
    }

    /// https://w3c.github.io/touch-events/#widl-TouchEvent-altKey
    fn AltKey(&self) -> bool {
        self.alt_key.get()
    }

    /// https://w3c.github.io/touch-events/#widl-TouchEvent-metaKey
    fn MetaKey(&self) -> bool {
        self.meta_key.get()
    }

    /// https://w3c.github.io/touch-events/#widl-TouchEventInit-touches
    fn Touches(&self) -> Root<TouchList> {
        Root::from_rooted(self.touches.get())
    }

    /// https://w3c.github.io/touch-events/#widl-TouchEvent-targetTouches
    fn TargetTouches(&self) -> Root<TouchList> {
        Root::from_rooted(self.target_touches.get())
    }

    /// https://w3c.github.io/touch-events/#widl-TouchEvent-changedTouches
    fn ChangedTouches(&self) -> Root<TouchList> {
        Root::from_rooted(self.changed_touches.get())
    }
}
