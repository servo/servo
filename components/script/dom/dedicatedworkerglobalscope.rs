/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use std::sync::Arc;
use std::sync::atomic::AtomicBool;
use std::thread::{self, JoinHandle};

use base::id::{BrowsingContextId, PipelineId, WebViewId};
use constellation_traits::{WorkerGlobalScopeInit, WorkerScriptLoadOrigin};
use crossbeam_channel::{Receiver, Sender, unbounded};
use devtools_traits::DevtoolScriptControlMsg;
use dom_struct::dom_struct;
use headers::{HeaderMapExt, ReferrerPolicy as ReferrerPolicyHeader};
use ipc_channel::ipc::IpcReceiver;
use ipc_channel::router::ROUTER;
use js::jsapi::{Heap, JS_AddInterruptCallback, JSContext, JSObject};
use js::jsval::UndefinedValue;
use js::rust::{CustomAutoRooter, CustomAutoRooterGuard, HandleValue};
use net_traits::image_cache::ImageCache;
use net_traits::policy_container::PolicyContainer;
use net_traits::request::{
    CredentialsMode, Destination, InsecureRequestsPolicy, ParserMetadata, Referrer, RequestBuilder,
    RequestMode,
};
use net_traits::{IpcSend, Metadata};
use servo_rand::random;
use servo_url::{ImmutableOrigin, ServoUrl};
use style::thread_state::{self, ThreadState};

use crate::devtools;
use crate::dom::abstractworker::{SimpleWorkerErrorHandler, WorkerScriptMsg};
use crate::dom::abstractworkerglobalscope::{WorkerEventLoopMethods, run_worker_event_loop};
use crate::dom::bindings::cell::DomRefCell;
use crate::dom::bindings::codegen::Bindings::DedicatedWorkerGlobalScopeBinding;
use crate::dom::bindings::codegen::Bindings::DedicatedWorkerGlobalScopeBinding::DedicatedWorkerGlobalScopeMethods;
use crate::dom::bindings::codegen::Bindings::MessagePortBinding::StructuredSerializeOptions;
use crate::dom::bindings::codegen::Bindings::WorkerBinding::WorkerType;
use crate::dom::bindings::error::{ErrorInfo, ErrorResult};
use crate::dom::bindings::inheritance::Castable;
use crate::dom::bindings::reflector::DomGlobal;
use crate::dom::bindings::root::{DomRoot, RootCollection, ThreadLocalStackRoots};
use crate::dom::bindings::str::DOMString;
use crate::dom::bindings::structuredclone;
use crate::dom::bindings::trace::{CustomTraceable, RootedTraceableBox};
use crate::dom::bindings::utils::define_all_exposed_interfaces;
use crate::dom::csp::{Violation, parse_csp_list_from_metadata};
use crate::dom::errorevent::ErrorEvent;
use crate::dom::event::{Event, EventBubbles, EventCancelable, EventStatus};
use crate::dom::eventtarget::EventTarget;
use crate::dom::globalscope::GlobalScope;
use crate::dom::messageevent::MessageEvent;
use crate::dom::reportingendpoint::ReportingEndpoint;
#[cfg(feature = "webgpu")]
use crate::dom::webgpu::identityhub::IdentityHub;
use crate::dom::worker::{TrustedWorkerAddress, Worker};
use crate::dom::workerglobalscope::WorkerGlobalScope;
use crate::fetch::{CspViolationsProcessor, load_whole_resource};
use crate::messaging::{CommonScriptMsg, ScriptEventLoopReceiver, ScriptEventLoopSender};
use crate::realms::{AlreadyInRealm, InRealm, enter_realm};
use crate::script_runtime::ScriptThreadEventCategory::WorkerEvent;
use crate::script_runtime::{CanGc, JSContext as SafeJSContext, Runtime, ThreadSafeJSContext};
use crate::task_queue::{QueuedTask, QueuedTaskConversion, TaskQueue};
use crate::task_source::{SendableTaskSource, TaskSourceName};

/// Set the `worker` field of a related DedicatedWorkerGlobalScope object to a particular
/// value for the duration of this object's lifetime. This ensures that the related Worker
/// object only lives as long as necessary (ie. while events are being executed), while
/// providing a reference that can be cloned freely.
pub(crate) struct AutoWorkerReset<'a> {
    workerscope: &'a DedicatedWorkerGlobalScope,
    old_worker: Option<TrustedWorkerAddress>,
}

impl<'a> AutoWorkerReset<'a> {
    fn new(
        workerscope: &'a DedicatedWorkerGlobalScope,
        worker: TrustedWorkerAddress,
    ) -> AutoWorkerReset<'a> {
        let old_worker = workerscope.replace_worker(Some(worker));
        AutoWorkerReset {
            workerscope,
            old_worker,
        }
    }
}

impl Drop for AutoWorkerReset<'_> {
    fn drop(&mut self) {
        self.workerscope
            .replace_worker(std::mem::take(&mut self.old_worker));
    }
}

/// Messages sent from the owning global.
pub(crate) enum DedicatedWorkerControlMsg {
    /// Shutdown the worker.
    Exit,
}

pub(crate) enum DedicatedWorkerScriptMsg {
    /// Standard message from a worker.
    CommonWorker(TrustedWorkerAddress, WorkerScriptMsg),
    /// Wake-up call from the task queue.
    WakeUp,
}

pub(crate) enum MixedMessage {
    Worker(DedicatedWorkerScriptMsg),
    Devtools(DevtoolScriptControlMsg),
    Control(DedicatedWorkerControlMsg),
    Timer,
}

impl QueuedTaskConversion for DedicatedWorkerScriptMsg {
    fn task_source_name(&self) -> Option<&TaskSourceName> {
        let common_worker_msg = match self {
            DedicatedWorkerScriptMsg::CommonWorker(_, common_worker_msg) => common_worker_msg,
            _ => return None,
        };
        let script_msg = match common_worker_msg {
            WorkerScriptMsg::Common(script_msg) => script_msg,
            _ => return None,
        };
        match script_msg {
            CommonScriptMsg::Task(_category, _boxed, _pipeline_id, source_name) => {
                Some(source_name)
            },
            _ => None,
        }
    }

    fn pipeline_id(&self) -> Option<PipelineId> {
        // Workers always return None, since the pipeline_id is only used to check for document activity,
        // and this check does not apply to worker event-loops.
        None
    }

    fn into_queued_task(self) -> Option<QueuedTask> {
        let (worker, common_worker_msg) = match self {
            DedicatedWorkerScriptMsg::CommonWorker(worker, common_worker_msg) => {
                (worker, common_worker_msg)
            },
            _ => return None,
        };
        let script_msg = match common_worker_msg {
            WorkerScriptMsg::Common(script_msg) => script_msg,
            _ => return None,
        };
        let (category, boxed, pipeline_id, task_source) = match script_msg {
            CommonScriptMsg::Task(category, boxed, pipeline_id, task_source) => {
                (category, boxed, pipeline_id, task_source)
            },
            _ => return None,
        };
        Some((Some(worker), category, boxed, pipeline_id, task_source))
    }

    fn from_queued_task(queued_task: QueuedTask) -> Self {
        let (worker, category, boxed, pipeline_id, task_source) = queued_task;
        let script_msg = CommonScriptMsg::Task(category, boxed, pipeline_id, task_source);
        DedicatedWorkerScriptMsg::CommonWorker(worker.unwrap(), WorkerScriptMsg::Common(script_msg))
    }

    fn inactive_msg() -> Self {
        // Inactive is only relevant in the context of a browsing-context event-loop.
        panic!("Workers should never receive messages marked as inactive");
    }

    fn wake_up_msg() -> Self {
        DedicatedWorkerScriptMsg::WakeUp
    }

    fn is_wake_up(&self) -> bool {
        matches!(self, DedicatedWorkerScriptMsg::WakeUp)
    }
}

unsafe_no_jsmanaged_fields!(TaskQueue<DedicatedWorkerScriptMsg>);

struct DedicatedWorkerCspProcessor {
    parent_event_loop_sender: ScriptEventLoopSender,
    pipeline_id: PipelineId,
}

impl CspViolationsProcessor for DedicatedWorkerCspProcessor {
    fn process_csp_violations(&self, violations: Vec<Violation>) {
        let _ = self
            .parent_event_loop_sender
            .send(CommonScriptMsg::ReportCspViolations(
                self.pipeline_id,
                violations,
            ));
    }
}

// https://html.spec.whatwg.org/multipage/#dedicatedworkerglobalscope
#[dom_struct]
pub(crate) struct DedicatedWorkerGlobalScope {
    workerglobalscope: WorkerGlobalScope,
    #[ignore_malloc_size_of = "Defined in std"]
    task_queue: TaskQueue<DedicatedWorkerScriptMsg>,
    own_sender: Sender<DedicatedWorkerScriptMsg>,
    #[ignore_malloc_size_of = "Trusted<T> has unclear ownership like Dom<T>"]
    worker: DomRefCell<Option<TrustedWorkerAddress>>,
    #[ignore_malloc_size_of = "Can't measure trait objects"]
    /// Sender to the parent thread.
    parent_event_loop_sender: ScriptEventLoopSender,
    #[ignore_malloc_size_of = "Arc"]
    #[no_trace]
    image_cache: Arc<dyn ImageCache>,
    #[no_trace]
    browsing_context: Option<BrowsingContextId>,
    /// A receiver of control messages,
    /// currently only used to signal shutdown.
    #[ignore_malloc_size_of = "Channels are hard"]
    #[no_trace]
    control_receiver: Receiver<DedicatedWorkerControlMsg>,
}

impl WorkerEventLoopMethods for DedicatedWorkerGlobalScope {
    type WorkerMsg = DedicatedWorkerScriptMsg;
    type ControlMsg = DedicatedWorkerControlMsg;
    type Event = MixedMessage;

    fn task_queue(&self) -> &TaskQueue<DedicatedWorkerScriptMsg> {
        &self.task_queue
    }

    fn handle_event(&self, event: MixedMessage, can_gc: CanGc) -> bool {
        self.handle_mixed_message(event, can_gc)
    }

    fn handle_worker_post_event(&self, worker: &TrustedWorkerAddress) -> Option<AutoWorkerReset> {
        let ar = AutoWorkerReset::new(self, worker.clone());
        Some(ar)
    }

    fn from_control_msg(msg: DedicatedWorkerControlMsg) -> MixedMessage {
        MixedMessage::Control(msg)
    }

    fn from_worker_msg(msg: DedicatedWorkerScriptMsg) -> MixedMessage {
        MixedMessage::Worker(msg)
    }

    fn from_devtools_msg(msg: DevtoolScriptControlMsg) -> MixedMessage {
        MixedMessage::Devtools(msg)
    }

    fn from_timer_msg() -> MixedMessage {
        MixedMessage::Timer
    }

    fn control_receiver(&self) -> &Receiver<DedicatedWorkerControlMsg> {
        &self.control_receiver
    }
}

impl DedicatedWorkerGlobalScope {
    pub(crate) fn webview_id(&self) -> Option<WebViewId> {
        WebViewId::installed()
    }

    #[allow(clippy::too_many_arguments)]
    fn new_inherited(
        init: WorkerGlobalScopeInit,
        worker_name: DOMString,
        worker_type: WorkerType,
        worker_url: ServoUrl,
        from_devtools_receiver: Receiver<DevtoolScriptControlMsg>,
        runtime: Runtime,
        parent_event_loop_sender: ScriptEventLoopSender,
        own_sender: Sender<DedicatedWorkerScriptMsg>,
        receiver: Receiver<DedicatedWorkerScriptMsg>,
        closing: Arc<AtomicBool>,
        image_cache: Arc<dyn ImageCache>,
        browsing_context: Option<BrowsingContextId>,
        #[cfg(feature = "webgpu")] gpu_id_hub: Arc<IdentityHub>,
        control_receiver: Receiver<DedicatedWorkerControlMsg>,
        insecure_requests_policy: InsecureRequestsPolicy,
    ) -> DedicatedWorkerGlobalScope {
        DedicatedWorkerGlobalScope {
            workerglobalscope: WorkerGlobalScope::new_inherited(
                init,
                worker_name,
                worker_type,
                worker_url,
                runtime,
                from_devtools_receiver,
                closing,
                #[cfg(feature = "webgpu")]
                gpu_id_hub,
                insecure_requests_policy,
            ),
            task_queue: TaskQueue::new(receiver, own_sender.clone()),
            own_sender,
            parent_event_loop_sender,
            worker: DomRefCell::new(None),
            image_cache,
            browsing_context,
            control_receiver,
        }
    }

    #[allow(unsafe_code, clippy::too_many_arguments)]
    pub(crate) fn new(
        init: WorkerGlobalScopeInit,
        worker_name: DOMString,
        worker_type: WorkerType,
        worker_url: ServoUrl,
        from_devtools_receiver: Receiver<DevtoolScriptControlMsg>,
        runtime: Runtime,
        parent_event_loop_sender: ScriptEventLoopSender,
        own_sender: Sender<DedicatedWorkerScriptMsg>,
        receiver: Receiver<DedicatedWorkerScriptMsg>,
        closing: Arc<AtomicBool>,
        image_cache: Arc<dyn ImageCache>,
        browsing_context: Option<BrowsingContextId>,
        #[cfg(feature = "webgpu")] gpu_id_hub: Arc<IdentityHub>,
        control_receiver: Receiver<DedicatedWorkerControlMsg>,
        insecure_requests_policy: InsecureRequestsPolicy,
    ) -> DomRoot<DedicatedWorkerGlobalScope> {
        let cx = runtime.cx();
        let scope = Box::new(DedicatedWorkerGlobalScope::new_inherited(
            init,
            worker_name,
            worker_type,
            worker_url,
            from_devtools_receiver,
            runtime,
            parent_event_loop_sender,
            own_sender,
            receiver,
            closing,
            image_cache,
            browsing_context,
            #[cfg(feature = "webgpu")]
            gpu_id_hub,
            control_receiver,
            insecure_requests_policy,
        ));
        unsafe {
            DedicatedWorkerGlobalScopeBinding::Wrap::<crate::DomTypeHolder>(
                SafeJSContext::from_ptr(cx),
                scope,
            )
        }
    }

    /// <https://html.spec.whatwg.org/multipage/#run-a-worker>
    #[allow(unsafe_code, clippy::too_many_arguments)]
    pub(crate) fn run_worker_scope(
        mut init: WorkerGlobalScopeInit,
        worker_url: ServoUrl,
        from_devtools_receiver: IpcReceiver<DevtoolScriptControlMsg>,
        worker: TrustedWorkerAddress,
        parent_event_loop_sender: ScriptEventLoopSender,
        own_sender: Sender<DedicatedWorkerScriptMsg>,
        receiver: Receiver<DedicatedWorkerScriptMsg>,
        worker_load_origin: WorkerScriptLoadOrigin,
        worker_name: String,
        worker_type: WorkerType,
        closing: Arc<AtomicBool>,
        image_cache: Arc<dyn ImageCache>,
        browsing_context: Option<BrowsingContextId>,
        #[cfg(feature = "webgpu")] gpu_id_hub: Arc<IdentityHub>,
        control_receiver: Receiver<DedicatedWorkerControlMsg>,
        context_sender: Sender<ThreadSafeJSContext>,
        insecure_requests_policy: InsecureRequestsPolicy,
        policy_container: PolicyContainer,
    ) -> JoinHandle<()> {
        let serialized_worker_url = worker_url.to_string();
        let webview_id = WebViewId::installed();
        let current_global = GlobalScope::current().expect("No current global object");
        let origin = current_global.origin().immutable().clone();
        let referrer = current_global.get_referrer();
        let parent = current_global.runtime_handle();
        let current_global_https_state = current_global.get_https_state();
        let current_global_ancestor_trustworthy = current_global.has_trustworthy_ancestor_origin();
        let is_secure_context = current_global.is_secure_context();

        thread::Builder::new()
            .name(format!("WW:{}", worker_url.debug_compact()))
            .spawn(move || {
                thread_state::initialize(ThreadState::SCRIPT | ThreadState::IN_WORKER);

                if let Some(webview_id) = webview_id {
                    WebViewId::install(webview_id);
                }

                let roots = RootCollection::new();
                let _stack_roots = ThreadLocalStackRoots::new(&roots);

                let WorkerScriptLoadOrigin {
                    referrer_url,
                    referrer_policy,
                    pipeline_id,
                } = worker_load_origin;

                let referrer = referrer_url.map(Referrer::ReferrerUrl).unwrap_or(referrer);

                let request = RequestBuilder::new(webview_id, worker_url.clone(), referrer)
                    .destination(Destination::Worker)
                    .mode(RequestMode::SameOrigin)
                    .credentials_mode(CredentialsMode::CredentialsSameOrigin)
                    .parser_metadata(ParserMetadata::NotParserInserted)
                    .use_url_credentials(true)
                    .pipeline_id(Some(pipeline_id))
                    .referrer_policy(referrer_policy)
                    .insecure_requests_policy(insecure_requests_policy)
                    .has_trustworthy_ancestor_origin(current_global_ancestor_trustworthy)
                    .policy_container(policy_container.clone())
                    .origin(origin);

                let runtime = unsafe {
                    let task_source = SendableTaskSource {
                        sender: ScriptEventLoopSender::DedicatedWorker {
                            sender: own_sender.clone(),
                            main_thread_worker: worker.clone(),
                        },
                        pipeline_id,
                        name: TaskSourceName::Networking,
                        canceller: Default::default(),
                    };
                    Runtime::new_with_parent(Some(parent), Some(task_source))
                };

                let context_for_interrupt = runtime.thread_safe_js_context();
                let _ = context_sender.send(context_for_interrupt);

                let (devtools_mpsc_chan, devtools_mpsc_port) = unbounded();
                ROUTER.route_ipc_receiver_to_crossbeam_sender(
                    from_devtools_receiver,
                    devtools_mpsc_chan,
                );

                // Step 8 "Set up a worker environment settings object [...]"
                //
                // <https://html.spec.whatwg.org/multipage/#script-settings-for-workers>
                //
                // > The origin: Return a unique opaque origin if `worker global
                // > scope`'s url's scheme is "data", and `inherited origin`
                // > otherwise.
                if worker_url.scheme() == "data" {
                    // Workers created from a data: url are secure if they were created from secure contexts
                    if is_secure_context {
                        init.origin = ImmutableOrigin::new_opaque_data_url_worker();
                    } else {
                        init.origin = ImmutableOrigin::new_opaque();
                    }
                }

                let global = DedicatedWorkerGlobalScope::new(
                    init,
                    DOMString::from_string(worker_name),
                    worker_type,
                    worker_url,
                    devtools_mpsc_port,
                    runtime,
                    parent_event_loop_sender.clone(),
                    own_sender,
                    receiver,
                    closing,
                    image_cache,
                    browsing_context,
                    #[cfg(feature = "webgpu")]
                    gpu_id_hub,
                    control_receiver,
                    insecure_requests_policy,
                );
                // FIXME(njn): workers currently don't have a unique ID suitable for using in reporter
                // registration (#6631), so we instead use a random number and cross our fingers.
                let scope = global.upcast::<WorkerGlobalScope>();
                let global_scope = global.upcast::<GlobalScope>();

                global_scope.set_https_state(current_global_https_state);

                let (metadata, bytes) = match load_whole_resource(
                    request,
                    &global_scope.resource_threads().sender(),
                    global_scope,
                    &DedicatedWorkerCspProcessor {
                        parent_event_loop_sender: parent_event_loop_sender.clone(),
                        pipeline_id,
                    },
                    CanGc::note(),
                ) {
                    Err(e) => {
                        error!("error loading script {} ({:?})", serialized_worker_url, e);
                        parent_event_loop_sender
                            .send(CommonScriptMsg::Task(
                                WorkerEvent,
                                Box::new(SimpleWorkerErrorHandler::new(worker)),
                                Some(pipeline_id),
                                TaskSourceName::DOMManipulation,
                            ))
                            .unwrap();
                        scope.clear_js_runtime();
                        return;
                    },
                    Ok((metadata, bytes)) => (metadata, bytes),
                };
                scope.set_url(metadata.final_url.clone());
                Self::initialize_policy_container_for_worker_global_scope(
                    scope,
                    &metadata,
                    &policy_container,
                );
                scope.set_endpoints_list(ReportingEndpoint::parse_reporting_endpoints_header(
                    &metadata.final_url.clone(),
                    &metadata.headers,
                ));
                global_scope.set_https_state(metadata.https_state);
                let source = String::from_utf8_lossy(&bytes);

                unsafe {
                    // Handle interrupt requests
                    JS_AddInterruptCallback(*scope.get_cx(), Some(interrupt_callback));
                }

                if scope.is_closing() {
                    scope.clear_js_runtime();
                    return;
                }

                {
                    let _ar = AutoWorkerReset::new(&global, worker.clone());
                    let realm = enter_realm(scope);
                    define_all_exposed_interfaces(
                        global.upcast(),
                        InRealm::entered(&realm),
                        CanGc::note(),
                    );
                    scope.execute_script(DOMString::from(source), CanGc::note());
                }

                let reporter_name = format!("dedicated-worker-reporter-{}", random::<u64>());
                scope
                    .upcast::<GlobalScope>()
                    .mem_profiler_chan()
                    .run_with_memory_reporting(
                        || {
                            // Step 27, Run the responsible event loop specified
                            // by inside settings until it is destroyed.
                            // The worker processing model remains on this step
                            // until the event loop is destroyed,
                            // which happens after the closing flag is set to true.
                            while !scope.is_closing() {
                                run_worker_event_loop(&*global, Some(&worker), CanGc::note());
                            }
                        },
                        reporter_name,
                        parent_event_loop_sender,
                        CommonScriptMsg::CollectReports,
                    );

                scope.clear_js_runtime();
            })
            .expect("Thread spawning failed")
    }

    /// <https://html.spec.whatwg.org/multipage/#initialize-worker-policy-container> and
    /// <https://html.spec.whatwg.org/multipage/#creating-a-policy-container-from-a-fetch-response>
    fn initialize_policy_container_for_worker_global_scope(
        scope: &WorkerGlobalScope,
        metadata: &Metadata,
        parent_policy_container: &PolicyContainer,
    ) {
        // Step 1. If workerGlobalScope's url is local but its scheme is not "blob":
        //
        // Note that we also allow for blob here, as the parent_policy_container is in both cases
        // the container that we need to clone.
        if metadata.final_url.is_local_scheme() {
            // Step 1.2. Set workerGlobalScope's policy container to a clone of workerGlobalScope's
            // owner set[0]'s relevant settings object's policy container.
            //
            // Step 1. If response's URL's scheme is "blob", then return a clone of response's URL's
            // blob URL entry's environment's policy container.
            scope.set_csp_list(parent_policy_container.csp_list.clone());
            scope.set_referrer_policy(parent_policy_container.get_referrer_policy());
            return;
        }
        // Step 3. Set result's CSP list to the result of parsing a response's Content Security Policies given response.
        scope.set_csp_list(parse_csp_list_from_metadata(&metadata.headers));
        // Step 5. Set result's referrer policy to the result of parsing the `Referrer-Policy`
        // header given response. [REFERRERPOLICY]
        let referrer_policy = metadata
            .headers
            .as_ref()
            .and_then(|headers| headers.typed_get::<ReferrerPolicyHeader>())
            .into();
        scope.set_referrer_policy(referrer_policy);
    }

    /// The non-None value of the `worker` field can contain a rooted [`TrustedWorkerAddress`]
    /// version of the main thread's worker object. This is set while handling messages and then
    /// unset otherwise, ensuring that the main thread object can be garbage collected. See
    /// [`AutoWorkerReset`].
    fn replace_worker(
        &self,
        new_worker: Option<TrustedWorkerAddress>,
    ) -> Option<TrustedWorkerAddress> {
        let old_worker = std::mem::replace(&mut *self.worker.borrow_mut(), new_worker);

        // The `TaskManager` maintains a handle to this `DedicatedWorkerGlobalScope`'s
        // event_loop_sender, which might in turn have a `TrustedWorkerAddress` rooting of the main
        // thread's worker, which prevents garbage collection. Resetting it here ensures that
        // garbage collection of the main thread object can happen again (assuming the new `worker`
        // is `None`).
        self.upcast::<GlobalScope>()
            .task_manager()
            .set_sender(self.event_loop_sender());

        old_worker
    }

    pub(crate) fn image_cache(&self) -> Arc<dyn ImageCache> {
        self.image_cache.clone()
    }

    pub(crate) fn event_loop_sender(&self) -> Option<ScriptEventLoopSender> {
        Some(ScriptEventLoopSender::DedicatedWorker {
            sender: self.own_sender.clone(),
            main_thread_worker: self.worker.borrow().clone()?,
        })
    }

    pub(crate) fn new_script_pair(&self) -> (ScriptEventLoopSender, ScriptEventLoopReceiver) {
        let (sender, receiver) = unbounded();
        let main_thread_worker = self.worker.borrow().as_ref().unwrap().clone();
        (
            ScriptEventLoopSender::DedicatedWorker {
                sender,
                main_thread_worker,
            },
            ScriptEventLoopReceiver::DedicatedWorker(receiver),
        )
    }

    fn handle_script_event(&self, msg: WorkerScriptMsg, can_gc: CanGc) {
        match msg {
            WorkerScriptMsg::DOMMessage { origin, data } => {
                let scope = self.upcast::<WorkerGlobalScope>();
                let target = self.upcast();
                let _ac = enter_realm(self);
                rooted!(in(*scope.get_cx()) let mut message = UndefinedValue());
                if let Ok(ports) =
                    structuredclone::read(scope.upcast(), *data, message.handle_mut())
                {
                    MessageEvent::dispatch_jsval(
                        target,
                        scope.upcast(),
                        message.handle(),
                        Some(&origin.ascii_serialization()),
                        None,
                        ports,
                        can_gc,
                    );
                } else {
                    MessageEvent::dispatch_error(target, scope.upcast(), can_gc);
                }
            },
            WorkerScriptMsg::Common(msg) => {
                self.upcast::<WorkerGlobalScope>().process_event(msg);
            },
        }
    }

    fn handle_mixed_message(&self, msg: MixedMessage, can_gc: CanGc) -> bool {
        if self.upcast::<WorkerGlobalScope>().is_closing() {
            return false;
        }
        // FIXME(#26324): `self.worker` is None in devtools messages.
        match msg {
            MixedMessage::Devtools(msg) => match msg {
                DevtoolScriptControlMsg::EvaluateJS(_pipe_id, string, sender) => {
                    devtools::handle_evaluate_js(self.upcast(), string, sender, can_gc)
                },
                DevtoolScriptControlMsg::WantsLiveNotifications(_pipe_id, bool_val) => {
                    devtools::handle_wants_live_notifications(self.upcast(), bool_val)
                },
                _ => debug!("got an unusable devtools control message inside the worker!"),
            },
            MixedMessage::Worker(DedicatedWorkerScriptMsg::CommonWorker(linked_worker, msg)) => {
                let _ar = AutoWorkerReset::new(self, linked_worker);
                self.handle_script_event(msg, can_gc);
            },
            MixedMessage::Worker(DedicatedWorkerScriptMsg::WakeUp) => {},
            MixedMessage::Control(DedicatedWorkerControlMsg::Exit) => {
                return false;
            },
            MixedMessage::Timer => {},
        }
        true
    }

    // https://html.spec.whatwg.org/multipage/#runtime-script-errors-2
    #[allow(unsafe_code)]
    pub(crate) fn forward_error_to_worker_object(&self, error_info: ErrorInfo) {
        let worker = self.worker.borrow().as_ref().unwrap().clone();
        let pipeline_id = self.upcast::<GlobalScope>().pipeline_id();
        let task = Box::new(task!(forward_error_to_worker_object: move || {
            let worker = worker.root();
            let global = worker.global();

            // Step 1.
            let event = ErrorEvent::new(
                &global,
                atom!("error"),
                EventBubbles::DoesNotBubble,
                EventCancelable::Cancelable,
                error_info.message.as_str().into(),
                error_info.filename.as_str().into(),
                error_info.lineno,
                error_info.column,
                HandleValue::null(),
                CanGc::note(),
            );
            let event_status =
                event.upcast::<Event>().fire(worker.upcast::<EventTarget>(), CanGc::note());

            // Step 2.
            if event_status == EventStatus::NotCanceled {
                global.report_an_error(error_info, HandleValue::null(), CanGc::note());
            }
        }));
        self.parent_event_loop_sender
            .send(CommonScriptMsg::Task(
                WorkerEvent,
                task,
                Some(pipeline_id),
                TaskSourceName::DOMManipulation,
            ))
            .unwrap();
    }

    // https://html.spec.whatwg.org/multipage/#dom-dedicatedworkerglobalscope-postmessage
    fn post_message_impl(
        &self,
        cx: SafeJSContext,
        message: HandleValue,
        transfer: CustomAutoRooterGuard<Vec<*mut JSObject>>,
    ) -> ErrorResult {
        let data = structuredclone::write(cx, message, Some(transfer))?;
        let worker = self.worker.borrow().as_ref().unwrap().clone();
        let global_scope = self.upcast::<GlobalScope>();
        let pipeline_id = global_scope.pipeline_id();
        let task = Box::new(task!(post_worker_message: move || {
            Worker::handle_message(worker, data, CanGc::note());
        }));
        self.parent_event_loop_sender
            .send(CommonScriptMsg::Task(
                WorkerEvent,
                task,
                Some(pipeline_id),
                TaskSourceName::DOMManipulation,
            ))
            .expect("Sending to parent failed");
        Ok(())
    }

    pub(crate) fn browsing_context(&self) -> Option<BrowsingContextId> {
        self.browsing_context
    }
}

#[allow(unsafe_code)]
unsafe extern "C" fn interrupt_callback(cx: *mut JSContext) -> bool {
    let in_realm_proof = AlreadyInRealm::assert_for_cx(SafeJSContext::from_ptr(cx));
    let global = GlobalScope::from_context(cx, InRealm::Already(&in_realm_proof));
    let worker =
        DomRoot::downcast::<WorkerGlobalScope>(global).expect("global is not a worker scope");
    assert!(worker.is::<DedicatedWorkerGlobalScope>());

    // A false response causes the script to terminate
    !worker.is_closing()
}

impl DedicatedWorkerGlobalScopeMethods<crate::DomTypeHolder> for DedicatedWorkerGlobalScope {
    /// <https://html.spec.whatwg.org/multipage/#dom-dedicatedworkerglobalscope-postmessage>
    fn PostMessage(
        &self,
        cx: SafeJSContext,
        message: HandleValue,
        transfer: CustomAutoRooterGuard<Vec<*mut JSObject>>,
    ) -> ErrorResult {
        self.post_message_impl(cx, message, transfer)
    }

    /// <https://html.spec.whatwg.org/multipage/#dom-dedicatedworkerglobalscope-postmessage>
    fn PostMessage_(
        &self,
        cx: SafeJSContext,
        message: HandleValue,
        options: RootedTraceableBox<StructuredSerializeOptions>,
    ) -> ErrorResult {
        let mut rooted = CustomAutoRooter::new(
            options
                .transfer
                .iter()
                .map(|js: &RootedTraceableBox<Heap<*mut JSObject>>| js.get())
                .collect(),
        );
        let guard = CustomAutoRooterGuard::new(*cx, &mut rooted);
        self.post_message_impl(cx, message, guard)
    }

    // https://html.spec.whatwg.org/multipage/#dom-dedicatedworkerglobalscope-close
    fn Close(&self) {
        // Step 2
        self.upcast::<WorkerGlobalScope>().close();
    }

    // https://html.spec.whatwg.org/multipage/#handler-dedicatedworkerglobalscope-onmessage
    event_handler!(message, GetOnmessage, SetOnmessage);
}
