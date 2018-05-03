/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use dom::bindings::codegen::Bindings::EventBinding::EventMethods;
use dom::bindings::codegen::Bindings::PageTransitionEventBinding;
use dom::bindings::codegen::Bindings::PageTransitionEventBinding::PageTransitionEventMethods;
use dom::bindings::error::Fallible;
use dom::bindings::inheritance::Castable;
use dom::bindings::reflector::reflect_dom_object;
use dom::bindings::root::DomRoot;
use dom::bindings::str::DOMString;
use dom::event::Event;
use dom::window::Window;
use dom_struct::dom_struct;
use servo_atoms::Atom;
use std::cell::Cell;
use typeholder::TypeHolderTrait;
// https://html.spec.whatwg.org/multipage/#pagetransitionevent
#[dom_struct]
pub struct PageTransitionEvent<TH: TypeHolderTrait> {
    event: Event<TH>,
    persisted: Cell<bool>,
}

impl<TH: TypeHolderTrait> PageTransitionEvent<TH> {
    fn new_inherited() -> PageTransitionEvent<TH> {
        PageTransitionEvent {
            event: Event::new_inherited(),
            persisted: Cell::new(false),
        }
    }

    pub fn new_uninitialized(window: &Window<TH>) -> DomRoot<PageTransitionEvent<TH>> {
        reflect_dom_object(Box::new(PageTransitionEvent::new_inherited()),
                           window,
                           PageTransitionEventBinding::Wrap)
    }

    pub fn new(window: &Window<TH>,
               type_: Atom,
               bubbles: bool,
               cancelable: bool,
               persisted: bool)
               -> DomRoot<PageTransitionEvent<TH>> {
        let ev = PageTransitionEvent::new_uninitialized(window);
        ev.persisted.set(persisted);
        {
            let event = ev.upcast::<Event<TH>>();
            event.init_event(type_, bubbles, cancelable);
        }
        ev
    }

    pub fn Constructor(window: &Window<TH>,
                       type_: DOMString,
                       init: &PageTransitionEventBinding::PageTransitionEventInit)
                       -> Fallible<DomRoot<PageTransitionEvent<TH>>> {
        Ok(PageTransitionEvent::new(window,
                              Atom::from(type_),
                              init.parent.bubbles,
                              init.parent.cancelable,
                              init.persisted))
    }
}

impl<TH: TypeHolderTrait> PageTransitionEventMethods for PageTransitionEvent<TH> {
    // https://html.spec.whatwg.org/multipage/#dom-pagetransitionevent-persisted
    fn Persisted(&self) -> bool {
        self.persisted.get()
    }

    // https://dom.spec.whatwg.org/#dom-event-istrusted
    fn IsTrusted(&self) -> bool {
        self.event.IsTrusted()
    }
}
