/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use crate::dom::bindings::codegen::Bindings::EventHandlerBinding::EventHandlerNonNull;
use crate::dom::bindings::codegen::Bindings::MessagePortBinding::{
    MessagePortMethods, PostMessageOptions, Wrap,
};
use crate::dom::bindings::conversions::root_from_object;
use crate::dom::bindings::error::{Error, ErrorResult};
use crate::dom::bindings::inheritance::{Castable, HasParent};
use crate::dom::bindings::refcounted::Trusted;
use crate::dom::bindings::reflector::{reflect_dom_object, DomObject};
use crate::dom::bindings::root::DomRoot;
use crate::dom::bindings::structuredclone::{self, StructuredCloneHolder};
use crate::dom::bindings::trace::{JSTraceable, RootedTraceableBox};
use crate::dom::bindings::transferable::Transferable;
use crate::dom::eventtarget::EventTarget;
use crate::dom::globalscope::GlobalScope;
use crate::script_runtime::JSContext as SafeJSContext;
use crate::task_source::TaskSource;
use js::jsapi::Heap;
use js::jsapi::{JSObject, JSTracer, MutableHandleObject};
use js::rust::{CustomAutoRooter, CustomAutoRooterGuard, HandleValue};
use msg::constellation_msg::{
    MessagePortId, MessagePortIndex, PipelineNamespaceId, PortMessageTask,
};
use std::cell::{Cell, RefCell};
use std::collections::VecDeque;
use std::convert::TryInto;
use std::num::NonZeroU32;
use std::rc::Rc;

#[derive(DenyPublicFields, DomObject, MallocSizeOf)]
#[must_root]
#[repr(C)]
/// The MessagePort used in the DOM.
pub struct MessagePort {
    eventtarget: EventTarget,
    message_port_id: MessagePortId,
    entangled_port: RefCell<Option<MessagePortId>>,
    detached: Cell<bool>,
}

#[allow(unsafe_code)]
unsafe impl JSTraceable for MessagePort {
    unsafe fn trace(&self, trc: *mut JSTracer) {
        if !self.detached.get() {
            self.eventtarget.trace(trc);
        }
    }
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
    pub fn new(owner: &GlobalScope) -> DomRoot<MessagePort> {
        let port_id = MessagePortId::new();
        reflect_dom_object(Box::new(MessagePort::new_inherited(port_id)), owner, Wrap)
    }

    /// Create a new port for an incoming transfer-received one.
    /// Using an existing Id and setting transferred to true.
    fn new_transferred(
        owner: &GlobalScope,
        transferred_port: MessagePortId,
        entangled_port: Option<MessagePortId>,
    ) -> DomRoot<MessagePort> {
        reflect_dom_object(
            Box::new(MessagePort {
                message_port_id: transferred_port,
                eventtarget: EventTarget::new_inherited(),
                detached: Cell::new(false),
                entangled_port: RefCell::new(entangled_port),
            }),
            owner,
            Wrap,
        )
    }

    /// <https://html.spec.whatwg.org/multipage/#entangle>
    pub fn entangle(&self, other_id: MessagePortId) {
        *self.entangled_port.borrow_mut() = Some(other_id);
    }

    pub fn message_port_id(&self) -> &MessagePortId {
        &self.message_port_id
    }

    /// <https://html.spec.whatwg.org/multipage/#handler-messageport-onmessage>
    pub fn set_onmessage(&self, listener: Option<Rc<EventHandlerNonNull>>) {
        let eventtarget = self.upcast::<EventTarget>();
        eventtarget.set_event_handler_common("message", listener);
    }

    pub fn detached(&self) -> bool {
        self.detached.get()
    }

    /// <https://html.spec.whatwg.org/multipage/#message-port-post-message-steps>
    pub fn post_message_impl(
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
        let data = structuredclone::write(*cx, message, Some(transfer))?;

        if doomed {
            // TODO: The spec says to optionally report such a case to a dev console.
            return Ok(());
        }

        // Step 6, done in MessagePortImpl.

        // Step 7
        let task = PortMessageTask {
            origin: self.global().origin().immutable().clone(),
            data,
        };

        // Have the global proxy this call to the corresponding MessagePortImpl.
        self.global()
            .post_messageport_msg(self.message_port_id().clone(), task);
        Ok(())
    }
}

impl HasParent for MessagePort {
    type Parent = EventTarget;

    fn as_parent(&self) -> &EventTarget {
        &self.eventtarget
    }
}

#[derive(Debug, DenyPublicFields, Deserialize, MallocSizeOf, Serialize)]
/// The data and logic backing the DOM managed MessagePort.
pub struct MessagePortImpl {
    detached: Cell<bool>,
    enabled: Cell<bool>,
    awaiting_transfer: Cell<bool>,
    entangled_port: RefCell<Option<MessagePortId>>,
    message_buffer: RefCell<VecDeque<PortMessageTask>>,
    has_been_shipped: Cell<bool>,
    message_port_id: MessagePortId,
}

impl MessagePortImpl {
    pub fn new(port_id: MessagePortId) -> MessagePortImpl {
        MessagePortImpl {
            detached: Cell::new(false),
            enabled: Cell::new(false),
            awaiting_transfer: Cell::new(false),
            entangled_port: RefCell::new(None),
            message_buffer: RefCell::new(VecDeque::new()),
            has_been_shipped: Cell::new(false),
            message_port_id: port_id,
        }
    }

    pub fn message_port_id(&self) -> &MessagePortId {
        &self.message_port_id
    }

    pub fn entangled_port_id(&self) -> Option<MessagePortId> {
        self.entangled_port.borrow().clone()
    }

    pub fn entangle(&self, other_id: MessagePortId) {
        *self.entangled_port.borrow_mut() = Some(other_id);
    }

    pub fn enabled(&self) -> bool {
        self.enabled.get()
    }

    pub fn set_has_been_shipped(&self) {
        self.has_been_shipped.set(true);
        self.awaiting_transfer.set(true);
    }

    /// Handle the completion of the transfer,
    /// this is data received from the constellation.
    pub fn complete_transfer(&self, tasks: Option<VecDeque<PortMessageTask>>) {
        if self.detached.get() {
            return;
        }
        self.awaiting_transfer.set(false);

        if let Some(mut tasks) = tasks {
            // Note: these are the tasks that were buffered while the transfer was ongoing,
            // hence they need to execute first.
            // The global will call `start` if we are enabled,
            // which will add tasks on the event-loop to dispatch incoming messages.
            let mut incoming_buffer = self.message_buffer.borrow_mut();
            while let Some(task) = tasks.pop_back() {
                incoming_buffer.push_front(task);
            }
        }
    }

    /// A message was received from our entangled port.
    pub fn handle_incoming(&self, task: &PortMessageTask) -> bool {
        if self.detached.get() {
            return false;
        }

        if self.enabled.get() && !self.awaiting_transfer.get() {
            true
        } else {
            self.message_buffer.borrow_mut().push_back(task.clone());
            false
        }
    }

    /// <https://html.spec.whatwg.org/multipage/#message-port-post-message-steps>
    // Steps 6 and 7
    pub fn post_message(&self, owner: &GlobalScope, task: PortMessageTask) {
        // Note: we do not assert here, as Step 6 simply returns in this case.
        let target_port_id = match *self.entangled_port.borrow() {
            Some(port_id) => port_id.clone(),
            None => return,
        };

        // Step 7
        let this = Trusted::new(&*owner);
        let _ = owner.port_message_queue().queue(
            task!(post_message: move || {
                let global = this.root();
                // Note: we do this in a task, as this will ensure the global and constellation
                // are aware of any transfer that might still take place in the current task.
                global.upcast::<GlobalScope>().route_task_to_port(target_port_id, task);
            }),
            owner,
        );
    }

    /// <https://html.spec.whatwg.org/multipage/#dom-messageport-start>
    pub fn start(&self, owner: &GlobalScope) {
        self.enabled.set(true);
        if self.awaiting_transfer.get() {
            return;
        }
        let port_id = self.message_port_id().clone();
        for task in self.message_buffer.borrow_mut().drain(0..) {
            let this = Trusted::new(&*owner);
            let _ = owner.port_message_queue().queue(
                task!(process_pending_port_messages: move || {
                    let target_global = this.root();
                    target_global.upcast::<GlobalScope>().route_task_to_port(port_id, task);
                }),
                &owner,
            );
        }
    }

    /// <https://html.spec.whatwg.org/multipage/#dom-messageport-close>
    pub fn close(&self) {
        // Step 1
        self.detached.set(true);
    }
}

impl Transferable for MessagePort {
    /// <https://html.spec.whatwg.org/multipage/#message-ports:transfer-steps>
    fn transfer(&self, sc_holder: &mut StructuredCloneHolder) -> Result<u64, ()> {
        self.detached.set(true);
        let id = self.message_port_id();
        // 1. Run local transfer logic, and return a serialized object for transfer.
        if let Ok(port) = self.global().mark_port_as_transferred(id) {
            // 2. Store the serialized object to be transferred,
            // at a given key.
            sc_holder.ports_impl.insert(id.clone(), port);

            let PipelineNamespaceId(name_space) = id.clone().namespace_id;
            let MessagePortIndex(index) = id.clone().index;
            let index = index.get();

            let mut big: [u8; 8] = [0; 8];
            let name_space = name_space.to_ne_bytes();
            let index = index.to_ne_bytes();

            let (left, right) = big.split_at_mut(4);
            left.copy_from_slice(&name_space);
            right.copy_from_slice(&index);
            // 3. Return a u64 representation of the key where the object is stored.
            return Ok(u64::from_ne_bytes(big));
        }
        Err(())
    }

    /// https://html.spec.whatwg.org/multipage/#message-ports:transfer-receiving-steps
    fn transfer_receive(
        owner: &DomRoot<GlobalScope>,
        sc_holder: &mut StructuredCloneHolder,
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

        // 2. Get the transferred object from it's storage, using the key.
        let port_impl_data = sc_holder
            .ports_impl
            .remove(&id)
            .expect("Transferred port to be stored");

        // 3. Deserialize the object that is to be transfered in to this realm.
        let port_impl: MessagePortImpl = bincode::deserialize(&port_impl_data[..])
            .expect("MessagePortImpl to be desirealizeable");

        let transferred_port =
            MessagePort::new_transferred(&**owner, id.clone(), port_impl.entangled_port_id());
        owner.track_message_port(&transferred_port, Some(port_impl));

        return_object.set(transferred_port.reflector().rootable().get());

        sc_holder.message_ports.push(transferred_port);

        Ok(())
    }
}

impl MessagePortMethods for MessagePort {
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
        options: RootedTraceableBox<PostMessageOptions>,
    ) -> ErrorResult {
        if self.detached.get() {
            return Ok(());
        }
        let mut rooted = CustomAutoRooter::new(
            options
                .transfer
                .as_ref()
                .unwrap_or(&Vec::new())
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
    fn GetOnmessage(&self) -> Option<Rc<EventHandlerNonNull>> {
        if self.detached.get() {
            return None;
        }
        let eventtarget = self.upcast::<EventTarget>();
        eventtarget.get_event_handler_common("message")
    }

    /// <https://html.spec.whatwg.org/multipage/#handler-messageport-onmessage>
    fn SetOnmessage(&self, listener: Option<Rc<EventHandlerNonNull>>) {
        if self.detached.get() {
            return;
        }
        self.set_onmessage(listener);
        self.global().start_message_port(self.message_port_id());
    }
}
