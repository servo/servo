/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use std::collections::VecDeque;
use std::io;
use std::sync::Arc;
use std::sync::atomic::AtomicBool;
use std::thread::{self, JoinHandle};

use crossbeam_channel::{Receiver, Sender, unbounded};
use devtools_traits::DevtoolScriptControlMsg;
use dom_struct::dom_struct;
use fonts::FontContext;
use js::context::JSContext;
use js::jsval::UndefinedValue;
use net_traits::blob_url_store::UrlWithBlobClaim;
use net_traits::policy_container::{PolicyContainer, RequestPolicyContainer};
use net_traits::request::{
    CredentialsMode, Destination, InsecureRequestsPolicy, Origin, PreloadedResources, Referrer,
    RequestClient,
};
use script_bindings::cell::DomRefCell;
use script_bindings::conversions::SafeToJSValConvertible;
use servo_base::generic_channel::{GenericReceiver, RoutedReceiver};
use servo_base::id::ScriptEventLoopId;
use servo_constellation_traits::{MessagePortImpl, WorkerGlobalScopeInit, WorkerScriptLoadOrigin};
use servo_url::{ImmutableOrigin, ServoUrl};
use style::thread_state::{self, ThreadState};
use stylo_atoms::Atom;
use uuid::Uuid;

use crate::dom::abstractworker::{SimpleWorkerErrorHandler, WorkerScriptMsg};
use crate::dom::abstractworkerglobalscope::{WorkerEventLoopMethods, run_worker_event_loop};
use crate::dom::bindings::codegen::Bindings::SharedWorkerGlobalScopeBinding;
use crate::dom::bindings::codegen::Bindings::SharedWorkerGlobalScopeBinding::SharedWorkerGlobalScopeMethods;
use crate::dom::bindings::codegen::Bindings::WorkerBinding::WorkerType;
use crate::dom::bindings::codegen::UnionTypes::WindowProxyOrMessagePortOrServiceWorker;
use crate::dom::bindings::inheritance::Castable;
use crate::dom::bindings::refcounted::Trusted;
use crate::dom::bindings::root::{Dom, DomRoot};
use crate::dom::bindings::str::DOMString;
use crate::dom::bindings::trace::CustomTraceable;
use crate::dom::dedicatedworkerglobalscope::fetch_a_classic_worker_script;
use crate::dom::event::Event;
use crate::dom::event::messageevent::MessageEvent;
use crate::dom::eventtarget::EventTarget;
use crate::dom::globalscope::GlobalScope;
use crate::dom::html::htmlscriptelement::Script;
use crate::dom::messageport::MessagePort;
use crate::dom::sharedworker::{SharedWorker, SharedWorkerStorageKey, TrustedSharedWorkerAddress};
use crate::dom::types::DebuggerGlobalScope;
#[cfg(feature = "webgpu")]
use crate::dom::webgpu::identityhub::IdentityHub;
use crate::dom::workerglobalscope::WorkerGlobalScope;
use crate::messaging::{CommonScriptMsg, ScriptEventLoopReceiver, ScriptEventLoopSender};
use crate::script_module::{ModuleFetchClient, fetch_a_module_worker_script_graph};
use crate::script_runtime::ScriptThreadEventCategory::WorkerEvent;
use crate::script_runtime::{CanGc, Runtime};
use crate::task_queue::{QueuedTask, QueuedTaskConversion, TaskQueue};
use crate::task_source::TaskSourceName;

pub(crate) enum SharedWorkerScriptMsg {
    CommonWorker(WorkerScriptMsg),
    Connect(MessagePortImpl),
    WakeUp,
}

#[allow(dead_code)]
pub(crate) enum SharedWorkerControlMsg {
    Exit,
}

pub(crate) enum MixedMessage {
    SharedWorker(SharedWorkerScriptMsg),
    Devtools(DevtoolScriptControlMsg),
    Control(SharedWorkerControlMsg),
    Timer,
}

struct SharedWorkerRegistrationCleanup {
    registration_id: Uuid,
}

impl Drop for SharedWorkerRegistrationCleanup {
    fn drop(&mut self) {
        SharedWorker::unregister_shared_worker(self.registration_id);
    }
}

impl QueuedTaskConversion for SharedWorkerScriptMsg {
    fn task_source_name(&self) -> Option<&TaskSourceName> {
        let script_msg = match self {
            SharedWorkerScriptMsg::CommonWorker(WorkerScriptMsg::Common(script_msg)) => script_msg,
            _ => return None,
        };
        match script_msg {
            CommonScriptMsg::Task(_category, _boxed, _pipeline_id, task_source) => {
                Some(task_source)
            },
            _ => None,
        }
    }

    fn pipeline_id(&self) -> Option<servo_base::id::PipelineId> {
        None
    }

    fn into_queued_task(self) -> Option<QueuedTask> {
        let script_msg = match self {
            SharedWorkerScriptMsg::CommonWorker(WorkerScriptMsg::Common(script_msg)) => script_msg,
            _ => return None,
        };
        let (event_category, task, pipeline_id, task_source) = match script_msg {
            CommonScriptMsg::Task(category, boxed, pipeline_id, task_source) => {
                (category, boxed, pipeline_id, task_source)
            },
            _ => return None,
        };
        Some(QueuedTask {
            worker: None,
            event_category,
            task,
            pipeline_id,
            task_source,
        })
    }

    fn from_queued_task(queued_task: QueuedTask) -> Self {
        let script_msg = CommonScriptMsg::Task(
            queued_task.event_category,
            queued_task.task,
            queued_task.pipeline_id,
            queued_task.task_source,
        );
        SharedWorkerScriptMsg::CommonWorker(WorkerScriptMsg::Common(script_msg))
    }

    fn inactive_msg() -> Self {
        panic!("Workers should never receive messages marked as inactive");
    }

    fn wake_up_msg() -> Self {
        SharedWorkerScriptMsg::WakeUp
    }

    fn is_wake_up(&self) -> bool {
        matches!(self, SharedWorkerScriptMsg::WakeUp)
    }
}

unsafe_no_jsmanaged_fields!(TaskQueue<SharedWorkerScriptMsg>);

// https://html.spec.whatwg.org/multipage/#shared-workers-and-the-sharedworkerglobalscope-interface
#[dom_struct]
pub(crate) struct SharedWorkerGlobalScope {
    workerglobalscope: WorkerGlobalScope,
    #[ignore_malloc_size_of = "Defined in std"]
    task_queue: TaskQueue<SharedWorkerScriptMsg>,
    own_sender: Sender<SharedWorkerScriptMsg>,
    worker: DomRefCell<Option<TrustedSharedWorkerAddress>>,
    parent_event_loop_sender: ScriptEventLoopSender,
    // Shared workers receive message ports through `connect` events on their `SharedWorkerGlobalScope` object for each connection.
    pending_connect: DomRefCell<VecDeque<Dom<MessagePort>>>,
    #[no_trace]
    control_receiver: Receiver<SharedWorkerControlMsg>,
    debugger_global: Dom<DebuggerGlobalScope>,
    // A `SharedWorkerGlobalScope` object has associated constructor origin (an origin), constructor URL (a URL record), and credentials (a credentials mode), and extended lifetime (a boolean).
    #[no_trace]
    storage_key: SharedWorkerStorageKey,
    #[no_trace]
    constructor_origin: ImmutableOrigin,
    #[no_trace]
    constructor_url: ServoUrl,
    #[no_trace]
    credentials: CredentialsMode,
    extended_lifetime: bool,
    #[no_trace]
    registration_id: Uuid,
}

impl WorkerEventLoopMethods for SharedWorkerGlobalScope {
    type WorkerMsg = SharedWorkerScriptMsg;
    type ControlMsg = SharedWorkerControlMsg;
    type Event = MixedMessage;

    fn task_queue(&self) -> &TaskQueue<SharedWorkerScriptMsg> {
        &self.task_queue
    }

    fn handle_event(&self, event: MixedMessage, cx: &mut JSContext) -> bool {
        self.handle_mixed_message(event, cx)
    }

    fn handle_worker_post_event(
        &self,
        _worker: &crate::dom::worker::TrustedWorkerAddress,
    ) -> Option<crate::dom::dedicatedworkerglobalscope::AutoWorkerReset<'_>> {
        None
    }

    fn from_control_msg(msg: SharedWorkerControlMsg) -> MixedMessage {
        MixedMessage::Control(msg)
    }

    fn from_worker_msg(msg: SharedWorkerScriptMsg) -> MixedMessage {
        MixedMessage::SharedWorker(msg)
    }

    fn from_devtools_msg(msg: DevtoolScriptControlMsg) -> MixedMessage {
        MixedMessage::Devtools(msg)
    }

    fn from_timer_msg() -> MixedMessage {
        MixedMessage::Timer
    }

    fn control_receiver(&self) -> &Receiver<SharedWorkerControlMsg> {
        &self.control_receiver
    }
}

impl SharedWorkerGlobalScope {
    #[allow(clippy::too_many_arguments)]
    fn new_inherited(
        init: WorkerGlobalScopeInit,
        worker_name: DOMString,
        worker_type: WorkerType,
        worker_url: ServoUrl,
        worker: TrustedSharedWorkerAddress,
        parent_event_loop_sender: ScriptEventLoopSender,
        from_devtools_receiver: RoutedReceiver<DevtoolScriptControlMsg>,
        runtime: Runtime,
        own_sender: Sender<SharedWorkerScriptMsg>,
        receiver: Receiver<SharedWorkerScriptMsg>,
        closing: Arc<AtomicBool>,
        #[cfg(feature = "webgpu")] gpu_id_hub: Arc<IdentityHub>,
        control_receiver: Receiver<SharedWorkerControlMsg>,
        insecure_requests_policy: InsecureRequestsPolicy,
        font_context: Option<Arc<FontContext>>,
        debugger_global: &DebuggerGlobalScope,
        storage_key: SharedWorkerStorageKey,
        constructor_origin: ImmutableOrigin,
        constructor_url: ServoUrl,
        credentials: CredentialsMode,
        extended_lifetime: bool,
        registration_id: Uuid,
    ) -> SharedWorkerGlobalScope {
        SharedWorkerGlobalScope {
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
                font_context,
            ),
            task_queue: TaskQueue::new(receiver, own_sender.clone()),
            own_sender,
            worker: DomRefCell::new(Some(worker)),
            parent_event_loop_sender,
            pending_connect: DomRefCell::new(VecDeque::new()),
            control_receiver,
            debugger_global: Dom::from_ref(debugger_global),
            storage_key,
            constructor_origin,
            constructor_url,
            credentials,
            extended_lifetime,
            registration_id,
        }
    }

    #[allow(clippy::too_many_arguments)]
    pub(crate) fn new(
        init: WorkerGlobalScopeInit,
        worker_name: DOMString,
        worker_type: WorkerType,
        worker_url: ServoUrl,
        worker: TrustedSharedWorkerAddress,
        parent_event_loop_sender: ScriptEventLoopSender,
        from_devtools_receiver: RoutedReceiver<DevtoolScriptControlMsg>,
        runtime: Runtime,
        own_sender: Sender<SharedWorkerScriptMsg>,
        receiver: Receiver<SharedWorkerScriptMsg>,
        closing: Arc<AtomicBool>,
        #[cfg(feature = "webgpu")] gpu_id_hub: Arc<IdentityHub>,
        control_receiver: Receiver<SharedWorkerControlMsg>,
        insecure_requests_policy: InsecureRequestsPolicy,
        font_context: Option<Arc<FontContext>>,
        debugger_global: &DebuggerGlobalScope,
        storage_key: SharedWorkerStorageKey,
        constructor_origin: ImmutableOrigin,
        constructor_url: ServoUrl,
        credentials: CredentialsMode,
        extended_lifetime: bool,
        registration_id: Uuid,
        cx: &mut js::context::JSContext,
    ) -> DomRoot<SharedWorkerGlobalScope> {
        let scope = Box::new(SharedWorkerGlobalScope::new_inherited(
            init,
            worker_name,
            worker_type,
            worker_url,
            worker,
            parent_event_loop_sender,
            from_devtools_receiver,
            runtime,
            own_sender,
            receiver,
            closing,
            #[cfg(feature = "webgpu")]
            gpu_id_hub,
            control_receiver,
            insecure_requests_policy,
            font_context,
            debugger_global,
            storage_key,
            constructor_origin,
            constructor_url,
            credentials,
            extended_lifetime,
            registration_id,
        ));
        SharedWorkerGlobalScopeBinding::Wrap::<crate::DomTypeHolder>(cx, scope)
    }

    /// <https://html.spec.whatwg.org/multipage/#run-a-worker>
    #[expect(unsafe_code)]
    #[allow(clippy::too_many_arguments)]
    pub(crate) fn run_shared_worker_scope(
        mut init: WorkerGlobalScopeInit,
        worker_name: DOMString,
        worker_type: WorkerType,
        worker_url: UrlWithBlobClaim,
        worker: TrustedSharedWorkerAddress,
        parent_event_loop_sender: ScriptEventLoopSender,
        from_devtools_receiver: GenericReceiver<DevtoolScriptControlMsg>,
        own_sender: Sender<SharedWorkerScriptMsg>,
        receiver: Receiver<SharedWorkerScriptMsg>,
        worker_load_origin: WorkerScriptLoadOrigin,
        closing: Arc<AtomicBool>,
        #[cfg(feature = "webgpu")] gpu_id_hub: Arc<IdentityHub>,
        control_receiver: Receiver<SharedWorkerControlMsg>,
        setup_sender: Sender<bool>,
        registration_receiver: Receiver<()>,
        registration_id: Uuid,
        credentials: CredentialsMode,
        extended_lifetime: bool,
        constructor_origin: ImmutableOrigin,
        constructor_url: ServoUrl,
        storage_key: SharedWorkerStorageKey,
        insecure_requests_policy: InsecureRequestsPolicy,
        policy_container: PolicyContainer,
        font_context: Option<Arc<FontContext>>,
    ) -> io::Result<JoinHandle<()>> {
        let event_loop_id = ScriptEventLoopId::installed()
            .expect("Should always be in a ScriptThread or in a worker");
        let current_global = GlobalScope::current().expect("No current global object");
        let origin = current_global.origin().immutable().clone();
        let referrer = current_global.get_referrer();
        let is_secure_context = current_global.is_secure_context();
        let current_global_ancestor_trustworthy = current_global.has_trustworthy_ancestor_origin();
        let is_nested_browsing_context = current_global.is_nested_browsing_context();
        let webview_id = current_global.webview_id();
        let worker_name = worker_name.to_string();

        thread::Builder::new()
            .name(format!("SWW:{}", worker_url.debug_compact()))
            .spawn(move || {
                // Step 4. Let agent be the result of obtaining a dedicated/shared worker agent
                // given outside settings and is shared. Run the rest of these steps in that
                // agent.
                thread_state::initialize(ThreadState::SCRIPT | ThreadState::IN_WORKER);
                ScriptEventLoopId::install(event_loop_id);

                let WorkerScriptLoadOrigin {
                    referrer_url,
                    pipeline_id,
                    ..
                } = worker_load_origin;

                let referrer = referrer_url.map(Referrer::ReferrerUrl).unwrap_or(referrer);

                let request_client = RequestClient {
                    preloaded_resources: PreloadedResources::default(),
                    policy_container: RequestPolicyContainer::PolicyContainer(
                        policy_container.clone(),
                    ),
                    origin: Origin::Origin(origin.clone()),
                    is_nested_browsing_context,
                    insecure_requests_policy,
                };

                let event_loop_sender = ScriptEventLoopSender::SharedWorker(own_sender.clone());

                // Shared workers currently run on a dedicated script worker thread but
                // still create their own JS runtime in that thread; this avoids child
                // runtime lifetime coupling with the embedding script thread.
                let runtime = Runtime::new(Some(event_loop_sender.clone()));
                // SAFETY: We are in a new thread, so this first cx.
                // It is OK to have it separated of runtime here,
                // because it will never outlive it (runtime destruction happens at the end of this function)
                let mut cx = unsafe { runtime.cx() };
                let cx = &mut cx;
                let debugger_global = DebuggerGlobalScope::new(
                    pipeline_id,
                    init.to_devtools_sender.clone(),
                    init.from_devtools_sender
                        .clone()
                        .expect("Guaranteed by SharedWorker::Constructor"),
                    init.mem_profiler_chan.clone(),
                    init.time_profiler_chan.clone(),
                    init.script_to_constellation_chan.clone(),
                    init.script_to_embedder_chan.clone(),
                    init.resource_threads.clone(),
                    init.storage_threads.clone(),
                    #[cfg(feature = "webgpu")]
                    gpu_id_hub.clone(),
                    cx,
                );
                debugger_global.execute(cx);

                let devtools_mpsc_port = from_devtools_receiver.route_preserving_errors();

                let worker_id = init.worker_id;
                let devtools_enabled = init.to_devtools_sender.is_some();
                // Step 3. Let origin be a unique opaque origin if worker global scope's url's scheme is "data"; otherwise outside settings's origin.
                if worker_url.scheme() == "data" {
                    if is_secure_context {
                        init.origin = ImmutableOrigin::new_opaque_data_url_worker();
                    } else {
                        init.origin = ImmutableOrigin::new_opaque();
                    }
                }
                // Step 8. Set worker global scope's name to options["name"].
                // Step 10.3. Set worker global scope's type to options["type"].
                let global = SharedWorkerGlobalScope::new(
                    init,
                    worker_name.into(),
                    worker_type,
                    worker_url.url(),
                    worker,
                    parent_event_loop_sender,
                    devtools_mpsc_port,
                    runtime,
                    own_sender,
                    receiver,
                    closing,
                    #[cfg(feature = "webgpu")]
                    gpu_id_hub,
                    control_receiver,
                    insecure_requests_policy,
                    font_context,
                    &debugger_global,
                    storage_key,
                    constructor_origin,
                    constructor_url,
                    credentials,
                    extended_lifetime,
                    registration_id,
                    cx,
                );
                let scope = global.upcast::<WorkerGlobalScope>();
                let global_scope = global.upcast::<GlobalScope>();
                // Step 11.5.2. Let workerIsSecureContext be true if insideSettings is a secure context; otherwise, false.
                let worker_is_secure_context = global_scope.is_secure_context();
                if devtools_enabled {
                    debugger_global.fire_add_debuggee(
                        cx,
                        global_scope,
                        pipeline_id,
                        Some(worker_id),
                    );
                }

                if setup_sender.send(worker_is_secure_context).is_err() {
                    scope.clear_js_runtime();
                    return;
                }

                if registration_receiver.recv().is_err() {
                    scope.clear_js_runtime();
                    return;
                }
                // Keep cleanup guard alive for the remainder of worker execution.
                // It is intentionally unused because its Drop unregisters the worker.
                let _registration_cleanup = SharedWorkerRegistrationCleanup { registration_id };

                let fetch_client = ModuleFetchClient {
                    insecure_requests_policy,
                    has_trustworthy_ancestor_origin: current_global_ancestor_trustworthy,
                    policy_container,
                    client: request_client,
                    pipeline_id,
                    origin,
                };

                // Step 11. Let destination be "sharedworker" if is shared is true, and
                // "worker" otherwise.
                // Step 12. Obtain script by switching on options["type"]:
                match worker_type {
                    WorkerType::Classic => {
                        fetch_a_classic_worker_script(
                            scope,
                            worker_url,
                            fetch_client,
                            Destination::SharedWorker,
                            webview_id,
                            referrer,
                        );
                    },
                    WorkerType::Module => {
                        let worker_scope = DomRoot::from_ref(scope);
                        fetch_a_module_worker_script_graph(
                            cx,
                            global_scope,
                            worker_url.url(),
                            fetch_client,
                            Destination::SharedWorker,
                            referrer,
                            credentials,
                            move |cx, module_tree| {
                                worker_scope.on_complete(cx, module_tree.map(Script::Module));
                            },
                        );
                    },
                }

                let reporter_name = format!("shared-worker-reporter-{}", worker_id);
                scope
                    .upcast::<GlobalScope>()
                    .mem_profiler_chan()
                    .run_with_memory_reporting(
                        || {
                            // Event loop: Run the responsible event loop specified by inside settings until it is destroyed.
                            while !scope.is_closing() {
                                run_worker_event_loop(&*global, None, cx);
                            }
                        },
                        reporter_name,
                        event_loop_sender,
                        CommonScriptMsg::CollectReports,
                    );

                scope.clear_js_runtime();
            })
    }

    pub(crate) fn event_loop_sender(&self) -> ScriptEventLoopSender {
        ScriptEventLoopSender::SharedWorker(self.own_sender.clone())
    }

    pub(crate) fn new_script_pair(&self) -> (ScriptEventLoopSender, ScriptEventLoopReceiver) {
        let (sender, receiver) = unbounded();
        (
            ScriptEventLoopSender::SharedWorker(sender),
            ScriptEventLoopReceiver::SharedWorker(receiver),
        )
    }

    /// Step 1.1 of onComplete of <https://html.spec.whatwg.org/multipage/#run-a-worker>
    pub(crate) fn forward_simple_error_at_worker(&self) {
        SharedWorker::unregister_shared_worker(self.registration_id);
        let pipeline_id = self.upcast::<GlobalScope>().pipeline_id();
        let worker = self.worker.borrow().clone().expect("worker must be set");
        // Step 1.1. Queue a global task on the DOM manipulation task source given worker's relevant global object to fire an event named error at worker.
        self.parent_event_loop_sender
            .send(CommonScriptMsg::Task(
                WorkerEvent,
                Box::new(SimpleWorkerErrorHandler::new(worker)),
                Some(pipeline_id),
                TaskSourceName::DOMManipulation,
            ))
            .expect("Sending to parent failed");
    }

    /// Step 11 of onComplete of <https://html.spec.whatwg.org/multipage/#run-a-worker>
    pub(crate) fn enable_outside_port_message_queue(&self) {
        let pipeline_id = self.upcast::<GlobalScope>().pipeline_id();
        let worker = self.worker.borrow().clone().expect("worker must be set");

        self.parent_event_loop_sender
            .send(CommonScriptMsg::Task(
                WorkerEvent,
                Box::new(
                    task!(sharedworker_enable_outside_port_message_queue: move |cx| {
                        SharedWorker::enable_outside_port_message_queue(worker, cx);
                    }),
                ),
                Some(pipeline_id),
                TaskSourceName::DOMManipulation,
            ))
            .expect("Sending to parent failed");
    }

    fn handle_connect(
        &self,
        port_impl: MessagePortImpl,
        cx: &mut JSContext,
    ) -> DomRoot<MessagePort> {
        // Let inside port be a new MessagePort object in inside settings's realm.
        let inside_port = MessagePort::new_transferred(
            self.upcast::<GlobalScope>(),
            *port_impl.message_port_id(),
            port_impl.entangled_port_id(),
            CanGc::from_cx(cx),
        );
        self.upcast::<GlobalScope>()
            .track_message_port(&inside_port, Some(port_impl));
        inside_port
    }

    // Step 13. If is shared is true, then queue a global task on the DOM manipulation task source given worker global scope to fire an event named connect at worker global scope, using MessageEvent, with the data attribute initialized to the empty string, the ports attribute initialized to a new frozen array containing inside port, and the source attribute initialized to inside port.
    fn dispatch_connect_event(&self, inside_port: &MessagePort) {
        let worker_global = Trusted::new(self);
        let inside_port = Trusted::new(inside_port);

        self.upcast::<GlobalScope>()
            .task_manager()
            .dom_manipulation_task_source()
            .queue(task!(sharedworker_connect_event: move |cx| {
                let worker_global = worker_global.root();
                let worker_global = &*worker_global;
                let inside_port = inside_port.root();

                rooted!(&in(cx) let mut data = UndefinedValue());
                DOMString::from("").safe_to_jsval(
                    cx.into(),
                    data.handle_mut(),
                    CanGc::from_cx(cx),
                );

                let source = WindowProxyOrMessagePortOrServiceWorker::MessagePort(
                    inside_port.clone(),
                );
                let event = MessageEvent::new(
                    cx,
                    worker_global.upcast::<GlobalScope>(),
                    Atom::from("connect"),
                    false,
                    false,
                    data.handle(),
                    DOMString::from(""),
                    Some(&source),
                    DOMString::new(),
                    vec![inside_port],
                );

                event
                    .upcast::<Event>()
                    .fire(cx, worker_global.upcast::<EventTarget>());
            }));
    }

    pub(crate) fn fire_pending_connect(&self, _cx: &mut JSContext) {
        loop {
            let inside_port = self
                .pending_connect
                .borrow_mut()
                .pop_front()
                .map(|inside_port| inside_port.as_rooted());
            let Some(inside_port) = inside_port else {
                break;
            };
            if self.upcast::<WorkerGlobalScope>().is_closing() {
                return;
            }
            // Step 13. If is shared is true, then queue a global task on the DOM manipulation task source given worker global scope to fire an event named connect at worker global scope, using MessageEvent, with the data attribute initialized to the empty string, the ports attribute initialized to a new frozen array containing inside port, and the source attribute initialized to inside port.
            self.dispatch_connect_event(&inside_port);
        }
    }

    fn handle_script_event(&self, msg: SharedWorkerScriptMsg, cx: &mut JSContext) {
        match msg {
            SharedWorkerScriptMsg::CommonWorker(WorkerScriptMsg::Common(msg)) => {
                self.upcast::<WorkerGlobalScope>().process_event(msg, cx);
            },
            SharedWorkerScriptMsg::Connect(port_impl) => {
                let inside_port = self.handle_connect(port_impl, cx);
                if self.upcast::<WorkerGlobalScope>().is_execution_ready() {
                    // Step 13. If is shared is true, then queue a global task on the DOM manipulation task source given worker global scope to fire an event named connect at worker global scope, using MessageEvent, with the data attribute initialized to the empty string, the ports attribute initialized to a new frozen array containing inside port, and the source attribute initialized to inside port.
                    self.dispatch_connect_event(&inside_port);
                } else {
                    // Step 13. If is shared is true, then queue a global task on the DOM manipulation task source given worker global scope to fire an event named connect at worker global scope, using MessageEvent, with the data attribute initialized to the empty string, the ports attribute initialized to a new frozen array containing inside port, and the source attribute initialized to inside port.
                    self.pending_connect
                        .borrow_mut()
                        .push_back(Dom::from_ref(&*inside_port));
                }
            },
            SharedWorkerScriptMsg::CommonWorker(WorkerScriptMsg::DOMMessage(_)) => {
                // SharedWorker messages arrive through the entangled MessagePort and are
                // surfaced as connect/message events, not as direct WorkerScriptMsg::DOMMessage.
                debug_assert!(
                    false,
                    "SharedWorkerGlobalScope does not support direct DOMMessage dispatch"
                );
            },
            SharedWorkerScriptMsg::WakeUp => {},
        }
    }

    fn handle_mixed_message(&self, msg: MixedMessage, cx: &mut JSContext) -> bool {
        if self.upcast::<WorkerGlobalScope>().is_closing() {
            return false;
        }

        match msg {
            MixedMessage::Devtools(msg) => match msg {
                DevtoolScriptControlMsg::WantsLiveNotifications(_pipe_id, _bool_val) => {},
                DevtoolScriptControlMsg::Eval(code, id, frame_actor_id, reply) => {
                    self.debugger_global.fire_eval(
                        cx,
                        code.into(),
                        id,
                        Some(self.upcast::<WorkerGlobalScope>().worker_id()),
                        frame_actor_id,
                        reply,
                    );
                },
                _ => debug!("got an unusable devtools control message inside the worker!"),
            },
            MixedMessage::SharedWorker(msg) => {
                self.handle_script_event(msg, cx);
            },
            MixedMessage::Control(SharedWorkerControlMsg::Exit) => {
                return false;
            },
            MixedMessage::Timer => {},
        }

        true
    }
}

impl SharedWorkerGlobalScopeMethods<crate::DomTypeHolder> for SharedWorkerGlobalScope {
    /// <https://html.spec.whatwg.org/multipage/#dom-sharedworkerglobalscope-name>
    fn Name(&self) -> DOMString {
        // The name getter steps are to return this's name.
        // Its value represents the name that can be used to obtain a reference to the worker using the SharedWorker constructor.
        self.workerglobalscope.worker_name()
    }

    /// <https://html.spec.whatwg.org/multipage/#dom-sharedworkerglobalscope-close>
    fn Close(&self) {
        // The close() method steps are to close a worker given this.
        self.upcast::<WorkerGlobalScope>().close()
    }

    // <https://html.spec.whatwg.org/multipage/#handler-sharedworkerglobalscope-onconnect>
    event_handler!(connect, GetOnconnect, SetOnconnect);
}
