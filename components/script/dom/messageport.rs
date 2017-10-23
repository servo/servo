/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use dom::bindings::codegen::Bindings::EventHandlerBinding::EventHandlerNonNull;
use dom::bindings::codegen::Bindings::MessagePortBinding::{MessagePortMethods, Wrap};
use dom::bindings::error::ErrorResult;
use dom::bindings::inheritance::Castable;
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
use std::cell::{Cell, RefCell};
use std::collections::VecDeque;
use std::os::raw;
use std::rc::Rc;
use std::sync::{Arc, Mutex};
use task_source::TaskSource;

// FIXME: This is wrong, we need to figure out a better way of collecting message port objects per transfer
thread_local! {
    pub static TRANSFERRED_MESSAGE_PORTS: RefCell<Vec<DomRoot<MessagePort>>> = RefCell::new(Vec::new())
}

struct PortMessageTask {
    data: Vec<u8>,
}

pub struct MessagePortInternal {
    dom_port: Option<Trusted<MessagePort>>,
    enabled: bool,
    has_been_shipped: bool,
    entangled_port: Option<Arc<Mutex<MessagePortInternal>>>,
    pending_port_messages: VecDeque<PortMessageTask>,
}

impl MessagePortInternal {
    fn new() -> MessagePortInternal {
        MessagePortInternal {
            dom_port: None,
            enabled: false,
            has_been_shipped: false,
            entangled_port: None,
            pending_port_messages: VecDeque::new(),
        }
    }

    /// <https://html.spec.whatwg.org/multipage/#dom-messageport-postmessage>
    // Step 7 substeps
    fn process_pending_port_messages(&mut self) {
        if let Some(task) = self.pending_port_messages.pop_front() {
            // Substep 1
            let final_target_port = self.dom_port.as_ref().unwrap().root();

            // Substep 2
            let target_global = final_target_port.global();

            // Substep 3-4
            rooted!(in(target_global.get_cx()) let mut message_clone = UndefinedValue());
            let deserialize_result = StructuredCloneData::Vector(task.data).read(
                &target_global,
                message_clone.handle_mut()
            );
            if !deserialize_result {
                return;
            }

            // Substep 5

            // Substep 6
            MessageEvent::dispatch_jsval(
                final_target_port.upcast(),
                &target_global,
                message_clone.handle()
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
    message_port_internal: Arc<Mutex<MessagePortInternal>>,
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

impl MessagePort {
    fn new_inherited() -> MessagePort {
        MessagePort {
            eventtarget: EventTarget::new_inherited(),
            detached: Cell::new(false),
            message_port_internal: Arc::new(Mutex::new(MessagePortInternal::new())),
        }
    }

    fn new_transferred(message_port_internal: Arc<Mutex<MessagePortInternal>>) -> MessagePort {
        MessagePort {
            eventtarget: EventTarget::new_inherited(),
            detached: Cell::new(false),
            message_port_internal,
        }
    }

    /// <https://html.spec.whatwg.org/multipage/#create-a-new-messageport-object>
    pub fn new(owner: &GlobalScope) -> DomRoot<MessagePort> {
        let message_port = reflect_dom_object(Box::new(MessagePort::new_inherited()), owner, Wrap);
        {
            let mut internal = message_port.message_port_internal.lock().unwrap();
            internal.dom_port = Some(Trusted::new(&*message_port));
        }
        message_port
    }

    /// <https://html.spec.whatwg.org/multipage/#entangle>
    pub fn entangle(&self, other: &MessagePort) {
        {
            let mut internal = self.message_port_internal.lock().unwrap();
            internal.entangled_port = Some(other.message_port_internal.clone());
        }
        let mut internal = other.message_port_internal.lock().unwrap();
        internal.entangled_port = Some(self.message_port_internal.clone());
    }

    /// <https://html.spec.whatwg.org/multipage/#dom-messageport-postmessage>
    // Step 7 substeps
    fn process_pending_port_messages(&self) {
        if self.detached.get() { return; }
        let mut internal = self.message_port_internal.lock().unwrap();
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
            let mut internal = self.message_port_internal.lock().unwrap();
            // Step 1
            internal.has_been_shipped = true;

            // Step 3
            if let Some(ref other_port) = internal.entangled_port {
                let mut entangled_internal = other_port.lock().unwrap();
                // Substep 1
                entangled_internal.has_been_shipped = true;
            }
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
        let internal = unsafe { Arc::from_raw(content as *const Mutex<MessagePortInternal>) };
        let value = MessagePort::new_transferred(internal);

        // Step 2
        let owner = unsafe { GlobalScope::from_context(cx) };
        let message_port = reflect_dom_object(Box::new(value), &*owner, Wrap);

        {
            let mut internal = message_port.message_port_internal.lock().unwrap();

            // Step 1
            internal.has_been_shipped = true;

            let dom_port = Trusted::new(&*message_port);
            internal.enabled = false;
            internal.dom_port = Some(dom_port);
        }
        return_object.set(message_port.reflector().rootable().get());
        TRANSFERRED_MESSAGE_PORTS.with(|list| {
            list.borrow_mut().push(Dom::from_ref(&*message_port));
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
        Ok(())
    }

    /// <https://html.spec.whatwg.org/multipage/#dom-messageport-start>
    fn Start(&self) {
        let len = {
            let mut internal = self.message_port_internal.lock().unwrap();
            if internal.enabled {
                return;
            }
            internal.enabled = true;
            internal.pending_port_messages.len()
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
            let mut internal = self.message_port_internal.lock().unwrap();
            internal.entangled_port.take()
        };

        if let Some(other) = maybe_port {
            let mut other_internal = other.lock().unwrap();
            other_internal.entangled_port = None;
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
