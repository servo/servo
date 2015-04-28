/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use dom::bindings::codegen::Bindings::EventBinding::EventMethods;
use dom::bindings::codegen::Bindings::UIEventBinding;
use dom::bindings::codegen::Bindings::UIEventBinding::UIEventMethods;
use dom::bindings::codegen::InheritTypes::{EventCast, UIEventDerived};
use dom::bindings::error::Fallible;
use dom::bindings::global::GlobalRef;
use dom::bindings::js::{JS, JSRef, MutNullableHeap, Rootable, RootedReference};
use dom::bindings::js::Temporary;

use dom::bindings::utils::reflect_dom_object;
use dom::event::{Event, EventTypeId, EventBubbles, EventCancelable};
use dom::window::Window;
use util::str::DOMString;

use std::cell::Cell;
use std::default::Default;

// https://dvcs.w3.org/hg/dom3events/raw-file/tip/html/DOM3-Events.html#interface-UIEvent
#[dom_struct]
pub struct UIEvent {
    event: Event,
    view: MutNullableHeap<JS<Window>>,
    detail: Cell<i32>
}

impl UIEventDerived for Event {
    fn is_uievent(&self) -> bool {
        *self.type_id() == EventTypeId::UIEvent
    }
}

impl UIEvent {
    pub fn new_inherited(type_id: EventTypeId) -> UIEvent {
        UIEvent {
            event: Event::new_inherited(type_id),
            view: Default::default(),
            detail: Cell::new(0),
        }
    }

    pub fn new_uninitialized(window: JSRef<Window>) -> Temporary<UIEvent> {
        reflect_dom_object(box UIEvent::new_inherited(EventTypeId::UIEvent),
                           GlobalRef::Window(window),
                           UIEventBinding::Wrap)
    }

    pub fn new(window: JSRef<Window>,
               type_: DOMString,
               can_bubble: EventBubbles,
               cancelable: EventCancelable,
               view: Option<JSRef<Window>>,
               detail: i32) -> Temporary<UIEvent> {
        let ev = UIEvent::new_uninitialized(window).root();
        ev.r().InitUIEvent(type_, can_bubble == EventBubbles::Bubbles, cancelable == EventCancelable::Cancelable, view, detail);
        Temporary::from_rooted(ev.r())
    }

    pub fn Constructor(global: GlobalRef,
                       type_: DOMString,
                       init: &UIEventBinding::UIEventInit) -> Fallible<Temporary<UIEvent>> {
        let bubbles = if init.parent.bubbles { EventBubbles::Bubbles } else { EventBubbles::DoesNotBubble };
        let cancelable = if init.parent.cancelable { EventCancelable::Cancelable } else { EventCancelable::NotCancelable };
        let event = UIEvent::new(global.as_window(), type_,
                                 bubbles, cancelable,
                                 init.view.r(), init.detail);
        Ok(event)
    }
}

impl<'a> UIEventMethods for JSRef<'a, UIEvent> {
    // https://dvcs.w3.org/hg/dom3events/raw-file/tip/html/DOM3-Events.html#widl-UIEvent-view
    fn GetView(self) -> Option<Temporary<Window>> {
        self.view.get().map(Temporary::from_rooted)
    }

    // https://dvcs.w3.org/hg/dom3events/raw-file/tip/html/DOM3-Events.html#widl-UIEvent-detail
    fn Detail(self) -> i32 {
        self.detail.get()
    }

    fn InitUIEvent(self,
                   type_: DOMString,
                   can_bubble: bool,
                   cancelable: bool,
                   view: Option<JSRef<Window>>,
                   detail: i32) {
        let event: JSRef<Event> = EventCast::from_ref(self);
        if event.dispatching() {
            return;
        }

        event.InitEvent(type_, can_bubble, cancelable);
        self.view.set(view.map(JS::from_rooted));
        self.detail.set(detail);
    }
}

