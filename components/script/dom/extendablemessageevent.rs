/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use dom_struct::dom_struct;
use js::jsapi::Heap;
use js::jsval::JSVal;
use js::rust::{HandleObject, HandleValue};
use servo_atoms::Atom;

use crate::dom::bindings::cell::DomRefCell;
use crate::dom::bindings::codegen::Bindings::ExtendableMessageEventBinding;
use crate::dom::bindings::codegen::Bindings::ExtendableMessageEventBinding::ExtendableMessageEventMethods;
use crate::dom::bindings::error::Fallible;
use crate::dom::bindings::inheritance::Castable;
use crate::dom::bindings::reflector::reflect_dom_object_with_proto;
use crate::dom::bindings::root::{Dom, DomRoot};
use crate::dom::bindings::str::DOMString;
use crate::dom::bindings::trace::RootedTraceableBox;
use crate::dom::bindings::utils::to_frozen_array;
use crate::dom::event::Event;
use crate::dom::eventtarget::EventTarget;
use crate::dom::extendableevent::ExtendableEvent;
use crate::dom::globalscope::GlobalScope;
use crate::dom::messageport::MessagePort;
use crate::dom::serviceworkerglobalscope::ServiceWorkerGlobalScope;
use crate::script_runtime::JSContext;

#[dom_struct]
#[allow(non_snake_case)]
pub struct ExtendableMessageEvent {
    /// <https://w3c.github.io/ServiceWorker/#extendableevent>
    event: ExtendableEvent,
    /// <https://w3c.github.io/ServiceWorker/#dom-extendablemessageevent-data>
    #[ignore_malloc_size_of = "mozjs"]
    data: Heap<JSVal>,
    /// <https://w3c.github.io/ServiceWorker/#extendablemessage-event-origin>
    origin: DOMString,
    /// <https://w3c.github.io/ServiceWorker/#dom-extendablemessageevent-lasteventid>
    lastEventId: DOMString,
    /// <https://w3c.github.io/ServiceWorker/#dom-extendablemessageevent-ports>
    ports: Vec<Dom<MessagePort>>,
    #[ignore_malloc_size_of = "mozjs"]
    frozen_ports: DomRefCell<Option<Heap<JSVal>>>,
}

#[allow(non_snake_case)]
impl ExtendableMessageEvent {
    pub fn new_inherited(
        origin: DOMString,
        lastEventId: DOMString,
        ports: Vec<DomRoot<MessagePort>>,
    ) -> ExtendableMessageEvent {
        ExtendableMessageEvent {
            event: ExtendableEvent::new_inherited(),
            data: Heap::default(),
            origin,
            lastEventId,
            ports: ports
                .into_iter()
                .map(|port| Dom::from_ref(&*port))
                .collect(),
            frozen_ports: DomRefCell::new(None),
        }
    }

    #[allow(clippy::too_many_arguments)]
    pub fn new(
        global: &GlobalScope,
        type_: Atom,
        bubbles: bool,
        cancelable: bool,
        data: HandleValue,
        origin: DOMString,
        lastEventId: DOMString,
        ports: Vec<DomRoot<MessagePort>>,
    ) -> DomRoot<ExtendableMessageEvent> {
        Self::new_with_proto(
            global,
            None,
            type_,
            bubbles,
            cancelable,
            data,
            origin,
            lastEventId,
            ports,
        )
    }

    #[allow(clippy::too_many_arguments)]
    fn new_with_proto(
        global: &GlobalScope,
        proto: Option<HandleObject>,
        type_: Atom,
        bubbles: bool,
        cancelable: bool,
        data: HandleValue,
        origin: DOMString,
        lastEventId: DOMString,
        ports: Vec<DomRoot<MessagePort>>,
    ) -> DomRoot<ExtendableMessageEvent> {
        let ev = Box::new(ExtendableMessageEvent::new_inherited(
            origin,
            lastEventId,
            ports,
        ));
        let ev = reflect_dom_object_with_proto(ev, global, proto);
        {
            let event = ev.upcast::<Event>();
            event.init_event(type_, bubbles, cancelable);
        }
        ev.data.set(data.get());

        ev
    }

    pub fn Constructor(
        worker: &ServiceWorkerGlobalScope,
        proto: Option<HandleObject>,
        type_: DOMString,
        init: RootedTraceableBox<ExtendableMessageEventBinding::ExtendableMessageEventInit>,
    ) -> Fallible<DomRoot<ExtendableMessageEvent>> {
        let global = worker.upcast::<GlobalScope>();
        let ev = ExtendableMessageEvent::new_with_proto(
            global,
            proto,
            Atom::from(type_),
            init.parent.parent.bubbles,
            init.parent.parent.cancelable,
            init.data.handle(),
            init.origin.clone(),
            init.lastEventId.clone(),
            vec![],
        );
        Ok(ev)
    }
}

#[allow(non_snake_case)]
impl ExtendableMessageEvent {
    pub fn dispatch_jsval(
        target: &EventTarget,
        scope: &GlobalScope,
        message: HandleValue,
        ports: Vec<DomRoot<MessagePort>>,
    ) {
        let Extendablemessageevent = ExtendableMessageEvent::new(
            scope,
            atom!("message"),
            false,
            false,
            message,
            DOMString::new(),
            DOMString::new(),
            ports,
        );
        Extendablemessageevent.upcast::<Event>().fire(target);
    }

    pub fn dispatch_error(target: &EventTarget, scope: &GlobalScope) {
        let init = ExtendableMessageEventBinding::ExtendableMessageEventInit::empty();
        let ExtendableMsgEvent = ExtendableMessageEvent::new(
            scope,
            atom!("messageerror"),
            init.parent.parent.bubbles,
            init.parent.parent.cancelable,
            init.data.handle(),
            init.origin.clone(),
            init.lastEventId.clone(),
            init.ports.clone(),
        );
        ExtendableMsgEvent.upcast::<Event>().fire(target);
    }
}

impl ExtendableMessageEventMethods for ExtendableMessageEvent {
    // https://w3c.github.io/ServiceWorker/#extendablemessage-event-data-attribute
    fn Data(&self, _cx: JSContext) -> JSVal {
        self.data.get()
    }

    // https://w3c.github.io/ServiceWorker/#extendablemessage-event-origin-attribute
    fn Origin(&self) -> DOMString {
        self.origin.clone()
    }

    // https://w3c.github.io/ServiceWorker/#extendablemessage-event-lasteventid-attribute
    fn LastEventId(&self) -> DOMString {
        self.lastEventId.clone()
    }

    // https://dom.spec.whatwg.org/#dom-event-istrusted
    fn IsTrusted(&self) -> bool {
        self.event.IsTrusted()
    }

    /// <https://w3c.github.io/ServiceWorker/#extendablemessage-event-ports>
    fn Ports(&self, cx: JSContext) -> JSVal {
        if let Some(ports) = &*self.frozen_ports.borrow() {
            return ports.get();
        }

        let ports: Vec<DomRoot<MessagePort>> = self
            .ports
            .iter()
            .map(|port| DomRoot::from_ref(&**port))
            .collect();
        let frozen_ports = to_frozen_array(ports.as_slice(), cx);

        // Safety: need to create the Heap value in its final memory location before setting it.
        *self.frozen_ports.borrow_mut() = Some(Heap::default());
        self.frozen_ports
            .borrow()
            .as_ref()
            .unwrap()
            .set(frozen_ports);

        frozen_ports
    }
}
