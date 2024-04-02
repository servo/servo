/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use std::cell::Cell;

use dom_struct::dom_struct;
use js::jsapi::Heap;
use js::jsval::JSVal;
use js::rust::{HandleObject, HandleValue};
use servo_atoms::Atom;

use crate::dom::bindings::cell::DomRefCell;
use crate::dom::bindings::codegen::Bindings::ErrorEventBinding;
use crate::dom::bindings::codegen::Bindings::ErrorEventBinding::ErrorEventMethods;
use crate::dom::bindings::codegen::Bindings::EventBinding::EventMethods;
use crate::dom::bindings::error::Fallible;
use crate::dom::bindings::inheritance::Castable;
use crate::dom::bindings::reflector::reflect_dom_object_with_proto;
use crate::dom::bindings::root::DomRoot;
use crate::dom::bindings::str::DOMString;
use crate::dom::bindings::trace::RootedTraceableBox;
use crate::dom::event::{Event, EventBubbles, EventCancelable};
use crate::dom::globalscope::GlobalScope;
use crate::script_runtime::JSContext;

#[dom_struct]
pub struct ErrorEvent {
    event: Event,
    message: DomRefCell<DOMString>,
    filename: DomRefCell<DOMString>,
    lineno: Cell<u32>,
    colno: Cell<u32>,
    #[ignore_malloc_size_of = "Defined in rust-mozjs"]
    error: Heap<JSVal>,
}

impl ErrorEvent {
    fn new_inherited() -> ErrorEvent {
        ErrorEvent {
            event: Event::new_inherited(),
            message: DomRefCell::new(DOMString::new()),
            filename: DomRefCell::new(DOMString::new()),
            lineno: Cell::new(0),
            colno: Cell::new(0),
            error: Heap::default(),
        }
    }

    fn new_uninitialized(global: &GlobalScope, proto: Option<HandleObject>) -> DomRoot<ErrorEvent> {
        reflect_dom_object_with_proto(Box::new(ErrorEvent::new_inherited()), global, proto)
    }

    #[allow(clippy::too_many_arguments)]
    pub fn new(
        global: &GlobalScope,
        type_: Atom,
        bubbles: EventBubbles,
        cancelable: EventCancelable,
        message: DOMString,
        filename: DOMString,
        lineno: u32,
        colno: u32,
        error: HandleValue,
    ) -> DomRoot<ErrorEvent> {
        Self::new_with_proto(
            global, None, type_, bubbles, cancelable, message, filename, lineno, colno, error,
        )
    }

    #[allow(clippy::too_many_arguments)]
    fn new_with_proto(
        global: &GlobalScope,
        proto: Option<HandleObject>,
        type_: Atom,
        bubbles: EventBubbles,
        cancelable: EventCancelable,
        message: DOMString,
        filename: DOMString,
        lineno: u32,
        colno: u32,
        error: HandleValue,
    ) -> DomRoot<ErrorEvent> {
        let ev = ErrorEvent::new_uninitialized(global, proto);
        {
            let event = ev.upcast::<Event>();
            event.init_event(type_, bool::from(bubbles), bool::from(cancelable));
            *ev.message.borrow_mut() = message;
            *ev.filename.borrow_mut() = filename;
            ev.lineno.set(lineno);
            ev.colno.set(colno);
        }
        ev.error.set(error.get());
        ev
    }

    #[allow(non_snake_case)]
    pub fn Constructor(
        global: &GlobalScope,
        proto: Option<HandleObject>,
        type_: DOMString,
        init: RootedTraceableBox<ErrorEventBinding::ErrorEventInit>,
    ) -> Fallible<DomRoot<ErrorEvent>> {
        let msg = match init.message.as_ref() {
            Some(message) => message.clone(),
            None => DOMString::new(),
        };

        let file_name = match init.filename.as_ref() {
            Some(filename) => filename.clone(),
            None => DOMString::new(),
        };

        let line_num = init.lineno.unwrap_or(0);

        let col_num = init.colno.unwrap_or(0);

        let bubbles = EventBubbles::from(init.parent.bubbles);

        let cancelable = EventCancelable::from(init.parent.cancelable);

        let event = ErrorEvent::new_with_proto(
            global,
            proto,
            Atom::from(type_),
            bubbles,
            cancelable,
            msg,
            file_name,
            line_num,
            col_num,
            init.error.handle(),
        );
        Ok(event)
    }
}

impl ErrorEventMethods for ErrorEvent {
    // https://html.spec.whatwg.org/multipage/#dom-errorevent-lineno
    fn Lineno(&self) -> u32 {
        self.lineno.get()
    }

    // https://html.spec.whatwg.org/multipage/#dom-errorevent-colno
    fn Colno(&self) -> u32 {
        self.colno.get()
    }

    // https://html.spec.whatwg.org/multipage/#dom-errorevent-message
    fn Message(&self) -> DOMString {
        self.message.borrow().clone()
    }

    // https://html.spec.whatwg.org/multipage/#dom-errorevent-filename
    fn Filename(&self) -> DOMString {
        self.filename.borrow().clone()
    }

    // https://html.spec.whatwg.org/multipage/#dom-errorevent-error
    fn Error(&self, _cx: JSContext) -> JSVal {
        self.error.get()
    }

    // https://dom.spec.whatwg.org/#dom-event-istrusted
    fn IsTrusted(&self) -> bool {
        self.event.IsTrusted()
    }
}
