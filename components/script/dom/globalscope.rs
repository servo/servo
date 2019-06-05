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
use crate::dom::bindings::root::{DomRoot, MutNullableDom};
use crate::dom::bindings::settings_stack::{entry_global, incumbent_global, AutoEntryScript};
use crate::dom::bindings::str::DOMString;
use crate::dom::bindings::structuredclone;
use crate::dom::bindings::weakref::{DOMTracker, WeakRef};
use crate::dom::blob::Blob;
use crate::dom::crypto::Crypto;
use crate::dom::dedicatedworkerglobalscope::DedicatedWorkerGlobalScope;
use crate::dom::errorevent::ErrorEvent;
use crate::dom::event::{Event, EventBubbles, EventCancelable, EventStatus};
use crate::dom::eventsource::EventSource;
use crate::dom::eventtarget::EventTarget;
use crate::dom::file::File;
use crate::dom::htmlscriptelement::ScriptId;
use crate::dom::messageevent::MessageEvent;
use crate::dom::messageport::MessagePort;
use crate::dom::paintworkletglobalscope::PaintWorkletGlobalScope;
use crate::dom::performance::Performance;
use crate::dom::window::Window;
use crate::dom::workerglobalscope::WorkerGlobalScope;
use crate::dom::workletglobalscope::WorkletGlobalScope;
use crate::microtask::{Microtask, MicrotaskQueue};
use crate::script_module::ModuleTree;
use crate::script_runtime::{CommonScriptMsg, JSContext as SafeJSContext, ScriptChan, ScriptPort};
use crate::script_thread::{MainThreadScriptChan, ScriptThread};
use crate::task::TaskCanceller;
use crate::task_source::dom_manipulation::DOMManipulationTaskSource;
use crate::task_source::file_reading::FileReadingTaskSource;
use crate::task_source::networking::NetworkingTaskSource;
use crate::task_source::performance_timeline::PerformanceTimelineTaskSource;
use crate::task_source::port_message::PortMessageQueue;
use crate::task_source::remote_event::RemoteEventTaskSource;
use crate::task_source::timer::TimerTaskSource;
use crate::task_source::websocket::WebsocketTaskSource;
use crate::task_source::TaskSource;
use crate::task_source::TaskSourceName;
use crate::timers::{IsInterval, OneshotTimerCallback, OneshotTimerHandle};
use crate::timers::{OneshotTimers, TimerCallback};
use content_security_policy::CspList;
use devtools_traits::{PageError, ScriptToDevtoolsControlMsg};
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
use msg::constellation_msg::{BlobId, MessagePortId, MessagePortRouterId, PipelineId};
use net_traits::blob_url_store::{get_blob_origin, BlobBuf};
use net_traits::filemanager_thread::{FileManagerThreadMsg, ReadFileProgress, RelativePos};
use net_traits::image_cache::ImageCache;
use net_traits::{CoreResourceMsg, CoreResourceThread, IpcSend, ResourceThreads};
use profile_traits::{ipc as profile_ipc, mem as profile_mem, time as profile_time};
use script_traits::serializable::{BlobData, BlobImpl, FileBlob};
use script_traits::transferable::MessagePortImpl;
use script_traits::{
    MessagePortMsg, MsDuration, PortMessageTask, ScriptMsg, ScriptToConstellationChan, TimerEvent,
};
use script_traits::{TimerEventId, TimerSchedulerMsg, TimerSource};
use servo_url::{MutableOrigin, ServoUrl};
use std::borrow::Cow;
use std::cell::Cell;
use std::collections::hash_map::Entry;
use std::collections::{HashMap, VecDeque};
use std::ffi::CString;
use std::mem;
use std::ops::Index;
use std::rc::Rc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use time::{get_time, Timespec};
use uuid::Uuid;

#[derive(JSTraceable)]
pub struct AutoCloseWorker(Arc<AtomicBool>);

impl Drop for AutoCloseWorker {
    fn drop(&mut self) {
        self.0.store(true, Ordering::SeqCst);
    }
}

#[dom_struct]
pub struct GlobalScope {
    eventtarget: EventTarget,
    crypto: MutNullableDom<Crypto>,

    /// The message-port router id for this global, if it is managing ports.
    message_port_state: DomRefCell<MessagePortState>,

    /// The blobs managed by this global, if any.
    blob_state: DomRefCell<BlobState>,

    /// Pipeline id associated with this global.
    pipeline_id: PipelineId,

    /// A flag to indicate whether the developer tools has requested
    /// live updates from the worker.
    devtools_wants_updates: Cell<bool>,

    /// Timers used by the Console API.
    console_timers: DomRefCell<HashMap<DOMString, u64>>,

    /// module map is used when importing JavaScript modules
    /// https://html.spec.whatwg.org/multipage/#concept-settings-object-module-map
    #[ignore_malloc_size_of = "mozjs"]
    module_map: DomRefCell<HashMap<ServoUrl, Rc<ModuleTree>>>,

    #[ignore_malloc_size_of = "mozjs"]
    inline_module_map: DomRefCell<HashMap<ScriptId, Rc<ModuleTree>>>,

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

    /// The mechanism by which time-outs and intervals are scheduled.
    /// <https://html.spec.whatwg.org/multipage/#timers>
    timers: OneshotTimers,

    /// Have timers been initialized?
    init_timers: Cell<bool>,

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

/// A wrapper between timer events coming in over IPC, and the event-loop.
struct TimerListener {
    canceller: TaskCanceller,
    task_source: TimerTaskSource,
    context: Trusted<GlobalScope>,
}

#[derive(JSTraceable, MallocSizeOf)]
/// A holder of a weak reference for a DOM blob or file.
pub enum BlobTracker {
    /// A weak ref to a DOM file.
    File(WeakRef<File>),
    /// A weak ref to a DOM blob.
    Blob(WeakRef<Blob>),
}

#[derive(JSTraceable, MallocSizeOf)]
/// The info pertaining to a blob managed by this global.
pub struct BlobInfo {
    /// The weak ref to the corresponding DOM object.
    tracker: BlobTracker,
    /// The data and logic backing the DOM object.
    blob_impl: BlobImpl,
    /// Whether this blob has an outstanding URL,
    /// <https://w3c.github.io/FileAPI/#url>.
    has_url: bool,
}

/// State representing whether this global is currently managing blobs.
#[derive(JSTraceable, MallocSizeOf)]
pub enum BlobState {
    /// A map of managed blobs.
    Managed(HashMap<BlobId, BlobInfo>),
    /// This global is not managing any blobs at this time.
    UnManaged,
}

/// Data representing a message-port managed by this global.
#[derive(JSTraceable, MallocSizeOf)]
pub enum ManagedMessagePort {
    /// We keep ports pending when they are first transfer-received,
    /// and only add them, and ask the constellation to complete the transfer,
    /// in a subsequent task if the port hasn't been re-transfered.
    Pending(MessagePortImpl, WeakRef<MessagePort>),
    /// A port who was transferred into, or initially created in, this realm,
    /// and that hasn't been re-transferred in the same task it was noted.
    Added(MessagePortImpl, WeakRef<MessagePort>),
}

/// State representing whether this global is currently managing messageports.
#[derive(JSTraceable, MallocSizeOf)]
pub enum MessagePortState {
    /// The message-port router id for this global, and a map of managed ports.
    Managed(
        MessagePortRouterId,
        HashMap<MessagePortId, ManagedMessagePort>,
    ),
    /// This global is not managing any ports at this time.
    UnManaged,
}

impl TimerListener {
    /// Handle a timer-event coming-in over IPC,
    /// by queuing the appropriate task on the relevant event-loop.
    fn handle(&self, event: TimerEvent) {
        let context = self.context.clone();
        // Step 18, queue a task,
        // https://html.spec.whatwg.org/multipage/#timer-initialisation-steps
        let _ = self.task_source.queue_with_canceller(
            task!(timer_event: move || {
                let global = context.root();
                let TimerEvent(source, id) = event;
                match source {
                    TimerSource::FromWorker => {
                        global.downcast::<WorkerGlobalScope>().expect("Window timer delivered to worker");
                    },
                    TimerSource::FromWindow(pipeline) => {
                        assert_eq!(pipeline, global.pipeline_id());
                        global.downcast::<Window>().expect("Worker timer delivered to window");
                    },
                };
                // Step 7, substeps run in a task.
                global.fire_timer(id);
            }),
            &self.canceller,
        );
    }
}

impl MessageListener {
    /// A new message came in, handle it via a task enqueued on the event-loop.
    /// A task is required, since we are using a trusted globalscope,
    /// and we can only access the root from the event-loop.
    fn notify(&self, msg: MessagePortMsg) {
        match msg {
            MessagePortMsg::CompleteTransfer(ports) => {
                let context = self.context.clone();
                let _ = self.task_source.queue_with_canceller(
                    task!(process_complete_transfer: move || {
                        let global = context.root();

                        let router_id = match global.port_router_id() {
                            Some(router_id) => router_id,
                            None => {
                                // If not managing any ports, no transfer can succeed,
                                // so just send back everything.
                                let _ = global.script_to_constellation_chan().send(
                                    ScriptMsg::MessagePortTransferResult(None, vec![], ports),
                                );
                                return;
                            }
                        };

                        let mut succeeded = vec![];
                        let mut failed = HashMap::new();

                        for (id, buffer) in ports.into_iter() {
                            if global.is_managing_port(&id) {
                                succeeded.push(id.clone());
                                global.complete_port_transfer(id, buffer);
                            } else {
                                failed.insert(id, buffer);
                            }
                        }
                        let _ = global.script_to_constellation_chan().send(
                            ScriptMsg::MessagePortTransferResult(Some(router_id), succeeded, failed),
                        );
                    }),
                    &self.canceller,
                );
            },
            MessagePortMsg::CompletePendingTransfer(port_id, buffer) => {
                let context = self.context.clone();
                let _ = self.task_source.queue_with_canceller(
                    task!(complete_pending: move || {
                        let global = context.root();
                        global.complete_port_transfer(port_id, buffer);
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
                        global.remove_message_port(&port_id);
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
        origin: MutableOrigin,
        microtask_queue: Rc<MicrotaskQueue>,
        is_headless: bool,
        user_agent: Cow<'static, str>,
    ) -> Self {
        Self {
            message_port_state: DomRefCell::new(MessagePortState::UnManaged),
            blob_state: DomRefCell::new(BlobState::UnManaged),
            eventtarget: EventTarget::new_inherited(),
            crypto: Default::default(),
            pipeline_id,
            devtools_wants_updates: Default::default(),
            console_timers: DomRefCell::new(Default::default()),
            module_map: DomRefCell::new(Default::default()),
            inline_module_map: DomRefCell::new(Default::default()),
            devtools_chan,
            mem_profiler_chan,
            time_profiler_chan,
            script_to_constellation_chan,
            scheduler_chan: scheduler_chan.clone(),
            in_error_reporting_mode: Default::default(),
            resource_threads,
            timers: OneshotTimers::new(scheduler_chan),
            init_timers: Default::default(),
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

    /// The message-port router Id of the global, if any
    fn port_router_id(&self) -> Option<MessagePortRouterId> {
        if let MessagePortState::Managed(id, _message_ports) = &*self.message_port_state.borrow() {
            Some(id.clone())
        } else {
            None
        }
    }

    /// Is this global managing a given port?
    fn is_managing_port(&self, port_id: &MessagePortId) -> bool {
        if let MessagePortState::Managed(_router_id, message_ports) =
            &*self.message_port_state.borrow()
        {
            return message_ports.contains_key(port_id);
        }
        false
    }

    /// Setup the IPC-to-event-loop glue for timers to schedule themselves.
    fn setup_timers(&self) {
        if self.init_timers.get() {
            return;
        }
        self.init_timers.set(true);

        let (timer_ipc_chan, timer_ipc_port) = ipc::channel().unwrap();
        self.timers.setup_scheduling(timer_ipc_chan);

        // Setup route from IPC to task-queue for the timer-task-source.
        let context = Trusted::new(&*self);
        let (task_source, canceller) = (
            self.timer_task_source(),
            self.task_canceller(TaskSourceName::Timer),
        );
        let timer_listener = TimerListener {
            context,
            task_source,
            canceller,
        };
        ROUTER.add_route(
            timer_ipc_port.to_opaque(),
            Box::new(move |message| {
                let event = message.to().unwrap();
                timer_listener.handle(event);
            }),
        );
    }

    /// Complete the transfer of a message-port.
    fn complete_port_transfer(&self, port_id: MessagePortId, tasks: VecDeque<PortMessageTask>) {
        let should_start = if let MessagePortState::Managed(_id, message_ports) =
            &mut *self.message_port_state.borrow_mut()
        {
            match message_ports.get_mut(&port_id) {
                None => {
                    panic!("complete_port_transfer called for an unknown port.");
                },
                Some(ManagedMessagePort::Pending(_, _)) => {
                    panic!("CompleteTransfer msg received for a pending port.");
                },
                Some(ManagedMessagePort::Added(port_impl, _port)) => {
                    port_impl.complete_transfer(tasks);
                    port_impl.enabled()
                },
            }
        } else {
            panic!("complete_port_transfer called for an unknown port.");
        };
        if should_start {
            self.start_message_port(&port_id);
        }
    }

    /// Clean-up DOM related resources
    pub fn perform_a_dom_garbage_collection_checkpoint(&self) {
        self.perform_a_message_port_garbage_collection_checkpoint();
        self.perform_a_blob_garbage_collection_checkpoint();
    }

    /// Update our state to un-managed,
    /// and tell the constellation to drop the sender to our message-port router.
    pub fn remove_message_ports_router(&self) {
        if let MessagePortState::Managed(router_id, _message_ports) =
            &*self.message_port_state.borrow()
        {
            let _ = self
                .script_to_constellation_chan()
                .send(ScriptMsg::RemoveMessagePortRouter(router_id.clone()));
        }
        *self.message_port_state.borrow_mut() = MessagePortState::UnManaged;
    }

    /// <https://html.spec.whatwg.org/multipage/#entangle>
    pub fn entangle_ports(&self, port1: MessagePortId, port2: MessagePortId) {
        if let MessagePortState::Managed(_id, message_ports) =
            &mut *self.message_port_state.borrow_mut()
        {
            for (port_id, entangled_id) in &[(port1, port2), (port2, port1)] {
                match message_ports.get_mut(&port_id) {
                    None => {
                        return warn!("entangled_ports called on a global not managing the port.");
                    },
                    Some(ManagedMessagePort::Pending(port_impl, dom_port)) => {
                        dom_port
                            .root()
                            .expect("Port to be entangled to not have been GC'ed")
                            .entangle(entangled_id.clone());
                        port_impl.entangle(entangled_id.clone());
                    },
                    Some(ManagedMessagePort::Added(port_impl, dom_port)) => {
                        dom_port
                            .root()
                            .expect("Port to be entangled to not have been GC'ed")
                            .entangle(entangled_id.clone());
                        port_impl.entangle(entangled_id.clone());
                    },
                }
            }
        } else {
            panic!("entangled_ports called on a global not managing any ports.");
        }

        let _ = self
            .script_to_constellation_chan()
            .send(ScriptMsg::EntanglePorts(port1, port2));
    }

    /// Remove all referrences to a port.
    pub fn remove_message_port(&self, port_id: &MessagePortId) {
        let is_empty = if let MessagePortState::Managed(_id, message_ports) =
            &mut *self.message_port_state.borrow_mut()
        {
            match message_ports.remove(&port_id) {
                None => panic!("remove_message_port called on a global not managing the port."),
                Some(_) => message_ports.is_empty(),
            }
        } else {
            return warn!("remove_message_port called on a global not managing any ports.");
        };
        if is_empty {
            // Remove our port router,
            // it will be setup again if we start managing ports again.
            self.remove_message_ports_router();
        }
    }

    /// Handle the transfer of a port in the current task.
    pub fn mark_port_as_transferred(&self, port_id: &MessagePortId) -> MessagePortImpl {
        if let MessagePortState::Managed(_id, message_ports) =
            &mut *self.message_port_state.borrow_mut()
        {
            let mut port = match message_ports.remove(&port_id) {
                None => {
                    panic!("mark_port_as_transferred called on a global not managing the port.")
                },
                Some(ManagedMessagePort::Pending(port_impl, _)) => port_impl,
                Some(ManagedMessagePort::Added(port_impl, _)) => port_impl,
            };
            port.set_has_been_shipped();
            let _ = self
                .script_to_constellation_chan()
                .send(ScriptMsg::MessagePortShipped(port_id.clone()));
            port
        } else {
            panic!("mark_port_as_transferred called on a global not managing any ports.");
        }
    }

    /// <https://html.spec.whatwg.org/multipage/#dom-messageport-start>
    pub fn start_message_port(&self, port_id: &MessagePortId) {
        if let MessagePortState::Managed(_id, message_ports) =
            &mut *self.message_port_state.borrow_mut()
        {
            let port = match message_ports.get_mut(&port_id) {
                None => panic!("start_message_port called on a unknown port."),
                Some(ManagedMessagePort::Pending(port_impl, _)) => port_impl,
                Some(ManagedMessagePort::Added(port_impl, _)) => port_impl,
            };
            if let Some(message_buffer) = port.start() {
                for task in message_buffer {
                    let port_id = port_id.clone();
                    let this = Trusted::new(&*self);
                    let _ = self.port_message_queue().queue(
                        task!(process_pending_port_messages: move || {
                            let target_global = this.root();
                            target_global.route_task_to_port(port_id, task);
                        }),
                        &self,
                    );
                }
            }
        } else {
            return warn!("start_message_port called on a global not managing any ports.");
        }
    }

    /// <https://html.spec.whatwg.org/multipage/#dom-messageport-close>
    pub fn close_message_port(&self, port_id: &MessagePortId) {
        if let MessagePortState::Managed(_id, message_ports) =
            &mut *self.message_port_state.borrow_mut()
        {
            let port = match message_ports.get_mut(&port_id) {
                None => panic!("close_message_port called on an unknown port."),
                Some(ManagedMessagePort::Pending(port_impl, _)) => port_impl,
                Some(ManagedMessagePort::Added(port_impl, _)) => port_impl,
            };
            port.close();
        } else {
            return warn!("close_message_port called on a global not managing any ports.");
        }
    }

    /// <https://html.spec.whatwg.org/multipage/#message-port-post-message-steps>
    // Steps 6 and 7
    pub fn post_messageport_msg(&self, port_id: MessagePortId, task: PortMessageTask) {
        if let MessagePortState::Managed(_id, message_ports) =
            &mut *self.message_port_state.borrow_mut()
        {
            let port = match message_ports.get_mut(&port_id) {
                None => panic!("post_messageport_msg called on an unknown port."),
                Some(ManagedMessagePort::Pending(port_impl, _)) => port_impl,
                Some(ManagedMessagePort::Added(port_impl, _)) => port_impl,
            };
            if let Some(entangled_id) = port.entangled_port_id() {
                // Step 7
                let this = Trusted::new(&*self);
                let _ = self.port_message_queue().queue(
                    task!(post_message: move || {
                        let global = this.root();
                        // Note: we do this in a task, as this will ensure the global and constellation
                        // are aware of any transfer that might still take place in the current task.
                        global.route_task_to_port(entangled_id, task);
                    }),
                    self,
                );
            }
        } else {
            return warn!("post_messageport_msg called on a global not managing any ports.");
        }
    }

    /// If we don't know about the port,
    /// send the message to the constellation for routing.
    fn re_route_port_task(&self, port_id: MessagePortId, task: PortMessageTask) {
        let _ = self
            .script_to_constellation_chan()
            .send(ScriptMsg::RerouteMessagePort(port_id, task));
    }

    /// Route the task to be handled by the relevant port.
    pub fn route_task_to_port(&self, port_id: MessagePortId, task: PortMessageTask) {
        let should_dispatch = if let MessagePortState::Managed(_id, message_ports) =
            &mut *self.message_port_state.borrow_mut()
        {
            if !message_ports.contains_key(&port_id) {
                self.re_route_port_task(port_id, task);
                return;
            }
            let (port_impl, dom_port) = match message_ports.get_mut(&port_id) {
                None => panic!("route_task_to_port called for an unknown port."),
                Some(ManagedMessagePort::Pending(port_impl, dom_port)) => (port_impl, dom_port),
                Some(ManagedMessagePort::Added(port_impl, dom_port)) => (port_impl, dom_port),
            };

            // If the port is not enabled yet, or if is awaiting the completion of it's transfer,
            // the task will be buffered and dispatched upon enablement or completion of the transfer.
            if let Some(task_to_dispatch) = port_impl.handle_incoming(task) {
                // Get a corresponding DOM message-port object.
                let dom_port = match dom_port.root() {
                    Some(dom_port) => dom_port,
                    None => panic!("Messageport Gc'ed too early"),
                };
                Some((dom_port, task_to_dispatch))
            } else {
                None
            }
        } else {
            self.re_route_port_task(port_id, task);
            return;
        };
        if let Some((dom_port, PortMessageTask { origin, data })) = should_dispatch {
            // Substep 3-4
            rooted!(in(*self.get_cx()) let mut message_clone = UndefinedValue());
            if let Ok(ports) = structuredclone::read(self, data, message_clone.handle_mut()) {
                // Substep 6
                // Dispatch the event, using the dom message-port.
                MessageEvent::dispatch_jsval(
                    &dom_port.upcast(),
                    self,
                    message_clone.handle(),
                    Some(&origin.ascii_serialization()),
                    None,
                    ports,
                );
            } else {
                // Step 4, fire messageerror event.
                MessageEvent::dispatch_error(&dom_port.upcast(), self);
            }
        }
    }

    /// Check all ports that have been transfer-received in the previous task,
    /// and complete their transfer if they haven't been re-transferred.
    pub fn maybe_add_pending_ports(&self) {
        if let MessagePortState::Managed(router_id, message_ports) =
            &mut *self.message_port_state.borrow_mut()
        {
            let to_be_added: Vec<MessagePortId> = message_ports
                .iter()
                .filter_map(|(id, port_info)| match port_info {
                    ManagedMessagePort::Pending(_, _) => Some(id.clone()),
                    _ => None,
                })
                .collect();
            for id in to_be_added.iter() {
                let (id, port_info) = message_ports
                    .remove_entry(&id)
                    .expect("Collected port-id to match an entry");
                match port_info {
                    ManagedMessagePort::Pending(port_impl, dom_port) => {
                        let new_port_info = ManagedMessagePort::Added(port_impl, dom_port);
                        let present = message_ports.insert(id, new_port_info);
                        assert!(present.is_none());
                    },
                    _ => panic!("Only pending ports should be found in to_be_added"),
                }
            }
            let _ =
                self.script_to_constellation_chan()
                    .send(ScriptMsg::CompleteMessagePortTransfer(
                        router_id.clone(),
                        to_be_added,
                    ));
        } else {
            warn!("maybe_add_pending_ports called on a global not managing any ports.");
        }
    }

    /// https://html.spec.whatwg.org/multipage/#ports-and-garbage-collection
    pub fn perform_a_message_port_garbage_collection_checkpoint(&self) {
        let is_empty = if let MessagePortState::Managed(_id, message_ports) =
            &mut *self.message_port_state.borrow_mut()
        {
            let to_be_removed: Vec<MessagePortId> = message_ports
                .iter()
                .filter_map(|(id, port_info)| {
                    if let ManagedMessagePort::Added(_port_impl, dom_port) = port_info {
                        if dom_port.root().is_none() {
                            // Let the constellation know to drop this port and the one it is entangled with,
                            // and to forward this message to the script-process where the entangled is found.
                            let _ = self
                                .script_to_constellation_chan()
                                .send(ScriptMsg::RemoveMessagePort(id.clone()));
                            return Some(id.clone());
                        }
                    }
                    None
                })
                .collect();
            for id in to_be_removed {
                message_ports.remove(&id);
            }
            message_ports.is_empty()
        } else {
            false
        };
        if is_empty {
            self.remove_message_ports_router();
        }
    }

    /// Start tracking a message-port
    pub fn track_message_port(&self, dom_port: &MessagePort, port_impl: Option<MessagePortImpl>) {
        let mut current_state = self.message_port_state.borrow_mut();

        if let MessagePortState::UnManaged = &*current_state {
            // Setup a route for IPC, for messages from the constellation to our ports.
            let (port_control_sender, port_control_receiver) =
                ipc::channel().expect("ipc channel failure");
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
            let router_id = MessagePortRouterId::new();
            *current_state = MessagePortState::Managed(router_id.clone(), HashMap::new());
            let _ = self
                .script_to_constellation_chan()
                .send(ScriptMsg::NewMessagePortRouter(
                    router_id,
                    port_control_sender,
                ));
        }

        if let MessagePortState::Managed(router_id, message_ports) = &mut *current_state {
            if let Some(port_impl) = port_impl {
                // We keep transfer-received ports as "pending",
                // and only ask the constellation to complete the transfer
                // if they're not re-shipped in the current task.
                message_ports.insert(
                    dom_port.message_port_id().clone(),
                    ManagedMessagePort::Pending(port_impl, WeakRef::new(dom_port)),
                );

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
                let port_impl = MessagePortImpl::new(dom_port.message_port_id().clone());
                message_ports.insert(
                    dom_port.message_port_id().clone(),
                    ManagedMessagePort::Added(port_impl, WeakRef::new(dom_port)),
                );
                let _ = self
                    .script_to_constellation_chan()
                    .send(ScriptMsg::NewMessagePort(
                        router_id.clone(),
                        dom_port.message_port_id().clone(),
                    ));
            };
        } else {
            panic!("track_message_port should have first switched the state to managed.");
        }
    }

    /// <https://html.spec.whatwg.org/multipage/#serialization-steps>
    /// defined at <https://w3c.github.io/FileAPI/#blob-section>.
    /// Get the snapshot state and underlying bytes of the blob.
    pub fn serialize_blob(&self, blob_id: &BlobId) -> BlobImpl {
        // Note: we combine the snapshot state and underlying bytes into one call,
        // which seems spec compliant.
        // See https://w3c.github.io/FileAPI/#snapshot-state
        let bytes = self
            .get_blob_bytes(blob_id)
            .expect("Could not read bytes from blob as part of serialization steps.");
        let type_string = self.get_blob_type_string(blob_id);

        // Note: the new BlobImpl is a clone, but with it's own BlobId.
        BlobImpl::new_from_bytes(bytes, type_string)
    }

    fn track_blob_info(&self, blob_info: BlobInfo, blob_id: BlobId) {
        let mut blob_state = self.blob_state.borrow_mut();

        match &mut *blob_state {
            BlobState::UnManaged => {
                let mut blobs_map = HashMap::new();
                blobs_map.insert(blob_id, blob_info);
                *blob_state = BlobState::Managed(blobs_map);
            },
            BlobState::Managed(blobs_map) => {
                blobs_map.insert(blob_id, blob_info);
            },
        }
    }

    /// Start tracking a blob
    pub fn track_blob(&self, dom_blob: &Blob, blob_impl: BlobImpl) {
        let blob_id = blob_impl.blob_id();

        let blob_info = BlobInfo {
            blob_impl,
            tracker: BlobTracker::Blob(WeakRef::new(dom_blob)),
            has_url: false,
        };

        self.track_blob_info(blob_info, blob_id);
    }

    /// Start tracking a file
    pub fn track_file(&self, file: &File, blob_impl: BlobImpl) {
        let blob_id = blob_impl.blob_id();

        let blob_info = BlobInfo {
            blob_impl,
            tracker: BlobTracker::File(WeakRef::new(file)),
            has_url: false,
        };

        self.track_blob_info(blob_info, blob_id);
    }

    /// Clean-up any file or blob that is unreachable from script,
    /// unless it has an oustanding blob url.
    /// <https://w3c.github.io/FileAPI/#lifeTime>
    fn perform_a_blob_garbage_collection_checkpoint(&self) {
        let mut blob_state = self.blob_state.borrow_mut();
        if let BlobState::Managed(blobs_map) = &mut *blob_state {
            blobs_map.retain(|_id, blob_info| {
                let garbage_collected = match &blob_info.tracker {
                    BlobTracker::File(weak) => weak.root().is_none(),
                    BlobTracker::Blob(weak) => weak.root().is_none(),
                };
                if garbage_collected && !blob_info.has_url {
                    if let BlobData::File(ref f) = blob_info.blob_impl.blob_data() {
                        self.decrement_file_ref(f.get_id());
                    }
                    false
                } else {
                    true
                }
            });
            if blobs_map.is_empty() {
                *blob_state = BlobState::UnManaged;
            }
        }
    }

    /// Clean-up all file related resources on document unload.
    /// <https://w3c.github.io/FileAPI/#lifeTime>
    pub fn clean_up_all_file_resources(&self) {
        let mut blob_state = self.blob_state.borrow_mut();
        if let BlobState::Managed(blobs_map) = &mut *blob_state {
            blobs_map.drain().for_each(|(_id, blob_info)| {
                if let BlobData::File(ref f) = blob_info.blob_impl.blob_data() {
                    self.decrement_file_ref(f.get_id());
                }
            });
        }
        *blob_state = BlobState::UnManaged;
    }

    fn decrement_file_ref(&self, id: Uuid) {
        let origin = get_blob_origin(&self.get_url());

        let (tx, rx) = profile_ipc::channel(self.time_profiler_chan().clone()).unwrap();

        let msg = FileManagerThreadMsg::DecRef(id, origin, tx);
        self.send_to_file_manager(msg);
        let _ = rx.recv();
    }

    /// Get a slice to the inner data of a Blob,
    /// In the case of a File-backed blob, this might incur synchronous read and caching.
    pub fn get_blob_bytes(&self, blob_id: &BlobId) -> Result<Vec<u8>, ()> {
        let parent = {
            let blob_state = self.blob_state.borrow();
            if let BlobState::Managed(blobs_map) = &*blob_state {
                let blob_info = blobs_map
                    .get(blob_id)
                    .expect("get_blob_bytes for an unknown blob.");
                match blob_info.blob_impl.blob_data() {
                    BlobData::Sliced(ref parent, ref rel_pos) => {
                        Some((parent.clone(), rel_pos.clone()))
                    },
                    _ => None,
                }
            } else {
                panic!("get_blob_bytes called on a global not managing any blobs.");
            }
        };

        match parent {
            Some((parent_id, rel_pos)) => self.get_blob_bytes_non_sliced(&parent_id).map(|v| {
                let range = rel_pos.to_abs_range(v.len());
                v.index(range).to_vec()
            }),
            None => self.get_blob_bytes_non_sliced(blob_id),
        }
    }

    /// Get bytes from a non-sliced blob
    fn get_blob_bytes_non_sliced(&self, blob_id: &BlobId) -> Result<Vec<u8>, ()> {
        let blob_state = self.blob_state.borrow();
        if let BlobState::Managed(blobs_map) = &*blob_state {
            let blob_info = blobs_map
                .get(blob_id)
                .expect("get_blob_bytes_non_sliced called for a unknown blob.");
            match blob_info.blob_impl.blob_data() {
                BlobData::File(ref f) => {
                    let (buffer, is_new_buffer) = match f.get_cache() {
                        Some(bytes) => (bytes, false),
                        None => {
                            let bytes = self.read_file(f.get_id())?;
                            (bytes, true)
                        },
                    };

                    // Cache
                    if is_new_buffer {
                        f.cache_bytes(buffer.clone());
                    }

                    Ok(buffer)
                },
                BlobData::Memory(ref s) => Ok(s.clone()),
                BlobData::Sliced(_, _) => panic!("This blob doesn't have a parent."),
            }
        } else {
            panic!("get_blob_bytes_non_sliced called on a global not managing any blobs.");
        }
    }

    /// Get a copy of the type_string of a blob.
    pub fn get_blob_type_string(&self, blob_id: &BlobId) -> String {
        let blob_state = self.blob_state.borrow();
        if let BlobState::Managed(blobs_map) = &*blob_state {
            let blob_info = blobs_map
                .get(blob_id)
                .expect("get_blob_type_string called for a unknown blob.");
            blob_info.blob_impl.type_string()
        } else {
            panic!("get_blob_type_string called on a global not managing any blobs.");
        }
    }

    /// https://w3c.github.io/FileAPI/#dfn-size
    pub fn get_blob_size(&self, blob_id: &BlobId) -> u64 {
        let blob_state = self.blob_state.borrow();
        if let BlobState::Managed(blobs_map) = &*blob_state {
            let parent = {
                let blob_info = blobs_map
                    .get(blob_id)
                    .expect("get_blob_size called for a unknown blob.");
                match blob_info.blob_impl.blob_data() {
                    BlobData::Sliced(ref parent, ref rel_pos) => {
                        Some((parent.clone(), rel_pos.clone()))
                    },
                    _ => None,
                }
            };
            match parent {
                Some((parent_id, rel_pos)) => {
                    let parent_info = blobs_map
                        .get(&parent_id)
                        .expect("Parent of blob whose size is unknown.");
                    let parent_size = match parent_info.blob_impl.blob_data() {
                        BlobData::File(ref f) => f.get_size(),
                        BlobData::Memory(ref v) => v.len() as u64,
                        BlobData::Sliced(_, _) => panic!("Blob ancestry should be only one level."),
                    };
                    rel_pos.to_abs_range(parent_size as usize).len() as u64
                },
                None => {
                    let blob_info = blobs_map.get(blob_id).expect("Blob whose size is unknown.");
                    match blob_info.blob_impl.blob_data() {
                        BlobData::File(ref f) => f.get_size(),
                        BlobData::Memory(ref v) => v.len() as u64,
                        BlobData::Sliced(_, _) => panic!(
                            "It was previously checked that this blob does not have a parent."
                        ),
                    }
                },
            }
        } else {
            panic!("get_blob_size called on a global not managing any blobs.");
        }
    }

    pub fn get_blob_url_id(&self, blob_id: &BlobId) -> Uuid {
        let mut blob_state = self.blob_state.borrow_mut();
        if let BlobState::Managed(blobs_map) = &mut *blob_state {
            let parent = {
                let blob_info = blobs_map
                    .get_mut(blob_id)
                    .expect("get_blob_url_id called for a unknown blob.");

                // Keep track of blobs with outstanding URLs.
                blob_info.has_url = true;

                match blob_info.blob_impl.blob_data() {
                    BlobData::Sliced(ref parent, ref rel_pos) => {
                        Some((parent.clone(), rel_pos.clone()))
                    },
                    _ => None,
                }
            };
            match parent {
                Some((parent_id, rel_pos)) => {
                    let parent_file_id = {
                        let parent_info = blobs_map
                            .get_mut(&parent_id)
                            .expect("Parent of blob whose url is requested is unknown.");
                        self.promote(parent_info, /* set_valid is */ false)
                    };
                    let parent_size = self.get_blob_size(&parent_id);
                    let blob_info = blobs_map
                        .get_mut(blob_id)
                        .expect("Blob whose url is requested is unknown.");
                    self.create_sliced_url_id(blob_info, &parent_file_id, &rel_pos, parent_size)
                },
                None => {
                    let blob_info = blobs_map
                        .get_mut(blob_id)
                        .expect("Blob whose url is requested is unknown.");
                    self.promote(blob_info, /* set_valid is */ true)
                },
            }
        } else {
            panic!("get_blob_url_id called on a global not managing any blobs.");
        }
    }

    /// Get a FileID representing sliced parent-blob content
    fn create_sliced_url_id(
        &self,
        blob_info: &mut BlobInfo,
        parent_file_id: &Uuid,
        rel_pos: &RelativePos,
        parent_len: u64,
    ) -> Uuid {
        let origin = get_blob_origin(&self.get_url());

        let (tx, rx) = profile_ipc::channel(self.time_profiler_chan().clone()).unwrap();
        let msg = FileManagerThreadMsg::AddSlicedURLEntry(
            parent_file_id.clone(),
            rel_pos.clone(),
            tx,
            origin.clone(),
        );
        self.send_to_file_manager(msg);
        match rx.recv().expect("File manager thread is down.") {
            Ok(new_id) => {
                *blob_info.blob_impl.blob_data_mut() = BlobData::File(FileBlob::new(
                    new_id.clone(),
                    None,
                    None,
                    rel_pos.to_abs_range(parent_len as usize).len() as u64,
                ));

                // Return the indirect id reference
                new_id
            },
            Err(_) => {
                // Return dummy id
                Uuid::new_v4()
            },
        }
    }

    /// Promote non-Slice blob:
    /// 1. Memory-based: The bytes in data slice will be transferred to file manager thread.
    /// 2. File-based: If set_valid, then activate the FileID so it can serve as URL
    /// Depending on set_valid, the returned FileID can be part of
    /// valid or invalid Blob URL.
    pub fn promote(&self, blob_info: &mut BlobInfo, set_valid: bool) -> Uuid {
        let mut bytes = vec![];
        let global_url = self.get_url();

        match blob_info.blob_impl.blob_data_mut() {
            BlobData::Sliced(_, _) => {
                panic!("Sliced blobs should use create_sliced_url_id instead of promote.");
            },
            BlobData::File(ref f) => {
                if set_valid {
                    let origin = get_blob_origin(&global_url);
                    let (tx, rx) = profile_ipc::channel(self.time_profiler_chan().clone()).unwrap();

                    let msg = FileManagerThreadMsg::ActivateBlobURL(f.get_id(), tx, origin.clone());
                    self.send_to_file_manager(msg);

                    match rx.recv().unwrap() {
                        Ok(_) => return f.get_id(),
                        // Return a dummy id on error
                        Err(_) => return Uuid::new_v4(),
                    }
                } else {
                    // no need to activate
                    return f.get_id();
                }
            },
            BlobData::Memory(ref mut bytes_in) => mem::swap(bytes_in, &mut bytes),
        };

        let origin = get_blob_origin(&global_url);

        let blob_buf = BlobBuf {
            filename: None,
            type_string: blob_info.blob_impl.type_string(),
            size: bytes.len() as u64,
            bytes: bytes.to_vec(),
        };

        let id = Uuid::new_v4();
        let msg = FileManagerThreadMsg::PromoteMemory(id, blob_buf, set_valid, origin.clone());
        self.send_to_file_manager(msg);

        *blob_info.blob_impl.blob_data_mut() = BlobData::File(FileBlob::new(
            id.clone(),
            None,
            Some(bytes.to_vec()),
            bytes.len() as u64,
        ));

        id
    }

    fn send_to_file_manager(&self, msg: FileManagerThreadMsg) {
        let resource_threads = self.resource_threads();
        let _ = resource_threads.send(CoreResourceMsg::ToFileManager(msg));
    }

    fn read_file(&self, id: Uuid) -> Result<Vec<u8>, ()> {
        let resource_threads = self.resource_threads();
        let (chan, recv) =
            profile_ipc::channel(self.time_profiler_chan().clone()).map_err(|_| ())?;
        let origin = get_blob_origin(&self.get_url());
        let check_url_validity = false;
        let msg = FileManagerThreadMsg::ReadFile(chan, id, check_url_validity, origin);
        let _ = resource_threads.send(CoreResourceMsg::ToFileManager(msg));

        let mut bytes = vec![];

        loop {
            match recv.recv().unwrap() {
                Ok(ReadFileProgress::Meta(mut blob_buf)) => {
                    bytes.append(&mut blob_buf.bytes);
                },
                Ok(ReadFileProgress::Partial(mut bytes_in)) => {
                    bytes.append(&mut bytes_in);
                },
                Ok(ReadFileProgress::EOF) => {
                    return Ok(bytes);
                },
                Err(_) => return Err(()),
            }
        }
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

    pub fn set_module_map(&self, url: ServoUrl, module: ModuleTree) {
        self.module_map.borrow_mut().insert(url, Rc::new(module));
    }

    pub fn get_module_map(&self) -> &DomRefCell<HashMap<ServoUrl, Rc<ModuleTree>>> {
        &self.module_map
    }

    pub fn set_inline_module_map(&self, script_id: ScriptId, module: ModuleTree) {
        self.inline_module_map
            .borrow_mut()
            .insert(script_id, Rc::new(module));
    }

    pub fn get_inline_module_map(&self) -> &DomRefCell<HashMap<ScriptId, Rc<ModuleTree>>> {
        &self.inline_module_map
    }

    #[allow(unsafe_code)]
    pub fn get_cx(&self) -> SafeJSContext {
        unsafe { SafeJSContext::from_ptr(Runtime::get()) }
    }

    pub fn crypto(&self) -> DomRoot<Crypto> {
        self.crypto.or_init(|| Crypto::new(self))
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

    pub fn issue_page_warning(&self, warning: &str) {
        if let Some(ref chan) = self.devtools_chan {
            let _ = chan.send(ScriptToDevtoolsControlMsg::ReportPageError(
                self.pipeline_id.clone(),
                PageError {
                    type_: "PageError".to_string(),
                    errorMessage: warning.to_string(),
                    sourceName: self.get_url().to_string(),
                    lineText: "".to_string(),
                    lineNumber: 0,
                    columnNumber: 0,
                    category: "script".to_string(),
                    timeStamp: 0, //TODO
                    error: false,
                    warning: true,
                    exception: true,
                    strict: false,
                    private: false,
                },
            ));
        }
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
            } else if self.is::<Window>() {
                if let Some(ref chan) = self.devtools_chan {
                    let _ = chan.send(ScriptToDevtoolsControlMsg::ReportPageError(
                        self.pipeline_id.clone(),
                        PageError {
                            type_: "PageError".to_string(),
                            errorMessage: error_info.message.clone(),
                            sourceName: error_info.filename.clone(),
                            lineText: "".to_string(), //TODO
                            lineNumber: error_info.lineno,
                            columnNumber: error_info.column,
                            category: "script".to_string(),
                            timeStamp: 0, //TODO
                            error: true,
                            warning: false,
                            exception: true,
                            strict: false,
                            private: false,
                        },
                    ));
                }
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

    /// `TaskSource` to send messages to the timer queue of
    /// this global scope.
    pub fn timer_task_source(&self) -> TimerTaskSource {
        if let Some(window) = self.downcast::<Window>() {
            return window.task_manager().timer_task_source();
        }
        if let Some(worker) = self.downcast::<WorkerGlobalScope>() {
            return worker.timer_task_source();
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

    /// <https://html.spec.whatwg.org/multipage/#timer-initialisation-steps>
    pub fn schedule_callback(
        &self,
        callback: OneshotTimerCallback,
        duration: MsDuration,
    ) -> OneshotTimerHandle {
        self.setup_timers();
        self.timers
            .schedule_callback(callback, duration, self.timer_source())
    }

    pub fn unschedule_callback(&self, handle: OneshotTimerHandle) {
        self.timers.unschedule_callback(handle);
    }

    /// <https://html.spec.whatwg.org/multipage/#timer-initialisation-steps>
    pub fn set_timeout_or_interval(
        &self,
        callback: TimerCallback,
        arguments: Vec<HandleValue>,
        timeout: i32,
        is_interval: IsInterval,
    ) -> i32 {
        self.setup_timers();
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
        self.timers.clear_timeout_or_interval(self, handle);
    }

    pub fn fire_timer(&self, handle: TimerEventId) {
        self.timers.fire_timer(handle, self);
    }

    pub fn resume(&self) {
        self.timers.resume();
    }

    pub fn suspend(&self) {
        self.timers.suspend();
    }

    pub fn slow_down_timers(&self) {
        self.timers.slow_down();
    }

    pub fn speed_up_timers(&self) {
        self.timers.speed_up();
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
    pub fn perform_a_microtask_checkpoint(&self) {
        self.microtask_queue.checkpoint(
            self.get_cx(),
            |_| Some(DomRoot::from_ref(self)),
            vec![DomRoot::from_ref(self)],
        );
    }

    /// Enqueue a microtask for subsequent execution.
    pub fn enqueue_microtask(&self, job: Microtask) {
        self.microtask_queue.enqueue(job, self.get_cx());
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

    /// https://www.w3.org/TR/CSP/#get-csp-of-object
    pub fn get_csp_list(&self) -> Option<CspList> {
        if let Some(window) = self.downcast::<Window>() {
            return window.Document().get_csp_list().map(|c| c.clone());
        }
        // TODO: Worker and Worklet global scopes.
        None
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
