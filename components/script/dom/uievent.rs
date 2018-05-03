/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use dom::bindings::codegen::Bindings::EventBinding::EventMethods;
use dom::bindings::codegen::Bindings::UIEventBinding;
use dom::bindings::codegen::Bindings::UIEventBinding::UIEventMethods;
use dom::bindings::error::Fallible;
use dom::bindings::inheritance::Castable;
use dom::bindings::reflector::reflect_dom_object;
use dom::bindings::root::{DomRoot, MutNullableDom, RootedReference};
use dom::bindings::str::DOMString;
use dom::event::{Event, EventBubbles, EventCancelable};
use dom::window::Window;
use dom_struct::dom_struct;
use servo_atoms::Atom;
use std::cell::Cell;
use std::default::Default;
use typeholder::TypeHolderTrait;

// https://w3c.github.io/uievents/#interface-uievent
#[dom_struct]
pub struct UIEvent<TH: TypeHolderTrait> {
    event: Event<TH>,
    view: MutNullableDom<Window<TH>>,
    detail: Cell<i32>
}

impl<TH: TypeHolderTrait> UIEvent<TH> {
    pub fn new_inherited() -> UIEvent<TH> {
        UIEvent {
            event: Event::new_inherited(),
            view: Default::default(),
            detail: Cell::new(0),
        }
    }

    pub fn new_uninitialized(window: &Window<TH>) -> DomRoot<UIEvent<TH>> {
        reflect_dom_object(Box::new(UIEvent::new_inherited()),
                           window,
                           UIEventBinding::Wrap)
    }

    pub fn new(window: &Window<TH>,
               type_: DOMString,
               can_bubble: EventBubbles,
               cancelable: EventCancelable,
               view: Option<&Window<TH>>,
               detail: i32) -> DomRoot<UIEvent<TH>> {
        let ev = UIEvent::new_uninitialized(window);
        ev.InitUIEvent(type_, bool::from(can_bubble), bool::from(cancelable), view, detail);
        ev
    }

    pub fn Constructor(window: &Window<TH>,
                       type_: DOMString,
                       init: &UIEventBinding::UIEventInit<TH>) -> Fallible<DomRoot<UIEvent<TH>>> {
        let bubbles = EventBubbles::from(init.parent.bubbles);
        let cancelable = EventCancelable::from(init.parent.cancelable);
        let event = UIEvent::new(window,
                                 type_,
                                 bubbles, cancelable,
                                 init.view.r(), init.detail);
        Ok(event)
    }
}

impl<TH: TypeHolderTrait> UIEventMethods<TH> for UIEvent<TH> {
    // https://w3c.github.io/uievents/#widl-UIEvent-view
    fn GetView(&self) -> Option<DomRoot<Window<TH>>> {
        self.view.get()
    }

    // https://w3c.github.io/uievents/#widl-UIEvent-detail
    fn Detail(&self) -> i32 {
        self.detail.get()
    }

    // https://w3c.github.io/uievents/#widl-UIEvent-initUIEvent
    fn InitUIEvent(&self,
                   type_: DOMString,
                   can_bubble: bool,
                   cancelable: bool,
                   view: Option<&Window<TH>>,
                   detail: i32) {
        let event = self.upcast::<Event<TH>>();
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
