/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use dom::bindings::codegen::Bindings::EventBinding::EventMethods;
use dom::bindings::codegen::Bindings::UIEventBinding;
use dom::bindings::codegen::Bindings::UIEventBinding::UIEventMethods;
use dom::bindings::codegen::InheritTypes::{EventCast, UIEventDerived};
use dom::bindings::error::Fallible;
use dom::bindings::global::{GlobalRef, Window};
use dom::bindings::js::{JS, JSRef, RootedReference, Temporary, OptionalSettable};
use dom::bindings::trace::Traceable;
use dom::bindings::utils::{Reflectable, Reflector, reflect_dom_object};
use dom::event::{Event, EventTypeId, UIEventTypeId};
use dom::window::Window;
use servo_util::str::DOMString;

use std::cell::Cell;

#[deriving(Encodable)]
#[must_root]
pub struct UIEvent {
    pub event: Event,
    view: Cell<Option<JS<Window>>>,
    detail: Traceable<Cell<i32>>
}

impl UIEventDerived for Event {
    fn is_uievent(&self) -> bool {
        self.type_id == UIEventTypeId
    }
}

impl UIEvent {
    pub fn new_inherited(type_id: EventTypeId) -> UIEvent {
        UIEvent {
            event: Event::new_inherited(type_id),
            view: Cell::new(None),
            detail: Traceable::new(Cell::new(0)),
        }
    }

    pub fn new_uninitialized(window: JSRef<Window>) -> Temporary<UIEvent> {
        reflect_dom_object(box UIEvent::new_inherited(UIEventTypeId),
                           &Window(window),
                           UIEventBinding::Wrap)
    }

    pub fn new(window: JSRef<Window>,
               type_: DOMString,
               can_bubble: bool,
               cancelable: bool,
               view: Option<JSRef<Window>>,
               detail: i32) -> Temporary<UIEvent> {
        let ev = UIEvent::new_uninitialized(window).root();
        ev.deref().InitUIEvent(type_, can_bubble, cancelable, view, detail);
        Temporary::from_rooted(*ev)
    }

    pub fn Constructor(global: &GlobalRef,
                       type_: DOMString,
                       init: &UIEventBinding::UIEventInit) -> Fallible<Temporary<UIEvent>> {
        let event = UIEvent::new(global.as_window(), type_,
                                 init.parent.bubbles, init.parent.cancelable,
                                 init.view.root_ref(), init.detail);
        Ok(event)
    }
}

impl<'a> UIEventMethods for JSRef<'a, UIEvent> {
    fn GetView(&self) -> Option<Temporary<Window>> {
        self.view.get().map(|view| Temporary::new(view))
    }

    fn Detail(&self) -> i32 {
        self.detail.deref().get()
    }

    fn InitUIEvent(&self,
                   type_: DOMString,
                   can_bubble: bool,
                   cancelable: bool,
                   view: Option<JSRef<Window>>,
                   detail: i32) {
        let event: JSRef<Event> = EventCast::from_ref(*self);
        event.InitEvent(type_, can_bubble, cancelable);
        self.view.assign(view);
        self.detail.deref().set(detail);
    }
}

impl Reflectable for UIEvent {
    fn reflector<'a>(&'a self) -> &'a Reflector {
        self.event.reflector()
    }
}
