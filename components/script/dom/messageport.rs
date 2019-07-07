/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use crate::dom::bindings::codegen::Bindings::EventHandlerBinding::EventHandlerNonNull;
use crate::dom::bindings::codegen::Bindings::MessagePortBinding::{
    MessagePortMethods, PostMessageOptions, Wrap,
};
use crate::dom::bindings::conversions::{root_from_object, ToJSValConvertible};
use crate::dom::bindings::error::{Error, ErrorResult};
use crate::dom::bindings::inheritance::{Castable, HasParent};
use crate::dom::bindings::refcounted::Trusted;
use crate::dom::bindings::reflector::{reflect_dom_object, DomObject};
use crate::dom::bindings::root::DomRoot;
use crate::dom::bindings::structuredclone::{StructuredCloneData, StructuredCloneHolder};
use crate::dom::bindings::trace::{JSTraceable, RootedTraceableBox};
use crate::dom::bindings::transferable::Transferable;
use crate::dom::eventtarget::EventTarget;
use crate::dom::globalscope::GlobalScope;
use crate::dom::messageevent::MessageEvent;
use crate::task_source::TaskSource;
use ipc_channel::ipc::IpcSender;
use js::jsapi::Heap;
use js::jsapi::{JSContext, JSObject, JSStructuredCloneReader, JSTracer, MutableHandleObject};
use js::jsval::UndefinedValue;
use js::rust::{CustomAutoRooterGuard, HandleValue};
use msg::constellation_msg::{
    MessagePortId, MessagePortIndex, MessagePortMsg, PipelineNamespaceId, PortMessageTask,
};
use script_traits::ScriptMsg;
use std::cell::{Cell, RefCell};
use std::collections::VecDeque;
use std::num::NonZeroU32;
use std::os::raw;
use std::rc::Rc;

#[derive(DenyPublicFields, DomObject, MallocSizeOf)]
#[must_root]
#[repr(C)]
pub struct MessagePort {
    eventtarget: EventTarget,
    detached: Cell<bool>,
    transferred: Cell<bool>,
    enabled: Cell<bool>,
    awaiting_transfer: Cell<bool>,
    entangled_port: RefCell<Option<MessagePortId>>,
    #[ignore_malloc_size_of = "Channels are hard"]
    entangled_sender: RefCell<Option<IpcSender<MessagePortMsg>>>,
    #[ignore_malloc_size_of = "Task queues are hard"]
    message_buffer: RefCell<VecDeque<PortMessageTask>>,
    #[ignore_malloc_size_of = "Messages are hard"]
    outgoing_message_buffer: RefCell<VecDeque<PortMessageTask>>,
    has_been_shipped: Cell<bool>,
    #[ignore_malloc_size_of = "Defined in std"]
    message_port_id: MessagePortId,
}

#[allow(unsafe_code)]
unsafe impl JSTraceable for MessagePort {
    unsafe fn trace(&self, trc: *mut JSTracer) {
        if !self.detached.get() {
            self.eventtarget.trace(trc);
        }
    }
}

impl HasParent for MessagePort {
    type Parent = EventTarget;

    fn as_parent(&self) -> &EventTarget {
        &self.eventtarget
    }
}

impl MessagePort {
    fn new_inherited() -> MessagePort {
        MessagePort {
            eventtarget: EventTarget::new_inherited(),
            detached: Cell::new(false),
            transferred: Cell::new(false),
            enabled: Cell::new(false),
            awaiting_transfer: Cell::new(false),
            entangled_port: RefCell::new(None),
            entangled_sender: RefCell::new(None),
            message_buffer: RefCell::new(VecDeque::new()),
            outgoing_message_buffer: RefCell::new(VecDeque::new()),
            has_been_shipped: Cell::new(false),
            message_port_id: MessagePortId::new(),
        }
    }

    /// <https://html.spec.whatwg.org/multipage/#create-a-new-messageport-object>
    pub fn new(owner: &GlobalScope) -> DomRoot<MessagePort> {
        reflect_dom_object(Box::new(MessagePort::new_inherited()), owner, Wrap)
    }

    fn new_transferred(transferred_port: MessagePortId) -> MessagePort {
        MessagePort {
            eventtarget: EventTarget::new_inherited(),
            detached: Cell::new(false),
            transferred: Cell::new(false),
            enabled: Cell::new(false),
            awaiting_transfer: Cell::new(true),
            entangled_port: RefCell::new(None),
            entangled_sender: RefCell::new(None),
            message_buffer: RefCell::new(VecDeque::new()),
            outgoing_message_buffer: RefCell::new(VecDeque::new()),
            has_been_shipped: Cell::new(true),
            message_port_id: transferred_port,
        }
    }

    pub fn message_port_id(&self) -> &MessagePortId {
        &self.message_port_id
    }

    /// We received an ipc-sender to communicate with our entangled, and shipped, port.
    /// Drain the buffer of messages waiting to be sent, and use the ipc-sender going forward.
    pub fn set_entangled_sender(&self, sender: IpcSender<MessagePortMsg>) {
        if self.awaiting_transfer.get() || self.transferred.get() || self.detached.get() {
            // Note: we don't accept new senders while we are awaiting completion of our transfer,
            // because we don't know yet which port we're entangled with, if any,
            // and we'll get the new sender along with the entangled info when the transfer completes.
            return;
        }
        // Note: since this relates to a new sender for a port we're entangled with,
        // we expect to be entangled with a port.
        let target_port_id = match *self.entangled_port.borrow() {
            Some(port_id) => port_id.clone(),
            None => unreachable!(
                "A port should only receive an updated sender when it's already entangled"
            ),
        };
        for task in self.outgoing_message_buffer.borrow_mut().drain(0..) {
            let _ = sender.send(MessagePortMsg::NewTask(target_port_id, task));
        }
        *self.entangled_sender.borrow_mut() = Some(sender);
    }

    pub fn detached(&self) -> bool {
        self.detached.get()
    }

    /// <https://html.spec.whatwg.org/multipage/#entangle>
    pub fn entangle(&self, other: &MessagePort) {
        let other_id = other.message_port_id().clone();
        let self_id = self.message_port_id().clone();
        *self.entangled_port.borrow_mut() = Some(other_id);
        *other.entangled_port.borrow_mut() = Some(self_id);
        let _ = self
            .global()
            .script_to_constellation_chan()
            .send(ScriptMsg::EntanglePorts(other_id, self_id));
    }

    pub fn has_been_shipped(&self) -> bool {
        self.has_been_shipped.get()
    }

    pub fn set_has_been_shipped(&self) {
        self.has_been_shipped.set(true);
        // We also set our sender to None,
        // since it's outdated due to the port having been transferred.
        // We should get an updated sender later.
        *self.entangled_sender.borrow_mut() = None;
    }

    pub fn complete_transfer(
        &self,
        tasks: Option<VecDeque<PortMessageTask>>,
        outgoing_msgs: Option<VecDeque<PortMessageTask>>,
        entangled_with: Option<MessagePortId>,
        entangled_sender: Option<IpcSender<MessagePortMsg>>,
    ) {
        if self.detached.get() || self.transferred.get() {
            return;
        }
        self.awaiting_transfer.set(false);

        *self.entangled_port.borrow_mut() = entangled_with;

        if let Some(mut tasks) = outgoing_msgs {
            let mut outgoing_buffer = self.outgoing_message_buffer.borrow_mut();
            while let Some(task) = tasks.pop_back() {
                outgoing_buffer.push_front(task);
            }
        }

        if let Some(sender) = entangled_sender {
            self.set_entangled_sender(sender);
        }

        if let Some(mut tasks) = tasks {
            // Note: these are the tasks that were buffered prior to transfer,
            // hence they need to execute first.
            let mut incoming_buffer = self.message_buffer.borrow_mut();
            while let Some(task) = tasks.pop_back() {
                incoming_buffer.push_front(task);
            }
        }

        if self.enabled.get() {
            // Execute tasks that might have been buffered after the transfer.
            self.Start();
        }
    }

    /// A message was received from our entangled port over ipc.
    /// This method should always run as part of a Task on the relevant event-loop.
    /// We buffer incoming messages if we haven't been enabled yet.
    pub fn handle_incoming(&self, task: PortMessageTask) {
        if self.detached.get() || self.transferred.get() {
            return;
        }

        if self.enabled.get() {
            let PortMessageTask { origin, data } = task;

            // Substep 2
            let target_global = self.global();

            // Substep 3-4
            rooted!(in(target_global.get_cx()) let mut message_clone = UndefinedValue());
            if let Ok(deserialize_result) =
                StructuredCloneData::Vector(data).read(&target_global, message_clone.handle_mut())
            {
                // Substep 6
                MessageEvent::dispatch_jsval(
                    self.upcast(),
                    &target_global,
                    message_clone.handle(),
                    Some(&origin),
                    None,
                    deserialize_result.message_ports.into_iter().collect(),
                );
            }
        } else {
            self.message_buffer.borrow_mut().push_back(task);
        }
    }

    /// <https://html.spec.whatwg.org/multipage/#dom-messageport-postmessage>
    // Step 7 substeps
    pub fn post_message(&self, task: PortMessageTask) {
        if self.awaiting_transfer.get() {
            // If this port has been transfered and is waiting on the transfer to complete,
            // we will not have up to date data on
            // which port we are entangled with, if any.
            // Buffer outgoing tasks while we wait for this data to come in.
            self.outgoing_message_buffer.borrow_mut().push_back(task);
            return;
        }

        let target_port_id = match *self.entangled_port.borrow() {
            Some(port_id) => port_id.clone(),
            None => return,
        };
        if self.has_been_shipped.get() {
            if let Some(sender) = &*self.entangled_sender.borrow() {
                let _ = sender.send(MessagePortMsg::NewTask(target_port_id, task));
            } else {
                // Note: this is the mirror case of when we're awaiting transfer.
                //
                // In case the entangled port has been shipped, but we haven't received the new sender yet.
                // This could happen if a port is shipped in a task because it's entangled port is transferred,
                // and the same task immediately starts sending messages meant for the transferred port.
                self.outgoing_message_buffer.borrow_mut().push_back(task);
            }
        } else {
            let this = Trusted::new(&*self.global());
            let _ = self.global().port_message_queue().queue(
                task!(process_pending_port_messages: move || {
                    let global = this.root();
                    global.upcast::<GlobalScope>().route_task_to_port(target_port_id, task);
                }),
                &self.global(),
            );
        }
    }

    #[allow(unsafe_code)]
    fn post_message_impl(
        &self,
        cx: *mut JSContext,
        message: HandleValue,
        transfer: Vec<*mut JSObject>,
    ) -> ErrorResult {
        if self.detached.get() || self.transferred.get() {
            return Ok(());
        }
        // Step 1
        let target_port = self.entangled_port.borrow();

        // Step 3
        let mut doomed = false;

        rooted!(in(cx) let mut val = UndefinedValue());
        let ports = transfer
            .iter()
            .filter_map(|&obj| root_from_object::<MessagePort>(obj, cx).ok());
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

        unsafe { transfer.to_jsval(cx, val.handle_mut()) };

        // Step 5
        let data = StructuredCloneData::write(cx, message, val.handle())?.move_to_arraybuffer();

        if doomed {
            return Ok(());
        }

        // Step 6
        if target_port.is_none() {
            // TODO: find a way to deal with target_port being none
            // because we haven't complete the transfer.
            // One way would be to transfer the entangled port ID along with our own ID.
        }

        // Step 7
        let task = PortMessageTask {
            origin: self.global().origin().immutable().ascii_serialization(),
            data,
        };

        self.post_message(task);
        Ok(())
    }
}

impl Transferable for MessagePort {
    /// <https://html.spec.whatwg.org/multipage/#message-ports:transfer-steps>
    #[allow(unsafe_code)]
    fn transfer(
        &self,
        _closure: *mut raw::c_void,
        _content: *mut *mut raw::c_void,
        extra_data: *mut u64,
    ) -> bool {
        // Step 1
        self.has_been_shipped.set(true);

        // Step 3
        // Substep 1

        // Used to GC the port.
        self.transferred.set(true);

        self.global().mark_port_as_shipped(
            self.message_port_id().clone(),
            self.entangled_port.borrow().clone(),
        );
        let _ = self
            .global()
            .script_to_constellation_chan()
            .send(ScriptMsg::MessagePortShipped(
                self.message_port_id().clone(),
                self.entangled_port.borrow().clone(),
                self.message_buffer.borrow().clone(),
                self.outgoing_message_buffer.borrow().clone(),
            ));

        unsafe {
            // TODO: also transfer the Id of the entangled port, using content?

            // Steps 2, 3.2 and 4

            let PipelineNamespaceId(name_space) = self.message_port_id().clone().namespace_id;
            let MessagePortIndex(index) = self.message_port_id().clone().index;
            let index = index.get();

            let mut big: [u8; 8] = [0, 0, 0, 0, 0, 0, 0, 0];
            let name_space = name_space.to_ne_bytes();
            let index = index.to_ne_bytes();

            big[0] = name_space[0];
            big[1] = name_space[1];
            big[2] = name_space[2];
            big[3] = name_space[3];
            big[4] = index[0];
            big[5] = index[1];
            big[6] = index[2];
            big[7] = index[3];

            *extra_data = u64::from_ne_bytes(big);
        }

        // Disable the port.
        *self.entangled_sender.borrow_mut() = None;
        *self.entangled_port.borrow_mut() = None;

        true
    }

    /// https://html.spec.whatwg.org/multipage/#message-ports:transfer-receiving-steps
    #[allow(unrooted_must_root, unsafe_code)]
    fn transfer_receive(
        cx: *mut JSContext,
        _r: *mut JSStructuredCloneReader,
        closure: *mut raw::c_void,
        _content: *mut raw::c_void,
        extra_data: u64,
        return_object: MutableHandleObject,
    ) -> bool {
        let sc_holder = unsafe { &mut *(closure as *mut StructuredCloneHolder) };
        // Step 2
        let owner = unsafe { GlobalScope::from_context(cx) };

        let big: [u8; 8] = extra_data.to_ne_bytes();
        let mut name_space: [u8; 4] = [0, 0, 0, 0];
        let mut index: [u8; 4] = [0, 0, 0, 0];
        name_space[0] = big[0];
        name_space[1] = big[1];
        name_space[2] = big[2];
        name_space[3] = big[3];
        index[0] = big[4];
        index[1] = big[5];
        index[2] = big[6];
        index[3] = big[7];

        let namespace_id = PipelineNamespaceId(u32::from_ne_bytes(name_space));
        let index =
            unsafe { MessagePortIndex(NonZeroU32::new_unchecked(u32::from_ne_bytes(index))) };

        let id = MessagePortId {
            namespace_id,
            index,
        };

        let transferred_port = MessagePort::new_transferred(id.clone());
        let value = reflect_dom_object(Box::new(transferred_port), &*owner, Wrap);
        owner.track_message_port(&value, true);

        return_object.set(value.reflector().rootable().get());

        sc_holder.message_ports.push_back(value);

        true
    }

    fn transferred(&self) -> bool {
        self.transferred.get()
    }
}

impl MessagePortMethods for MessagePort {
    #[allow(unsafe_code)]
    /// <https://html.spec.whatwg.org/multipage/#dom-messageport-postmessage>
    unsafe fn PostMessage(
        &self,
        cx: *mut JSContext,
        message: HandleValue,
        transfer: CustomAutoRooterGuard<Vec<*mut JSObject>>,
    ) -> ErrorResult {
        self.post_message_impl(cx, message, transfer.to_vec())
    }

    #[allow(unsafe_code)]
    /// <https://html.spec.whatwg.org/multipage/#dom-messageport-postmessage>
    unsafe fn PostMessage_(
        &self,
        cx: *mut JSContext,
        message: HandleValue,
        options: RootedTraceableBox<PostMessageOptions>,
    ) -> ErrorResult {
        //let transfer:
        let transfer: Vec<*mut JSObject> = options
            .transfer
            .iter()
            .map(|js: &RootedTraceableBox<Heap<*mut JSObject>>| js.get())
            .collect();
        self.post_message_impl(cx, message, transfer)
    }

    /// <https://html.spec.whatwg.org/multipage/#dom-messageport-start>
    fn Start(&self) {
        self.enabled.set(true);
        if self.awaiting_transfer.get() || self.transferred.get() {
            return;
        }
        let port_id = self.message_port_id().clone();
        for task in self.message_buffer.borrow_mut().drain(0..) {
            let this = Trusted::new(&*self.global());
            let _ = self.global().port_message_queue().queue(
                task!(process_pending_port_messages: move || {
                    let target_global = this.root();
                    target_global.route_task_to_port(port_id, task);
                }),
                &self.global(),
            );
        }
    }

    /// <https://html.spec.whatwg.org/multipage/#dom-messageport-close>
    fn Close(&self) {
        // Step 1
        self.detached.set(true);
        // Disable the port.
        *self.entangled_sender.borrow_mut() = None;
        *self.entangled_port.borrow_mut() = None;
    }

    /// <https://html.spec.whatwg.org/multipage/#handler-messageport-onmessage>
    fn GetOnmessage(&self) -> Option<Rc<EventHandlerNonNull>> {
        let eventtarget = self.upcast::<EventTarget>();
        eventtarget.get_event_handler_common("message")
    }

    /// <https://html.spec.whatwg.org/multipage/#handler-messageport-onmessage>
    fn SetOnmessage(&self, listener: Option<Rc<EventHandlerNonNull>>) {
        self.Start();
        let eventtarget = self.upcast::<EventTarget>();
        eventtarget.set_event_handler_common("message", listener)
    }
}
