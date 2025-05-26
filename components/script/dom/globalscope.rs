/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use std::cell::{Cell, OnceCell, Ref};
use std::collections::hash_map::Entry;
use std::collections::{HashMap, VecDeque};
use std::ops::Index;
use std::rc::Rc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};
use std::thread::JoinHandle;
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};
use std::{mem, ptr};

use base::id::{
    BlobId, BroadcastChannelRouterId, MessagePortId, MessagePortRouterId, PipelineId,
    ServiceWorkerId, ServiceWorkerRegistrationId, WebViewId,
};
use constellation_traits::{
    BlobData, BlobImpl, BroadcastMsg, FileBlob, LoadData, LoadOrigin, MessagePortImpl,
    MessagePortMsg, PortMessageTask, ScriptToConstellationChan, ScriptToConstellationMessage,
};
use content_security_policy::{
    CheckResult, CspList, Destination, Initiator, NavigationCheckType, ParserMetadata,
    PolicyDisposition, PolicySource, Request, Violation, ViolationResource,
};
use crossbeam_channel::Sender;
use devtools_traits::{PageError, ScriptToDevtoolsControlMsg};
use dom_struct::dom_struct;
use embedder_traits::EmbedderMsg;
use http::HeaderMap;
use hyper_serde::Serde;
use ipc_channel::ipc::{self, IpcSender};
use ipc_channel::router::ROUTER;
use js::glue::{IsWrapper, UnwrapObjectDynamic};
use js::jsapi::{
    Compile1, CurrentGlobalOrNull, GetNonCCWObjectGlobal, HandleObject, Heap,
    InstantiateGlobalStencil, InstantiateOptions, JSContext, JSObject, JSScript, SetScriptPrivate,
};
use js::jsval::{PrivateValue, UndefinedValue};
use js::panic::maybe_resume_unwind;
use js::rust::wrappers::{JS_ExecuteScript, JS_GetScriptPrivate};
use js::rust::{
    CompileOptionsWrapper, CustomAutoRooter, CustomAutoRooterGuard, HandleValue,
    MutableHandleValue, ParentRuntime, Runtime, describe_scripted_caller, get_object_class,
    transform_str_to_source_text,
};
use js::{JSCLASS_IS_DOMJSCLASS, JSCLASS_IS_GLOBAL};
use net_traits::blob_url_store::{BlobBuf, get_blob_origin};
use net_traits::filemanager_thread::{
    FileManagerResult, FileManagerThreadMsg, ReadFileProgress, RelativePos,
};
use net_traits::image_cache::ImageCache;
use net_traits::policy_container::PolicyContainer;
use net_traits::request::{InsecureRequestsPolicy, Referrer, RequestBuilder};
use net_traits::response::HttpsState;
use net_traits::{
    CoreResourceMsg, CoreResourceThread, FetchResponseListener, IpcSend, ReferrerPolicy,
    ResourceThreads, fetch_async,
};
use profile_traits::{ipc as profile_ipc, mem as profile_mem, time as profile_time};
use script_bindings::interfaces::GlobalScopeHelpers;
use servo_url::{ImmutableOrigin, MutableOrigin, ServoUrl};
use timers::{TimerEventId, TimerEventRequest, TimerSource};
use url::Origin;
use uuid::Uuid;
#[cfg(feature = "webgpu")]
use webgpu_traits::{DeviceLostReason, WebGPUDevice};

use super::bindings::codegen::Bindings::MessagePortBinding::StructuredSerializeOptions;
#[cfg(feature = "webgpu")]
use super::bindings::codegen::Bindings::WebGPUBinding::GPUDeviceLostReason;
use super::bindings::error::Fallible;
use super::bindings::trace::{HashMapTracedValues, RootedTraceableBox};
use super::serviceworkerglobalscope::ServiceWorkerGlobalScope;
use super::transformstream::CrossRealmTransform;
use crate::dom::bindings::cell::{DomRefCell, RefMut};
use crate::dom::bindings::codegen::Bindings::BroadcastChannelBinding::BroadcastChannelMethods;
use crate::dom::bindings::codegen::Bindings::EventSourceBinding::EventSource_Binding::EventSourceMethods;
use crate::dom::bindings::codegen::Bindings::FunctionBinding::Function;
use crate::dom::bindings::codegen::Bindings::ImageBitmapBinding::{
    ImageBitmapOptions, ImageBitmapSource,
};
use crate::dom::bindings::codegen::Bindings::NotificationBinding::NotificationPermissionCallback;
use crate::dom::bindings::codegen::Bindings::PermissionStatusBinding::{
    PermissionName, PermissionState,
};
use crate::dom::bindings::codegen::Bindings::VoidFunctionBinding::VoidFunction;
use crate::dom::bindings::codegen::Bindings::WindowBinding::WindowMethods;
use crate::dom::bindings::codegen::Bindings::WorkerGlobalScopeBinding::WorkerGlobalScopeMethods;
use crate::dom::bindings::conversions::{root_from_object, root_from_object_static};
use crate::dom::bindings::error::{Error, ErrorInfo, report_pending_exception};
use crate::dom::bindings::frozenarray::CachedFrozenArray;
use crate::dom::bindings::inheritance::Castable;
use crate::dom::bindings::refcounted::{Trusted, TrustedPromise};
use crate::dom::bindings::reflector::{DomGlobal, DomObject};
use crate::dom::bindings::root::{Dom, DomRoot, MutNullableDom};
use crate::dom::bindings::settings_stack::{AutoEntryScript, entry_global, incumbent_global};
use crate::dom::bindings::str::DOMString;
use crate::dom::bindings::structuredclone;
use crate::dom::bindings::trace::CustomTraceable;
use crate::dom::bindings::weakref::{DOMTracker, WeakRef};
use crate::dom::blob::Blob;
use crate::dom::broadcastchannel::BroadcastChannel;
use crate::dom::crypto::Crypto;
use crate::dom::dedicatedworkerglobalscope::{
    DedicatedWorkerControlMsg, DedicatedWorkerGlobalScope,
};
use crate::dom::element::Element;
use crate::dom::errorevent::ErrorEvent;
use crate::dom::event::{Event, EventBubbles, EventCancelable, EventStatus};
use crate::dom::eventsource::EventSource;
use crate::dom::eventtarget::EventTarget;
use crate::dom::file::File;
use crate::dom::htmlscriptelement::{ScriptId, SourceCode};
use crate::dom::imagebitmap::ImageBitmap;
use crate::dom::messageevent::MessageEvent;
use crate::dom::messageport::MessagePort;
use crate::dom::node::Node;
use crate::dom::paintworkletglobalscope::PaintWorkletGlobalScope;
use crate::dom::performance::Performance;
use crate::dom::performanceobserver::VALID_ENTRY_TYPES;
use crate::dom::promise::Promise;
use crate::dom::readablestream::{CrossRealmTransformReadable, ReadableStream};
use crate::dom::serviceworker::ServiceWorker;
use crate::dom::serviceworkerregistration::ServiceWorkerRegistration;
use crate::dom::trustedtypepolicyfactory::TrustedTypePolicyFactory;
use crate::dom::underlyingsourcecontainer::UnderlyingSourceType;
#[cfg(feature = "webgpu")]
use crate::dom::webgpu::gpudevice::GPUDevice;
#[cfg(feature = "webgpu")]
use crate::dom::webgpu::identityhub::IdentityHub;
use crate::dom::window::Window;
use crate::dom::workerglobalscope::WorkerGlobalScope;
use crate::dom::workletglobalscope::WorkletGlobalScope;
use crate::dom::writablestream::CrossRealmTransformWritable;
use crate::messaging::{CommonScriptMsg, ScriptEventLoopReceiver, ScriptEventLoopSender};
use crate::microtask::{Microtask, MicrotaskQueue, UserMicrotask};
use crate::network_listener::{NetworkListener, PreInvoke};
use crate::realms::{AlreadyInRealm, InRealm, enter_realm};
use crate::script_module::{DynamicModuleList, ModuleScript, ModuleTree, ScriptFetchOptions};
use crate::script_runtime::{CanGc, JSContext as SafeJSContext, ThreadSafeJSContext};
use crate::script_thread::{ScriptThread, with_script_thread};
use crate::security_manager::{CSPViolationReportBuilder, CSPViolationReportTask};
use crate::task_manager::TaskManager;
use crate::task_source::SendableTaskSource;
use crate::timers::{
    IsInterval, OneshotTimerCallback, OneshotTimerHandle, OneshotTimers, TimerCallback,
};
use crate::unminify::unminified_path;

#[derive(JSTraceable)]
pub(crate) struct AutoCloseWorker {
    /// <https://html.spec.whatwg.org/multipage/#dom-workerglobalscope-closing>
    closing: Arc<AtomicBool>,
    /// A handle to join on the worker thread.
    join_handle: Option<JoinHandle<()>>,
    /// A sender of control messages,
    /// currently only used to signal shutdown.
    #[no_trace]
    control_sender: Sender<DedicatedWorkerControlMsg>,
    /// The context to request an interrupt on the worker thread.
    #[no_trace]
    context: ThreadSafeJSContext,
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

        self.context.request_interrupt_callback();

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
pub(crate) struct GlobalScope {
    eventtarget: EventTarget,
    crypto: MutNullableDom<Crypto>,

    /// A [`TaskManager`] for this [`GlobalScope`].
    task_manager: OnceCell<TaskManager>,

    /// The message-port router id for this global, if it is managing ports.
    message_port_state: DomRefCell<MessagePortState>,

    /// The broadcast channels state this global, if it is managing any.
    broadcast_channel_state: DomRefCell<BroadcastChannelState>,

    /// The blobs managed by this global, if any.
    blob_state: DomRefCell<HashMapTracedValues<BlobId, BlobInfo>>,

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

    /// <https://html.spec.whatwg.org/multipage/#in-error-reporting-mode>
    in_error_reporting_mode: Cell<bool>,

    /// Associated resource threads for use by DOM objects like XMLHttpRequest,
    /// including resource_thread, filemanager_thread and storage_thread
    #[no_trace]
    resource_threads: ResourceThreads,

    /// The mechanism by which time-outs and intervals are scheduled.
    /// <https://html.spec.whatwg.org/multipage/#timers>
    timers: OnceCell<OneshotTimers>,

    /// The origin of the globalscope
    #[no_trace]
    origin: MutableOrigin,

    /// <https://html.spec.whatwg.org/multipage/#concept-environment-creation-url>
    #[no_trace]
    creation_url: Option<ServoUrl>,

    /// A map for storing the previous permission state read results.
    permission_state_invocation_results: DomRefCell<HashMap<PermissionName, PermissionState>>,

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

    /// Identity Manager for WebGPU resources
    #[ignore_malloc_size_of = "defined in wgpu"]
    #[no_trace]
    #[cfg(feature = "webgpu")]
    gpu_id_hub: Arc<IdentityHub>,

    /// WebGPU devices
    #[cfg(feature = "webgpu")]
    gpu_devices: DomRefCell<HashMapTracedValues<WebGPUDevice, WeakRef<GPUDevice>>>,

    // https://w3c.github.io/performance-timeline/#supportedentrytypes-attribute
    #[ignore_malloc_size_of = "mozjs"]
    frozen_supported_performance_entry_types: CachedFrozenArray,

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

    /// Directory to store unminified scripts for this window if unminify-js
    /// opt is enabled.
    unminified_js_dir: Option<String>,

    /// The byte length queuing strategy size function that will be initialized once
    /// `size` getter of `ByteLengthQueuingStrategy` is called.
    ///
    /// <https://streams.spec.whatwg.org/#byte-length-queuing-strategy-size-function>
    #[ignore_malloc_size_of = "Rc<T> is hard"]
    byte_length_queuing_strategy_size_function: OnceCell<Rc<Function>>,

    /// The count queuing strategy size function that will be initialized once
    /// `size` getter of `CountQueuingStrategy` is called.
    ///
    /// <https://streams.spec.whatwg.org/#count-queuing-strategy-size-function>
    #[ignore_malloc_size_of = "Rc<T> is hard"]
    count_queuing_strategy_size_function: OnceCell<Rc<Function>>,

    #[ignore_malloc_size_of = "Rc<T> is hard"]
    notification_permission_request_callback_map:
        DomRefCell<HashMap<String, Rc<NotificationPermissionCallback>>>,
}

/// A wrapper for glue-code between the ipc router and the event-loop.
struct MessageListener {
    task_source: SendableTaskSource,
    context: Trusted<GlobalScope>,
}

/// A wrapper for broadcasts coming in over IPC, and the event-loop.
struct BroadcastListener {
    task_source: SendableTaskSource,
    context: Trusted<GlobalScope>,
}

type FileListenerCallback = Box<dyn Fn(Rc<Promise>, Fallible<Vec<u8>>) + Send>;

/// A wrapper for the handling of file data received by the ipc router
struct FileListener {
    /// State should progress as either of:
    /// - Some(Empty) => Some(Receiving) => None
    /// - Some(Empty) => None
    state: Option<FileListenerState>,
    task_source: SendableTaskSource,
}

enum FileListenerTarget {
    Promise(TrustedPromise, FileListenerCallback),
    Stream(Trusted<ReadableStream>),
}

enum FileListenerState {
    Empty(FileListenerTarget),
    Receiving(Vec<u8>, FileListenerTarget),
}

#[derive(JSTraceable, MallocSizeOf)]
/// A holder of a weak reference for a DOM blob or file.
pub(crate) enum BlobTracker {
    /// A weak ref to a DOM file.
    File(WeakRef<File>),
    /// A weak ref to a DOM blob.
    Blob(WeakRef<Blob>),
}

#[derive(JSTraceable, MallocSizeOf)]
/// The info pertaining to a blob managed by this global.
pub(crate) struct BlobInfo {
    /// The weak ref to the corresponding DOM object.
    tracker: BlobTracker,
    /// The data and logic backing the DOM object.
    #[no_trace]
    blob_impl: BlobImpl,
    /// Whether this blob has an outstanding URL,
    /// <https://w3c.github.io/FileAPI/#url>.
    has_url: bool,
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
#[cfg_attr(crown, crown::unrooted_must_root_lint::must_root)]
pub(crate) struct ManagedMessagePort {
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
    /// Whether the port has been closed by script in this global,
    /// so it can be removed.
    explicitly_closed: bool,
    /// The handler for `message` or `messageerror` used in the cross realm transform,
    /// if any was setup with this port.
    cross_realm_transform: Option<CrossRealmTransform>,
}

/// State representing whether this global is currently managing broadcast channels.
#[derive(JSTraceable, MallocSizeOf)]
#[cfg_attr(crown, crown::unrooted_must_root_lint::must_root)]
pub(crate) enum BroadcastChannelState {
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
#[cfg_attr(crown, crown::unrooted_must_root_lint::must_root)]
pub(crate) enum MessagePortState {
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
        self.task_source
            .queue(task!(broadcast_message_event: move || {
                let global = context.root();
                // Step 10 of https://html.spec.whatwg.org/multipage/#dom-broadcastchannel-postmessage,
                // For each BroadcastChannel object destination in destinations, queue a task.
                global.broadcast_message_event(event, None);
            }));
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
                self.task_source.queue(
                    task!(process_complete_transfer: move || {
                        let global = context.root();

                        let router_id = match global.port_router_id() {
                            Some(router_id) => router_id,
                            None => {
                                // If not managing any ports, no transfer can succeed,
                                // so just send back everything.
                                let _ = global.script_to_constellation_chan().send(
                                    ScriptToConstellationMessage::MessagePortTransferResult(None, vec![], ports),
                                );
                                return;
                            }
                        };

                        let mut succeeded = vec![];
                        let mut failed = HashMap::new();

                        for (id, info) in ports.into_iter() {
                            if global.is_managing_port(&id) {
                                succeeded.push(id);
                                global.complete_port_transfer(
                                    id,
                                    info.port_message_queue,
                                    info.disentangled,
                                    CanGc::note()
                                );
                            } else {
                                failed.insert(id, info);
                            }
                        }
                        let _ = global.script_to_constellation_chan().send(
                            ScriptToConstellationMessage::MessagePortTransferResult(Some(router_id), succeeded, failed),
                        );
                    })
                );
            },
            MessagePortMsg::CompletePendingTransfer(port_id, info) => {
                let context = self.context.clone();
                self.task_source.queue(task!(complete_pending: move || {
                    let global = context.root();
                    global.complete_port_transfer(port_id, info.port_message_queue, info.disentangled, CanGc::note());
                }));
            },
            MessagePortMsg::CompleteDisentanglement(port_id) => {
                let context = self.context.clone();
                self.task_source
                    .queue(task!(try_complete_disentanglement: move || {
                        let global = context.root();
                        global.try_complete_disentanglement(port_id, CanGc::note());
                    }));
            },
            MessagePortMsg::NewTask(port_id, task) => {
                let context = self.context.clone();
                self.task_source.queue(task!(process_new_task: move || {
                    let global = context.root();
                    global.route_task_to_port(port_id, task, CanGc::note());
                }));
            },
        }
    }
}

/// Callback used to enqueue file chunks to streams as part of FileListener.
fn stream_handle_incoming(stream: &ReadableStream, bytes: Fallible<Vec<u8>>, can_gc: CanGc) {
    match bytes {
        Ok(b) => {
            stream.enqueue_native(b, can_gc);
        },
        Err(e) => {
            stream.error_native(e, can_gc);
        },
    }
}

/// Callback used to close streams as part of FileListener.
fn stream_handle_eof(stream: &ReadableStream, can_gc: CanGc) {
    stream.controller_close_native(can_gc);
}

impl FileListener {
    fn handle(&mut self, msg: FileManagerResult<ReadFileProgress>) {
        match msg {
            Ok(ReadFileProgress::Meta(blob_buf)) => match self.state.take() {
                Some(FileListenerState::Empty(target)) => {
                    let bytes = if let FileListenerTarget::Stream(ref trusted_stream) = target {
                        let trusted = trusted_stream.clone();

                        let task = task!(enqueue_stream_chunk: move || {
                            let stream = trusted.root();
                            stream_handle_incoming(&stream, Ok(blob_buf.bytes), CanGc::note());
                        });
                        self.task_source.queue(task);

                        Vec::with_capacity(0)
                    } else {
                        blob_buf.bytes
                    };

                    self.state = Some(FileListenerState::Receiving(bytes, target));
                },
                _ => panic!(
                    "Unexpected FileListenerState when receiving ReadFileProgress::Meta msg."
                ),
            },
            Ok(ReadFileProgress::Partial(mut bytes_in)) => match self.state.take() {
                Some(FileListenerState::Receiving(mut bytes, target)) => {
                    if let FileListenerTarget::Stream(ref trusted_stream) = target {
                        let trusted = trusted_stream.clone();

                        let task = task!(enqueue_stream_chunk: move || {
                            let stream = trusted.root();
                            stream_handle_incoming(&stream, Ok(bytes_in), CanGc::note());
                        });

                        self.task_source.queue(task);
                    } else {
                        bytes.append(&mut bytes_in);
                    };

                    self.state = Some(FileListenerState::Receiving(bytes, target));
                },
                _ => panic!(
                    "Unexpected FileListenerState when receiving ReadFileProgress::Partial msg."
                ),
            },
            Ok(ReadFileProgress::EOF) => match self.state.take() {
                Some(FileListenerState::Receiving(bytes, target)) => match target {
                    FileListenerTarget::Promise(trusted_promise, callback) => {
                        let task = task!(resolve_promise: move || {
                            let promise = trusted_promise.root();
                            let _ac = enter_realm(&*promise.global());
                            callback(promise, Ok(bytes));
                        });

                        self.task_source.queue(task);
                    },
                    FileListenerTarget::Stream(trusted_stream) => {
                        let trusted = trusted_stream.clone();

                        let task = task!(enqueue_stream_chunk: move || {
                            let stream = trusted.root();
                            stream_handle_eof(&stream, CanGc::note());
                        });

                        self.task_source.queue(task);
                    },
                },
                _ => {
                    panic!("Unexpected FileListenerState when receiving ReadFileProgress::EOF msg.")
                },
            },
            Err(_) => match self.state.take() {
                Some(FileListenerState::Receiving(_, target)) |
                Some(FileListenerState::Empty(target)) => {
                    let error = Err(Error::Network);

                    match target {
                        FileListenerTarget::Promise(trusted_promise, callback) => {
                            self.task_source.queue(task!(reject_promise: move || {
                                let promise = trusted_promise.root();
                                let _ac = enter_realm(&*promise.global());
                                callback(promise, error);
                            }));
                        },
                        FileListenerTarget::Stream(trusted_stream) => {
                            self.task_source.queue(task!(error_stream: move || {
                                let stream = trusted_stream.root();
                                stream_handle_incoming(&stream, error, CanGc::note());
                            }));
                        },
                    }
                },
                _ => panic!("Unexpected FileListenerState when receiving Err msg."),
            },
        }
    }
}

impl GlobalScope {
    /// A sender to the event loop of this global scope. This either sends to the Worker event loop
    /// or the ScriptThread event loop in the case of a `Window`. This can be `None` for dedicated
    /// workers that are not currently handling a message.
    pub(crate) fn webview_id(&self) -> Option<WebViewId> {
        if let Some(window) = self.downcast::<Window>() {
            Some(window.webview_id())
        } else if let Some(dedicated) = self.downcast::<DedicatedWorkerGlobalScope>() {
            dedicated.webview_id()
        } else {
            // ServiceWorkerGlobalScope, PaintWorklet, or DissimilarOriginWindow
            None
        }
    }

    #[allow(clippy::too_many_arguments)]
    pub(crate) fn new_inherited(
        pipeline_id: PipelineId,
        devtools_chan: Option<IpcSender<ScriptToDevtoolsControlMsg>>,
        mem_profiler_chan: profile_mem::ProfilerChan,
        time_profiler_chan: profile_time::ProfilerChan,
        script_to_constellation_chan: ScriptToConstellationChan,
        resource_threads: ResourceThreads,
        origin: MutableOrigin,
        creation_url: Option<ServoUrl>,
        microtask_queue: Rc<MicrotaskQueue>,
        #[cfg(feature = "webgpu")] gpu_id_hub: Arc<IdentityHub>,
        inherited_secure_context: Option<bool>,
        unminify_js: bool,
    ) -> Self {
        Self {
            task_manager: Default::default(),
            message_port_state: DomRefCell::new(MessagePortState::UnManaged),
            broadcast_channel_state: DomRefCell::new(BroadcastChannelState::UnManaged),
            blob_state: Default::default(),
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
            in_error_reporting_mode: Default::default(),
            resource_threads,
            timers: OnceCell::default(),
            origin,
            creation_url,
            permission_state_invocation_results: Default::default(),
            microtask_queue,
            list_auto_close_worker: Default::default(),
            event_source_tracker: DOMTracker::new(),
            uncaught_rejections: Default::default(),
            consumed_rejections: Default::default(),
            #[cfg(feature = "webgpu")]
            gpu_id_hub,
            #[cfg(feature = "webgpu")]
            gpu_devices: DomRefCell::new(HashMapTracedValues::new()),
            frozen_supported_performance_entry_types: CachedFrozenArray::new(),
            https_state: Cell::new(HttpsState::None),
            console_group_stack: DomRefCell::new(Vec::new()),
            console_count_map: Default::default(),
            dynamic_modules: DomRefCell::new(DynamicModuleList::new()),
            inherited_secure_context,
            unminified_js_dir: unminify_js.then(|| unminified_path("unminified-js")),
            byte_length_queuing_strategy_size_function: OnceCell::new(),
            count_queuing_strategy_size_function: OnceCell::new(),
            notification_permission_request_callback_map: Default::default(),
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

    fn timers(&self) -> &OneshotTimers {
        self.timers.get_or_init(|| OneshotTimers::new(self))
    }

    /// <https://w3c.github.io/ServiceWorker/#get-the-service-worker-registration-object>
    #[allow(clippy::too_many_arguments)]
    pub(crate) fn get_serviceworker_registration(
        &self,
        script_url: &ServoUrl,
        scope: &ServoUrl,
        registration_id: ServiceWorkerRegistrationId,
        installing_worker: Option<ServiceWorkerId>,
        _waiting_worker: Option<ServiceWorkerId>,
        _active_worker: Option<ServiceWorkerId>,
        can_gc: CanGc,
    ) -> DomRoot<ServiceWorkerRegistration> {
        // Step 1
        let mut registrations = self.registration_map.borrow_mut();

        if let Some(registration) = registrations.get(&registration_id) {
            // Step 3
            return DomRoot::from_ref(&**registration);
        }

        // Step 2.1 -> 2.5
        let new_registration =
            ServiceWorkerRegistration::new(self, scope.clone(), registration_id, can_gc);

        // Step 2.6
        if let Some(worker_id) = installing_worker {
            let worker = self.get_serviceworker(script_url, scope, worker_id, can_gc);
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
    pub(crate) fn get_serviceworker(
        &self,
        script_url: &ServoUrl,
        scope: &ServoUrl,
        worker_id: ServiceWorkerId,
        can_gc: CanGc,
    ) -> DomRoot<ServiceWorker> {
        // Step 1
        let mut workers = self.worker_map.borrow_mut();

        if let Some(worker) = workers.get(&worker_id) {
            // Step 3
            DomRoot::from_ref(&**worker)
        } else {
            // Step 2.1
            // TODO: step 2.2, worker state.
            let new_worker =
                ServiceWorker::new(self, script_url.clone(), scope.clone(), worker_id, can_gc);

            // Step 2.3
            workers.insert(worker_id, Dom::from_ref(&*new_worker));

            // Step 3
            new_worker
        }
    }

    /// Complete the transfer of a message-port.
    fn complete_port_transfer(
        &self,
        port_id: MessagePortId,
        tasks: VecDeque<PortMessageTask>,
        disentangled: bool,
        can_gc: CanGc,
    ) {
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
                        if disentangled {
                            port_impl.disentangle();
                            managed_port.dom_port.disentangle();
                        }
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
            self.start_message_port(&port_id, can_gc);
        }
    }

    /// The closing of `otherPort`, if it is in a different global.
    /// <https://html.spec.whatwg.org/multipage/#disentangle>
    fn try_complete_disentanglement(&self, port_id: MessagePortId, can_gc: CanGc) {
        let dom_port = if let MessagePortState::Managed(_id, message_ports) =
            &mut *self.message_port_state.borrow_mut()
        {
            let dom_port = if let Some(managed_port) = message_ports.get_mut(&port_id) {
                if managed_port.pending {
                    unreachable!("CompleteDisentanglement msg received for a pending port.");
                }
                let port_impl = managed_port
                    .port_impl
                    .as_mut()
                    .expect("managed-port has no port-impl.");
                port_impl.disentangle();
                managed_port.dom_port.as_rooted()
            } else {
                // Note: this, and the other return below,
                // can happen if the port has already been transferred out of this global,
                // in which case the disentanglement will complete along with the transfer.
                return;
            };
            dom_port
        } else {
            return;
        };

        // Fire an event named close at otherPort.
        dom_port.upcast().fire_event(atom!("close"), can_gc);

        let res = self.script_to_constellation_chan().send(
            ScriptToConstellationMessage::DisentanglePorts(port_id, None),
        );
        if res.is_err() {
            warn!("Sending DisentanglePorts failed");
        }
    }

    /// Clean-up DOM related resources
    pub(crate) fn perform_a_dom_garbage_collection_checkpoint(&self) {
        self.perform_a_message_port_garbage_collection_checkpoint();
        self.perform_a_blob_garbage_collection_checkpoint();
        self.perform_a_broadcast_channel_garbage_collection_checkpoint();
    }

    /// Remove the routers for ports and broadcast-channels.
    /// Drain the list of workers.
    pub(crate) fn remove_web_messaging_and_dedicated_workers_infra(&self) {
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
            let _ = self.script_to_constellation_chan().send(
                ScriptToConstellationMessage::RemoveMessagePortRouter(*router_id),
            );
        }
        *self.message_port_state.borrow_mut() = MessagePortState::UnManaged;
    }

    /// Update our state to un-managed,
    /// and tell the constellation to drop the sender to our broadcast router.
    fn remove_broadcast_channel_router(&self) {
        if let BroadcastChannelState::Managed(router_id, _channels) =
            &*self.broadcast_channel_state.borrow()
        {
            let _ = self.script_to_constellation_chan().send(
                ScriptToConstellationMessage::RemoveBroadcastChannelRouter(
                    *router_id,
                    self.origin().immutable().clone(),
                ),
            );
        }
        *self.broadcast_channel_state.borrow_mut() = BroadcastChannelState::UnManaged;
    }

    /// <https://html.spec.whatwg.org/multipage/#disentangle>
    pub(crate) fn disentangle_port(&self, port: &MessagePort, can_gc: CanGc) {
        let initiator_port = port.message_port_id();
        // Let otherPort be the MessagePort which initiatorPort was entangled with.
        let Some(other_port) = port.disentangle() else {
            // Assert: otherPort exists.
            // Note: ignoring the assert,
            // because the streams spec seems to disentangle ports that are disentangled already.
            return;
        };

        // Disentangle initiatorPort and otherPort, so that they are no longer entangled or associated with each other.
        // Note: this is done in part here, and in part at the constellation(if otherPort is in another global).
        let dom_port = if let MessagePortState::Managed(_id, message_ports) =
            &mut *self.message_port_state.borrow_mut()
        {
            let mut dom_port = None;
            for port_id in &[initiator_port, &other_port] {
                match message_ports.get_mut(port_id) {
                    None => {
                        continue;
                    },
                    Some(managed_port) => {
                        let port_impl = managed_port
                            .port_impl
                            .as_mut()
                            .expect("managed-port has no port-impl.");
                        managed_port.dom_port.disentangle();
                        port_impl.disentangle();

                        if **port_id == other_port {
                            dom_port = Some(managed_port.dom_port.as_rooted())
                        }
                    },
                }
            }
            dom_port
        } else {
            panic!("disentangle_port called on a global not managing any ports.");
        };

        // Fire an event named close at `otherPort`.
        // Note: done here if the port is managed by the same global as `initialPort`.
        if let Some(dom_port) = dom_port {
            dom_port.upcast().fire_event(atom!("close"), can_gc);
        }

        let chan = self.script_to_constellation_chan().clone();
        let initiator_port = *initiator_port;
        self.task_manager()
            .port_message_queue()
            .queue(task!(post_message: move || {
                // Note: we do this in a task to ensure it doesn't affect messages that are still to be routed,
                // see the task queueing in `post_messageport_msg`.
                let res = chan.send(ScriptToConstellationMessage::DisentanglePorts(initiator_port, Some(other_port)));
                if res.is_err() {
                    warn!("Sending DisentanglePorts failed");
                }
            }));
    }

    /// <https://html.spec.whatwg.org/multipage/#entangle>
    pub(crate) fn entangle_ports(&self, port1: MessagePortId, port2: MessagePortId) {
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
            .send(ScriptToConstellationMessage::EntanglePorts(port1, port2));
    }

    /// Handle the transfer of a port in the current task.
    pub(crate) fn mark_port_as_transferred(&self, port_id: &MessagePortId) -> MessagePortImpl {
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
                .send(ScriptToConstellationMessage::MessagePortShipped(*port_id));
            port_impl
        } else {
            panic!("mark_port_as_transferred called on a global not managing any ports.");
        }
    }

    /// <https://html.spec.whatwg.org/multipage/#dom-messageport-start>
    pub(crate) fn start_message_port(&self, port_id: &MessagePortId, can_gc: CanGc) {
        let (message_buffer, dom_port) = if let MessagePortState::Managed(_id, message_ports) =
            &mut *self.message_port_state.borrow_mut()
        {
            let (message_buffer, dom_port) = match message_ports.get_mut(port_id) {
                None => panic!("start_message_port called on a unknown port."),
                Some(managed_port) => {
                    if let Some(port_impl) = managed_port.port_impl.as_mut() {
                        (port_impl.start(), managed_port.dom_port.as_rooted())
                    } else {
                        panic!("managed-port has no port-impl.");
                    }
                },
            };
            (message_buffer, dom_port)
        } else {
            return warn!("start_message_port called on a global not managing any ports.");
        };
        if let Some(message_buffer) = message_buffer {
            for task in message_buffer {
                self.route_task_to_port(*port_id, task, CanGc::note());
            }
            if dom_port.disentangled() {
                // <https://html.spec.whatwg.org/multipage/#disentangle>
                // Fire an event named close at otherPort.
                dom_port.upcast().fire_event(atom!("close"), can_gc);

                let res = self.script_to_constellation_chan().send(
                    ScriptToConstellationMessage::DisentanglePorts(*port_id, None),
                );
                if res.is_err() {
                    warn!("Sending DisentanglePorts failed");
                }
            }
        }
    }

    /// <https://html.spec.whatwg.org/multipage/#dom-messageport-close>
    pub(crate) fn close_message_port(&self, port_id: &MessagePortId) {
        if let MessagePortState::Managed(_id, message_ports) =
            &mut *self.message_port_state.borrow_mut()
        {
            match message_ports.get_mut(port_id) {
                None => panic!("close_message_port called on an unknown port."),
                Some(managed_port) => {
                    if let Some(port_impl) = managed_port.port_impl.as_mut() {
                        port_impl.close();
                        managed_port.explicitly_closed = true;
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
    pub(crate) fn post_messageport_msg(&self, port_id: MessagePortId, task: PortMessageTask) {
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
                self.task_manager()
                    .port_message_queue()
                    .queue(task!(post_message: move || {
                        let global = this.root();
                        // Note: we do this in a task, as this will ensure the global and constellation
                        // are aware of any transfer that might still take place in the current task.
                        global.route_task_to_port(entangled_id, task, CanGc::note());
                    }));
            }
        } else {
            warn!("post_messageport_msg called on a global not managing any ports.");
        }
    }

    /// If we don't know about the port,
    /// send the message to the constellation for routing.
    fn re_route_port_task(&self, port_id: MessagePortId, task: PortMessageTask) {
        let _ = self.script_to_constellation_chan().send(
            ScriptToConstellationMessage::RerouteMessagePort(port_id, task),
        );
    }

    /// <https://html.spec.whatwg.org/multipage/#dom-broadcastchannel-postmessage>
    /// Step 7 and following steps.
    pub(crate) fn schedule_broadcast(&self, msg: BroadcastMsg, channel_id: &Uuid) {
        // First, broadcast locally.
        self.broadcast_message_event(msg.clone(), Some(channel_id));

        if let BroadcastChannelState::Managed(router_id, _) =
            &*self.broadcast_channel_state.borrow()
        {
            // Second, broadcast to other globals via the constellation.
            //
            // Note: for globals in the same script-thread,
            // we could skip the hop to the constellation.
            let _ = self.script_to_constellation_chan().send(
                ScriptToConstellationMessage::ScheduleBroadcast(*router_id, msg),
            );
        } else {
            panic!("Attemps to broadcast a message via global not managing any channels.");
        }
    }

    /// <https://html.spec.whatwg.org/multipage/#dom-broadcastchannel-postmessage>
    /// Step 7 and following steps.
    pub(crate) fn broadcast_message_event(&self, event: BroadcastMsg, channel_id: Option<&Uuid>) {
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
                        self.task_manager().dom_manipulation_task_source().queue(
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
                                        CanGc::note()
                                    );
                                } else {
                                    // Step 10.3, fire an event named messageerror at destination.
                                    MessageEvent::dispatch_error(destination.upcast(), &global, CanGc::note());
                                }
                            })
                        );
                    });
            }
        }
    }

    /// <https://streams.spec.whatwg.org/#abstract-opdef-setupcrossrealmtransformreadable>
    /// The "Add a handler for port’s message event with the following steps:"
    /// and "Add a handler for port’s messageerror event with the following steps:" part.
    pub(crate) fn note_cross_realm_transform_readable(
        &self,
        cross_realm_transform_readable: &CrossRealmTransformReadable,
        port_id: &MessagePortId,
    ) {
        let MessagePortState::Managed(_id, message_ports) =
            &mut *self.message_port_state.borrow_mut()
        else {
            unreachable!(
                "Cross realm transform readable must be called on a global managing ports"
            );
        };

        let Some(managed_port) = message_ports.get_mut(port_id) else {
            unreachable!("Cross realm transform readable must match a managed port");
        };

        managed_port.cross_realm_transform = Some(CrossRealmTransform::Readable(
            cross_realm_transform_readable.clone(),
        ));
    }

    /// <https://streams.spec.whatwg.org/#abstract-opdef-setupcrossrealmtransformwritable>
    /// The "Add a handler for port’s message event with the following steps:"
    /// and "Add a handler for port’s messageerror event with the following steps:" part.
    pub(crate) fn note_cross_realm_transform_writable(
        &self,
        cross_realm_transform_writable: &CrossRealmTransformWritable,
        port_id: &MessagePortId,
    ) {
        let MessagePortState::Managed(_id, message_ports) =
            &mut *self.message_port_state.borrow_mut()
        else {
            unreachable!(
                "Cross realm transform writable must be called on a global managing ports"
            );
        };

        let Some(managed_port) = message_ports.get_mut(port_id) else {
            unreachable!("Cross realm transform writable must match a managed port");
        };

        managed_port.cross_realm_transform = Some(CrossRealmTransform::Writable(
            cross_realm_transform_writable.clone(),
        ));
    }

    /// Custom routing logic, followed by the task steps of
    /// <https://html.spec.whatwg.org/multipage/#message-port-post-message-steps>
    pub(crate) fn route_task_to_port(
        &self,
        port_id: MessagePortId,
        task: PortMessageTask,
        can_gc: CanGc,
    ) {
        let cx = GlobalScope::get_cx();
        rooted!(in(*cx) let mut cross_realm_transform = None);

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
                        let to_dispatch = port_impl.handle_incoming(task).map(|to_dispatch| {
                            (DomRoot::from_ref(&*managed_port.dom_port), to_dispatch)
                        });
                        cross_realm_transform.set(managed_port.cross_realm_transform.clone());
                        to_dispatch
                    } else {
                        panic!("managed-port has no port-impl.");
                    }
                },
            }
        } else {
            self.re_route_port_task(port_id, task);
            return;
        };

        // Add a task that runs the following steps to the port message queue of targetPort:
        // Note: we are in the task, and running the relevant steps.

        // Let finalTargetPort be the MessagePort in whose port message queue the task now finds itself.
        if let Some((dom_port, PortMessageTask { origin, data })) = should_dispatch {
            // Let messageEventTarget be finalTargetPort's message event target.
            let message_event_target = dom_port.upcast();

            // Let targetRealm be finalTargetPort's relevant realm.
            // Done via the routing logic here and in the constellation: `self` is the target realm.

            // Let messageClone be deserializeRecord.[[Deserialized]].
            // Re-ordered because we need to pass it to `structuredclone::read`.
            rooted!(in(*cx) let mut message_clone = UndefinedValue());

            let realm = enter_realm(self);
            let comp = InRealm::Entered(&realm);

            // Note: this is necessary, on top of entering the realm above,
            // for the call to `GlobalScope::incumbent`,
            // in `MessagePort::post_message_impl` to succeed.
            let _aes = AutoEntryScript::new(self);

            // Let deserializeRecord be StructuredDeserializeWithTransfer(serializeWithTransferResult, targetRealm).
            // Let newPorts be a new frozen array
            // consisting of all MessagePort objects in deserializeRecord.[[TransferredValues]],
            // if any, maintaining their relative order.
            // Note: both done in `structuredclone::read`.
            if let Ok(ports) = structuredclone::read(self, data, message_clone.handle_mut()) {
                // Note: if this port is used to transfer a stream, we handle the events in Rust.
                if let Some(transform) = cross_realm_transform.as_ref() {
                    match transform {
                        // Add a handler for port’s message event with the following steps:
                        // from <https://streams.spec.whatwg.org/#abstract-opdef-setupcrossrealmtransformreadable>
                        CrossRealmTransform::Readable(readable) => {
                            readable.handle_message(
                                cx,
                                self,
                                &dom_port,
                                message_clone.handle(),
                                comp,
                                can_gc,
                            );
                        },
                        // Add a handler for port’s message event with the following steps:
                        // from <https://streams.spec.whatwg.org/#abstract-opdef-setupcrossrealmtransformwritable>
                        CrossRealmTransform::Writable(writable) => {
                            writable.handle_message(cx, self, message_clone.handle(), comp, can_gc);
                        },
                    }
                } else {
                    // Fire an event named message at messageEventTarget,
                    // using MessageEvent,
                    // with the data attribute initialized to messageClone
                    // and the ports attribute initialized to newPorts.
                    MessageEvent::dispatch_jsval(
                        message_event_target,
                        self,
                        message_clone.handle(),
                        Some(&origin.ascii_serialization()),
                        None,
                        ports,
                        can_gc,
                    );
                }
            } else if let Some(transform) = cross_realm_transform.as_ref() {
                match transform {
                    // Add a handler for port’s messageerror event with the following steps:
                    // from <https://streams.spec.whatwg.org/#abstract-opdef-setupcrossrealmtransformreadable>
                    CrossRealmTransform::Readable(readable) => {
                        readable.handle_error(cx, self, &dom_port, comp, can_gc);
                    },
                    // Add a handler for port’s messageerror event with the following steps:
                    // from <https://streams.spec.whatwg.org/#abstract-opdef-setupcrossrealmtransformwritable>
                    CrossRealmTransform::Writable(writable) => {
                        writable.handle_error(cx, self, &dom_port, comp, can_gc);
                    },
                }
            } else {
                // If this throws an exception, catch it,
                // fire an event named messageerror at messageEventTarget,
                // using MessageEvent, and then return.
                MessageEvent::dispatch_error(message_event_target, self, can_gc);
            }
        }
    }

    /// Check all ports that have been transfer-received in the previous task,
    /// and complete their transfer if they haven't been re-transferred.
    pub(crate) fn maybe_add_pending_ports(&self) {
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
            let _ = self.script_to_constellation_chan().send(
                ScriptToConstellationMessage::CompleteMessagePortTransfer(*router_id, to_be_added),
            );
        } else {
            warn!("maybe_add_pending_ports called on a global not managing any ports.");
        }
    }

    /// <https://html.spec.whatwg.org/multipage/#ports-and-garbage-collection>
    pub(crate) fn perform_a_message_port_garbage_collection_checkpoint(&self) {
        let is_empty = if let MessagePortState::Managed(_id, message_ports) =
            &mut *self.message_port_state.borrow_mut()
        {
            let to_be_removed: Vec<MessagePortId> = message_ports
                .iter()
                .filter_map(|(id, managed_port)| {
                    if managed_port.explicitly_closed {
                        Some(*id)
                    } else {
                        None
                    }
                })
                .collect();
            for id in to_be_removed {
                message_ports.remove(&id);
            }
            // Note: ports are only removed throught explicit closure by script in this global.
            // TODO: #25772
            // TODO: remove ports when we can be sure their port message queue is empty(via the constellation).
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
    pub(crate) fn perform_a_broadcast_channel_garbage_collection_checkpoint(&self) {
        let is_empty = if let BroadcastChannelState::Managed(router_id, channels) =
            &mut *self.broadcast_channel_state.borrow_mut()
        {
            channels.retain(|name, ref mut channels| {
                channels.retain(|chan| !chan.closed());
                if channels.is_empty() {
                    let _ = self.script_to_constellation_chan().send(
                        ScriptToConstellationMessage::RemoveBroadcastChannelNameInRouter(
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
    pub(crate) fn track_broadcast_channel(&self, dom_channel: &BroadcastChannel) {
        let mut current_state = self.broadcast_channel_state.borrow_mut();

        if let BroadcastChannelState::UnManaged = &*current_state {
            // Setup a route for IPC, for broadcasts from the constellation to our channels.
            let (broadcast_control_sender, broadcast_control_receiver) =
                ipc::channel().expect("ipc channel failure");
            let context = Trusted::new(self);
            let listener = BroadcastListener {
                task_source: self.task_manager().dom_manipulation_task_source().into(),
                context,
            };
            ROUTER.add_typed_route(
                broadcast_control_receiver,
                Box::new(move |message| match message {
                    Ok(msg) => listener.handle(msg),
                    Err(err) => warn!("Error receiving a BroadcastMsg: {:?}", err),
                }),
            );
            let router_id = BroadcastChannelRouterId::new();
            *current_state = BroadcastChannelState::Managed(router_id, HashMap::new());
            let _ = self.script_to_constellation_chan().send(
                ScriptToConstellationMessage::NewBroadcastChannelRouter(
                    router_id,
                    broadcast_control_sender,
                    self.origin().immutable().clone(),
                ),
            );
        }

        if let BroadcastChannelState::Managed(router_id, channels) = &mut *current_state {
            let entry = channels.entry(dom_channel.Name()).or_insert_with(|| {
                let _ = self.script_to_constellation_chan().send(
                    ScriptToConstellationMessage::NewBroadcastChannelNameInRouter(
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
    pub(crate) fn track_message_port(
        &self,
        dom_port: &MessagePort,
        port_impl: Option<MessagePortImpl>,
    ) {
        let mut current_state = self.message_port_state.borrow_mut();

        if let MessagePortState::UnManaged = &*current_state {
            // Setup a route for IPC, for messages from the constellation to our ports.
            let (port_control_sender, port_control_receiver) =
                ipc::channel().expect("ipc channel failure");
            let context = Trusted::new(self);
            let listener = MessageListener {
                task_source: self.task_manager().port_message_queue().into(),
                context,
            };
            ROUTER.add_typed_route(
                port_control_receiver,
                Box::new(move |message| match message {
                    Ok(msg) => listener.notify(msg),
                    Err(err) => warn!("Error receiving a MessagePortMsg: {:?}", err),
                }),
            );
            let router_id = MessagePortRouterId::new();
            *current_state = MessagePortState::Managed(router_id, HashMapTracedValues::new());
            let _ = self.script_to_constellation_chan().send(
                ScriptToConstellationMessage::NewMessagePortRouter(router_id, port_control_sender),
            );
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
                        explicitly_closed: false,
                        cross_realm_transform: None,
                    },
                );

                // Queue a task to complete the transfer,
                // unless the port is re-transferred in the current task.
                let this = Trusted::new(self);
                self.task_manager().port_message_queue().queue(
                    task!(process_pending_port_messages: move || {
                        let target_global = this.root();
                        target_global.maybe_add_pending_ports();
                    }),
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
                        explicitly_closed: false,
                        cross_realm_transform: None,
                    },
                );
                let _ = self.script_to_constellation_chan().send(
                    ScriptToConstellationMessage::NewMessagePort(
                        *router_id,
                        *dom_port.message_port_id(),
                    ),
                );
            };
        } else {
            panic!("track_message_port should have first switched the state to managed.");
        }
    }

    /// <https://html.spec.whatwg.org/multipage/#serialization-steps>
    /// defined at <https://w3c.github.io/FileAPI/#blob-section>.
    /// Get the snapshot state and underlying bytes of the blob.
    pub(crate) fn serialize_blob(&self, blob_id: &BlobId) -> BlobImpl {
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
        self.blob_state.borrow_mut().insert(blob_id, blob_info);
    }

    /// Start tracking a blob
    pub(crate) fn track_blob(&self, dom_blob: &Blob, blob_impl: BlobImpl) {
        let blob_id = blob_impl.blob_id();

        let blob_info = BlobInfo {
            blob_impl,
            tracker: BlobTracker::Blob(WeakRef::new(dom_blob)),
            has_url: false,
        };

        self.track_blob_info(blob_info, blob_id);
    }

    /// Start tracking a file
    pub(crate) fn track_file(&self, file: &File, blob_impl: BlobImpl) {
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
        blob_state.0.retain(|_id, blob_info| {
            let garbage_collected = match &blob_info.tracker {
                BlobTracker::File(weak) => weak.root().is_none(),
                BlobTracker::Blob(weak) => weak.root().is_none(),
            };
            if garbage_collected && !blob_info.has_url {
                if let BlobData::File(f) = blob_info.blob_impl.blob_data() {
                    self.decrement_file_ref(f.get_id());
                }
                false
            } else {
                true
            }
        });
    }

    /// Clean-up all file related resources on document unload.
    /// <https://w3c.github.io/FileAPI/#lifeTime>
    pub(crate) fn clean_up_all_file_resources(&self) {
        self.blob_state
            .borrow_mut()
            .drain()
            .for_each(|(_id, blob_info)| {
                if let BlobData::File(f) = blob_info.blob_impl.blob_data() {
                    self.decrement_file_ref(f.get_id());
                }
            });
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
    pub(crate) fn get_blob_bytes(&self, blob_id: &BlobId) -> Result<Vec<u8>, ()> {
        let parent = {
            match *self.get_blob_data(blob_id) {
                BlobData::Sliced(parent, rel_pos) => Some((parent, rel_pos)),
                _ => None,
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

    /// Retrieve information about a specific blob from the blob store
    ///
    /// # Panics
    /// This function panics if there is no blob with the given ID.
    pub(crate) fn get_blob_data<'a>(&'a self, blob_id: &BlobId) -> Ref<'a, BlobData> {
        Ref::map(self.blob_state.borrow(), |blob_state| {
            blob_state
                .get(blob_id)
                .expect("get_blob_impl called for a unknown blob")
                .blob_impl
                .blob_data()
        })
    }

    /// Get bytes from a non-sliced blob
    fn get_blob_bytes_non_sliced(&self, blob_id: &BlobId) -> Result<Vec<u8>, ()> {
        match *self.get_blob_data(blob_id) {
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
    }

    /// Get a slice to the inner data of a Blob,
    /// if it's a memory blob, or it's file-id and file-size otherwise.
    ///
    /// Note: this is almost a duplicate of `get_blob_bytes`,
    /// tweaked for integration with streams.
    /// TODO: merge with `get_blob_bytes` by way of broader integration with blob streams.
    fn get_blob_bytes_or_file_id(&self, blob_id: &BlobId) -> BlobResult {
        let parent = {
            match *self.get_blob_data(blob_id) {
                BlobData::Sliced(parent, rel_pos) => Some((parent, rel_pos)),
                _ => None,
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
        match *self.get_blob_data(blob_id) {
            BlobData::File(ref f) => match f.get_cache() {
                Some(bytes) => BlobResult::Bytes(bytes.clone()),
                None => BlobResult::File(f.get_id(), f.get_size() as usize),
            },
            BlobData::Memory(ref s) => BlobResult::Bytes(s.clone()),
            BlobData::Sliced(_, _) => panic!("This blob doesn't have a parent."),
        }
    }

    /// Get a copy of the type_string of a blob.
    pub(crate) fn get_blob_type_string(&self, blob_id: &BlobId) -> String {
        let blob_state = self.blob_state.borrow();
        let blob_info = blob_state
            .get(blob_id)
            .expect("get_blob_type_string called for a unknown blob.");
        blob_info.blob_impl.type_string()
    }

    /// <https://w3c.github.io/FileAPI/#dfn-size>
    pub(crate) fn get_blob_size(&self, blob_id: &BlobId) -> u64 {
        let parent = {
            match *self.get_blob_data(blob_id) {
                BlobData::Sliced(parent, rel_pos) => Some((parent, rel_pos)),
                _ => None,
            }
        };
        match parent {
            Some((parent_id, rel_pos)) => {
                let parent_size = match *self.get_blob_data(&parent_id) {
                    BlobData::File(ref f) => f.get_size(),
                    BlobData::Memory(ref v) => v.len() as u64,
                    BlobData::Sliced(_, _) => panic!("Blob ancestry should be only one level."),
                };
                rel_pos.to_abs_range(parent_size as usize).len() as u64
            },
            None => match *self.get_blob_data(blob_id) {
                BlobData::File(ref f) => f.get_size(),
                BlobData::Memory(ref v) => v.len() as u64,
                BlobData::Sliced(_, _) => {
                    panic!("It was previously checked that this blob does not have a parent.")
                },
            },
        }
    }

    pub(crate) fn get_blob_url_id(&self, blob_id: &BlobId) -> Uuid {
        let mut blob_state = self.blob_state.borrow_mut();
        let parent = {
            let blob_info = blob_state
                .get_mut(blob_id)
                .expect("get_blob_url_id called for a unknown blob.");

            // Keep track of blobs with outstanding URLs.
            blob_info.has_url = true;

            match blob_info.blob_impl.blob_data() {
                BlobData::Sliced(parent, rel_pos) => Some((*parent, *rel_pos)),
                _ => None,
            }
        };
        match parent {
            Some((parent_id, rel_pos)) => {
                let parent_info = blob_state
                    .get_mut(&parent_id)
                    .expect("Parent of blob whose url is requested is unknown.");
                let parent_file_id = self.promote(parent_info, /* set_valid is */ false);
                let parent_size = match parent_info.blob_impl.blob_data() {
                    BlobData::File(f) => f.get_size(),
                    BlobData::Memory(v) => v.len() as u64,
                    BlobData::Sliced(_, _) => panic!("Blob ancestry should be only one level."),
                };
                let parent_size = rel_pos.to_abs_range(parent_size as usize).len() as u64;
                let blob_info = blob_state
                    .get_mut(blob_id)
                    .expect("Blob whose url is requested is unknown.");
                self.create_sliced_url_id(blob_info, &parent_file_id, &rel_pos, parent_size)
            },
            None => {
                let blob_info = blob_state
                    .get_mut(blob_id)
                    .expect("Blob whose url is requested is unknown.");
                self.promote(blob_info, /* set_valid is */ true)
            },
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
        let msg =
            FileManagerThreadMsg::AddSlicedURLEntry(*parent_file_id, *rel_pos, tx, origin.clone());
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
    ///     Depending on set_valid, the returned FileID can be part of
    ///     valid or invalid Blob URL.
    pub(crate) fn promote(&self, blob_info: &mut BlobInfo, set_valid: bool) -> Uuid {
        let mut bytes = vec![];
        let global_url = self.get_url();

        match blob_info.blob_impl.blob_data_mut() {
            BlobData::Sliced(_, _) => {
                panic!("Sliced blobs should use create_sliced_url_id instead of promote.");
            },
            BlobData::File(f) => {
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
            BlobData::Memory(bytes_in) => mem::swap(bytes_in, &mut bytes),
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
    pub(crate) fn get_blob_stream(
        &self,
        blob_id: &BlobId,
        can_gc: CanGc,
    ) -> Fallible<DomRoot<ReadableStream>> {
        let (file_id, size) = match self.get_blob_bytes_or_file_id(blob_id) {
            BlobResult::Bytes(bytes) => {
                // If we have all the bytes in memory, queue them and close the stream.
                return ReadableStream::new_from_bytes(self, bytes, can_gc);
            },
            BlobResult::File(id, size) => (id, size),
        };

        let stream = ReadableStream::new_with_external_underlying_source(
            self,
            UnderlyingSourceType::Blob(size),
            can_gc,
        )?;

        let recv = self.send_msg(file_id);

        let trusted_stream = Trusted::new(&*stream.clone());
        let mut file_listener = FileListener {
            state: Some(FileListenerState::Empty(FileListenerTarget::Stream(
                trusted_stream,
            ))),
            task_source: self.task_manager().file_reading_task_source().into(),
        };

        ROUTER.add_typed_route(
            recv.to_ipc_receiver(),
            Box::new(move |msg| {
                file_listener.handle(msg.expect("Deserialization of file listener msg failed."));
            }),
        );

        Ok(stream)
    }

    pub(crate) fn read_file_async(
        &self,
        id: Uuid,
        promise: Rc<Promise>,
        callback: FileListenerCallback,
    ) {
        let recv = self.send_msg(id);

        let trusted_promise = TrustedPromise::new(promise);
        let mut file_listener = FileListener {
            state: Some(FileListenerState::Empty(FileListenerTarget::Promise(
                trusted_promise,
                callback,
            ))),
            task_source: self.task_manager().file_reading_task_source().into(),
        };

        ROUTER.add_typed_route(
            recv.to_ipc_receiver(),
            Box::new(move |msg| {
                file_listener.handle(msg.expect("Deserialization of file listener msg failed."));
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

    pub(crate) fn permission_state_invocation_results(
        &self,
    ) -> &DomRefCell<HashMap<PermissionName, PermissionState>> {
        &self.permission_state_invocation_results
    }

    pub(crate) fn track_worker(
        &self,
        closing: Arc<AtomicBool>,
        join_handle: JoinHandle<()>,
        control_sender: Sender<DedicatedWorkerControlMsg>,
        context: ThreadSafeJSContext,
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

    pub(crate) fn track_event_source(&self, event_source: &EventSource) {
        self.event_source_tracker.track(event_source);
    }

    pub(crate) fn close_event_sources(&self) -> bool {
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
    pub(crate) fn from_reflector<T: DomObject>(reflector: &T, _realm: InRealm) -> DomRoot<Self> {
        unsafe { GlobalScope::from_object(*reflector.reflector().get_jsobject()) }
    }

    /// Returns the global scope of the realm that the given JS object was created in.
    #[allow(unsafe_code)]
    pub(crate) unsafe fn from_object(obj: *mut JSObject) -> DomRoot<Self> {
        assert!(!obj.is_null());
        let global = GetNonCCWObjectGlobal(obj);
        global_scope_from_global_static(global)
    }

    /// Returns the global scope for the given JSContext
    #[allow(unsafe_code)]
    pub(crate) unsafe fn from_context(cx: *mut JSContext, _realm: InRealm) -> DomRoot<Self> {
        let global = CurrentGlobalOrNull(cx);
        assert!(!global.is_null());
        global_scope_from_global(global, cx)
    }

    /// Returns the global scope for the given SafeJSContext
    #[allow(unsafe_code)]
    pub(crate) fn from_safe_context(cx: SafeJSContext, realm: InRealm) -> DomRoot<Self> {
        unsafe { Self::from_context(*cx, realm) }
    }

    /// Returns the global object of the realm that the given JS object
    /// was created in, after unwrapping any wrappers.
    #[allow(unsafe_code)]
    pub(crate) unsafe fn from_object_maybe_wrapped(
        mut obj: *mut JSObject,
        cx: *mut JSContext,
    ) -> DomRoot<Self> {
        if IsWrapper(obj) {
            obj = UnwrapObjectDynamic(obj, cx, /* stopAtWindowProxy = */ false);
            assert!(!obj.is_null());
        }
        GlobalScope::from_object(obj)
    }

    pub(crate) fn add_uncaught_rejection(&self, rejection: HandleObject) {
        self.uncaught_rejections
            .borrow_mut()
            .push(Heap::boxed(rejection.get()));
    }

    pub(crate) fn remove_uncaught_rejection(&self, rejection: HandleObject) {
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
    pub(crate) fn get_uncaught_rejections(&self) -> &DomRefCell<Vec<Box<Heap<*mut JSObject>>>> {
        &self.uncaught_rejections
    }

    pub(crate) fn add_consumed_rejection(&self, rejection: HandleObject) {
        self.consumed_rejections
            .borrow_mut()
            .push(Heap::boxed(rejection.get()));
    }

    pub(crate) fn remove_consumed_rejection(&self, rejection: HandleObject) {
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
    pub(crate) fn get_consumed_rejections(&self) -> &DomRefCell<Vec<Box<Heap<*mut JSObject>>>> {
        &self.consumed_rejections
    }

    pub(crate) fn set_module_map(&self, url: ServoUrl, module: ModuleTree) {
        self.module_map.borrow_mut().insert(url, Rc::new(module));
    }

    pub(crate) fn get_module_map(
        &self,
    ) -> &DomRefCell<HashMapTracedValues<ServoUrl, Rc<ModuleTree>>> {
        &self.module_map
    }

    pub(crate) fn set_inline_module_map(&self, script_id: ScriptId, module: ModuleTree) {
        self.inline_module_map
            .borrow_mut()
            .insert(script_id, Rc::new(module));
    }

    pub(crate) fn get_inline_module_map(&self) -> &DomRefCell<HashMap<ScriptId, Rc<ModuleTree>>> {
        &self.inline_module_map
    }

    #[allow(unsafe_code)]
    pub(crate) fn get_cx() -> SafeJSContext {
        let cx = Runtime::get()
            .expect("Can't obtain context after runtime shutdown")
            .as_ptr();
        unsafe { SafeJSContext::from_ptr(cx) }
    }

    pub(crate) fn crypto(&self, can_gc: CanGc) -> DomRoot<Crypto> {
        self.crypto.or_init(|| Crypto::new(self, can_gc))
    }

    pub(crate) fn live_devtools_updates(&self) -> bool {
        self.devtools_wants_updates.get()
    }

    pub(crate) fn set_devtools_wants_updates(&self, value: bool) {
        self.devtools_wants_updates.set(value);
    }

    pub(crate) fn time(&self, label: DOMString) -> Result<(), ()> {
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

    /// Computes the delta time since a label has been created
    ///
    /// Returns an error if the label does not exist.
    pub(crate) fn time_log(&self, label: &str) -> Result<u64, ()> {
        self.console_timers
            .borrow()
            .get(label)
            .ok_or(())
            .map(|&start| (Instant::now() - start).as_millis() as u64)
    }

    /// Computes the delta time since a label has been created and stops
    /// tracking the label.
    ///
    /// Returns an error if the label does not exist.
    pub(crate) fn time_end(&self, label: &str) -> Result<u64, ()> {
        self.console_timers
            .borrow_mut()
            .remove(label)
            .ok_or(())
            .map(|start| (Instant::now() - start).as_millis() as u64)
    }

    /// Get an `&IpcSender<ScriptToDevtoolsControlMsg>` to send messages
    /// to the devtools thread when available.
    pub(crate) fn devtools_chan(&self) -> Option<&IpcSender<ScriptToDevtoolsControlMsg>> {
        self.devtools_chan.as_ref()
    }

    pub(crate) fn issue_page_warning(&self, warning: &str) {
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
    pub(crate) fn mem_profiler_chan(&self) -> &profile_mem::ProfilerChan {
        &self.mem_profiler_chan
    }

    /// Get a sender to the time profiler thread.
    pub(crate) fn time_profiler_chan(&self) -> &profile_time::ProfilerChan {
        &self.time_profiler_chan
    }

    /// Get a sender to the constellation thread.
    pub(crate) fn script_to_constellation_chan(&self) -> &ScriptToConstellationChan {
        &self.script_to_constellation_chan
    }

    pub(crate) fn send_to_embedder(&self, msg: EmbedderMsg) {
        self.send_to_constellation(ScriptToConstellationMessage::ForwardToEmbedder(msg));
    }

    pub(crate) fn send_to_constellation(&self, msg: ScriptToConstellationMessage) {
        self.script_to_constellation_chan().send(msg).unwrap();
    }

    /// Get the `PipelineId` for this global scope.
    pub(crate) fn pipeline_id(&self) -> PipelineId {
        self.pipeline_id
    }

    /// Get the origin for this global scope
    pub(crate) fn origin(&self) -> &MutableOrigin {
        &self.origin
    }

    /// Get the creation_url for this global scope
    pub(crate) fn creation_url(&self) -> &Option<ServoUrl> {
        &self.creation_url
    }

    pub(crate) fn image_cache(&self) -> Arc<dyn ImageCache> {
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

    /// Schedule a [`TimerEventRequest`] on this [`GlobalScope`]'s [`timers::TimerScheduler`].
    /// Every Worker has its own scheduler, which handles events in the Worker event loop,
    /// but `Window`s use a shared scheduler associated with their [`ScriptThread`].
    pub(crate) fn schedule_timer(&self, request: TimerEventRequest) {
        match self.downcast::<WorkerGlobalScope>() {
            Some(worker_global) => worker_global.timer_scheduler().schedule_timer(request),
            _ => with_script_thread(|script_thread| script_thread.schedule_timer(request)),
        }
    }

    /// <https://html.spec.whatwg.org/multipage/#concept-settings-object-policy-container>
    pub(crate) fn policy_container(&self) -> PolicyContainer {
        if let Some(window) = self.downcast::<Window>() {
            return window.Document().policy_container().to_owned();
        }
        if let Some(worker) = self.downcast::<WorkerGlobalScope>() {
            return worker.policy_container().to_owned();
        }
        unreachable!();
    }

    /// <https://www.w3.org/TR/CSP/#initialize-document-csp>
    pub(crate) fn parse_csp_list_from_metadata(
        headers: &Option<Serde<HeaderMap>>,
    ) -> Option<CspList> {
        // TODO: Implement step 1 (local scheme special case)
        let headers = headers.as_ref()?;
        let mut csp = headers.get_all("content-security-policy").iter();
        // This silently ignores the CSP if it contains invalid Unicode.
        // We should probably report an error somewhere.
        let c = csp.next().and_then(|c| c.to_str().ok())?;
        let mut csp_list = CspList::parse(c, PolicySource::Header, PolicyDisposition::Enforce);
        for c in csp {
            let c = c.to_str().ok()?;
            csp_list.append(CspList::parse(
                c,
                PolicySource::Header,
                PolicyDisposition::Enforce,
            ));
        }
        let csp_report = headers
            .get_all("content-security-policy-report-only")
            .iter();
        // This silently ignores the CSP if it contains invalid Unicode.
        // We should probably report an error somewhere.
        for c in csp_report {
            let c = c.to_str().ok()?;
            csp_list.append(CspList::parse(
                c,
                PolicySource::Header,
                PolicyDisposition::Report,
            ));
        }
        Some(csp_list)
    }

    /// Get the [base url](https://html.spec.whatwg.org/multipage/#api-base-url)
    /// for this global scope.
    pub(crate) fn api_base_url(&self) -> ServoUrl {
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
    pub(crate) fn get_url(&self) -> ServoUrl {
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

    /// Get the Referrer Policy for this global scope.
    pub(crate) fn get_referrer_policy(&self) -> ReferrerPolicy {
        if let Some(window) = self.downcast::<Window>() {
            let document = window.Document();

            return document.get_referrer_policy();
        }
        if let Some(worker) = self.downcast::<WorkerGlobalScope>() {
            let policy_container = worker.policy_container().to_owned();

            return policy_container.get_referrer_policy();
        }
        unreachable!();
    }

    /// Determine the Referrer for a request whose Referrer is "client"
    pub(crate) fn get_referrer(&self) -> Referrer {
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
                // Return early if we cannot get a parent document. This might happen if
                // this iframe was already removed from the parent page.
                let Some(parent_document) =
                    document.browsing_context().and_then(|browsing_context| {
                        browsing_context
                            .parent()
                            .and_then(|parent| parent.document())
                    })
                else {
                    return Referrer::NoReferrer;
                };
                document = parent_document;
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
    pub(crate) fn as_window(&self) -> &Window {
        self.downcast::<Window>().expect("expected a Window scope")
    }

    /// Returns a policy that should be used for fetches initiated from this global.
    pub(crate) fn insecure_requests_policy(&self) -> InsecureRequestsPolicy {
        if let Some(window) = self.downcast::<Window>() {
            return window.Document().insecure_requests_policy();
        }
        if let Some(worker) = self.downcast::<WorkerGlobalScope>() {
            return worker.insecure_requests_policy();
        }
        debug!("unsupported global, defaulting insecure requests policy to DoNotUpgrade");
        InsecureRequestsPolicy::DoNotUpgrade
    }

    /// Whether this document has ancestor navigables that are trustworthy
    pub(crate) fn has_trustworthy_ancestor_origin(&self) -> bool {
        self.downcast::<Window>()
            .is_some_and(|window| window.Document().has_trustworthy_ancestor_origin())
    }

    // Whether this document has a trustworthy origin or has trustowrthy ancestor navigables
    pub(crate) fn has_trustworthy_ancestor_or_current_origin(&self) -> bool {
        self.downcast::<Window>().is_some_and(|window| {
            window
                .Document()
                .has_trustworthy_ancestor_or_current_origin()
        })
    }

    /// <https://html.spec.whatwg.org/multipage/#report-the-error>
    pub(crate) fn report_an_error(&self, error_info: ErrorInfo, value: HandleValue, can_gc: CanGc) {
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
            can_gc,
        );

        // Step 7.
        let event_status = event
            .upcast::<Event>()
            .fire(self.upcast::<EventTarget>(), can_gc);

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
    pub(crate) fn resource_threads(&self) -> &ResourceThreads {
        &self.resource_threads
    }

    /// Get the `CoreResourceThread` for this global scope.
    pub(crate) fn core_resource_thread(&self) -> CoreResourceThread {
        self.resource_threads().sender()
    }

    /// A sender to the event loop of this global scope. This either sends to the Worker event loop
    /// or the ScriptThread event loop in the case of a `Window`. This can be `None` for dedicated
    /// workers that are not currently handling a message.
    pub(crate) fn event_loop_sender(&self) -> Option<ScriptEventLoopSender> {
        if let Some(window) = self.downcast::<Window>() {
            Some(window.event_loop_sender())
        } else if let Some(dedicated) = self.downcast::<DedicatedWorkerGlobalScope>() {
            dedicated.event_loop_sender()
        } else if let Some(service_worker) = self.downcast::<ServiceWorkerGlobalScope>() {
            Some(service_worker.event_loop_sender())
        } else {
            unreachable!(
                "Tried to access event loop sender for incompatible \
                 GlobalScope (PaintWorklet or DissimilarOriginWindow)"
            );
        }
    }

    /// A reference to the [`TaskManager`] used to schedule tasks for this [`GlobalScope`].
    pub(crate) fn task_manager(&self) -> &TaskManager {
        let shared_canceller = self
            .downcast::<WorkerGlobalScope>()
            .map(WorkerGlobalScope::shared_task_canceller);
        self.task_manager.get_or_init(|| {
            TaskManager::new(
                self.event_loop_sender(),
                self.pipeline_id(),
                shared_canceller,
            )
        })
    }

    /// Evaluate JS code on this global scope.
    pub(crate) fn evaluate_js_on_global_with_result(
        &self,
        code: &str,
        rval: MutableHandleValue,
        fetch_options: ScriptFetchOptions,
        script_base_url: ServoUrl,
        can_gc: CanGc,
    ) -> bool {
        let source_code = SourceCode::Text(Rc::new(DOMString::from_string((*code).to_string())));
        self.evaluate_script_on_global_with_result(
            &source_code,
            "",
            rval,
            1,
            fetch_options,
            script_base_url,
            can_gc,
        )
    }

    /// Evaluate a JS script on this global scope.
    #[allow(unsafe_code)]
    #[allow(clippy::too_many_arguments)]
    pub(crate) fn evaluate_script_on_global_with_result(
        &self,
        code: &SourceCode,
        filename: &str,
        rval: MutableHandleValue,
        line_number: u32,
        fetch_options: ScriptFetchOptions,
        script_base_url: ServoUrl,
        can_gc: CanGc,
    ) -> bool {
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
                        report_pending_exception(cx, true, InRealm::Entered(&ar), can_gc);
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
                report_pending_exception(cx, true, InRealm::Entered(&ar), can_gc);
            }

            maybe_resume_unwind();
            result
        }
    }

    /// <https://html.spec.whatwg.org/multipage/#timer-initialisation-steps>
    pub(crate) fn schedule_callback(
        &self,
        callback: OneshotTimerCallback,
        duration: Duration,
    ) -> OneshotTimerHandle {
        self.timers()
            .schedule_callback(callback, duration, self.timer_source())
    }

    pub(crate) fn unschedule_callback(&self, handle: OneshotTimerHandle) {
        self.timers().unschedule_callback(handle);
    }

    /// <https://html.spec.whatwg.org/multipage/#timer-initialisation-steps>
    pub(crate) fn set_timeout_or_interval(
        &self,
        callback: TimerCallback,
        arguments: Vec<HandleValue>,
        timeout: Duration,
        is_interval: IsInterval,
    ) -> i32 {
        self.timers().set_timeout_or_interval(
            self,
            callback,
            arguments,
            timeout,
            is_interval,
            self.timer_source(),
        )
    }

    pub(crate) fn clear_timeout_or_interval(&self, handle: i32) {
        self.timers().clear_timeout_or_interval(self, handle);
    }

    pub(crate) fn queue_function_as_microtask(&self, callback: Rc<VoidFunction>) {
        self.enqueue_microtask(Microtask::User(UserMicrotask {
            callback,
            pipeline: self.pipeline_id(),
        }))
    }

    pub(crate) fn is_js_evaluation_allowed(&self, source: &str) -> bool {
        let Some(csp_list) = self.get_csp_list() else {
            return true;
        };

        let (is_js_evaluation_allowed, violations) = csp_list.is_js_evaluation_allowed(source);

        self.report_csp_violations(violations, None);

        is_js_evaluation_allowed == CheckResult::Allowed
    }

    pub(crate) fn should_navigation_request_be_blocked(&self, load_data: &LoadData) -> bool {
        let Some(csp_list) = self.get_csp_list() else {
            return false;
        };
        let request = Request {
            url: load_data.url.clone().into_url(),
            origin: match &load_data.load_origin {
                LoadOrigin::Script(immutable_origin) => immutable_origin.clone().into_url_origin(),
                _ => Origin::new_opaque(),
            },
            // TODO: populate this field correctly
            redirect_count: 0,
            destination: Destination::None,
            initiator: Initiator::None,
            nonce: "".to_owned(),
            integrity_metadata: "".to_owned(),
            parser_metadata: ParserMetadata::None,
        };
        // TODO: set correct navigation check type for form submission if applicable
        let (result, violations) =
            csp_list.should_navigation_request_be_blocked(&request, NavigationCheckType::Other);

        self.report_csp_violations(violations, None);

        result == CheckResult::Blocked
    }

    pub(crate) fn create_image_bitmap(
        &self,
        image: ImageBitmapSource,
        options: &ImageBitmapOptions,
        can_gc: CanGc,
    ) -> Rc<Promise> {
        let in_realm_proof = AlreadyInRealm::assert::<crate::DomTypeHolder>();
        let p = Promise::new_in_current_realm(InRealm::Already(&in_realm_proof), can_gc);
        if options.resizeWidth.is_some_and(|w| w == 0) {
            p.reject_error(Error::InvalidState, can_gc);
            return p;
        }

        if options.resizeHeight.is_some_and(|w| w == 0) {
            p.reject_error(Error::InvalidState, can_gc);
            return p;
        }

        match image {
            ImageBitmapSource::HTMLCanvasElement(ref canvas) => {
                // https://html.spec.whatwg.org/multipage/#check-the-usability-of-the-image-argument
                if !canvas.is_valid() {
                    p.reject_error(Error::InvalidState, can_gc);
                    return p;
                }

                if let Some(snapshot) = canvas.get_image_data() {
                    let size = snapshot.size().cast();
                    let image_bitmap =
                        ImageBitmap::new(self, size.width, size.height, can_gc).unwrap();
                    image_bitmap.set_bitmap_data(snapshot.to_vec());
                    image_bitmap.set_origin_clean(canvas.origin_is_clean());
                    p.resolve_native(&(image_bitmap), can_gc);
                }
                p
            },
            ImageBitmapSource::OffscreenCanvas(ref canvas) => {
                // https://html.spec.whatwg.org/multipage/#check-the-usability-of-the-image-argument
                if !canvas.is_valid() {
                    p.reject_error(Error::InvalidState, can_gc);
                    return p;
                }

                if let Some(snapshot) = canvas.get_image_data() {
                    let size = snapshot.size().cast();
                    let image_bitmap =
                        ImageBitmap::new(self, size.width, size.height, can_gc).unwrap();
                    image_bitmap.set_bitmap_data(snapshot.to_vec());
                    image_bitmap.set_origin_clean(canvas.origin_is_clean());
                    p.resolve_native(&(image_bitmap), can_gc);
                }
                p
            },
            _ => {
                p.reject_error(Error::NotSupported, can_gc);
                p
            },
        }
    }

    pub(crate) fn fire_timer(&self, handle: TimerEventId, can_gc: CanGc) {
        self.timers().fire_timer(handle, self, can_gc);
    }

    pub(crate) fn resume(&self) {
        self.timers().resume();
    }

    pub(crate) fn suspend(&self) {
        self.timers().suspend();
    }

    pub(crate) fn slow_down_timers(&self) {
        self.timers().slow_down();
    }

    pub(crate) fn speed_up_timers(&self) {
        self.timers().speed_up();
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
    pub(crate) fn can_continue_running(&self) -> bool {
        if self.is::<Window>() {
            return ScriptThread::can_continue_running();
        }
        if let Some(worker) = self.downcast::<WorkerGlobalScope>() {
            return !worker.is_closing();
        }

        // TODO: plug worklets into this.
        true
    }

    /// Perform a microtask checkpoint.
    pub(crate) fn perform_a_microtask_checkpoint(&self, can_gc: CanGc) {
        // Only perform the checkpoint if we're not shutting down.
        if self.can_continue_running() {
            self.microtask_queue.checkpoint(
                GlobalScope::get_cx(),
                |_| Some(DomRoot::from_ref(self)),
                vec![DomRoot::from_ref(self)],
                can_gc,
            );
        }
    }

    /// Enqueue a microtask for subsequent execution.
    pub(crate) fn enqueue_microtask(&self, job: Microtask) {
        self.microtask_queue.enqueue(job, GlobalScope::get_cx());
    }

    /// Create a new sender/receiver pair that can be used to implement an on-demand
    /// event loop. Used for implementing web APIs that require blocking semantics
    /// without resorting to nested event loops.
    pub(crate) fn new_script_pair(&self) -> (ScriptEventLoopSender, ScriptEventLoopReceiver) {
        if let Some(window) = self.downcast::<Window>() {
            return window.new_script_pair();
        }
        if let Some(worker) = self.downcast::<WorkerGlobalScope>() {
            return worker.new_script_pair();
        }
        unreachable!();
    }

    /// Returns the microtask queue of this global.
    pub(crate) fn microtask_queue(&self) -> &Rc<MicrotaskQueue> {
        &self.microtask_queue
    }

    /// Process a single event as if it were the next event
    /// in the queue for the event-loop where this global scope is running on.
    /// Returns a boolean indicating whether further events should be processed.
    pub(crate) fn process_event(&self, msg: CommonScriptMsg) -> bool {
        if self.is::<Window>() {
            return ScriptThread::process_event(msg);
        }
        if let Some(worker) = self.downcast::<WorkerGlobalScope>() {
            return worker.process_event(msg);
        }
        unreachable!();
    }

    pub(crate) fn runtime_handle(&self) -> ParentRuntime {
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
    pub(crate) fn current() -> Option<DomRoot<Self>> {
        let cx = Runtime::get()?;
        unsafe {
            let global = CurrentGlobalOrNull(cx.as_ptr());
            if global.is_null() {
                None
            } else {
                Some(global_scope_from_global(global, cx.as_ptr()))
            }
        }
    }

    /// Returns the ["entry"] global object.
    ///
    /// ["entry"]: https://html.spec.whatwg.org/multipage/#entry
    pub(crate) fn entry() -> DomRoot<Self> {
        entry_global()
    }

    /// Returns the ["incumbent"] global object.
    ///
    /// ["incumbent"]: https://html.spec.whatwg.org/multipage/#incumbent
    pub(crate) fn incumbent() -> Option<DomRoot<Self>> {
        incumbent_global()
    }

    pub(crate) fn performance(&self) -> DomRoot<Performance> {
        if let Some(window) = self.downcast::<Window>() {
            return window.Performance();
        }
        if let Some(worker) = self.downcast::<WorkerGlobalScope>() {
            return worker.Performance();
        }
        unreachable!();
    }

    /// <https://w3c.github.io/performance-timeline/#supportedentrytypes-attribute>
    pub(crate) fn supported_performance_entry_types(
        &self,
        cx: SafeJSContext,
        retval: MutableHandleValue,
        can_gc: CanGc,
    ) {
        self.frozen_supported_performance_entry_types.get_or_init(
            || {
                VALID_ENTRY_TYPES
                    .iter()
                    .map(|t| DOMString::from(t.to_string()))
                    .collect()
            },
            cx,
            retval,
            can_gc,
        );
    }

    pub(crate) fn get_https_state(&self) -> HttpsState {
        self.https_state.get()
    }

    pub(crate) fn set_https_state(&self, https_state: HttpsState) {
        self.https_state.set(https_state);
    }

    /// <https://html.spec.whatwg.org/multipage/#secure-context>
    pub(crate) fn is_secure_context(&self) -> bool {
        // This differs from the specification, but it seems that
        // `inherited_secure_context` implements more-or-less the exact same logic, in a
        // different manner. Workers inherit whether or not their in a secure context and
        // worklets do as well (they can only be created in secure contexts).
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
    pub(crate) fn get_csp_list(&self) -> Option<CspList> {
        if self.downcast::<Window>().is_some() || self.downcast::<WorkerGlobalScope>().is_some() {
            return self.policy_container().csp_list;
        }
        // TODO: Worklet global scopes.
        None
    }

    pub(crate) fn status_code(&self) -> Option<u16> {
        if let Some(window) = self.downcast::<Window>() {
            return window.Document().status_code();
        }
        None
    }

    #[cfg(feature = "webgpu")]
    pub(crate) fn wgpu_id_hub(&self) -> Arc<IdentityHub> {
        self.gpu_id_hub.clone()
    }

    #[cfg(feature = "webgpu")]
    pub(crate) fn add_gpu_device(&self, device: &GPUDevice) {
        self.gpu_devices
            .borrow_mut()
            .insert(device.id(), WeakRef::new(device));
    }

    #[cfg(feature = "webgpu")]
    pub(crate) fn remove_gpu_device(&self, device: WebGPUDevice) {
        let device = self
            .gpu_devices
            .borrow_mut()
            .remove(&device)
            .expect("GPUDevice should still be in devices hashmap");
        assert!(device.root().is_none())
    }

    #[cfg(feature = "webgpu")]
    pub(crate) fn gpu_device_lost(
        &self,
        device: WebGPUDevice,
        reason: DeviceLostReason,
        msg: String,
        can_gc: CanGc,
    ) {
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
            device.lose(reason, msg, can_gc);
        }
    }

    #[cfg(feature = "webgpu")]
    pub(crate) fn handle_uncaptured_gpu_error(
        &self,
        device: WebGPUDevice,
        error: webgpu_traits::Error,
        can_gc: CanGc,
    ) {
        if let Some(gpu_device) = self
            .gpu_devices
            .borrow()
            .get(&device)
            .and_then(|device| device.root())
        {
            gpu_device.fire_uncaptured_error(error, can_gc);
        } else {
            warn!("Recived error for lost GPUDevice!")
        }
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

    pub(crate) fn structured_clone(
        &self,
        cx: SafeJSContext,
        value: HandleValue,
        options: RootedTraceableBox<StructuredSerializeOptions>,
        retval: MutableHandleValue,
    ) -> Fallible<()> {
        let mut rooted = CustomAutoRooter::new(
            options
                .transfer
                .iter()
                .map(|js: &RootedTraceableBox<Heap<*mut JSObject>>| js.get())
                .collect(),
        );
        let guard = CustomAutoRooterGuard::new(*cx, &mut rooted);

        let data = structuredclone::write(cx, value, Some(guard))?;

        structuredclone::read(self, data, retval)?;

        Ok(())
    }

    pub(crate) fn fetch<Listener: FetchResponseListener + PreInvoke + Send + 'static>(
        &self,
        request_builder: RequestBuilder,
        context: Arc<Mutex<Listener>>,
        task_source: SendableTaskSource,
    ) {
        let network_listener = NetworkListener {
            context,
            task_source,
        };
        self.fetch_with_network_listener(request_builder, network_listener);
    }

    pub(crate) fn fetch_with_network_listener<
        Listener: FetchResponseListener + PreInvoke + Send + 'static,
    >(
        &self,
        request_builder: RequestBuilder,
        network_listener: NetworkListener<Listener>,
    ) {
        fetch_async(
            &self.core_resource_thread(),
            request_builder,
            None,
            network_listener.into_callback(),
        );
    }

    pub(crate) fn unminified_js_dir(&self) -> Option<String> {
        self.unminified_js_dir.clone()
    }

    pub(crate) fn set_byte_length_queuing_strategy_size(&self, function: Rc<Function>) {
        if self
            .byte_length_queuing_strategy_size_function
            .set(function)
            .is_err()
        {
            warn!("byte length queuing strategy size function is set twice.");
        };
    }

    pub(crate) fn get_byte_length_queuing_strategy_size(&self) -> Option<Rc<Function>> {
        self.byte_length_queuing_strategy_size_function
            .get()
            .cloned()
    }

    pub(crate) fn set_count_queuing_strategy_size(&self, function: Rc<Function>) {
        if self
            .count_queuing_strategy_size_function
            .set(function)
            .is_err()
        {
            warn!("count queuing strategy size function is set twice.");
        };
    }

    pub(crate) fn get_count_queuing_strategy_size(&self) -> Option<Rc<Function>> {
        self.count_queuing_strategy_size_function.get().cloned()
    }

    pub(crate) fn add_notification_permission_request_callback(
        &self,
        callback_id: String,
        callback: Rc<NotificationPermissionCallback>,
    ) {
        self.notification_permission_request_callback_map
            .borrow_mut()
            .insert(callback_id, callback.clone());
    }

    pub(crate) fn remove_notification_permission_request_callback(
        &self,
        callback_id: String,
    ) -> Option<Rc<NotificationPermissionCallback>> {
        self.notification_permission_request_callback_map
            .borrow_mut()
            .remove(&callback_id)
    }

    pub(crate) fn trusted_types(&self, can_gc: CanGc) -> DomRoot<TrustedTypePolicyFactory> {
        if let Some(window) = self.downcast::<Window>() {
            return window.TrustedTypes(can_gc);
        }
        if let Some(worker) = self.downcast::<WorkerGlobalScope>() {
            return worker.TrustedTypes(can_gc);
        }
        unreachable!();
    }

    /// <https://www.w3.org/TR/CSP/#report-violation>
    #[allow(unsafe_code)]
    pub(crate) fn report_csp_violations(
        &self,
        violations: Vec<Violation>,
        element: Option<&Element>,
    ) {
        let scripted_caller =
            unsafe { describe_scripted_caller(*GlobalScope::get_cx()) }.unwrap_or_default();
        for violation in violations {
            let (sample, resource) = match violation.resource {
                ViolationResource::Inline { sample } => (sample, "inline".to_owned()),
                ViolationResource::Url(url) => (None, url.into()),
                ViolationResource::TrustedTypePolicy { sample } => {
                    (Some(sample), "trusted-types-policy".to_owned())
                },
                ViolationResource::TrustedTypeSink { sample } => {
                    (Some(sample), "trusted-types-sink".to_owned())
                },
                ViolationResource::Eval { sample } => (sample, "eval".to_owned()),
                ViolationResource::WasmEval => (None, "wasm-eval".to_owned()),
            };
            let report = CSPViolationReportBuilder::default()
                .resource(resource)
                .sample(sample)
                .effective_directive(violation.directive.name)
                .original_policy(violation.policy.to_string())
                .report_only(violation.policy.disposition == PolicyDisposition::Report)
                .source_file(scripted_caller.filename.clone())
                .line_number(scripted_caller.line)
                .column_number(scripted_caller.col + 1)
                .build(self);
            // Step 1: Let global be violation’s global object.
            // We use `self` as `global`;
            // Step 2: Let target be violation’s element.
            let target = element.and_then(|event_target| {
                // Step 3.1: If target is not null, and global is a Window,
                // and target’s shadow-including root is not global’s associated Document, set target to null.
                if let Some(window) = self.downcast::<Window>() {
                    if !window
                        .Document()
                        .upcast::<Node>()
                        .is_shadow_including_inclusive_ancestor_of(event_target.upcast())
                    {
                        return None;
                    }
                }
                Some(event_target)
            });
            let target = match target {
                // Step 3.2: If target is null:
                None => {
                    // Step 3.2.2: If target is a Window, set target to target’s associated Document.
                    if let Some(window) = self.downcast::<Window>() {
                        Trusted::new(window.Document().upcast())
                    } else {
                        // Step 3.2.1: Set target to violation’s global object.
                        Trusted::new(self.upcast())
                    }
                },
                Some(event_target) => Trusted::new(event_target.upcast()),
            };
            // Step 3: Queue a task to run the following steps:
            let task = CSPViolationReportTask::new(Trusted::new(self), target, report);
            self.task_manager()
                .dom_manipulation_task_source()
                .queue(task);
        }
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

#[allow(unsafe_code)]
impl GlobalScopeHelpers<crate::DomTypeHolder> for GlobalScope {
    unsafe fn from_context(cx: *mut JSContext, realm: InRealm) -> DomRoot<Self> {
        GlobalScope::from_context(cx, realm)
    }

    fn get_cx() -> SafeJSContext {
        GlobalScope::get_cx()
    }

    unsafe fn from_object(obj: *mut JSObject) -> DomRoot<Self> {
        GlobalScope::from_object(obj)
    }

    fn from_reflector(reflector: &impl DomObject, realm: InRealm) -> DomRoot<Self> {
        GlobalScope::from_reflector(reflector, realm)
    }

    fn origin(&self) -> &MutableOrigin {
        GlobalScope::origin(self)
    }

    fn incumbent() -> Option<DomRoot<Self>> {
        GlobalScope::incumbent()
    }

    fn perform_a_microtask_checkpoint(&self, can_gc: CanGc) {
        GlobalScope::perform_a_microtask_checkpoint(self, can_gc)
    }

    fn get_url(&self) -> ServoUrl {
        self.get_url()
    }

    fn is_secure_context(&self) -> bool {
        self.is_secure_context()
    }
}
