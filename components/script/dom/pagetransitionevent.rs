/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use dom::bindings::codegen::Bindings::PageTransitionEventBinding;
use dom::bindings::codegen::Bindings::PageTransitionEventBinding::PageTransitionEventMethods;
use dom::bindings::codegen::Bindings::EventBinding::EventMethods;
use dom::bindings::error::Fallible;
use dom::bindings::global::GlobalRef;
use dom::bindings::inheritance::Castable;
use dom::bindings::js::Root;
use dom::bindings::reflector::reflect_dom_object;
use dom::event::Event;
use std::cell::Cell;
use string_cache::Atom;
use util::str::DOMString;

// https://html.spec.whatwg.org/multipage/#pagetransitionevent
#[dom_struct]
pub struct PageTransitionEvent {
    event: Event,
    persisted: Cell<bool>,
}

impl PageTransitionEvent {
    fn new_inherited() -> PageTransitionEvent {
        PageTransitionEvent {
            event: Event::new_inherited(),
            persisted: Cell::new(false),
        }
    }

    pub fn new_uninitialized(global: GlobalRef) -> Root<PageTransitionEvent> {
        reflect_dom_object(box PageTransitionEvent::new_inherited(),
                           global,
                           PageTransitionEventBinding::Wrap)
    }

    pub fn new(global: GlobalRef,
               type_: Atom,
               bubbles: bool,
               cancelable: bool,
               persisted: bool)
               -> Root<PageTransitionEvent> {
        let ev = PageTransitionEvent::new_uninitialized(global);
        ev.persisted.set(persisted);
        {
            let event = ev.upcast::<Event>();
            event.init_event(type_, bubbles, cancelable);
        }
        ev
    }

    #[allow(unsafe_code)]
    pub fn Constructor(global: GlobalRef,
                       type_: DOMString,
                       init: &PageTransitionEventBinding::PageTransitionEventInit)
                       -> Fallible<Root<PageTransitionEvent>> {
        Ok(PageTransitionEvent::new(global,
                              Atom::from(type_),
                              init.parent.bubbles,
                              init.parent.cancelable,
                              init.persisted))
    }
}

impl PageTransitionEventMethods for PageTransitionEvent {
    // https://html.spec.whatwg.org/multipage/#dom-pagetransitionevent-persisted
    fn Persisted(&self) -> bool {
        self.persisted.get()
    }

    // https://dom.spec.whatwg.org/#dom-event-istrusted
    fn IsTrusted(&self) -> bool {
        self.event.IsTrusted()
    }
}
