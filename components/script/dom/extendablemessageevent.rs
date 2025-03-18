/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use dom_struct::dom_struct;
use js::jsapi::Heap;
use js::jsval::JSVal;
use js::rust::{HandleObject, HandleValue, MutableHandleValue};
use stylo_atoms::Atom;

use crate::dom::bindings::codegen::Bindings::ExtendableEventBinding::ExtendableEvent_Binding::ExtendableEventMethods;
use crate::dom::bindings::codegen::Bindings::ExtendableMessageEventBinding;
use crate::dom::bindings::codegen::Bindings::ExtendableMessageEventBinding::ExtendableMessageEventMethods;
use crate::dom::bindings::error::Fallible;
use crate::dom::bindings::frozenarray::CachedFrozenArray;
use crate::dom::bindings::inheritance::Castable;
use crate::dom::bindings::reflector::reflect_dom_object_with_proto;
use crate::dom::bindings::root::{Dom, DomRoot};
use crate::dom::bindings::str::DOMString;
use crate::dom::bindings::trace::RootedTraceableBox;
use crate::dom::event::Event;
use crate::dom::eventtarget::EventTarget;
use crate::dom::extendableevent::ExtendableEvent;
use crate::dom::globalscope::GlobalScope;
use crate::dom::messageport::MessagePort;
use crate::dom::serviceworkerglobalscope::ServiceWorkerGlobalScope;
use crate::script_runtime::{CanGc, JSContext};

#[dom_struct]
#[allow(non_snake_case)]
pub(crate) struct ExtendableMessageEvent {
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
    frozen_ports: CachedFrozenArray,
}

#[allow(non_snake_case)]
impl ExtendableMessageEvent {
    pub(crate) fn new_inherited(
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
            frozen_ports: CachedFrozenArray::new(),
        }
    }

    #[allow(clippy::too_many_arguments)]
    pub(crate) fn new(
        global: &GlobalScope,
        type_: Atom,
        bubbles: bool,
        cancelable: bool,
        data: HandleValue,
        origin: DOMString,
        lastEventId: DOMString,
        ports: Vec<DomRoot<MessagePort>>,
        can_gc: CanGc,
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
            can_gc,
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
        can_gc: CanGc,
    ) -> DomRoot<ExtendableMessageEvent> {
        let ev = Box::new(ExtendableMessageEvent::new_inherited(
            origin,
            lastEventId,
            ports,
        ));
        let ev = reflect_dom_object_with_proto(ev, global, proto, can_gc);
        {
            let event = ev.upcast::<Event>();
            event.init_event(type_, bubbles, cancelable);
        }
        ev.data.set(data.get());

        ev
    }
}

#[allow(non_snake_case)]
impl ExtendableMessageEvent {
    pub(crate) fn dispatch_jsval(
        target: &EventTarget,
        scope: &GlobalScope,
        message: HandleValue,
        ports: Vec<DomRoot<MessagePort>>,
        can_gc: CanGc,
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
            can_gc,
        );
        Extendablemessageevent
            .upcast::<Event>()
            .fire(target, can_gc);
    }

    pub(crate) fn dispatch_error(target: &EventTarget, scope: &GlobalScope, can_gc: CanGc) {
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
            can_gc,
        );
        ExtendableMsgEvent.upcast::<Event>().fire(target, can_gc);
    }
}

impl ExtendableMessageEventMethods<crate::DomTypeHolder> for ExtendableMessageEvent {
    /// <https://w3c.github.io/ServiceWorker/#dom-extendablemessageevent-extendablemessageevent>
    fn Constructor(
        worker: &ServiceWorkerGlobalScope,
        proto: Option<HandleObject>,
        can_gc: CanGc,
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
            can_gc,
        );
        Ok(ev)
    }

    /// <https://w3c.github.io/ServiceWorker/#dom-extendablemessageevent-data>
    fn Data(&self, _cx: JSContext, mut retval: MutableHandleValue) {
        retval.set(self.data.get())
    }

    /// <https://w3c.github.io/ServiceWorker/#dom-extendablemessageevent-origin>
    fn Origin(&self) -> DOMString {
        self.origin.clone()
    }

    /// <https://w3c.github.io/ServiceWorker/#dom-extendablemessageevent-lasteventid>
    fn LastEventId(&self) -> DOMString {
        self.lastEventId.clone()
    }

    /// <https://dom.spec.whatwg.org/#dom-event-istrusted>
    fn IsTrusted(&self) -> bool {
        self.event.IsTrusted()
    }

    /// <https://w3c.github.io/ServiceWorker/#extendablemessage-event-ports>
    fn Ports(&self, cx: JSContext, can_gc: CanGc, retval: MutableHandleValue) {
        self.frozen_ports.get_or_init(
            || {
                self.ports
                    .iter()
                    .map(|port| DomRoot::from_ref(&**port))
                    .collect()
            },
            cx,
            retval,
            can_gc,
        );
    }
}
