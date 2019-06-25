/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use crate::dom::bindings::codegen::Bindings::EventHandlerBinding::EventHandlerNonNull;
use crate::dom::bindings::codegen::Bindings::MessagePortBinding::{MessagePortMethods, Wrap};
use crate::dom::bindings::conversions::{root_from_object, ToJSValConvertible};
use crate::dom::bindings::error::{Error, ErrorResult};
use crate::dom::bindings::inheritance::{Castable, HasParent};
use crate::dom::bindings::reflector::{reflect_dom_object, DomObject};
use crate::dom::bindings::root::DomRoot;
use crate::dom::bindings::structuredclone::{StructuredCloneData, StructuredCloneHolder};
use crate::dom::bindings::trace::JSTraceable;
use crate::dom::bindings::transferable::Transferable;
use crate::dom::eventtarget::EventTarget;
use crate::dom::globalscope::GlobalScope;
use crate::dom::messageevent::MessageEvent;
use crate::script_thread::ScriptThread;
use crate::task_source::TaskSource;
use js::jsapi::{JSContext, JSObject, JSStructuredCloneReader, JSTracer, MutableHandleObject};
use js::jsval::UndefinedValue;
use js::rust::{CustomAutoRooterGuard, HandleValue};
use msg::constellation_msg::{MessagePortId, PortMessageTask};
use script_traits::ScriptMsg;
use std::cell::{Cell, RefCell};
use std::collections::VecDeque;
use std::os::raw;

#[derive(DenyPublicFields, DomObject, MallocSizeOf)]
#[must_root]
#[repr(C)]
pub struct MessagePort {
    eventtarget: EventTarget,
    detached: Cell<bool>,
    enabled: Cell<bool>,
    entangled_port: RefCell<Option<MessagePortId>>,
    #[ignore_malloc_size_of = "Task queues are hard"]
    message_buffer: RefCell<VecDeque<PortMessageTask>>,
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
        // Otherwise, do nothing.
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
            enabled: Cell::new(false),
            entangled_port: RefCell::new(None),
            message_buffer: RefCell::new(VecDeque::new()),
            has_been_shipped: Cell::new(false),
            message_port_id: MessagePortId::new(),
        }
    }

    /// <https://html.spec.whatwg.org/multipage/#create-a-new-messageport-object>
    pub fn new(owner: &GlobalScope) -> DomRoot<MessagePort> {
        reflect_dom_object(Box::new(MessagePort::new_inherited(owner)), owner, Wrap)
    }

    fn new_transferred(
        transferred_port: MessagePortId,
        entangled_port: Option<MessagePortId>,
        message_buffer: VecDeque<PortMessageTask>,
    ) -> MessagePort {
        MessagePort {
            eventtarget: EventTarget::new_inherited(),
            detached: Cell::new(false),
            enabled: Cell::new(false),
            entangled_port: RefCell::new(entangled_port),
            message_buffer: RefCell::new(message_buffer),
            has_been_shipped: Cell::new(true),
            message_port_id: transferred_port,
        }
    }

    pub fn message_port_id(&self) -> &MessagePortId {
        &self.message_port_id
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

    pub fn handle_incoming(&self, task: PortMessageTask) {
        if self.detached.get() {
            return;
        }

        if self.enabled.get() {
            let PortMessageTask { origin, data } = task;

            // Substep 2
            let target_global = self.global();

            // Substep 3-4
            rooted!(in(target_global.get_cx()) let mut message_clone = UndefinedValue());
            if let Ok(mut deserialize_result) =
                StructuredCloneData::Vector(data).read(&target_global, message_clone.handle_mut())
            {
                // Substep 6
                MessageEvent::dispatch_jsval(
                    self.upcast(),
                    &target_global,
                    message_clone.handle(),
                    Some(&origin),
                    None,
                    deserialize_result.message_ports.drain(0..).collect(),
                );
            }
        } else {
            self.message_buffer.borrow_mut().push_back(task);
        }
    }

    /// <https://html.spec.whatwg.org/multipage/#dom-messageport-postmessage>
    // Step 7 substeps
    fn post_message(&self, task: PortMessageTask) {
        let target_port_id = match *self.entangled_port.borrow() {
            Some(port_id) => port_id.clone(),
            None => return,
        };
        if self.has_been_shipped.get() {
            let _ = self
                .global()
                .script_to_constellation_chan()
                .send(ScriptMsg::PortMessage(target_port_id, task));
        } else {
            ScriptThread::with_message_port(
                &target_port_id,
                Box::new(move |target_port: &mut DomRoot<MessagePort>| {
                    let _ = target_port.global().port_message_queue().queue(
                        task!(process_pending_port_messages: move || {
                            ScriptThread::with_message_port(&target_port_id,
                                Box::new(move |target_port: &mut DomRoot<MessagePort>|
                                    target_port.handle_incoming(task)
                                )
                            );
                        }),
                        &target_port.global(),
                    );
                }),
            );
        }
    }
}

impl Transferable for MessagePort {
    /// <https://html.spec.whatwg.org/multipage/#message-ports:transfer-steps>
    #[allow(unsafe_code)]
    fn transfer(
        &self,
        _closure: *mut raw::c_void,
        content: *mut *mut raw::c_void,
        extra_data: *mut u64,
    ) -> bool {
        {
            // Step 1
            self.has_been_shipped.set(true);

            // Step 3
            if let Some(other_port) = *self.entangled_port.borrow() {
                // Substep 1
                ScriptThread::with_message_port(
                    &other_port,
                    Box::new(|target_port: &mut DomRoot<MessagePort>| {
                        target_port.has_been_shipped.set(true)
                    }),
                );
            }; // This line MUST contain a semicolon, due to the strict drop check rule
        }

        unsafe {
            // Steps 2, 3.2 and 4
            let message_port_ptr = &mut (
                self.message_port_id().clone(),
                self.entangled_port.borrow().clone(),
                self.message_buffer.borrow().clone(),
            ) as *mut _;
            *content = message_port_ptr as *mut raw::c_void;

            *extra_data = 0;
        }

        ScriptThread::message_port_transfered(self.message_port_id());

        true
    }

    /// https://html.spec.whatwg.org/multipage/#message-ports:transfer-receiving-steps
    #[allow(unrooted_must_root, unsafe_code)]
    fn transfer_receive(
        cx: *mut JSContext,
        _r: *mut JSStructuredCloneReader,
        closure: *mut raw::c_void,
        content: *mut raw::c_void,
        _extra_data: u64,
        return_object: MutableHandleObject,
    ) -> bool {
        let sc_holder = unsafe { &mut *(closure as *mut StructuredCloneHolder) };
        // Step 2
        let owner = unsafe { GlobalScope::from_context(cx) };
        let (id, entangled, messages) = unsafe {
            &mut *(content
                as *mut (
                    MessagePortId,
                    Option<MessagePortId>,
                    VecDeque<PortMessageTask>,
                ))
        };
        let transferred_port =
            MessagePort::new_transferred(id.clone(), entangled.clone(), messages.clone());
        let value = reflect_dom_object(Box::new(transferred_port), &*owner, Wrap);

        let _ = owner
            .script_to_constellation_chan()
            .send(ScriptMsg::MessagePortTransfered(
                value.message_port_id().clone(),
            ));

        return_object.set(value.reflector().rootable().get());

        sc_holder.message_ports.push_back(value);

        true
    }

    fn detached(&self) -> Option<bool> {
        Some(self.detached.get())
    }

    fn set_detached(&self, value: bool) {
        self.detached.set(value);
    }

    fn transferable(&self) -> bool {
        !self.detached.get()
    }
}

impl MessagePortMethods for MessagePort {
    #[allow(unsafe_code)]
    /// <https://html.spec.whatwg.org/multipage/#dom-messageport-postmessage>
    unsafe fn PostMessage(
        &self,
        cx: *mut JSContext,
        message: HandleValue,
        transfer: CustomAutoRooterGuard<Option<Vec<*mut JSObject>>>,
    ) -> ErrorResult {
        if self.detached.get() {
            return Ok(());
        }
        // Step 1
        let target_port = self.entangled_port.borrow();

        // Step 3
        let mut doomed = false;

        rooted!(in(cx) let mut val = UndefinedValue());
        let transfer = match *transfer {
            Some(ref vec) => {
                let ports = vec
                    .iter()
                    .filter_map(|&obj| root_from_object::<MessagePort>(obj).ok());
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

                vec.to_jsval(cx, val.handle_mut());
                val
            },
            None => {
                Vec::<*mut JSObject>::new().to_jsval(cx, val.handle_mut());
                val
            },
        };

        // Step 5
        let data =
            StructuredCloneData::write(cx, message, transfer.handle())?.move_to_arraybuffer();

        // Step 6
        if target_port.is_none() || doomed {
            return Ok(());
        }

        // Step 7
        let task = PortMessageTask {
            origin: self.global().origin().immutable().ascii_serialization(),
            data,
        };

        self.post_message(task);

        Ok(())
    }

    /// <https://html.spec.whatwg.org/multipage/#dom-messageport-start>
    fn Start(&self) {
        self.enabled.set(true);
        let port_id = self.message_port_id().clone();
        for task in self.message_buffer.borrow_mut().drain(0..) {
            let _ = self.global().port_message_queue().queue(
                task!(process_pending_port_messages: move || {

                    let PortMessageTask { origin, data } = task;

                    ScriptThread::with_message_port(&port_id,
                        Box::new(move |target_port: &mut DomRoot<MessagePort>| {
                            // Substep 2
                            let target_global = target_port.global();

                            // Substep 3-4
                            rooted!(in(target_global.get_cx()) let mut message_clone = UndefinedValue());

                            if let Ok(mut deserialize_result) =
                                StructuredCloneData::Vector(data).read(&target_global, message_clone.handle_mut()) {
                                // Substep 6
                                MessageEvent::dispatch_jsval(
                                    target_port.upcast(),
                                    &target_global,
                                    message_clone.handle(),
                                    Some(&origin),
                                    None,
                                    deserialize_result.message_ports.drain(0..).collect(),
                                );
                            }
                        })
                    );
                }),
                &self.global(),
            );
        }
    }

    /// <https://html.spec.whatwg.org/multipage/#dom-messageport-close>
    fn Close(&self) {
        // Step 1
        self.detached.set(true);
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
