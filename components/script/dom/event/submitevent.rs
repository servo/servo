/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use dom_struct::dom_struct;
use js::context::JSContext;
use js::rust::HandleObject;
use script_bindings::reflector::reflect_dom_object_with_proto_and_cx;
use stylo_atoms::Atom;

use crate::dom::bindings::codegen::Bindings::EventBinding::EventMethods;
use crate::dom::bindings::codegen::Bindings::SubmitEventBinding;
use crate::dom::bindings::codegen::Bindings::SubmitEventBinding::SubmitEventMethods;
use crate::dom::bindings::inheritance::Castable;
use crate::dom::bindings::root::{Dom, DomRoot};
use crate::dom::bindings::str::DOMString;
use crate::dom::event::Event;
use crate::dom::html::htmlelement::HTMLElement;
use crate::dom::window::Window;

#[dom_struct]
pub(crate) struct SubmitEvent {
    event: Event,
    submitter: Option<Dom<HTMLElement>>,
}

impl SubmitEvent {
    fn new_inherited(submitter: Option<&HTMLElement>) -> SubmitEvent {
        SubmitEvent {
            event: Event::new_inherited(),
            submitter: submitter.map(Dom::from_ref),
        }
    }

    pub(crate) fn new(
        cx: &mut JSContext,
        window: &Window,
        type_: Atom,
        bubbles: bool,
        cancelable: bool,
        submitter: Option<&HTMLElement>,
    ) -> DomRoot<SubmitEvent> {
        Self::new_with_proto(cx, window, None, type_, bubbles, cancelable, submitter)
    }

    fn new_with_proto(
        cx: &mut JSContext,
        window: &Window,
        proto: Option<HandleObject>,
        type_: Atom,
        bubbles: bool,
        cancelable: bool,
        submitter: Option<&HTMLElement>,
    ) -> DomRoot<SubmitEvent> {
        let ev = reflect_dom_object_with_proto_and_cx(
            Box::new(SubmitEvent::new_inherited(submitter)),
            window,
            proto,
            cx,
        );
        {
            let event = ev.upcast::<Event>();
            event.init_event(type_, bubbles, cancelable);
        }
        ev
    }
}

impl SubmitEventMethods<crate::DomTypeHolder> for SubmitEvent {
    /// <https://html.spec.whatwg.org/multipage/#submitevent>
    fn Constructor(
        cx: &mut JSContext,
        window: &Window,
        proto: Option<HandleObject>,
        type_: DOMString,
        init: &SubmitEventBinding::SubmitEventInit,
    ) -> DomRoot<SubmitEvent> {
        SubmitEvent::new_with_proto(
            cx,
            window,
            proto,
            Atom::from(type_),
            init.parent.bubbles,
            init.parent.cancelable,
            init.submitter.as_deref(),
        )
    }

    /// <https://dom.spec.whatwg.org/#dom-event-istrusted>
    fn IsTrusted(&self) -> bool {
        self.event.IsTrusted()
    }

    /// <https://html.spec.whatwg.org/multipage/#dom-submitevent-submitter>
    fn GetSubmitter(&self) -> Option<DomRoot<HTMLElement>> {
        self.submitter.as_ref().map(|s| DomRoot::from_ref(&**s))
    }
}
