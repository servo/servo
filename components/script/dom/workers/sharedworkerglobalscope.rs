/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

#![cfg_attr(crown, allow(crown::jscontext_first_arg))]

use std::collections::VecDeque;
use std::sync::Arc;
use std::sync::atomic::AtomicBool;

use content_security_policy::Violation;
use crossbeam_channel::{Receiver, Sender, unbounded};
#[cfg(feature = "devtools")]
use devtools_traits::DevtoolScriptControlMsg;
use dom_struct::dom_struct;
use fonts::FontContext;
use js::context::JSContext;
use js::conversions::ToJSValConvertible;
use js::jsval::UndefinedValue;
use net_traits::image_cache::ImageCache;
use net_traits::request::{
    Destination, InsecureRequestsPolicy, Origin, PreloadedResources, RequestClient,
};
use script_bindings::cell::DomRefCell;
use script_bindings::interfaces::HasOrigin;
use script_bindings::trace::RootedTraceableBox;
#[cfg(feature = "devtools")]
use servo_base::generic_channel::GenericReceiver;
use servo_base::generic_channel::RoutedReceiver;
use servo_base::id::{BrowsingContextId, MessagePortId, ScriptEventLoopId, WebViewId, WorkerId};
use servo_constellation_traits::{MessagePortImpl, SharedWorkerType, WorkerGlobalScopeInit};
use servo_url::{ImmutableOrigin, MutableOrigin, ServoUrl};
use style::thread_state::{self, ThreadState};
use stylo_atoms::Atom;

use crate::dom::abstractworker::WorkerScriptMsg;
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
#[cfg(feature = "devtools")]
use crate::dom::types::DebuggerGlobalScope;
#[cfg(feature = "webgpu")]
use crate::dom::webgpu::identityhub::IdentityHub;
use crate::dom::workerglobalscope::{
    WorkerDebuggerGlobalScope, WorkerDevtoolsControlMsg, WorkerGlobalScope,
};
use crate::messaging::{CommonScriptMsg, ScriptEventLoopReceiver, ScriptEventLoopSender};
use crate::runtime::script_runtime::{IntroductionType, Runtime};
use crate::scripting::script_module::fetch_a_module_script_graph;
use crate::sharedworker_host::{SharedWorkerStart, SharedWorkerThreadControl};
use crate::sharedworker_manager::SharedWorkerEvent;
use crate::tasks::task_queue::{QueuedTask, QueuedTaskConversion, TaskQueue};

#[derive(MallocSizeOf)]
pub(crate) enum SharedWorkerScriptMsg {
    CommonWorker(WorkerScriptMsg),
    Connect {
        connection_id: MessagePortId,
        port: MessagePortImpl,
    },
    WakeUp,
}

pub(crate) enum SharedWorkerControlMsg {
    Exit,
    SetNetworkOnlineState(bool),
}

pub(crate) enum MixedMessage {
    SharedWorker(SharedWorkerScriptMsg),
    #[cfg(feature = "devtools")]
    Devtools(DevtoolScriptControlMsg),
    Control(SharedWorkerControlMsg),
    Timer,
}

impl QueuedTaskConversion for SharedWorkerScriptMsg {
    fn pipeline_id(&self) -> Option<servo_base::id::PipelineId> {
        None
    }

    fn into_queued_task(self) -> Option<QueuedTask> {
        let Self::CommonWorker(WorkerScriptMsg::Common(CommonScriptMsg::Task(
            event_category,
            task,
            pipeline_id,
            task_source,
        ))) = self
        else {
            return None;
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
        Self::CommonWorker(WorkerScriptMsg::Common(CommonScriptMsg::Task(
            queued_task.event_category,
            queued_task.task,
            queued_task.pipeline_id,
            queued_task.task_source,
        )))
    }

    fn inactive_msg() -> Self {
        panic!("Workers should never receive messages marked as inactive");
    }

    fn wake_up_msg() -> Self {
        Self::WakeUp
    }

    fn is_wake_up(&self) -> bool {
        matches!(self, Self::WakeUp)
    }
}

unsafe_no_jsmanaged_fields!(TaskQueue<SharedWorkerScriptMsg>);

/// <https://html.spec.whatwg.org/multipage/#shared-workers-and-the-sharedworkerglobalscope-interface>
#[dom_struct]
pub(crate) struct SharedWorkerGlobalScope {
    workerglobalscope: WorkerGlobalScope,
    #[no_trace]
    webview_id: WebViewId,
    task_queue: TaskQueue<SharedWorkerScriptMsg>,
    own_sender: Sender<SharedWorkerScriptMsg>,
    #[no_trace]
    events: Sender<(WorkerId, SharedWorkerEvent)>,
    #[ignore_malloc_size_of = "ImageCache"]
    #[no_trace]
    image_cache: Arc<dyn ImageCache>,
    #[no_trace]
    browsing_context: Option<BrowsingContextId>,
    pending_connect: DomRefCell<VecDeque<Dom<MessagePort>>>,
    #[no_trace]
    control_receiver: Receiver<SharedWorkerControlMsg>,
    #[cfg(feature = "devtools")]
    debugger_global: Dom<DebuggerGlobalScope>,
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

    #[cfg(feature = "devtools")]
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
    fn new(
        init: WorkerGlobalScopeInit,
        webview_id: WebViewId,
        worker_name: DOMString,
        worker_type: WorkerType,
        worker_url: ServoUrl,
        events: Sender<(WorkerId, SharedWorkerEvent)>,
        from_devtools_receiver: Option<RoutedReceiver<WorkerDevtoolsControlMsg>>,
        runtime: Runtime,
        own_sender: Sender<SharedWorkerScriptMsg>,
        receiver: Receiver<SharedWorkerScriptMsg>,
        closing: Arc<AtomicBool>,
        image_cache: Arc<dyn ImageCache>,
        browsing_context: Option<BrowsingContextId>,
        #[cfg(feature = "webgpu")] gpu_id_hub: Arc<IdentityHub>,
        control_receiver: Receiver<SharedWorkerControlMsg>,
        insecure_requests_policy: InsecureRequestsPolicy,
        font_context: Arc<FontContext>,
        debugger_global: Option<&WorkerDebuggerGlobalScope>,
        cx: &mut JSContext,
    ) -> DomRoot<Self> {
        let scope = Box::new(Self {
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
                Some(ScriptEventLoopSender::SharedWorker(own_sender.clone())),
            ),
            webview_id,
            task_queue: TaskQueue::new(receiver),
            own_sender,
            events,
            image_cache,
            browsing_context,
            pending_connect: DomRefCell::new(VecDeque::new()),
            control_receiver,
            #[cfg(feature = "devtools")]
            debugger_global: Dom::from_ref(
                debugger_global.expect("debugger global must exist when devtools is enabled"),
            ),
        });
        SharedWorkerGlobalScopeBinding::Wrap::<crate::DomTypeHolder>(cx, &scope.origin(), scope)
    }

    /// <https://html.spec.whatwg.org/multipage/#run-a-worker>
    #[expect(unsafe_code)]
    #[allow(clippy::too_many_arguments)]
    pub(crate) fn run_shared_worker_scope(
        start: SharedWorkerStart,
        own_sender: Sender<SharedWorkerScriptMsg>,
        receiver: Receiver<SharedWorkerScriptMsg>,
        control: Arc<SharedWorkerThreadControl>,
        control_receiver: Receiver<SharedWorkerControlMsg>,
        image_cache: Arc<dyn ImageCache>,
        font_context: Arc<FontContext>,
        #[cfg(feature = "webgpu")] gpu_id_hub: Arc<IdentityHub>,
        #[cfg(feature = "devtools")] from_devtools_receiver: GenericReceiver<
            DevtoolScriptControlMsg,
        >,
    ) {
        // Step 4. Let agent be the result of obtaining a dedicated/shared worker agent
        // given outside settings and is shared. Run the rest of these steps in that
        // agent.
        thread_state::initialize(ThreadState::SCRIPT | ThreadState::IN_WORKER);
        ScriptEventLoopId::install(start.event_loop_id);
        let creation = start.creation;
        let worker_url = creation.url.into_url();
        let mut init = creation.init;
        let pipeline_id = init.pipeline_id;
        let webview_id = creation.webview_id;
        let referrer = creation.referrer;
        let request_client = RequestClient {
            preloaded_resources: PreloadedResources::default(),
            policy_container: creation.policy_container,
            origin: Origin::Origin(init.origin.clone()),
            is_nested_browsing_context: creation.is_nested_browsing_context,
            insecure_requests_policy: creation.insecure_requests_policy,
            upgrade_insecure_navigations_set: init
                .inherited_upgrade_insecure_navigations_set
                .clone(),
            has_trustworthy_ancestor_origin: creation.has_trustworthy_ancestor_origin,
            network_partition_key: creation.top_level_site,
        };
        // <https://html.spec.whatwg.org/multipage/#set-up-a-worker-environment-settings-object>
        // Step 3. Let origin be a unique opaque origin if worker global scope's url's scheme is "data"; otherwise outside settings's origin.
        if worker_url.scheme() == "data" {
            init.origin = if init.inherited_secure_context == Some(true) {
                ImmutableOrigin::new_opaque_data_url_worker()
            } else {
                ImmutableOrigin::new_opaque()
            };
        }
        let event_loop_sender = ScriptEventLoopSender::SharedWorker(own_sender.clone());
        let runtime = Runtime::new(Some(event_loop_sender.clone()));
        let mut cx = unsafe { runtime.cx() };
        let cx = &mut cx;

        #[cfg(feature = "devtools")]
        let debugger_global = DebuggerGlobalScope::new_if_needed(
            cx,
            pipeline_id,
            &init.to_devtools_sender,
            &init.from_devtools_sender,
            &init.mem_profiler_chan,
            &init.time_profiler_chan,
            &init.script_to_constellation_chan,
            &init.script_to_embedder_chan,
            &init.resource_threads,
            &init.storage_threads,
            #[cfg(feature = "webgpu")]
            &gpu_id_hub,
        );
        #[cfg(feature = "devtools")]
        let (devtools_receiver, debugger_global_for_scope, devtools_enabled) = (
            Some(from_devtools_receiver.route_preserving_errors()),
            debugger_global.as_deref(),
            init.to_devtools_sender.is_some(),
        );
        #[cfg(not(feature = "devtools"))]
        let (devtools_receiver, debugger_global_for_scope) = (None, None);

        // <https://html.spec.whatwg.org/multipage/#run-a-worker>
        // Step 10.3. Set worker global scope's type to options["type"].
        let worker_type = match creation.options.worker_type {
            SharedWorkerType::Classic => WorkerType::Classic,
            SharedWorkerType::Module => WorkerType::Module,
        };
        // Step 8. Set worker global scope's name to options["name"].
        let global = Self::new(
            init,
            webview_id,
            creation.options.name.into(),
            worker_type,
            worker_url.url(),
            start.events,
            devtools_receiver,
            runtime,
            own_sender,
            receiver,
            control.closing(),
            image_cache,
            Some(creation.browsing_context_id),
            #[cfg(feature = "webgpu")]
            gpu_id_hub,
            control_receiver,
            creation.insecure_requests_policy,
            font_context,
            debugger_global_for_scope,
            cx,
        );

        struct ClearJsRuntime<'a> {
            scope: &'a WorkerGlobalScope,
        }
        impl Drop for ClearJsRuntime<'_> {
            fn drop(&mut self) {
                self.scope.upcast::<GlobalScope>().tear_down();
                self.scope.clear_js_runtime();
            }
        }
        let scope = global.upcast::<WorkerGlobalScope>();
        let _clear_js_runtime = ClearJsRuntime { scope };
        scope.set_shared_worker_control(control);
        let global_scope = global.upcast::<GlobalScope>();
        #[cfg(feature = "devtools")]
        if devtools_enabled && let Some(debugger_global) = &debugger_global {
            debugger_global.fire_add_debuggee(
                cx,
                global_scope,
                pipeline_id,
                Some(scope.worker_id()),
            );
        }
        // <https://html.spec.whatwg.org/multipage/#dom-sharedworker>
        // Step 11.5.2. Let workerIsSecureContext be true if insideSettings is a secure context; otherwise, false.
        if !global.send_event(SharedWorkerEvent::ScopeCreated {
            secure_context: global_scope.is_secure_context(),
        }) || scope.is_closing()
        {
            return;
        }
        // <https://html.spec.whatwg.org/multipage/#run-a-worker>
        // Step 11. Let destination be "sharedworker" if is shared is true, and
        // "worker" otherwise.
        // Step 12. Obtain script by switching on options["type"]:
        match worker_type {
            WorkerType::Classic => fetch_a_classic_worker_script(
                scope,
                worker_url,
                request_client,
                Destination::SharedWorker,
                Some(webview_id),
                referrer,
            ),
            WorkerType::Module => {
                let worker_scope = DomRoot::from_ref(scope);
                fetch_a_module_script_graph(
                    cx,
                    global_scope,
                    worker_url,
                    request_client,
                    Destination::SharedWorker,
                    referrer,
                    creation.options.credentials,
                    Some(IntroductionType::WORKER),
                    move |cx, module_tree| {
                        let script = module_tree
                            .map(|module| Script::Module(RootedTraceableBox::new(module)));
                        worker_scope.on_complete(cx, script);
                    },
                );
            },
        }
        global_scope.mem_profiler_chan().run_with_memory_reporting(
            || {
                // Step 15 of onComplete. Event loop: Run the responsible event loop specified by inside settings until it is destroyed.
                while !scope.is_closing() {
                    run_worker_event_loop(&*global, None, cx);
                }
            },
            format!("shared-worker-reporter-{}", scope.worker_id()),
            event_loop_sender,
            CommonScriptMsg::CollectReports,
        );
    }

    fn send_event(&self, event: SharedWorkerEvent) -> bool {
        if let Err(error) = self
            .events
            .send((self.workerglobalscope.worker_id(), event))
        {
            warn!("Failed to notify SharedWorker manager: {error}");
            self.workerglobalscope.close();
            return false;
        }
        true
    }

    pub(crate) fn event_loop_sender(&self) -> ScriptEventLoopSender {
        ScriptEventLoopSender::SharedWorker(self.own_sender.clone())
    }

    pub(crate) fn webview_id(&self) -> WebViewId {
        self.webview_id
    }

    pub(crate) fn image_cache(&self) -> Arc<dyn ImageCache> {
        self.image_cache.clone()
    }

    pub(crate) fn browsing_context(&self) -> Option<BrowsingContextId> {
        self.browsing_context
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
        // Step 1.1. Queue a global task on the DOM manipulation task source given
        // worker's relevant global object to fire an event named error at worker.
        self.send_event(SharedWorkerEvent::ScriptLoadFailed);
    }

    pub(crate) fn report_csp_violations(&self, violations: Vec<Violation>) {
        self.send_event(SharedWorkerEvent::CspViolations(violations));
    }

    /// Step 11 of onComplete of <https://html.spec.whatwg.org/multipage/#run-a-worker>
    pub(crate) fn enable_outside_port_message_queue(&self) {
        self.send_event(SharedWorkerEvent::ScriptLoaded);
    }

    pub(crate) fn notify_closing(&self) {
        self.send_event(SharedWorkerEvent::Closing);
    }

    /// Step 3 of onComplete of <https://html.spec.whatwg.org/multipage/#run-a-worker>
    fn handle_connect(&self, port: MessagePortImpl, cx: &mut JSContext) -> DomRoot<MessagePort> {
        // Let inside port be a new MessagePort object in inside settings's realm.
        let inside_port = MessagePort::new_transferred(
            cx,
            self.upcast::<GlobalScope>(),
            *port.message_port_id(),
            port.entangled_port_id(),
        );
        self.upcast::<GlobalScope>()
            .track_message_port(&inside_port, Some(port));
        inside_port
    }

    /// Step 13 of onComplete of <https://html.spec.whatwg.org/multipage/#run-a-worker>
    // Step 13. If is shared is true, then queue a global task on the DOM manipulation task source given worker global scope to fire an event named connect at worker global scope, using MessageEvent, with the data attribute initialized to the empty string, the ports attribute initialized to a new frozen array containing inside port, and the source attribute initialized to inside port.
    fn dispatch_connect_event(&self, inside_port: &MessagePort) {
        let worker_global = Trusted::new(self);
        let inside_port = Trusted::new(inside_port);
        self.upcast::<GlobalScope>().task_manager().dom_manipulation_task_source().queue(
            task!(sharedworker_connect_event: move |cx| {
                let worker_global = worker_global.root();
                let inside_port = inside_port.root();
                rooted!(&in(cx) let mut data = UndefinedValue());
                DOMString::new().to_jsval(cx, data.handle_mut());
                let source = WindowProxyOrMessagePortOrServiceWorker::MessagePort(inside_port.clone());
                let event = MessageEvent::new(
                    cx,
                    worker_global.upcast::<GlobalScope>(),
                    Atom::from("connect"),
                    false,
                    false,
                    data.handle(),
                    DOMString::new(),
                    Some(&source),
                    DOMString::new(),
                    vec![inside_port],
                );
                event.upcast::<Event>().fire(cx, worker_global.upcast::<EventTarget>());
            }),
        );
    }

    pub(crate) fn fire_pending_connect(&self, _cx: &mut JSContext) {
        loop {
            let port = self
                .pending_connect
                .borrow_mut()
                .pop_front()
                .map(|port| port.as_rooted());
            let Some(port) = port else {
                break;
            };
            if self.workerglobalscope.is_closing() {
                return;
            }
            // Step 13. If is shared is true, then queue a global task on the DOM manipulation task source given worker global scope to fire an event named connect at worker global scope, using MessageEvent, with the data attribute initialized to the empty string, the ports attribute initialized to a new frozen array containing inside port, and the source attribute initialized to inside port.
            self.dispatch_connect_event(&port);
        }
    }

    fn handle_script_event(&self, msg: SharedWorkerScriptMsg, cx: &mut JSContext) {
        match msg {
            SharedWorkerScriptMsg::CommonWorker(WorkerScriptMsg::Common(msg)) => {
                self.workerglobalscope.process_event(msg, cx);
            },
            SharedWorkerScriptMsg::Connect {
                connection_id,
                port,
            } => {
                let port = self.handle_connect(port, cx);
                self.send_event(SharedWorkerEvent::PortReceived(connection_id));
                if self.workerglobalscope.is_execution_ready() {
                    // Step 13. If is shared is true, then queue a global task on the DOM manipulation task source given worker global scope to fire an event named connect at worker global scope, using MessageEvent, with the data attribute initialized to the empty string, the ports attribute initialized to a new frozen array containing inside port, and the source attribute initialized to inside port.
                    self.dispatch_connect_event(&port);
                } else {
                    // Step 13. If is shared is true, then queue a global task on the DOM manipulation task source given worker global scope to fire an event named connect at worker global scope, using MessageEvent, with the data attribute initialized to the empty string, the ports attribute initialized to a new frozen array containing inside port, and the source attribute initialized to inside port.
                    self.pending_connect
                        .borrow_mut()
                        .push_back(Dom::from_ref(&*port));
                }
            },
            SharedWorkerScriptMsg::CommonWorker(WorkerScriptMsg::DOMMessage(_)) => {
                debug_assert!(
                    false,
                    "SharedWorker messages must arrive through MessagePorts"
                );
            },
            SharedWorkerScriptMsg::WakeUp => {},
        }
    }

    fn handle_mixed_message(&self, msg: MixedMessage, cx: &mut JSContext) -> bool {
        if self.workerglobalscope.is_closing() {
            return false;
        }
        match msg {
            #[cfg(feature = "devtools")]
            MixedMessage::Devtools(msg) => match msg {
                DevtoolScriptControlMsg::WantsLiveNotifications(_, _) => {},
                DevtoolScriptControlMsg::Eval(code, id, frame_actor_id, eager, reply) => {
                    self.debugger_global.fire_eval(
                        cx,
                        code.into(),
                        id,
                        Some(self.workerglobalscope.worker_id()),
                        frame_actor_id,
                        eager,
                        reply,
                    );
                },
                _ => debug!("Unusable devtools control message inside SharedWorker"),
            },
            MixedMessage::SharedWorker(msg) => self.handle_script_event(msg, cx),
            MixedMessage::Control(SharedWorkerControlMsg::Exit) => {
                self.workerglobalscope.close();
                return false;
            },
            MixedMessage::Control(SharedWorkerControlMsg::SetNetworkOnlineState(online)) => {
                self.workerglobalscope.set_network_online_state(online);
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
        self.workerglobalscope.close();
    }

    // <https://html.spec.whatwg.org/multipage/#handler-sharedworkerglobalscope-onconnect>
    event_handler!(connect, GetOnconnect, SetOnconnect);
}

impl HasOrigin for SharedWorkerGlobalScope {
    fn origin(&self) -> MutableOrigin {
        self.workerglobalscope.origin()
    }
}
