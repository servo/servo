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
use dom::eventtarget::EventTarget;
use dom::globalscope::GlobalScope;
use dom::messageevent::MessageEvent;
use dom_struct::dom_struct;
use js::jsapi::{HandleValue, JSContext};
use js::jsval::UndefinedValue;
use std::cell::Cell;
use std::collections::VecDeque;
use std::rc::Rc;
use std::sync::{Arc, Mutex};
use task_source::TaskSource;

#[derive(MallocSizeOf)]
struct PortMessageTask {
    data: Vec<u8>,
}

#[derive(JSTraceable, MallocSizeOf)]
pub struct MessagePortInternal {
    #[ignore_malloc_size_of = "Task sources are hard"]
    dom_port: Option<Trusted<MessagePort>>,
    enabled: bool,
    has_been_shipped: bool,
    #[ignore_malloc_size_of = "Defined in std"]
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

#[dom_struct]
pub struct MessagePort {
    eventtarget: EventTarget,
    detached: Cell<bool>,
    #[ignore_malloc_size_of = "Defined in std"]
    message_port_internal: Arc<Mutex<MessagePortInternal>>,
}

impl MessagePort {
    fn new_inherited() -> MessagePort {
        MessagePort {
            eventtarget: EventTarget::new_inherited(),
            detached: Cell::new(false),
            message_port_internal: Arc::new(Mutex::new(MessagePortInternal::new())),
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
        let mut internal = self.message_port_internal.lock().unwrap();
        internal.process_pending_port_messages();
    }
}

impl MessagePortMethods for MessagePort {
    #[allow(unsafe_code)]
    /// <https://html.spec.whatwg.org/multipage/#dom-messageport-postmessage>
    unsafe fn PostMessage(
        &self,
        cx: *mut JSContext,
        message: HandleValue,
        transfer: HandleValue
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
