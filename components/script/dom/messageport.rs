/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use dom::bindings::cell::DOMRefCell;
use dom::bindings::codegen::Bindings::EventHandlerBinding::EventHandlerNonNull;
use dom::bindings::codegen::Bindings::MessagePortBinding::{MessagePortMethods, Wrap};
use dom::bindings::error::{Error, ErrorResult, Fallible};
use dom::bindings::inheritance::Castable;
use dom::bindings::js::{MutNullableJS, Root};
use dom::bindings::refcounted::Trusted;
use dom::bindings::reflector::{DomObject, reflect_dom_object};
use dom::bindings::structuredclone::StructuredCloneData;
use dom::bindings::transferable::Transferable;
use dom::eventtarget::EventTarget;
use dom::globalscope::GlobalScope;
use dom::messageevent::MessageEvent;
use dom_struct::dom_struct;
use js::jsapi::{HandleValue, JSContext};
use js::jsval::UndefinedValue;
use script_thread::{Runnable, ScriptThread};
use std::cell::Cell;
use std::mem;
use std::rc::Rc;
use task_source::TaskSource;

#[dom_struct]
pub struct MessagePort {
    eventtarget: EventTarget,
    has_been_shipped: Cell<bool>,
    detached: Cell<bool>,
    enabled: Cell<bool>,
    #[ignore_heap_size_of = "Defined in std"]
    entangled_port: MutNullableJS<MessagePort>,
    pending_port_messages: DOMRefCell<Vec<PortMessageRunnable>>,
}

impl Transferable for MessagePort {
    // https://html.spec.whatwg.org/multipage/#message-ports:transfer-()
    #[allow(unsafe_code)]
    fn transfer(&self, target_global: &GlobalScope) -> Fallible<Root<MessagePort>> {
        // Step 1
        self.has_been_shipped.set(true);

        // Step 2
        let new = MessagePort::new(target_global);

        // Step 3
        new.has_been_shipped.set(true);

        // Step 4
        let trusted = Trusted::new(&*new);
        *new.pending_port_messages.borrow_mut() = if self.enabled.get() {
            ScriptThread::collect_message_port_tasks().into_iter().map(|task| {
                let mut runnable = *task.as_boxed_any().downcast::<PortMessageRunnable>().unwrap();
                runnable.target_port = trusted.clone();
                runnable
            }).collect()
        } else {
            let mut tasks = mem::replace(&mut *self.pending_port_messages.borrow_mut(), vec![]);
            for runnable in &mut tasks {
                runnable.target_port = trusted.clone();
            }
            tasks
        };

        // Step 5
        if let Some(remote_port) = self.entangled_port.take() {
            // Substep 2
            remote_port.has_been_shipped.set(true);

            // Substep 3
            new.entangle(&remote_port);
        }

        // Step 6
        self.detached.set(true);

        // Step 7
        Ok(new)
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

impl MessagePort {
    // https://html.spec.whatwg.org/multipage/#create-a-new-messageport-object
    fn new_inherited() -> MessagePort {
        MessagePort {
            eventtarget: EventTarget::new_inherited(),
            has_been_shipped: Cell::new(false),
            detached: Cell::new(false),
            enabled: Cell::new(false),
            entangled_port: MutNullableJS::default(),
            pending_port_messages: DOMRefCell::new(vec![]),
        }
    }

    pub fn new(owner: &GlobalScope) -> Root<MessagePort> {
        reflect_dom_object(box MessagePort::new_inherited(), owner, Wrap)
    }

    // https://html.spec.whatwg.org/multipage/#entangle
    pub fn entangle(&self, other: &MessagePort) {
        self.entangled_port.set(Some(other));
        other.entangled_port.set(Some(self));
    }
}

impl MessagePortMethods for MessagePort {
    #[allow(unsafe_code)]
    // https://html.spec.whatwg.org/multipage/#dom-messageport-postmessage
    unsafe fn PostMessage(&self,
                          cx: *mut JSContext,
                          message: HandleValue,
                          transfer: Option<Vec<HandleValue>>)
                          -> ErrorResult {
        // Step 1
        let target_port = self.entangled_port.get();

        // Step 2
        let mut doomed = false;

        if let Some(ref handles) = transfer {
            for handle in handles {
                let object = handle.to_object();

                // Step 3
                if object == self.reflector().rootable().get() {
                    return Err(Error::DataClone);
                }

                // Step 4
                if let Some(ref other) = target_port {
                    if object == other.reflector().rootable().get() {
                        doomed = true;
                    }
                }
            }
        }

        // TODO: Step 6 Support sending transferList

        // Step 7
        let message_clone = StructuredCloneData::write(cx, message)?.move_to_arraybuffer();

        // TODO: Step 8

        // Step 9
        if let (Some(ref other), false) = (target_port, doomed) {
            // Step 5
            let target_global = other.global();

            // Steps 10-11 performed in runnable
            let trusted = Trusted::new(&**other);
            let runnable = PortMessageRunnable {
                data: message_clone,
                target_port: trusted,
            };

            // Step 12
            if other.enabled.get() {
                let _ = target_global.port_message_queue().queue(box runnable, &target_global);
            } else {
                other.pending_port_messages.borrow_mut().push(runnable);
            }
        }

        Ok(())
    }

    // https://html.spec.whatwg.org/multipage/#dom-messageport-start
    fn Start(&self) {
        if self.enabled.get() {
            return;
        }
        self.enabled.set(true);

        if let Some(other) = self.entangled_port.get() {
            let global = other.global();
            for runnable in mem::replace(&mut *self.pending_port_messages.borrow_mut(), vec![]) {
                let _ = global.port_message_queue().queue(box runnable, &global);
            }
        }
    }

    // https://html.spec.whatwg.org/multipage/#dom-messageport-close
    fn Close(&self) {
        if let Some(entangled_port) = self.entangled_port.take() {
            entangled_port.entangled_port.set(None);
        }
    }

    // https://html.spec.whatwg.org/multipage/#handler-messageport-onmessage
    fn GetOnmessage(&self) -> Option<Rc<EventHandlerNonNull>> {
        let eventtarget = self.upcast::<EventTarget>();
        eventtarget.get_event_handler_common("message")
    }

    // https://html.spec.whatwg.org/multipage/#handler-messageport-onmessage
    fn SetOnmessage(&self, listener: Option<Rc<EventHandlerNonNull>>) {
        self.Start();
        let eventtarget = self.upcast::<EventTarget>();
        eventtarget.set_event_handler_common("message", listener)
    }
}

#[derive(HeapSizeOf, JSTraceable)]
struct PortMessageRunnable {
    data: Vec<u8>,
    #[ignore_heap_size_of = "Defined in std"]
    target_port: Trusted<MessagePort>,
}

impl Runnable for PortMessageRunnable {
    fn name(&self) -> &'static str { "PortMessageRunnable" }

    fn handler(self: Box<PortMessageRunnable>) {
        // Step 1
        let target = self.target_port.root();
        let global = target.global();

        rooted!(in(global.get_cx()) let mut message = UndefinedValue());
        StructuredCloneData::Vector(self.data).read(&global, message.handle_mut());

        // Step 2
        MessageEvent::dispatch_jsval(target.upcast(), &global, message.handle());
    }
}
