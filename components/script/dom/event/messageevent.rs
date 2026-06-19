/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use dom_struct::dom_struct;
use js::context::JSContext;
use js::jsapi::Heap;
use js::jsval::JSVal;
use js::rust::{HandleObject, HandleValue, MutableHandleValue};
use script_bindings::cell::DomRefCell;
use script_bindings::reflector::reflect_dom_object_with_proto_and_cx;
use stylo_atoms::Atom;

use crate::dom::bindings::codegen::Bindings::EventBinding::EventMethods;
use crate::dom::bindings::codegen::Bindings::MessageEventBinding;
use crate::dom::bindings::codegen::Bindings::MessageEventBinding::MessageEventMethods;
use crate::dom::bindings::codegen::UnionTypes::WindowProxyOrMessagePortOrServiceWorker;
use crate::dom::bindings::error::Fallible;
use crate::dom::bindings::frozenarray::CachedFrozenArray;
use crate::dom::bindings::inheritance::Castable;
use crate::dom::bindings::root::{Dom, DomRoot};
use crate::dom::bindings::str::DOMString;
use crate::dom::bindings::trace::RootedTraceableBox;
use crate::dom::event::Event;
use crate::dom::eventtarget::EventTarget;
use crate::dom::globalscope::GlobalScope;
use crate::dom::messageport::MessagePort;
use crate::dom::serviceworker::ServiceWorker;
use crate::dom::windowproxy::WindowProxy;

#[cfg_attr(crown, crown::unrooted_must_root_lint::must_root)]
#[derive(JSTraceable, MallocSizeOf)]
enum SrcObject {
    WindowProxy(Dom<WindowProxy>),
    MessagePort(Dom<MessagePort>),
    ServiceWorker(Dom<ServiceWorker>),
}

impl From<&WindowProxyOrMessagePortOrServiceWorker> for SrcObject {
    fn from(src_object: &WindowProxyOrMessagePortOrServiceWorker) -> SrcObject {
        match src_object {
            WindowProxyOrMessagePortOrServiceWorker::WindowProxy(blob) => {
                SrcObject::WindowProxy(Dom::from_ref(blob))
            },
            WindowProxyOrMessagePortOrServiceWorker::MessagePort(stream) => {
                SrcObject::MessagePort(Dom::from_ref(stream))
            },
            WindowProxyOrMessagePortOrServiceWorker::ServiceWorker(stream) => {
                SrcObject::ServiceWorker(Dom::from_ref(stream))
            },
        }
    }
}

#[dom_struct]
#[expect(non_snake_case)]
pub(crate) struct MessageEvent {
    event: Event,
    #[ignore_malloc_size_of = "mozjs"]
    data: Heap<JSVal>,
    origin: DomRefCell<DOMString>,
    source: DomRefCell<Option<SrcObject>>,
    lastEventId: DomRefCell<DOMString>,
    ports: DomRefCell<Vec<Dom<MessagePort>>>,
    #[ignore_malloc_size_of = "mozjs"]
    frozen_ports: CachedFrozenArray,
}

#[expect(non_snake_case)]
impl MessageEvent {
    pub(crate) fn new_inherited(
        origin: DOMString,
        source: Option<&WindowProxyOrMessagePortOrServiceWorker>,
        lastEventId: DOMString,
        ports: Vec<DomRoot<MessagePort>>,
    ) -> MessageEvent {
        MessageEvent {
            event: Event::new_inherited(),
            data: Heap::default(),
            source: DomRefCell::new(source.map(|source| source.into())),
            origin: DomRefCell::new(origin),
            lastEventId: DomRefCell::new(lastEventId),
            ports: DomRefCell::new(
                ports
                    .into_iter()
                    .map(|port| Dom::from_ref(&*port))
                    .collect(),
            ),
            frozen_ports: CachedFrozenArray::new(),
        }
    }

    pub(crate) fn new_uninitialized(
        cx: &mut JSContext,
        global: &GlobalScope,
    ) -> DomRoot<MessageEvent> {
        Self::new_uninitialized_with_proto(cx, global, None)
    }

    fn new_uninitialized_with_proto(
        cx: &mut JSContext,
        global: &GlobalScope,
        proto: Option<HandleObject>,
    ) -> DomRoot<MessageEvent> {
        MessageEvent::new_initialized(
            cx,
            global,
            proto,
            HandleValue::undefined(),
            DOMString::new(),
            None,
            DOMString::new(),
            vec![],
        )
    }

    #[allow(clippy::too_many_arguments)]
    fn new_initialized(
        cx: &mut JSContext,
        global: &GlobalScope,
        proto: Option<HandleObject>,
        data: HandleValue,
        origin: DOMString,
        source: Option<&WindowProxyOrMessagePortOrServiceWorker>,
        lastEventId: DOMString,
        ports: Vec<DomRoot<MessagePort>>,
    ) -> DomRoot<MessageEvent> {
        let ev = Box::new(MessageEvent::new_inherited(
            origin,
            source,
            lastEventId,
            ports,
        ));
        let ev = reflect_dom_object_with_proto_and_cx(ev, global, proto, cx);
        ev.data.set(data.get());

        ev
    }

    #[allow(clippy::too_many_arguments)]
    pub(crate) fn new(
        cx: &mut JSContext,
        global: &GlobalScope,
        type_: Atom,
        bubbles: bool,
        cancelable: bool,
        data: HandleValue,
        origin: DOMString,
        source: Option<&WindowProxyOrMessagePortOrServiceWorker>,
        lastEventId: DOMString,
        ports: Vec<DomRoot<MessagePort>>,
    ) -> DomRoot<MessageEvent> {
        Self::new_with_proto(
            cx,
            global,
            None,
            type_,
            bubbles,
            cancelable,
            data,
            origin,
            source,
            lastEventId,
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
        source: Option<&WindowProxyOrMessagePortOrServiceWorker>,
        lastEventId: DOMString,
        ports: Vec<DomRoot<MessagePort>>,
    ) -> DomRoot<MessageEvent> {
        let ev = MessageEvent::new_initialized(
            cx,
            global,
            proto,
            data,
            origin,
            source,
            lastEventId,
            ports,
        );
        {
            let event = ev.upcast::<Event>();
            event.init_event(type_, bubbles, cancelable);
        }
        ev
    }

    pub(crate) fn dispatch_jsval(
        cx: &mut JSContext,
        target: &EventTarget,
        scope: &GlobalScope,
        message: HandleValue,
        origin: Option<&str>,
        source: Option<&WindowProxy>,
        ports: Vec<DomRoot<MessagePort>>,
    ) {
        let messageevent = MessageEvent::new(
            cx,
            scope,
            atom!("message"),
            false,
            false,
            message,
            DOMString::from(origin.unwrap_or("")),
            source
                .map(|source| {
                    WindowProxyOrMessagePortOrServiceWorker::WindowProxy(DomRoot::from_ref(source))
                })
                .as_ref(),
            DOMString::new(),
            ports,
        );
        messageevent.upcast::<Event>().fire(cx, target);
    }

    pub(crate) fn dispatch_error(cx: &mut JSContext, target: &EventTarget, scope: &GlobalScope) {
        let init = MessageEventBinding::MessageEventInit::empty();
        let messageevent = MessageEvent::new(
            cx,
            scope,
            atom!("messageerror"),
            init.parent.bubbles,
            init.parent.cancelable,
            init.data.handle(),
            init.origin.clone(),
            init.source.as_ref(),
            init.lastEventId.clone(),
            init.ports.clone(),
        );
        messageevent.upcast::<Event>().fire(cx, target);
    }
}

impl MessageEventMethods<crate::DomTypeHolder> for MessageEvent {
    /// <https://html.spec.whatwg.org/multipage/#messageevent>
    fn Constructor(
        cx: &mut JSContext,
        global: &GlobalScope,
        proto: Option<HandleObject>,

        type_: DOMString,
        init: RootedTraceableBox<MessageEventBinding::MessageEventInit>,
    ) -> Fallible<DomRoot<MessageEvent>> {
        let ev = MessageEvent::new_with_proto(
            cx,
            global,
            proto,
            Atom::from(type_),
            init.parent.bubbles,
            init.parent.cancelable,
            init.data.handle(),
            init.origin.clone(),
            init.source.as_ref(),
            init.lastEventId.clone(),
            init.ports.clone(),
        );
        Ok(ev)
    }

    /// <https://html.spec.whatwg.org/multipage/#dom-messageevent-data>
    fn Data(&self, _cx: &mut JSContext, mut retval: MutableHandleValue) {
        retval.set(self.data.get())
    }

    /// <https://html.spec.whatwg.org/multipage/#dom-messageevent-origin>
    fn Origin(&self) -> DOMString {
        self.origin.borrow().clone()
    }

    /// <https://html.spec.whatwg.org/multipage/#dom-messageevent-source>
    fn GetSource(&self) -> Option<WindowProxyOrMessagePortOrServiceWorker> {
        match &*self.source.borrow() {
            Some(SrcObject::WindowProxy(i)) => Some(
                WindowProxyOrMessagePortOrServiceWorker::WindowProxy(DomRoot::from_ref(i)),
            ),
            Some(SrcObject::MessagePort(i)) => Some(
                WindowProxyOrMessagePortOrServiceWorker::MessagePort(DomRoot::from_ref(i)),
            ),
            Some(SrcObject::ServiceWorker(i)) => Some(
                WindowProxyOrMessagePortOrServiceWorker::ServiceWorker(DomRoot::from_ref(i)),
            ),
            None => None,
        }
    }

    /// <https://html.spec.whatwg.org/multipage/#dom-messageevent-lasteventid>
    fn LastEventId(&self) -> DOMString {
        self.lastEventId.borrow().clone()
    }

    /// <https://dom.spec.whatwg.org/#dom-event-istrusted>
    fn IsTrusted(&self) -> bool {
        self.event.IsTrusted()
    }

    /// <https://html.spec.whatwg.org/multipage/#dom-messageevent-ports>
    fn Ports(&self, cx: &mut JSContext, retval: MutableHandleValue) {
        self.frozen_ports.get_or_init(
            cx,
            || {
                self.ports
                    .borrow()
                    .iter()
                    .map(|port| DomRoot::from_ref(&**port))
                    .collect()
            },
            retval,
        );
    }

    /// <https://html.spec.whatwg.org/multipage/#dom-messageevent-initmessageevent>
    #[expect(non_snake_case)]
    fn InitMessageEvent(
        &self,
        _cx: &mut JSContext,
        type_: DOMString,
        bubbles: bool,
        cancelable: bool,
        data: HandleValue,
        origin: DOMString,
        lastEventId: DOMString,
        source: Option<WindowProxyOrMessagePortOrServiceWorker>,
        ports: Vec<DomRoot<MessagePort>>,
    ) {
        self.data.set(data.get());
        *self.origin.borrow_mut() = origin;
        *self.source.borrow_mut() = source.as_ref().map(|source| source.into());
        *self.lastEventId.borrow_mut() = lastEventId;
        *self.ports.borrow_mut() = ports
            .into_iter()
            .map(|port| Dom::from_ref(&*port))
            .collect();
        self.frozen_ports.clear();
        self.event
            .init_event(Atom::from(type_), bubbles, cancelable);
    }
}
