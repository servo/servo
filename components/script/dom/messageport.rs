/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use std::cell::{Cell, RefCell};
use std::collections::HashMap;
use std::convert::TryInto;
use std::num::NonZeroU32;
use std::rc::Rc;

use base::id::{MessagePortId, MessagePortIndex, PipelineNamespaceId};
use dom_struct::dom_struct;
use js::jsapi::{Heap, JSObject, MutableHandleObject};
use js::rust::{CustomAutoRooter, CustomAutoRooterGuard, HandleValue};
use script_traits::PortMessageTask;

use crate::dom::bindings::codegen::Bindings::EventHandlerBinding::EventHandlerNonNull;
use crate::dom::bindings::codegen::Bindings::MessagePortBinding::{
    MessagePortMethods, StructuredSerializeOptions,
};
use crate::dom::bindings::conversions::root_from_object;
use crate::dom::bindings::error::{Error, ErrorResult};
use crate::dom::bindings::inheritance::Castable;
use crate::dom::bindings::reflector::{reflect_dom_object, DomGlobal, DomObject};
use crate::dom::bindings::root::DomRoot;
use crate::dom::bindings::structuredclone::{self, StructuredDataReader, StructuredDataWriter};
use crate::dom::bindings::trace::RootedTraceableBox;
use crate::dom::bindings::transferable::Transferable;
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
            .filter_map(|&obj| root_from_object::<MessagePort>(obj, *cx).ok());
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
    /// <https://html.spec.whatwg.org/multipage/#message-ports:transfer-steps>
    fn transfer(&self, sc_writer: &mut StructuredDataWriter) -> Result<u64, ()> {
        if self.detached.get() {
            return Err(());
        }

        self.detached.set(true);
        let id = self.message_port_id();

        // 1. Run local transfer logic, and return the object to be transferred.
        let transferred_port = self.global().mark_port_as_transferred(id);

        // 2. Store the transferred object at a given key.
        if let Some(ports) = sc_writer.ports.as_mut() {
            ports.insert(*id, transferred_port);
        } else {
            let mut ports = HashMap::new();
            ports.insert(*id, transferred_port);
            sc_writer.ports = Some(ports);
        }

        let PipelineNamespaceId(name_space) = (id).namespace_id;
        let MessagePortIndex(index) = (id).index;
        let index = index.get();

        let mut big: [u8; 8] = [0; 8];
        let name_space = name_space.to_ne_bytes();
        let index = index.to_ne_bytes();

        let (left, right) = big.split_at_mut(4);
        left.copy_from_slice(&name_space);
        right.copy_from_slice(&index);

        // 3. Return a u64 representation of the key where the object is stored.
        Ok(u64::from_ne_bytes(big))
    }

    /// <https://html.spec.whatwg.org/multipage/#message-ports:transfer-receiving-steps>
    fn transfer_receive(
        owner: &GlobalScope,
        sc_reader: &mut StructuredDataReader,
        extra_data: u64,
        return_object: MutableHandleObject,
    ) -> Result<(), ()> {
        // 1. Re-build the key for the storage location
        // of the transferred object.
        let big: [u8; 8] = extra_data.to_ne_bytes();
        let (name_space, index) = big.split_at(4);

        let namespace_id = PipelineNamespaceId(u32::from_ne_bytes(
            name_space
                .try_into()
                .expect("name_space to be a slice of four."),
        ));
        let index = MessagePortIndex(
            NonZeroU32::new(u32::from_ne_bytes(
                index.try_into().expect("index to be a slice of four."),
            ))
            .expect("Index to be non-zero"),
        );

        let id = MessagePortId {
            namespace_id,
            index,
        };

        // 2. Get the transferred object from its storage, using the key.
        // Assign the transfer-received port-impl, and total number of transferred ports.
        let (ports_len, port_impl) = if let Some(ports) = sc_reader.port_impls.as_mut() {
            let ports_len = ports.len();
            let port_impl = ports.remove(&id).expect("Transferred port to be stored");
            if ports.is_empty() {
                sc_reader.port_impls = None;
            }
            (ports_len, port_impl)
        } else {
            panic!("A messageport was transfer-received, yet the SC holder does not have any port impls");
        };

        let transferred_port =
            MessagePort::new_transferred(owner, id, port_impl.entangled_port_id(), CanGc::note());
        owner.track_message_port(&transferred_port, Some(port_impl));

        return_object.set(transferred_port.reflector().rootable().get());

        // Store the DOM port where it will be passed along to script in the message-event.
        if let Some(ports) = sc_reader.message_ports.as_mut() {
            ports.push(transferred_port);
        } else {
            let mut ports = Vec::with_capacity(ports_len);
            ports.push(transferred_port);
            sc_reader.message_ports = Some(ports);
        }

        Ok(())
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
