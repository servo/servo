/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use dom_struct::dom_struct;
use js::rust::HandleObject;
use servo_atoms::Atom;

use crate::dom::bindings::codegen::Bindings::EventBinding::EventMethods;
use crate::dom::bindings::codegen::Bindings::SubmitEventBinding;
use crate::dom::bindings::codegen::Bindings::SubmitEventBinding::SubmitEventMethods;
use crate::dom::bindings::inheritance::Castable;
use crate::dom::bindings::reflector::{reflect_dom_object_with_proto, DomObject};
use crate::dom::bindings::root::DomRoot;
use crate::dom::bindings::str::DOMString;
use crate::dom::event::Event;
use crate::dom::globalscope::GlobalScope;
use crate::dom::htmlelement::HTMLElement;
use crate::dom::window::Window;

#[dom_struct]
#[allow(non_snake_case)]
pub struct SubmitEvent {
    event: Event,
    submitter: Option<DomRoot<HTMLElement>>,
}

impl SubmitEvent {
    fn new_inherited(submitter: Option<DomRoot<HTMLElement>>) -> SubmitEvent {
        SubmitEvent {
            event: Event::new_inherited(),
            submitter,
        }
    }

    pub fn new(
        global: &GlobalScope,
        type_: Atom,
        bubbles: bool,
        cancelable: bool,
        submitter: Option<DomRoot<HTMLElement>>,
    ) -> DomRoot<SubmitEvent> {
        Self::new_with_proto(global, None, type_, bubbles, cancelable, submitter)
    }

    fn new_with_proto(
        global: &GlobalScope,
        proto: Option<HandleObject>,
        type_: Atom,
        bubbles: bool,
        cancelable: bool,
        submitter: Option<DomRoot<HTMLElement>>,
    ) -> DomRoot<SubmitEvent> {
        let ev = reflect_dom_object_with_proto(
            Box::new(SubmitEvent::new_inherited(submitter)),
            global,
            proto,
        );
        {
            let event = ev.upcast::<Event>();
            event.init_event(type_, bubbles, cancelable);
        }
        ev
    }

    #[allow(non_snake_case)]
    pub fn Constructor(
        window: &Window,
        proto: Option<HandleObject>,
        type_: DOMString,
        init: &SubmitEventBinding::SubmitEventInit,
    ) -> DomRoot<SubmitEvent> {
        SubmitEvent::new_with_proto(
            &window.global(),
            proto,
            Atom::from(type_),
            init.parent.bubbles,
            init.parent.cancelable,
            init.submitter.as_ref().map(|s| DomRoot::from_ref(&**s)),
        )
    }
}

impl SubmitEventMethods for SubmitEvent {
    /// <https://dom.spec.whatwg.org/#dom-event-istrusted>
    fn IsTrusted(&self) -> bool {
        self.event.IsTrusted()
    }

    /// <https://html.spec.whatwg.org/multipage/#dom-submitevent-submitter>
    fn GetSubmitter(&self) -> Option<DomRoot<HTMLElement>> {
        self.submitter.as_ref().map(|s| DomRoot::from_ref(&**s))
    }
}
