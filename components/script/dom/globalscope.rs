/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use crate::dom::bindings::cell::DomRefCell;
use crate::dom::bindings::codegen::Bindings::EventHandlerBinding::EventHandlerNonNull;
use crate::dom::bindings::codegen::Bindings::EventSourceBinding::EventSourceBinding::EventSourceMethods;
use crate::dom::bindings::codegen::Bindings::WindowBinding::WindowMethods;
use crate::dom::bindings::codegen::Bindings::WorkerGlobalScopeBinding::WorkerGlobalScopeMethods;
use crate::dom::bindings::conversions::{root_from_object, root_from_object_static};
use crate::dom::bindings::error::{report_pending_exception, ErrorInfo};
use crate::dom::bindings::inheritance::Castable;
use crate::dom::bindings::refcounted::Trusted;
use crate::dom::bindings::reflector::DomObject;
use crate::dom::bindings::root::{DomRoot, MutNullableDom};
use crate::dom::bindings::settings_stack::{entry_global, incumbent_global, AutoEntryScript};
use crate::dom::bindings::str::DOMString;
use crate::dom::bindings::structuredclone::StructuredCloneData;
use crate::dom::bindings::weakref::{DOMTracker, WeakRef};
use crate::dom::crypto::Crypto;
use crate::dom::dedicatedworkerglobalscope::DedicatedWorkerGlobalScope;
use crate::dom::errorevent::ErrorEvent;
use crate::dom::event::{Event, EventBubbles, EventCancelable, EventStatus};
use crate::dom::eventsource::EventSource;
use crate::dom::eventtarget::EventTarget;
use crate::dom::messageevent::MessageEvent;
use crate::dom::messageport::{MessagePort, MessagePortImpl};
use crate::dom::paintworkletglobalscope::PaintWorkletGlobalScope;
use crate::dom::performance::Performance;
use crate::dom::window::Window;
use crate::dom::workerglobalscope::WorkerGlobalScope;
use crate::dom::workletglobalscope::WorkletGlobalScope;
use crate::microtask::{Microtask, MicrotaskQueue};
use crate::script_runtime::{CommonScriptMsg, JSContext as SafeJSContext, ScriptChan, ScriptPort};
use crate::script_thread::{MainThreadScriptChan, ScriptThread};
use crate::task::TaskCanceller;
use crate::task_source::dom_manipulation::DOMManipulationTaskSource;
use crate::task_source::file_reading::FileReadingTaskSource;
use crate::task_source::networking::NetworkingTaskSource;
use crate::task_source::performance_timeline::PerformanceTimelineTaskSource;
use crate::task_source::port_message::PortMessageQueue;
use crate::task_source::remote_event::RemoteEventTaskSource;
use crate::task_source::websocket::WebsocketTaskSource;
use crate::task_source::TaskSource;
use crate::task_source::TaskSourceName;
use crate::timers::{IsInterval, OneshotTimerCallback, OneshotTimerHandle};
use crate::timers::{OneshotTimers, TimerCallback};
use devtools_traits::{ScriptToDevtoolsControlMsg, WorkerId};
use dom_struct::dom_struct;
use ipc_channel::ipc::{self, IpcSender};
use ipc_channel::router::ROUTER;
use js::glue::{IsWrapper, UnwrapObjectDynamic};
use js::jsapi::JSObject;
use js::jsapi::{CurrentGlobalOrNull, GetNonCCWObjectGlobal};
use js::jsapi::{HandleObject, Heap};
use js::jsapi::{JSAutoRealm, JSContext};
use js::jsval::UndefinedValue;
use js::panic::maybe_resume_unwind;
use js::rust::wrappers::EvaluateUtf8;
use js::rust::{get_object_class, CompileOptionsWrapper, ParentRuntime, Runtime};
use js::rust::{HandleValue, MutableHandleValue};
use js::{JSCLASS_IS_DOMJSCLASS, JSCLASS_IS_GLOBAL};
use msg::constellation_msg::{MessagePortId, MessagePortMsg, PipelineId, PortMessageTask};
use net_traits::image_cache::ImageCache;
use net_traits::{CoreResourceThread, IpcSend, ResourceThreads};
use profile_traits::{mem as profile_mem, time as profile_time};
use script_traits::{MsDuration, ScriptMsg, ScriptToConstellationChan, TimerEvent};
use script_traits::{TimerEventId, TimerSchedulerMsg, TimerSource};
use servo_url::{MutableOrigin, ServoUrl};
use std::borrow::Cow;
use std::cell::Cell;
use std::collections::hash_map::Entry;
use std::collections::{HashMap, HashSet};
use std::ffi::CString;
use std::rc::Rc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use time::{get_time, Timespec};

#[derive(JSTraceable)]
pub struct AutoCloseWorker(Arc<AtomicBool>);

impl Drop for AutoCloseWorker {
    fn drop(&mut self) {
        self.0.store(true, Ordering::SeqCst);
    }
}

unsafe_no_jsmanaged_fields!(MessagePortImpl);
unsafe_no_jsmanaged_fields!(MessagePortId);

#[dom_struct]
pub struct GlobalScope {
    eventtarget: EventTarget,
    crypto: MutNullableDom<Crypto>,
    next_worker_id: Cell<WorkerId>,

    #[ignore_malloc_size_of = "Rc<T> is hard"]
    /// The listeners for message event for a given port.
    message_listeners: DomRefCell<HashMap<MessagePortId, Option<Rc<EventHandlerNonNull>>>>,

    /// The message-ports know to this global.
    message_ports: DomRefCell<HashMap<MessagePortId, MessagePortImpl>>,

    /// Message-ports we know about, but whose transfer is pending.
    pending_message_ports: DomRefCell<HashSet<MessagePortId>>,

    #[ignore_malloc_size_of = "Channels are hard"]
    /// Senders to port we know about, that are currently pending to be managed by this global.
    port_senders: DomRefCell<HashMap<MessagePortId, IpcSender<MessagePortMsg>>>,

    /// The DOM messageport objects.
    message_port_tracker: DomRefCell<HashMap<MessagePortId, WeakRef<MessagePort>>>,

    /// The MessagePorts that might be GC'ed
    /// The first u32 is a counter, the second is the point where we want to re-check the GC.
    /// Each time we reach a point, we double it's value to avoid checking too often.
    message_port_potential_gc: DomRefCell<HashMap<MessagePortId, (u32, u32)>>,

    /// Message-ports to remove. Needed to avoid a double-borrow.
    message_ports_to_remove: DomRefCell<HashSet<MessagePortId>>,

    /// The message-ports that have been transferred out of this global.
    message_ports_transferred: DomRefCell<HashSet<MessagePortId>>,

    /// Pipeline id associated with this global.
    pipeline_id: PipelineId,

    /// A flag to indicate whether the developer tools has requested
    /// live updates from the worker.
    devtools_wants_updates: Cell<bool>,

    /// Timers used by the Console API.
    console_timers: DomRefCell<HashMap<DOMString, u64>>,

    /// For providing instructions to an optional devtools server.
    #[ignore_malloc_size_of = "channels are hard"]
    devtools_chan: Option<IpcSender<ScriptToDevtoolsControlMsg>>,

    /// For sending messages to the memory profiler.
    #[ignore_malloc_size_of = "channels are hard"]
    mem_profiler_chan: profile_mem::ProfilerChan,

    /// For sending messages to the time profiler.
    #[ignore_malloc_size_of = "channels are hard"]
    time_profiler_chan: profile_time::ProfilerChan,

    /// A handle for communicating messages to the constellation thread.
    #[ignore_malloc_size_of = "channels are hard"]
    script_to_constellation_chan: ScriptToConstellationChan,

    #[ignore_malloc_size_of = "channels are hard"]
    scheduler_chan: IpcSender<TimerSchedulerMsg>,

    /// <https://html.spec.whatwg.org/multipage/#in-error-reporting-mode>
    in_error_reporting_mode: Cell<bool>,

    /// Associated resource threads for use by DOM objects like XMLHttpRequest,
    /// including resource_thread, filemanager_thread and storage_thread
    resource_threads: ResourceThreads,

    timers: OneshotTimers,

    /// The origin of the globalscope
    origin: MutableOrigin,

    /// The microtask queue associated with this global.
    ///
    /// It is refcounted because windows in the same script thread share the
    /// same microtask queue.
    ///
    /// <https://html.spec.whatwg.org/multipage/#microtask-queue>
    #[ignore_malloc_size_of = "Rc<T> is hard"]
    microtask_queue: Rc<MicrotaskQueue>,

    /// Vector storing closing references of all workers
    #[ignore_malloc_size_of = "Arc"]
    list_auto_close_worker: DomRefCell<Vec<AutoCloseWorker>>,

    /// Vector storing references of all eventsources.
    event_source_tracker: DOMTracker<EventSource>,

    /// Storage for watching rejected promises waiting for some client to
    /// consume their rejection.
    /// Promises in this list have been rejected in the last turn of the
    /// event loop without the rejection being handled.
    /// Note that this can contain nullptrs in place of promises removed because
    /// they're consumed before it'd be reported.
    ///
    /// <https://html.spec.whatwg.org/multipage/#about-to-be-notified-rejected-promises-list>
    #[ignore_malloc_size_of = "mozjs"]
    uncaught_rejections: DomRefCell<Vec<Box<Heap<*mut JSObject>>>>,

    /// Promises in this list have previously been reported as rejected
    /// (because they were in the above list), but the rejection was handled
    /// in the last turn of the event loop.
    ///
    /// <https://html.spec.whatwg.org/multipage/#outstanding-rejected-promises-weak-set>
    #[ignore_malloc_size_of = "mozjs"]
    consumed_rejections: DomRefCell<Vec<Box<Heap<*mut JSObject>>>>,

    /// True if headless mode.
    is_headless: bool,

    /// An optional string allowing the user agent to be set for testing.
    user_agent: Cow<'static, str>,
}

/// A wrapper for glue-code between the ipc router and the event-loop.
struct MessageListener {
    canceller: TaskCanceller,
    task_source: PortMessageQueue,
    context: Trusted<GlobalScope>,
}

impl MessageListener {
    /// A new message came in, handle it via a task enqueued on the event-loop.
    /// A task is required, since we are using a trusted globalscope,
    /// and we can only access the root from the event-loop.
    fn notify(&self, msg: MessagePortMsg) {
        match msg {
            MessagePortMsg::CompleteTransfer(
                port_id,
                tasks,
                outgoing_msgs,
                entangled_with,
                entangled_sender,
            ) => {
                let context = self.context.clone();
                let _ = self.task_source.queue_with_canceller(
                    task!(process_new_entangled_sender: move || {
                        let global = context.root();
                        if let Some(port) = global.message_ports.borrow_mut().get_mut(&port_id) {
                            port.complete_transfer(tasks, outgoing_msgs, entangled_with, entangled_sender);
                            if port.enabled() {
                                port.start(&global);
                            }
                        };
                    }),
                    &self.canceller,
                );
            },
            MessagePortMsg::NewEntangledSender(port_id, ipc_sender) => {
                let context = self.context.clone();
                let _ = self.task_source.queue_with_canceller(
                    task!(process_new_entangled_sender: move || {
                        let global = context.root();
                        if let Some(port) = global.message_ports.borrow_mut().get_mut(&port_id) {
                            port.set_entangled_sender(ipc_sender);
                        };
                    }),
                    &self.canceller,
                );
            },
            MessagePortMsg::EntangledPortShipped(port_id) => {
                let context = self.context.clone();
                let _ = self.task_source.queue_with_canceller(
                    task!(process_entangled_port_shipped: move || {
                        let global = context.root();
                        if let Some(port) = global.message_ports.borrow_mut().get_mut(&port_id) {
                            port.set_has_been_shipped();
                        };
                    }),
                    &self.canceller,
                );
            },
            MessagePortMsg::NewTask(port_id, task) => {
                let context = self.context.clone();
                let _ = self.task_source.queue_with_canceller(
                    task!(process_new_task: move || {
                        let global = context.root();
                        global.route_task_to_port(port_id, task);
                    }),
                    &self.canceller,
                );
            },
            MessagePortMsg::RemoveMessagePort(port_id) => {
                let context = self.context.clone();
                let _ = self.task_source.queue_with_canceller(
                    task!(process_remove_message_port: move || {
                        let global = context.root();
                        global.remove_message_port(port_id);
                    }),
                    &self.canceller,
                );
            },
            MessagePortMsg::PotentialGC(port_id) => {
                let context = self.context.clone();
                let _ = self.task_source.queue_with_canceller(
                    task!(process_remove_message_port: move || {
                        let global = context.root();
                        global.handle_check_gc_messageport(port_id);
                    }),
                    &self.canceller,
                );
            },
            MessagePortMsg::ComfirmGC(port_id) => {
                let context = self.context.clone();
                let _ = self.task_source.queue_with_canceller(
                    task!(process_remove_message_port: move || {
                        let global = context.root();
                        global.handle_comfirmed_gc_messageport(port_id);
                    }),
                    &self.canceller,
                );
            },
        }
    }
}

impl GlobalScope {
    pub fn new_inherited(
        pipeline_id: PipelineId,
        devtools_chan: Option<IpcSender<ScriptToDevtoolsControlMsg>>,
        mem_profiler_chan: profile_mem::ProfilerChan,
        time_profiler_chan: profile_time::ProfilerChan,
        script_to_constellation_chan: ScriptToConstellationChan,
        scheduler_chan: IpcSender<TimerSchedulerMsg>,
        resource_threads: ResourceThreads,
        timer_event_chan: IpcSender<TimerEvent>,
        origin: MutableOrigin,
        microtask_queue: Rc<MicrotaskQueue>,
        is_headless: bool,
        user_agent: Cow<'static, str>,
    ) -> Self {
        Self {
            message_listeners: DomRefCell::new(HashMap::new()),
            message_ports: DomRefCell::new(HashMap::new()),
            pending_message_ports: DomRefCell::new(HashSet::new()),
            message_ports_to_remove: DomRefCell::new(HashSet::new()),
            message_port_tracker: DomRefCell::new(HashMap::new()),
            message_port_potential_gc: DomRefCell::new(HashMap::new()),
            message_ports_transferred: DomRefCell::new(HashSet::new()),
            port_senders: DomRefCell::new(HashMap::new()),
            eventtarget: EventTarget::new_inherited(),
            crypto: Default::default(),
            next_worker_id: Cell::new(WorkerId(0)),
            pipeline_id,
            devtools_wants_updates: Default::default(),
            console_timers: DomRefCell::new(Default::default()),
            devtools_chan,
            mem_profiler_chan,
            time_profiler_chan,
            script_to_constellation_chan,
            scheduler_chan: scheduler_chan.clone(),
            in_error_reporting_mode: Default::default(),
            resource_threads,
            timers: OneshotTimers::new(timer_event_chan, scheduler_chan),
            origin,
            microtask_queue,
            list_auto_close_worker: Default::default(),
            event_source_tracker: DOMTracker::new(),
            uncaught_rejections: Default::default(),
            consumed_rejections: Default::default(),
            is_headless,
            user_agent,
        }
    }

    /// <https://html.spec.whatwg.org/multipage/#entangle>
    pub fn entangle_ports(&self, port1: MessagePortId, port2: MessagePortId) {
        let dom_port1 = self.get_dom_message_port(&port1);
        dom_port1.entangle(port2.clone());

        let dom_port2 = self.get_dom_message_port(&port2);
        dom_port2.entangle(port1.clone());

        match self.message_ports.borrow_mut().get(&port1) {
            Some(port1) => {
                port1.entangle(port2.clone());
            },
            _ => unreachable!("Ports to entangle should exist"),
        }

        match self.message_ports.borrow_mut().get(&port2) {
            Some(port2) => {
                port2.entangle(port1.clone());
            },
            _ => unreachable!("Ports to entangle should exist"),
        }

        let _ = self
            .script_to_constellation_chan()
            .send(ScriptMsg::EntanglePorts(port1, port2));
    }

    /// Remove all referrences to a port.
    pub fn remove_message_port(&self, port_id: MessagePortId) {
        self.message_ports.borrow_mut().remove(&port_id);
        self.message_port_tracker.borrow_mut().remove(&port_id);
        self.message_listeners.borrow_mut().remove(&port_id);
    }

    /// Handle the transfer of a port in the current task.
    pub fn mark_port_as_transferred(&self, port_id: MessagePortId) {
        // Keep track of ports we transferred, for potential re-routing.
        self.message_ports_transferred
            .borrow_mut()
            .insert(port_id.clone());

        // Remove port, and let a locally tracked entangled port know.
        let entangled_id = self
            .message_ports
            .borrow_mut()
            .remove(&port_id)
            .and_then(|port| {
                let (incoming, outgoing) = port.message_buffers();
                let _ = self
                    .script_to_constellation_chan()
                    .send(ScriptMsg::MessagePortShipped(
                        port_id.clone(),
                        port.entangled_port_id().clone(),
                        incoming,
                        outgoing,
                    ));
                port.entangled_port_id()
            });
        let entangled_id = match entangled_id {
            Some(id) => id,
            None => return,
        };
        if let Some(entangled) = self.message_ports.borrow().get(&entangled_id) {
            entangled.set_has_been_shipped()
        }
    }

    /// Enable a port.
    pub fn start_message_port(&self, port_id: &MessagePortId) {
        let message_ports = self.message_ports.borrow();
        let port = message_ports
            .get(port_id)
            .expect("Port whose start was called to exist");
        port.start(self);
    }

    /// Close a port.
    pub fn close_message_port(&self, port_id: &MessagePortId) {
        let message_ports = self.message_ports.borrow();
        let port = message_ports
            .get(port_id)
            .expect("Port whose close was called to exist");
        port.close();
    }

    /// Get the onmessage handler for a port.
    pub fn get_message_port_onmessage(
        &self,
        port_id: &MessagePortId,
    ) -> Option<Rc<EventHandlerNonNull>> {
        if let Some(listener) = self.message_listeners.borrow().get(port_id) {
            return listener.clone();
        }
        None
    }

    /// Set the onmessage handler for a port.
    pub fn set_message_port_onmessage(
        &self,
        port_id: &MessagePortId,
        listener: Option<Rc<EventHandlerNonNull>>,
    ) {
        self.message_listeners
            .borrow_mut()
            .insert(port_id.clone(), listener);
        let message_ports = self.message_ports.borrow();
        let port = message_ports
            .get(port_id)
            .expect("Port whose onmessage was set to exist");
        port.start(self);
    }

    /// Post a message via a port.
    pub fn post_messageport_msg(&self, port_id: MessagePortId, task: PortMessageTask) {
        let message_ports = self.message_ports.borrow();
        let port = message_ports
            .get(&port_id)
            .expect("Port whose postMessage was set to exist");
        port.post_message(self, task);
    }

    /// Route the task to be handled by the relevant port.
    pub fn route_task_to_port(&self, port_id: MessagePortId, task: PortMessageTask) {
        if self
            .message_ports_transferred
            .borrow_mut()
            .contains(&port_id)
        {
            // If the port has already been transferred before this message arrived, re-route it.
            // This should only happen initially, when the entangled ports sends a message
            // and hasn't been updated about the transfer yet.
            let _ = self
                .script_to_constellation_chan()
                .send(ScriptMsg::RerouteMessagePort(port_id.clone(), task));
            return;
        }

        // If the port is not enabled yet, or if it is awaiting the completion of it's transfer,
        // the task will be buffered and dispatched upon enablement or completion of the transfer.
        let should_dispatch = self
            .message_ports
            .borrow_mut()
            .get_mut(&port_id)
            .map_or(false, |port| port.handle_incoming(&task));

        if should_dispatch {
            // Get a corresponding DOM message-port object.
            // Any existing event-listeners will be set on it.
            let dom_port = self.get_dom_message_port(&port_id);

            let PortMessageTask { origin, data } = task;

            // Substep 3-4
            rooted!(in(*self.get_cx()) let mut message_clone = UndefinedValue());
            if let Ok(deserialize_result) =
                StructuredCloneData::Vector(data).read(self, message_clone.handle_mut())
            {
                // Substep 6
                // Dispatch the event, using the dom message-port.
                MessageEvent::dispatch_jsval(
                    &dom_port.upcast(),
                    self,
                    message_clone.handle(),
                    Some(&origin),
                    None,
                    deserialize_result.message_ports.into_iter().collect(),
                );
            }
        }
    }

    /// Check all ports that have been transfer-received in the previous task,
    /// and complete their transfer if they haven't been re-transferred.
    pub fn maybe_add_pending_ports(&self) {
        for port_id in self.pending_message_ports.borrow_mut().drain() {
            let control_sender = self
                .port_senders
                .borrow_mut()
                .remove(&port_id)
                .expect("This global to have stored a sender for this pending port");
            if !self.message_ports_transferred.borrow().contains(&port_id) {
                let _ = self
                    .script_to_constellation_chan()
                    .send(ScriptMsg::NewMessagePort(port_id.clone(), control_sender));
            }
        }
    }

    /// https://html.spec.whatwg.org/multipage/#ports-and-garbage-collection
    pub fn perform_a_message_port_garbage_collection_checkpoint(&self) {
        let mut to_be_removed = self.message_ports_to_remove.borrow_mut();
        for (id, port) in self.message_ports.borrow().iter() {
            let alive_js = match self.message_port_tracker.borrow().get(&id) {
                Some(weak) => weak.root().is_some(),
                None => false,
            };

            if !alive_js && !port.is_entangled() {
                to_be_removed.insert(id.clone());
                // Let the constellation know to drop this port and the one it is entangled with,
                // and to forward this message to the script-process where the entangled is found.
                let _ = self
                    .script_to_constellation_chan()
                    .send(ScriptMsg::RemoveMessagePort(id.clone()));
                continue;
            }

            if !alive_js && port.possibly_unreachable() {
                let mut potential_gc = self.message_port_potential_gc.borrow_mut();
                let mut entry = potential_gc.entry(id.clone()).or_insert((1, 1));
                // Each time entry.0 reaches entry.1, we send a message to check if we can GC.
                // To avoid sending too many messages, entry.1 doubles each time it is reached,
                // while entry.0 increments at each GC checkpoint.
                if entry.0 == entry.1 {
                    entry.1 = entry.1.checked_mul(2).unwrap_or(u32::max_value());
                    port.send_potential_gc_msg();
                }
                entry.0 = entry.0.checked_add(1).unwrap_or(0);
            }
        }
        for id in to_be_removed.drain() {
            self.remove_message_port(id);
        }
    }

    /// Handle the case were our entangled port is ready for GC,
    /// and wants to know if we are still used.
    pub fn handle_check_gc_messageport(&self, port_id: MessagePortId) {
        let alive_js = match self.message_port_tracker.borrow().get(&port_id) {
            Some(weak) => weak.root().is_some(),
            None => false,
        };
        if !alive_js {
            if let Some(port) = self.message_ports.borrow().get(&port_id) {
                port.comfirm_gc();
            }
        }
    }

    /// Handle the case were our entangled port has confirmed to us we can GC.
    pub fn handle_comfirmed_gc_messageport(&self, port_id: MessagePortId) {
        let alive_js = match self.message_port_tracker.borrow().get(&port_id) {
            Some(weak) => weak.root().is_some(),
            None => false,
        };
        // Check again, in case a message was in transit and received after we last checked.
        let still_possibly_unreachable =
            if let Some(port) = self.message_ports.borrow().get(&port_id) {
                port.possibly_unreachable()
            } else {
                false
            };
        if !alive_js && still_possibly_unreachable {
            self.remove_message_port(port_id.clone());
            // Let the constellation know to drop this port and the one it is entangled with,
            // and to forward this message to the script-process where the entangled is found.
            let _ = self
                .script_to_constellation_chan()
                .send(ScriptMsg::RemoveMessagePort(port_id.clone()));
        }
    }

    /// Get a DOM object corresponding to this MessagePortId.
    fn get_dom_message_port(&self, port_id: &MessagePortId) -> DomRoot<MessagePort> {
        let listener = self.get_message_port_onmessage(port_id);
        let mut dom_ports = self.message_port_tracker.borrow_mut();
        let weak = dom_ports.get(port_id).expect("MessagePort to be tracked");
        let dom_port = match weak.root() {
            Some(dom_port) => dom_port,
            None => {
                // If the current one has been GC'ed, we create a new one.
                let dom_port = MessagePort::new_existing(self, port_id.clone());
                dom_ports.insert(port_id.clone(), WeakRef::new(&dom_port));
                dom_port
            },
        };
        // Set any existing message event-listener for this port on the dom port.
        dom_port.set_onmessage(listener);
        dom_port
    }

    /// Start tracking a message-port
    pub fn track_message_port(&self, dom_port: &DomRoot<MessagePort>, transfer_received: bool) {
        let message_port_id = dom_port.message_port_id().clone();

        // Store the DOM object.
        self.message_port_tracker
            .borrow_mut()
            .insert(message_port_id.clone(), WeakRef::new(dom_port));

        let port = MessagePortImpl::new(message_port_id.clone(), transfer_received);

        // If the port was transferred back into here,
        // remove it from the transferred ports.
        self.message_ports_transferred
            .borrow_mut()
            .remove(&message_port_id);

        let (port_control_sender, port_control_receiver) =
            ipc::channel().expect("ipc channel failure");

        if transfer_received {
            // We keep transfer-received ports as "pending",
            // and only ask the constellation to complete the tranfer
            // if they're not re-shipped in the current task.
            self.pending_message_ports
                .borrow_mut()
                .insert(message_port_id.clone());

            // Keep the control_sender, to send later to the constellation,
            // if we request the transfer to be completed.
            self.port_senders
                .borrow_mut()
                .insert(message_port_id.clone(), port_control_sender);

            // Queue a task to complete the transfer,
            // unless the port is re-transferred in the current task.
            let this = Trusted::new(&*self);
            let _ = self.port_message_queue().queue(
                task!(process_pending_port_messages: move || {
                    let target_global = this.root();
                    target_global.maybe_add_pending_ports();
                }),
                &self,
            );
        } else {
            // If this is a newly-created port, let the constellation immediately know.
            let _ = self
                .script_to_constellation_chan()
                .send(ScriptMsg::NewMessagePort(
                    message_port_id.clone(),
                    port_control_sender.clone(),
                ));
        }

        // Store the MessagePortImpl.
        self.message_ports
            .borrow_mut()
            .insert(message_port_id.clone(), port);

        // Setup a route for IPC, for messages from the constellation and our entangled port.
        let context = Trusted::new(self);
        let (task_source, canceller) = (
            self.port_message_queue(),
            self.task_canceller(TaskSourceName::PortMessage),
        );
        let listener = MessageListener {
            canceller,
            task_source,
            context,
        };
        ROUTER.add_route(
            port_control_receiver.to_opaque(),
            Box::new(move |message| {
                let msg = message.to();
                match msg {
                    Ok(msg) => listener.notify(msg),
                    Err(err) => warn!("Error receiving a MessagePortMsg: {:?}", err),
                }
            }),
        );
    }

    pub fn track_worker(&self, closing_worker: Arc<AtomicBool>) {
        self.list_auto_close_worker
            .borrow_mut()
            .push(AutoCloseWorker(closing_worker));
    }

    pub fn track_event_source(&self, event_source: &EventSource) {
        self.event_source_tracker.track(event_source);
    }

    pub fn close_event_sources(&self) -> bool {
        let mut canceled_any_fetch = false;
        self.event_source_tracker
            .for_each(
                |event_source: DomRoot<EventSource>| match event_source.ReadyState() {
                    2 => {},
                    _ => {
                        event_source.cancel();
                        canceled_any_fetch = true;
                    },
                },
            );
        canceled_any_fetch
    }

    /// Returns the global scope of the realm that the given DOM object's reflector
    /// was created in.
    #[allow(unsafe_code)]
    pub fn from_reflector<T: DomObject>(reflector: &T) -> DomRoot<Self> {
        unsafe { GlobalScope::from_object(*reflector.reflector().get_jsobject()) }
    }

    /// Returns the global scope of the realm that the given JS object was created in.
    #[allow(unsafe_code)]
    pub unsafe fn from_object(obj: *mut JSObject) -> DomRoot<Self> {
        assert!(!obj.is_null());
        let global = GetNonCCWObjectGlobal(obj);
        global_scope_from_global_static(global)
    }

    /// Returns the global scope for the given JSContext
    #[allow(unsafe_code)]
    pub unsafe fn from_context(cx: *mut JSContext) -> DomRoot<Self> {
        let global = CurrentGlobalOrNull(cx);
        global_scope_from_global(global, cx)
    }

    /// Returns the global object of the realm that the given JS object
    /// was created in, after unwrapping any wrappers.
    #[allow(unsafe_code)]
    pub unsafe fn from_object_maybe_wrapped(
        mut obj: *mut JSObject,
        cx: *mut JSContext,
    ) -> DomRoot<Self> {
        if IsWrapper(obj) {
            obj = UnwrapObjectDynamic(obj, cx, /* stopAtWindowProxy = */ 0);
            assert!(!obj.is_null());
        }
        GlobalScope::from_object(obj)
    }

    pub fn add_uncaught_rejection(&self, rejection: HandleObject) {
        self.uncaught_rejections
            .borrow_mut()
            .push(Heap::boxed(rejection.get()));
    }

    pub fn remove_uncaught_rejection(&self, rejection: HandleObject) {
        let mut uncaught_rejections = self.uncaught_rejections.borrow_mut();

        if let Some(index) = uncaught_rejections
            .iter()
            .position(|promise| *promise == Heap::boxed(rejection.get()))
        {
            uncaught_rejections.remove(index);
        }
    }

    pub fn get_uncaught_rejections(&self) -> &DomRefCell<Vec<Box<Heap<*mut JSObject>>>> {
        &self.uncaught_rejections
    }

    pub fn add_consumed_rejection(&self, rejection: HandleObject) {
        self.consumed_rejections
            .borrow_mut()
            .push(Heap::boxed(rejection.get()));
    }

    pub fn remove_consumed_rejection(&self, rejection: HandleObject) {
        let mut consumed_rejections = self.consumed_rejections.borrow_mut();

        if let Some(index) = consumed_rejections
            .iter()
            .position(|promise| *promise == Heap::boxed(rejection.get()))
        {
            consumed_rejections.remove(index);
        }
    }

    pub fn get_consumed_rejections(&self) -> &DomRefCell<Vec<Box<Heap<*mut JSObject>>>> {
        &self.consumed_rejections
    }

    #[allow(unsafe_code)]
    pub fn get_cx(&self) -> SafeJSContext {
        unsafe { SafeJSContext::from_ptr(Runtime::get()) }
    }

    pub fn crypto(&self) -> DomRoot<Crypto> {
        self.crypto.or_init(|| Crypto::new(self))
    }

    /// Get next worker id.
    pub fn get_next_worker_id(&self) -> WorkerId {
        let worker_id = self.next_worker_id.get();
        let WorkerId(id_num) = worker_id;
        self.next_worker_id.set(WorkerId(id_num + 1));
        worker_id
    }

    pub fn live_devtools_updates(&self) -> bool {
        self.devtools_wants_updates.get()
    }

    pub fn set_devtools_wants_updates(&self, value: bool) {
        self.devtools_wants_updates.set(value);
    }

    pub fn time(&self, label: DOMString) -> Result<(), ()> {
        let mut timers = self.console_timers.borrow_mut();
        if timers.len() >= 10000 {
            return Err(());
        }
        match timers.entry(label) {
            Entry::Vacant(entry) => {
                entry.insert(timestamp_in_ms(get_time()));
                Ok(())
            },
            Entry::Occupied(_) => Err(()),
        }
    }

    pub fn time_end(&self, label: &str) -> Result<u64, ()> {
        self.console_timers
            .borrow_mut()
            .remove(label)
            .ok_or(())
            .map(|start| timestamp_in_ms(get_time()) - start)
    }

    /// Get an `&IpcSender<ScriptToDevtoolsControlMsg>` to send messages
    /// to the devtools thread when available.
    pub fn devtools_chan(&self) -> Option<&IpcSender<ScriptToDevtoolsControlMsg>> {
        self.devtools_chan.as_ref()
    }

    /// Get a sender to the memory profiler thread.
    pub fn mem_profiler_chan(&self) -> &profile_mem::ProfilerChan {
        &self.mem_profiler_chan
    }

    /// Get a sender to the time profiler thread.
    pub fn time_profiler_chan(&self) -> &profile_time::ProfilerChan {
        &self.time_profiler_chan
    }

    /// Get a sender to the constellation thread.
    pub fn script_to_constellation_chan(&self) -> &ScriptToConstellationChan {
        &self.script_to_constellation_chan
    }

    pub fn scheduler_chan(&self) -> &IpcSender<TimerSchedulerMsg> {
        &self.scheduler_chan
    }

    /// Get the `PipelineId` for this global scope.
    pub fn pipeline_id(&self) -> PipelineId {
        self.pipeline_id
    }

    /// Get the origin for this global scope
    pub fn origin(&self) -> &MutableOrigin {
        &self.origin
    }

    pub fn image_cache(&self) -> Arc<dyn ImageCache> {
        if let Some(window) = self.downcast::<Window>() {
            return window.image_cache();
        }
        if let Some(worker) = self.downcast::<DedicatedWorkerGlobalScope>() {
            return worker.image_cache();
        }
        if let Some(worker) = self.downcast::<PaintWorkletGlobalScope>() {
            return worker.image_cache();
        }
        unreachable!();
    }

    /// Get the [base url](https://html.spec.whatwg.org/multipage/#api-base-url)
    /// for this global scope.
    pub fn api_base_url(&self) -> ServoUrl {
        if let Some(window) = self.downcast::<Window>() {
            // https://html.spec.whatwg.org/multipage/#script-settings-for-browsing-contexts:api-base-url
            return window.Document().base_url();
        }
        if let Some(worker) = self.downcast::<WorkerGlobalScope>() {
            // https://html.spec.whatwg.org/multipage/#script-settings-for-workers:api-base-url
            return worker.get_url().clone();
        }
        if let Some(worklet) = self.downcast::<WorkletGlobalScope>() {
            // https://drafts.css-houdini.org/worklets/#script-settings-for-worklets
            return worklet.base_url();
        }
        unreachable!();
    }

    /// Get the URL for this global scope.
    pub fn get_url(&self) -> ServoUrl {
        if let Some(window) = self.downcast::<Window>() {
            return window.get_url();
        }
        if let Some(worker) = self.downcast::<WorkerGlobalScope>() {
            return worker.get_url().clone();
        }
        if let Some(worklet) = self.downcast::<WorkletGlobalScope>() {
            // TODO: is this the right URL to return?
            return worklet.base_url();
        }
        unreachable!();
    }

    /// Extract a `Window`, panic if the global object is not a `Window`.
    pub fn as_window(&self) -> &Window {
        self.downcast::<Window>().expect("expected a Window scope")
    }

    /// <https://html.spec.whatwg.org/multipage/#report-the-error>
    pub fn report_an_error(&self, error_info: ErrorInfo, value: HandleValue) {
        // Step 1.
        if self.in_error_reporting_mode.get() {
            return;
        }

        // Step 2.
        self.in_error_reporting_mode.set(true);

        // Steps 3-6.
        // FIXME(#13195): muted errors.
        let event = ErrorEvent::new(
            self,
            atom!("error"),
            EventBubbles::DoesNotBubble,
            EventCancelable::Cancelable,
            error_info.message.as_str().into(),
            error_info.filename.as_str().into(),
            error_info.lineno,
            error_info.column,
            value,
        );

        // Step 7.
        let event_status = event.upcast::<Event>().fire(self.upcast::<EventTarget>());

        // Step 8.
        self.in_error_reporting_mode.set(false);

        // Step 9.
        if event_status == EventStatus::NotCanceled {
            // https://html.spec.whatwg.org/multipage/#runtime-script-errors-2
            if let Some(dedicated) = self.downcast::<DedicatedWorkerGlobalScope>() {
                dedicated.forward_error_to_worker_object(error_info);
            }
        }
    }

    /// Get the `&ResourceThreads` for this global scope.
    pub fn resource_threads(&self) -> &ResourceThreads {
        &self.resource_threads
    }

    /// Get the `CoreResourceThread` for this global scope.
    pub fn core_resource_thread(&self) -> CoreResourceThread {
        self.resource_threads().sender()
    }

    /// `ScriptChan` to send messages to the event loop of this global scope.
    pub fn script_chan(&self) -> Box<dyn ScriptChan + Send> {
        if let Some(window) = self.downcast::<Window>() {
            return MainThreadScriptChan(window.main_thread_script_chan().clone()).clone();
        }
        if let Some(worker) = self.downcast::<WorkerGlobalScope>() {
            return worker.script_chan();
        }
        unreachable!();
    }

    /// `TaskSource` to send messages to the networking task source of
    /// this global scope.
    pub fn networking_task_source(&self) -> NetworkingTaskSource {
        if let Some(window) = self.downcast::<Window>() {
            return window.task_manager().networking_task_source();
        }
        if let Some(worker) = self.downcast::<WorkerGlobalScope>() {
            return worker.networking_task_source();
        }
        unreachable!();
    }

    /// `TaskSource` to send messages to the port message queue of
    /// this global scope.
    pub fn port_message_queue(&self) -> PortMessageQueue {
        if let Some(window) = self.downcast::<Window>() {
            return window.task_manager().port_message_queue();
        }
        if let Some(worker) = self.downcast::<WorkerGlobalScope>() {
            return worker.port_message_queue();
        }
        unreachable!();
    }

    /// `TaskSource` to send messages to the remote-event task source of
    /// this global scope.
    pub fn remote_event_task_source(&self) -> RemoteEventTaskSource {
        if let Some(window) = self.downcast::<Window>() {
            return window.task_manager().remote_event_task_source();
        }
        if let Some(worker) = self.downcast::<WorkerGlobalScope>() {
            return worker.remote_event_task_source();
        }
        unreachable!();
    }

    /// `TaskSource` to send messages to the websocket task source of
    /// this global scope.
    pub fn websocket_task_source(&self) -> WebsocketTaskSource {
        if let Some(window) = self.downcast::<Window>() {
            return window.task_manager().websocket_task_source();
        }
        if let Some(worker) = self.downcast::<WorkerGlobalScope>() {
            return worker.websocket_task_source();
        }
        unreachable!();
    }

    /// Evaluate JS code on this global scope.
    pub fn evaluate_js_on_global_with_result(&self, code: &str, rval: MutableHandleValue) -> bool {
        self.evaluate_script_on_global_with_result(code, "", rval, 1)
    }

    /// Evaluate a JS script on this global scope.
    #[allow(unsafe_code)]
    pub fn evaluate_script_on_global_with_result(
        &self,
        code: &str,
        filename: &str,
        rval: MutableHandleValue,
        line_number: u32,
    ) -> bool {
        let metadata = profile_time::TimerMetadata {
            url: if filename.is_empty() {
                self.get_url().as_str().into()
            } else {
                filename.into()
            },
            iframe: profile_time::TimerMetadataFrameType::RootWindow,
            incremental: profile_time::TimerMetadataReflowType::FirstReflow,
        };
        profile_time::profile(
            profile_time::ProfilerCategory::ScriptEvaluate,
            Some(metadata),
            self.time_profiler_chan().clone(),
            || {
                let cx = self.get_cx();
                let globalhandle = self.reflector().get_jsobject();
                let filename = CString::new(filename).unwrap();

                let _ac = JSAutoRealm::new(*cx, globalhandle.get());
                let _aes = AutoEntryScript::new(self);
                let options = CompileOptionsWrapper::new(*cx, filename.as_ptr(), line_number);

                debug!("evaluating Dom string");
                let result = unsafe {
                    EvaluateUtf8(
                        *cx,
                        options.ptr,
                        code.as_ptr() as *const _,
                        code.len() as libc::size_t,
                        rval,
                    )
                };

                if !result {
                    debug!("error evaluating Dom string");
                    unsafe { report_pending_exception(*cx, true) };
                }

                maybe_resume_unwind();
                result
            },
        )
    }

    pub fn schedule_callback(
        &self,
        callback: OneshotTimerCallback,
        duration: MsDuration,
    ) -> OneshotTimerHandle {
        self.timers
            .schedule_callback(callback, duration, self.timer_source())
    }

    pub fn unschedule_callback(&self, handle: OneshotTimerHandle) {
        self.timers.unschedule_callback(handle);
    }

    pub fn set_timeout_or_interval(
        &self,
        callback: TimerCallback,
        arguments: Vec<HandleValue>,
        timeout: i32,
        is_interval: IsInterval,
    ) -> i32 {
        self.timers.set_timeout_or_interval(
            self,
            callback,
            arguments,
            timeout,
            is_interval,
            self.timer_source(),
        )
    }

    pub fn clear_timeout_or_interval(&self, handle: i32) {
        self.timers.clear_timeout_or_interval(self, handle)
    }

    pub fn fire_timer(&self, handle: TimerEventId) {
        self.timers.fire_timer(handle, self)
    }

    pub fn resume(&self) {
        self.timers.resume()
    }

    pub fn suspend(&self) {
        self.timers.suspend()
    }

    pub fn slow_down_timers(&self) {
        self.timers.slow_down()
    }

    pub fn speed_up_timers(&self) {
        self.timers.speed_up()
    }

    fn timer_source(&self) -> TimerSource {
        if self.is::<Window>() {
            return TimerSource::FromWindow(self.pipeline_id());
        }
        if self.is::<WorkerGlobalScope>() {
            return TimerSource::FromWorker;
        }
        unreachable!();
    }

    /// Returns the task canceller of this global to ensure that everything is
    /// properly cancelled when the global scope is destroyed.
    pub fn task_canceller(&self, name: TaskSourceName) -> TaskCanceller {
        if let Some(window) = self.downcast::<Window>() {
            return window.task_manager().task_canceller(name);
        }
        if let Some(worker) = self.downcast::<WorkerGlobalScope>() {
            // Note: the "name" is not passed to the worker,
            // because 'closing' it only requires one task canceller for all task sources.
            // https://html.spec.whatwg.org/multipage/#dom-workerglobalscope-closing
            return worker.task_canceller();
        }
        unreachable!();
    }

    /// Perform a microtask checkpoint.
    #[allow(unsafe_code)]
    pub fn perform_a_microtask_checkpoint(&self) {
        unsafe {
            self.microtask_queue.checkpoint(
                *self.get_cx(),
                |_| Some(DomRoot::from_ref(self)),
                vec![DomRoot::from_ref(self)],
            );
        }
    }

    /// Enqueue a microtask for subsequent execution.
    #[allow(unsafe_code)]
    pub fn enqueue_microtask(&self, job: Microtask) {
        unsafe {
            self.microtask_queue.enqueue(job, *self.get_cx());
        }
    }

    /// Create a new sender/receiver pair that can be used to implement an on-demand
    /// event loop. Used for implementing web APIs that require blocking semantics
    /// without resorting to nested event loops.
    pub fn new_script_pair(&self) -> (Box<dyn ScriptChan + Send>, Box<dyn ScriptPort + Send>) {
        if let Some(window) = self.downcast::<Window>() {
            return window.new_script_pair();
        }
        if let Some(worker) = self.downcast::<WorkerGlobalScope>() {
            return worker.new_script_pair();
        }
        unreachable!();
    }

    /// Returns the microtask queue of this global.
    pub fn microtask_queue(&self) -> &Rc<MicrotaskQueue> {
        &self.microtask_queue
    }

    /// Process a single event as if it were the next event
    /// in the thread queue for this global scope.
    pub fn process_event(&self, msg: CommonScriptMsg) {
        if self.is::<Window>() {
            return ScriptThread::process_event(msg);
        }
        if let Some(worker) = self.downcast::<WorkerGlobalScope>() {
            return worker.process_event(msg);
        }
        unreachable!();
    }

    pub fn dom_manipulation_task_source(&self) -> DOMManipulationTaskSource {
        if let Some(window) = self.downcast::<Window>() {
            return window.task_manager().dom_manipulation_task_source();
        }
        if let Some(worker) = self.downcast::<WorkerGlobalScope>() {
            return worker.dom_manipulation_task_source();
        }
        unreachable!();
    }

    /// Channel to send messages to the file reading task source of
    /// this of this global scope.
    pub fn file_reading_task_source(&self) -> FileReadingTaskSource {
        if let Some(window) = self.downcast::<Window>() {
            return window.task_manager().file_reading_task_source();
        }
        if let Some(worker) = self.downcast::<WorkerGlobalScope>() {
            return worker.file_reading_task_source();
        }
        unreachable!();
    }

    pub fn runtime_handle(&self) -> ParentRuntime {
        if self.is::<Window>() {
            ScriptThread::runtime_handle()
        } else if let Some(worker) = self.downcast::<WorkerGlobalScope>() {
            worker.runtime_handle()
        } else {
            unreachable!()
        }
    }

    /// Returns the ["current"] global object.
    ///
    /// ["current"]: https://html.spec.whatwg.org/multipage/#current
    #[allow(unsafe_code)]
    pub fn current() -> Option<DomRoot<Self>> {
        unsafe {
            let cx = Runtime::get();
            assert!(!cx.is_null());
            let global = CurrentGlobalOrNull(cx);
            if global.is_null() {
                None
            } else {
                Some(global_scope_from_global(global, cx))
            }
        }
    }

    /// Returns the ["entry"] global object.
    ///
    /// ["entry"]: https://html.spec.whatwg.org/multipage/#entry
    pub fn entry() -> DomRoot<Self> {
        entry_global()
    }

    /// Returns the ["incumbent"] global object.
    ///
    /// ["incumbent"]: https://html.spec.whatwg.org/multipage/#incumbent
    pub fn incumbent() -> Option<DomRoot<Self>> {
        incumbent_global()
    }

    pub fn performance(&self) -> DomRoot<Performance> {
        if let Some(window) = self.downcast::<Window>() {
            return window.Performance();
        }
        if let Some(worker) = self.downcast::<WorkerGlobalScope>() {
            return worker.Performance();
        }
        unreachable!();
    }

    /// Channel to send messages to the performance timeline task source
    /// of this global scope.
    pub fn performance_timeline_task_source(&self) -> PerformanceTimelineTaskSource {
        if let Some(window) = self.downcast::<Window>() {
            return window.task_manager().performance_timeline_task_source();
        }
        if let Some(worker) = self.downcast::<WorkerGlobalScope>() {
            return worker.performance_timeline_task_source();
        }
        unreachable!();
    }

    pub fn is_headless(&self) -> bool {
        self.is_headless
    }

    pub fn get_user_agent(&self) -> Cow<'static, str> {
        self.user_agent.clone()
    }
}

fn timestamp_in_ms(time: Timespec) -> u64 {
    (time.sec * 1000 + (time.nsec / 1000000) as i64) as u64
}

/// Returns the Rust global scope from a JS global object.
#[allow(unsafe_code)]
unsafe fn global_scope_from_global(
    global: *mut JSObject,
    cx: *mut JSContext,
) -> DomRoot<GlobalScope> {
    assert!(!global.is_null());
    let clasp = get_object_class(global);
    assert_ne!(
        ((*clasp).flags & (JSCLASS_IS_DOMJSCLASS | JSCLASS_IS_GLOBAL)),
        0
    );
    root_from_object(global, cx).unwrap()
}

/// Returns the Rust global scope from a JS global object.
#[allow(unsafe_code)]
unsafe fn global_scope_from_global_static(global: *mut JSObject) -> DomRoot<GlobalScope> {
    assert!(!global.is_null());
    let clasp = get_object_class(global);
    assert_ne!(
        ((*clasp).flags & (JSCLASS_IS_DOMJSCLASS | JSCLASS_IS_GLOBAL)),
        0
    );
    root_from_object_static(global).unwrap()
}
