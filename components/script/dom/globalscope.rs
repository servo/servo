/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use crate::dom::bindings::cell::DomRefCell;
use crate::dom::bindings::codegen::Bindings::EventSourceBinding::EventSourceBinding::EventSourceMethods;
use crate::dom::bindings::codegen::Bindings::WindowBinding::WindowMethods;
use crate::dom::bindings::codegen::Bindings::WorkerGlobalScopeBinding::WorkerGlobalScopeMethods;
use crate::dom::bindings::conversions::{root_from_object, root_from_object_static};
use crate::dom::bindings::error::{report_pending_exception, ErrorInfo};
use crate::dom::bindings::inheritance::Castable;
use crate::dom::bindings::refcounted::Trusted;
use crate::dom::bindings::reflector::DomObject;
use crate::dom::bindings::root::{Dom, DomRoot, MutNullableDom};
use crate::dom::bindings::settings_stack::{entry_global, incumbent_global, AutoEntryScript};
use crate::dom::bindings::str::DOMString;
use crate::dom::bindings::transferable::Transferable;
use crate::dom::bindings::weakref::DOMTracker;
use crate::dom::crypto::Crypto;
use crate::dom::dedicatedworkerglobalscope::DedicatedWorkerGlobalScope;
use crate::dom::errorevent::ErrorEvent;
use crate::dom::event::{Event, EventBubbles, EventCancelable, EventStatus};
use crate::dom::eventsource::EventSource;
use crate::dom::eventtarget::EventTarget;
use crate::dom::messageport::MessagePort;
use crate::dom::paintworkletglobalscope::PaintWorkletGlobalScope;
use crate::dom::performance::Performance;
use crate::dom::window::Window;
use crate::dom::workerglobalscope::WorkerGlobalScope;
use crate::dom::workletglobalscope::WorkletGlobalScope;
use crate::microtask::{Microtask, MicrotaskQueue};
use crate::script_runtime::{CommonScriptMsg, ScriptChan, ScriptPort};
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
use js::panic::maybe_resume_unwind;
use js::rust::wrappers::EvaluateUtf8;
use js::rust::{get_object_class, CompileOptionsWrapper, ParentRuntime, Runtime};
use js::rust::{HandleValue, MutableHandleValue};
use js::{JSCLASS_IS_DOMJSCLASS, JSCLASS_IS_GLOBAL};
use msg::constellation_msg::{MessagePortId, MessagePortMsg, PipelineId, PortMessageTask};
use net_traits::image_cache::ImageCache;
use net_traits::{CoreResourceThread, IpcSend, ResourceThreads};
use parking_lot::Mutex;
use profile_traits::{mem as profile_mem, time as profile_time};
use script_traits::{MsDuration, ScriptMsg, ScriptToConstellationChan, TimerEvent};
use script_traits::{TimerEventId, TimerSchedulerMsg, TimerSource};
use servo_url::{MutableOrigin, ServoUrl};
use std::borrow::Cow;
use std::cell::Cell;
use std::collections::hash_map::{self, Entry};
use std::collections::HashMap;
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

#[derive(JSTraceable)]
#[must_root]
pub struct MessagePorts {
    ports: HashMap<MessagePortId, Dom<MessagePort>>,
}

impl MessagePorts {
    pub fn new() -> MessagePorts {
        MessagePorts {
            ports: HashMap::new(),
        }
    }

    pub fn insert(&mut self, port_id: MessagePortId, port: &MessagePort) {
        self.ports.insert(port_id, Dom::from_ref(port));
    }

    pub fn remove(&mut self, port_id: &MessagePortId) -> Option<DomRoot<MessagePort>> {
        self.ports
            .remove(port_id)
            .map(|ref port| DomRoot::from_ref(&**port))
    }

    pub fn find_port(&self, port_id: &MessagePortId) -> Option<DomRoot<MessagePort>> {
        self.ports
            .get(port_id)
            .map(|port| DomRoot::from_ref(&**port))
    }

    pub fn drain_ports(&mut self) -> Vec<(MessagePortId, DomRoot<MessagePort>)> {
        self.ports
            .drain()
            .map(|(id, ref port)| (id, DomRoot::from_ref(&**port)))
            .collect()
    }

    pub fn iter<'a>(&'a self) -> MessagePortsIter<'a> {
        MessagePortsIter {
            iter: self.ports.iter(),
        }
    }
}

#[allow(unrooted_must_root)]
pub struct MessagePortsIter<'a> {
    iter: hash_map::Iter<'a, MessagePortId, Dom<MessagePort>>,
}

impl<'a> Iterator for MessagePortsIter<'a> {
    type Item = (MessagePortId, DomRoot<MessagePort>);

    fn next(&mut self) -> Option<(MessagePortId, DomRoot<MessagePort>)> {
        self.iter
            .next()
            .map(|(id, port)| (*id, DomRoot::from_ref(&**port)))
    }
}

#[dom_struct]
pub struct GlobalScope {
    eventtarget: EventTarget,
    crypto: MutNullableDom<Crypto>,
    next_worker_id: Cell<WorkerId>,

    /// The message-ports know to this global.
    #[ignore_malloc_size_of = "MessagePorts are hard"]
    message_ports: DomRefCell<MessagePorts>,

    /// Message-ports we know about, but haven't used yet.
    /// Only necessary because we run message_ports tasks while borrowing message_ports,
    #[ignore_malloc_size_of = "MessagePorts are hard"]
    pending_message_ports: DomRefCell<MessagePorts>,

    /// The message-ports that are up for garbage-collection.
    message_ports_to_be_collected: DomRefCell<Vec<(MessagePortId, usize)>>,

    /// The message-ports that have been shipped in the last turn of the event-loop.
    /// The optional second Id is the port they're entangled with, if any.
    message_ports_shipped: DomRefCell<HashMap<MessagePortId, Option<MessagePortId>>>,

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
    task_source: Arc<Mutex<PortMessageQueue>>,
    context: Arc<Mutex<Trusted<GlobalScope>>>,
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
                let _ = self.task_source.lock().queue_with_canceller(
                    task!(process_new_entangled_sender: move || {
                        let global = context.lock().root();
                        global.maybe_add_message_port(&port_id);
                        if let Some(port) = global.upcast::<GlobalScope>().message_ports.borrow().find_port(&port_id) {
                            if !global.message_ports_shipped.borrow_mut().remove(&port_id).is_some() {
                                port.complete_transfer(tasks, outgoing_msgs, entangled_with, entangled_sender);
                            }
                        };
                    }),
                    &self.canceller,
                );
            },
            MessagePortMsg::NewEntangledSender(port_id, ipc_sender) => {
                let context = self.context.clone();
                let _ = self.task_source.lock().queue_with_canceller(
                    task!(process_new_entangled_sender: move || {
                        let global = context.lock().root();
                        global.maybe_add_message_port(&port_id);
                        if let Some(port) = global.upcast::<GlobalScope>().message_ports.borrow().find_port(&port_id) {
                            port.set_entangled_sender(ipc_sender);
                        };
                    }),
                    &self.canceller,
                );
            },
            MessagePortMsg::EntangledPortShipped(port_id) => {
                let context = self.context.clone();
                let _ = self.task_source.lock().queue_with_canceller(
                    task!(process_entangled_port_shipped: move || {
                        let global = context.lock().root();
                        global.maybe_add_message_port(&port_id);
                        if let Some(port) = global.upcast::<GlobalScope>().message_ports.borrow().find_port(&port_id) {
                            port.set_has_been_shipped();
                        };
                    }),
                    &self.canceller,
                );
            },
            MessagePortMsg::NewTask(port_id, task) => {
                let context = self.context.clone();
                let _ = self.task_source.lock().queue_with_canceller(
                    task!(process_new_task: move || {
                        let global = context.lock().root();
                        global.maybe_add_message_port(&port_id);
                        global.upcast::<GlobalScope>().route_task_to_port(port_id, task);
                    }),
                    &self.canceller,
                );
            },
            MessagePortMsg::RemoveMessagePort(port_id) => {
                let context = self.context.clone();
                let _ = self.task_source.lock().queue_with_canceller(
                    task!(process_remove_message_port: move || {
                        let global = context.lock().root();
                        global.maybe_add_message_port(&port_id);
                        if let Some(port) = global.upcast::<GlobalScope>().message_ports.borrow().find_port(&port_id) {
                            port.set_detached(true);
                        };
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
            message_ports: DomRefCell::new(MessagePorts::new()),
            pending_message_ports: DomRefCell::new(MessagePorts::new()),
            message_ports_to_be_collected: DomRefCell::new(vec![]),
            message_ports_shipped: DomRefCell::new(HashMap::new()),
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

    pub fn mark_port_as_shipped(
        &self,
        port_id: MessagePortId,
        entangled_port_id: Option<MessagePortId>,
    ) {
        self.message_ports_shipped
            .borrow_mut()
            .insert(port_id, entangled_port_id);
    }

    /// Route the task to be handled by the relevant port.
    /// Note that the task is run while message_ports is borrowed.
    /// Therefore, pending_message_ports should be used to track ports,
    /// that are either transfer-received or created in a task.
    pub fn route_task_to_port(&self, port_id: MessagePortId, task: PortMessageTask) {
        match self.message_ports_shipped.borrow_mut().remove(&port_id) {
            Some(Some(entangled)) => {
                if let Some(entangled_port) = self.message_ports.borrow().find_port(&entangled) {
                    if !entangled_port.has_been_shipped() {
                        // This is a case of a message having been sent immediately after transfer,
                        // before the entangled knew it was shipped.
                        // Make sure it knows, and re-route the message(via buffering).
                        entangled_port.set_has_been_shipped();
                        entangled_port.post_message(task);
                        return;
                    }
                }
            },
            _ => {},
        }
        if let Some(port) = self.message_ports.borrow().find_port(&port_id) {
            port.handle_incoming(task);
        }
    }

    /// Mark a message-port as eligible for garbage collection.
    pub fn start_garbage_collecting_message_port(&self, port_id: &MessagePortId) {
        self.message_ports_to_be_collected
            .borrow_mut()
            .push((port_id.clone(), 0));
    }

    /// After a port has been marked as up for collection,
    /// we wait two turns of the event-loop,
    /// to ensure any tasks left in the queue are handled.
    /// No new messages/tasks can be enqueued while this is ongoing(detached is set).
    ///
    /// https://html.spec.whatwg.org/multipage/#ports-and-garbage-collection
    pub fn perform_a_message_port_garbage_collection_checkpoint(&self) {
        // Add pending ports to message-ports.
        for (port_id, port) in self.pending_message_ports.borrow_mut().drain_ports() {
            self.message_ports.borrow_mut().insert(port_id, &port);
        }

        self.message_ports_to_be_collected
            .borrow_mut()
            .retain(|(port_id, mut counter)| {
                counter = counter + 1;
                if counter > 1 {
                    self.message_ports.borrow_mut().remove(port_id);
                    self.message_ports_shipped.borrow_mut().remove(port_id);
                    let _ = self
                        .script_to_constellation_chan()
                        .send(ScriptMsg::RemoveMessagePort(*port_id));
                    return false;
                }
                true
            });

        // Look for any ports that have been detached.
        for (id, port) in self.message_ports.borrow().iter() {
            if port.detached().is_some() {
                self.start_garbage_collecting_message_port(&id);
            }
        }
    }

    /// Before handling an incoming message, if necessary add port to message-ports.
    pub fn maybe_add_message_port(&self, port_id: &MessagePortId) {
        if let Some(port) = self.pending_message_ports.borrow_mut().remove(&port_id) {
            self.message_ports
                .borrow_mut()
                .insert(port_id.clone(), &port);
        };
    }

    /// Start tracking a message-port:
    ///
    /// 1. Add port to pending_message_ports(avoiding a double-borrow on message_ports).
    /// 2. Send an ipc-sender to the constellation,
    ///    which will be used both to communicate the ipc-sender of the entangled port,
    ///    when/if it ships as well,
    ///    or to let this scope know when the entangled port has been dropped,
    ///    The sender will also act as a way for the entangled port to send messages directly to here.
    /// 3. Setup a route with a MessageListner to act as go-between for the ipc and the event-loop.
    pub fn track_message_port(&self, port: &DomRoot<MessagePort>) {
        let message_port_id = port.message_port_id().clone();
        let (port_control_sender, port_control_receiver) =
            ipc::channel().expect("ipc channel failure");
        let _ = self
            .script_to_constellation_chan()
            .send(ScriptMsg::NewMessagePort(
                message_port_id,
                port_control_sender,
            ));
        self.pending_message_ports
            .borrow_mut()
            .insert(message_port_id, port);
        let context = Arc::new(Mutex::new(Trusted::new(self)));
        let (task_source, canceller) = (
            Arc::new(Mutex::new(self.port_message_queue())),
            self.task_canceller(TaskSourceName::PortMessage),
        );
        let listener = Arc::new(Mutex::new(MessageListener {
            canceller,
            task_source,
            context,
        }));
        ROUTER.add_route(
            port_control_receiver.to_opaque(),
            Box::new(move |message| {
                let msg = message.to();
                match msg {
                    Ok(msg) => listener.lock().notify(msg),
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
    pub fn get_cx(&self) -> *mut JSContext {
        Runtime::get()
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
        unreachable!()
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

                let _ac = JSAutoRealm::new(cx, globalhandle.get());
                let _aes = AutoEntryScript::new(self);
                let options = CompileOptionsWrapper::new(cx, filename.as_ptr(), line_number);

                debug!("evaluating Dom string");
                let result = unsafe {
                    EvaluateUtf8(
                        cx,
                        options.ptr,
                        code.as_ptr() as *const _,
                        code.len() as libc::size_t,
                        rval,
                    )
                };

                if !result {
                    debug!("error evaluating Dom string");
                    unsafe { report_pending_exception(cx, true) };
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
                self.get_cx(),
                |_| Some(DomRoot::from_ref(self)),
                vec![DomRoot::from_ref(self)],
            );
        }
    }

    /// Enqueue a microtask for subsequent execution.
    #[allow(unsafe_code)]
    pub fn enqueue_microtask(&self, job: Microtask) {
        unsafe {
            self.microtask_queue.enqueue(job, self.get_cx());
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
