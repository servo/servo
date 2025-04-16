/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

//! The script thread is the thread that owns the DOM in memory, runs JavaScript, and triggers
//! layout. It's in charge of processing events for all same-origin pages in a frame
//! tree, and manages the entire lifetime of pages in the frame tree from initial request to
//! teardown.
//!
//! Page loads follow a two-step process. When a request for a new page load is received, the
//! network request is initiated and the relevant data pertaining to the new page is stashed.
//! While the non-blocking request is ongoing, the script thread is free to process further events,
//! noting when they pertain to ongoing loads (such as resizes/viewport adjustments). When the
//! initial response is received for an ongoing load, the second phase starts - the frame tree
//! entry is created, along with the Window and Document objects, and the appropriate parser
//! takes over the response body. Once parsing is complete, the document lifecycle for loading
//! a page runs its course and the script thread returns to processing events in the main event
//! loop.

use std::cell::{Cell, RefCell};
use std::collections::{HashMap, HashSet};
use std::default::Default;
use std::option::Option;
use std::rc::Rc;
use std::result::Result;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::thread;
use std::time::{Duration, Instant, SystemTime};

use background_hang_monitor_api::{
    BackgroundHangMonitor, BackgroundHangMonitorExitSignal, HangAnnotation, MonitoredComponentId,
    MonitoredComponentType,
};
use base::cross_process_instant::CrossProcessInstant;
use base::id::{BrowsingContextId, HistoryStateId, PipelineId, PipelineNamespace, WebViewId};
use canvas_traits::webgl::WebGLPipeline;
use chrono::{DateTime, Local};
use compositing_traits::CrossProcessCompositorApi;
use constellation_traits::{
    JsEvalResult, LoadData, LoadOrigin, NavigationHistoryBehavior, ScriptToConstellationChan,
    ScriptToConstellationMessage, ScrollState, StructuredSerializedData, WindowSizeType,
};
use content_security_policy::{self as csp};
use crossbeam_channel::unbounded;
use data_url::mime::Mime;
use devtools_traits::{
    CSSError, DevtoolScriptControlMsg, DevtoolsPageInfo, NavigationState,
    ScriptToDevtoolsControlMsg, WorkerId,
};
use embedder_traits::user_content_manager::UserContentManager;
use embedder_traits::{
    CompositorHitTestResult, EmbedderMsg, InputEvent, MediaSessionActionType, Theme,
    ViewportDetails, WebDriverScriptCommand,
};
use euclid::default::Rect;
use fonts::{FontContext, SystemFontServiceProxy};
use headers::{HeaderMapExt, LastModified, ReferrerPolicy as ReferrerPolicyHeader};
use html5ever::{local_name, namespace_url, ns};
use http::header::REFRESH;
use hyper_serde::Serde;
use ipc_channel::ipc;
use ipc_channel::router::ROUTER;
use js::glue::GetWindowProxyClass;
use js::jsapi::{
    JS_AddInterruptCallback, JSContext as UnsafeJSContext, JSTracer, SetWindowProxyClass,
};
use js::jsval::UndefinedValue;
use js::rust::ParentRuntime;
use media::WindowGLContext;
use metrics::MAX_TASK_NS;
use net_traits::image_cache::{ImageCache, PendingImageResponse};
use net_traits::request::{Referrer, RequestId};
use net_traits::response::ResponseInit;
use net_traits::storage_thread::StorageType;
use net_traits::{
    FetchMetadata, FetchResponseListener, FetchResponseMsg, Metadata, NetworkError,
    ResourceFetchTiming, ResourceThreads, ResourceTimingType,
};
use percent_encoding::percent_decode;
use profile_traits::mem::{ProcessReports, ReportsChan};
use profile_traits::time::ProfilerCategory;
use profile_traits::time_profile;
use script_layout_interface::{
    LayoutConfig, LayoutFactory, ReflowGoal, ScriptThreadFactory, node_id_from_scroll_id,
};
use script_traits::{
    ConstellationInputEvent, DiscardBrowsingContext, DocumentActivity, InitialScriptState,
    NewLayoutInfo, Painter, ProgressiveWebMetricType, ScriptThreadMessage, UpdatePipelineIdReason,
};
use servo_config::opts;
use servo_url::{ImmutableOrigin, MutableOrigin, ServoUrl};
use style::dom::OpaqueNode;
use style::thread_state::{self, ThreadState};
use stylo_atoms::Atom;
use timers::{TimerEventRequest, TimerScheduler};
use url::Position;
#[cfg(feature = "webgpu")]
use webgpu_traits::{WebGPUDevice, WebGPUMsg};
use webrender_api::DocumentId;

use crate::document_collection::DocumentCollection;
use crate::document_loader::DocumentLoader;
use crate::dom::bindings::cell::DomRefCell;
use crate::dom::bindings::codegen::Bindings::DocumentBinding::{
    DocumentMethods, DocumentReadyState,
};
use crate::dom::bindings::codegen::Bindings::NavigatorBinding::NavigatorMethods;
use crate::dom::bindings::codegen::Bindings::WindowBinding::WindowMethods;
use crate::dom::bindings::conversions::{
    ConversionResult, FromJSValConvertible, StringificationBehavior,
};
use crate::dom::bindings::inheritance::Castable;
use crate::dom::bindings::refcounted::Trusted;
use crate::dom::bindings::reflector::DomGlobal;
use crate::dom::bindings::root::{
    Dom, DomRoot, MutNullableDom, RootCollection, ThreadLocalStackRoots,
};
use crate::dom::bindings::settings_stack::AutoEntryScript;
use crate::dom::bindings::str::DOMString;
use crate::dom::bindings::trace::{HashMapTracedValues, JSTraceable};
use crate::dom::customelementregistry::{
    CallbackReaction, CustomElementDefinition, CustomElementReactionStack,
};
use crate::dom::document::{
    Document, DocumentSource, FocusType, HasBrowsingContext, IsHTMLDocument, TouchEventResult,
};
use crate::dom::element::Element;
use crate::dom::globalscope::GlobalScope;
use crate::dom::htmlanchorelement::HTMLAnchorElement;
use crate::dom::htmliframeelement::HTMLIFrameElement;
use crate::dom::htmlslotelement::HTMLSlotElement;
use crate::dom::mutationobserver::MutationObserver;
use crate::dom::node::{Node, NodeTraits, ShadowIncluding};
use crate::dom::servoparser::{ParserContext, ServoParser};
#[cfg(feature = "webgpu")]
use crate::dom::webgpu::identityhub::IdentityHub;
use crate::dom::window::Window;
use crate::dom::windowproxy::{CreatorBrowsingContextInfo, WindowProxy};
use crate::dom::worklet::WorkletThreadPool;
use crate::dom::workletglobalscope::WorkletGlobalScopeInit;
use crate::fetch::FetchCanceller;
use crate::messaging::{
    CommonScriptMsg, MainThreadScriptMsg, MixedMessage, ScriptEventLoopSender,
    ScriptThreadReceivers, ScriptThreadSenders,
};
use crate::microtask::{Microtask, MicrotaskQueue};
use crate::mime::{APPLICATION, MimeExt, TEXT, XML};
use crate::navigation::{InProgressLoad, NavigationListener};
use crate::realms::enter_realm;
use crate::script_module::ScriptFetchOptions;
use crate::script_runtime::{
    CanGc, JSContext, JSContextHelper, Runtime, ScriptThreadEventCategory, ThreadSafeJSContext,
};
use crate::task_queue::TaskQueue;
use crate::task_source::{SendableTaskSource, TaskSourceName};
use crate::{devtools, webdriver_handlers};

thread_local!(static SCRIPT_THREAD_ROOT: Cell<Option<*const ScriptThread>> = const { Cell::new(None) });

fn with_optional_script_thread<R>(f: impl FnOnce(Option<&ScriptThread>) -> R) -> R {
    SCRIPT_THREAD_ROOT.with(|root| {
        f(root
            .get()
            .and_then(|script_thread| unsafe { script_thread.as_ref() }))
    })
}

pub(crate) fn with_script_thread<R: Default>(f: impl FnOnce(&ScriptThread) -> R) -> R {
    with_optional_script_thread(|script_thread| script_thread.map(f).unwrap_or_default())
}

/// # Safety
///
/// The `JSTracer` argument must point to a valid `JSTracer` in memory. In addition,
/// implementors of this method must ensure that all active objects are properly traced
/// or else the garbage collector may end up collecting objects that are still reachable.
pub(crate) unsafe fn trace_thread(tr: *mut JSTracer) {
    with_script_thread(|script_thread| {
        trace!("tracing fields of ScriptThread");
        script_thread.trace(tr);
    })
}

// We borrow the incomplete parser contexts mutably during parsing,
// which is fine except that parsing can trigger evaluation,
// which can trigger GC, and so we can end up tracing the script
// thread during parsing. For this reason, we don't trace the
// incomplete parser contexts during GC.
pub(crate) struct IncompleteParserContexts(RefCell<Vec<(PipelineId, ParserContext)>>);

unsafe_no_jsmanaged_fields!(TaskQueue<MainThreadScriptMsg>);

#[derive(JSTraceable)]
// ScriptThread instances are rooted on creation, so this is okay
#[cfg_attr(crown, allow(crown::unrooted_must_root))]
pub struct ScriptThread {
    /// <https://html.spec.whatwg.org/multipage/#last-render-opportunity-time>
    last_render_opportunity_time: DomRefCell<Option<Instant>>,
    /// The documents for pipelines managed by this thread
    documents: DomRefCell<DocumentCollection>,
    /// The window proxies known by this thread
    /// TODO: this map grows, but never shrinks. Issue #15258.
    window_proxies: DomRefCell<HashMapTracedValues<BrowsingContextId, Dom<WindowProxy>>>,
    /// A list of data pertaining to loads that have not yet received a network response
    incomplete_loads: DomRefCell<Vec<InProgressLoad>>,
    /// A vector containing parser contexts which have not yet been fully processed
    incomplete_parser_contexts: IncompleteParserContexts,
    /// Image cache for this script thread.
    #[no_trace]
    image_cache: Arc<dyn ImageCache>,

    /// A [`ScriptThreadReceivers`] holding all of the incoming `Receiver`s for messages
    /// to this [`ScriptThread`].
    receivers: ScriptThreadReceivers,

    /// A [`ScriptThreadSenders`] that holds all outgoing sending channels necessary to communicate
    /// to other parts of Servo.
    senders: ScriptThreadSenders,

    /// A handle to the resource thread. This is an `Arc` to avoid running out of file descriptors if
    /// there are many iframes.
    #[no_trace]
    resource_threads: ResourceThreads,

    /// A queue of tasks to be executed in this script-thread.
    task_queue: TaskQueue<MainThreadScriptMsg>,

    /// The dedicated means of communication with the background-hang-monitor for this script-thread.
    #[no_trace]
    background_hang_monitor: Box<dyn BackgroundHangMonitor>,
    /// A flag set to `true` by the BHM on exit, and checked from within the interrupt handler.
    closing: Arc<AtomicBool>,

    /// A [`TimerScheduler`] used to schedule timers for this [`ScriptThread`]. Timers are handled
    /// in the [`ScriptThread`] event loop.
    #[no_trace]
    timer_scheduler: RefCell<TimerScheduler>,

    /// A proxy to the `SystemFontService` to use for accessing system font lists.
    #[no_trace]
    system_font_service: Arc<SystemFontServiceProxy>,

    /// The JavaScript runtime.
    js_runtime: Rc<Runtime>,

    /// The topmost element over the mouse.
    topmost_mouse_over_target: MutNullableDom<Element>,

    /// List of pipelines that have been owned and closed by this script thread.
    #[no_trace]
    closed_pipelines: DomRefCell<HashSet<PipelineId>>,

    /// <https://html.spec.whatwg.org/multipage/#microtask-queue>
    microtask_queue: Rc<MicrotaskQueue>,

    /// Microtask Queue for adding support for mutation observer microtasks
    mutation_observer_microtask_queued: Cell<bool>,

    /// The unit of related similar-origin browsing contexts' list of MutationObserver objects
    mutation_observers: DomRefCell<Vec<Dom<MutationObserver>>>,

    /// <https://dom.spec.whatwg.org/#signal-slot-list>
    signal_slots: DomRefCell<Vec<Dom<HTMLSlotElement>>>,

    /// A handle to the WebGL thread
    #[no_trace]
    webgl_chan: Option<WebGLPipeline>,

    /// The WebXR device registry
    #[no_trace]
    #[cfg(feature = "webxr")]
    webxr_registry: Option<webxr_api::Registry>,

    /// The worklet thread pool
    worklet_thread_pool: DomRefCell<Option<Rc<WorkletThreadPool>>>,

    /// A list of pipelines containing documents that finished loading all their blocking
    /// resources during a turn of the event loop.
    docs_with_no_blocking_loads: DomRefCell<HashSet<Dom<Document>>>,

    /// <https://html.spec.whatwg.org/multipage/#custom-element-reactions-stack>
    custom_element_reaction_stack: CustomElementReactionStack,

    /// The Webrender Document ID associated with this thread.
    #[no_trace]
    webrender_document: DocumentId,

    /// Cross-process access to the compositor's API.
    #[no_trace]
    compositor_api: CrossProcessCompositorApi,

    /// Periodically print out on which events script threads spend their processing time.
    profile_script_events: bool,

    /// Print Progressive Web Metrics to console.
    print_pwm: bool,

    /// Emits notifications when there is a relayout.
    relayout_event: bool,

    /// Unminify Javascript.
    unminify_js: bool,

    /// Directory with stored unminified scripts
    local_script_source: Option<String>,

    /// Unminify Css.
    unminify_css: bool,

    /// User content manager
    #[no_trace]
    user_content_manager: UserContentManager,

    /// Application window's GL Context for Media player
    #[no_trace]
    player_context: WindowGLContext,

    /// A set of all nodes ever created in this script thread
    node_ids: DomRefCell<HashSet<String>>,

    /// Code is running as a consequence of a user interaction
    is_user_interacting: Cell<bool>,

    /// Identity manager for WebGPU resources
    #[no_trace]
    #[cfg(feature = "webgpu")]
    gpu_id_hub: Arc<IdentityHub>,

    // Secure context
    inherited_secure_context: Option<bool>,

    /// A factory for making new layouts. This allows layout to depend on script.
    #[no_trace]
    layout_factory: Arc<dyn LayoutFactory>,
}

struct BHMExitSignal {
    closing: Arc<AtomicBool>,
    js_context: ThreadSafeJSContext,
}

impl BackgroundHangMonitorExitSignal for BHMExitSignal {
    fn signal_to_exit(&self) {
        self.closing.store(true, Ordering::SeqCst);
        self.js_context.request_interrupt_callback();
    }
}

#[allow(unsafe_code)]
unsafe extern "C" fn interrupt_callback(_cx: *mut UnsafeJSContext) -> bool {
    let res = ScriptThread::can_continue_running();
    if !res {
        ScriptThread::prepare_for_shutdown();
    }
    res
}

/// In the event of thread panic, all data on the stack runs its destructor. However, there
/// are no reachable, owning pointers to the DOM memory, so it never gets freed by default
/// when the script thread fails. The ScriptMemoryFailsafe uses the destructor bomb pattern
/// to forcibly tear down the JS realms for pages associated with the failing ScriptThread.
struct ScriptMemoryFailsafe<'a> {
    owner: Option<&'a ScriptThread>,
}

impl<'a> ScriptMemoryFailsafe<'a> {
    fn neuter(&mut self) {
        self.owner = None;
    }

    fn new(owner: &'a ScriptThread) -> ScriptMemoryFailsafe<'a> {
        ScriptMemoryFailsafe { owner: Some(owner) }
    }
}

impl Drop for ScriptMemoryFailsafe<'_> {
    #[cfg_attr(crown, allow(crown::unrooted_must_root))]
    fn drop(&mut self) {
        if let Some(owner) = self.owner {
            for (_, document) in owner.documents.borrow().iter() {
                document.window().clear_js_runtime_for_script_deallocation();
            }
        }
    }
}

impl ScriptThreadFactory for ScriptThread {
    fn create(
        state: InitialScriptState,
        layout_factory: Arc<dyn LayoutFactory>,
        system_font_service: Arc<SystemFontServiceProxy>,
        load_data: LoadData,
    ) {
        thread::Builder::new()
            .name(format!("Script{:?}", state.id))
            .spawn(move || {
                thread_state::initialize(ThreadState::SCRIPT | ThreadState::LAYOUT);
                PipelineNamespace::install(state.pipeline_namespace_id);
                WebViewId::install(state.webview_id);
                let roots = RootCollection::new();
                let _stack_roots = ThreadLocalStackRoots::new(&roots);
                let id = state.id;
                let browsing_context_id = state.browsing_context_id;
                let webview_id = state.webview_id;
                let parent_info = state.parent_info;
                let opener = state.opener;
                let memory_profiler_sender = state.memory_profiler_sender.clone();
                let viewport_details = state.viewport_details;

                let script_thread = ScriptThread::new(state, layout_factory, system_font_service);

                SCRIPT_THREAD_ROOT.with(|root| {
                    root.set(Some(&script_thread as *const _));
                });

                let mut failsafe = ScriptMemoryFailsafe::new(&script_thread);

                let origin = MutableOrigin::new(load_data.url.origin());
                script_thread.pre_page_load(InProgressLoad::new(
                    id,
                    browsing_context_id,
                    webview_id,
                    parent_info,
                    opener,
                    viewport_details,
                    origin,
                    load_data,
                ));

                let reporter_name = format!("script-reporter-{:?}", id);
                memory_profiler_sender.run_with_memory_reporting(
                    || {
                        script_thread.start(CanGc::note());

                        let _ = script_thread
                            .senders
                            .content_process_shutdown_sender
                            .send(());
                    },
                    reporter_name,
                    ScriptEventLoopSender::MainThread(script_thread.senders.self_sender.clone()),
                    CommonScriptMsg::CollectReports,
                );

                // This must always be the very last operation performed before the thread completes
                failsafe.neuter();
            })
            .expect("Thread spawning failed");
    }
}

impl ScriptThread {
    pub(crate) fn runtime_handle() -> ParentRuntime {
        with_optional_script_thread(|script_thread| {
            script_thread.unwrap().js_runtime.prepare_for_new_child()
        })
    }

    pub(crate) fn can_continue_running() -> bool {
        with_script_thread(|script_thread| script_thread.can_continue_running_inner())
    }

    pub(crate) fn prepare_for_shutdown() {
        with_script_thread(|script_thread| {
            script_thread.prepare_for_shutdown_inner();
        })
    }

    pub(crate) fn set_mutation_observer_microtask_queued(value: bool) {
        with_script_thread(|script_thread| {
            script_thread.mutation_observer_microtask_queued.set(value);
        })
    }

    pub(crate) fn is_mutation_observer_microtask_queued() -> bool {
        with_script_thread(|script_thread| script_thread.mutation_observer_microtask_queued.get())
    }

    pub(crate) fn add_mutation_observer(observer: &MutationObserver) {
        with_script_thread(|script_thread| {
            script_thread
                .mutation_observers
                .borrow_mut()
                .push(Dom::from_ref(observer));
        })
    }

    pub(crate) fn get_mutation_observers() -> Vec<DomRoot<MutationObserver>> {
        with_script_thread(|script_thread| {
            script_thread
                .mutation_observers
                .borrow()
                .iter()
                .map(|o| DomRoot::from_ref(&**o))
                .collect()
        })
    }

    pub(crate) fn add_signal_slot(observer: &HTMLSlotElement) {
        with_script_thread(|script_thread| {
            script_thread
                .signal_slots
                .borrow_mut()
                .push(Dom::from_ref(observer));
        })
    }

    pub(crate) fn take_signal_slots() -> Vec<DomRoot<HTMLSlotElement>> {
        with_script_thread(|script_thread| {
            script_thread
                .signal_slots
                .take()
                .into_iter()
                .inspect(|slot| {
                    slot.remove_from_signal_slots();
                })
                .map(|slot| slot.as_rooted())
                .collect()
        })
    }

    pub(crate) fn mark_document_with_no_blocked_loads(doc: &Document) {
        with_script_thread(|script_thread| {
            script_thread
                .docs_with_no_blocking_loads
                .borrow_mut()
                .insert(Dom::from_ref(doc));
        })
    }

    pub(crate) fn page_headers_available(
        id: &PipelineId,
        metadata: Option<Metadata>,
        can_gc: CanGc,
    ) -> Option<DomRoot<ServoParser>> {
        with_script_thread(|script_thread| {
            script_thread.handle_page_headers_available(id, metadata, can_gc)
        })
    }

    /// Process a single event as if it were the next event
    /// in the queue for this window event-loop.
    /// Returns a boolean indicating whether further events should be processed.
    pub(crate) fn process_event(msg: CommonScriptMsg) -> bool {
        with_script_thread(|script_thread| {
            if !script_thread.can_continue_running_inner() {
                return false;
            }
            script_thread.handle_msg_from_script(MainThreadScriptMsg::Common(msg));
            true
        })
    }

    /// Schedule a [`TimerEventRequest`] on this [`ScriptThread`]'s [`TimerScheduler`].
    pub(crate) fn schedule_timer(&self, request: TimerEventRequest) {
        self.timer_scheduler.borrow_mut().schedule_timer(request);
    }

    // https://html.spec.whatwg.org/multipage/#await-a-stable-state
    pub(crate) fn await_stable_state(task: Microtask) {
        with_script_thread(|script_thread| {
            script_thread
                .microtask_queue
                .enqueue(task, script_thread.get_cx());
        });
    }

    /// Check that two origins are "similar enough",
    /// for now only used to prevent cross-origin JS url evaluation.
    ///
    /// <https://github.com/whatwg/html/issues/2591>
    pub(crate) fn check_load_origin(source: &LoadOrigin, target: &ImmutableOrigin) -> bool {
        match (source, target) {
            (LoadOrigin::Constellation, _) | (LoadOrigin::WebDriver, _) => {
                // Always allow loads initiated by the constellation or webdriver.
                true
            },
            (_, ImmutableOrigin::Opaque(_)) => {
                // If the target is opaque, allow.
                // This covers newly created about:blank auxiliaries, and iframe with no src.
                // TODO: https://github.com/servo/servo/issues/22879
                true
            },
            (LoadOrigin::Script(source_origin), _) => source_origin == target,
        }
    }

    /// Step 13 of <https://html.spec.whatwg.org/multipage/#navigate>
    pub(crate) fn navigate(
        browsing_context: BrowsingContextId,
        pipeline_id: PipelineId,
        mut load_data: LoadData,
        history_handling: NavigationHistoryBehavior,
    ) {
        with_script_thread(|script_thread| {
            let is_javascript = load_data.url.scheme() == "javascript";
            // If resource is a request whose url's scheme is "javascript"
            // https://html.spec.whatwg.org/multipage/#javascript-protocol
            if is_javascript {
                let window = match script_thread.documents.borrow().find_window(pipeline_id) {
                    None => return,
                    Some(window) => window,
                };
                let global = window.as_global_scope();
                let trusted_global = Trusted::new(global);
                let sender = script_thread
                    .senders
                    .pipeline_to_constellation_sender
                    .clone();
                let task = task!(navigate_javascript: move || {
                    // Important re security. See https://github.com/servo/servo/issues/23373
                    // TODO: check according to https://w3c.github.io/webappsec-csp/#should-block-navigation-request
                    if let Some(window) = trusted_global.root().downcast::<Window>() {
                        if ScriptThread::check_load_origin(&load_data.load_origin, &window.get_url().origin()) {
                            ScriptThread::eval_js_url(&trusted_global.root(), &mut load_data, CanGc::note());
                            sender
                                .send((pipeline_id, ScriptToConstellationMessage::LoadUrl(load_data, history_handling)))
                                .unwrap();
                        }
                    }
                });
                global
                    .task_manager()
                    .dom_manipulation_task_source()
                    .queue(task);
            } else {
                if let Some(ref sender) = script_thread.senders.devtools_server_sender {
                    let _ = sender.send(ScriptToDevtoolsControlMsg::Navigate(
                        browsing_context,
                        NavigationState::Start(load_data.url.clone()),
                    ));
                }

                script_thread
                    .senders
                    .pipeline_to_constellation_sender
                    .send((
                        pipeline_id,
                        ScriptToConstellationMessage::LoadUrl(load_data, history_handling),
                    ))
                    .expect("Sending a LoadUrl message to the constellation failed");
            }
        });
    }

    pub(crate) fn process_attach_layout(new_layout_info: NewLayoutInfo, origin: MutableOrigin) {
        with_script_thread(|script_thread| {
            let pipeline_id = Some(new_layout_info.new_pipeline_id);
            script_thread.profile_event(
                ScriptThreadEventCategory::AttachLayout,
                pipeline_id,
                || {
                    script_thread.handle_new_layout(new_layout_info, origin);
                },
            )
        });
    }

    pub(crate) fn get_top_level_for_browsing_context(
        sender_pipeline: PipelineId,
        browsing_context_id: BrowsingContextId,
    ) -> Option<WebViewId> {
        with_script_thread(|script_thread| {
            script_thread.ask_constellation_for_top_level_info(sender_pipeline, browsing_context_id)
        })
    }

    pub(crate) fn find_document(id: PipelineId) -> Option<DomRoot<Document>> {
        with_script_thread(|script_thread| script_thread.documents.borrow().find_document(id))
    }

    pub(crate) fn set_user_interacting(interacting: bool) {
        with_script_thread(|script_thread| {
            script_thread.is_user_interacting.set(interacting);
        });
    }

    pub(crate) fn is_user_interacting() -> bool {
        with_script_thread(|script_thread| script_thread.is_user_interacting.get())
    }

    pub(crate) fn get_fully_active_document_ids() -> HashSet<PipelineId> {
        with_script_thread(|script_thread| {
            script_thread
                .documents
                .borrow()
                .iter()
                .filter_map(|(id, document)| {
                    if document.is_fully_active() {
                        Some(id)
                    } else {
                        None
                    }
                })
                .fold(HashSet::new(), |mut set, id| {
                    let _ = set.insert(id);
                    set
                })
        })
    }

    pub(crate) fn find_window_proxy(id: BrowsingContextId) -> Option<DomRoot<WindowProxy>> {
        with_script_thread(|script_thread| {
            script_thread
                .window_proxies
                .borrow()
                .get(&id)
                .map(|context| DomRoot::from_ref(&**context))
        })
    }

    pub(crate) fn find_window_proxy_by_name(name: &DOMString) -> Option<DomRoot<WindowProxy>> {
        with_script_thread(|script_thread| {
            for (_, proxy) in script_thread.window_proxies.borrow().iter() {
                if proxy.get_name() == *name {
                    return Some(DomRoot::from_ref(&**proxy));
                }
            }
            None
        })
    }

    pub(crate) fn worklet_thread_pool() -> Rc<WorkletThreadPool> {
        with_optional_script_thread(|script_thread| {
            let script_thread = script_thread.unwrap();
            script_thread
                .worklet_thread_pool
                .borrow_mut()
                .get_or_insert_with(|| {
                    let init = WorkletGlobalScopeInit {
                        to_script_thread_sender: script_thread.senders.self_sender.clone(),
                        resource_threads: script_thread.resource_threads.clone(),
                        mem_profiler_chan: script_thread.senders.memory_profiler_sender.clone(),
                        time_profiler_chan: script_thread.senders.time_profiler_sender.clone(),
                        devtools_chan: script_thread.senders.devtools_server_sender.clone(),
                        to_constellation_sender: script_thread
                            .senders
                            .pipeline_to_constellation_sender
                            .clone(),
                        image_cache: script_thread.image_cache.clone(),
                        #[cfg(feature = "webgpu")]
                        gpu_id_hub: script_thread.gpu_id_hub.clone(),
                        inherited_secure_context: script_thread.inherited_secure_context,
                    };
                    Rc::new(WorkletThreadPool::spawn(init))
                })
                .clone()
        })
    }

    fn handle_register_paint_worklet(
        &self,
        pipeline_id: PipelineId,
        name: Atom,
        properties: Vec<Atom>,
        painter: Box<dyn Painter>,
    ) {
        let Some(window) = self.documents.borrow().find_window(pipeline_id) else {
            warn!("Paint worklet registered after pipeline {pipeline_id} closed.");
            return;
        };

        window
            .layout_mut()
            .register_paint_worklet_modules(name, properties, painter);
    }

    pub(crate) fn push_new_element_queue() {
        with_script_thread(|script_thread| {
            script_thread
                .custom_element_reaction_stack
                .push_new_element_queue();
        })
    }

    pub(crate) fn pop_current_element_queue(can_gc: CanGc) {
        with_script_thread(|script_thread| {
            script_thread
                .custom_element_reaction_stack
                .pop_current_element_queue(can_gc);
        })
    }

    pub(crate) fn enqueue_callback_reaction(
        element: &Element,
        reaction: CallbackReaction,
        definition: Option<Rc<CustomElementDefinition>>,
    ) {
        with_script_thread(|script_thread| {
            script_thread
                .custom_element_reaction_stack
                .enqueue_callback_reaction(element, reaction, definition);
        })
    }

    pub(crate) fn enqueue_upgrade_reaction(
        element: &Element,
        definition: Rc<CustomElementDefinition>,
    ) {
        with_script_thread(|script_thread| {
            script_thread
                .custom_element_reaction_stack
                .enqueue_upgrade_reaction(element, definition);
        })
    }

    pub(crate) fn invoke_backup_element_queue(can_gc: CanGc) {
        with_script_thread(|script_thread| {
            script_thread
                .custom_element_reaction_stack
                .invoke_backup_element_queue(can_gc);
        })
    }

    pub(crate) fn save_node_id(node_id: String) {
        with_script_thread(|script_thread| {
            script_thread.node_ids.borrow_mut().insert(node_id);
        })
    }

    pub(crate) fn has_node_id(node_id: &str) -> bool {
        with_script_thread(|script_thread| script_thread.node_ids.borrow().contains(node_id))
    }

    /// Creates a new script thread.
    pub(crate) fn new(
        state: InitialScriptState,
        layout_factory: Arc<dyn LayoutFactory>,
        system_font_service: Arc<SystemFontServiceProxy>,
    ) -> ScriptThread {
        let (self_sender, self_receiver) = unbounded();
        let runtime = Runtime::new(Some(SendableTaskSource {
            sender: ScriptEventLoopSender::MainThread(self_sender.clone()),
            pipeline_id: state.id,
            name: TaskSourceName::Networking,
            canceller: Default::default(),
        }));
        let cx = runtime.cx();

        unsafe {
            SetWindowProxyClass(cx, GetWindowProxyClass());
            JS_AddInterruptCallback(cx, Some(interrupt_callback));
        }

        // Ask the router to proxy IPC messages from the control port to us.
        let constellation_receiver =
            ROUTER.route_ipc_receiver_to_new_crossbeam_receiver(state.constellation_receiver);

        // Ask the router to proxy IPC messages from the devtools to us.
        let devtools_server_sender = state.devtools_server_sender;
        let (ipc_devtools_sender, ipc_devtools_receiver) = ipc::channel().unwrap();
        let devtools_server_receiver = devtools_server_sender
            .as_ref()
            .map(|_| ROUTER.route_ipc_receiver_to_new_crossbeam_receiver(ipc_devtools_receiver))
            .unwrap_or_else(crossbeam_channel::never);

        let task_queue = TaskQueue::new(self_receiver, self_sender.clone());

        let closing = Arc::new(AtomicBool::new(false));
        let background_hang_monitor_exit_signal = BHMExitSignal {
            closing: closing.clone(),
            js_context: runtime.thread_safe_js_context(),
        };

        let background_hang_monitor = state.background_hang_monitor_register.register_component(
            MonitoredComponentId(state.id, MonitoredComponentType::Script),
            Duration::from_millis(1000),
            Duration::from_millis(5000),
            Some(Box::new(background_hang_monitor_exit_signal)),
        );

        let (image_cache_sender, image_cache_receiver) = unbounded();
        let (ipc_image_cache_sender, ipc_image_cache_receiver) = ipc::channel().unwrap();
        ROUTER.add_typed_route(
            ipc_image_cache_receiver,
            Box::new(move |message| {
                let _ = image_cache_sender.send(message.unwrap());
            }),
        );

        let receivers = ScriptThreadReceivers {
            constellation_receiver,
            image_cache_receiver,
            devtools_server_receiver,
            // Initialized to `never` until WebGPU is initialized.
            #[cfg(feature = "webgpu")]
            webgpu_receiver: RefCell::new(crossbeam_channel::never()),
        };

        let opts = opts::get();
        let senders = ScriptThreadSenders {
            self_sender,
            #[cfg(feature = "bluetooth")]
            bluetooth_sender: state.bluetooth_sender,
            constellation_sender: state.constellation_sender,
            pipeline_to_constellation_sender: state.pipeline_to_constellation_sender.sender.clone(),
            image_cache_sender: ipc_image_cache_sender,
            time_profiler_sender: state.time_profiler_sender,
            memory_profiler_sender: state.memory_profiler_sender,
            devtools_server_sender,
            devtools_client_to_script_thread_sender: ipc_devtools_sender,
            content_process_shutdown_sender: state.content_process_shutdown_sender,
        };

        ScriptThread {
            documents: DomRefCell::new(DocumentCollection::default()),
            last_render_opportunity_time: Default::default(),
            window_proxies: DomRefCell::new(HashMapTracedValues::new()),
            incomplete_loads: DomRefCell::new(vec![]),
            incomplete_parser_contexts: IncompleteParserContexts(RefCell::new(vec![])),
            senders,
            receivers,
            image_cache: state.image_cache.clone(),
            resource_threads: state.resource_threads,
            task_queue,
            background_hang_monitor,
            closing,
            timer_scheduler: Default::default(),
            microtask_queue: runtime.microtask_queue.clone(),
            js_runtime: Rc::new(runtime),
            topmost_mouse_over_target: MutNullableDom::new(Default::default()),
            closed_pipelines: DomRefCell::new(HashSet::new()),
            mutation_observer_microtask_queued: Default::default(),
            mutation_observers: Default::default(),
            signal_slots: Default::default(),
            system_font_service,
            webgl_chan: state.webgl_chan,
            #[cfg(feature = "webxr")]
            webxr_registry: state.webxr_registry,
            worklet_thread_pool: Default::default(),
            docs_with_no_blocking_loads: Default::default(),
            custom_element_reaction_stack: CustomElementReactionStack::new(),
            webrender_document: state.webrender_document,
            compositor_api: state.compositor_api,
            profile_script_events: opts.debug.profile_script_events,
            print_pwm: opts.print_pwm,
            relayout_event: opts.debug.relayout_event,
            unminify_js: opts.unminify_js,
            local_script_source: opts.local_script_source.clone(),
            unminify_css: opts.unminify_css,
            user_content_manager: state.user_content_manager,
            player_context: state.player_context,
            node_ids: Default::default(),
            is_user_interacting: Cell::new(false),
            #[cfg(feature = "webgpu")]
            gpu_id_hub: Arc::new(IdentityHub::default()),
            inherited_secure_context: state.inherited_secure_context,
            layout_factory,
        }
    }

    #[allow(unsafe_code)]
    pub(crate) fn get_cx(&self) -> JSContext {
        unsafe { JSContext::from_ptr(self.js_runtime.cx()) }
    }

    /// Check if we are closing.
    fn can_continue_running_inner(&self) -> bool {
        if self.closing.load(Ordering::SeqCst) {
            return false;
        }
        true
    }

    /// We are closing, ensure no script can run and potentially hang.
    fn prepare_for_shutdown_inner(&self) {
        let docs = self.documents.borrow();
        for (_, document) in docs.iter() {
            document
                .owner_global()
                .task_manager()
                .cancel_all_tasks_and_ignore_future_tasks();
        }
    }

    /// Starts the script thread. After calling this method, the script thread will loop receiving
    /// messages on its port.
    pub(crate) fn start(&self, can_gc: CanGc) {
        debug!("Starting script thread.");
        while self.handle_msgs(can_gc) {
            // Go on...
            debug!("Running script thread.");
        }
        debug!("Stopped script thread.");
    }

    /// Process a compositor mouse move event.
    fn process_mouse_move_event(
        &self,
        document: &Document,
        hit_test_result: Option<CompositorHitTestResult>,
        pressed_mouse_buttons: u16,
        can_gc: CanGc,
    ) {
        // Get the previous target temporarily
        let prev_mouse_over_target = self.topmost_mouse_over_target.get();

        unsafe {
            document.handle_mouse_move_event(
                hit_test_result,
                pressed_mouse_buttons,
                &self.topmost_mouse_over_target,
                can_gc,
            )
        }

        // Short-circuit if nothing changed
        if self.topmost_mouse_over_target.get() == prev_mouse_over_target {
            return;
        }

        let mut state_already_changed = false;

        // Notify Constellation about the topmost anchor mouse over target.
        let window = document.window();
        if let Some(target) = self.topmost_mouse_over_target.get() {
            if let Some(anchor) = target
                .upcast::<Node>()
                .inclusive_ancestors(ShadowIncluding::No)
                .filter_map(DomRoot::downcast::<HTMLAnchorElement>)
                .next()
            {
                let status = anchor
                    .upcast::<Element>()
                    .get_attribute(&ns!(), &local_name!("href"))
                    .and_then(|href| {
                        let value = href.value();
                        let url = document.url();
                        url.join(&value).map(|url| url.to_string()).ok()
                    });
                let event = EmbedderMsg::Status(document.webview_id(), status);
                window.send_to_embedder(event);

                state_already_changed = true;
            }
        }

        // We might have to reset the anchor state
        if !state_already_changed {
            if let Some(target) = prev_mouse_over_target {
                if target
                    .upcast::<Node>()
                    .inclusive_ancestors(ShadowIncluding::No)
                    .filter_map(DomRoot::downcast::<HTMLAnchorElement>)
                    .next()
                    .is_some()
                {
                    let event = EmbedderMsg::Status(window.webview_id(), None);
                    window.send_to_embedder(event);
                }
            }
        }
    }

    /// Process compositor events as part of a "update the rendering task".
    fn process_pending_input_events(&self, pipeline_id: PipelineId, can_gc: CanGc) {
        let Some(document) = self.documents.borrow().find_document(pipeline_id) else {
            warn!("Processing pending compositor events for closed pipeline {pipeline_id}.");
            return;
        };
        // Do not handle events if the BC has been, or is being, discarded
        if document.window().Closed() {
            warn!("Compositor event sent to a pipeline with a closed window {pipeline_id}.");
            return;
        }
        ScriptThread::set_user_interacting(true);

        let window = document.window();
        let _realm = enter_realm(document.window());
        for event in document.take_pending_input_events().into_iter() {
            document.update_active_keyboard_modifiers(event.active_keyboard_modifiers);

            match event.event {
                InputEvent::MouseButton(mouse_button_event) => {
                    document.handle_mouse_button_event(
                        mouse_button_event,
                        event.hit_test_result,
                        event.pressed_mouse_buttons,
                        can_gc,
                    );
                },
                InputEvent::MouseMove(_) => {
                    // The event itself is unecessary here, because the point in the viewport is in the hit test.
                    self.process_mouse_move_event(
                        &document,
                        event.hit_test_result,
                        event.pressed_mouse_buttons,
                        can_gc,
                    );
                },
                InputEvent::Touch(touch_event) => {
                    let touch_result =
                        document.handle_touch_event(touch_event, event.hit_test_result, can_gc);
                    if let (TouchEventResult::Processed(handled), true) =
                        (touch_result, touch_event.is_cancelable())
                    {
                        let sequence_id = touch_event.expect_sequence_id();
                        let result = if handled {
                            embedder_traits::TouchEventResult::DefaultAllowed(
                                sequence_id,
                                touch_event.event_type,
                            )
                        } else {
                            embedder_traits::TouchEventResult::DefaultPrevented(
                                sequence_id,
                                touch_event.event_type,
                            )
                        };
                        let message = ScriptToConstellationMessage::TouchEventProcessed(result);
                        self.senders
                            .pipeline_to_constellation_sender
                            .send((pipeline_id, message))
                            .unwrap();
                    }
                },
                InputEvent::Wheel(wheel_event) => {
                    document.handle_wheel_event(wheel_event, event.hit_test_result, can_gc);
                },
                InputEvent::Keyboard(keyboard_event) => {
                    document.dispatch_key_event(keyboard_event, can_gc);
                },
                InputEvent::Ime(ime_event) => {
                    document.dispatch_ime_event(ime_event, can_gc);
                },
                InputEvent::Gamepad(gamepad_event) => {
                    window.as_global_scope().handle_gamepad_event(gamepad_event);
                },
                InputEvent::EditingAction(editing_action_event) => {
                    document.handle_editing_action(editing_action_event, can_gc);
                },
            }
        }
        ScriptThread::set_user_interacting(false);
    }

    /// <https://html.spec.whatwg.org/multipage/#update-the-rendering>
    ///
    /// Attempt to update the rendering and then do a microtask checkpoint if rendering was actually
    /// updated.
    pub(crate) fn update_the_rendering(&self, requested_by_compositor: bool, can_gc: CanGc) {
        *self.last_render_opportunity_time.borrow_mut() = Some(Instant::now());

        if !self.can_continue_running_inner() {
            return;
        }

        // Run rafs for all pipeline, if a raf tick was received for any.
        // This ensures relative ordering of rafs between parent doc and iframes.
        let should_run_rafs = self
            .documents
            .borrow()
            .iter()
            .any(|(_, doc)| doc.is_fully_active() && doc.has_received_raf_tick());

        let any_animations_running = self.documents.borrow().iter().any(|(_, document)| {
            document.is_fully_active() && document.animations().running_animation_count() != 0
        });

        // TODO: The specification says to filter out non-renderable documents,
        // as well as those for which a rendering update would be unnecessary,
        // but this isn't happening here.

        // TODO(#31242): the filtering of docs is extended to not exclude the ones that
        // has pending initial observation targets
        // https://w3c.github.io/IntersectionObserver/#pending-initial-observation

        // If we aren't explicitly running rAFs, this update wasn't requested by the compositor,
        // and we are running animations, then wait until the compositor tells us it is time to
        // update the rendering via a TickAllAnimations message.
        if !requested_by_compositor && any_animations_running {
            return;
        }

        // > 2. Let docs be all fully active Document objects whose relevant agent's event loop
        // > is eventLoop, sorted arbitrarily except that the following conditions must be
        // > met:
        //
        // > Any Document B whose container document is A must be listed after A in the
        // > list.
        //
        // > If there are two documents A and B that both have the same non-null container
        // > document C, then the order of A and B in the list must match the
        // > shadow-including tree order of their respective navigable containers in C's
        // > node tree.
        //
        // > In the steps below that iterate over docs, each Document must be processed in
        // > the order it is found in the list.
        let documents_in_order = self.documents.borrow().documents_in_order();

        // TODO: The specification reads: "for doc in docs" at each step whereas this runs all
        // steps per doc in docs. Currently `<iframe>` resizing depends on a parent being able to
        // queue resize events on a child and have those run in the same call to this method, so
        // that needs to be sorted out to fix this.
        for pipeline_id in documents_in_order.iter() {
            let document = self
                .documents
                .borrow()
                .find_document(*pipeline_id)
                .expect("Got pipeline for Document not managed by this ScriptThread.");

            if !document.is_fully_active() {
                continue;
            }

            // TODO(#31581): The steps in the "Revealing the document" section need to be implemented
            // `process_pending_input_events` handles the focusing steps as well as other events
            // from the compositor.

            // TODO: Should this be broken and to match the specification more closely? For instance see
            // https://html.spec.whatwg.org/multipage/#flush-autofocus-candidates.
            self.process_pending_input_events(*pipeline_id, can_gc);

            // TODO(#31665): Implement the "run the scroll steps" from
            // https://drafts.csswg.org/cssom-view/#document-run-the-scroll-steps.

            // > 8. For each doc of docs, run the resize steps for doc. [CSSOMVIEW]
            if document.window().run_the_resize_steps(can_gc) {
                // Evaluate media queries and report changes.
                document
                    .window()
                    .evaluate_media_queries_and_report_changes(can_gc);

                // https://html.spec.whatwg.org/multipage/#img-environment-changes
                // As per the spec, this can be run at any time.
                document.react_to_environment_changes()
            }

            // > 11. For each doc of docs, update animations and send events for doc, passing
            // > in relative high resolution time given frameTimestamp and doc's relevant
            // > global object as the timestamp [WEBANIMATIONS]
            document.update_animations_and_send_events(can_gc);

            // TODO(#31866): Implement "run the fullscreen steps" from
            // https://fullscreen.spec.whatwg.org/multipage/#run-the-fullscreen-steps.

            // TODO(#31868): Implement the "context lost steps" from
            // https://html.spec.whatwg.org/multipage/#context-lost-steps.

            // > 14. For each doc of docs, run the animation frame callbacks for doc, passing
            // > in the relative high resolution time given frameTimestamp and doc's
            // > relevant global object as the timestamp.
            if should_run_rafs {
                document.run_the_animation_frame_callbacks(can_gc);
            }

            // Run the resize observer steps.
            let _realm = enter_realm(&*document);
            let mut depth = Default::default();
            while document.gather_active_resize_observations_at_depth(&depth, can_gc) {
                // Note: this will reflow the doc.
                depth = document.broadcast_active_resize_observations(can_gc);
            }

            if document.has_skipped_resize_observations() {
                document.deliver_resize_loop_error_notification(can_gc);
            }

            // TODO(#31870): Implement step 17: if the focused area of doc is not a focusable area,
            // then run the focusing steps for document's viewport.

            // TODO: Perform pending transition operations from
            // https://drafts.csswg.org/css-view-transitions/#perform-pending-transition-operations.

            // > 19. For each doc of docs, run the update intersection observations steps for doc,
            // > passing in the relative high resolution time given now and
            // > doc's relevant global object as the timestamp. [INTERSECTIONOBSERVER]
            // TODO(stevennovaryo): The time attribute should be relative to the time origin of the global object
            document.update_intersection_observer_steps(CrossProcessInstant::now(), can_gc);

            // TODO: Mark paint timing from https://w3c.github.io/paint-timing.

            #[cfg(feature = "webgpu")]
            document.update_rendering_of_webgpu_canvases();

            // > Step 22: For each doc of docs, update the rendering or user interface of
            // > doc and its node navigable to reflect the current state.
            let window = document.window();
            if document.is_fully_active() {
                window.reflow(ReflowGoal::UpdateTheRendering, can_gc);
            }

            // TODO: Process top layer removals according to
            // https://drafts.csswg.org/css-position-4/#process-top-layer-removals.
        }

        // Perform a microtask checkpoint as the specifications says that *update the rendering*
        // should be run in a task and a microtask checkpoint is always done when running tasks.
        self.perform_a_microtask_checkpoint(can_gc);

        // If there are pending reflows, they were probably caused by the execution of
        // the microtask checkpoint above and we should spin the event loop one more
        // time to resolve them.
        self.schedule_rendering_opportunity_if_necessary();
    }

    // If there are any pending reflows and we are not having rendering opportunities
    // driven by the compositor, then schedule the next rendering opportunity.
    //
    // TODO: This is a workaround until rendering opportunities can be triggered from a
    // timer in the script thread.
    fn schedule_rendering_opportunity_if_necessary(&self) {
        // If any Document has active animations of rAFs, then we should be receiving
        // regular rendering opportunities from the compositor (or fake animation frame
        // ticks). In this case, don't schedule an opportunity, just wait for the next
        // one.
        if self.documents.borrow().iter().any(|(_, document)| {
            document.is_fully_active() &&
                (document.animations().running_animation_count() != 0 ||
                    document.has_active_request_animation_frame_callbacks())
        }) {
            return;
        }

        let Some((_, document)) = self.documents.borrow().iter().find(|(_, document)| {
            document.is_fully_active() &&
                !document.window().layout_blocked() &&
                document.needs_reflow().is_some()
        }) else {
            return;
        };

        // Queues a task to update the rendering.
        // <https://html.spec.whatwg.org/multipage/#event-loop-processing-model:queue-a-global-task>
        //
        // Note: The specification says to queue a task using the navigable's active
        // window, but then updates the rendering for all documents.
        //
        // This task is empty because any new IPC messages in the ScriptThread trigger a
        // rendering update when animations are not running.
        let _realm = enter_realm(&*document);
        document
            .owner_global()
            .task_manager()
            .rendering_task_source()
            .queue_unconditionally(task!(update_the_rendering: move || { }));
    }

    /// Handle incoming messages from other tasks and the task queue.
    fn handle_msgs(&self, can_gc: CanGc) -> bool {
        // Proritize rendering tasks and others, and gather all other events as `sequential`.
        let mut sequential = vec![];

        // Notify the background-hang-monitor we are waiting for an event.
        self.background_hang_monitor.notify_wait();

        // Receive at least one message so we don't spinloop.
        debug!("Waiting for event.");
        let mut event = self
            .receivers
            .recv(&self.task_queue, &self.timer_scheduler.borrow());

        let mut compositor_requested_update_the_rendering = false;
        loop {
            debug!("Handling event: {event:?}");

            // Dispatch any completed timers, so that their tasks can be run below.
            self.timer_scheduler
                .borrow_mut()
                .dispatch_completed_timers();

            let _realm = event.pipeline_id().map(|id| {
                let global = self.documents.borrow().find_global(id);
                global.map(|global| enter_realm(&*global))
            });

            // https://html.spec.whatwg.org/multipage/#event-loop-processing-model step 7
            match event {
                // This has to be handled before the ResizeMsg below,
                // otherwise the page may not have been added to the
                // child list yet, causing the find() to fail.
                MixedMessage::FromConstellation(ScriptThreadMessage::AttachLayout(
                    new_layout_info,
                )) => {
                    let pipeline_id = new_layout_info.new_pipeline_id;
                    self.profile_event(
                        ScriptThreadEventCategory::AttachLayout,
                        Some(pipeline_id),
                        || {
                            // If this is an about:blank or about:srcdoc load, it must share the
                            // creator's origin. This must match the logic in the constellation
                            // when creating a new pipeline
                            let not_an_about_blank_and_about_srcdoc_load =
                                new_layout_info.load_data.url.as_str() != "about:blank" &&
                                    new_layout_info.load_data.url.as_str() != "about:srcdoc";
                            let origin = if not_an_about_blank_and_about_srcdoc_load {
                                MutableOrigin::new(new_layout_info.load_data.url.origin())
                            } else if let Some(parent) =
                                new_layout_info.parent_info.and_then(|pipeline_id| {
                                    self.documents.borrow().find_document(pipeline_id)
                                })
                            {
                                parent.origin().clone()
                            } else if let Some(creator) = new_layout_info
                                .load_data
                                .creator_pipeline_id
                                .and_then(|pipeline_id| {
                                    self.documents.borrow().find_document(pipeline_id)
                                })
                            {
                                creator.origin().clone()
                            } else {
                                MutableOrigin::new(ImmutableOrigin::new_opaque())
                            };

                            self.handle_new_layout(new_layout_info, origin);
                        },
                    )
                },
                MixedMessage::FromConstellation(ScriptThreadMessage::Resize(
                    id,
                    size,
                    size_type,
                )) => {
                    self.handle_resize_message(id, size, size_type);
                },
                MixedMessage::FromConstellation(ScriptThreadMessage::Viewport(id, rect)) => self
                    .profile_event(ScriptThreadEventCategory::SetViewport, Some(id), || {
                        self.handle_viewport(id, rect);
                    }),
                MixedMessage::FromConstellation(ScriptThreadMessage::TickAllAnimations(
                    pipeline_id,
                    tick_type,
                )) => {
                    if let Some(document) = self.documents.borrow().find_document(pipeline_id) {
                        document.note_pending_animation_tick(tick_type);
                        compositor_requested_update_the_rendering = true;
                    } else {
                        warn!(
                            "Trying to note pending animation tick for closed pipeline {}.",
                            pipeline_id
                        )
                    }
                },
                MixedMessage::FromConstellation(ScriptThreadMessage::SendInputEvent(id, event)) => {
                    self.handle_input_event(id, event)
                },
                MixedMessage::FromScript(MainThreadScriptMsg::Common(CommonScriptMsg::Task(
                    _,
                    _,
                    _,
                    TaskSourceName::Rendering,
                ))) => {
                    // Instead of interleaving any number of update the rendering tasks with other
                    // message handling, we run those steps only once at the end of each call of
                    // this function.
                },
                MixedMessage::FromScript(MainThreadScriptMsg::Inactive) => {
                    // An event came-in from a document that is not fully-active, it has been stored by the task-queue.
                    // Continue without adding it to "sequential".
                },
                MixedMessage::FromConstellation(ScriptThreadMessage::ExitFullScreen(id)) => self
                    .profile_event(ScriptThreadEventCategory::ExitFullscreen, Some(id), || {
                        self.handle_exit_fullscreen(id, can_gc);
                    }),
                _ => {
                    sequential.push(event);
                },
            }

            // If any of our input sources has an event pending, we'll perform another iteration
            // and check for more resize events. If there are no events pending, we'll move
            // on and execute the sequential non-resize events we've seen.
            match self.receivers.try_recv(&self.task_queue) {
                Some(new_event) => event = new_event,
                None => break,
            }
        }

        // Process the gathered events.
        debug!("Processing events.");
        for msg in sequential {
            debug!("Processing event {:?}.", msg);
            let category = self.categorize_msg(&msg);
            let pipeline_id = msg.pipeline_id();
            let _realm = pipeline_id.and_then(|id| {
                let global = self.documents.borrow().find_global(id);
                global.map(|global| enter_realm(&*global))
            });

            if self.closing.load(Ordering::SeqCst) {
                // If we've received the closed signal from the BHM, only handle exit messages.
                match msg {
                    MixedMessage::FromConstellation(ScriptThreadMessage::ExitScriptThread) => {
                        self.handle_exit_script_thread_msg(can_gc);
                        return false;
                    },
                    MixedMessage::FromConstellation(ScriptThreadMessage::ExitPipeline(
                        pipeline_id,
                        discard_browsing_context,
                    )) => {
                        self.handle_exit_pipeline_msg(
                            pipeline_id,
                            discard_browsing_context,
                            can_gc,
                        );
                    },
                    _ => {},
                }
                continue;
            }

            let exiting = self.profile_event(category, pipeline_id, move || {
                match msg {
                    MixedMessage::FromConstellation(ScriptThreadMessage::ExitScriptThread) => {
                        self.handle_exit_script_thread_msg(can_gc);
                        return true;
                    },
                    MixedMessage::FromConstellation(inner_msg) => {
                        self.handle_msg_from_constellation(inner_msg, can_gc)
                    },
                    MixedMessage::FromScript(inner_msg) => self.handle_msg_from_script(inner_msg),
                    MixedMessage::FromDevtools(inner_msg) => {
                        self.handle_msg_from_devtools(inner_msg, can_gc)
                    },
                    MixedMessage::FromImageCache(inner_msg) => {
                        self.handle_msg_from_image_cache(inner_msg)
                    },
                    #[cfg(feature = "webgpu")]
                    MixedMessage::FromWebGPUServer(inner_msg) => {
                        self.handle_msg_from_webgpu_server(inner_msg, can_gc)
                    },
                    MixedMessage::TimerFired => {},
                }

                false
            });

            // If an `ExitScriptThread` message was handled above, bail out now.
            if exiting {
                return false;
            }

            // https://html.spec.whatwg.org/multipage/#event-loop-processing-model step 6
            // TODO(#32003): A microtask checkpoint is only supposed to be performed after running a task.
            self.perform_a_microtask_checkpoint(can_gc);
        }

        for (_, doc) in self.documents.borrow().iter() {
            let window = doc.window();
            window
                .upcast::<GlobalScope>()
                .perform_a_dom_garbage_collection_checkpoint();
        }

        {
            // https://html.spec.whatwg.org/multipage/#the-end step 6
            let mut docs = self.docs_with_no_blocking_loads.borrow_mut();
            for document in docs.iter() {
                let _realm = enter_realm(&**document);
                document.maybe_queue_document_completion();
            }
            docs.clear();
        }

        // Update the rendering whenever we receive an IPC message. This may not actually do anything if
        // we are running animations and the compositor hasn't requested a new frame yet via a TickAllAnimatons
        // message.
        self.update_the_rendering(compositor_requested_update_the_rendering, can_gc);

        true
    }

    fn categorize_msg(&self, msg: &MixedMessage) -> ScriptThreadEventCategory {
        match *msg {
            MixedMessage::FromConstellation(ref inner_msg) => match *inner_msg {
                ScriptThreadMessage::SendInputEvent(_, _) => ScriptThreadEventCategory::InputEvent,
                _ => ScriptThreadEventCategory::ConstellationMsg,
            },
            // TODO https://github.com/servo/servo/issues/18998
            MixedMessage::FromDevtools(_) => ScriptThreadEventCategory::DevtoolsMsg,
            MixedMessage::FromImageCache(_) => ScriptThreadEventCategory::ImageCacheMsg,
            MixedMessage::FromScript(ref inner_msg) => match *inner_msg {
                MainThreadScriptMsg::Common(CommonScriptMsg::Task(category, ..)) => category,
                MainThreadScriptMsg::RegisterPaintWorklet { .. } => {
                    ScriptThreadEventCategory::WorkletEvent
                },
                _ => ScriptThreadEventCategory::ScriptEvent,
            },
            #[cfg(feature = "webgpu")]
            MixedMessage::FromWebGPUServer(_) => ScriptThreadEventCategory::WebGPUMsg,
            MixedMessage::TimerFired => ScriptThreadEventCategory::TimerEvent,
        }
    }

    fn profile_event<F, R>(
        &self,
        category: ScriptThreadEventCategory,
        pipeline_id: Option<PipelineId>,
        f: F,
    ) -> R
    where
        F: FnOnce() -> R,
    {
        self.background_hang_monitor
            .notify_activity(HangAnnotation::Script(category.into()));
        let start = Instant::now();
        let value = if self.profile_script_events {
            let profiler_chan = self.senders.time_profiler_sender.clone();
            match category {
                ScriptThreadEventCategory::AttachLayout => {
                    time_profile!(ProfilerCategory::ScriptAttachLayout, None, profiler_chan, f)
                },
                ScriptThreadEventCategory::ConstellationMsg => time_profile!(
                    ProfilerCategory::ScriptConstellationMsg,
                    None,
                    profiler_chan,
                    f
                ),
                ScriptThreadEventCategory::DevtoolsMsg => {
                    time_profile!(ProfilerCategory::ScriptDevtoolsMsg, None, profiler_chan, f)
                },
                ScriptThreadEventCategory::DocumentEvent => time_profile!(
                    ProfilerCategory::ScriptDocumentEvent,
                    None,
                    profiler_chan,
                    f
                ),
                ScriptThreadEventCategory::InputEvent => {
                    time_profile!(ProfilerCategory::ScriptInputEvent, None, profiler_chan, f)
                },
                ScriptThreadEventCategory::FileRead => {
                    time_profile!(ProfilerCategory::ScriptFileRead, None, profiler_chan, f)
                },
                ScriptThreadEventCategory::FontLoading => {
                    time_profile!(ProfilerCategory::ScriptFontLoading, None, profiler_chan, f)
                },
                ScriptThreadEventCategory::FormPlannedNavigation => time_profile!(
                    ProfilerCategory::ScriptPlannedNavigation,
                    None,
                    profiler_chan,
                    f
                ),
                ScriptThreadEventCategory::HistoryEvent => {
                    time_profile!(ProfilerCategory::ScriptHistoryEvent, None, profiler_chan, f)
                },
                ScriptThreadEventCategory::ImageCacheMsg => time_profile!(
                    ProfilerCategory::ScriptImageCacheMsg,
                    None,
                    profiler_chan,
                    f
                ),
                ScriptThreadEventCategory::NetworkEvent => {
                    time_profile!(ProfilerCategory::ScriptNetworkEvent, None, profiler_chan, f)
                },
                ScriptThreadEventCategory::PortMessage => {
                    time_profile!(ProfilerCategory::ScriptPortMessage, None, profiler_chan, f)
                },
                ScriptThreadEventCategory::Resize => {
                    time_profile!(ProfilerCategory::ScriptResize, None, profiler_chan, f)
                },
                ScriptThreadEventCategory::ScriptEvent => {
                    time_profile!(ProfilerCategory::ScriptEvent, None, profiler_chan, f)
                },
                ScriptThreadEventCategory::SetScrollState => time_profile!(
                    ProfilerCategory::ScriptSetScrollState,
                    None,
                    profiler_chan,
                    f
                ),
                ScriptThreadEventCategory::UpdateReplacedElement => time_profile!(
                    ProfilerCategory::ScriptUpdateReplacedElement,
                    None,
                    profiler_chan,
                    f
                ),
                ScriptThreadEventCategory::StylesheetLoad => time_profile!(
                    ProfilerCategory::ScriptStylesheetLoad,
                    None,
                    profiler_chan,
                    f
                ),
                ScriptThreadEventCategory::SetViewport => {
                    time_profile!(ProfilerCategory::ScriptSetViewport, None, profiler_chan, f)
                },
                ScriptThreadEventCategory::TimerEvent => {
                    time_profile!(ProfilerCategory::ScriptTimerEvent, None, profiler_chan, f)
                },
                ScriptThreadEventCategory::WebSocketEvent => time_profile!(
                    ProfilerCategory::ScriptWebSocketEvent,
                    None,
                    profiler_chan,
                    f
                ),
                ScriptThreadEventCategory::WorkerEvent => {
                    time_profile!(ProfilerCategory::ScriptWorkerEvent, None, profiler_chan, f)
                },
                ScriptThreadEventCategory::WorkletEvent => {
                    time_profile!(ProfilerCategory::ScriptWorkletEvent, None, profiler_chan, f)
                },
                ScriptThreadEventCategory::ServiceWorkerEvent => time_profile!(
                    ProfilerCategory::ScriptServiceWorkerEvent,
                    None,
                    profiler_chan,
                    f
                ),
                ScriptThreadEventCategory::EnterFullscreen => time_profile!(
                    ProfilerCategory::ScriptEnterFullscreen,
                    None,
                    profiler_chan,
                    f
                ),
                ScriptThreadEventCategory::ExitFullscreen => time_profile!(
                    ProfilerCategory::ScriptExitFullscreen,
                    None,
                    profiler_chan,
                    f
                ),
                ScriptThreadEventCategory::PerformanceTimelineTask => time_profile!(
                    ProfilerCategory::ScriptPerformanceEvent,
                    None,
                    profiler_chan,
                    f
                ),
                ScriptThreadEventCategory::Rendering => {
                    time_profile!(ProfilerCategory::ScriptRendering, None, profiler_chan, f)
                },
                #[cfg(feature = "webgpu")]
                ScriptThreadEventCategory::WebGPUMsg => {
                    time_profile!(ProfilerCategory::ScriptWebGPUMsg, None, profiler_chan, f)
                },
            }
        } else {
            f()
        };
        let task_duration = start.elapsed();
        for (doc_id, doc) in self.documents.borrow().iter() {
            if let Some(pipeline_id) = pipeline_id {
                if pipeline_id == doc_id && task_duration.as_nanos() > MAX_TASK_NS {
                    if self.print_pwm {
                        println!(
                            "Task took longer than max allowed ({:?}) {:?}",
                            category,
                            task_duration.as_nanos()
                        );
                    }
                    doc.start_tti();
                }
            }
            doc.record_tti_if_necessary();
        }
        value
    }

    fn handle_msg_from_constellation(&self, msg: ScriptThreadMessage, can_gc: CanGc) {
        match msg {
            ScriptThreadMessage::StopDelayingLoadEventsMode(pipeline_id) => {
                self.handle_stop_delaying_load_events_mode(pipeline_id)
            },
            ScriptThreadMessage::NavigateIframe(
                parent_pipeline_id,
                browsing_context_id,
                load_data,
                history_handling,
            ) => self.handle_navigate_iframe(
                parent_pipeline_id,
                browsing_context_id,
                load_data,
                history_handling,
                can_gc,
            ),
            ScriptThreadMessage::UnloadDocument(pipeline_id) => {
                self.handle_unload_document(pipeline_id, can_gc)
            },
            ScriptThreadMessage::ResizeInactive(id, new_size) => {
                self.handle_resize_inactive_msg(id, new_size)
            },
            ScriptThreadMessage::ThemeChange(_, theme) => {
                self.handle_theme_change_msg(theme);
            },
            ScriptThreadMessage::GetTitle(pipeline_id) => self.handle_get_title_msg(pipeline_id),
            ScriptThreadMessage::SetDocumentActivity(pipeline_id, activity) => {
                self.handle_set_document_activity_msg(pipeline_id, activity, can_gc)
            },
            ScriptThreadMessage::SetThrottled(pipeline_id, throttled) => {
                self.handle_set_throttled_msg(pipeline_id, throttled)
            },
            ScriptThreadMessage::SetThrottledInContainingIframe(
                parent_pipeline_id,
                browsing_context_id,
                throttled,
            ) => self.handle_set_throttled_in_containing_iframe_msg(
                parent_pipeline_id,
                browsing_context_id,
                throttled,
            ),
            ScriptThreadMessage::PostMessage {
                target: target_pipeline_id,
                source: source_pipeline_id,
                source_browsing_context,
                target_origin: origin,
                source_origin,
                data,
            } => self.handle_post_message_msg(
                target_pipeline_id,
                source_pipeline_id,
                source_browsing_context,
                origin,
                source_origin,
                data,
            ),
            ScriptThreadMessage::UpdatePipelineId(
                parent_pipeline_id,
                browsing_context_id,
                webview_id,
                new_pipeline_id,
                reason,
            ) => self.handle_update_pipeline_id(
                parent_pipeline_id,
                browsing_context_id,
                webview_id,
                new_pipeline_id,
                reason,
                can_gc,
            ),
            ScriptThreadMessage::UpdateHistoryState(pipeline_id, history_state_id, url) => {
                self.handle_update_history_state_msg(pipeline_id, history_state_id, url, can_gc)
            },
            ScriptThreadMessage::RemoveHistoryStates(pipeline_id, history_states) => {
                self.handle_remove_history_states(pipeline_id, history_states)
            },
            ScriptThreadMessage::FocusIFrame(parent_pipeline_id, frame_id) => {
                self.handle_focus_iframe_msg(parent_pipeline_id, frame_id, can_gc)
            },
            ScriptThreadMessage::WebDriverScriptCommand(pipeline_id, msg) => {
                self.handle_webdriver_msg(pipeline_id, msg, can_gc)
            },
            ScriptThreadMessage::WebFontLoaded(pipeline_id, success) => {
                self.handle_web_font_loaded(pipeline_id, success)
            },
            ScriptThreadMessage::DispatchIFrameLoadEvent {
                target: browsing_context_id,
                parent: parent_id,
                child: child_id,
            } => self.handle_iframe_load_event(parent_id, browsing_context_id, child_id, can_gc),
            ScriptThreadMessage::DispatchStorageEvent(
                pipeline_id,
                storage,
                url,
                key,
                old_value,
                new_value,
            ) => self.handle_storage_event(pipeline_id, storage, url, key, old_value, new_value),
            ScriptThreadMessage::ReportCSSError(pipeline_id, filename, line, column, msg) => {
                self.handle_css_error_reporting(pipeline_id, filename, line, column, msg)
            },
            ScriptThreadMessage::Reload(pipeline_id) => self.handle_reload(pipeline_id, can_gc),
            ScriptThreadMessage::ExitPipeline(pipeline_id, discard_browsing_context) => {
                self.handle_exit_pipeline_msg(pipeline_id, discard_browsing_context, can_gc)
            },
            ScriptThreadMessage::PaintMetric(
                pipeline_id,
                metric_type,
                metric_value,
                first_reflow,
            ) => self.handle_paint_metric(
                pipeline_id,
                metric_type,
                metric_value,
                first_reflow,
                can_gc,
            ),
            ScriptThreadMessage::MediaSessionAction(pipeline_id, action) => {
                self.handle_media_session_action(pipeline_id, action, can_gc)
            },
            #[cfg(feature = "webgpu")]
            ScriptThreadMessage::SetWebGPUPort(port) => {
                *self.receivers.webgpu_receiver.borrow_mut() =
                    ROUTER.route_ipc_receiver_to_new_crossbeam_receiver(port);
            },
            msg @ ScriptThreadMessage::AttachLayout(..) |
            msg @ ScriptThreadMessage::Viewport(..) |
            msg @ ScriptThreadMessage::Resize(..) |
            msg @ ScriptThreadMessage::ExitFullScreen(..) |
            msg @ ScriptThreadMessage::SendInputEvent(..) |
            msg @ ScriptThreadMessage::TickAllAnimations(..) |
            msg @ ScriptThreadMessage::ExitScriptThread => {
                panic!("should have handled {:?} already", msg)
            },
            ScriptThreadMessage::SetScrollStates(pipeline_id, scroll_states) => {
                self.handle_set_scroll_states(pipeline_id, scroll_states)
            },
        }
    }

    fn handle_set_scroll_states(&self, pipeline_id: PipelineId, scroll_states: Vec<ScrollState>) {
        let Some(window) = self.documents.borrow().find_window(pipeline_id) else {
            warn!("Received scroll states for closed pipeline {pipeline_id}");
            return;
        };

        self.profile_event(
            ScriptThreadEventCategory::SetScrollState,
            Some(pipeline_id),
            || {
                window.layout_mut().set_scroll_offsets(&scroll_states);

                let mut scroll_offsets = HashMap::new();
                for scroll_state in scroll_states.into_iter() {
                    let scroll_offset = scroll_state.scroll_offset;
                    if scroll_state.scroll_id.is_root() {
                        window.update_viewport_for_scroll(-scroll_offset.x, -scroll_offset.y);
                    } else if let Some(node_id) =
                        node_id_from_scroll_id(scroll_state.scroll_id.0 as usize)
                    {
                        scroll_offsets.insert(OpaqueNode(node_id), -scroll_offset);
                    }
                }
                window.set_scroll_offsets(scroll_offsets)
            },
        )
    }

    #[cfg(feature = "webgpu")]
    fn handle_msg_from_webgpu_server(&self, msg: WebGPUMsg, can_gc: CanGc) {
        match msg {
            WebGPUMsg::FreeAdapter(id) => self.gpu_id_hub.free_adapter_id(id),
            WebGPUMsg::FreeDevice {
                device_id,
                pipeline_id,
            } => {
                self.gpu_id_hub.free_device_id(device_id);
                if let Some(global) = self.documents.borrow().find_global(pipeline_id) {
                    global.remove_gpu_device(WebGPUDevice(device_id));
                } // page can already be destroyed
            },
            WebGPUMsg::FreeBuffer(id) => self.gpu_id_hub.free_buffer_id(id),
            WebGPUMsg::FreePipelineLayout(id) => self.gpu_id_hub.free_pipeline_layout_id(id),
            WebGPUMsg::FreeComputePipeline(id) => self.gpu_id_hub.free_compute_pipeline_id(id),
            WebGPUMsg::FreeBindGroup(id) => self.gpu_id_hub.free_bind_group_id(id),
            WebGPUMsg::FreeBindGroupLayout(id) => self.gpu_id_hub.free_bind_group_layout_id(id),
            WebGPUMsg::FreeCommandBuffer(id) => self
                .gpu_id_hub
                .free_command_buffer_id(id.into_command_encoder_id()),
            WebGPUMsg::FreeSampler(id) => self.gpu_id_hub.free_sampler_id(id),
            WebGPUMsg::FreeShaderModule(id) => self.gpu_id_hub.free_shader_module_id(id),
            WebGPUMsg::FreeRenderBundle(id) => self.gpu_id_hub.free_render_bundle_id(id),
            WebGPUMsg::FreeRenderPipeline(id) => self.gpu_id_hub.free_render_pipeline_id(id),
            WebGPUMsg::FreeTexture(id) => self.gpu_id_hub.free_texture_id(id),
            WebGPUMsg::FreeTextureView(id) => self.gpu_id_hub.free_texture_view_id(id),
            WebGPUMsg::FreeComputePass(id) => self.gpu_id_hub.free_compute_pass_id(id),
            WebGPUMsg::FreeRenderPass(id) => self.gpu_id_hub.free_render_pass_id(id),
            WebGPUMsg::Exit => {
                *self.receivers.webgpu_receiver.borrow_mut() = crossbeam_channel::never()
            },
            WebGPUMsg::DeviceLost {
                pipeline_id,
                device,
                reason,
                msg,
            } => {
                let global = self.documents.borrow().find_global(pipeline_id).unwrap();
                global.gpu_device_lost(device, reason, msg, can_gc);
            },
            WebGPUMsg::UncapturedError {
                device,
                pipeline_id,
                error,
            } => {
                let global = self.documents.borrow().find_global(pipeline_id).unwrap();
                let _ac = enter_realm(&*global);
                global.handle_uncaptured_gpu_error(device, error, can_gc);
            },
            _ => {},
        }
    }

    fn handle_msg_from_script(&self, msg: MainThreadScriptMsg) {
        match msg {
            MainThreadScriptMsg::Common(CommonScriptMsg::Task(_, task, pipeline_id, _)) => {
                let _realm = pipeline_id.and_then(|id| {
                    let global = self.documents.borrow().find_global(id);
                    global.map(|global| enter_realm(&*global))
                });
                task.run_box()
            },
            MainThreadScriptMsg::Common(CommonScriptMsg::CollectReports(chan)) => {
                self.collect_reports(chan)
            },
            MainThreadScriptMsg::NavigationResponse {
                pipeline_id,
                message,
            } => {
                self.handle_navigation_response(pipeline_id, *message);
            },
            MainThreadScriptMsg::WorkletLoaded(pipeline_id) => {
                self.handle_worklet_loaded(pipeline_id)
            },
            MainThreadScriptMsg::RegisterPaintWorklet {
                pipeline_id,
                name,
                properties,
                painter,
            } => self.handle_register_paint_worklet(pipeline_id, name, properties, painter),
            MainThreadScriptMsg::Inactive => {},
            MainThreadScriptMsg::WakeUp => {},
        }
    }

    fn handle_msg_from_devtools(&self, msg: DevtoolScriptControlMsg, can_gc: CanGc) {
        let documents = self.documents.borrow();
        match msg {
            DevtoolScriptControlMsg::EvaluateJS(id, s, reply) => match documents.find_window(id) {
                Some(window) => {
                    let global = window.as_global_scope();
                    let _aes = AutoEntryScript::new(global);
                    devtools::handle_evaluate_js(global, s, reply, can_gc)
                },
                None => warn!("Message sent to closed pipeline {}.", id),
            },
            DevtoolScriptControlMsg::GetRootNode(id, reply) => {
                devtools::handle_get_root_node(&documents, id, reply, can_gc)
            },
            DevtoolScriptControlMsg::GetDocumentElement(id, reply) => {
                devtools::handle_get_document_element(&documents, id, reply, can_gc)
            },
            DevtoolScriptControlMsg::GetChildren(id, node_id, reply) => {
                devtools::handle_get_children(&documents, id, node_id, reply, can_gc)
            },
            DevtoolScriptControlMsg::GetAttributeStyle(id, node_id, reply) => {
                devtools::handle_get_attribute_style(&documents, id, node_id, reply, can_gc)
            },
            DevtoolScriptControlMsg::GetStylesheetStyle(
                id,
                node_id,
                selector,
                stylesheet,
                reply,
            ) => devtools::handle_get_stylesheet_style(
                &documents, id, node_id, selector, stylesheet, reply, can_gc,
            ),
            DevtoolScriptControlMsg::GetSelectors(id, node_id, reply) => {
                devtools::handle_get_selectors(&documents, id, node_id, reply, can_gc)
            },
            DevtoolScriptControlMsg::GetComputedStyle(id, node_id, reply) => {
                devtools::handle_get_computed_style(&documents, id, node_id, reply, can_gc)
            },
            DevtoolScriptControlMsg::GetLayout(id, node_id, reply) => {
                devtools::handle_get_layout(&documents, id, node_id, reply, can_gc)
            },
            DevtoolScriptControlMsg::ModifyAttribute(id, node_id, modifications) => {
                devtools::handle_modify_attribute(&documents, id, node_id, modifications, can_gc)
            },
            DevtoolScriptControlMsg::ModifyRule(id, node_id, modifications) => {
                devtools::handle_modify_rule(&documents, id, node_id, modifications, can_gc)
            },
            DevtoolScriptControlMsg::WantsLiveNotifications(id, to_send) => match documents
                .find_window(id)
            {
                Some(window) => devtools::handle_wants_live_notifications(window.upcast(), to_send),
                None => warn!("Message sent to closed pipeline {}.", id),
            },
            DevtoolScriptControlMsg::SetTimelineMarkers(id, marker_types, reply) => {
                devtools::handle_set_timeline_markers(&documents, id, marker_types, reply)
            },
            DevtoolScriptControlMsg::DropTimelineMarkers(id, marker_types) => {
                devtools::handle_drop_timeline_markers(&documents, id, marker_types)
            },
            DevtoolScriptControlMsg::RequestAnimationFrame(id, name) => {
                devtools::handle_request_animation_frame(&documents, id, name)
            },
            DevtoolScriptControlMsg::Reload(id) => devtools::handle_reload(&documents, id, can_gc),
            DevtoolScriptControlMsg::GetCssDatabase(reply) => {
                devtools::handle_get_css_database(reply)
            },
            DevtoolScriptControlMsg::SimulateColorScheme(id, theme) => {
                match documents.find_window(id) {
                    Some(window) => {
                        window.handle_theme_change(theme);
                    },
                    None => warn!("Message sent to closed pipeline {}.", id),
                }
            },
        }
    }

    fn handle_msg_from_image_cache(&self, response: PendingImageResponse) {
        let window = self.documents.borrow().find_window(response.pipeline_id);
        if let Some(ref window) = window {
            window.pending_image_notification(response);
        }
    }

    fn handle_webdriver_msg(
        &self,
        pipeline_id: PipelineId,
        msg: WebDriverScriptCommand,
        can_gc: CanGc,
    ) {
        // https://github.com/servo/servo/issues/23535
        // These two messages need different treatment since the JS script might mutate
        // `self.documents`, which would conflict with the immutable borrow of it that
        // occurs for the rest of the messages
        match msg {
            WebDriverScriptCommand::ExecuteScript(script, reply) => {
                let window = self.documents.borrow().find_window(pipeline_id);
                return webdriver_handlers::handle_execute_script(window, script, reply, can_gc);
            },
            WebDriverScriptCommand::ExecuteAsyncScript(script, reply) => {
                let window = self.documents.borrow().find_window(pipeline_id);
                return webdriver_handlers::handle_execute_async_script(
                    window, script, reply, can_gc,
                );
            },
            _ => (),
        }

        let documents = self.documents.borrow();
        match msg {
            WebDriverScriptCommand::AddCookie(params, reply) => {
                webdriver_handlers::handle_add_cookie(&documents, pipeline_id, params, reply)
            },
            WebDriverScriptCommand::DeleteCookies(reply) => {
                webdriver_handlers::handle_delete_cookies(&documents, pipeline_id, reply)
            },
            WebDriverScriptCommand::DeleteCookie(name, reply) => {
                webdriver_handlers::handle_delete_cookie(&documents, pipeline_id, name, reply)
            },
            WebDriverScriptCommand::FindElementCSS(selector, reply) => {
                webdriver_handlers::handle_find_element_css(
                    &documents,
                    pipeline_id,
                    selector,
                    reply,
                )
            },
            WebDriverScriptCommand::FindElementLinkText(selector, partial, reply) => {
                webdriver_handlers::handle_find_element_link_text(
                    &documents,
                    pipeline_id,
                    selector,
                    partial,
                    reply,
                )
            },
            WebDriverScriptCommand::FindElementTagName(selector, reply) => {
                webdriver_handlers::handle_find_element_tag_name(
                    &documents,
                    pipeline_id,
                    selector,
                    reply,
                    can_gc,
                )
            },
            WebDriverScriptCommand::FindElementsCSS(selector, reply) => {
                webdriver_handlers::handle_find_elements_css(
                    &documents,
                    pipeline_id,
                    selector,
                    reply,
                )
            },
            WebDriverScriptCommand::FindElementsLinkText(selector, partial, reply) => {
                webdriver_handlers::handle_find_elements_link_text(
                    &documents,
                    pipeline_id,
                    selector,
                    partial,
                    reply,
                )
            },
            WebDriverScriptCommand::FindElementsTagName(selector, reply) => {
                webdriver_handlers::handle_find_elements_tag_name(
                    &documents,
                    pipeline_id,
                    selector,
                    reply,
                    can_gc,
                )
            },
            WebDriverScriptCommand::FindElementElementCSS(selector, element_id, reply) => {
                webdriver_handlers::handle_find_element_element_css(
                    &documents,
                    pipeline_id,
                    element_id,
                    selector,
                    reply,
                )
            },
            WebDriverScriptCommand::FindElementElementLinkText(
                selector,
                element_id,
                partial,
                reply,
            ) => webdriver_handlers::handle_find_element_element_link_text(
                &documents,
                pipeline_id,
                element_id,
                selector,
                partial,
                reply,
            ),
            WebDriverScriptCommand::FindElementElementTagName(selector, element_id, reply) => {
                webdriver_handlers::handle_find_element_element_tag_name(
                    &documents,
                    pipeline_id,
                    element_id,
                    selector,
                    reply,
                    can_gc,
                )
            },
            WebDriverScriptCommand::FindElementElementsCSS(selector, element_id, reply) => {
                webdriver_handlers::handle_find_element_elements_css(
                    &documents,
                    pipeline_id,
                    element_id,
                    selector,
                    reply,
                )
            },
            WebDriverScriptCommand::FindElementElementsLinkText(
                selector,
                element_id,
                partial,
                reply,
            ) => webdriver_handlers::handle_find_element_elements_link_text(
                &documents,
                pipeline_id,
                element_id,
                selector,
                partial,
                reply,
            ),
            WebDriverScriptCommand::FindElementElementsTagName(selector, element_id, reply) => {
                webdriver_handlers::handle_find_element_elements_tag_name(
                    &documents,
                    pipeline_id,
                    element_id,
                    selector,
                    reply,
                    can_gc,
                )
            },
            WebDriverScriptCommand::FocusElement(element_id, reply) => {
                webdriver_handlers::handle_focus_element(
                    &documents,
                    pipeline_id,
                    element_id,
                    reply,
                    can_gc,
                )
            },
            WebDriverScriptCommand::ElementClick(element_id, reply) => {
                webdriver_handlers::handle_element_click(
                    &documents,
                    pipeline_id,
                    element_id,
                    reply,
                    can_gc,
                )
            },
            WebDriverScriptCommand::GetActiveElement(reply) => {
                webdriver_handlers::handle_get_active_element(&documents, pipeline_id, reply)
            },
            WebDriverScriptCommand::GetComputedRole(node_id, reply) => {
                webdriver_handlers::handle_get_computed_role(
                    &documents,
                    pipeline_id,
                    node_id,
                    reply,
                )
            },
            WebDriverScriptCommand::GetPageSource(reply) => {
                webdriver_handlers::handle_get_page_source(&documents, pipeline_id, reply, can_gc)
            },
            WebDriverScriptCommand::GetCookies(reply) => {
                webdriver_handlers::handle_get_cookies(&documents, pipeline_id, reply)
            },
            WebDriverScriptCommand::GetCookie(name, reply) => {
                webdriver_handlers::handle_get_cookie(&documents, pipeline_id, name, reply)
            },
            WebDriverScriptCommand::GetElementTagName(node_id, reply) => {
                webdriver_handlers::handle_get_name(&documents, pipeline_id, node_id, reply)
            },
            WebDriverScriptCommand::GetElementAttribute(node_id, name, reply) => {
                webdriver_handlers::handle_get_attribute(
                    &documents,
                    pipeline_id,
                    node_id,
                    name,
                    reply,
                )
            },
            WebDriverScriptCommand::GetElementProperty(node_id, name, reply) => {
                webdriver_handlers::handle_get_property(
                    &documents,
                    pipeline_id,
                    node_id,
                    name,
                    reply,
                )
            },
            WebDriverScriptCommand::GetElementCSS(node_id, name, reply) => {
                webdriver_handlers::handle_get_css(
                    &documents,
                    pipeline_id,
                    node_id,
                    name,
                    reply,
                    can_gc,
                )
            },
            WebDriverScriptCommand::GetElementRect(node_id, reply) => {
                webdriver_handlers::handle_get_rect(&documents, pipeline_id, node_id, reply, can_gc)
            },
            WebDriverScriptCommand::GetBoundingClientRect(node_id, reply) => {
                webdriver_handlers::handle_get_bounding_client_rect(
                    &documents,
                    pipeline_id,
                    node_id,
                    reply,
                    can_gc,
                )
            },
            WebDriverScriptCommand::GetElementText(node_id, reply) => {
                webdriver_handlers::handle_get_text(&documents, pipeline_id, node_id, reply)
            },
            WebDriverScriptCommand::GetElementInViewCenterPoint(node_id, reply) => {
                webdriver_handlers::handle_get_element_in_view_center_point(
                    &documents,
                    pipeline_id,
                    node_id,
                    reply,
                    can_gc,
                )
            },
            WebDriverScriptCommand::GetBrowsingContextId(webdriver_frame_id, reply) => {
                webdriver_handlers::handle_get_browsing_context_id(
                    &documents,
                    pipeline_id,
                    webdriver_frame_id,
                    reply,
                )
            },
            WebDriverScriptCommand::GetUrl(reply) => {
                webdriver_handlers::handle_get_url(&documents, pipeline_id, reply, can_gc)
            },
            WebDriverScriptCommand::IsEnabled(element_id, reply) => {
                webdriver_handlers::handle_is_enabled(&documents, pipeline_id, element_id, reply)
            },
            WebDriverScriptCommand::IsSelected(element_id, reply) => {
                webdriver_handlers::handle_is_selected(&documents, pipeline_id, element_id, reply)
            },
            WebDriverScriptCommand::GetTitle(reply) => {
                webdriver_handlers::handle_get_title(&documents, pipeline_id, reply)
            },
            _ => (),
        }
    }

    /// Batch window resize operations into a single "update the rendering" task,
    /// or, if a load is in progress, set the window size directly.
    pub(crate) fn handle_resize_message(
        &self,
        id: PipelineId,
        viewport_details: ViewportDetails,
        size_type: WindowSizeType,
    ) {
        self.profile_event(ScriptThreadEventCategory::Resize, Some(id), || {
            let window = self.documents.borrow().find_window(id);
            if let Some(ref window) = window {
                window.add_resize_event(viewport_details, size_type);
                return;
            }
            let mut loads = self.incomplete_loads.borrow_mut();
            if let Some(ref mut load) = loads.iter_mut().find(|load| load.pipeline_id == id) {
                load.viewport_details = viewport_details;
            }
        })
    }

    /// Handle changes to the theme, triggering reflow if the theme actually changed.
    fn handle_theme_change_msg(&self, theme: Theme) {
        for (_, document) in self.documents.borrow().iter() {
            document.window().handle_theme_change(theme);
        }
    }

    // exit_fullscreen creates a new JS promise object, so we need to have entered a realm
    fn handle_exit_fullscreen(&self, id: PipelineId, can_gc: CanGc) {
        let document = self.documents.borrow().find_document(id);
        if let Some(document) = document {
            let _ac = enter_realm(&*document);
            document.exit_fullscreen(can_gc);
        }
    }

    fn handle_viewport(&self, id: PipelineId, rect: Rect<f32>) {
        let document = self.documents.borrow().find_document(id);
        if let Some(document) = document {
            document.window().set_page_clip_rect_with_new_viewport(rect);
            return;
        }
        let loads = self.incomplete_loads.borrow();
        if loads.iter().any(|load| load.pipeline_id == id) {
            return;
        }
        warn!("Page rect message sent to nonexistent pipeline");
    }

    fn handle_new_layout(&self, new_layout_info: NewLayoutInfo, origin: MutableOrigin) {
        let NewLayoutInfo {
            parent_info,
            new_pipeline_id,
            browsing_context_id,
            webview_id,
            opener,
            load_data,
            viewport_details,
        } = new_layout_info;

        // Kick off the fetch for the new resource.
        let url = load_data.url.clone();
        let new_load = InProgressLoad::new(
            new_pipeline_id,
            browsing_context_id,
            webview_id,
            parent_info,
            opener,
            viewport_details,
            origin,
            load_data,
        );
        if url.as_str() == "about:blank" {
            self.start_page_load_about_blank(new_load);
        } else if url.as_str() == "about:srcdoc" {
            self.page_load_about_srcdoc(new_load);
        } else {
            self.pre_page_load(new_load);
        }
    }

    fn collect_reports(&self, reports_chan: ReportsChan) {
        let documents = self.documents.borrow();
        let urls = itertools::join(documents.iter().map(|(_, d)| d.url().to_string()), ", ");

        let prefix = format!("url({urls})");
        let mut reports = self.get_cx().get_reports(prefix.clone());
        for (_, document) in documents.iter() {
            document.window().layout().collect_reports(&mut reports);
        }

        reports.push(self.image_cache.memory_report(&prefix));

        reports_chan.send(ProcessReports::new(reports));
    }

    /// Updates iframe element after a change in visibility
    fn handle_set_throttled_in_containing_iframe_msg(
        &self,
        parent_pipeline_id: PipelineId,
        browsing_context_id: BrowsingContextId,
        throttled: bool,
    ) {
        let iframe = self
            .documents
            .borrow()
            .find_iframe(parent_pipeline_id, browsing_context_id);
        if let Some(iframe) = iframe {
            iframe.set_throttled(throttled);
        }
    }

    fn handle_set_throttled_msg(&self, id: PipelineId, throttled: bool) {
        // Separate message sent since parent script thread could be different (Iframe of different
        // domain)
        self.senders
            .pipeline_to_constellation_sender
            .send((
                id,
                ScriptToConstellationMessage::SetThrottledComplete(throttled),
            ))
            .unwrap();

        let window = self.documents.borrow().find_window(id);
        match window {
            Some(window) => {
                window.set_throttled(throttled);
                return;
            },
            None => {
                let mut loads = self.incomplete_loads.borrow_mut();
                if let Some(ref mut load) = loads.iter_mut().find(|load| load.pipeline_id == id) {
                    load.throttled = throttled;
                    return;
                }
            },
        }

        warn!("SetThrottled sent to nonexistent pipeline");
    }

    /// Handles activity change message
    fn handle_set_document_activity_msg(
        &self,
        id: PipelineId,
        activity: DocumentActivity,
        can_gc: CanGc,
    ) {
        debug!(
            "Setting activity of {} to be {:?} in {:?}.",
            id,
            activity,
            thread::current().name()
        );
        let document = self.documents.borrow().find_document(id);
        if let Some(document) = document {
            document.set_activity(activity, can_gc);
            return;
        }
        let mut loads = self.incomplete_loads.borrow_mut();
        if let Some(ref mut load) = loads.iter_mut().find(|load| load.pipeline_id == id) {
            load.activity = activity;
            return;
        }
        warn!("change of activity sent to nonexistent pipeline");
    }

    fn handle_focus_iframe_msg(
        &self,
        parent_pipeline_id: PipelineId,
        browsing_context_id: BrowsingContextId,
        can_gc: CanGc,
    ) {
        let document = self
            .documents
            .borrow()
            .find_document(parent_pipeline_id)
            .unwrap();

        let Some(iframe_element_root) = ({
            // Enclose `iframes()` call and create a new root to avoid retaining
            // borrow.
            let iframes = document.iframes();
            iframes
                .get(browsing_context_id)
                .map(|iframe| DomRoot::from_ref(iframe.element.upcast()))
        }) else {
            return;
        };

        document.request_focus(Some(&iframe_element_root), FocusType::Parent, can_gc);
    }

    fn handle_post_message_msg(
        &self,
        pipeline_id: PipelineId,
        source_pipeline_id: PipelineId,
        source_browsing_context: WebViewId,
        origin: Option<ImmutableOrigin>,
        source_origin: ImmutableOrigin,
        data: StructuredSerializedData,
    ) {
        let window = self.documents.borrow().find_window(pipeline_id);
        match window {
            None => warn!("postMessage after target pipeline {} closed.", pipeline_id),
            Some(window) => {
                // FIXME: synchronously talks to constellation.
                // send the required info as part of postmessage instead.
                let source = match self.remote_window_proxy(
                    window.upcast::<GlobalScope>(),
                    source_browsing_context,
                    source_pipeline_id,
                    None,
                ) {
                    None => {
                        return warn!(
                            "postMessage after source pipeline {} closed.",
                            source_pipeline_id,
                        );
                    },
                    Some(source) => source,
                };
                // FIXME(#22512): enqueues a task; unnecessary delay.
                window.post_message(origin, source_origin, &source, data)
            },
        }
    }

    fn handle_stop_delaying_load_events_mode(&self, pipeline_id: PipelineId) {
        let window = self.documents.borrow().find_window(pipeline_id);
        if let Some(window) = window {
            match window.undiscarded_window_proxy() {
                Some(window_proxy) => window_proxy.stop_delaying_load_events_mode(),
                None => warn!(
                    "Attempted to take {} of 'delaying-load-events-mode' after having been discarded.",
                    pipeline_id
                ),
            };
        }
    }

    fn handle_unload_document(&self, pipeline_id: PipelineId, can_gc: CanGc) {
        let document = self.documents.borrow().find_document(pipeline_id);
        if let Some(document) = document {
            document.unload(false, can_gc);
        }
    }

    fn handle_update_pipeline_id(
        &self,
        parent_pipeline_id: PipelineId,
        browsing_context_id: BrowsingContextId,
        webview_id: WebViewId,
        new_pipeline_id: PipelineId,
        reason: UpdatePipelineIdReason,
        can_gc: CanGc,
    ) {
        let frame_element = self
            .documents
            .borrow()
            .find_iframe(parent_pipeline_id, browsing_context_id);
        if let Some(frame_element) = frame_element {
            frame_element.update_pipeline_id(new_pipeline_id, reason, can_gc);
        }

        if let Some(window) = self.documents.borrow().find_window(new_pipeline_id) {
            // Ensure that the state of any local window proxies accurately reflects
            // the new pipeline.
            let _ = self.local_window_proxy(
                &window,
                browsing_context_id,
                webview_id,
                Some(parent_pipeline_id),
                // Any local window proxy has already been created, so there
                // is no need to pass along existing opener information that
                // will be discarded.
                None,
            );
        }
    }

    fn handle_update_history_state_msg(
        &self,
        pipeline_id: PipelineId,
        history_state_id: Option<HistoryStateId>,
        url: ServoUrl,
        can_gc: CanGc,
    ) {
        let window = self.documents.borrow().find_window(pipeline_id);
        match window {
            None => {
                warn!(
                    "update history state after pipeline {} closed.",
                    pipeline_id
                );
            },
            Some(window) => window
                .History()
                .activate_state(history_state_id, url, can_gc),
        }
    }

    fn handle_remove_history_states(
        &self,
        pipeline_id: PipelineId,
        history_states: Vec<HistoryStateId>,
    ) {
        let window = self.documents.borrow().find_window(pipeline_id);
        match window {
            None => {
                warn!(
                    "update history state after pipeline {} closed.",
                    pipeline_id
                );
            },
            Some(window) => window.History().remove_states(history_states),
        }
    }

    /// Window was resized, but this script was not active, so don't reflow yet
    fn handle_resize_inactive_msg(&self, id: PipelineId, new_viewport_details: ViewportDetails) {
        let window = self.documents.borrow().find_window(id)
            .expect("ScriptThread: received a resize msg for a pipeline not in this script thread. This is a bug.");
        window.set_viewport_details(new_viewport_details);
    }

    /// We have received notification that the response associated with a load has completed.
    /// Kick off the document and frame tree creation process using the result.
    fn handle_page_headers_available(
        &self,
        id: &PipelineId,
        metadata: Option<Metadata>,
        can_gc: CanGc,
    ) -> Option<DomRoot<ServoParser>> {
        let idx = self
            .incomplete_loads
            .borrow()
            .iter()
            .position(|load| load.pipeline_id == *id);
        // The matching in progress load structure may not exist if
        // the pipeline exited before the page load completed.
        match idx {
            Some(idx) => {
                // https://html.spec.whatwg.org/multipage/#process-a-navigate-response
                // 2. If response's status is 204 or 205, then abort these steps.
                let is20x = match metadata {
                    Some(ref metadata) => metadata.status.in_range(204..=205),
                    _ => false,
                };

                if is20x {
                    // If we have an existing window that is being navigated:
                    if let Some(window) = self.documents.borrow().find_window(*id) {
                        let window_proxy = window.window_proxy();
                        // https://html.spec.whatwg.org/multipage/
                        // #navigating-across-documents:delaying-load-events-mode-2
                        if window_proxy.parent().is_some() {
                            // The user agent must take this nested browsing context
                            // out of the delaying load events mode
                            // when this navigation algorithm later matures,
                            // or when it terminates (whether due to having run all the steps,
                            // or being canceled, or being aborted), whichever happens first.
                            window_proxy.stop_delaying_load_events_mode();
                        }
                    }
                    self.senders
                        .pipeline_to_constellation_sender
                        .send((*id, ScriptToConstellationMessage::AbortLoadUrl))
                        .unwrap();
                    return None;
                };

                let load = self.incomplete_loads.borrow_mut().remove(idx);
                metadata.map(|meta| self.load(meta, load, can_gc))
            },
            None => {
                assert!(self.closed_pipelines.borrow().contains(id));
                None
            },
        }
    }

    /// Handles a request for the window title.
    fn handle_get_title_msg(&self, pipeline_id: PipelineId) {
        let document = match self.documents.borrow().find_document(pipeline_id) {
            Some(document) => document,
            None => return warn!("Message sent to closed pipeline {}.", pipeline_id),
        };
        document.send_title_to_embedder();
    }

    /// Handles a request to exit a pipeline and shut down layout.
    fn handle_exit_pipeline_msg(
        &self,
        id: PipelineId,
        discard_bc: DiscardBrowsingContext,
        can_gc: CanGc,
    ) {
        debug!("{id}: Starting pipeline exit.");

        self.closed_pipelines.borrow_mut().insert(id);

        // Abort the parser, if any,
        // to prevent any further incoming networking messages from being handled.
        let document = self.documents.borrow_mut().remove(id);
        if let Some(document) = document {
            // We should never have a pipeline that's still an incomplete load, but also has a Document.
            debug_assert!(
                !self
                    .incomplete_loads
                    .borrow()
                    .iter()
                    .any(|load| load.pipeline_id == id)
            );

            if let Some(parser) = document.get_current_parser() {
                parser.abort(can_gc);
            }

            debug!("{id}: Shutting down layout");
            document.window().layout_mut().exit_now();

            debug!("{id}: Sending PipelineExited message to constellation");
            self.senders
                .pipeline_to_constellation_sender
                .send((id, ScriptToConstellationMessage::PipelineExited))
                .ok();

            // Clear any active animations and unroot all of the associated DOM objects.
            debug!("{id}: Clearing animations");
            document.animations().clear();

            // We don't want to dispatch `mouseout` event pointing to non-existing element
            if let Some(target) = self.topmost_mouse_over_target.get() {
                if target.upcast::<Node>().owner_doc() == document {
                    self.topmost_mouse_over_target.set(None);
                }
            }

            // We discard the browsing context after requesting layout shut down,
            // to avoid running layout on detached iframes.
            let window = document.window();
            if discard_bc == DiscardBrowsingContext::Yes {
                window.discard_browsing_context();
            }

            debug!("{id}: Clearing JavaScript runtime");
            window.clear_js_runtime();
        }

        debug!("{id}: Finished pipeline exit");
    }

    /// Handles a request to exit the script thread and shut down layout.
    fn handle_exit_script_thread_msg(&self, can_gc: CanGc) {
        debug!("Exiting script thread.");

        let mut pipeline_ids = Vec::new();
        pipeline_ids.extend(
            self.incomplete_loads
                .borrow()
                .iter()
                .next()
                .map(|load| load.pipeline_id),
        );
        pipeline_ids.extend(
            self.documents
                .borrow()
                .iter()
                .next()
                .map(|(pipeline_id, _)| pipeline_id),
        );

        for pipeline_id in pipeline_ids {
            self.handle_exit_pipeline_msg(pipeline_id, DiscardBrowsingContext::Yes, can_gc);
        }

        self.background_hang_monitor.unregister();

        // If we're in multiprocess mode, shut-down the IPC router for this process.
        if opts::get().multiprocess {
            debug!("Exiting IPC router thread in script thread.");
            ROUTER.shutdown();
        }

        debug!("Exited script thread.");
    }

    /// Handles animation tick requested during testing.
    pub(crate) fn handle_tick_all_animations_for_testing(id: PipelineId) {
        with_script_thread(|script_thread| {
            let Some(document) = script_thread.documents.borrow().find_document(id) else {
                warn!("Animation tick for tests for closed pipeline {id}.");
                return;
            };
            document.maybe_mark_animating_nodes_as_dirty();
        });
    }

    /// Handles a Web font being loaded. Does nothing if the page no longer exists.
    fn handle_web_font_loaded(&self, pipeline_id: PipelineId, _success: bool) {
        let Some(document) = self.documents.borrow().find_document(pipeline_id) else {
            warn!("Web font loaded in closed pipeline {}.", pipeline_id);
            return;
        };

        // TODO: This should only dirty nodes that are waiting for a web font to finish loading!
        document.dirty_all_nodes();
    }

    /// Handles a worklet being loaded by triggering a relayout of the page. Does nothing if the
    /// page no longer exists.
    fn handle_worklet_loaded(&self, pipeline_id: PipelineId) {
        if let Some(document) = self.documents.borrow().find_document(pipeline_id) {
            document.set_needs_paint(true)
        }
    }

    /// Notify a window of a storage event
    fn handle_storage_event(
        &self,
        pipeline_id: PipelineId,
        storage_type: StorageType,
        url: ServoUrl,
        key: Option<String>,
        old_value: Option<String>,
        new_value: Option<String>,
    ) {
        let window = match self.documents.borrow().find_window(pipeline_id) {
            None => return warn!("Storage event sent to closed pipeline {}.", pipeline_id),
            Some(window) => window,
        };

        let storage = match storage_type {
            StorageType::Local => window.LocalStorage(),
            StorageType::Session => window.SessionStorage(),
        };

        storage.queue_storage_event(url, key, old_value, new_value);
    }

    /// Notify the containing document of a child iframe that has completed loading.
    fn handle_iframe_load_event(
        &self,
        parent_id: PipelineId,
        browsing_context_id: BrowsingContextId,
        child_id: PipelineId,
        can_gc: CanGc,
    ) {
        let iframe = self
            .documents
            .borrow()
            .find_iframe(parent_id, browsing_context_id);
        match iframe {
            Some(iframe) => iframe.iframe_load_event_steps(child_id, can_gc),
            None => warn!("Message sent to closed pipeline {}.", parent_id),
        }
    }

    fn ask_constellation_for_browsing_context_info(
        &self,
        pipeline_id: PipelineId,
    ) -> Option<(BrowsingContextId, Option<PipelineId>)> {
        let (result_sender, result_receiver) = ipc::channel().unwrap();
        let msg = ScriptToConstellationMessage::GetBrowsingContextInfo(pipeline_id, result_sender);
        self.senders
            .pipeline_to_constellation_sender
            .send((pipeline_id, msg))
            .expect("Failed to send to constellation.");
        result_receiver
            .recv()
            .expect("Failed to get browsing context info from constellation.")
    }

    fn ask_constellation_for_top_level_info(
        &self,
        sender_pipeline: PipelineId,
        browsing_context_id: BrowsingContextId,
    ) -> Option<WebViewId> {
        let (result_sender, result_receiver) = ipc::channel().unwrap();
        let msg = ScriptToConstellationMessage::GetTopForBrowsingContext(
            browsing_context_id,
            result_sender,
        );
        self.senders
            .pipeline_to_constellation_sender
            .send((sender_pipeline, msg))
            .expect("Failed to send to constellation.");
        result_receiver
            .recv()
            .expect("Failed to get top-level id from constellation.")
    }

    // Get the browsing context for a pipeline that may exist in another
    // script thread.  If the browsing context already exists in the
    // `window_proxies` map, we return it, otherwise we recursively
    // get the browsing context for the parent if there is one,
    // construct a new dissimilar-origin browsing context, add it
    // to the `window_proxies` map, and return it.
    fn remote_window_proxy(
        &self,
        global_to_clone: &GlobalScope,
        webview_id: WebViewId,
        pipeline_id: PipelineId,
        opener: Option<BrowsingContextId>,
    ) -> Option<DomRoot<WindowProxy>> {
        let (browsing_context_id, parent_pipeline_id) =
            self.ask_constellation_for_browsing_context_info(pipeline_id)?;
        if let Some(window_proxy) = self.window_proxies.borrow().get(&browsing_context_id) {
            return Some(DomRoot::from_ref(window_proxy));
        }

        let parent_browsing_context = parent_pipeline_id.and_then(|parent_id| {
            self.remote_window_proxy(global_to_clone, webview_id, parent_id, opener)
        });

        let opener_browsing_context = opener.and_then(ScriptThread::find_window_proxy);

        let creator = CreatorBrowsingContextInfo::from(
            parent_browsing_context.as_deref(),
            opener_browsing_context.as_deref(),
        );

        let window_proxy = WindowProxy::new_dissimilar_origin(
            global_to_clone,
            browsing_context_id,
            webview_id,
            parent_browsing_context.as_deref(),
            opener,
            creator,
        );
        self.window_proxies
            .borrow_mut()
            .insert(browsing_context_id, Dom::from_ref(&*window_proxy));
        Some(window_proxy)
    }

    // Get the browsing context for a pipeline that exists in this
    // script thread.  If the browsing context already exists in the
    // `window_proxies` map, we return it, otherwise we recursively
    // get the browsing context for the parent if there is one,
    // construct a new similar-origin browsing context, add it
    // to the `window_proxies` map, and return it.
    fn local_window_proxy(
        &self,
        window: &Window,
        browsing_context_id: BrowsingContextId,
        webview_id: WebViewId,
        parent_info: Option<PipelineId>,
        opener: Option<BrowsingContextId>,
    ) -> DomRoot<WindowProxy> {
        if let Some(window_proxy) = self.window_proxies.borrow().get(&browsing_context_id) {
            // Note: we do not set the window to be the currently-active one,
            // this will be done instead when the script-thread handles the `SetDocumentActivity` msg.
            return DomRoot::from_ref(window_proxy);
        }
        let iframe = parent_info.and_then(|parent_id| {
            self.documents
                .borrow()
                .find_iframe(parent_id, browsing_context_id)
        });
        let parent_browsing_context = match (parent_info, iframe.as_ref()) {
            (_, Some(iframe)) => Some(iframe.owner_window().window_proxy()),
            (Some(parent_id), _) => {
                self.remote_window_proxy(window.upcast(), webview_id, parent_id, opener)
            },
            _ => None,
        };

        let opener_browsing_context = opener.and_then(ScriptThread::find_window_proxy);

        let creator = CreatorBrowsingContextInfo::from(
            parent_browsing_context.as_deref(),
            opener_browsing_context.as_deref(),
        );

        let window_proxy = WindowProxy::new(
            window,
            browsing_context_id,
            webview_id,
            iframe.as_deref().map(Castable::upcast),
            parent_browsing_context.as_deref(),
            opener,
            creator,
        );
        self.window_proxies
            .borrow_mut()
            .insert(browsing_context_id, Dom::from_ref(&*window_proxy));
        window_proxy
    }

    /// The entry point to document loading. Defines bindings, sets up the window and document
    /// objects, parses HTML and CSS, and kicks off initial layout.
    fn load(
        &self,
        metadata: Metadata,
        incomplete: InProgressLoad,
        can_gc: CanGc,
    ) -> DomRoot<ServoParser> {
        let final_url = metadata.final_url.clone();
        {
            self.senders
                .pipeline_to_constellation_sender
                .send((
                    incomplete.pipeline_id,
                    ScriptToConstellationMessage::SetFinalUrl(final_url.clone()),
                ))
                .unwrap();
        }
        debug!(
            "ScriptThread: loading {} on pipeline {:?}",
            incomplete.load_data.url, incomplete.pipeline_id
        );

        let origin = if final_url.as_str() == "about:blank" || final_url.as_str() == "about:srcdoc"
        {
            incomplete.origin.clone()
        } else {
            MutableOrigin::new(final_url.origin())
        };

        let script_to_constellation_chan = ScriptToConstellationChan {
            sender: self.senders.pipeline_to_constellation_sender.clone(),
            pipeline_id: incomplete.pipeline_id,
        };

        let font_context = Arc::new(FontContext::new(
            self.system_font_service.clone(),
            self.compositor_api.clone(),
            self.resource_threads.clone(),
        ));

        let layout_config = LayoutConfig {
            id: incomplete.pipeline_id,
            webview_id: incomplete.webview_id,
            url: final_url.clone(),
            is_iframe: incomplete.parent_info.is_some(),
            script_chan: self.senders.constellation_sender.clone(),
            image_cache: self.image_cache.clone(),
            font_context: font_context.clone(),
            time_profiler_chan: self.senders.time_profiler_sender.clone(),
            compositor_api: self.compositor_api.clone(),
            viewport_details: incomplete.viewport_details,
        };

        // Create the window and document objects.
        let window = Window::new(
            incomplete.webview_id,
            self.js_runtime.clone(),
            self.senders.self_sender.clone(),
            self.layout_factory.create(layout_config),
            font_context,
            self.senders.image_cache_sender.clone(),
            self.image_cache.clone(),
            self.resource_threads.clone(),
            #[cfg(feature = "bluetooth")]
            self.senders.bluetooth_sender.clone(),
            self.senders.memory_profiler_sender.clone(),
            self.senders.time_profiler_sender.clone(),
            self.senders.devtools_server_sender.clone(),
            script_to_constellation_chan,
            self.senders.constellation_sender.clone(),
            incomplete.pipeline_id,
            incomplete.parent_info,
            incomplete.viewport_details,
            origin.clone(),
            final_url.clone(),
            incomplete.navigation_start,
            self.webgl_chan.as_ref().map(|chan| chan.channel()),
            #[cfg(feature = "webxr")]
            self.webxr_registry.clone(),
            self.microtask_queue.clone(),
            self.webrender_document,
            self.compositor_api.clone(),
            self.relayout_event,
            self.unminify_js,
            self.unminify_css,
            self.local_script_source.clone(),
            self.user_content_manager.clone(),
            self.player_context.clone(),
            #[cfg(feature = "webgpu")]
            self.gpu_id_hub.clone(),
            incomplete.load_data.inherited_secure_context,
        );

        let _realm = enter_realm(&*window);

        // Initialize the browsing context for the window.
        let window_proxy = self.local_window_proxy(
            &window,
            incomplete.browsing_context_id,
            incomplete.webview_id,
            incomplete.parent_info,
            incomplete.opener,
        );
        if window_proxy.parent().is_some() {
            // https://html.spec.whatwg.org/multipage/#navigating-across-documents:delaying-load-events-mode-2
            // The user agent must take this nested browsing context
            // out of the delaying load events mode
            // when this navigation algorithm later matures.
            window_proxy.stop_delaying_load_events_mode();
        }
        window.init_window_proxy(&window_proxy);

        let last_modified = metadata.headers.as_ref().and_then(|headers| {
            headers.typed_get::<LastModified>().map(|tm| {
                let tm: SystemTime = tm.into();
                let local_time: DateTime<Local> = tm.into();
                local_time.format("%m/%d/%Y %H:%M:%S").to_string()
            })
        });

        let loader = DocumentLoader::new_with_threads(
            self.resource_threads.clone(),
            Some(final_url.clone()),
        );

        let content_type: Option<Mime> = metadata
            .content_type
            .map(Serde::into_inner)
            .map(Mime::from_ct);

        let is_html_document = match content_type {
            Some(ref mime) if mime.type_ == APPLICATION && mime.has_suffix("xml") => {
                IsHTMLDocument::NonHTMLDocument
            },

            Some(ref mime) if mime.matches(TEXT, XML) || mime.matches(APPLICATION, XML) => {
                IsHTMLDocument::NonHTMLDocument
            },
            _ => IsHTMLDocument::HTMLDocument,
        };

        let referrer = metadata
            .referrer
            .as_ref()
            .map(|referrer| referrer.clone().into_string());

        let is_initial_about_blank = final_url.as_str() == "about:blank";

        let document = Document::new(
            &window,
            HasBrowsingContext::Yes,
            Some(final_url.clone()),
            origin,
            is_html_document,
            content_type,
            last_modified,
            incomplete.activity,
            DocumentSource::FromParser,
            loader,
            referrer,
            Some(metadata.status.raw_code()),
            incomplete.canceller,
            is_initial_about_blank,
            true,
            incomplete.load_data.inherited_insecure_requests_policy,
            incomplete.load_data.has_trustworthy_ancestor_origin,
            can_gc,
        );

        let referrer_policy = metadata
            .headers
            .as_deref()
            .and_then(|h| h.typed_get::<ReferrerPolicyHeader>())
            .into();
        document.set_referrer_policy(referrer_policy);

        let refresh_header = metadata.headers.as_deref().and_then(|h| h.get(REFRESH));
        if let Some(refresh_val) = refresh_header {
            // There are tests that this header handles Unicode code points
            document.shared_declarative_refresh_steps(refresh_val.as_bytes());
        }

        document.set_ready_state(DocumentReadyState::Loading, can_gc);

        self.documents
            .borrow_mut()
            .insert(incomplete.pipeline_id, &document);

        window.init_document(&document);

        // For any similar-origin iframe, ensure that the contentWindow/contentDocument
        // APIs resolve to the new window/document as soon as parsing starts.
        if let Some(frame) = window_proxy
            .frame_element()
            .and_then(|e| e.downcast::<HTMLIFrameElement>())
        {
            let parent_pipeline = frame.global().pipeline_id();
            self.handle_update_pipeline_id(
                parent_pipeline,
                window_proxy.browsing_context_id(),
                window_proxy.webview_id(),
                incomplete.pipeline_id,
                UpdatePipelineIdReason::Navigation,
                can_gc,
            );
        }

        self.senders
            .pipeline_to_constellation_sender
            .send((
                incomplete.pipeline_id,
                ScriptToConstellationMessage::ActivateDocument,
            ))
            .unwrap();

        // Notify devtools that a new script global exists.
        let is_top_level_global = incomplete.webview_id.0 == incomplete.browsing_context_id;
        self.notify_devtools(
            document.Title(),
            final_url.clone(),
            is_top_level_global,
            (
                incomplete.browsing_context_id,
                incomplete.pipeline_id,
                None,
                incomplete.webview_id,
            ),
        );

        document.set_https_state(metadata.https_state);
        document.set_navigation_start(incomplete.navigation_start);

        if is_html_document == IsHTMLDocument::NonHTMLDocument {
            ServoParser::parse_xml_document(&document, None, final_url, can_gc);
        } else {
            ServoParser::parse_html_document(&document, None, final_url, can_gc);
        }

        if incomplete.activity == DocumentActivity::FullyActive {
            window.resume(can_gc);
        } else {
            window.suspend(can_gc);
        }

        if incomplete.throttled {
            window.set_throttled(true);
        }

        document.get_current_parser().unwrap()
    }

    fn notify_devtools(
        &self,
        title: DOMString,
        url: ServoUrl,
        is_top_level_global: bool,
        (browsing_context_id, pipeline_id, worker_id, webview_id): (
            BrowsingContextId,
            PipelineId,
            Option<WorkerId>,
            WebViewId,
        ),
    ) {
        if let Some(ref chan) = self.senders.devtools_server_sender {
            let page_info = DevtoolsPageInfo {
                title: String::from(title),
                url,
                is_top_level_global,
            };
            chan.send(ScriptToDevtoolsControlMsg::NewGlobal(
                (browsing_context_id, pipeline_id, worker_id, webview_id),
                self.senders.devtools_client_to_script_thread_sender.clone(),
                page_info.clone(),
            ))
            .unwrap();

            let state = NavigationState::Stop(pipeline_id, page_info);
            let _ = chan.send(ScriptToDevtoolsControlMsg::Navigate(
                browsing_context_id,
                state,
            ));
        }
    }

    /// Queue compositor events for later dispatching as part of a
    /// `update_the_rendering` task.
    fn handle_input_event(&self, pipeline_id: PipelineId, event: ConstellationInputEvent) {
        let Some(document) = self.documents.borrow().find_document(pipeline_id) else {
            warn!("Compositor event sent to closed pipeline {pipeline_id}.");
            return;
        };
        document.note_pending_input_event(event);
    }

    /// Handle a "navigate an iframe" message from the constellation.
    fn handle_navigate_iframe(
        &self,
        parent_pipeline_id: PipelineId,
        browsing_context_id: BrowsingContextId,
        load_data: LoadData,
        history_handling: NavigationHistoryBehavior,
        can_gc: CanGc,
    ) {
        let iframe = self
            .documents
            .borrow()
            .find_iframe(parent_pipeline_id, browsing_context_id);
        if let Some(iframe) = iframe {
            iframe.navigate_or_reload_child_browsing_context(load_data, history_handling, can_gc);
        }
    }

    /// Turn javascript: URL into JS code to eval, according to the steps in
    /// <https://html.spec.whatwg.org/multipage/#javascript-protocol>
    pub(crate) fn eval_js_url(global_scope: &GlobalScope, load_data: &mut LoadData, can_gc: CanGc) {
        // This slice of the URL’s serialization is equivalent to (5.) to (7.):
        // Start with the scheme data of the parsed URL;
        // append question mark and query component, if any;
        // append number sign and fragment component if any.
        let encoded = &load_data.url.clone()[Position::BeforePath..];

        // Percent-decode (8.) and UTF-8 decode (9.)
        let script_source = percent_decode(encoded.as_bytes()).decode_utf8_lossy();

        // Script source is ready to be evaluated (11.)
        let _ac = enter_realm(global_scope);
        rooted!(in(*GlobalScope::get_cx()) let mut jsval = UndefinedValue());
        global_scope.evaluate_js_on_global_with_result(
            &script_source,
            jsval.handle_mut(),
            ScriptFetchOptions::default_classic_script(global_scope),
            global_scope.api_base_url(),
            can_gc,
        );

        load_data.js_eval_result = if jsval.get().is_string() {
            unsafe {
                let strval = DOMString::from_jsval(
                    *GlobalScope::get_cx(),
                    jsval.handle(),
                    StringificationBehavior::Empty,
                );
                match strval {
                    Ok(ConversionResult::Success(s)) => {
                        Some(JsEvalResult::Ok(String::from(s).as_bytes().to_vec()))
                    },
                    _ => None,
                }
            }
        } else {
            Some(JsEvalResult::NoContent)
        };

        load_data.url = ServoUrl::parse("about:blank").unwrap();
    }

    /// Instructs the constellation to fetch the document that will be loaded. Stores the InProgressLoad
    /// argument until a notification is received that the fetch is complete.
    fn pre_page_load(&self, mut incomplete: InProgressLoad) {
        let context = ParserContext::new(incomplete.pipeline_id, incomplete.load_data.url.clone());
        self.incomplete_parser_contexts
            .0
            .borrow_mut()
            .push((incomplete.pipeline_id, context));

        let request_builder = incomplete.request_builder();
        incomplete.canceller = FetchCanceller::new(request_builder.id);
        NavigationListener::new(request_builder, self.senders.self_sender.clone())
            .initiate_fetch(&self.resource_threads.core_thread, None);
        self.incomplete_loads.borrow_mut().push(incomplete);
    }

    fn handle_navigation_response(&self, pipeline_id: PipelineId, message: FetchResponseMsg) {
        if let Some(metadata) = NavigationListener::http_redirect_metadata(&message) {
            self.handle_navigation_redirect(pipeline_id, metadata);
            return;
        };

        match message {
            FetchResponseMsg::ProcessResponse(request_id, metadata) => {
                self.handle_fetch_metadata(pipeline_id, request_id, metadata)
            },
            FetchResponseMsg::ProcessResponseChunk(request_id, chunk) => {
                self.handle_fetch_chunk(pipeline_id, request_id, chunk)
            },
            FetchResponseMsg::ProcessResponseEOF(request_id, eof) => {
                self.handle_fetch_eof(pipeline_id, request_id, eof)
            },
            FetchResponseMsg::ProcessCspViolations(request_id, violations) => {
                self.handle_csp_violations(pipeline_id, request_id, violations)
            },
            FetchResponseMsg::ProcessRequestBody(..) | FetchResponseMsg::ProcessRequestEOF(..) => {
            },
        }
    }

    fn handle_fetch_metadata(
        &self,
        id: PipelineId,
        request_id: RequestId,
        fetch_metadata: Result<FetchMetadata, NetworkError>,
    ) {
        match fetch_metadata {
            Ok(_) => (),
            Err(NetworkError::Crash(..)) => (),
            Err(ref e) => {
                warn!("Network error: {:?}", e);
            },
        };

        let mut incomplete_parser_contexts = self.incomplete_parser_contexts.0.borrow_mut();
        let parser = incomplete_parser_contexts
            .iter_mut()
            .find(|&&mut (pipeline_id, _)| pipeline_id == id);
        if let Some(&mut (_, ref mut ctxt)) = parser {
            ctxt.process_response(request_id, fetch_metadata);
        }
    }

    fn handle_fetch_chunk(&self, pipeline_id: PipelineId, request_id: RequestId, chunk: Vec<u8>) {
        let mut incomplete_parser_contexts = self.incomplete_parser_contexts.0.borrow_mut();
        let parser = incomplete_parser_contexts
            .iter_mut()
            .find(|&&mut (parser_pipeline_id, _)| parser_pipeline_id == pipeline_id);
        if let Some(&mut (_, ref mut ctxt)) = parser {
            ctxt.process_response_chunk(request_id, chunk);
        }
    }

    fn handle_fetch_eof(
        &self,
        id: PipelineId,
        request_id: RequestId,
        eof: Result<ResourceFetchTiming, NetworkError>,
    ) {
        let idx = self
            .incomplete_parser_contexts
            .0
            .borrow()
            .iter()
            .position(|&(pipeline_id, _)| pipeline_id == id);

        if let Some(idx) = idx {
            let (_, mut ctxt) = self.incomplete_parser_contexts.0.borrow_mut().remove(idx);
            ctxt.process_response_eof(request_id, eof);
        }
    }

    fn handle_csp_violations(&self, id: PipelineId, _: RequestId, violations: Vec<csp::Violation>) {
        if let Some(global) = self.documents.borrow().find_global(id) {
            global.report_csp_violations(violations);
        }
    }

    fn handle_navigation_redirect(&self, id: PipelineId, metadata: &Metadata) {
        // TODO(mrobinson): This tries to accomplish some steps from
        // <https://html.spec.whatwg.org/multipage/#process-a-navigate-fetch>, but it's
        // very out of sync with the specification.
        assert!(metadata.location_url.is_some());

        let mut incomplete_loads = self.incomplete_loads.borrow_mut();
        let Some(incomplete_load) = incomplete_loads
            .iter_mut()
            .find(|incomplete_load| incomplete_load.pipeline_id == id)
        else {
            return;
        };

        // Update the `url_list` of the incomplete load to track all redirects. This will be reflected
        // in the new `RequestBuilder` as well.
        incomplete_load.url_list.push(metadata.final_url.clone());

        let mut request_builder = incomplete_load.request_builder();
        request_builder.referrer = metadata
            .referrer
            .clone()
            .map(Referrer::ReferrerUrl)
            .unwrap_or(Referrer::NoReferrer);
        request_builder.referrer_policy = metadata.referrer_policy;

        let headers = metadata
            .headers
            .as_ref()
            .map(|headers| headers.clone().into_inner())
            .unwrap_or_default();

        let response_init = Some(ResponseInit {
            url: metadata.final_url.clone(),
            location_url: metadata.location_url.clone(),
            headers,
            referrer: metadata.referrer.clone(),
            status_code: metadata
                .status
                .try_code()
                .map(|code| code.as_u16())
                .unwrap_or(200),
        });

        incomplete_load.canceller = FetchCanceller::new(request_builder.id);
        NavigationListener::new(request_builder, self.senders.self_sender.clone())
            .initiate_fetch(&self.resource_threads.core_thread, response_init);
    }

    /// Synchronously fetch `about:blank`. Stores the `InProgressLoad`
    /// argument until a notification is received that the fetch is complete.
    fn start_page_load_about_blank(&self, mut incomplete: InProgressLoad) {
        let id = incomplete.pipeline_id;

        let url = ServoUrl::parse("about:blank").unwrap();
        let mut context = ParserContext::new(id, url.clone());

        let mut meta = Metadata::default(url);
        meta.set_content_type(Some(&mime::TEXT_HTML));
        meta.set_referrer_policy(incomplete.load_data.referrer_policy);

        // If this page load is the result of a javascript scheme url, map
        // the evaluation result into a response.
        let chunk = match incomplete.load_data.js_eval_result {
            Some(JsEvalResult::Ok(ref mut content)) => std::mem::take(content),
            Some(JsEvalResult::NoContent) => {
                meta.status = http::StatusCode::NO_CONTENT.into();
                vec![]
            },
            None => vec![],
        };

        self.incomplete_loads.borrow_mut().push(incomplete);

        let dummy_request_id = RequestId::default();
        context.process_response(dummy_request_id, Ok(FetchMetadata::Unfiltered(meta)));
        context.process_response_chunk(dummy_request_id, chunk);
        context.process_response_eof(
            dummy_request_id,
            Ok(ResourceFetchTiming::new(ResourceTimingType::None)),
        );
    }

    /// Synchronously parse a srcdoc document from a giving HTML string.
    fn page_load_about_srcdoc(&self, mut incomplete: InProgressLoad) {
        let id = incomplete.pipeline_id;

        let url = ServoUrl::parse("about:srcdoc").unwrap();
        let mut meta = Metadata::default(url.clone());
        meta.set_content_type(Some(&mime::TEXT_HTML));
        meta.set_referrer_policy(incomplete.load_data.referrer_policy);

        let srcdoc = std::mem::take(&mut incomplete.load_data.srcdoc);
        let chunk = srcdoc.into_bytes();

        self.incomplete_loads.borrow_mut().push(incomplete);

        let mut context = ParserContext::new(id, url);
        let dummy_request_id = RequestId::default();

        context.process_response(dummy_request_id, Ok(FetchMetadata::Unfiltered(meta)));
        context.process_response_chunk(dummy_request_id, chunk);
        context.process_response_eof(
            dummy_request_id,
            Ok(ResourceFetchTiming::new(ResourceTimingType::None)),
        );
    }

    fn handle_css_error_reporting(
        &self,
        pipeline_id: PipelineId,
        filename: String,
        line: u32,
        column: u32,
        msg: String,
    ) {
        let sender = match self.senders.devtools_server_sender {
            Some(ref sender) => sender,
            None => return,
        };

        if let Some(global) = self.documents.borrow().find_global(pipeline_id) {
            if global.live_devtools_updates() {
                let css_error = CSSError {
                    filename,
                    line,
                    column,
                    msg,
                };
                let message = ScriptToDevtoolsControlMsg::ReportCSSError(pipeline_id, css_error);
                sender.send(message).unwrap();
            }
        }
    }

    fn handle_reload(&self, pipeline_id: PipelineId, can_gc: CanGc) {
        let window = self.documents.borrow().find_window(pipeline_id);
        if let Some(window) = window {
            window.Location().reload_without_origin_check(can_gc);
        }
    }

    fn handle_paint_metric(
        &self,
        pipeline_id: PipelineId,
        metric_type: ProgressiveWebMetricType,
        metric_value: CrossProcessInstant,
        first_reflow: bool,
        can_gc: CanGc,
    ) {
        match self.documents.borrow().find_document(pipeline_id) {
            Some(document) => {
                document.handle_paint_metric(metric_type, metric_value, first_reflow, can_gc)
            },
            None => warn!(
                "Received paint metric ({metric_type:?}) for unknown document: {pipeline_id:?}"
            ),
        }
    }

    fn handle_media_session_action(
        &self,
        pipeline_id: PipelineId,
        action: MediaSessionActionType,
        can_gc: CanGc,
    ) {
        if let Some(window) = self.documents.borrow().find_window(pipeline_id) {
            let media_session = window.Navigator().MediaSession();
            media_session.handle_action(action, can_gc);
        } else {
            warn!("No MediaSession for this pipeline ID");
        };
    }

    pub(crate) fn enqueue_microtask(job: Microtask) {
        with_script_thread(|script_thread| {
            script_thread
                .microtask_queue
                .enqueue(job, script_thread.get_cx());
        });
    }

    fn perform_a_microtask_checkpoint(&self, can_gc: CanGc) {
        // Only perform the checkpoint if we're not shutting down.
        if self.can_continue_running_inner() {
            let globals = self
                .documents
                .borrow()
                .iter()
                .map(|(_id, document)| DomRoot::from_ref(document.window().upcast()))
                .collect();

            self.microtask_queue.checkpoint(
                self.get_cx(),
                |id| self.documents.borrow().find_global(id),
                globals,
                can_gc,
            )
        }
    }
}

impl Drop for ScriptThread {
    fn drop(&mut self) {
        SCRIPT_THREAD_ROOT.with(|root| {
            root.set(None);
        });
    }
}
