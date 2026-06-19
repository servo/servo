/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use dom_struct::dom_struct;
use js::context::JSContext;
use js::rust::HandleObject;
use script_bindings::reflector::reflect_dom_object_with_proto_and_cx;
use stylo_atoms::Atom;

use crate::dom::bindings::codegen::Bindings::EventBinding::EventMethods;
use crate::dom::bindings::codegen::Bindings::HashChangeEventBinding;
use crate::dom::bindings::codegen::Bindings::HashChangeEventBinding::HashChangeEventMethods;
use crate::dom::bindings::error::Fallible;
use crate::dom::bindings::inheritance::Castable;
use crate::dom::bindings::root::DomRoot;
use crate::dom::bindings::str::{DOMString, USVString};
use crate::dom::event::Event;
use crate::dom::window::Window;

// https://html.spec.whatwg.org/multipage/#hashchangeevent
#[dom_struct]
pub(crate) struct HashChangeEvent {
    event: Event,
    old_url: String,
    new_url: String,
}

impl HashChangeEvent {
    fn new_inherited(old_url: String, new_url: String) -> HashChangeEvent {
        HashChangeEvent {
            event: Event::new_inherited(),
            old_url,
            new_url,
        }
    }

    pub(crate) fn new_uninitialized(
        cx: &mut JSContext,
        window: &Window,
    ) -> DomRoot<HashChangeEvent> {
        Self::new_uninitialized_with_proto(cx, window, None)
    }

    fn new_uninitialized_with_proto(
        cx: &mut JSContext,
        window: &Window,
        proto: Option<HandleObject>,
    ) -> DomRoot<HashChangeEvent> {
        reflect_dom_object_with_proto_and_cx(
            Box::new(HashChangeEvent::new_inherited(String::new(), String::new())),
            window,
            proto,
            cx,
        )
    }

    pub(crate) fn new(
        cx: &mut JSContext,
        window: &Window,
        type_: Atom,
        bubbles: bool,
        cancelable: bool,
        old_url: String,
        new_url: String,
    ) -> DomRoot<HashChangeEvent> {
        Self::new_with_proto(
            cx, window, None, type_, bubbles, cancelable, old_url, new_url,
        )
    }

    #[expect(clippy::too_many_arguments)]
    fn new_with_proto(
        cx: &mut JSContext,
        window: &Window,
        proto: Option<HandleObject>,
        type_: Atom,
        bubbles: bool,
        cancelable: bool,
        old_url: String,
        new_url: String,
    ) -> DomRoot<HashChangeEvent> {
        let ev = reflect_dom_object_with_proto_and_cx(
            Box::new(HashChangeEvent::new_inherited(old_url, new_url)),
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

impl HashChangeEventMethods<crate::DomTypeHolder> for HashChangeEvent {
    /// <https://html.spec.whatwg.org/multipage/#hashchangeevent>
    fn Constructor(
        cx: &mut JSContext,
        window: &Window,
        proto: Option<HandleObject>,
        type_: DOMString,
        init: &HashChangeEventBinding::HashChangeEventInit,
    ) -> Fallible<DomRoot<HashChangeEvent>> {
        Ok(HashChangeEvent::new_with_proto(
            cx,
            window,
            proto,
            Atom::from(type_),
            init.parent.bubbles,
            init.parent.cancelable,
            init.oldURL.0.clone(),
            init.newURL.0.clone(),
        ))
    }

    /// <https://html.spec.whatwg.org/multipage/#dom-hashchangeevent-oldurl>
    fn OldURL(&self) -> USVString {
        USVString(self.old_url.clone())
    }

    /// <https://html.spec.whatwg.org/multipage/#dom-hashchangeevent-newurl>
    fn NewURL(&self) -> USVString {
        USVString(self.new_url.clone())
    }

    /// <https://dom.spec.whatwg.org/#dom-event-istrusted>
    fn IsTrusted(&self) -> bool {
        self.event.IsTrusted()
    }
}
