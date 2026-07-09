/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use dom_struct::dom_struct;
use js::context::JSContext;
use js::jsapi::Heap;
use js::jsval::JSVal;
use js::rust::{HandleObject, HandleValue, MutableHandleValue};
use script_bindings::reflector::reflect_dom_object_with_proto_and_cx;
use stylo_atoms::Atom;

use crate::dom::bindings::codegen::Bindings::ExtendableEventBinding::ExtendableEvent_Binding::ExtendableEventMethods;
use crate::dom::bindings::codegen::Bindings::ExtendableMessageEventBinding;
use crate::dom::bindings::codegen::Bindings::ExtendableMessageEventBinding::ExtendableMessageEventMethods;
use crate::dom::bindings::codegen::UnionTypes::ClientOrServiceWorkerOrMessagePort;
use crate::dom::bindings::error::Fallible;
use crate::dom::bindings::frozenarray::CachedFrozenArray;
use crate::dom::bindings::inheritance::Castable;
use crate::dom::bindings::root::{Dom, DomRoot};
use crate::dom::bindings::str::DOMString;
use crate::dom::bindings::trace::RootedTraceableBox;
use crate::dom::client::Client;
use crate::dom::event::Event;
use crate::dom::eventtarget::EventTarget;
use crate::dom::extendableevent::ExtendableEvent;
use crate::dom::globalscope::GlobalScope;
use crate::dom::messageport::MessagePort;
use crate::dom::serviceworker::ServiceWorker;
use crate::dom::serviceworkerglobalscope::ServiceWorkerGlobalScope;

/// <https://w3c.github.io/ServiceWorker/#dom-extendablemessageevent-source>
#[derive(JSTraceable, MallocSizeOf)]
#[cfg_attr(crown, crown::unrooted_must_root_lint::must_root)]
pub(crate) enum MessageSource {
    Client(Dom<Client>),
    ServiceWorker(Dom<ServiceWorker>),
    MessagePort(Dom<MessagePort>),
}

impl MessageSource {
    fn root(&self) -> MessageSourceRoot {
        match self {
            MessageSource::Client(client) => MessageSourceRoot::Client(DomRoot::from_ref(client)),
            MessageSource::ServiceWorker(sw) => {
                MessageSourceRoot::ServiceWorker(DomRoot::from_ref(sw))
            },
            MessageSource::MessagePort(port) => {
                MessageSourceRoot::MessagePort(DomRoot::from_ref(port))
            },
        }
    }
}

pub(crate) enum MessageSourceRoot {
    Client(DomRoot<Client>),
    ServiceWorker(DomRoot<ServiceWorker>),
    MessagePort(DomRoot<MessagePort>),
}

impl From<&ClientOrServiceWorkerOrMessagePort> for MessageSourceRoot {
    fn from(value: &ClientOrServiceWorkerOrMessagePort) -> Self {
        match value {
            ClientOrServiceWorkerOrMessagePort::Client(client) => {
                MessageSourceRoot::Client(DomRoot::from_ref(client))
            },
            ClientOrServiceWorkerOrMessagePort::ServiceWorker(sw) => {
                MessageSourceRoot::ServiceWorker(DomRoot::from_ref(sw))
            },
            ClientOrServiceWorkerOrMessagePort::MessagePort(port) => {
                MessageSourceRoot::MessagePort(DomRoot::from_ref(port))
            },
        }
    }
}

impl From<MessageSourceRoot> for ClientOrServiceWorkerOrMessagePort {
    fn from(value: MessageSourceRoot) -> Self {
        match value {
            MessageSourceRoot::Client(client) => ClientOrServiceWorkerOrMessagePort::Client(client),
            MessageSourceRoot::ServiceWorker(sw) => {
                ClientOrServiceWorkerOrMessagePort::ServiceWorker(sw)
            },
            MessageSourceRoot::MessagePort(port) => {
                ClientOrServiceWorkerOrMessagePort::MessagePort(port)
            },
        }
    }
}

#[dom_struct]
#[expect(non_snake_case)]
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
    /// <https://w3c.github.io/ServiceWorker/#dom-extendablemessageevent-source>
    source: Option<MessageSource>,
    /// <https://w3c.github.io/ServiceWorker/#dom-extendablemessageevent-ports>
    ports: Vec<Dom<MessagePort>>,
    #[ignore_malloc_size_of = "mozjs"]
    frozen_ports: CachedFrozenArray,
}

#[expect(non_snake_case)]
impl ExtendableMessageEvent {
    pub(crate) fn new_inherited(
        origin: DOMString,
        lastEventId: DOMString,
        source: Option<MessageSourceRoot>,
        ports: Vec<DomRoot<MessagePort>>,
    ) -> ExtendableMessageEvent {
        ExtendableMessageEvent {
            event: ExtendableEvent::new_inherited(),
            data: Heap::default(),
            origin,
            lastEventId,
            source: source.map(|source| match source {
                MessageSourceRoot::Client(client) => MessageSource::Client(client.as_traced()),
                MessageSourceRoot::ServiceWorker(sw) => {
                    MessageSource::ServiceWorker(sw.as_traced())
                },
                MessageSourceRoot::MessagePort(port) => {
                    MessageSource::MessagePort(port.as_traced())
                },
            }),
            ports: ports
                .into_iter()
                .map(|port| Dom::from_ref(&*port))
                .collect(),
            frozen_ports: CachedFrozenArray::new(),
        }
    }

    #[allow(clippy::too_many_arguments)]
    fn new(
        cx: &mut JSContext,
        global: &GlobalScope,
        type_: Atom,
        bubbles: bool,
        cancelable: bool,
        data: HandleValue,
        origin: DOMString,
        lastEventId: DOMString,
        source: Option<MessageSourceRoot>,
        ports: Vec<DomRoot<MessagePort>>,
    ) -> DomRoot<ExtendableMessageEvent> {
        Self::new_with_proto(
            cx,
            global,
            None,
            type_,
            bubbles,
            cancelable,
            data,
            origin,
            lastEventId,
            source,
            ports,
        )
    }

    #[allow(clippy::too_many_arguments)]
    fn new_with_proto(
        cx: &mut JSContext,
        global: &GlobalScope,
        proto: Option<HandleObject>,
        type_: Atom,
        bubbles: bool,
        cancelable: bool,
        data: HandleValue,
        origin: DOMString,
        lastEventId: DOMString,
        source: Option<MessageSourceRoot>,
        ports: Vec<DomRoot<MessagePort>>,
    ) -> DomRoot<ExtendableMessageEvent> {
        let ev = Box::new(ExtendableMessageEvent::new_inherited(
            origin,
            lastEventId,
            source,
            ports,
        ));
        let ev = reflect_dom_object_with_proto_and_cx(ev, global, proto, cx);
        {
            let event = ev.upcast::<Event>();
            event.init_event(type_, bubbles, cancelable);
        }
        ev.data.set(data.get());

        ev
    }

    pub(crate) fn dispatch_jsval(
        cx: &mut JSContext,
        target: &EventTarget,
        scope: &GlobalScope,
        message: HandleValue,
        source: Option<MessageSourceRoot>,
        ports: Vec<DomRoot<MessagePort>>,
    ) {
        let Extendablemessageevent = ExtendableMessageEvent::new(
            cx,
            scope,
            atom!("message"),
            false,
            false,
            message,
            DOMString::new(),
            DOMString::new(),
            source,
            ports,
        );
        Extendablemessageevent.upcast::<Event>().fire(cx, target);
    }

    pub(crate) fn dispatch_error(cx: &mut JSContext, target: &EventTarget, scope: &GlobalScope) {
        let ExtendableMsgEvent = ExtendableMessageEvent::new(
            cx,
            scope,
            atom!("messageerror"),
            false,
            false,
            HandleValue::null(),
            DOMString::new(),
            DOMString::new(),
            None,
            Vec::new(),
        );
        ExtendableMsgEvent.upcast::<Event>().fire(cx, target);
    }
}

impl ExtendableMessageEventMethods<crate::DomTypeHolder> for ExtendableMessageEvent {
    /// <https://w3c.github.io/ServiceWorker/#dom-extendablemessageevent-extendablemessageevent>
    fn Constructor(
        cx: &mut JSContext,
        worker: &ServiceWorkerGlobalScope,
        proto: Option<HandleObject>,
        type_: DOMString,
        init: RootedTraceableBox<ExtendableMessageEventBinding::ExtendableMessageEventInit>,
    ) -> Fallible<DomRoot<ExtendableMessageEvent>> {
        let global = worker.upcast::<GlobalScope>();
        let ev = ExtendableMessageEvent::new_with_proto(
            cx,
            global,
            proto,
            Atom::from(type_),
            init.parent.parent.bubbles,
            init.parent.parent.cancelable,
            init.data.handle(),
            init.origin.clone(),
            init.lastEventId.clone(),
            init.source
                .as_ref()
                .and_then(|s| s.as_ref().map(|s| s.into())),
            vec![],
        );
        Ok(ev)
    }

    /// <https://w3c.github.io/ServiceWorker/#dom-extendablemessageevent-data>
    fn Data(&self, _cx: &mut JSContext, mut retval: MutableHandleValue) {
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

    /// <https://w3c.github.io/ServiceWorker/#dom-extendablemessageevent-source>
    fn GetSource(&self) -> Option<ClientOrServiceWorkerOrMessagePort> {
        self.source
            .as_ref()
            .map(|source| source.root())
            .map(|s| s.into())
    }

    /// <https://w3c.github.io/ServiceWorker/#extendablemessage-event-ports>
    fn Ports(&self, cx: &mut JSContext, retval: MutableHandleValue) {
        self.frozen_ports.get_or_init(
            cx,
            || {
                self.ports
                    .iter()
                    .map(|port| DomRoot::from_ref(&**port))
                    .collect()
            },
            retval,
        );
    }
}
