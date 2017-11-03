/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use dom::bindings::codegen::Bindings::EventBinding::EventMethods;
use dom::bindings::codegen::Bindings::MessageEventBinding;
use dom::bindings::codegen::Bindings::MessageEventBinding::MessageEventMethods;
use dom::bindings::error::Fallible;
use dom::bindings::inheritance::Castable;
use dom::bindings::reflector::reflect_dom_object;
use dom::bindings::root::DomRoot;
use dom::bindings::str::DOMString;
use dom::bindings::trace::RootedTraceableBox;
use dom::event::Event;
use dom::eventtarget::EventTarget;
use dom::globalscope::GlobalScope;
use dom::messageport::MessagePort;
use dom_struct::dom_struct;
use js::conversions::ToJSValConvertible;
use js::jsapi::{Heap, JS_FreezeObject, JSContext};
use js::jsapi::HandleObject as RawHandleObject;
use js::jsval::{JSVal, UndefinedValue};
use js::rust::HandleValue;
use servo_atoms::Atom;

#[dom_struct]
pub struct MessageEvent {
    event: Event,
    data: Heap<JSVal>,
    origin: DOMString,
    lastEventId: DOMString,
    ports: Vec<DomRoot<MessagePort>>,
}

impl MessageEvent {
    pub fn new_uninitialized(global: &GlobalScope) -> DomRoot<MessageEvent> {
        MessageEvent::new_initialized(
            global,
            HandleValue::undefined(),
            DOMString::new(),
            DOMString::new(),
            vec![]
        )
    }

    pub fn new_initialized(
        global: &GlobalScope,
        data: HandleValue,
        origin: DOMString,
        lastEventId: DOMString,
        ports: Vec<DomRoot<MessagePort>>,
    ) -> DomRoot<MessageEvent> {
        let ev = Box::new(MessageEvent {
            event: Event::new_inherited(),
            data: Heap::default(),
            origin,
            lastEventId,
            ports,
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
        lastEventId: DOMString,
        ports: Vec<DomRoot<MessagePort>>,
    ) -> DomRoot<MessageEvent> {
        let ev = MessageEvent::new_initialized(global, data, origin, lastEventId, ports);
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
        let ev = MessageEvent::new(global,
                                   Atom::from(type_),
                                   init.parent.bubbles,
                                   init.parent.cancelable,
                                   init.data.handle(),
                                   init.origin.clone(),
                                   init.lastEventId.clone(),
                                   init.ports.clone().unwrap_or(vec![]));
        Ok(ev)
    }
}

impl MessageEvent {
    pub fn dispatch_jsval(
        target: &EventTarget,
        scope: &GlobalScope,
        message: HandleValue,
        origin: Option<&str>,
        ports: Vec<DomRoot<MessagePort>>,
    ) {
        let messageevent = MessageEvent::new(
            scope,
            atom!("message"),
            false,
            false,
            message,
            DOMString::from(origin.unwrap_or("")),
            DOMString::new(),
            ports,
        );
        messageevent.upcast::<Event>().fire(target);
    }
}

impl MessageEventMethods for MessageEvent {
    #[allow(unsafe_code)]
    /// <https://html.spec.whatwg.org/multipage/#dom-messageevent-data>
    unsafe fn Data(&self, _cx: *mut JSContext) -> JSVal {
        self.data.get()
    }

    /// <https://html.spec.whatwg.org/multipage/#dom-messageevent-origin>
    fn Origin(&self) -> DOMString {
        self.origin.clone()
    }

    /// <https://html.spec.whatwg.org/multipage/#dom-messageevent-lasteventid>
    fn LastEventId(&self) -> DOMString {
        self.lastEventId.clone()
    }

    /// <https://dom.spec.whatwg.org/#dom-event-istrusted>
    fn IsTrusted(&self) -> bool {
        self.event.IsTrusted()
    }

    #[allow(unsafe_code)]
    /// <https://html.spec.whatwg.org/multipage/#dom-messageevent-ports>
    unsafe fn Ports(&self, cx: *mut JSContext) -> JSVal {
        rooted!(in(cx) let mut ports = UndefinedValue());
        self.ports.to_jsval(cx, ports.handle_mut());

        rooted!(in(cx) let obj = ports.to_object());
        JS_FreezeObject(cx, RawHandleObject::from(obj.handle()));
        *ports
    }
}
