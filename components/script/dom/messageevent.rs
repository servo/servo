/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use crate::dom::bindings::codegen::Bindings::EventBinding::EventMethods;
use crate::dom::bindings::codegen::Bindings::MessageEventBinding;
use crate::dom::bindings::codegen::Bindings::MessageEventBinding::MessageEventMethods;
use crate::dom::bindings::error::Fallible;
use crate::dom::bindings::inheritance::Castable;
use crate::dom::bindings::reflector::{reflect_dom_object, DomObject};
use crate::dom::bindings::root::{Dom, DomRoot};
use crate::dom::bindings::str::DOMString;
use crate::dom::bindings::trace::RootedTraceableBox;
use crate::dom::event::Event;
use crate::dom::eventtarget::EventTarget;
use crate::dom::globalscope::GlobalScope;
use crate::dom::windowproxy::WindowProxy;
use dom_struct::dom_struct;
use js::jsapi::{Heap, JSContext, JSObject};
use js::jsval::JSVal;
use js::rust::HandleValue;
use servo_atoms::Atom;
use std::ptr::NonNull;

#[dom_struct]
pub struct MessageEvent {
    event: Event,
    data: Heap<JSVal>,
    origin: DOMString,
    source: Option<Dom<WindowProxy>>,
    lastEventId: DOMString,
}

impl MessageEvent {
    pub fn new_uninitialized(global: &GlobalScope) -> DomRoot<MessageEvent> {
        MessageEvent::new_initialized(
            global,
            HandleValue::undefined(),
            DOMString::new(),
            None,
            DOMString::new(),
        )
    }

    pub fn new_initialized(
        global: &GlobalScope,
        data: HandleValue,
        origin: DOMString,
        source: Option<&WindowProxy>,
        lastEventId: DOMString,
    ) -> DomRoot<MessageEvent> {
        let ev = Box::new(MessageEvent {
            event: Event::new_inherited(),
            data: Heap::default(),
            origin: origin,
            source: source.map(Dom::from_ref),
            lastEventId: lastEventId,
        });
        let ev = reflect_dom_object(ev, global, MessageEventBinding::Wrap);
        ev.data.set(data.get());

        ev
    }

    pub fn new(
        global: &GlobalScope,
        type_: Atom,
        bubbles: bool,
        cancelable: bool,
        data: HandleValue,
        origin: DOMString,
        source: Option<&WindowProxy>,
        lastEventId: DOMString,
    ) -> DomRoot<MessageEvent> {
        let ev = MessageEvent::new_initialized(global, data, origin, source, lastEventId);
        {
            let event = ev.upcast::<Event>();
            event.init_event(type_, bubbles, cancelable);
        }
        ev
    }

    pub fn Constructor(
        global: &GlobalScope,
        type_: DOMString,
        init: RootedTraceableBox<MessageEventBinding::MessageEventInit>,
    ) -> Fallible<DomRoot<MessageEvent>> {
        let source = init
            .source
            .as_ref()
            .and_then(|inner| inner.as_ref().map(|source| source.window_proxy()));
        let ev = MessageEvent::new(
            global,
            Atom::from(type_),
            init.parent.bubbles,
            init.parent.cancelable,
            init.data.handle(),
            init.origin.clone(),
            source.as_ref().map(|source| &**source),
            init.lastEventId.clone(),
        );
        Ok(ev)
    }
}

impl MessageEvent {
    pub fn dispatch_jsval(
        target: &EventTarget,
        scope: &GlobalScope,
        message: HandleValue,
        origin: Option<&str>,
        source: Option<&WindowProxy>,
    ) {
        let messageevent = MessageEvent::new(
            scope,
            atom!("message"),
            false,
            false,
            message,
            DOMString::from(origin.unwrap_or("")),
            source,
            DOMString::new(),
        );
        messageevent.upcast::<Event>().fire(target);
    }
}

impl MessageEventMethods for MessageEvent {
    #[allow(unsafe_code)]
    // https://html.spec.whatwg.org/multipage/#dom-messageevent-data
    unsafe fn Data(&self, _cx: *mut JSContext) -> JSVal {
        self.data.get()
    }

    // https://html.spec.whatwg.org/multipage/#dom-messageevent-origin
    fn Origin(&self) -> DOMString {
        self.origin.clone()
    }

    // https://html.spec.whatwg.org/multipage/#dom-messageevent-source
    #[allow(unsafe_code)]
    unsafe fn GetSource(&self, _cx: *mut JSContext) -> Option<NonNull<JSObject>> {
        self.source
            .as_ref()
            .and_then(|source| NonNull::new(source.reflector().get_jsobject().get()))
    }

    // https://html.spec.whatwg.org/multipage/#dom-messageevent-lasteventid
    fn LastEventId(&self) -> DOMString {
        self.lastEventId.clone()
    }

    // https://dom.spec.whatwg.org/#dom-event-istrusted
    fn IsTrusted(&self) -> bool {
        self.event.IsTrusted()
    }
}
