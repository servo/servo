/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use dom::bindings::codegen::Bindings::EventBinding::EventMethods;
use dom::bindings::codegen::Bindings::UIEventBinding;
use dom::bindings::codegen::Bindings::UIEventBinding::UIEventMethods;
use dom::bindings::codegen::InheritTypes::{EventCast, UIEventDerived};
use dom::bindings::error::Fallible;
use dom::bindings::global::GlobalRef;
use dom::bindings::js::Root;
use dom::bindings::js::{JS, MutNullableHeap, RootedReference};

use dom::bindings::utils::reflect_dom_object;
use dom::event::{Event, EventTypeId, EventBubbles, EventCancelable};
use dom::window::Window;
use util::str::DOMString;

use std::cell::Cell;
use std::default::Default;

#[derive(JSTraceable, PartialEq, HeapSizeOf)]
pub enum UIEventTypeId {
    MouseEvent,
    KeyboardEvent,
    UIEvent,
}

// https://dvcs.w3.org/hg/dom3events/raw-file/tip/html/DOM3-Events.html#interface-UIEvent
#[dom_struct]
#[derive(HeapSizeOf)]
pub struct UIEvent {
    event: Event,
    view: MutNullableHeap<JS<Window>>,
    detail: Cell<i32>
}

impl UIEventDerived for Event {
    fn is_uievent(&self) -> bool {
        match *self.type_id() {
            EventTypeId::UIEvent(_) => true,
            _ => false
        }
    }
}

impl UIEvent {
    pub fn new_inherited(type_id: UIEventTypeId) -> UIEvent {
        UIEvent {
            event: Event::new_inherited(EventTypeId::UIEvent(type_id)),
            view: Default::default(),
            detail: Cell::new(0),
        }
    }

    pub fn new_uninitialized(window: &Window) -> Root<UIEvent> {
        reflect_dom_object(box UIEvent::new_inherited(UIEventTypeId::UIEvent),
                           GlobalRef::Window(window),
                           UIEventBinding::Wrap)
    }

    pub fn new(window: &Window,
               type_: DOMString,
               can_bubble: EventBubbles,
               cancelable: EventCancelable,
               view: Option<&Window>,
               detail: i32) -> Root<UIEvent> {
        let ev = UIEvent::new_uninitialized(window);
        ev.r().InitUIEvent(type_, can_bubble == EventBubbles::Bubbles,
                           cancelable == EventCancelable::Cancelable, view, detail);
        ev
    }

    pub fn Constructor(global: GlobalRef,
                       type_: DOMString,
                       init: &UIEventBinding::UIEventInit) -> Fallible<Root<UIEvent>> {
        let bubbles = if init.parent.bubbles { EventBubbles::Bubbles } else { EventBubbles::DoesNotBubble };
        let cancelable = if init.parent.cancelable {
            EventCancelable::Cancelable
        } else {
            EventCancelable::NotCancelable
        };
        let event = UIEvent::new(global.as_window(), type_,
                                 bubbles, cancelable,
                                 init.view.r(), init.detail);
        Ok(event)
    }
}

impl<'a> UIEventMethods for &'a UIEvent {
    // https://w3c.github.io/uievents/#widl-UIEvent-view
    fn GetView(self) -> Option<Root<Window>> {
        self.view.get().map(Root::from_rooted)
    }

    // https://w3c.github.io/uievents/#widl-UIEvent-detail
    fn Detail(self) -> i32 {
        self.detail.get()
    }

    // https://w3c.github.io/uievents/#widl-UIEvent-initUIEvent
    fn InitUIEvent(self,
                   type_: DOMString,
                   can_bubble: bool,
                   cancelable: bool,
                   view: Option<&Window>,
                   detail: i32) {
        let event: &Event = EventCast::from_ref(self);
        if event.dispatching() {
            return;
        }

        event.InitEvent(type_, can_bubble, cancelable);
        self.view.set(view.map(JS::from_ref));
        self.detail.set(detail);
    }
}

