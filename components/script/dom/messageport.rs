/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use std::cell::{Cell, RefCell};
use std::collections::HashMap;
use std::num::NonZeroU32;
use std::rc::Rc;

use base::id::{MessagePortId, MessagePortIndex, PipelineNamespaceId};
use constellation_traits::{MessagePortImpl, PortMessageTask};
use dom_struct::dom_struct;
use js::jsapi::{Heap, JSObject};
use js::rust::{CustomAutoRooter, CustomAutoRooterGuard, HandleValue};

use crate::dom::bindings::codegen::Bindings::EventHandlerBinding::EventHandlerNonNull;
use crate::dom::bindings::codegen::Bindings::MessagePortBinding::{
    MessagePortMethods, StructuredSerializeOptions,
};
use crate::dom::bindings::conversions::root_from_object;
use crate::dom::bindings::error::{Error, ErrorResult};
use crate::dom::bindings::inheritance::Castable;
use crate::dom::bindings::reflector::{DomGlobal, reflect_dom_object};
use crate::dom::bindings::root::DomRoot;
use crate::dom::bindings::structuredclone::{self, StructuredData, StructuredDataReader};
use crate::dom::bindings::trace::RootedTraceableBox;
use crate::dom::bindings::transferable::{ExtractComponents, IdFromComponents, Transferable};
use crate::dom::eventtarget::EventTarget;
use crate::dom::globalscope::GlobalScope;
use crate::script_runtime::{CanGc, JSContext as SafeJSContext};

#[dom_struct]
/// The MessagePort used in the DOM.
pub(crate) struct MessagePort {
    eventtarget: EventTarget,
    #[no_trace]
    message_port_id: MessagePortId,
    #[no_trace]
    entangled_port: RefCell<Option<MessagePortId>>,
    detached: Cell<bool>,
}

impl MessagePort {
    fn new_inherited(message_port_id: MessagePortId) -> MessagePort {
        MessagePort {
            eventtarget: EventTarget::new_inherited(),
            entangled_port: RefCell::new(None),
            detached: Cell::new(false),
            message_port_id,
        }
    }

    /// <https://html.spec.whatwg.org/multipage/#create-a-new-messageport-object>
    pub(crate) fn new(owner: &GlobalScope, can_gc: CanGc) -> DomRoot<MessagePort> {
        let port_id = MessagePortId::new();
        reflect_dom_object(Box::new(MessagePort::new_inherited(port_id)), owner, can_gc)
    }

    /// Create a new port for an incoming transfer-received one.
    fn new_transferred(
        owner: &GlobalScope,
        transferred_port: MessagePortId,
        entangled_port: Option<MessagePortId>,
        can_gc: CanGc,
    ) -> DomRoot<MessagePort> {
        reflect_dom_object(
            Box::new(MessagePort {
                message_port_id: transferred_port,
                eventtarget: EventTarget::new_inherited(),
                detached: Cell::new(false),
                entangled_port: RefCell::new(entangled_port),
            }),
            owner,
            can_gc,
        )
    }

    /// <https://html.spec.whatwg.org/multipage/#entangle>
    pub(crate) fn entangle(&self, other_id: MessagePortId) {
        *self.entangled_port.borrow_mut() = Some(other_id);
    }

    pub(crate) fn message_port_id(&self) -> &MessagePortId {
        &self.message_port_id
    }

    pub(crate) fn detached(&self) -> bool {
        self.detached.get()
    }

    /// <https://html.spec.whatwg.org/multipage/#handler-messageport-onmessage>
    fn set_onmessage(&self, listener: Option<Rc<EventHandlerNonNull>>) {
        let eventtarget = self.upcast::<EventTarget>();
        eventtarget.set_event_handler_common("message", listener);
    }

    /// <https://html.spec.whatwg.org/multipage/#message-port-post-message-steps>
    #[allow(unsafe_code)]
    fn post_message_impl(
        &self,
        cx: SafeJSContext,
        message: HandleValue,
        transfer: CustomAutoRooterGuard<Vec<*mut JSObject>>,
    ) -> ErrorResult {
        if self.detached.get() {
            return Ok(());
        }

        // Step 1 is the transfer argument.

        let target_port = self.entangled_port.borrow();

        // Step 3
        let mut doomed = false;

        let ports = transfer
            .iter()
            .filter_map(|&obj| unsafe { root_from_object::<MessagePort>(obj, *cx).ok() });
        for port in ports {
            // Step 2
            if port.message_port_id() == self.message_port_id() {
                return Err(Error::DataClone);
            }

            // Step 4
            if let Some(target_id) = target_port.as_ref() {
                if port.message_port_id() == target_id {
                    doomed = true;
                }
            }
        }

        // Step 5
        let data = structuredclone::write(cx, message, Some(transfer))?;

        if doomed {
            // TODO: The spec says to optionally report such a case to a dev console.
            return Ok(());
        }

        // Step 6, done in MessagePortImpl.

        let incumbent = match GlobalScope::incumbent() {
            None => unreachable!("postMessage called with no incumbent global"),
            Some(incumbent) => incumbent,
        };

        // Step 7
        let task = PortMessageTask {
            origin: incumbent.origin().immutable().clone(),
            data,
        };

        // Have the global proxy this call to the corresponding MessagePortImpl.
        self.global()
            .post_messageport_msg(*self.message_port_id(), task);
        Ok(())
    }
}

impl Transferable for MessagePort {
    type Id = MessagePortId;
    type Data = MessagePortImpl;

    /// <https://html.spec.whatwg.org/multipage/#message-ports:transfer-steps>
    fn transfer(&self) -> Result<(MessagePortId, MessagePortImpl), ()> {
        if self.detached.get() {
            return Err(());
        }

        self.detached.set(true);
        let id = self.message_port_id();

        // 1. Run local transfer logic, and return the object to be transferred.
        let transferred_port = self.global().mark_port_as_transferred(id);

        Ok((*id, transferred_port))
    }

    /// <https://html.spec.whatwg.org/multipage/#message-ports:transfer-receiving-steps>
    fn transfer_receive(
        owner: &GlobalScope,
        id: MessagePortId,
        port_impl: MessagePortImpl,
    ) -> Result<DomRoot<Self>, ()> {
        let transferred_port =
            MessagePort::new_transferred(owner, id, port_impl.entangled_port_id(), CanGc::note());
        owner.track_message_port(&transferred_port, Some(port_impl));
        Ok(transferred_port)
    }

    fn serialized_storage(data: StructuredData<'_>) -> &mut Option<HashMap<Self::Id, Self::Data>> {
        match data {
            StructuredData::Reader(r) => &mut r.port_impls,
            StructuredData::Writer(w) => &mut w.ports,
        }
    }

    fn deserialized_storage(reader: &mut StructuredDataReader) -> &mut Option<Vec<DomRoot<Self>>> {
        &mut reader.message_ports
    }
}

impl IdFromComponents for MessagePortId {
    fn from(namespace_id: PipelineNamespaceId, index: NonZeroU32) -> MessagePortId {
        MessagePortId {
            namespace_id,
            index: MessagePortIndex(index),
        }
    }
}

impl ExtractComponents for MessagePortId {
    fn components(&self) -> (PipelineNamespaceId, NonZeroU32) {
        (self.namespace_id, self.index.0)
    }
}

impl MessagePortMethods<crate::DomTypeHolder> for MessagePort {
    /// <https://html.spec.whatwg.org/multipage/#dom-messageport-postmessage>
    fn PostMessage(
        &self,
        cx: SafeJSContext,
        message: HandleValue,
        transfer: CustomAutoRooterGuard<Vec<*mut JSObject>>,
    ) -> ErrorResult {
        if self.detached.get() {
            return Ok(());
        }
        self.post_message_impl(cx, message, transfer)
    }

    /// <https://html.spec.whatwg.org/multipage/#dom-messageport-postmessage>
    fn PostMessage_(
        &self,
        cx: SafeJSContext,
        message: HandleValue,
        options: RootedTraceableBox<StructuredSerializeOptions>,
    ) -> ErrorResult {
        if self.detached.get() {
            return Ok(());
        }
        let mut rooted = CustomAutoRooter::new(
            options
                .transfer
                .iter()
                .map(|js: &RootedTraceableBox<Heap<*mut JSObject>>| js.get())
                .collect(),
        );
        let guard = CustomAutoRooterGuard::new(*cx, &mut rooted);
        self.post_message_impl(cx, message, guard)
    }

    /// <https://html.spec.whatwg.org/multipage/#dom-messageport-start>
    fn Start(&self) {
        if self.detached.get() {
            return;
        }
        self.global().start_message_port(self.message_port_id());
    }

    /// <https://html.spec.whatwg.org/multipage/#dom-messageport-close>
    fn Close(&self) {
        if self.detached.get() {
            return;
        }
        self.detached.set(true);
        self.global().close_message_port(self.message_port_id());
    }

    /// <https://html.spec.whatwg.org/multipage/#handler-messageport-onmessage>
    fn GetOnmessage(&self, can_gc: CanGc) -> Option<Rc<EventHandlerNonNull>> {
        if self.detached.get() {
            return None;
        }
        let eventtarget = self.upcast::<EventTarget>();
        eventtarget.get_event_handler_common("message", can_gc)
    }

    /// <https://html.spec.whatwg.org/multipage/#handler-messageport-onmessage>
    fn SetOnmessage(&self, listener: Option<Rc<EventHandlerNonNull>>) {
        if self.detached.get() {
            return;
        }
        self.set_onmessage(listener);
        // Note: we cannot use the event_handler macro, due to the need to start the port.
        self.global().start_message_port(self.message_port_id());
    }

    // <https://html.spec.whatwg.org/multipage/#handler-messageport-onmessageerror>
    event_handler!(messageerror, GetOnmessageerror, SetOnmessageerror);
}
