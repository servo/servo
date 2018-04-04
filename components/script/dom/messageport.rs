/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use dom::bindings::codegen::Bindings::EventHandlerBinding::EventHandlerNonNull;
use dom::bindings::codegen::Bindings::MessagePortBinding::{MessagePortMethods, Wrap};
use dom::bindings::conversions::{ToJSValConvertible, root_from_object};
use dom::bindings::error::{Error, ErrorResult};
use dom::bindings::inheritance::{Castable, HasParent};
use dom::bindings::refcounted::Trusted;
use dom::bindings::reflector::{DomObject, reflect_dom_object};
use dom::bindings::root::DomRoot;
use dom::bindings::structuredclone::StructuredCloneData;
use dom::bindings::trace::JSTraceable;
use dom::bindings::transferable::Transferable;
use dom::eventtarget::EventTarget;
use dom::globalscope::GlobalScope;
use dom::messageevent::MessageEvent;
use js::jsapi::{JSContext, JSStructuredCloneReader, JSObject, JSTracer, MutableHandleObject};
use js::jsval::UndefinedValue;
use js::rust::{CustomAutoRooterGuard, HandleValue};
use servo_remutex::ReentrantMutex;
use std::cell::{Cell, RefCell};
use std::collections::VecDeque;
use std::mem;
use std::os::raw;
use std::rc::Rc;
use std::sync::Arc;
use task_source::TaskSource;
use task_source::port_message::PortMessageQueue;

// FIXME: This is wrong, we need to figure out a better way of collecting message port objects per transfer
thread_local! {
    pub static TRANSFERRED_MESSAGE_PORTS: RefCell<Vec<DomRoot<MessagePort>>> = RefCell::new(Vec::new())
}

struct PortMessageTask {
    origin: String,
    data: Vec<u8>,
}

pub struct MessagePortInternal {
    dom_port: RefCell<Option<Trusted<MessagePort>>>,
    port_message_queue: RefCell<PortMessageQueue>,
    enabled: Cell<bool>,
    has_been_shipped: Cell<bool>,
    entangled_port: RefCell<Option<Arc<ReentrantMutex<MessagePortInternal>>>>,
    pending_port_messages: RefCell<VecDeque<PortMessageTask>>,
}

impl MessagePortInternal {
    fn new(port_message_queue: PortMessageQueue) -> MessagePortInternal {
        MessagePortInternal {
            dom_port: RefCell::new(None),
            port_message_queue: RefCell::new(port_message_queue),
            enabled: Cell::new(false),
            has_been_shipped: Cell::new(false),
            entangled_port: RefCell::new(None),
            pending_port_messages: RefCell::new(VecDeque::new()),
        }
    }

    /// <https://html.spec.whatwg.org/multipage/#dom-messageport-postmessage>
    // Step 7 substeps
    #[allow(unrooted_must_root)]
    fn process_pending_port_messages(&self) {
        if let Some(PortMessageTask { origin, data }) = self.pending_port_messages.borrow_mut().pop_front() {
            // Substep 1
            let final_target_port = self.dom_port.borrow().as_ref().unwrap().root();

            // Substep 2
            let target_global = final_target_port.global();

            // Substep 3-4
            rooted!(in(target_global.get_cx()) let mut message_clone = UndefinedValue());
            let deserialize_result = StructuredCloneData::Vector(data).read(
                &target_global,
                message_clone.handle_mut(),
            );
            if !deserialize_result {
                return;
            }

            // Substep 5
            let new_ports = TRANSFERRED_MESSAGE_PORTS.with(|list| {
                mem::replace(&mut *list.borrow_mut(), vec![])
            });

            // Substep 6
            MessageEvent::dispatch_jsval(
                final_target_port.upcast(),
                &target_global,
                message_clone.handle(),
                Some(&origin),
                new_ports,
            );
        }
    }
}

#[derive(DenyPublicFields, DomObject, MallocSizeOf)]
#[must_root]
#[repr(C)]
pub struct MessagePort {
    eventtarget: EventTarget,
    detached: Cell<bool>,
    #[ignore_malloc_size_of = "Defined in std"]
    message_port_internal: Arc<ReentrantMutex<MessagePortInternal>>,
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
    fn new_inherited(global: &GlobalScope) -> MessagePort {
        MessagePort {
            eventtarget: EventTarget::new_inherited(),
            detached: Cell::new(false),
            message_port_internal: Arc::new(
                ReentrantMutex::new(
                    MessagePortInternal::new(global.port_message_queue().clone())
                )
            ),
        }
    }

    fn new_transferred(message_port_internal: Arc<ReentrantMutex<MessagePortInternal>>) -> MessagePort {
        MessagePort {
            eventtarget: EventTarget::new_inherited(),
            detached: Cell::new(false),
            message_port_internal,
        }
    }

    /// <https://html.spec.whatwg.org/multipage/#create-a-new-messageport-object>
    pub fn new(owner: &GlobalScope) -> DomRoot<MessagePort> {
        let message_port = reflect_dom_object(Box::new(MessagePort::new_inherited(owner)), owner, Wrap);
        {
            let internal = message_port.message_port_internal.lock().unwrap();
            *internal.dom_port.borrow_mut() = Some(Trusted::new(&*message_port));
        }
        message_port
    }

    /// <https://html.spec.whatwg.org/multipage/#entangle>
    pub fn entangle(&self, other: &MessagePort) {
        {
            let internal = self.message_port_internal.lock().unwrap();
            *internal.entangled_port.borrow_mut() = Some(other.message_port_internal.clone());
        }
        let internal = other.message_port_internal.lock().unwrap();
        *internal.entangled_port.borrow_mut() = Some(self.message_port_internal.clone());
    }

    /// <https://html.spec.whatwg.org/multipage/#dom-messageport-postmessage>
    // Step 7 substeps
    fn process_pending_port_messages(&self) {
        if self.detached.get() { return; }
        let internal = self.message_port_internal.lock().unwrap();
        internal.process_pending_port_messages();
    }
}

impl Transferable for MessagePort {
    /// <https://html.spec.whatwg.org/multipage/#message-ports:transfer-steps>
    #[allow(unsafe_code)]
    fn transfer(
        &self,
        _closure: *mut raw::c_void,
        content: *mut *mut raw::c_void,
        extra_data: *mut u64
    ) -> bool {
        {
            let internal = self.message_port_internal.lock().unwrap();
            // Step 1
            internal.has_been_shipped.set(true);

            // Step 3
            if let Some(ref other_port) = *internal.entangled_port.borrow() {
                let entangled_internal = other_port.lock().unwrap();
                // Substep 1
                entangled_internal.has_been_shipped.set(true);
            }; // This line MUST contain a semicolon, due to the strict drop check rule
        }

        unsafe {
            // Steps 2, 3.2 and 4
            *content = Arc::into_raw(self.message_port_internal.clone()) as *mut raw::c_void;

            *extra_data = 0;
        }

        true
    }

    /// https://html.spec.whatwg.org/multipage/#message-ports:transfer-receiving-steps
    #[allow(unrooted_must_root, unsafe_code)]
    fn transfer_receive(
        cx: *mut JSContext,
        _r: *mut JSStructuredCloneReader,
        _closure: *mut raw::c_void,
        content: *mut raw::c_void,
        _extra_data: u64,
        return_object: MutableHandleObject
    ) -> bool {
        let internal = unsafe { Arc::from_raw(content as *const ReentrantMutex<MessagePortInternal>) };
        let value = MessagePort::new_transferred(internal);

        // Step 2
        let owner = unsafe { GlobalScope::from_context(cx) };
        let message_port = reflect_dom_object(Box::new(value), &*owner, Wrap);

        {
            let internal = message_port.message_port_internal.lock().unwrap();

            // Step 1
            internal.has_been_shipped.set(true);

            let dom_port = Trusted::new(&*message_port);
            internal.enabled.set(false);
            *internal.dom_port.borrow_mut() = Some(dom_port);
            *internal.port_message_queue.borrow_mut() = owner.port_message_queue().clone();
        }
        return_object.set(message_port.reflector().rootable().get());
        TRANSFERRED_MESSAGE_PORTS.with(|list| {
            list.borrow_mut().push(message_port);
        });

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
        if self.detached.get() { return Ok(()); }
        let internal = self.message_port_internal.lock().unwrap();
        // Step 1
        let target_port = internal.entangled_port.borrow();

        // Step 3
        let mut doomed = false;

        rooted!(in(cx) let mut val = UndefinedValue());
        let transfer = match *transfer {
            Some(ref vec) => {
                let ports = vec.iter().filter_map(|&obj| root_from_object::<MessagePort>(obj).ok());
                for port in ports {
                    // Step 2
                    if Arc::ptr_eq(&port.message_port_internal, &self.message_port_internal) {
                        return Err(Error::DataClone);
                    }

                    // Step 4
                    if let Some(target) = target_port.as_ref() {
                        if Arc::ptr_eq(&port.message_port_internal, target) {
                            doomed = true;
                        }
                    }
                }

                vec.to_jsval(cx, val.handle_mut());
                val
            }
            None => {
                Vec::<*mut JSObject>::new().to_jsval(cx, val.handle_mut());
                val
            }
        };

        // Step 5
       let data = StructuredCloneData::write(cx, message, transfer.handle())?.move_to_arraybuffer();

        // Step 6
        if target_port.is_none() || doomed { return Ok(()); }

        // Step 7
        let task = PortMessageTask {
            origin: self.global().origin().immutable().ascii_serialization(),
            data,
        };

        {
            let target_port = target_port.as_ref().unwrap();
            let target_internal = target_port.lock().unwrap();
            target_internal.pending_port_messages.borrow_mut().push_back(task);

            if target_internal.enabled.get() {
                let target_port = target_port.clone();
                let _ = target_internal.port_message_queue.borrow().queue(
                    task!(process_pending_port_messages: move || {
                        let internal = target_port.lock().unwrap();
                        internal.process_pending_port_messages();
                    }),
                    &self.global()
                );
            }
        }

        Ok(())
    }

    /// <https://html.spec.whatwg.org/multipage/#dom-messageport-start>
    fn Start(&self) {
        let len = {
            let internal = self.message_port_internal.lock().unwrap();
            if internal.enabled.get() {
                return;
            }
            internal.enabled.set(true);
            let messages = internal.pending_port_messages.borrow();
            messages.len()
        };

        let global = self.global();
        for _ in 0..len {
            let port = Trusted::new(self);
            let _ = global.port_message_queue().queue(
                task!(process_pending_port_messages: move || {
                    let this = port.root();
                    this.process_pending_port_messages();
                }),
                &global
            );
        }
    }

    /// <https://html.spec.whatwg.org/multipage/#dom-messageport-close>
    fn Close(&self) {
        let maybe_port = {
            let internal = self.message_port_internal.lock().unwrap();
            let mut maybe_port = internal.entangled_port.borrow_mut();
            maybe_port.take()
        };

        if let Some(other) = maybe_port {
            let other_internal = other.lock().unwrap();
            *other_internal.entangled_port.borrow_mut() = None;
        }
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
