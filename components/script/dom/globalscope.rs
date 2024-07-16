/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use std::borrow::Cow;
use std::cell::Cell;
use std::collections::hash_map::Entry;
use std::collections::{HashMap, VecDeque};
use std::ops::Index;
use std::rc::Rc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::thread::JoinHandle;
use std::time::{Instant, SystemTime, UNIX_EPOCH};
use std::{mem, ptr};

use base::id::{
    BlobId, BroadcastChannelRouterId, MessagePortId, MessagePortRouterId, PipelineId,
    ServiceWorkerId, ServiceWorkerRegistrationId,
};
use content_security_policy::CspList;
use crossbeam_channel::Sender;
use devtools_traits::{PageError, ScriptToDevtoolsControlMsg};
use dom_struct::dom_struct;
use embedder_traits::EmbedderMsg;
use ipc_channel::ipc::{self, IpcSender};
use ipc_channel::router::ROUTER;
use js::glue::{IsWrapper, UnwrapObjectDynamic};
use js::jsapi::{
    Compile1, CurrentGlobalOrNull, GetNonCCWObjectGlobal, HandleObject, Heap,
    InstantiateGlobalStencil, InstantiateOptions, JSContext, JSObject, JSScript, SetScriptPrivate,
};
use js::jsval::{JSVal, PrivateValue, UndefinedValue};
use js::panic::maybe_resume_unwind;
use js::rust::wrappers::{JS_ExecuteScript, JS_GetScriptPrivate};
use js::rust::{
    get_object_class, transform_str_to_source_text, CompileOptionsWrapper, HandleValue,
    MutableHandleValue, ParentRuntime, Runtime,
};
use js::{JSCLASS_IS_DOMJSCLASS, JSCLASS_IS_GLOBAL};
use net_traits::blob_url_store::{get_blob_origin, BlobBuf};
use net_traits::filemanager_thread::{
    FileManagerResult, FileManagerThreadMsg, ReadFileProgress, RelativePos,
};
use net_traits::image_cache::ImageCache;
use net_traits::request::Referrer;
use net_traits::response::HttpsState;
use net_traits::{CoreResourceMsg, CoreResourceThread, IpcSend, ResourceThreads};
use profile_traits::{ipc as profile_ipc, mem as profile_mem, time as profile_time};
use script_traits::serializable::{BlobData, BlobImpl, FileBlob};
use script_traits::transferable::MessagePortImpl;
use script_traits::{
    BroadcastMsg, GamepadEvent, GamepadSupportedHapticEffects, GamepadUpdateType, MessagePortMsg,
    MsDuration, PortMessageTask, ScriptMsg, ScriptToConstellationChan, TimerEvent, TimerEventId,
    TimerSchedulerMsg, TimerSource,
};
use servo_url::{ImmutableOrigin, MutableOrigin, ServoUrl};
use uuid::Uuid;
use webgpu::{DeviceLostReason, WebGPUDevice};

use super::bindings::codegen::Bindings::WebGPUBinding::GPUDeviceLostReason;
use super::bindings::trace::HashMapTracedValues;
use crate::dom::bindings::cell::{DomRefCell, RefMut};
use crate::dom::bindings::codegen::Bindings::BroadcastChannelBinding::BroadcastChannelMethods;
use crate::dom::bindings::codegen::Bindings::EventSourceBinding::EventSource_Binding::EventSourceMethods;
use crate::dom::bindings::codegen::Bindings::ImageBitmapBinding::{
    ImageBitmapOptions, ImageBitmapSource,
};
use crate::dom::bindings::codegen::Bindings::NavigatorBinding::NavigatorMethods;
use crate::dom::bindings::codegen::Bindings::PerformanceBinding::Performance_Binding::PerformanceMethods;
use crate::dom::bindings::codegen::Bindings::PermissionStatusBinding::PermissionState;
use crate::dom::bindings::codegen::Bindings::VoidFunctionBinding::VoidFunction;
use crate::dom::bindings::codegen::Bindings::WindowBinding::WindowMethods;
use crate::dom::bindings::codegen::Bindings::WorkerGlobalScopeBinding::WorkerGlobalScopeMethods;
use crate::dom::bindings::conversions::{root_from_object, root_from_object_static};
use crate::dom::bindings::error::{report_pending_exception, Error, ErrorInfo};
use crate::dom::bindings::inheritance::Castable;
use crate::dom::bindings::refcounted::{Trusted, TrustedPromise};
use crate::dom::bindings::reflector::DomObject;
use crate::dom::bindings::root::{Dom, DomRoot, MutNullableDom};
use crate::dom::bindings::settings_stack::{entry_global, incumbent_global, AutoEntryScript};
use crate::dom::bindings::str::DOMString;
use crate::dom::bindings::structuredclone;
use crate::dom::bindings::utils::to_frozen_array;
use crate::dom::bindings::weakref::{DOMTracker, WeakRef};
use crate::dom::blob::Blob;
use crate::dom::broadcastchannel::BroadcastChannel;
use crate::dom::crypto::Crypto;
use crate::dom::dedicatedworkerglobalscope::{
    DedicatedWorkerControlMsg, DedicatedWorkerGlobalScope,
};
use crate::dom::errorevent::ErrorEvent;
use crate::dom::event::{Event, EventBubbles, EventCancelable, EventStatus};
use crate::dom::eventsource::EventSource;
use crate::dom::eventtarget::EventTarget;
use crate::dom::file::File;
use crate::dom::gamepad::{contains_user_gesture, Gamepad};
use crate::dom::gamepadevent::GamepadEventType;
use crate::dom::gpudevice::GPUDevice;
use crate::dom::htmlscriptelement::{ScriptId, SourceCode};
use crate::dom::identityhub::Identities;
use crate::dom::imagebitmap::ImageBitmap;
use crate::dom::messageevent::MessageEvent;
use crate::dom::messageport::MessagePort;
use crate::dom::paintworkletglobalscope::PaintWorkletGlobalScope;
use crate::dom::performance::Performance;
use crate::dom::performanceobserver::VALID_ENTRY_TYPES;
use crate::dom::promise::Promise;
use crate::dom::readablestream::{ExternalUnderlyingSource, ReadableStream};
use crate::dom::serviceworker::ServiceWorker;
use crate::dom::serviceworkerregistration::ServiceWorkerRegistration;
use crate::dom::window::Window;
use crate::dom::workerglobalscope::WorkerGlobalScope;
use crate::dom::workletglobalscope::WorkletGlobalScope;
use crate::microtask::{Microtask, MicrotaskQueue, UserMicrotask};
use crate::realms::{enter_realm, AlreadyInRealm, InRealm};
use crate::script_module::{DynamicModuleList, ModuleScript, ModuleTree, ScriptFetchOptions};
use crate::script_runtime::{
    CommonScriptMsg, ContextForRequestInterrupt, JSContext as SafeJSContext, ScriptChan, ScriptPort,
};
use crate::script_thread::{MainThreadScriptChan, ScriptThread};
use crate::task::TaskCanceller;
use crate::task_source::dom_manipulation::DOMManipulationTaskSource;
use crate::task_source::file_reading::FileReadingTaskSource;
use crate::task_source::gamepad::GamepadTaskSource;
use crate::task_source::networking::NetworkingTaskSource;
use crate::task_source::performance_timeline::PerformanceTimelineTaskSource;
use crate::task_source::port_message::PortMessageQueue;
use crate::task_source::remote_event::RemoteEventTaskSource;
use crate::task_source::timer::TimerTaskSource;
use crate::task_source::websocket::WebsocketTaskSource;
use crate::task_source::{TaskSource, TaskSourceName};
use crate::timers::{
    IsInterval, OneshotTimerCallback, OneshotTimerHandle, OneshotTimers, TimerCallback,
};

#[derive(JSTraceable)]
pub struct AutoCloseWorker {
    /// <https://html.spec.whatwg.org/multipage/#dom-workerglobalscope-closing>
    closing: Arc<AtomicBool>,
    /// A handle to join on the worker thread.
    join_handle: Option<JoinHandle<()>>,
    /// A sender of control messages,
    /// currently only used to signal shutdown.
    #[no_trace]
    control_sender: Sender<DedicatedWorkerControlMsg>,
    /// The context to request an interrupt on the worker thread.
    context: ContextForRequestInterrupt,
}

impl Drop for AutoCloseWorker {
    /// <https://html.spec.whatwg.org/multipage/#terminate-a-worker>
    fn drop(&mut self) {
        // Step 1.
        self.closing.store(true, Ordering::SeqCst);

        if self
            .control_sender
            .send(DedicatedWorkerControlMsg::Exit)
            .is_err()
        {
            warn!("Couldn't send an exit message to a dedicated worker.");
        }

        self.context.request_interrupt();

        // TODO: step 2 and 3.
        // Step 4 is unnecessary since we don't use actual ports for dedicated workers.
        if self
            .join_handle
            .take()
            .expect("No handle to join on worker.")
            .join()
            .is_err()
        {
            warn!("Failed to join on dedicated worker thread.");
        }
    }
}

#[dom_struct]
pub struct GlobalScope {
    eventtarget: EventTarget,
    crypto: MutNullableDom<Crypto>,

    /// The message-port router id for this global, if it is managing ports.
    message_port_state: DomRefCell<MessagePortState>,

    /// The broadcast channels state this global, if it is managing any.
    broadcast_channel_state: DomRefCell<BroadcastChannelState>,

    /// The blobs managed by this global, if any.
    blob_state: DomRefCell<BlobState>,

    /// <https://w3c.github.io/ServiceWorker/#environment-settings-object-service-worker-registration-object-map>
    registration_map: DomRefCell<
        HashMapTracedValues<ServiceWorkerRegistrationId, Dom<ServiceWorkerRegistration>>,
    >,

    /// <https://w3c.github.io/ServiceWorker/#environment-settings-object-service-worker-object-map>
    worker_map: DomRefCell<HashMapTracedValues<ServiceWorkerId, Dom<ServiceWorker>>>,

    /// Pipeline id associated with this global.
    #[no_trace]
    pipeline_id: PipelineId,

    /// A flag to indicate whether the developer tools has requested
    /// live updates from the worker.
    devtools_wants_updates: Cell<bool>,

    /// Timers (milliseconds) used by the Console API.
    console_timers: DomRefCell<HashMap<DOMString, Instant>>,

    /// module map is used when importing JavaScript modules
    /// <https://html.spec.whatwg.org/multipage/#concept-settings-object-module-map>
    #[ignore_malloc_size_of = "mozjs"]
    module_map: DomRefCell<HashMapTracedValues<ServoUrl, Rc<ModuleTree>>>,

    #[ignore_malloc_size_of = "mozjs"]
    inline_module_map: DomRefCell<HashMap<ScriptId, Rc<ModuleTree>>>,

    /// For providing instructions to an optional devtools server.
    #[ignore_malloc_size_of = "channels are hard"]
    #[no_trace]
    devtools_chan: Option<IpcSender<ScriptToDevtoolsControlMsg>>,

    /// For sending messages to the memory profiler.
    #[ignore_malloc_size_of = "channels are hard"]
    #[no_trace]
    mem_profiler_chan: profile_mem::ProfilerChan,

    /// For sending messages to the time profiler.
    #[ignore_malloc_size_of = "channels are hard"]
    #[no_trace]
    time_profiler_chan: profile_time::ProfilerChan,

    /// A handle for communicating messages to the constellation thread.
    #[ignore_malloc_size_of = "channels are hard"]
    #[no_trace]
    script_to_constellation_chan: ScriptToConstellationChan,

    #[ignore_malloc_size_of = "channels are hard"]
    #[no_trace]
    scheduler_chan: IpcSender<TimerSchedulerMsg>,

    /// <https://html.spec.whatwg.org/multipage/#in-error-reporting-mode>
    in_error_reporting_mode: Cell<bool>,

    /// Associated resource threads for use by DOM objects like XMLHttpRequest,
    /// including resource_thread, filemanager_thread and storage_thread
    #[no_trace]
    resource_threads: ResourceThreads,

    /// The mechanism by which time-outs and intervals are scheduled.
    /// <https://html.spec.whatwg.org/multipage/#timers>
    timers: OneshotTimers,

    /// Have timers been initialized?
    init_timers: Cell<bool>,

    /// The origin of the globalscope
    #[no_trace]
    origin: MutableOrigin,

    /// <https://html.spec.whatwg.org/multipage/#concept-environment-creation-url>
    #[no_trace]
    creation_url: Option<ServoUrl>,

    /// A map for storing the previous permission state read results.
    permission_state_invocation_results: DomRefCell<HashMap<String, PermissionState>>,

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
    // `Heap` values must stay boxed, as they need semantics like `Pin`
    // (that is, they cannot be moved).
    #[allow(clippy::vec_box)]
    uncaught_rejections: DomRefCell<Vec<Box<Heap<*mut JSObject>>>>,

    /// Promises in this list have previously been reported as rejected
    /// (because they were in the above list), but the rejection was handled
    /// in the last turn of the event loop.
    ///
    /// <https://html.spec.whatwg.org/multipage/#outstanding-rejected-promises-weak-set>
    #[ignore_malloc_size_of = "mozjs"]
    // `Heap` values must stay boxed, as they need semantics like `Pin`
    // (that is, they cannot be moved).
    #[allow(clippy::vec_box)]
    consumed_rejections: DomRefCell<Vec<Box<Heap<*mut JSObject>>>>,

    /// True if headless mode.
    is_headless: bool,

    /// An optional string allowing the user agent to be set for testing.
    user_agent: Cow<'static, str>,

    /// Identity Manager for WebGPU resources
    #[ignore_malloc_size_of = "defined in wgpu"]
    #[no_trace]
    gpu_id_hub: Arc<Identities>,

    /// WebGPU devices
    gpu_devices: DomRefCell<HashMapTracedValues<WebGPUDevice, WeakRef<GPUDevice>>>,

    // https://w3c.github.io/performance-timeline/#supportedentrytypes-attribute
    #[ignore_malloc_size_of = "mozjs"]
    frozen_supported_performance_entry_types: DomRefCell<Option<Heap<JSVal>>>,

    /// currect https state (from previous request)
    #[no_trace]
    https_state: Cell<HttpsState>,

    /// The stack of active group labels for the Console APIs.
    console_group_stack: DomRefCell<Vec<DOMString>>,

    /// The count map for the Console APIs.
    ///
    /// <https://console.spec.whatwg.org/#count>
    console_count_map: DomRefCell<HashMap<DOMString, usize>>,

    /// List of ongoing dynamic module imports.
    dynamic_modules: DomRefCell<DynamicModuleList>,

    /// Is considered in a secure context
    inherited_secure_context: Option<bool>,
}

/// A wrapper for glue-code between the ipc router and the event-loop.
struct MessageListener {
    canceller: TaskCanceller,
    task_source: PortMessageQueue,
    context: Trusted<GlobalScope>,
}

/// A wrapper for broadcasts coming in over IPC, and the event-loop.
struct BroadcastListener {
    canceller: TaskCanceller,
    task_source: DOMManipulationTaskSource,
    context: Trusted<GlobalScope>,
}

/// A wrapper between timer events coming in over IPC, and the event-loop.
struct TimerListener {
    canceller: TaskCanceller,
    task_source: TimerTaskSource,
    context: Trusted<GlobalScope>,
}

/// A wrapper for the handling of file data received by the ipc router
struct FileListener {
    /// State should progress as either of:
    /// - Some(Empty) => Some(Receiving) => None
    /// - Some(Empty) => None
    state: Option<FileListenerState>,
    task_source: FileReadingTaskSource,
    task_canceller: TaskCanceller,
}

enum FileListenerCallback {
    Promise(Box<dyn Fn(Rc<Promise>, Result<Vec<u8>, Error>) + Send>),
    Stream,
}

enum FileListenerTarget {
    Promise(TrustedPromise),
    Stream(Trusted<ReadableStream>),
}

enum FileListenerState {
    Empty(FileListenerCallback, FileListenerTarget),
    Receiving(Vec<u8>, FileListenerCallback, FileListenerTarget),
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
    #[no_trace]
    blob_impl: BlobImpl,
    /// Whether this blob has an outstanding URL,
    /// <https://w3c.github.io/FileAPI/#url>.
    has_url: bool,
}

/// State representing whether this global is currently managing blobs.
#[derive(JSTraceable, MallocSizeOf)]
pub enum BlobState {
    /// A map of managed blobs.
    Managed(HashMapTracedValues<BlobId, BlobInfo>),
    /// This global is not managing any blobs at this time.
    UnManaged,
}

/// The result of looking-up the data for a Blob,
/// containing either the in-memory bytes,
/// or the file-id.
enum BlobResult {
    Bytes(Vec<u8>),
    File(Uuid, usize),
}

/// Data representing a message-port managed by this global.
#[derive(JSTraceable, MallocSizeOf)]
#[crown::unrooted_must_root_lint::must_root]
pub struct ManagedMessagePort {
    /// The DOM port.
    dom_port: Dom<MessagePort>,
    /// The logic and data backing the DOM port.
    /// The option is needed to take out the port-impl
    /// as part of its transferring steps,
    /// without having to worry about rooting the dom-port.
    #[no_trace]
    port_impl: Option<MessagePortImpl>,
    /// We keep ports pending when they are first transfer-received,
    /// and only add them, and ask the constellation to complete the transfer,
    /// in a subsequent task if the port hasn't been re-transfered.
    pending: bool,
    /// Has the port been closed? If closed, it can be dropped and later GC'ed.
    closed: bool,
}

/// State representing whether this global is currently managing broadcast channels.
#[derive(JSTraceable, MallocSizeOf)]
#[crown::unrooted_must_root_lint::must_root]
pub enum BroadcastChannelState {
    /// The broadcast-channel router id for this global, and a queue of managed channels.
    /// Step 9, "sort destinations"
    /// of <https://html.spec.whatwg.org/multipage/#dom-broadcastchannel-postmessage>
    /// requires keeping track of creation order, hence the queue.
    Managed(
        #[no_trace] BroadcastChannelRouterId,
        /// The map of channel-name to queue of channels, in order of creation.
        HashMap<DOMString, VecDeque<Dom<BroadcastChannel>>>,
    ),
    /// This global is not managing any broadcast channels at this time.
    UnManaged,
}

/// State representing whether this global is currently managing messageports.
#[derive(JSTraceable, MallocSizeOf)]
#[crown::unrooted_must_root_lint::must_root]
pub enum MessagePortState {
    /// The message-port router id for this global, and a map of managed ports.
    Managed(
        #[no_trace] MessagePortRouterId,
        HashMapTracedValues<MessagePortId, ManagedMessagePort>,
    ),
    /// This global is not managing any ports at this time.
    UnManaged,
}

impl BroadcastListener {
    /// Handle a broadcast coming in over IPC,
    /// by queueing the appropriate task on the relevant event-loop.
    fn handle(&self, event: BroadcastMsg) {
        let context = self.context.clone();

        // Note: strictly speaking we should just queue the message event tasks,
        // not queue a task that then queues more tasks.
        // This however seems to be hard to avoid in the light of the IPC.
        // One can imagine queueing tasks directly,
        // for channels that would be in the same script-thread.
        let _ = self.task_source.queue_with_canceller(
            task!(broadcast_message_event: move || {
                let global = context.root();
                // Step 10 of https://html.spec.whatwg.org/multipage/#dom-broadcastchannel-postmessage,
                // For each BroadcastChannel object destination in destinations, queue a task.
                global.broadcast_message_event(event, None);
            }),
            &self.canceller,
        );
    }
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
                                succeeded.push(id);
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
                        global.note_entangled_port_removed(&port_id);
                    }),
                    &self.canceller,
                );
            },
        }
    }
}

/// Callback used to enqueue file chunks to streams as part of FileListener.
fn stream_handle_incoming(stream: &ReadableStream, bytes: Result<Vec<u8>, Error>) {
    match bytes {
        Ok(b) => {
            stream.enqueue_native(b);
        },
        Err(e) => {
            stream.error_native(e);
        },
    }
}

/// Callback used to close streams as part of FileListener.
fn stream_handle_eof(stream: &ReadableStream) {
    stream.close_native();
}

impl FileListener {
    fn handle(&mut self, msg: FileManagerResult<ReadFileProgress>) {
        match msg {
            Ok(ReadFileProgress::Meta(blob_buf)) => match self.state.take() {
                Some(FileListenerState::Empty(callback, target)) => {
                    let bytes = if let FileListenerTarget::Stream(ref trusted_stream) = target {
                        let trusted = trusted_stream.clone();

                        let task = task!(enqueue_stream_chunk: move || {
                            let stream = trusted.root();
                            stream_handle_incoming(&stream, Ok(blob_buf.bytes));
                        });

                        let _ = self
                            .task_source
                            .queue_with_canceller(task, &self.task_canceller);
                        Vec::with_capacity(0)
                    } else {
                        blob_buf.bytes
                    };

                    self.state = Some(FileListenerState::Receiving(bytes, callback, target));
                },
                _ => panic!(
                    "Unexpected FileListenerState when receiving ReadFileProgress::Meta msg."
                ),
            },
            Ok(ReadFileProgress::Partial(mut bytes_in)) => match self.state.take() {
                Some(FileListenerState::Receiving(mut bytes, callback, target)) => {
                    if let FileListenerTarget::Stream(ref trusted_stream) = target {
                        let trusted = trusted_stream.clone();

                        let task = task!(enqueue_stream_chunk: move || {
                            let stream = trusted.root();
                            stream_handle_incoming(&stream, Ok(bytes_in));
                        });

                        let _ = self
                            .task_source
                            .queue_with_canceller(task, &self.task_canceller);
                    } else {
                        bytes.append(&mut bytes_in);
                    };

                    self.state = Some(FileListenerState::Receiving(bytes, callback, target));
                },
                _ => panic!(
                    "Unexpected FileListenerState when receiving ReadFileProgress::Partial msg."
                ),
            },
            Ok(ReadFileProgress::EOF) => match self.state.take() {
                Some(FileListenerState::Receiving(bytes, callback, target)) => match target {
                    FileListenerTarget::Promise(trusted_promise) => {
                        let callback = match callback {
                            FileListenerCallback::Promise(callback) => callback,
                            _ => panic!("Expected promise callback."),
                        };
                        let task = task!(resolve_promise: move || {
                            let promise = trusted_promise.root();
                            let _ac = enter_realm(&*promise.global());
                            callback(promise, Ok(bytes));
                        });

                        let _ = self
                            .task_source
                            .queue_with_canceller(task, &self.task_canceller);
                    },
                    FileListenerTarget::Stream(trusted_stream) => {
                        let trusted = trusted_stream.clone();

                        let task = task!(enqueue_stream_chunk: move || {
                            let stream = trusted.root();
                            stream_handle_eof(&stream);
                        });

                        let _ = self
                            .task_source
                            .queue_with_canceller(task, &self.task_canceller);
                    },
                },
                _ => {
                    panic!("Unexpected FileListenerState when receiving ReadFileProgress::EOF msg.")
                },
            },
            Err(_) => match self.state.take() {
                Some(FileListenerState::Receiving(_, callback, target)) |
                Some(FileListenerState::Empty(callback, target)) => {
                    let error = Err(Error::Network);

                    match target {
                        FileListenerTarget::Promise(trusted_promise) => {
                            let callback = match callback {
                                FileListenerCallback::Promise(callback) => callback,
                                _ => panic!("Expected promise callback."),
                            };
                            let _ = self.task_source.queue_with_canceller(
                                task!(reject_promise: move || {
                                    let promise = trusted_promise.root();
                                    let _ac = enter_realm(&*promise.global());
                                    callback(promise, error);
                                }),
                                &self.task_canceller,
                            );
                        },
                        FileListenerTarget::Stream(trusted_stream) => {
                            let _ = self.task_source.queue_with_canceller(
                                task!(error_stream: move || {
                                    let stream = trusted_stream.root();
                                    stream_handle_incoming(&stream, error);
                                }),
                                &self.task_canceller,
                            );
                        },
                    }
                },
                _ => panic!("Unexpected FileListenerState when receiving Err msg."),
            },
        }
    }
}

impl GlobalScope {
    #[allow(clippy::too_many_arguments)]
    pub fn new_inherited(
        pipeline_id: PipelineId,
        devtools_chan: Option<IpcSender<ScriptToDevtoolsControlMsg>>,
        mem_profiler_chan: profile_mem::ProfilerChan,
        time_profiler_chan: profile_time::ProfilerChan,
        script_to_constellation_chan: ScriptToConstellationChan,
        scheduler_chan: IpcSender<TimerSchedulerMsg>,
        resource_threads: ResourceThreads,
        origin: MutableOrigin,
        creation_url: Option<ServoUrl>,
        microtask_queue: Rc<MicrotaskQueue>,
        is_headless: bool,
        user_agent: Cow<'static, str>,
        gpu_id_hub: Arc<Identities>,
        inherited_secure_context: Option<bool>,
    ) -> Self {
        Self {
            message_port_state: DomRefCell::new(MessagePortState::UnManaged),
            broadcast_channel_state: DomRefCell::new(BroadcastChannelState::UnManaged),
            blob_state: DomRefCell::new(BlobState::UnManaged),
            eventtarget: EventTarget::new_inherited(),
            crypto: Default::default(),
            registration_map: DomRefCell::new(HashMapTracedValues::new()),
            worker_map: DomRefCell::new(HashMapTracedValues::new()),
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
            creation_url,
            permission_state_invocation_results: Default::default(),
            microtask_queue,
            list_auto_close_worker: Default::default(),
            event_source_tracker: DOMTracker::new(),
            uncaught_rejections: Default::default(),
            consumed_rejections: Default::default(),
            is_headless,
            user_agent,
            gpu_id_hub,
            gpu_devices: DomRefCell::new(HashMapTracedValues::new()),
            frozen_supported_performance_entry_types: DomRefCell::new(Default::default()),
            https_state: Cell::new(HttpsState::None),
            console_group_stack: DomRefCell::new(Vec::new()),
            console_count_map: Default::default(),
            dynamic_modules: DomRefCell::new(DynamicModuleList::new()),
            inherited_secure_context,
        }
    }

    /// The message-port router Id of the global, if any
    fn port_router_id(&self) -> Option<MessagePortRouterId> {
        if let MessagePortState::Managed(id, _message_ports) = &*self.message_port_state.borrow() {
            Some(*id)
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
        let context = Trusted::new(self);
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

    /// <https://w3c.github.io/ServiceWorker/#get-the-service-worker-registration-object>
    pub fn get_serviceworker_registration(
        &self,
        script_url: &ServoUrl,
        scope: &ServoUrl,
        registration_id: ServiceWorkerRegistrationId,
        installing_worker: Option<ServiceWorkerId>,
        _waiting_worker: Option<ServiceWorkerId>,
        _active_worker: Option<ServiceWorkerId>,
    ) -> DomRoot<ServiceWorkerRegistration> {
        // Step 1
        let mut registrations = self.registration_map.borrow_mut();

        if let Some(registration) = registrations.get(&registration_id) {
            // Step 3
            return DomRoot::from_ref(&**registration);
        }

        // Step 2.1 -> 2.5
        let new_registration = ServiceWorkerRegistration::new(self, scope.clone(), registration_id);

        // Step 2.6
        if let Some(worker_id) = installing_worker {
            let worker = self.get_serviceworker(script_url, scope, worker_id);
            new_registration.set_installing(&worker);
        }

        // TODO: 2.7 (waiting worker)

        // TODO: 2.8 (active worker)

        // Step 2.9
        registrations.insert(registration_id, Dom::from_ref(&*new_registration));

        // Step 3
        new_registration
    }

    /// <https://w3c.github.io/ServiceWorker/#get-the-service-worker-object>
    pub fn get_serviceworker(
        &self,
        script_url: &ServoUrl,
        scope: &ServoUrl,
        worker_id: ServiceWorkerId,
    ) -> DomRoot<ServiceWorker> {
        // Step 1
        let mut workers = self.worker_map.borrow_mut();

        if let Some(worker) = workers.get(&worker_id) {
            // Step 3
            DomRoot::from_ref(&**worker)
        } else {
            // Step 2.1
            // TODO: step 2.2, worker state.
            let new_worker = ServiceWorker::new(self, script_url.clone(), scope.clone(), worker_id);

            // Step 2.3
            workers.insert(worker_id, Dom::from_ref(&*new_worker));

            // Step 3
            new_worker
        }
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
                Some(managed_port) => {
                    if managed_port.pending {
                        panic!("CompleteTransfer msg received for a pending port.");
                    }
                    if let Some(port_impl) = managed_port.port_impl.as_mut() {
                        port_impl.complete_transfer(tasks);
                        port_impl.enabled()
                    } else {
                        panic!("managed-port has no port-impl.");
                    }
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
        self.perform_a_broadcast_channel_garbage_collection_checkpoint();
    }

    /// Remove the routers for ports and broadcast-channels.
    /// Drain the list of workers.
    pub fn remove_web_messaging_and_dedicated_workers_infra(&self) {
        self.remove_message_ports_router();
        self.remove_broadcast_channel_router();

        // Drop each ref to a worker explicitly now,
        // which will send a shutdown signal,
        // and join on the worker thread.
        self.list_auto_close_worker
            .borrow_mut()
            .drain(0..)
            .for_each(drop);
    }

    /// Update our state to un-managed,
    /// and tell the constellation to drop the sender to our message-port router.
    fn remove_message_ports_router(&self) {
        if let MessagePortState::Managed(router_id, _message_ports) =
            &*self.message_port_state.borrow()
        {
            let _ = self
                .script_to_constellation_chan()
                .send(ScriptMsg::RemoveMessagePortRouter(*router_id));
        }
        *self.message_port_state.borrow_mut() = MessagePortState::UnManaged;
    }

    /// Update our state to un-managed,
    /// and tell the constellation to drop the sender to our broadcast router.
    fn remove_broadcast_channel_router(&self) {
        if let BroadcastChannelState::Managed(router_id, _channels) =
            &*self.broadcast_channel_state.borrow()
        {
            let _ =
                self.script_to_constellation_chan()
                    .send(ScriptMsg::RemoveBroadcastChannelRouter(
                        *router_id,
                        self.origin().immutable().clone(),
                    ));
        }
        *self.broadcast_channel_state.borrow_mut() = BroadcastChannelState::UnManaged;
    }

    /// <https://html.spec.whatwg.org/multipage/#entangle>
    pub fn entangle_ports(&self, port1: MessagePortId, port2: MessagePortId) {
        if let MessagePortState::Managed(_id, message_ports) =
            &mut *self.message_port_state.borrow_mut()
        {
            for (port_id, entangled_id) in &[(port1, port2), (port2, port1)] {
                match message_ports.get_mut(port_id) {
                    None => {
                        return warn!("entangled_ports called on a global not managing the port.");
                    },
                    Some(managed_port) => {
                        if let Some(port_impl) = managed_port.port_impl.as_mut() {
                            managed_port.dom_port.entangle(*entangled_id);
                            port_impl.entangle(*entangled_id);
                        } else {
                            panic!("managed-port has no port-impl.");
                        }
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

    /// Note that the entangled port of `port_id` has been removed in another global.
    pub fn note_entangled_port_removed(&self, port_id: &MessagePortId) {
        // Note: currently this is a no-op,
        // as we only use the `close` method to manage the local lifecyle of a port.
        // This could be used as part of lifecyle management to determine a port can be GC'ed.
        // See https://github.com/servo/servo/issues/25772
        warn!(
            "Entangled port of {:?} has been removed in another global",
            port_id
        );
    }

    /// Handle the transfer of a port in the current task.
    pub fn mark_port_as_transferred(&self, port_id: &MessagePortId) -> MessagePortImpl {
        if let MessagePortState::Managed(_id, message_ports) =
            &mut *self.message_port_state.borrow_mut()
        {
            let mut port_impl = message_ports
                .remove(port_id)
                .map(|ref mut managed_port| {
                    managed_port
                        .port_impl
                        .take()
                        .expect("Managed port doesn't have a port-impl.")
                })
                .expect("mark_port_as_transferred called on a global not managing the port.");
            port_impl.set_has_been_shipped();
            let _ = self
                .script_to_constellation_chan()
                .send(ScriptMsg::MessagePortShipped(*port_id));
            port_impl
        } else {
            panic!("mark_port_as_transferred called on a global not managing any ports.");
        }
    }

    /// <https://html.spec.whatwg.org/multipage/#dom-messageport-start>
    pub fn start_message_port(&self, port_id: &MessagePortId) {
        if let MessagePortState::Managed(_id, message_ports) =
            &mut *self.message_port_state.borrow_mut()
        {
            let message_buffer = match message_ports.get_mut(port_id) {
                None => panic!("start_message_port called on a unknown port."),
                Some(managed_port) => {
                    if let Some(port_impl) = managed_port.port_impl.as_mut() {
                        port_impl.start()
                    } else {
                        panic!("managed-port has no port-impl.");
                    }
                },
            };
            if let Some(message_buffer) = message_buffer {
                for task in message_buffer {
                    let port_id = *port_id;
                    let this = Trusted::new(self);
                    let _ = self.port_message_queue().queue(
                        task!(process_pending_port_messages: move || {
                            let target_global = this.root();
                            target_global.route_task_to_port(port_id, task);
                        }),
                        self,
                    );
                }
            }
        } else {
            warn!("start_message_port called on a global not managing any ports.")
        }
    }

    /// <https://html.spec.whatwg.org/multipage/#dom-messageport-close>
    pub fn close_message_port(&self, port_id: &MessagePortId) {
        if let MessagePortState::Managed(_id, message_ports) =
            &mut *self.message_port_state.borrow_mut()
        {
            match message_ports.get_mut(port_id) {
                None => panic!("close_message_port called on an unknown port."),
                Some(managed_port) => {
                    if let Some(port_impl) = managed_port.port_impl.as_mut() {
                        port_impl.close();
                        managed_port.closed = true;
                    } else {
                        panic!("managed-port has no port-impl.");
                    }
                },
            };
        } else {
            warn!("close_message_port called on a global not managing any ports.")
        }
    }

    /// <https://html.spec.whatwg.org/multipage/#message-port-post-message-steps>
    // Steps 6 and 7
    pub fn post_messageport_msg(&self, port_id: MessagePortId, task: PortMessageTask) {
        if let MessagePortState::Managed(_id, message_ports) =
            &mut *self.message_port_state.borrow_mut()
        {
            let entangled_port = match message_ports.get_mut(&port_id) {
                None => panic!("post_messageport_msg called on an unknown port."),
                Some(managed_port) => {
                    if let Some(port_impl) = managed_port.port_impl.as_mut() {
                        port_impl.entangled_port_id()
                    } else {
                        panic!("managed-port has no port-impl.");
                    }
                },
            };
            if let Some(entangled_id) = entangled_port {
                // Step 7
                let this = Trusted::new(self);
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
            warn!("post_messageport_msg called on a global not managing any ports.");
        }
    }

    /// If we don't know about the port,
    /// send the message to the constellation for routing.
    fn re_route_port_task(&self, port_id: MessagePortId, task: PortMessageTask) {
        let _ = self
            .script_to_constellation_chan()
            .send(ScriptMsg::RerouteMessagePort(port_id, task));
    }

    /// <https://html.spec.whatwg.org/multipage/#dom-broadcastchannel-postmessage>
    /// Step 7 and following steps.
    pub fn schedule_broadcast(&self, msg: BroadcastMsg, channel_id: &Uuid) {
        // First, broadcast locally.
        self.broadcast_message_event(msg.clone(), Some(channel_id));

        if let BroadcastChannelState::Managed(router_id, _) =
            &*self.broadcast_channel_state.borrow()
        {
            // Second, broadcast to other globals via the constellation.
            //
            // Note: for globals in the same script-thread,
            // we could skip the hop to the constellation.
            let _ = self
                .script_to_constellation_chan()
                .send(ScriptMsg::ScheduleBroadcast(*router_id, msg));
        } else {
            panic!("Attemps to broadcast a message via global not managing any channels.");
        }
    }

    /// <https://html.spec.whatwg.org/multipage/#dom-broadcastchannel-postmessage>
    /// Step 7 and following steps.
    pub fn broadcast_message_event(&self, event: BroadcastMsg, channel_id: Option<&Uuid>) {
        if let BroadcastChannelState::Managed(_, channels) = &*self.broadcast_channel_state.borrow()
        {
            let BroadcastMsg {
                data,
                origin,
                channel_name,
            } = event;

            // Step 7, a few preliminary steps.

            // - Check the worker is not closing.
            if let Some(worker) = self.downcast::<WorkerGlobalScope>() {
                if worker.is_closing() {
                    return;
                }
            }

            // - Check the associated document is fully-active.
            if let Some(window) = self.downcast::<Window>() {
                if !window.Document().is_fully_active() {
                    return;
                }
            }

            // - Check for a case-sensitive match for the name of the channel.
            let channel_name = DOMString::from_string(channel_name);

            if let Some(channels) = channels.get(&channel_name) {
                channels
                    .iter()
                    .filter(|channel| {
                        // Step 8.
                        // Filter out the sender.
                        if let Some(id) = channel_id {
                            channel.id() != id
                        } else {
                            true
                        }
                    })
                    .map(|channel| DomRoot::from_ref(&**channel))
                    // Step 9, sort by creation order,
                    // done by using a queue to store channels in creation order.
                    .for_each(|channel| {
                        let data = data.clone_for_broadcast();
                        let origin = origin.clone();

                        // Step 10: Queue a task on the DOM manipulation task-source,
                        // to fire the message event
                        let channel = Trusted::new(&*channel);
                        let global = Trusted::new(self);
                        let _ = self.dom_manipulation_task_source().queue(
                            task!(process_pending_port_messages: move || {
                                let destination = channel.root();
                                let global = global.root();

                                // 10.1 Check for closed flag.
                                if destination.closed() {
                                    return;
                                }

                                rooted!(in(*GlobalScope::get_cx()) let mut message = UndefinedValue());

                                // Step 10.3 StructuredDeserialize(serialized, targetRealm).
                                if let Ok(ports) = structuredclone::read(&global, data, message.handle_mut()) {
                                    // Step 10.4, Fire an event named message at destination.
                                    MessageEvent::dispatch_jsval(
                                        destination.upcast(),
                                        &global,
                                        message.handle(),
                                        Some(&origin.ascii_serialization()),
                                        None,
                                        ports,
                                    );
                                } else {
                                    // Step 10.3, fire an event named messageerror at destination.
                                    MessageEvent::dispatch_error(destination.upcast(), &global);
                                }
                            }),
                            self,
                        );
                    });
            }
        }
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
            match message_ports.get_mut(&port_id) {
                None => panic!("route_task_to_port called for an unknown port."),
                Some(managed_port) => {
                    // If the port is not enabled yet, or if is awaiting the completion of it's transfer,
                    // the task will be buffered and dispatched upon enablement or completion of the transfer.
                    if let Some(port_impl) = managed_port.port_impl.as_mut() {
                        port_impl.handle_incoming(task).map(|to_dispatch| {
                            (DomRoot::from_ref(&*managed_port.dom_port), to_dispatch)
                        })
                    } else {
                        panic!("managed-port has no port-impl.");
                    }
                },
            }
        } else {
            self.re_route_port_task(port_id, task);
            return;
        };
        if let Some((dom_port, PortMessageTask { origin, data })) = should_dispatch {
            // Substep 3-4
            rooted!(in(*GlobalScope::get_cx()) let mut message_clone = UndefinedValue());
            if let Ok(ports) = structuredclone::read(self, data, message_clone.handle_mut()) {
                // Substep 6
                // Dispatch the event, using the dom message-port.
                MessageEvent::dispatch_jsval(
                    dom_port.upcast(),
                    self,
                    message_clone.handle(),
                    Some(&origin.ascii_serialization()),
                    None,
                    ports,
                );
            } else {
                // Step 4, fire messageerror event.
                MessageEvent::dispatch_error(dom_port.upcast(), self);
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
                .filter_map(|(id, managed_port)| {
                    if managed_port.pending {
                        Some(*id)
                    } else {
                        None
                    }
                })
                .collect();
            for id in to_be_added.iter() {
                let managed_port = message_ports
                    .get_mut(id)
                    .expect("Collected port-id to match an entry");
                if !managed_port.pending {
                    panic!("Only pending ports should be found in to_be_added")
                }
                managed_port.pending = false;
            }
            let _ =
                self.script_to_constellation_chan()
                    .send(ScriptMsg::CompleteMessagePortTransfer(
                        *router_id,
                        to_be_added,
                    ));
        } else {
            warn!("maybe_add_pending_ports called on a global not managing any ports.");
        }
    }

    /// <https://html.spec.whatwg.org/multipage/#ports-and-garbage-collection>
    pub fn perform_a_message_port_garbage_collection_checkpoint(&self) {
        let is_empty = if let MessagePortState::Managed(_id, message_ports) =
            &mut *self.message_port_state.borrow_mut()
        {
            let to_be_removed: Vec<MessagePortId> = message_ports
                .iter()
                .filter_map(|(id, managed_port)| {
                    if managed_port.closed {
                        // Let the constellation know to drop this port and the one it is entangled with,
                        // and to forward this message to the script-process where the entangled is found.
                        let _ = self
                            .script_to_constellation_chan()
                            .send(ScriptMsg::RemoveMessagePort(*id));
                        Some(*id)
                    } else {
                        None
                    }
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

    /// Remove broadcast-channels that are closed.
    /// TODO: Also remove them if they do not have an event-listener.
    /// see <https://github.com/servo/servo/issues/25772>
    pub fn perform_a_broadcast_channel_garbage_collection_checkpoint(&self) {
        let is_empty = if let BroadcastChannelState::Managed(router_id, ref mut channels) =
            &mut *self.broadcast_channel_state.borrow_mut()
        {
            channels.retain(|name, ref mut channels| {
                channels.retain(|chan| !chan.closed());
                if channels.is_empty() {
                    let _ = self.script_to_constellation_chan().send(
                        ScriptMsg::RemoveBroadcastChannelNameInRouter(
                            *router_id,
                            name.to_string(),
                            self.origin().immutable().clone(),
                        ),
                    );
                    false
                } else {
                    true
                }
            });
            channels.is_empty()
        } else {
            false
        };
        if is_empty {
            self.remove_broadcast_channel_router();
        }
    }

    /// Start tracking a broadcast-channel.
    pub fn track_broadcast_channel(&self, dom_channel: &BroadcastChannel) {
        let mut current_state = self.broadcast_channel_state.borrow_mut();

        if let BroadcastChannelState::UnManaged = &*current_state {
            // Setup a route for IPC, for broadcasts from the constellation to our channels.
            let (broadcast_control_sender, broadcast_control_receiver) =
                ipc::channel().expect("ipc channel failure");
            let context = Trusted::new(self);
            let (task_source, canceller) = (
                self.dom_manipulation_task_source(),
                self.task_canceller(TaskSourceName::DOMManipulation),
            );
            let listener = BroadcastListener {
                canceller,
                task_source,
                context,
            };
            ROUTER.add_route(
                broadcast_control_receiver.to_opaque(),
                Box::new(move |message| {
                    let msg = message.to();
                    match msg {
                        Ok(msg) => listener.handle(msg),
                        Err(err) => warn!("Error receiving a BroadcastMsg: {:?}", err),
                    }
                }),
            );
            let router_id = BroadcastChannelRouterId::new();
            *current_state = BroadcastChannelState::Managed(router_id, HashMap::new());
            let _ = self
                .script_to_constellation_chan()
                .send(ScriptMsg::NewBroadcastChannelRouter(
                    router_id,
                    broadcast_control_sender,
                    self.origin().immutable().clone(),
                ));
        }

        if let BroadcastChannelState::Managed(router_id, channels) = &mut *current_state {
            let entry = channels.entry(dom_channel.Name()).or_insert_with(|| {
                let _ = self.script_to_constellation_chan().send(
                    ScriptMsg::NewBroadcastChannelNameInRouter(
                        *router_id,
                        dom_channel.Name().to_string(),
                        self.origin().immutable().clone(),
                    ),
                );
                VecDeque::new()
            });
            entry.push_back(Dom::from_ref(dom_channel));
        } else {
            panic!("track_broadcast_channel should have first switched the state to managed.");
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
            *current_state = MessagePortState::Managed(router_id, HashMapTracedValues::new());
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
                    *dom_port.message_port_id(),
                    ManagedMessagePort {
                        port_impl: Some(port_impl),
                        dom_port: Dom::from_ref(dom_port),
                        pending: true,
                        closed: false,
                    },
                );

                // Queue a task to complete the transfer,
                // unless the port is re-transferred in the current task.
                let this = Trusted::new(self);
                let _ = self.port_message_queue().queue(
                    task!(process_pending_port_messages: move || {
                        let target_global = this.root();
                        target_global.maybe_add_pending_ports();
                    }),
                    self,
                );
            } else {
                // If this is a newly-created port, let the constellation immediately know.
                let port_impl = MessagePortImpl::new(*dom_port.message_port_id());
                message_ports.insert(
                    *dom_port.message_port_id(),
                    ManagedMessagePort {
                        port_impl: Some(port_impl),
                        dom_port: Dom::from_ref(dom_port),
                        pending: false,
                        closed: false,
                    },
                );
                let _ = self
                    .script_to_constellation_chan()
                    .send(ScriptMsg::NewMessagePort(
                        *router_id,
                        *dom_port.message_port_id(),
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
                let mut blobs_map = HashMapTracedValues::new();
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
            blobs_map.0.retain(|_id, blob_info| {
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
                    BlobData::Sliced(ref parent, ref rel_pos) => Some((*parent, rel_pos.clone())),
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

    /// Get a slice to the inner data of a Blob,
    /// if it's a memory blob, or it's file-id and file-size otherwise.
    ///
    /// Note: this is almost a duplicate of `get_blob_bytes`,
    /// tweaked for integration with streams.
    /// TODO: merge with `get_blob_bytes` by way of broader integration with blob streams.
    fn get_blob_bytes_or_file_id(&self, blob_id: &BlobId) -> BlobResult {
        let parent = {
            let blob_state = self.blob_state.borrow();
            if let BlobState::Managed(blobs_map) = &*blob_state {
                let blob_info = blobs_map
                    .get(blob_id)
                    .expect("get_blob_bytes_or_file_id for an unknown blob.");
                match blob_info.blob_impl.blob_data() {
                    BlobData::Sliced(ref parent, ref rel_pos) => Some((*parent, rel_pos.clone())),
                    _ => None,
                }
            } else {
                panic!("get_blob_bytes_or_file_id called on a global not managing any blobs.");
            }
        };

        match parent {
            Some((parent_id, rel_pos)) => {
                match self.get_blob_bytes_non_sliced_or_file_id(&parent_id) {
                    BlobResult::Bytes(bytes) => {
                        let range = rel_pos.to_abs_range(bytes.len());
                        BlobResult::Bytes(bytes.index(range).to_vec())
                    },
                    res => res,
                }
            },
            None => self.get_blob_bytes_non_sliced_or_file_id(blob_id),
        }
    }

    /// Get bytes from a non-sliced blob if in memory, or it's file-id and file-size.
    ///
    /// Note: this is almost a duplicate of `get_blob_bytes_non_sliced`,
    /// tweaked for integration with streams.
    /// TODO: merge with `get_blob_bytes` by way of broader integration with blob streams.
    fn get_blob_bytes_non_sliced_or_file_id(&self, blob_id: &BlobId) -> BlobResult {
        let blob_state = self.blob_state.borrow();
        if let BlobState::Managed(blobs_map) = &*blob_state {
            let blob_info = blobs_map
                .get(blob_id)
                .expect("get_blob_bytes_non_sliced_or_file_id called for a unknown blob.");
            match blob_info.blob_impl.blob_data() {
                BlobData::File(ref f) => match f.get_cache() {
                    Some(bytes) => BlobResult::Bytes(bytes.clone()),
                    None => BlobResult::File(f.get_id(), f.get_size() as usize),
                },
                BlobData::Memory(ref s) => BlobResult::Bytes(s.clone()),
                BlobData::Sliced(_, _) => panic!("This blob doesn't have a parent."),
            }
        } else {
            panic!(
                "get_blob_bytes_non_sliced_or_file_id called on a global not managing any blobs."
            );
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

    /// <https://w3c.github.io/FileAPI/#dfn-size>
    pub fn get_blob_size(&self, blob_id: &BlobId) -> u64 {
        let blob_state = self.blob_state.borrow();
        if let BlobState::Managed(blobs_map) = &*blob_state {
            let parent = {
                let blob_info = blobs_map
                    .get(blob_id)
                    .expect("get_blob_size called for a unknown blob.");
                match blob_info.blob_impl.blob_data() {
                    BlobData::Sliced(ref parent, ref rel_pos) => Some((*parent, rel_pos.clone())),
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
                    BlobData::Sliced(ref parent, ref rel_pos) => Some((*parent, rel_pos.clone())),
                    _ => None,
                }
            };
            match parent {
                Some((parent_id, rel_pos)) => {
                    let parent_info = blobs_map
                        .get_mut(&parent_id)
                        .expect("Parent of blob whose url is requested is unknown.");
                    let parent_file_id = self.promote(parent_info, /* set_valid is */ false);
                    let parent_size = match parent_info.blob_impl.blob_data() {
                        BlobData::File(ref f) => f.get_size(),
                        BlobData::Memory(ref v) => v.len() as u64,
                        BlobData::Sliced(_, _) => panic!("Blob ancestry should be only one level."),
                    };
                    let parent_size = rel_pos.to_abs_range(parent_size as usize).len() as u64;
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
            *parent_file_id,
            rel_pos.clone(),
            tx,
            origin.clone(),
        );
        self.send_to_file_manager(msg);
        match rx.recv().expect("File manager thread is down.") {
            Ok(new_id) => {
                *blob_info.blob_impl.blob_data_mut() = BlobData::File(FileBlob::new(
                    new_id,
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
            id,
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
        let recv = self.send_msg(id);
        GlobalScope::read_msg(recv)
    }

    /// <https://w3c.github.io/FileAPI/#blob-get-stream>
    pub fn get_blob_stream(&self, blob_id: &BlobId) -> DomRoot<ReadableStream> {
        let (file_id, size) = match self.get_blob_bytes_or_file_id(blob_id) {
            BlobResult::Bytes(bytes) => {
                // If we have all the bytes in memory, queue them and close the stream.
                let stream = ReadableStream::new_from_bytes(self, bytes);
                return stream;
            },
            BlobResult::File(id, size) => (id, size),
        };

        let stream = ReadableStream::new_with_external_underlying_source(
            self,
            ExternalUnderlyingSource::Blob(size),
        );

        let recv = self.send_msg(file_id);

        let trusted_stream = Trusted::new(&*stream.clone());
        let task_canceller = self.task_canceller(TaskSourceName::FileReading);
        let task_source = self.file_reading_task_source();

        let mut file_listener = FileListener {
            state: Some(FileListenerState::Empty(
                FileListenerCallback::Stream,
                FileListenerTarget::Stream(trusted_stream),
            )),
            task_source,
            task_canceller,
        };

        ROUTER.add_route(
            recv.to_opaque(),
            Box::new(move |msg| {
                file_listener.handle(
                    msg.to()
                        .expect("Deserialization of file listener msg failed."),
                );
            }),
        );

        stream
    }

    pub fn read_file_async(
        &self,
        id: Uuid,
        promise: Rc<Promise>,
        callback: Box<dyn Fn(Rc<Promise>, Result<Vec<u8>, Error>) + Send>,
    ) {
        let recv = self.send_msg(id);

        let trusted_promise = TrustedPromise::new(promise);
        let task_canceller = self.task_canceller(TaskSourceName::FileReading);
        let task_source = self.file_reading_task_source();

        let mut file_listener = FileListener {
            state: Some(FileListenerState::Empty(
                FileListenerCallback::Promise(callback),
                FileListenerTarget::Promise(trusted_promise),
            )),
            task_source,
            task_canceller,
        };

        ROUTER.add_route(
            recv.to_opaque(),
            Box::new(move |msg| {
                file_listener.handle(
                    msg.to()
                        .expect("Deserialization of file listener msg failed."),
                );
            }),
        );
    }

    fn send_msg(&self, id: Uuid) -> profile_ipc::IpcReceiver<FileManagerResult<ReadFileProgress>> {
        let resource_threads = self.resource_threads();
        let (chan, recv) = profile_ipc::channel(self.time_profiler_chan().clone()).unwrap();
        let origin = get_blob_origin(&self.get_url());
        let msg = FileManagerThreadMsg::ReadFile(chan, id, origin);
        let _ = resource_threads.send(CoreResourceMsg::ToFileManager(msg));
        recv
    }

    fn read_msg(
        receiver: profile_ipc::IpcReceiver<FileManagerResult<ReadFileProgress>>,
    ) -> Result<Vec<u8>, ()> {
        let mut bytes = vec![];

        loop {
            match receiver.recv().unwrap() {
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

    pub fn permission_state_invocation_results(
        &self,
    ) -> &DomRefCell<HashMap<String, PermissionState>> {
        &self.permission_state_invocation_results
    }

    pub fn track_worker(
        &self,
        closing: Arc<AtomicBool>,
        join_handle: JoinHandle<()>,
        control_sender: Sender<DedicatedWorkerControlMsg>,
        context: ContextForRequestInterrupt,
    ) {
        self.list_auto_close_worker
            .borrow_mut()
            .push(AutoCloseWorker {
                closing,
                join_handle: Some(join_handle),
                control_sender,
                context,
            });
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
    pub fn from_reflector<T: DomObject>(reflector: &T, _realm: &AlreadyInRealm) -> DomRoot<Self> {
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
    pub unsafe fn from_context(cx: *mut JSContext, _realm: InRealm) -> DomRoot<Self> {
        let global = CurrentGlobalOrNull(cx);
        assert!(!global.is_null());
        global_scope_from_global(global, cx)
    }

    /// Returns the global scope for the given SafeJSContext
    #[allow(unsafe_code)]
    pub fn from_safe_context(cx: SafeJSContext, realm: InRealm) -> DomRoot<Self> {
        unsafe { Self::from_context(*cx, realm) }
    }

    /// Returns the global object of the realm that the given JS object
    /// was created in, after unwrapping any wrappers.
    #[allow(unsafe_code)]
    pub unsafe fn from_object_maybe_wrapped(
        mut obj: *mut JSObject,
        cx: *mut JSContext,
    ) -> DomRoot<Self> {
        if IsWrapper(obj) {
            obj = UnwrapObjectDynamic(obj, cx, /* stopAtWindowProxy = */ false);
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

    // `Heap` values must stay boxed, as they need semantics like `Pin`
    // (that is, they cannot be moved).
    #[allow(clippy::vec_box)]
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

    // `Heap` values must stay boxed, as they need semantics like `Pin`
    // (that is, they cannot be moved).
    #[allow(clippy::vec_box)]
    pub fn get_consumed_rejections(&self) -> &DomRefCell<Vec<Box<Heap<*mut JSObject>>>> {
        &self.consumed_rejections
    }

    pub fn set_module_map(&self, url: ServoUrl, module: ModuleTree) {
        self.module_map.borrow_mut().insert(url, Rc::new(module));
    }

    pub fn get_module_map(&self) -> &DomRefCell<HashMapTracedValues<ServoUrl, Rc<ModuleTree>>> {
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
    pub fn get_cx() -> SafeJSContext {
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
                entry.insert(Instant::now());
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
            .map(|start| (Instant::now() - start).as_millis() as u64)
    }

    /// Get an `&IpcSender<ScriptToDevtoolsControlMsg>` to send messages
    /// to the devtools thread when available.
    pub fn devtools_chan(&self) -> Option<&IpcSender<ScriptToDevtoolsControlMsg>> {
        self.devtools_chan.as_ref()
    }

    pub fn issue_page_warning(&self, warning: &str) {
        if let Some(ref chan) = self.devtools_chan {
            let _ = chan.send(ScriptToDevtoolsControlMsg::ReportPageError(
                self.pipeline_id,
                PageError {
                    type_: "PageError".to_string(),
                    error_message: warning.to_string(),
                    source_name: self.get_url().to_string(),
                    line_text: "".to_string(),
                    line_number: 0,
                    column_number: 0,
                    category: "script".to_string(),
                    time_stamp: SystemTime::now()
                        .duration_since(UNIX_EPOCH)
                        .unwrap_or_default()
                        .as_millis() as u64,
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

    pub fn send_to_embedder(&self, msg: EmbedderMsg) {
        self.send_to_constellation(ScriptMsg::ForwardToEmbedder(msg));
    }

    pub fn send_to_constellation(&self, msg: ScriptMsg) {
        self.script_to_constellation_chan().send(msg).unwrap();
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

    /// Get the creation_url for this global scope
    pub fn creation_url(&self) -> &Option<ServoUrl> {
        &self.creation_url
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

    /// Determine the Referrer for a request whose Referrer is "client"
    pub fn get_referrer(&self) -> Referrer {
        // Step 3 of https://w3c.github.io/webappsec-referrer-policy/#determine-requests-referrer
        if let Some(window) = self.downcast::<Window>() {
            // Substep 3.1

            // Substep 3.1.1
            let mut document = window.Document();

            // Substep 3.1.2
            if let ImmutableOrigin::Opaque(_) = document.origin().immutable() {
                return Referrer::NoReferrer;
            }

            let mut url = document.url();

            // Substep 3.1.3
            while url.as_str() == "about:srcdoc" {
                document = document
                    .browsing_context()
                    .expect("iframe should have browsing context")
                    .parent()
                    .expect("iframes browsing_context should have parent")
                    .document()
                    .expect("iframes parent should have document");

                url = document.url();
            }

            // Substep 3.1.4
            Referrer::Client(url)
        } else {
            // Substep 3.2
            Referrer::Client(self.get_url())
        }
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
                        self.pipeline_id,
                        PageError {
                            type_: "PageError".to_string(),
                            error_message: error_info.message.clone(),
                            source_name: error_info.filename.clone(),
                            line_text: "".to_string(), //TODO
                            line_number: error_info.lineno,
                            column_number: error_info.column,
                            category: "script".to_string(),
                            time_stamp: SystemTime::now()
                                .duration_since(UNIX_EPOCH)
                                .unwrap_or_default()
                                .as_millis() as u64,
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

    /// `TaskSource` to send messages to the gamepad task source of
    /// this global scope.
    /// <https://w3c.github.io/gamepad/#dfn-gamepad-task-source>
    pub fn gamepad_task_source(&self) -> GamepadTaskSource {
        if let Some(window) = self.downcast::<Window>() {
            return window.task_manager().gamepad_task_source();
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
    pub fn evaluate_js_on_global_with_result(
        &self,
        code: &str,
        rval: MutableHandleValue,
        fetch_options: ScriptFetchOptions,
        script_base_url: ServoUrl,
    ) -> bool {
        let source_code = SourceCode::Text(Rc::new(DOMString::from_string((*code).to_string())));
        self.evaluate_script_on_global_with_result(
            &source_code,
            "",
            rval,
            1,
            fetch_options,
            script_base_url,
        )
    }

    /// Evaluate a JS script on this global scope.
    #[allow(unsafe_code)]
    pub fn evaluate_script_on_global_with_result(
        &self,
        code: &SourceCode,
        filename: &str,
        rval: MutableHandleValue,
        line_number: u32,
        fetch_options: ScriptFetchOptions,
        script_base_url: ServoUrl,
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
                let cx = GlobalScope::get_cx();

                let ar = enter_realm(self);

                let _aes = AutoEntryScript::new(self);

                unsafe {
                    rooted!(in(*cx) let mut compiled_script = std::ptr::null_mut::<JSScript>());
                    match code {
                        SourceCode::Text(text_code) => {
                            let options = CompileOptionsWrapper::new(*cx, filename, line_number);

                            debug!("compiling dom string");
                            compiled_script.set(Compile1(
                                *cx,
                                options.ptr,
                                &mut transform_str_to_source_text(text_code),
                            ));

                            if compiled_script.is_null() {
                                debug!("error compiling Dom string");
                                report_pending_exception(*cx, true, InRealm::Entered(&ar));
                                return false;
                            }
                        },
                        SourceCode::Compiled(pre_compiled_script) => {
                            let options = InstantiateOptions {
                                skipFilenameValidation: false,
                                hideScriptFromDebugger: false,
                                deferDebugMetadata: false,
                            };
                            let script = InstantiateGlobalStencil(
                                *cx,
                                &options,
                                *pre_compiled_script.source_code,
                                ptr::null_mut(),
                            );
                            compiled_script.set(script);
                        },
                    };

                    assert!(!compiled_script.is_null());

                    rooted!(in(*cx) let mut script_private = UndefinedValue());
                    JS_GetScriptPrivate(*compiled_script, script_private.handle_mut());

                    // When `ScriptPrivate` for the compiled script is undefined,
                    // we need to set it so that it can be used in dynamic import context.
                    if script_private.is_undefined() {
                        debug!("Set script private for {}", script_base_url);

                        let module_script_data = Rc::new(ModuleScript::new(
                            script_base_url,
                            fetch_options,
                            // We can't initialize an module owner here because
                            // the executing context of script might be different
                            // from the dynamic import script's executing context.
                            None,
                        ));

                        SetScriptPrivate(
                            *compiled_script,
                            &PrivateValue(Rc::into_raw(module_script_data) as *const _),
                        );
                    }

                    let result = JS_ExecuteScript(*cx, compiled_script.handle(), rval);

                    if !result {
                        debug!("error evaluating Dom string");
                        report_pending_exception(*cx, true, InRealm::Entered(&ar));
                    }

                    maybe_resume_unwind();
                    result
                }
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

    pub fn queue_function_as_microtask(&self, callback: Rc<VoidFunction>) {
        self.enqueue_microtask(Microtask::User(UserMicrotask {
            callback,
            pipeline: self.pipeline_id(),
        }))
    }

    pub fn create_image_bitmap(
        &self,
        image: ImageBitmapSource,
        options: &ImageBitmapOptions,
    ) -> Rc<Promise> {
        let in_realm_proof = AlreadyInRealm::assert();
        let p = Promise::new_in_current_realm(InRealm::Already(&in_realm_proof));
        if options.resizeWidth.map_or(false, |w| w == 0) {
            p.reject_error(Error::InvalidState);
            return p;
        }

        if options.resizeHeight.map_or(false, |w| w == 0) {
            p.reject_error(Error::InvalidState);
            return p;
        }

        match image {
            ImageBitmapSource::HTMLCanvasElement(ref canvas) => {
                // https://html.spec.whatwg.org/multipage/#check-the-usability-of-the-image-argument
                if !canvas.is_valid() {
                    p.reject_error(Error::InvalidState);
                    return p;
                }

                if let Some((data, size)) = canvas.fetch_all_data() {
                    let data = data
                        .map(|data| data.to_vec())
                        .unwrap_or_else(|| vec![0; size.area() as usize * 4]);

                    let image_bitmap = ImageBitmap::new(self, size.width, size.height).unwrap();

                    image_bitmap.set_bitmap_data(data);
                    image_bitmap.set_origin_clean(canvas.origin_is_clean());
                    p.resolve_native(&(image_bitmap));
                }
                p
            },
            ImageBitmapSource::OffscreenCanvas(ref canvas) => {
                // https://html.spec.whatwg.org/multipage/#check-the-usability-of-the-image-argument
                if !canvas.is_valid() {
                    p.reject_error(Error::InvalidState);
                    return p;
                }

                if let Some((data, size)) = canvas.fetch_all_data() {
                    let data = data
                        .map(|data| data.to_vec())
                        .unwrap_or_else(|| vec![0; size.area() as usize * 4]);

                    let image_bitmap = ImageBitmap::new(self, size.width, size.height).unwrap();
                    image_bitmap.set_bitmap_data(data);
                    image_bitmap.set_origin_clean(canvas.origin_is_clean());
                    p.resolve_native(&(image_bitmap));
                }
                p
            },
            _ => {
                p.reject_error(Error::NotSupported);
                p
            },
        }
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

    /// Returns a boolean indicating whether the event-loop
    /// where this global is running on can continue running JS.
    pub fn can_continue_running(&self) -> bool {
        if self.downcast::<Window>().is_some() {
            return ScriptThread::can_continue_running();
        }
        if let Some(worker) = self.downcast::<WorkerGlobalScope>() {
            return !worker.is_closing();
        }

        // TODO: plug worklets into this.
        true
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
        // Only perform the checkpoint if we're not shutting down.
        if self.can_continue_running() {
            self.microtask_queue.checkpoint(
                GlobalScope::get_cx(),
                |_| Some(DomRoot::from_ref(self)),
                vec![DomRoot::from_ref(self)],
            );
        }
    }

    /// Enqueue a microtask for subsequent execution.
    pub fn enqueue_microtask(&self, job: Microtask) {
        self.microtask_queue.enqueue(job, GlobalScope::get_cx());
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
    /// in the queue for the event-loop where this global scope is running on.
    /// Returns a boolean indicating whether further events should be processed.
    pub fn process_event(&self, msg: CommonScriptMsg) -> bool {
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

    /// <https://w3c.github.io/performance-timeline/#supportedentrytypes-attribute>
    pub fn supported_performance_entry_types(&self, cx: SafeJSContext) -> JSVal {
        if let Some(types) = &*self.frozen_supported_performance_entry_types.borrow() {
            return types.get();
        }

        let types: Vec<DOMString> = VALID_ENTRY_TYPES
            .iter()
            .map(|t| DOMString::from(t.to_string()))
            .collect();
        let frozen_types = to_frozen_array(types.as_slice(), cx);

        // Safety: need to create the Heap value in its final memory location before setting it.
        *self.frozen_supported_performance_entry_types.borrow_mut() = Some(Heap::default());
        self.frozen_supported_performance_entry_types
            .borrow()
            .as_ref()
            .unwrap()
            .set(frozen_types);

        frozen_types
    }

    pub fn is_headless(&self) -> bool {
        self.is_headless
    }

    pub fn get_user_agent(&self) -> Cow<'static, str> {
        self.user_agent.clone()
    }

    pub fn get_https_state(&self) -> HttpsState {
        self.https_state.get()
    }

    pub fn set_https_state(&self, https_state: HttpsState) {
        self.https_state.set(https_state);
    }

    pub fn is_secure_context(&self) -> bool {
        if Some(false) == self.inherited_secure_context {
            return false;
        }
        if let Some(creation_url) = self.creation_url() {
            if creation_url.scheme() == "blob" && Some(true) == self.inherited_secure_context {
                return true;
            }
            return creation_url.is_potentially_trustworthy();
        }
        false
    }

    /// <https://www.w3.org/TR/CSP/#get-csp-of-object>
    pub fn get_csp_list(&self) -> Option<CspList> {
        if let Some(window) = self.downcast::<Window>() {
            return window.Document().get_csp_list().map(|c| c.clone());
        }
        // TODO: Worker and Worklet global scopes.
        None
    }

    pub fn wgpu_id_hub(&self) -> Arc<Identities> {
        self.gpu_id_hub.clone()
    }

    pub fn add_gpu_device(&self, device: &GPUDevice) {
        self.gpu_devices
            .borrow_mut()
            .insert(device.id(), WeakRef::new(device));
    }

    pub fn remove_gpu_device(&self, device: WebGPUDevice) {
        let device = self
            .gpu_devices
            .borrow_mut()
            .remove(&device)
            .expect("GPUDevice should still be in devices hashmap");
        assert!(device.root().is_none())
    }

    pub fn gpu_device_lost(&self, device: WebGPUDevice, reason: DeviceLostReason, msg: String) {
        let reason = match reason {
            DeviceLostReason::Unknown => GPUDeviceLostReason::Unknown,
            DeviceLostReason::Destroyed => GPUDeviceLostReason::Destroyed,
        };
        let _ac = enter_realm(self);
        if let Some(device) = self
            .gpu_devices
            .borrow_mut()
            .get_mut(&device)
            .expect("GPUDevice should still be in devices hashmap")
            .root()
        {
            device.lose(reason, msg);
        }
    }

    pub fn handle_uncaptured_gpu_error(&self, device: WebGPUDevice, error: webgpu::Error) {
        if let Some(gpu_device) = self
            .gpu_devices
            .borrow()
            .get(&device)
            .and_then(|device| device.root())
        {
            gpu_device.fire_uncaptured_error(error);
        } else {
            warn!("Recived error for lost GPUDevice!")
        }
    }

    pub fn handle_gamepad_event(&self, gamepad_event: GamepadEvent) {
        match gamepad_event {
            GamepadEvent::Connected(index, name, bounds, supported_haptic_effects) => {
                self.handle_gamepad_connect(
                    index.0,
                    name,
                    bounds.axis_bounds,
                    bounds.button_bounds,
                    supported_haptic_effects,
                );
            },
            GamepadEvent::Disconnected(index) => {
                self.handle_gamepad_disconnect(index.0);
            },
            GamepadEvent::Updated(index, update_type) => {
                self.receive_new_gamepad_button_or_axis(index.0, update_type);
            },
        };
    }

    /// <https://www.w3.org/TR/gamepad/#dfn-gamepadconnected>
    pub fn handle_gamepad_connect(
        &self,
        // As the spec actually defines how to set the gamepad index, the GilRs index
        // is currently unused, though in practice it will almost always be the same.
        // More infra is currently needed to track gamepads across windows.
        _index: usize,
        name: String,
        axis_bounds: (f64, f64),
        button_bounds: (f64, f64),
        supported_haptic_effects: GamepadSupportedHapticEffects,
    ) {
        // TODO: 2. If document is not null and is not allowed to use the "gamepad" permission,
        //          then abort these steps.
        let this = Trusted::new(self);
        self.gamepad_task_source().queue_with_canceller(
            task!(gamepad_connected: move || {
                let global = this.root();

                if let Some(window) = global.downcast::<Window>() {
                    let navigator = window.Navigator();
                    let selected_index = navigator.select_gamepad_index();
                    let gamepad = Gamepad::new(
                        &global, selected_index, name, axis_bounds, button_bounds, supported_haptic_effects
                    );
                    navigator.set_gamepad(selected_index as usize, &gamepad);
                }
            }),
            &self.task_canceller(TaskSourceName::Gamepad)
        )
        .expect("Failed to queue gamepad connected task.");
    }

    /// <https://www.w3.org/TR/gamepad/#dfn-gamepaddisconnected>
    pub fn handle_gamepad_disconnect(&self, index: usize) {
        let this = Trusted::new(self);
        self.gamepad_task_source()
            .queue_with_canceller(
                task!(gamepad_disconnected: move || {
                    let global = this.root();
                    if let Some(window) = global.downcast::<Window>() {
                        let navigator = window.Navigator();
                        if let Some(gamepad) = navigator.get_gamepad(index) {
                            if window.Document().is_fully_active() {
                                gamepad.update_connected(false, gamepad.exposed());
                                navigator.remove_gamepad(index);
                            }
                        }
                    }
                }),
                &self.task_canceller(TaskSourceName::Gamepad),
            )
            .expect("Failed to queue gamepad disconnected task.");
    }

    /// <https://www.w3.org/TR/gamepad/#receiving-inputs>
    pub fn receive_new_gamepad_button_or_axis(&self, index: usize, update_type: GamepadUpdateType) {
        let this = Trusted::new(self);

        // <https://w3c.github.io/gamepad/#dfn-update-gamepad-state>
        self.gamepad_task_source()
            .queue_with_canceller(
                task!(update_gamepad_state: move || {
                    let global = this.root();
                    if let Some(window) = global.downcast::<Window>() {
                        let navigator = window.Navigator();
                        if let Some(gamepad) = navigator.get_gamepad(index) {
                            let current_time = global.performance().Now();
                            gamepad.update_timestamp(*current_time);
                            match update_type {
                                GamepadUpdateType::Axis(index, value) => {
                                    gamepad.map_and_normalize_axes(index, value);
                                },
                                GamepadUpdateType::Button(index, value) => {
                                    gamepad.map_and_normalize_buttons(index, value);
                                }
                            };
                            if !navigator.has_gamepad_gesture() && contains_user_gesture(update_type) {
                                navigator.set_has_gamepad_gesture(true);
                                navigator.GetGamepads()
                                    .iter()
                                    .filter_map(|g| g.as_ref())
                                    .for_each(|gamepad| {
                                        gamepad.set_exposed(true);
                                        gamepad.update_timestamp(*current_time);
                                        let new_gamepad = Trusted::new(&**gamepad);
                                        if window.Document().is_fully_active() {
                                            window.task_manager().gamepad_task_source().queue_with_canceller(
                                                task!(update_gamepad_connect: move || {
                                                    let gamepad = new_gamepad.root();
                                                    gamepad.notify_event(GamepadEventType::Connected);
                                                }),
                                                &window.upcast::<GlobalScope>()
                                                    .task_canceller(TaskSourceName::Gamepad),
                                            )
                                            .expect("Failed to queue update gamepad connect task.");
                                        }
                                });
                            }
                        }
                    }
                }),
                &self.task_canceller(TaskSourceName::Gamepad),
            )
            .expect("Failed to queue update gamepad state task.");
    }

    pub(crate) fn current_group_label(&self) -> Option<DOMString> {
        self.console_group_stack
            .borrow()
            .last()
            .map(|label| DOMString::from(format!("[{}]", label)))
    }

    pub(crate) fn push_console_group(&self, group: DOMString) {
        self.console_group_stack.borrow_mut().push(group);
    }

    pub(crate) fn pop_console_group(&self) {
        let _ = self.console_group_stack.borrow_mut().pop();
    }

    pub(crate) fn increment_console_count(&self, label: &DOMString) -> usize {
        *self
            .console_count_map
            .borrow_mut()
            .entry(label.clone())
            .and_modify(|e| *e += 1)
            .or_insert(1)
    }

    pub(crate) fn reset_console_count(&self, label: &DOMString) -> Result<(), ()> {
        match self.console_count_map.borrow_mut().get_mut(label) {
            Some(value) => {
                *value = 0;
                Ok(())
            },
            None => Err(()),
        }
    }

    pub(crate) fn dynamic_module_list(&self) -> RefMut<DynamicModuleList> {
        self.dynamic_modules.borrow_mut()
    }
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
