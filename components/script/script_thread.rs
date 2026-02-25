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
use std::collections::HashSet;
use std::default::Default;
use std::option::Option;
use std::rc::{Rc, Weak};
use std::result::Result;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::thread::{self, JoinHandle};
use std::time::{Duration, Instant, SystemTime};

use background_hang_monitor_api::{
    BackgroundHangMonitor, BackgroundHangMonitorExitSignal, BackgroundHangMonitorRegister,
    HangAnnotation, MonitoredComponentId, MonitoredComponentType,
};
use base::cross_process_instant::CrossProcessInstant;
use base::generic_channel;
use base::id::{
    BrowsingContextId, HistoryStateId, PipelineId, PipelineNamespace, ScriptEventLoopId,
    TEST_WEBVIEW_ID, WebViewId,
};
use canvas_traits::webgl::WebGLPipeline;
use chrono::{DateTime, Local};
use constellation_traits::{
    JsEvalResult, LoadData, LoadOrigin, NavigationHistoryBehavior, ScreenshotReadinessResponse,
    ScriptToConstellationChan, ScriptToConstellationMessage, ScrollStateUpdate,
    StructuredSerializedData, WindowSizeType,
};
use crossbeam_channel::unbounded;
use data_url::mime::Mime;
use devtools_traits::{
    CSSError, DevtoolScriptControlMsg, DevtoolsPageInfo, NavigationState,
    ScriptToDevtoolsControlMsg, WorkerId,
};
use embedder_traits::user_contents::{UserContentManagerId, UserContents, UserScript};
use embedder_traits::{
    EmbedderControlId, EmbedderControlResponse, EmbedderMsg, FocusSequenceNumber,
    JavaScriptEvaluationError, JavaScriptEvaluationId, MediaSessionActionType, Theme,
    ViewportDetails, WebDriverScriptCommand,
};
use encoding_rs::Encoding;
use fonts::{FontContext, SystemFontServiceProxy};
use headers::{HeaderMapExt, LastModified, ReferrerPolicy as ReferrerPolicyHeader};
use http::header::REFRESH;
use hyper_serde::Serde;
use ipc_channel::ipc;
use ipc_channel::router::ROUTER;
use js::glue::GetWindowProxyClass;
use js::jsapi::JSContext as UnsafeJSContext;
use js::jsval::UndefinedValue;
use js::rust::ParentRuntime;
use js::rust::wrappers2::{JS_AddInterruptCallback, SetWindowProxyClass};
use layout_api::{LayoutConfig, LayoutFactory, RestyleReason, ScriptThreadFactory};
use media::WindowGLContext;
use metrics::MAX_TASK_NS;
use net_traits::image_cache::{ImageCache, ImageCacheFactory, ImageCacheResponseMessage};
use net_traits::request::{Referrer, RequestId};
use net_traits::response::ResponseInit;
use net_traits::{
    FetchMetadata, FetchResponseMsg, Metadata, NetworkError, ResourceFetchTiming, ResourceThreads,
    ResourceTimingType,
};
use paint_api::{CrossProcessPaintApi, PinchZoomInfos, PipelineExitSource};
use percent_encoding::percent_decode;
use profile_traits::mem::{ProcessReports, ReportsChan, perform_memory_report};
use profile_traits::time::ProfilerCategory;
use profile_traits::time_profile;
use rustc_hash::{FxHashMap, FxHashSet};
use script_bindings::script_runtime::{JSContext, temp_cx};
use script_bindings::settings_stack::run_a_script;
use script_traits::{
    ConstellationInputEvent, DiscardBrowsingContext, DocumentActivity, InitialScriptState,
    NewPipelineInfo, Painter, ProgressiveWebMetricType, ScriptThreadMessage,
    UpdatePipelineIdReason,
};
use servo_arc::Arc as ServoArc;
use servo_config::{opts, pref, prefs};
use servo_url::{ImmutableOrigin, MutableOrigin, OriginSnapshot, ServoUrl};
use storage_traits::StorageThreads;
use storage_traits::webstorage_thread::WebStorageType;
use style::context::QuirksMode;
use style::error_reporting::RustLogReporter;
use style::global_style_data::GLOBAL_STYLE_DATA;
use style::media_queries::MediaList;
use style::stylesheets::{AllowImportRules, DocumentStyleSheet, Origin, Stylesheet};
use style::thread_state::{self, ThreadState};
use stylo_atoms::Atom;
use timers::{TimerEventRequest, TimerId, TimerScheduler};
use url::Position;
#[cfg(feature = "webgpu")]
use webgpu_traits::{WebGPUDevice, WebGPUMsg};

use crate::devtools::DevtoolsState;
use crate::document_collection::DocumentCollection;
use crate::document_loader::DocumentLoader;
use crate::dom::bindings::cell::DomRefCell;
use crate::dom::bindings::codegen::Bindings::DocumentBinding::{
    DocumentMethods, DocumentReadyState,
};
use crate::dom::bindings::codegen::Bindings::NavigatorBinding::NavigatorMethods;
use crate::dom::bindings::codegen::Bindings::WindowBinding::WindowMethods;
use crate::dom::bindings::conversions::{
    ConversionResult, SafeFromJSValConvertible, StringificationBehavior,
};
use crate::dom::bindings::inheritance::Castable;
use crate::dom::bindings::refcounted::Trusted;
use crate::dom::bindings::reflector::DomGlobal;
use crate::dom::bindings::root::{Dom, DomRoot};
use crate::dom::bindings::str::DOMString;
use crate::dom::csp::{CspReporting, GlobalCspReporting, Violation};
use crate::dom::customelementregistry::{
    CallbackReaction, CustomElementDefinition, CustomElementReactionStack,
};
use crate::dom::document::{
    Document, DocumentSource, FocusInitiator, HasBrowsingContext, IsHTMLDocument,
    RenderingUpdateReason,
};
use crate::dom::element::Element;
use crate::dom::globalscope::GlobalScope;
use crate::dom::html::htmliframeelement::{HTMLIFrameElement, IframeContext};
use crate::dom::node::{Node, NodeTraits};
use crate::dom::servoparser::{ParserContext, ServoParser};
use crate::dom::types::DebuggerGlobalScope;
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
use crate::mime::{APPLICATION, CHARSET, MimeExt, TEXT, XML};
use crate::navigation::{InProgressLoad, NavigationListener};
use crate::network_listener::{FetchResponseListener, submit_timing};
use crate::realms::{enter_auto_realm, enter_realm};
use crate::script_mutation_observers::ScriptMutationObservers;
use crate::script_runtime::{
    CanGc, IntroductionType, JSContextHelper, Runtime, ScriptThreadEventCategory,
    ThreadSafeJSContext,
};
use crate::script_window_proxies::ScriptWindowProxies;
use crate::task_queue::TaskQueue;
use crate::webdriver_handlers::jsval_to_webdriver;
use crate::{DomTypeHolder, devtools, webdriver_handlers};

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

// We borrow the incomplete parser contexts mutably during parsing,
// which is fine except that parsing can trigger evaluation,
// which can trigger GC, and so we can end up tracing the script
// thread during parsing. For this reason, we don't trace the
// incomplete parser contexts during GC.
pub(crate) struct IncompleteParserContexts(RefCell<Vec<(PipelineId, ParserContext)>>);

unsafe_no_jsmanaged_fields!(TaskQueue<MainThreadScriptMsg>);

type NodeIdSet = HashSet<String>;

/// A simple guard structure that restore the user interacting state when dropped
#[derive(Default)]
pub(crate) struct ScriptUserInteractingGuard {
    was_interacting: bool,
    user_interaction_cell: Rc<Cell<bool>>,
}

impl ScriptUserInteractingGuard {
    fn new(user_interaction_cell: Rc<Cell<bool>>) -> Self {
        let was_interacting = user_interaction_cell.get();
        user_interaction_cell.set(true);
        Self {
            was_interacting,
            user_interaction_cell,
        }
    }
}

impl Drop for ScriptUserInteractingGuard {
    fn drop(&mut self) {
        self.user_interaction_cell.set(self.was_interacting)
    }
}

/// This is the `ScriptThread`'s version of [`UserContents`] with the difference that user
/// stylesheets are represented as parsed `DocumentStyleSheet`s instead of simple source strings.
struct ScriptThreadUserContents {
    user_scripts: Rc<Vec<UserScript>>,
    user_stylesheets: Rc<Vec<DocumentStyleSheet>>,
}

impl From<UserContents> for ScriptThreadUserContents {
    fn from(user_contents: UserContents) -> Self {
        let shared_lock = &GLOBAL_STYLE_DATA.shared_lock;
        let user_stylesheets = user_contents
            .stylesheets
            .iter()
            .map(|user_stylesheet| {
                DocumentStyleSheet(ServoArc::new(Stylesheet::from_str(
                    user_stylesheet.source(),
                    user_stylesheet.url().into(),
                    Origin::User,
                    ServoArc::new(shared_lock.wrap(MediaList::empty())),
                    shared_lock.clone(),
                    None,
                    Some(&RustLogReporter),
                    QuirksMode::NoQuirks,
                    AllowImportRules::Yes,
                )))
            })
            .collect();
        Self {
            user_scripts: Rc::new(user_contents.scripts),
            user_stylesheets: Rc::new(user_stylesheets),
        }
    }
}

#[derive(JSTraceable)]
// ScriptThread instances are rooted on creation, so this is okay
#[cfg_attr(crown, expect(crown::unrooted_must_root))]
pub struct ScriptThread {
    /// A reference to the currently operating `ScriptThread`. This should always be
    /// upgradable to an `Rc` as long as the `ScriptThread` is running.
    #[no_trace]
    this: Weak<ScriptThread>,

    /// <https://html.spec.whatwg.org/multipage/#last-render-opportunity-time>
    last_render_opportunity_time: Cell<Option<Instant>>,

    /// The documents for pipelines managed by this thread
    documents: DomRefCell<DocumentCollection>,
    /// The window proxies known by this thread
    window_proxies: Rc<ScriptWindowProxies>,
    /// A list of data pertaining to loads that have not yet received a network response
    incomplete_loads: DomRefCell<Vec<InProgressLoad>>,
    /// A vector containing parser contexts which have not yet been fully processed
    incomplete_parser_contexts: IncompleteParserContexts,
    /// An [`ImageCacheFactory`] to use for creating [`ImageCache`]s for all of the
    /// child `Pipeline`s.
    #[no_trace]
    image_cache_factory: Arc<dyn ImageCacheFactory>,

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

    #[no_trace]
    storage_threads: StorageThreads,

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

    /// List of pipelines that have been owned and closed by this script thread.
    #[no_trace]
    closed_pipelines: DomRefCell<FxHashSet<PipelineId>>,

    /// <https://html.spec.whatwg.org/multipage/#microtask-queue>
    microtask_queue: Rc<MicrotaskQueue>,

    mutation_observers: Rc<ScriptMutationObservers>,

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
    docs_with_no_blocking_loads: DomRefCell<FxHashSet<Dom<Document>>>,

    /// <https://html.spec.whatwg.org/multipage/#custom-element-reactions-stack>
    custom_element_reaction_stack: Rc<CustomElementReactionStack>,

    /// Cross-process access to `Paint`'s API.
    #[no_trace]
    paint_api: CrossProcessPaintApi,

    /// Periodically print out on which events script threads spend their processing time.
    profile_script_events: bool,

    /// Print Progressive Web Metrics to console.
    print_pwm: bool,

    /// Unminify Javascript.
    unminify_js: bool,

    /// Directory with stored unminified scripts
    local_script_source: Option<String>,

    /// Unminify Css.
    unminify_css: bool,

    /// A map from [`UserContentManagerId`] to its [`UserContents`]. This is initialized
    /// with a copy of the map in constellation (via the `InitialScriptState`). After that,
    /// the constellation forwards any mutations to this `ScriptThread` using messages.
    #[no_trace]
    user_contents_for_manager_id:
        RefCell<FxHashMap<UserContentManagerId, ScriptThreadUserContents>>,

    /// Application window's GL Context for Media player
    #[no_trace]
    player_context: WindowGLContext,

    /// A map from pipelines to all owned nodes ever created in this script thread
    #[no_trace]
    pipeline_to_node_ids: DomRefCell<FxHashMap<PipelineId, NodeIdSet>>,

    /// Code is running as a consequence of a user interaction
    is_user_interacting: Rc<Cell<bool>>,

    /// Identity manager for WebGPU resources
    #[no_trace]
    #[cfg(feature = "webgpu")]
    gpu_id_hub: Arc<IdentityHub>,

    /// A factory for making new layouts. This allows layout to depend on script.
    #[no_trace]
    layout_factory: Arc<dyn LayoutFactory>,

    /// The [`TimerId`] of a ScriptThread-scheduled "update the rendering" call, if any.
    /// The ScriptThread schedules calls to "update the rendering," but the renderer can
    /// also do this when animating. Renderer-based calls always take precedence.
    #[no_trace]
    scheduled_update_the_rendering: RefCell<Option<TimerId>>,

    /// Whether an animation tick or ScriptThread-triggered rendering update is pending. This might
    /// either be because the Servo renderer is managing animations and the [`ScriptThread`] has
    /// received a [`ScriptThreadMessage::TickAllAnimations`] message, because the [`ScriptThread`]
    /// itself is managing animations the timer fired triggering a [`ScriptThread`]-based
    /// animation tick, or if there are no animations running and the [`ScriptThread`] has noticed a
    /// change that requires a rendering update.
    needs_rendering_update: Arc<AtomicBool>,

    debugger_global: Dom<DebuggerGlobalScope>,

    debugger_paused: Cell<bool>,

    /// A list of URLs that can access privileged internal APIs.
    #[no_trace]
    privileged_urls: Vec<ServoUrl>,

    /// Whether accessibility is active. If true, each Layout will maintain an accessibility tree
    /// and send accessibility updates to the embedder.
    accessibility_active: Cell<bool>,
    devtools_state: DevtoolsState,
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

#[expect(unsafe_code)]
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
        image_cache_factory: Arc<dyn ImageCacheFactory>,
        background_hang_monitor_register: Box<dyn BackgroundHangMonitorRegister>,
    ) -> JoinHandle<()> {
        // Setup pipeline-namespace-installing for all threads in this process.
        // Idempotent in single-process mode.
        PipelineNamespace::set_installer_sender(state.namespace_request_sender.clone());

        let script_thread_id = state.id;
        thread::Builder::new()
            .name(format!("Script#{script_thread_id}"))
            .spawn(move || {
                thread_state::initialize(ThreadState::SCRIPT | ThreadState::LAYOUT);
                PipelineNamespace::install(state.pipeline_namespace_id);
                ScriptEventLoopId::install(state.id);
                let memory_profiler_sender = state.memory_profiler_sender.clone();
                let reporter_name = format!("script-reporter-{script_thread_id:?}");
                let (script_thread, mut cx) = ScriptThread::new(
                    state,
                    layout_factory,
                    image_cache_factory,
                    background_hang_monitor_register,
                );
                SCRIPT_THREAD_ROOT.with(|root| {
                    root.set(Some(Rc::as_ptr(&script_thread)));
                });
                let mut failsafe = ScriptMemoryFailsafe::new(&script_thread);

                memory_profiler_sender.run_with_memory_reporting(
                    || script_thread.start(&mut cx),
                    reporter_name,
                    ScriptEventLoopSender::MainThread(script_thread.senders.self_sender.clone()),
                    CommonScriptMsg::CollectReports,
                );

                // This must always be the very last operation performed before the thread completes
                failsafe.neuter();
            })
            .expect("Thread spawning failed")
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

    pub(crate) fn mutation_observers() -> Rc<ScriptMutationObservers> {
        with_script_thread(|script_thread| script_thread.mutation_observers.clone())
    }

    pub(crate) fn microtask_queue() -> Rc<MicrotaskQueue> {
        with_script_thread(|script_thread| script_thread.microtask_queue.clone())
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
        webview_id: WebViewId,
        pipeline_id: PipelineId,
        metadata: Option<Metadata>,
        cx: &mut js::context::JSContext,
    ) -> Option<DomRoot<ServoParser>> {
        with_script_thread(|script_thread| {
            script_thread.handle_page_headers_available(webview_id, pipeline_id, metadata, cx)
        })
    }

    /// Process a single event as if it were the next event
    /// in the queue for this window event-loop.
    /// Returns a boolean indicating whether further events should be processed.
    pub(crate) fn process_event(msg: CommonScriptMsg, cx: &mut js::context::JSContext) -> bool {
        with_script_thread(|script_thread| {
            if !script_thread.can_continue_running_inner() {
                return false;
            }
            script_thread.handle_msg_from_script(MainThreadScriptMsg::Common(msg), cx);
            true
        })
    }

    /// Schedule a [`TimerEventRequest`] on this [`ScriptThread`]'s [`TimerScheduler`].
    pub(crate) fn schedule_timer(&self, request: TimerEventRequest) -> TimerId {
        self.timer_scheduler.borrow_mut().schedule_timer(request)
    }

    /// Cancel a the [`TimerEventRequest`] for the given [`TimerId`] on this
    /// [`ScriptThread`]'s [`TimerScheduler`].
    pub(crate) fn cancel_timer(&self, timer_id: TimerId) {
        self.timer_scheduler.borrow_mut().cancel_timer(timer_id)
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
    fn check_load_origin(source: &LoadOrigin, target: &OriginSnapshot) -> bool {
        match (source, target.immutable()) {
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
            (LoadOrigin::Script(source_origin), _) => source_origin.same_origin_domain(target),
        }
    }

    /// Inform the `ScriptThread` that it should make a call to
    /// [`ScriptThread::update_the_rendering`] as soon as possible, as the rendering
    /// update timer has fired or the renderer has asked us for a new rendering update.
    pub(crate) fn set_needs_rendering_update(&self) {
        self.needs_rendering_update.store(true, Ordering::Relaxed);
    }

    /// Step 13 of <https://html.spec.whatwg.org/multipage/#navigate>
    pub(crate) fn navigate(
        webview_id: WebViewId,
        pipeline_id: PipelineId,
        mut load_data: LoadData,
        history_handling: NavigationHistoryBehavior,
    ) {
        with_script_thread(|script_thread| {
            let is_javascript = load_data.url.scheme() == "javascript";
            // If resource is a request whose url's scheme is "javascript"
            // https://html.spec.whatwg.org/multipage/#navigate-to-a-javascript:-url
            if is_javascript {
                let Some(window) = script_thread.documents.borrow().find_window(pipeline_id) else {
                    return;
                };
                let global = window.as_global_scope();
                let trusted_global = Trusted::new(global);
                let sender = script_thread
                    .senders
                    .pipeline_to_constellation_sender
                    .clone();
                load_data.about_base_url = window.Document().about_base_url();
                let task = task!(navigate_javascript: move |cx| {
                    // Important re security. See https://github.com/servo/servo/issues/23373
                    if trusted_global.root().is::<Window>() {
                        let global = &trusted_global.root();
                        if Self::navigate_to_javascript_url(cx, global, global, &mut load_data, None) {
                            sender
                                .send((webview_id, pipeline_id, ScriptToConstellationMessage::LoadUrl(load_data, history_handling)))
                                .unwrap();
                        }
                    }
                });
                // Step 19 of <https://html.spec.whatwg.org/multipage/#navigate>
                global
                    .task_manager()
                    .dom_manipulation_task_source()
                    .queue(task);
            } else {
                script_thread
                    .senders
                    .pipeline_to_constellation_sender
                    .send((
                        webview_id,
                        pipeline_id,
                        ScriptToConstellationMessage::LoadUrl(load_data, history_handling),
                    ))
                    .expect("Sending a LoadUrl message to the constellation failed");
            }
        });
    }

    /// <https://html.spec.whatwg.org/multipage/#navigate-to-a-javascript:-url>
    pub(crate) fn can_navigate_to_javascript_url(
        initiator_global: &GlobalScope,
        target_global: &GlobalScope,
        load_data: &mut LoadData,
        container: Option<&Element>,
        can_gc: CanGc,
    ) -> bool {
        // Step 3. If initiatorOrigin is not same origin-domain with targetNavigable's active document's origin, then return.
        //
        // Important re security. See https://github.com/servo/servo/issues/23373
        if !Self::check_load_origin(&load_data.load_origin, &target_global.origin().snapshot()) {
            return false;
        }

        // Step 5: If the result of should navigation request of type be blocked by
        // Content Security Policy? given request and cspNavigationType is "Blocked", then return. [CSP]
        if initiator_global
            .get_csp_list()
            .should_navigation_request_be_blocked(initiator_global, load_data, container, can_gc)
        {
            return false;
        }

        true
    }

    pub(crate) fn navigate_to_javascript_url(
        cx: &mut js::context::JSContext,
        initiator_global: &GlobalScope,
        target_global: &GlobalScope,
        load_data: &mut LoadData,
        container: Option<&Element>,
    ) -> bool {
        if !Self::can_navigate_to_javascript_url(
            initiator_global,
            target_global,
            load_data,
            container,
            CanGc::from_cx(cx),
        ) {
            return false;
        }

        // Step 6. Let newDocument be the result of evaluating a javascript: URL given targetNavigable,
        // url, initiatorOrigin, and userInvolvement.
        Self::eval_js_url(cx, target_global, load_data);
        true
    }

    pub(crate) fn get_top_level_for_browsing_context(
        sender_webview_id: WebViewId,
        sender_pipeline_id: PipelineId,
        browsing_context_id: BrowsingContextId,
    ) -> Option<WebViewId> {
        with_script_thread(|script_thread| {
            script_thread.ask_constellation_for_top_level_info(
                sender_webview_id,
                sender_pipeline_id,
                browsing_context_id,
            )
        })
    }

    pub(crate) fn find_document(id: PipelineId) -> Option<DomRoot<Document>> {
        with_script_thread(|script_thread| script_thread.documents.borrow().find_document(id))
    }

    /// Creates a guard that sets user_is_interacting to true and returns the
    /// state of user_is_interacting on drop of the guard.
    /// Notice that you need to use `let _guard = ...` as `let _ = ...` is not enough
    #[must_use]
    pub(crate) fn user_interacting_guard() -> ScriptUserInteractingGuard {
        with_script_thread(|script_thread| {
            ScriptUserInteractingGuard::new(script_thread.is_user_interacting.clone())
        })
    }

    pub(crate) fn is_user_interacting() -> bool {
        with_script_thread(|script_thread| script_thread.is_user_interacting.get())
    }

    pub(crate) fn get_fully_active_document_ids(&self) -> FxHashSet<PipelineId> {
        self.documents
            .borrow()
            .iter()
            .filter_map(|(id, document)| {
                if document.is_fully_active() {
                    Some(id)
                } else {
                    None
                }
            })
            .fold(FxHashSet::default(), |mut set, id| {
                let _ = set.insert(id);
                set
            })
    }

    pub(crate) fn window_proxies() -> Rc<ScriptWindowProxies> {
        with_script_thread(|script_thread| script_thread.window_proxies.clone())
    }

    pub(crate) fn find_window_proxy_by_name(name: &DOMString) -> Option<DomRoot<WindowProxy>> {
        with_script_thread(|script_thread| {
            script_thread.window_proxies.find_window_proxy_by_name(name)
        })
    }

    /// The worklet will use the given `ImageCache`.
    pub(crate) fn worklet_thread_pool(image_cache: Arc<dyn ImageCache>) -> Rc<WorkletThreadPool> {
        with_optional_script_thread(|script_thread| {
            let script_thread = script_thread.unwrap();
            script_thread
                .worklet_thread_pool
                .borrow_mut()
                .get_or_insert_with(|| {
                    let init = WorkletGlobalScopeInit {
                        to_script_thread_sender: script_thread.senders.self_sender.clone(),
                        resource_threads: script_thread.resource_threads.clone(),
                        storage_threads: script_thread.storage_threads.clone(),
                        mem_profiler_chan: script_thread.senders.memory_profiler_sender.clone(),
                        time_profiler_chan: script_thread.senders.time_profiler_sender.clone(),
                        devtools_chan: script_thread.senders.devtools_server_sender.clone(),
                        to_constellation_sender: script_thread
                            .senders
                            .pipeline_to_constellation_sender
                            .clone(),
                        to_embedder_sender: script_thread
                            .senders
                            .pipeline_to_embedder_sender
                            .clone(),
                        image_cache,
                        #[cfg(feature = "webgpu")]
                        gpu_id_hub: script_thread.gpu_id_hub.clone(),
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

    pub(crate) fn custom_element_reaction_stack() -> Rc<CustomElementReactionStack> {
        with_optional_script_thread(|script_thread| {
            script_thread
                .as_ref()
                .unwrap()
                .custom_element_reaction_stack
                .clone()
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

    pub(crate) fn save_node_id(pipeline: PipelineId, node_id: String) {
        with_script_thread(|script_thread| {
            script_thread
                .pipeline_to_node_ids
                .borrow_mut()
                .entry(pipeline)
                .or_default()
                .insert(node_id);
        })
    }

    pub(crate) fn has_node_id(pipeline: PipelineId, node_id: &str) -> bool {
        with_script_thread(|script_thread| {
            script_thread
                .pipeline_to_node_ids
                .borrow()
                .get(&pipeline)
                .is_some_and(|node_ids| node_ids.contains(node_id))
        })
    }

    /// Creates a new script thread.
    pub(crate) fn new(
        state: InitialScriptState,
        layout_factory: Arc<dyn LayoutFactory>,
        image_cache_factory: Arc<dyn ImageCacheFactory>,
        background_hang_monitor_register: Box<dyn BackgroundHangMonitorRegister>,
    ) -> (Rc<ScriptThread>, js::context::JSContext) {
        let (self_sender, self_receiver) = unbounded();
        let mut runtime =
            Runtime::new(Some(ScriptEventLoopSender::MainThread(self_sender.clone())));

        // SAFETY: We ensure that only one JSContext exists in this thread.
        // This is the first one and the only one
        let mut cx = unsafe { runtime.cx() };

        unsafe {
            SetWindowProxyClass(&cx, GetWindowProxyClass());
            JS_AddInterruptCallback(&cx, Some(interrupt_callback));
        }

        let constellation_receiver = state
            .constellation_to_script_receiver
            .route_preserving_errors();

        // Ask the router to proxy IPC messages from the devtools to us.
        let devtools_server_sender = state.devtools_server_sender;
        let (ipc_devtools_sender, ipc_devtools_receiver) = generic_channel::channel().unwrap();
        let devtools_server_receiver = ipc_devtools_receiver.route_preserving_errors();

        let task_queue = TaskQueue::new(self_receiver, self_sender.clone());

        let closing = Arc::new(AtomicBool::new(false));
        let background_hang_monitor_exit_signal = BHMExitSignal {
            closing: closing.clone(),
            js_context: runtime.thread_safe_js_context(),
        };

        let background_hang_monitor = background_hang_monitor_register.register_component(
            // TODO: We shouldn't rely on this PipelineId as a ScriptThread can have multiple
            // Pipelines and any of them might disappear at any time.
            MonitoredComponentId(state.id, MonitoredComponentType::Script),
            Duration::from_millis(1000),
            Duration::from_millis(5000),
            Box::new(background_hang_monitor_exit_signal),
        );

        let (image_cache_sender, image_cache_receiver) = unbounded();

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
            constellation_sender: state.constellation_to_script_sender,
            pipeline_to_constellation_sender: state.script_to_constellation_sender,
            pipeline_to_embedder_sender: state.script_to_embedder_sender.clone(),
            image_cache_sender,
            time_profiler_sender: state.time_profiler_sender,
            memory_profiler_sender: state.memory_profiler_sender,
            devtools_server_sender,
            devtools_client_to_script_thread_sender: ipc_devtools_sender,
        };

        let microtask_queue = runtime.microtask_queue.clone();
        #[cfg(feature = "webgpu")]
        let gpu_id_hub = Arc::new(IdentityHub::default());

        let debugger_pipeline_id = PipelineId::new();
        let script_to_constellation_chan = ScriptToConstellationChan {
            sender: senders.pipeline_to_constellation_sender.clone(),
            // This channel is not expected to be used, so the `WebViewId` that we set here
            // does not matter.
            // TODO: Look at ways of removing the channel entirely for debugger globals.
            webview_id: TEST_WEBVIEW_ID,
            pipeline_id: debugger_pipeline_id,
        };
        let debugger_global = DebuggerGlobalScope::new(
            PipelineId::new(),
            senders.devtools_server_sender.clone(),
            senders.devtools_client_to_script_thread_sender.clone(),
            senders.memory_profiler_sender.clone(),
            senders.time_profiler_sender.clone(),
            script_to_constellation_chan,
            senders.pipeline_to_embedder_sender.clone(),
            state.resource_threads.clone(),
            state.storage_threads.clone(),
            #[cfg(feature = "webgpu")]
            gpu_id_hub.clone(),
            &mut cx,
        );

        debugger_global.execute(&mut cx);

        let user_contents_for_manager_id =
            FxHashMap::from_iter(state.user_contents_for_manager_id.into_iter().map(
                |(user_content_manager_id, user_contents)| {
                    (user_content_manager_id, user_contents.into())
                },
            ));

        (
            Rc::new_cyclic(|weak_script_thread| {
                runtime.set_script_thread(weak_script_thread.clone());
                Self {
                    documents: DomRefCell::new(DocumentCollection::default()),
                    last_render_opportunity_time: Default::default(),
                    window_proxies: Default::default(),
                    incomplete_loads: DomRefCell::new(vec![]),
                    incomplete_parser_contexts: IncompleteParserContexts(RefCell::new(vec![])),
                    senders,
                    receivers,
                    image_cache_factory,
                    resource_threads: state.resource_threads,
                    storage_threads: state.storage_threads,
                    task_queue,
                    background_hang_monitor,
                    closing,
                    timer_scheduler: Default::default(),
                    microtask_queue,
                    js_runtime: Rc::new(runtime),
                    closed_pipelines: DomRefCell::new(FxHashSet::default()),
                    mutation_observers: Default::default(),
                    system_font_service: Arc::new(state.system_font_service.to_proxy()),
                    webgl_chan: state.webgl_chan,
                    #[cfg(feature = "webxr")]
                    webxr_registry: state.webxr_registry,
                    worklet_thread_pool: Default::default(),
                    docs_with_no_blocking_loads: Default::default(),
                    custom_element_reaction_stack: Rc::new(CustomElementReactionStack::new()),
                    paint_api: state.cross_process_paint_api,
                    profile_script_events: opts.debug.profile_script_events,
                    print_pwm: opts.print_pwm,
                    unminify_js: opts.unminify_js,
                    local_script_source: opts.local_script_source.clone(),
                    unminify_css: opts.unminify_css,
                    user_contents_for_manager_id: RefCell::new(user_contents_for_manager_id),
                    player_context: state.player_context,
                    pipeline_to_node_ids: Default::default(),
                    is_user_interacting: Rc::new(Cell::new(false)),
                    #[cfg(feature = "webgpu")]
                    gpu_id_hub,
                    layout_factory,
                    scheduled_update_the_rendering: Default::default(),
                    needs_rendering_update: Arc::new(AtomicBool::new(false)),
                    debugger_global: debugger_global.as_traced(),
                    debugger_paused: Cell::new(false),
                    privileged_urls: state.privileged_urls,
                    this: weak_script_thread.clone(),
                    accessibility_active: Cell::new(state.accessibility_active),
                    devtools_state: Default::default(),
                }
            }),
            cx,
        )
    }

    #[expect(unsafe_code)]
    pub(crate) fn get_cx(&self) -> JSContext {
        unsafe { JSContext::from_ptr(js::rust::Runtime::get().unwrap().as_ptr()) }
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
    pub(crate) fn start(&self, cx: &mut js::context::JSContext) {
        debug!("Starting script thread.");
        while self.handle_msgs(cx) {
            // Go on...
            debug!("Running script thread.");
        }
        debug!("Stopped script thread.");
    }

    /// Process input events as part of a "update the rendering task".
    fn process_pending_input_events(&self, pipeline_id: PipelineId, can_gc: CanGc) {
        let Some(document) = self.documents.borrow().find_document(pipeline_id) else {
            warn!("Processing pending input events for closed pipeline {pipeline_id}.");
            return;
        };
        // Do not handle events if the BC has been, or is being, discarded
        if document.window().Closed() {
            warn!("Input event sent to a pipeline with a closed window {pipeline_id}.");
            return;
        }

        let _guard = ScriptUserInteractingGuard::new(self.is_user_interacting.clone());
        document.event_handler().handle_pending_input_events(can_gc);
    }

    fn cancel_scheduled_update_the_rendering(&self) {
        if let Some(timer_id) = self.scheduled_update_the_rendering.borrow_mut().take() {
            self.timer_scheduler.borrow_mut().cancel_timer(timer_id);
        }
    }

    fn schedule_update_the_rendering_timer_if_necessary(&self, delay: Duration) {
        if self.scheduled_update_the_rendering.borrow().is_some() {
            return;
        }

        debug!("Scheduling ScriptThread animation frame.");
        let trigger_script_thread_animation = self.needs_rendering_update.clone();
        let timer_id = self.schedule_timer(TimerEventRequest {
            callback: Box::new(move || {
                trigger_script_thread_animation.store(true, Ordering::Relaxed);
            }),
            duration: delay,
        });

        *self.scheduled_update_the_rendering.borrow_mut() = Some(timer_id);
    }

    /// <https://html.spec.whatwg.org/multipage/#update-the-rendering>
    ///
    /// Attempt to update the rendering and then do a microtask checkpoint if rendering was
    /// actually updated.
    ///
    /// Returns true if any reflows produced a new display list.
    pub(crate) fn update_the_rendering(&self, cx: &mut js::context::JSContext) -> bool {
        self.last_render_opportunity_time.set(Some(Instant::now()));
        self.cancel_scheduled_update_the_rendering();
        self.needs_rendering_update.store(false, Ordering::Relaxed);

        if !self.can_continue_running_inner() {
            return false;
        }

        // TODO: The specification says to filter out non-renderable documents,
        // as well as those for which a rendering update would be unnecessary,
        // but this isn't happening here.

        // TODO(#31242): the filtering of docs is extended to not exclude the ones that
        // has pending initial observation targets
        // https://w3c.github.io/IntersectionObserver/#pending-initial-observation

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
        let mut painters_generating_frames = FxHashSet::default();
        for pipeline_id in documents_in_order.iter() {
            let document = self
                .documents
                .borrow()
                .find_document(*pipeline_id)
                .expect("Got pipeline for Document not managed by this ScriptThread.");

            if !document.is_fully_active() {
                continue;
            }

            if document.waiting_on_canvas_image_updates() {
                continue;
            }

            // Clear this as early as possible so that any callbacks that
            // trigger new reasons for updating the rendering don't get lost.
            document.clear_rendering_update_reasons();

            // TODO(#31581): The steps in the "Revealing the document" section need to be implemented
            // `process_pending_input_events` handles the focusing steps as well as other events
            // from `Paint`.

            // TODO: Should this be broken and to match the specification more closely? For instance see
            // https://html.spec.whatwg.org/multipage/#flush-autofocus-candidates.
            self.process_pending_input_events(*pipeline_id, CanGc::from_cx(cx));

            // > 8. For each doc of docs, run the resize steps for doc. [CSSOMVIEW]
            let resized = document.window().run_the_resize_steps(CanGc::from_cx(cx));

            // > 9. For each doc of docs, run the scroll steps for doc.
            document.run_the_scroll_steps(CanGc::from_cx(cx));

            // Media queries is only relevant when there are resizing.
            if resized {
                // 10. For each doc of docs, evaluate media queries and report changes for doc.
                document
                    .window()
                    .evaluate_media_queries_and_report_changes(CanGc::from_cx(cx));

                // https://html.spec.whatwg.org/multipage/#img-environment-changes
                // As per the spec, this can be run at any time.
                document.react_to_environment_changes()
            }

            // > 11. For each doc of docs, update animations and send events for doc, passing
            // > in relative high resolution time given frameTimestamp and doc's relevant
            // > global object as the timestamp [WEBANIMATIONS]
            document.update_animations_and_send_events(cx);

            // TODO(#31866): Implement "run the fullscreen steps" from
            // https://fullscreen.spec.whatwg.org/multipage/#run-the-fullscreen-steps.

            // TODO(#31868): Implement the "context lost steps" from
            // https://html.spec.whatwg.org/multipage/#context-lost-steps.

            // > 14. For each doc of docs, run the animation frame callbacks for doc, passing
            // > in the relative high resolution time given frameTimestamp and doc's
            // > relevant global object as the timestamp.
            document.run_the_animation_frame_callbacks(CanGc::from_cx(cx));

            // Run the resize observer steps.
            let _realm = enter_realm(&*document);
            let mut depth = Default::default();
            while document.gather_active_resize_observations_at_depth(&depth) {
                // Note: this will reflow the doc.
                depth = document.broadcast_active_resize_observations(CanGc::from_cx(cx));
            }

            if document.has_skipped_resize_observations() {
                document.deliver_resize_loop_error_notification(CanGc::from_cx(cx));
                // Ensure that another turn of the event loop occurs to process
                // the skipped observations.
                document.add_rendering_update_reason(
                    RenderingUpdateReason::ResizeObserverStartedObservingTarget,
                );
            }

            // TODO(#31870): Implement step 17: if the focused area of doc is not a focusable area,
            // then run the focusing steps for document's viewport.

            // TODO: Perform pending transition operations from
            // https://drafts.csswg.org/css-view-transitions/#perform-pending-transition-operations.

            // > 19. For each doc of docs, run the update intersection observations steps for doc,
            // > passing in the relative high resolution time given now and
            // > doc's relevant global object as the timestamp. [INTERSECTIONOBSERVER]
            // TODO(stevennovaryo): The time attribute should be relative to the time origin of the global object
            document
                .update_intersection_observer_steps(CrossProcessInstant::now(), CanGc::from_cx(cx));

            // TODO: Mark paint timing from https://w3c.github.io/paint-timing.

            // > Step 22: For each doc of docs, update the rendering or user interface of
            // > doc and its node navigable to reflect the current state.
            if document.update_the_rendering().0.needs_frame() {
                painters_generating_frames.insert(document.webview_id().into());
            }

            // TODO: Process top layer removals according to
            // https://drafts.csswg.org/css-position-4/#process-top-layer-removals.
        }

        let should_generate_frame = !painters_generating_frames.is_empty();
        if should_generate_frame {
            self.paint_api
                .generate_frame(painters_generating_frames.into_iter().collect());
        }

        // Perform a microtask checkpoint as the specifications says that *update the rendering*
        // should be run in a task and a microtask checkpoint is always done when running tasks.
        self.perform_a_microtask_checkpoint(cx);
        should_generate_frame
    }

    /// Schedule a rendering update ("update the rendering"), if necessary. This
    /// can be necessary for a couple reasons. For instance, when the DOM
    /// changes a scheduled rendering update becomes necessary if one isn't
    /// scheduled already. Another example is if rAFs are running but no display
    /// lists are being produced. In that case the [`ScriptThread`] is
    /// responsible for scheduling animation ticks.
    fn maybe_schedule_rendering_opportunity_after_ipc_message(
        &self,
        built_any_display_lists: bool,
    ) {
        let needs_rendering_update = self
            .documents
            .borrow()
            .iter()
            .any(|(_, document)| document.needs_rendering_update());
        let running_animations = self.documents.borrow().iter().any(|(_, document)| {
            document.is_fully_active() &&
                !document.window().throttled() &&
                (document.animations().running_animation_count() != 0 ||
                    document.has_active_request_animation_frame_callbacks())
        });

        // If we are not running animations and no rendering update is
        // necessary, just exit early and schedule the next rendering update
        // when it becomes necessary.
        if !needs_rendering_update && !running_animations {
            return;
        }

        // If animations are running and a reflow in this event loop iteration
        // produced a display list, rely on the renderer to inform us of the
        // next animation tick / rendering opportunity.
        if running_animations && built_any_display_lists {
            return;
        }

        // There are two possibilities: rendering needs to be updated or we are
        // scheduling a new animation tick because animations are running, but
        // not changing the DOM. In the later case we can wait a bit longer
        // until the next "update the rendering" call as it's more efficient to
        // slow down rAFs that don't change the DOM.
        //
        // TODO: Should either of these delays be reduced to also reduce update latency?
        let animation_delay = if running_animations && !needs_rendering_update {
            // 30 milliseconds (33 FPS) is used here as the rendering isn't changing
            // so it isn't a problem to slow down rAF callback calls. In addition, this allows
            // renderer-based ticks to arrive first.
            Duration::from_millis(30)
        } else {
            // 20 milliseconds (50 FPS) is used here in order to allow any renderer-based
            // animation ticks to arrive first.
            Duration::from_millis(20)
        };

        let time_since_last_rendering_opportunity = self
            .last_render_opportunity_time
            .get()
            .map(|last_render_opportunity_time| Instant::now() - last_render_opportunity_time)
            .unwrap_or(Duration::MAX)
            .min(animation_delay);
        self.schedule_update_the_rendering_timer_if_necessary(
            animation_delay - time_since_last_rendering_opportunity,
        );
    }

    /// Fulfill the possibly-pending pending `document.fonts.ready` promise if
    /// all web fonts have loaded.
    fn maybe_fulfill_font_ready_promises(&self, cx: &mut js::context::JSContext) {
        let mut sent_message = false;
        for (_, document) in self.documents.borrow().iter() {
            sent_message =
                document.maybe_fulfill_font_ready_promise(CanGc::from_cx(cx)) || sent_message;
        }

        if sent_message {
            self.perform_a_microtask_checkpoint(cx);
        }
    }

    /// If any `Pipeline`s are waiting to become ready for the purpose of taking a
    /// screenshot, check to see if the `Pipeline` is now ready and send a message to the
    /// Constellation, if so.
    fn maybe_resolve_pending_screenshot_readiness_requests(&self, can_gc: CanGc) {
        for (_, document) in self.documents.borrow().iter() {
            document
                .window()
                .maybe_resolve_pending_screenshot_readiness_requests(can_gc);
        }
    }

    /// Handle incoming messages from other tasks and the task queue.
    fn handle_msgs(&self, cx: &mut js::context::JSContext) -> bool {
        // Proritize rendering tasks and others, and gather all other events as `sequential`.
        let mut sequential = vec![];

        // Notify the background-hang-monitor we are waiting for an event.
        self.background_hang_monitor.notify_wait();

        // Receive at least one message so we don't spinloop.
        debug!("Waiting for event.");
        let fully_active = self.get_fully_active_document_ids();
        let mut event = self.receivers.recv(
            &self.task_queue,
            &self.timer_scheduler.borrow(),
            &fully_active,
        );

        loop {
            debug!("Handling event: {event:?}");

            // Dispatch any completed timers, so that their tasks can be run below.
            self.timer_scheduler
                .borrow_mut()
                .dispatch_completed_timers();

            // https://html.spec.whatwg.org/multipage/#event-loop-processing-model step 7
            match event {
                // This has to be handled before the ResizeMsg below,
                // otherwise the page may not have been added to the
                // child list yet, causing the find() to fail.
                MixedMessage::FromConstellation(ScriptThreadMessage::SpawnPipeline(
                    new_pipeline_info,
                )) => {
                    self.spawn_pipeline(new_pipeline_info);
                },
                MixedMessage::FromScript(MainThreadScriptMsg::Inactive) => {
                    // An event came-in from a document that is not fully-active, it has been stored by the task-queue.
                    // Continue without adding it to "sequential".
                },
                MixedMessage::FromConstellation(ScriptThreadMessage::ExitFullScreen(id)) => self
                    .profile_event(ScriptThreadEventCategory::ExitFullscreen, Some(id), || {
                        self.handle_exit_fullscreen(id, cx);
                    }),
                _ => {
                    sequential.push(event);
                },
            }

            // If any of our input sources has an event pending, we'll perform another
            // iteration and check for events. If there are no events pending, we'll move
            // on and execute the sequential events.
            match self.receivers.try_recv(&self.task_queue, &fully_active) {
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
                        self.handle_exit_script_thread_msg(cx);
                        return false;
                    },
                    MixedMessage::FromConstellation(ScriptThreadMessage::ExitPipeline(
                        webview_id,
                        pipeline_id,
                        discard_browsing_context,
                    )) => {
                        self.handle_exit_pipeline_msg(
                            webview_id,
                            pipeline_id,
                            discard_browsing_context,
                            cx,
                        );
                    },
                    _ => {},
                }
                continue;
            }

            let exiting = self.profile_event(category, pipeline_id, || {
                match msg {
                    MixedMessage::FromConstellation(ScriptThreadMessage::ExitScriptThread) => {
                        self.handle_exit_script_thread_msg(cx);
                        return true;
                    },
                    MixedMessage::FromConstellation(inner_msg) => {
                        self.handle_msg_from_constellation(inner_msg, cx)
                    },
                    MixedMessage::FromScript(inner_msg) => {
                        self.handle_msg_from_script(inner_msg, cx)
                    },
                    MixedMessage::FromDevtools(inner_msg) => {
                        self.handle_msg_from_devtools(inner_msg, cx)
                    },
                    MixedMessage::FromImageCache(inner_msg) => {
                        self.handle_msg_from_image_cache(inner_msg, cx)
                    },
                    #[cfg(feature = "webgpu")]
                    MixedMessage::FromWebGPUServer(inner_msg) => {
                        self.handle_msg_from_webgpu_server(inner_msg, cx)
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
            self.perform_a_microtask_checkpoint(cx);
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
                let _realm = enter_auto_realm(cx, &**document);
                document.maybe_queue_document_completion();
            }
            docs.clear();
        }

        let built_any_display_lists =
            self.needs_rendering_update.load(Ordering::Relaxed) && self.update_the_rendering(cx);

        self.maybe_fulfill_font_ready_promises(cx);
        self.maybe_resolve_pending_screenshot_readiness_requests(CanGc::from_cx(cx));

        // This must happen last to detect if any change above makes a rendering update necessary.
        self.maybe_schedule_rendering_opportunity_after_ipc_message(built_any_display_lists);

        true
    }

    fn categorize_msg(&self, msg: &MixedMessage) -> ScriptThreadEventCategory {
        match *msg {
            MixedMessage::FromConstellation(ref inner_msg) => match *inner_msg {
                ScriptThreadMessage::SendInputEvent(..) => ScriptThreadEventCategory::InputEvent,
                _ => ScriptThreadEventCategory::ConstellationMsg,
            },
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
                ScriptThreadEventCategory::SpawnPipeline => {
                    time_profile!(
                        ProfilerCategory::ScriptSpawnPipeline,
                        None,
                        profiler_chan,
                        f
                    )
                },
                ScriptThreadEventCategory::ConstellationMsg => time_profile!(
                    ProfilerCategory::ScriptConstellationMsg,
                    None,
                    profiler_chan,
                    f
                ),
                ScriptThreadEventCategory::DatabaseAccessEvent => time_profile!(
                    ProfilerCategory::ScriptDatabaseAccessEvent,
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
                ScriptThreadEventCategory::GeolocationEvent => {
                    time_profile!(
                        ProfilerCategory::ScriptGeolocationEvent,
                        None,
                        profiler_chan,
                        f
                    )
                },
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

    fn handle_msg_from_constellation(
        &self,
        msg: ScriptThreadMessage,
        cx: &mut js::context::JSContext,
    ) {
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
                cx,
            ),
            ScriptThreadMessage::UnloadDocument(pipeline_id) => {
                self.handle_unload_document(pipeline_id, CanGc::from_cx(cx))
            },
            ScriptThreadMessage::ResizeInactive(id, new_size) => {
                self.handle_resize_inactive_msg(id, new_size)
            },
            ScriptThreadMessage::ThemeChange(_, theme) => {
                self.handle_theme_change_msg(theme);
            },
            ScriptThreadMessage::GetTitle(pipeline_id) => self.handle_get_title_msg(pipeline_id),
            ScriptThreadMessage::SetDocumentActivity(pipeline_id, activity) => {
                self.handle_set_document_activity_msg(cx, pipeline_id, activity)
            },
            ScriptThreadMessage::SetThrottled(webview_id, pipeline_id, throttled) => {
                self.handle_set_throttled_msg(webview_id, pipeline_id, throttled)
            },
            ScriptThreadMessage::SetThrottledInContainingIframe(
                _,
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
                source_webview,
                source_with_ancestry,
                target_origin: origin,
                source_origin,
                data,
            } => self.handle_post_message_msg(
                cx,
                target_pipeline_id,
                source_webview,
                source_with_ancestry,
                origin,
                source_origin,
                *data,
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
                cx,
            ),
            ScriptThreadMessage::UpdateHistoryState(pipeline_id, history_state_id, url) => self
                .handle_update_history_state_msg(
                    pipeline_id,
                    history_state_id,
                    url,
                    CanGc::from_cx(cx),
                ),
            ScriptThreadMessage::RemoveHistoryStates(pipeline_id, history_states) => {
                self.handle_remove_history_states(pipeline_id, history_states)
            },
            ScriptThreadMessage::FocusIFrame(parent_pipeline_id, frame_id, sequence) => self
                .handle_focus_iframe_msg(
                    parent_pipeline_id,
                    frame_id,
                    sequence,
                    CanGc::from_cx(cx),
                ),
            ScriptThreadMessage::FocusDocument(pipeline_id, sequence) => {
                self.handle_focus_document_msg(pipeline_id, sequence, CanGc::from_cx(cx))
            },
            ScriptThreadMessage::Unfocus(pipeline_id, sequence) => {
                self.handle_unfocus_msg(pipeline_id, sequence, CanGc::from_cx(cx))
            },
            ScriptThreadMessage::WebDriverScriptCommand(pipeline_id, msg) => {
                self.handle_webdriver_msg(pipeline_id, msg, cx)
            },
            ScriptThreadMessage::WebFontLoaded(pipeline_id, success) => {
                self.handle_web_font_loaded(pipeline_id, success)
            },
            ScriptThreadMessage::DispatchIFrameLoadEvent {
                target: browsing_context_id,
                parent: parent_id,
                child: child_id,
            } => self.handle_iframe_load_event(parent_id, browsing_context_id, child_id, cx),
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
            ScriptThreadMessage::Reload(pipeline_id) => {
                self.handle_reload(pipeline_id, CanGc::from_cx(cx))
            },
            ScriptThreadMessage::Resize(id, size, size_type) => {
                self.handle_resize_message(id, size, size_type);
            },
            ScriptThreadMessage::ExitPipeline(
                webview_id,
                pipeline_id,
                discard_browsing_context,
            ) => {
                self.handle_exit_pipeline_msg(webview_id, pipeline_id, discard_browsing_context, cx)
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
                CanGc::from_cx(cx),
            ),
            ScriptThreadMessage::MediaSessionAction(pipeline_id, action) => {
                self.handle_media_session_action(cx, pipeline_id, action)
            },
            ScriptThreadMessage::SendInputEvent(webview_id, id, event) => {
                self.handle_input_event(webview_id, id, event)
            },
            #[cfg(feature = "webgpu")]
            ScriptThreadMessage::SetWebGPUPort(port) => {
                *self.receivers.webgpu_receiver.borrow_mut() = port.route_preserving_errors();
            },
            ScriptThreadMessage::TickAllAnimations(_webviews) => {
                self.set_needs_rendering_update();
            },
            ScriptThreadMessage::NoLongerWaitingOnAsychronousImageUpdates(pipeline_id) => {
                if let Some(document) = self.documents.borrow().find_document(pipeline_id) {
                    document.handle_no_longer_waiting_on_asynchronous_image_updates();
                }
            },
            msg @ ScriptThreadMessage::SpawnPipeline(..) |
            msg @ ScriptThreadMessage::ExitFullScreen(..) |
            msg @ ScriptThreadMessage::ExitScriptThread => {
                panic!("should have handled {:?} already", msg)
            },
            ScriptThreadMessage::SetScrollStates(pipeline_id, scroll_states) => {
                self.handle_set_scroll_states(pipeline_id, scroll_states)
            },
            ScriptThreadMessage::EvaluateJavaScript(
                webview_id,
                pipeline_id,
                evaluation_id,
                script,
            ) => {
                self.handle_evaluate_javascript(webview_id, pipeline_id, evaluation_id, script, cx);
            },
            ScriptThreadMessage::SendImageKeysBatch(pipeline_id, image_keys) => {
                if let Some(window) = self.documents.borrow().find_window(pipeline_id) {
                    window
                        .image_cache()
                        .fill_key_cache_with_batch_of_keys(image_keys);
                } else {
                    warn!(
                        "Could not find window corresponding to an image cache to send image keys to pipeline {:?}",
                        pipeline_id
                    );
                }
            },
            ScriptThreadMessage::RefreshCursor(pipeline_id) => {
                self.handle_refresh_cursor(pipeline_id);
            },
            ScriptThreadMessage::PreferencesUpdated(updates) => {
                let mut current_preferences = prefs::get().clone();
                for (name, value) in updates {
                    current_preferences.set_value(&name, value);
                }
                prefs::set(current_preferences);
            },
            ScriptThreadMessage::ForwardKeyboardScroll(pipeline_id, scroll) => {
                if let Some(document) = self.documents.borrow().find_document(pipeline_id) {
                    document.event_handler().do_keyboard_scroll(scroll);
                }
            },
            ScriptThreadMessage::RequestScreenshotReadiness(webview_id, pipeline_id) => {
                self.handle_request_screenshot_readiness(
                    webview_id,
                    pipeline_id,
                    CanGc::from_cx(cx),
                );
            },
            ScriptThreadMessage::EmbedderControlResponse(id, response) => {
                self.handle_embedder_control_response(id, response, CanGc::from_cx(cx));
            },
            ScriptThreadMessage::SetUserContents(user_content_manager_id, user_contents) => {
                self.user_contents_for_manager_id
                    .borrow_mut()
                    .insert(user_content_manager_id, user_contents.into());
            },
            ScriptThreadMessage::DestroyUserContentManager(user_content_manager_id) => {
                self.user_contents_for_manager_id
                    .borrow_mut()
                    .remove(&user_content_manager_id);
            },
            ScriptThreadMessage::AccessibilityTreeUpdate(webview_id, tree_update) => {
                let _ = self.senders.pipeline_to_embedder_sender.send(
                    EmbedderMsg::AccessibilityTreeUpdate(webview_id, tree_update),
                );
            },
            ScriptThreadMessage::UpdatePinchZoomInfos(id, pinch_zoom_infos) => {
                self.handle_update_pinch_zoom_infos(id, pinch_zoom_infos, CanGc::from_cx(cx));
            },
            ScriptThreadMessage::SetAccessibilityActive(active) => {
                self.set_accessibility_active(active);
            },
        }
    }

    fn handle_set_scroll_states(&self, pipeline_id: PipelineId, scroll_states: ScrollStateUpdate) {
        let Some(window) = self.documents.borrow().find_window(pipeline_id) else {
            warn!("Received scroll states for closed pipeline {pipeline_id}");
            return;
        };

        self.profile_event(
            ScriptThreadEventCategory::SetScrollState,
            Some(pipeline_id),
            || {
                window
                    .layout_mut()
                    .set_scroll_offsets_from_renderer(&scroll_states.offsets);
            },
        );

        window
            .Document()
            .event_handler()
            .handle_embedder_scroll_event(scroll_states.scrolled_node);
    }

    #[cfg(feature = "webgpu")]
    fn handle_msg_from_webgpu_server(&self, msg: WebGPUMsg, cx: &mut js::context::JSContext) {
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
                global.gpu_device_lost(device, reason, msg);
            },
            WebGPUMsg::UncapturedError {
                device,
                pipeline_id,
                error,
            } => {
                let global = self.documents.borrow().find_global(pipeline_id).unwrap();
                let _ac = enter_auto_realm(cx, &*global);
                global.handle_uncaptured_gpu_error(device, error);
            },
            _ => {},
        }
    }

    fn handle_msg_from_script(&self, msg: MainThreadScriptMsg, cx: &mut js::context::JSContext) {
        match msg {
            MainThreadScriptMsg::Common(CommonScriptMsg::Task(_, task, pipeline_id, _)) => {
                let _realm = pipeline_id.and_then(|id| {
                    let global = self.documents.borrow().find_global(id);
                    global.map(|global| enter_realm(&*global))
                });
                task.run_box(cx)
            },
            MainThreadScriptMsg::Common(CommonScriptMsg::CollectReports(chan)) => {
                self.collect_reports(chan)
            },
            MainThreadScriptMsg::Common(CommonScriptMsg::ReportCspViolations(
                pipeline_id,
                violations,
            )) => {
                if let Some(global) = self.documents.borrow().find_global(pipeline_id) {
                    global.report_csp_violations(violations, None, None);
                }
            },
            MainThreadScriptMsg::NavigationResponse {
                pipeline_id,
                message,
            } => {
                self.handle_navigation_response(cx, pipeline_id, *message);
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
            MainThreadScriptMsg::ForwardEmbedderControlResponseFromFileManager(
                control_id,
                response,
            ) => {
                self.handle_embedder_control_response(control_id, response, CanGc::from_cx(cx));
            },
        }
    }

    fn handle_msg_from_devtools(
        &self,
        msg: DevtoolScriptControlMsg,
        cx: &mut js::context::JSContext,
    ) {
        let documents = self.documents.borrow();
        match msg {
            DevtoolScriptControlMsg::EvaluateJS(id, s, reply) => match documents.find_window(id) {
                Some(window) => {
                    let global = window.as_global_scope();
                    run_a_script::<DomTypeHolder, _>(global, || {
                        devtools::handle_evaluate_js(global, s, reply, cx)
                    });
                },
                None => warn!("Message sent to closed pipeline {}.", id),
            },
            DevtoolScriptControlMsg::GetEventListenerInfo(id, node, reply) => {
                devtools::handle_get_event_listener_info(&self.devtools_state, id, &node, reply)
            },
            DevtoolScriptControlMsg::GetRootNode(id, reply) => devtools::handle_get_root_node(
                &self.devtools_state,
                &documents,
                id,
                reply,
                CanGc::from_cx(cx),
            ),
            DevtoolScriptControlMsg::GetDocumentElement(id, reply) => {
                devtools::handle_get_document_element(
                    &self.devtools_state,
                    &documents,
                    id,
                    reply,
                    CanGc::from_cx(cx),
                )
            },
            DevtoolScriptControlMsg::GetChildren(id, node_id, reply) => {
                devtools::handle_get_children(
                    &self.devtools_state,
                    id,
                    &node_id,
                    reply,
                    CanGc::from_cx(cx),
                )
            },
            DevtoolScriptControlMsg::GetAttributeStyle(id, node_id, reply) => {
                devtools::handle_get_attribute_style(
                    &self.devtools_state,
                    id,
                    &node_id,
                    reply,
                    CanGc::from_cx(cx),
                )
            },
            DevtoolScriptControlMsg::GetStylesheetStyle(
                id,
                node_id,
                selector,
                stylesheet,
                reply,
            ) => devtools::handle_get_stylesheet_style(
                &self.devtools_state,
                &documents,
                id,
                &node_id,
                selector,
                stylesheet,
                reply,
                CanGc::from_cx(cx),
            ),
            DevtoolScriptControlMsg::GetSelectors(id, node_id, reply) => {
                devtools::handle_get_selectors(
                    &self.devtools_state,
                    &documents,
                    id,
                    &node_id,
                    reply,
                    CanGc::from_cx(cx),
                )
            },
            DevtoolScriptControlMsg::GetComputedStyle(id, node_id, reply) => {
                devtools::handle_get_computed_style(&self.devtools_state, id, &node_id, reply)
            },
            DevtoolScriptControlMsg::GetLayout(id, node_id, reply) => devtools::handle_get_layout(
                &self.devtools_state,
                id,
                &node_id,
                reply,
                CanGc::from_cx(cx),
            ),
            DevtoolScriptControlMsg::GetXPath(id, node_id, reply) => {
                devtools::handle_get_xpath(&self.devtools_state, id, &node_id, reply)
            },
            DevtoolScriptControlMsg::ModifyAttribute(id, node_id, modifications) => {
                devtools::handle_modify_attribute(
                    &self.devtools_state,
                    &documents,
                    id,
                    &node_id,
                    modifications,
                    CanGc::from_cx(cx),
                )
            },
            DevtoolScriptControlMsg::ModifyRule(id, node_id, modifications) => {
                devtools::handle_modify_rule(
                    &self.devtools_state,
                    &documents,
                    id,
                    &node_id,
                    modifications,
                    CanGc::from_cx(cx),
                )
            },
            DevtoolScriptControlMsg::WantsLiveNotifications(id, to_send) => {
                match documents.find_window(id) {
                    Some(window) => {
                        window
                            .upcast::<GlobalScope>()
                            .set_devtools_wants_updates(to_send);
                    },
                    None => warn!("Message sent to closed pipeline {}.", id),
                }
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
            DevtoolScriptControlMsg::Reload(id) => self.handle_reload(id, CanGc::from_cx(cx)),
            DevtoolScriptControlMsg::GetCssDatabase(reply) => {
                devtools::handle_get_css_database(reply)
            },
            DevtoolScriptControlMsg::SimulateColorScheme(id, theme) => {
                match documents.find_window(id) {
                    Some(window) => {
                        window.set_theme(theme);
                    },
                    None => warn!("Message sent to closed pipeline {}.", id),
                }
            },
            DevtoolScriptControlMsg::HighlightDomNode(id, node_id) => {
                devtools::handle_highlight_dom_node(
                    &self.devtools_state,
                    &documents,
                    id,
                    node_id.as_deref(),
                )
            },
            DevtoolScriptControlMsg::Eval(code, id, reply) => {
                self.debugger_global
                    .fire_eval(CanGc::from_cx(cx), code.into(), id, None, reply);
            },
            DevtoolScriptControlMsg::GetPossibleBreakpoints(spidermonkey_id, result_sender) => {
                self.debugger_global.fire_get_possible_breakpoints(
                    CanGc::from_cx(cx),
                    spidermonkey_id,
                    result_sender,
                );
            },
            DevtoolScriptControlMsg::SetBreakpoint(spidermonkey_id, script_id, offset) => {
                self.debugger_global.fire_set_breakpoint(
                    CanGc::from_cx(cx),
                    spidermonkey_id,
                    script_id,
                    offset,
                );
            },
            DevtoolScriptControlMsg::ClearBreakpoint(spidermonkey_id, script_id, offset) => {
                self.debugger_global.fire_clear_breakpoint(
                    CanGc::from_cx(cx),
                    spidermonkey_id,
                    script_id,
                    offset,
                );
            },
            DevtoolScriptControlMsg::Interrupt => {
                self.debugger_global.fire_interrupt(CanGc::from_cx(cx));
            },
            DevtoolScriptControlMsg::Resume => {
                self.debugger_paused.set(false);
            },
        }
    }

    /// Enter a nested event loop for debugger pause.
    /// TODO: This should also be called when manual pause is triggered.
    pub(crate) fn enter_debugger_pause_loop(&self) {
        self.debugger_paused.set(true);

        #[allow(unsafe_code)]
        let mut cx = unsafe { js::context::JSContext::from_ptr(js::rust::Runtime::get().unwrap()) };

        while self.debugger_paused.get() {
            match self.receivers.devtools_server_receiver.recv() {
                Ok(Ok(msg)) => self.handle_msg_from_devtools(msg, &mut cx),
                _ => {
                    self.debugger_paused.set(false);
                    break;
                },
            }
        }
    }

    fn handle_msg_from_image_cache(
        &self,
        response: ImageCacheResponseMessage,
        cx: &mut js::context::JSContext,
    ) {
        match response {
            ImageCacheResponseMessage::NotifyPendingImageLoadStatus(pending_image_response) => {
                let window = self
                    .documents
                    .borrow()
                    .find_window(pending_image_response.pipeline_id);
                if let Some(ref window) = window {
                    window.pending_image_notification(pending_image_response, cx);
                }
            },
            ImageCacheResponseMessage::VectorImageRasterizationComplete(response) => {
                let window = self.documents.borrow().find_window(response.pipeline_id);
                if let Some(ref window) = window {
                    window.handle_image_rasterization_complete_notification(response);
                }
            },
        };
    }

    fn handle_webdriver_msg(
        &self,
        pipeline_id: PipelineId,
        msg: WebDriverScriptCommand,
        cx: &mut js::context::JSContext,
    ) {
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
            WebDriverScriptCommand::ElementClear(element_id, reply) => {
                webdriver_handlers::handle_element_clear(
                    &documents,
                    pipeline_id,
                    element_id,
                    reply,
                    CanGc::from_cx(cx),
                )
            },
            WebDriverScriptCommand::FindElementsCSSSelector(selector, reply) => {
                webdriver_handlers::handle_find_elements_css_selector(
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
                    CanGc::from_cx(cx),
                )
            },
            WebDriverScriptCommand::FindElementsXpathSelector(selector, reply) => {
                webdriver_handlers::handle_find_elements_xpath_selector(
                    &documents,
                    pipeline_id,
                    selector,
                    reply,
                    CanGc::from_cx(cx),
                )
            },
            WebDriverScriptCommand::FindElementElementsCSSSelector(selector, element_id, reply) => {
                webdriver_handlers::handle_find_element_elements_css_selector(
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
                    CanGc::from_cx(cx),
                )
            },
            WebDriverScriptCommand::FindElementElementsXPathSelector(
                selector,
                element_id,
                reply,
            ) => webdriver_handlers::handle_find_element_elements_xpath_selector(
                &documents,
                pipeline_id,
                element_id,
                selector,
                reply,
                CanGc::from_cx(cx),
            ),
            WebDriverScriptCommand::FindShadowElementsCSSSelector(
                selector,
                shadow_root_id,
                reply,
            ) => webdriver_handlers::handle_find_shadow_elements_css_selector(
                &documents,
                pipeline_id,
                shadow_root_id,
                selector,
                reply,
            ),
            WebDriverScriptCommand::FindShadowElementsLinkText(
                selector,
                shadow_root_id,
                partial,
                reply,
            ) => webdriver_handlers::handle_find_shadow_elements_link_text(
                &documents,
                pipeline_id,
                shadow_root_id,
                selector,
                partial,
                reply,
            ),
            WebDriverScriptCommand::FindShadowElementsTagName(selector, shadow_root_id, reply) => {
                webdriver_handlers::handle_find_shadow_elements_tag_name(
                    &documents,
                    pipeline_id,
                    shadow_root_id,
                    selector,
                    reply,
                )
            },
            WebDriverScriptCommand::FindShadowElementsXPathSelector(
                selector,
                shadow_root_id,
                reply,
            ) => webdriver_handlers::handle_find_shadow_elements_xpath_selector(
                &documents,
                pipeline_id,
                shadow_root_id,
                selector,
                reply,
                CanGc::from_cx(cx),
            ),
            WebDriverScriptCommand::GetElementShadowRoot(element_id, reply) => {
                webdriver_handlers::handle_get_element_shadow_root(
                    &documents,
                    pipeline_id,
                    element_id,
                    reply,
                )
            },
            WebDriverScriptCommand::ElementClick(element_id, reply) => {
                webdriver_handlers::handle_element_click(
                    &documents,
                    pipeline_id,
                    element_id,
                    reply,
                    CanGc::from_cx(cx),
                )
            },
            WebDriverScriptCommand::GetKnownElement(element_id, reply) => {
                webdriver_handlers::handle_get_known_element(
                    &documents,
                    pipeline_id,
                    element_id,
                    reply,
                )
            },
            WebDriverScriptCommand::GetKnownWindow(webview_id, reply) => {
                webdriver_handlers::handle_get_known_window(
                    &documents,
                    pipeline_id,
                    webview_id,
                    reply,
                )
            },
            WebDriverScriptCommand::GetKnownShadowRoot(element_id, reply) => {
                webdriver_handlers::handle_get_known_shadow_root(
                    &documents,
                    pipeline_id,
                    element_id,
                    reply,
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
                webdriver_handlers::handle_get_page_source(
                    &documents,
                    pipeline_id,
                    reply,
                    CanGc::from_cx(cx),
                )
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
                    cx,
                )
            },
            WebDriverScriptCommand::GetElementCSS(node_id, name, reply) => {
                webdriver_handlers::handle_get_css(&documents, pipeline_id, node_id, name, reply)
            },
            WebDriverScriptCommand::GetElementRect(node_id, reply) => {
                webdriver_handlers::handle_get_rect(
                    &documents,
                    pipeline_id,
                    node_id,
                    reply,
                    CanGc::from_cx(cx),
                )
            },
            WebDriverScriptCommand::ScrollAndGetBoundingClientRect(node_id, reply) => {
                webdriver_handlers::handle_scroll_and_get_bounding_client_rect(
                    &documents,
                    pipeline_id,
                    node_id,
                    reply,
                    CanGc::from_cx(cx),
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
                    CanGc::from_cx(cx),
                )
            },
            WebDriverScriptCommand::GetParentFrameId(reply) => {
                webdriver_handlers::handle_get_parent_frame_id(&documents, pipeline_id, reply)
            },
            WebDriverScriptCommand::GetBrowsingContextId(webdriver_frame_id, reply) => {
                webdriver_handlers::handle_get_browsing_context_id(
                    &documents,
                    pipeline_id,
                    webdriver_frame_id,
                    reply,
                )
            },
            WebDriverScriptCommand::GetUrl(reply) => webdriver_handlers::handle_get_url(
                &documents,
                pipeline_id,
                reply,
                CanGc::from_cx(cx),
            ),
            WebDriverScriptCommand::IsEnabled(element_id, reply) => {
                webdriver_handlers::handle_is_enabled(&documents, pipeline_id, element_id, reply)
            },
            WebDriverScriptCommand::IsSelected(element_id, reply) => {
                webdriver_handlers::handle_is_selected(&documents, pipeline_id, element_id, reply)
            },
            WebDriverScriptCommand::GetTitle(reply) => {
                webdriver_handlers::handle_get_title(&documents, pipeline_id, reply)
            },
            WebDriverScriptCommand::WillSendKeys(
                element_id,
                text,
                strict_file_interactability,
                reply,
            ) => webdriver_handlers::handle_will_send_keys(
                &documents,
                pipeline_id,
                element_id,
                text,
                strict_file_interactability,
                reply,
                CanGc::from_cx(cx),
            ),
            WebDriverScriptCommand::AddLoadStatusSender(_, response_sender) => {
                webdriver_handlers::handle_add_load_status_sender(
                    &documents,
                    pipeline_id,
                    response_sender,
                )
            },
            WebDriverScriptCommand::RemoveLoadStatusSender(_) => {
                webdriver_handlers::handle_remove_load_status_sender(&documents, pipeline_id)
            },
            // https://github.com/servo/servo/issues/23535
            // The Script messages need different treatment since the JS script might mutate
            // `self.documents`, which would conflict with the immutable borrow of it that
            // occurs for the rest of the messages.
            // We manually drop the immutable borrow first, and quickly
            // end the borrow of documents to avoid runtime error.
            WebDriverScriptCommand::ExecuteScriptWithCallback(script, reply) => {
                let window = documents.find_window(pipeline_id);
                drop(documents);
                webdriver_handlers::handle_execute_async_script(window, script, reply, cx);
            },
            WebDriverScriptCommand::SetProtocolHandlerAutomationMode(mode) => {
                webdriver_handlers::set_protocol_handler_automation_mode(
                    &documents,
                    pipeline_id,
                    mode,
                )
            },
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
            document.window().set_theme(theme);
        }
        let mut loads = self.incomplete_loads.borrow_mut();
        for load in loads.iter_mut() {
            load.theme = theme;
        }
    }

    // exit_fullscreen creates a new JS promise object, so we need to have entered a realm
    fn handle_exit_fullscreen(&self, id: PipelineId, cx: &mut js::context::JSContext) {
        let document = self.documents.borrow().find_document(id);
        if let Some(document) = document {
            let mut realm = enter_auto_realm(cx, &*document);
            document.exit_fullscreen(CanGc::from_cx(&mut realm));
        }
    }

    #[expect(unsafe_code)]
    pub(crate) fn spawn_pipeline(&self, new_pipeline_info: NewPipelineInfo) {
        let mut cx = unsafe { temp_cx() };
        let cx = &mut cx;
        self.profile_event(
            ScriptThreadEventCategory::SpawnPipeline,
            Some(new_pipeline_info.new_pipeline_id),
            || {
                // If this is an about:blank or about:srcdoc load, it must share the
                // creator's origin. This must match the logic in the constellation
                // when creating a new pipeline
                let not_an_about_blank_and_about_srcdoc_load =
                    new_pipeline_info.load_data.url.as_str() != "about:blank" &&
                        new_pipeline_info.load_data.url.as_str() != "about:srcdoc";
                let origin = if not_an_about_blank_and_about_srcdoc_load {
                    MutableOrigin::new(new_pipeline_info.load_data.url.origin())
                } else if let Some(parent) = new_pipeline_info
                    .parent_info
                    .and_then(|pipeline_id| self.documents.borrow().find_document(pipeline_id))
                {
                    parent.origin().clone()
                } else if let Some(creator) = new_pipeline_info
                    .load_data
                    .creator_pipeline_id
                    .and_then(|pipeline_id| self.documents.borrow().find_document(pipeline_id))
                {
                    creator.origin().clone()
                } else {
                    MutableOrigin::new(ImmutableOrigin::new_opaque())
                };

                self.devtools_state
                    .notify_pipeline_created(new_pipeline_info.new_pipeline_id);

                // Kick off the fetch for the new resource.
                self.pre_page_load(cx, InProgressLoad::new(new_pipeline_info, origin));
            },
        );
    }

    fn collect_reports(&self, reports_chan: ReportsChan) {
        let documents = self.documents.borrow();
        let urls = itertools::join(documents.iter().map(|(_, d)| d.url().to_string()), ", ");

        let mut reports = vec![];
        perform_memory_report(|ops| {
            for (_, document) in documents.iter() {
                document
                    .window()
                    .layout()
                    .collect_reports(&mut reports, ops);
            }

            let prefix = format!("url({urls})");
            reports.extend(self.get_cx().get_reports(prefix.clone(), ops));
        });

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

    fn handle_set_throttled_msg(
        &self,
        webview_id: WebViewId,
        pipeline_id: PipelineId,
        throttled: bool,
    ) {
        // Separate message sent since parent script thread could be different (Iframe of different
        // domain)
        self.senders
            .pipeline_to_constellation_sender
            .send((
                webview_id,
                pipeline_id,
                ScriptToConstellationMessage::SetThrottledComplete(throttled),
            ))
            .unwrap();

        let window = self.documents.borrow().find_window(pipeline_id);
        match window {
            Some(window) => {
                window.set_throttled(throttled);
                return;
            },
            None => {
                let mut loads = self.incomplete_loads.borrow_mut();
                if let Some(ref mut load) = loads
                    .iter_mut()
                    .find(|load| load.pipeline_id == pipeline_id)
                {
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
        cx: &mut js::context::JSContext,
        id: PipelineId,
        activity: DocumentActivity,
    ) {
        debug!(
            "Setting activity of {} to be {:?} in {:?}.",
            id,
            activity,
            thread::current().name()
        );
        let document = self.documents.borrow().find_document(id);
        if let Some(document) = document {
            document.set_activity(cx, activity);
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
        sequence: FocusSequenceNumber,
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

        if document.get_focus_sequence() > sequence {
            debug!(
                "Disregarding the FocusIFrame message because the contained sequence number is \
                too old ({:?} < {:?})",
                sequence,
                document.get_focus_sequence()
            );
            return;
        }

        document.request_focus(Some(&iframe_element_root), FocusInitiator::Remote, can_gc);
    }

    fn handle_focus_document_msg(
        &self,
        pipeline_id: PipelineId,
        sequence: FocusSequenceNumber,
        can_gc: CanGc,
    ) {
        if let Some(doc) = self.documents.borrow().find_document(pipeline_id) {
            if doc.get_focus_sequence() > sequence {
                debug!(
                    "Disregarding the FocusDocument message because the contained sequence number is \
                    too old ({:?} < {:?})",
                    sequence,
                    doc.get_focus_sequence()
                );
                return;
            }
            doc.request_focus(None, FocusInitiator::Remote, can_gc);
        } else {
            warn!(
                "Couldn't find document by pipleline_id:{pipeline_id:?} when handle_focus_document_msg."
            );
        }
    }

    fn handle_unfocus_msg(
        &self,
        pipeline_id: PipelineId,
        sequence: FocusSequenceNumber,
        can_gc: CanGc,
    ) {
        if let Some(doc) = self.documents.borrow().find_document(pipeline_id) {
            if doc.get_focus_sequence() > sequence {
                debug!(
                    "Disregarding the Unfocus message because the contained sequence number is \
                    too old ({:?} < {:?})",
                    sequence,
                    doc.get_focus_sequence()
                );
                return;
            }
            doc.handle_container_unfocus(can_gc);
        } else {
            warn!(
                "Couldn't find document by pipleline_id:{pipeline_id:?} when handle_unfocus_msg."
            );
        }
    }

    #[expect(clippy::too_many_arguments)]
    /// <https://html.spec.whatwg.org/multipage/#window-post-message-steps>
    fn handle_post_message_msg(
        &self,
        cx: &mut js::context::JSContext,
        pipeline_id: PipelineId,
        source_webview: WebViewId,
        source_with_ancestry: Vec<BrowsingContextId>,
        origin: Option<ImmutableOrigin>,
        source_origin: ImmutableOrigin,
        data: StructuredSerializedData,
    ) {
        let window = self.documents.borrow().find_window(pipeline_id);
        match window {
            None => warn!("postMessage after target pipeline {} closed.", pipeline_id),
            Some(window) => {
                let mut last = None;
                for browsing_context_id in source_with_ancestry.into_iter().rev() {
                    if let Some(window_proxy) = self.window_proxies.get(browsing_context_id) {
                        last = Some(window_proxy);
                        continue;
                    }
                    let window_proxy = WindowProxy::new_dissimilar_origin(
                        cx,
                        window.upcast::<GlobalScope>(),
                        browsing_context_id,
                        source_webview,
                        last.as_deref(),
                        None,
                        CreatorBrowsingContextInfo::from(last.as_deref(), None),
                    );
                    self.window_proxies
                        .insert(browsing_context_id, window_proxy.clone());
                    last = Some(window_proxy);
                }

                // Step 8.3: Let source be the WindowProxy object corresponding to
                // incumbentSettings's global object (a Window object).
                let source = last.expect("Source with ancestry should contain at least one bc.");

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
        cx: &mut js::context::JSContext,
    ) {
        let frame_element = self
            .documents
            .borrow()
            .find_iframe(parent_pipeline_id, browsing_context_id);
        if let Some(frame_element) = frame_element {
            frame_element.update_pipeline_id(new_pipeline_id, reason, cx);
        }

        if let Some(window) = self.documents.borrow().find_window(new_pipeline_id) {
            // Ensure that the state of any local window proxies accurately reflects
            // the new pipeline.
            let _ = self.window_proxies.local_window_proxy(
                cx,
                &self.senders,
                &self.documents,
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
        let Some(window) = self.documents.borrow().find_window(pipeline_id) else {
            return warn!("update history state after pipeline {pipeline_id} closed.",);
        };
        window
            .History()
            .activate_state(history_state_id, url, can_gc);
    }

    fn handle_remove_history_states(
        &self,
        pipeline_id: PipelineId,
        history_states: Vec<HistoryStateId>,
    ) {
        let Some(window) = self.documents.borrow().find_window(pipeline_id) else {
            return warn!("update history state after pipeline {pipeline_id} closed.",);
        };
        window.History().remove_states(history_states);
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
        webview_id: WebViewId,
        pipeline_id: PipelineId,
        metadata: Option<Metadata>,
        cx: &mut js::context::JSContext,
    ) -> Option<DomRoot<ServoParser>> {
        if self.closed_pipelines.borrow().contains(&pipeline_id) {
            // If the pipeline closed, do not process the headers.
            return None;
        }

        let Some(idx) = self
            .incomplete_loads
            .borrow()
            .iter()
            .position(|load| load.pipeline_id == pipeline_id)
        else {
            unreachable!("Pipeline shouldn't have finished loading.");
        };

        // https://html.spec.whatwg.org/multipage/#process-a-navigate-response
        // 2. If response's status is 204 or 205, then abort these steps.
        let is_204_205 = match metadata {
            Some(ref metadata) => metadata.status.in_range(204..=205),
            _ => false,
        };

        if is_204_205 {
            // If we have an existing window that is being navigated:
            if let Some(window) = self.documents.borrow().find_window(pipeline_id) {
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
                .send((
                    webview_id,
                    pipeline_id,
                    ScriptToConstellationMessage::AbortLoadUrl,
                ))
                .unwrap();
            return None;
        };

        let load = self.incomplete_loads.borrow_mut().remove(idx);
        metadata.map(|meta| self.load(meta, load, cx))
    }

    /// Handles a request for the window title.
    fn handle_get_title_msg(&self, pipeline_id: PipelineId) {
        let Some(document) = self.documents.borrow().find_document(pipeline_id) else {
            return warn!("Message sent to closed pipeline {pipeline_id}.");
        };
        document.send_title_to_embedder();
    }

    /// Handles a request to exit a pipeline and shut down layout.
    fn handle_exit_pipeline_msg(
        &self,
        webview_id: WebViewId,
        pipeline_id: PipelineId,
        discard_bc: DiscardBrowsingContext,
        cx: &mut js::context::JSContext,
    ) {
        debug!("{pipeline_id}: Starting pipeline exit.");

        // Abort the parser, if any,
        // to prevent any further incoming networking messages from being handled.
        let document = self.documents.borrow_mut().remove(pipeline_id);
        if let Some(document) = document {
            // We should never have a pipeline that's still an incomplete load, but also has a Document.
            debug_assert!(
                !self
                    .incomplete_loads
                    .borrow()
                    .iter()
                    .any(|load| load.pipeline_id == pipeline_id)
            );

            if let Some(parser) = document.get_current_parser() {
                parser.abort(cx);
            }

            debug!("{pipeline_id}: Shutting down layout");
            document.window().layout_mut().exit_now();

            // Clear any active animations and unroot all of the associated DOM objects.
            debug!("{pipeline_id}: Clearing animations");
            document.animations().clear();

            // We discard the browsing context after requesting layout shut down,
            // to avoid running layout on detached iframes.
            let window = document.window();
            if discard_bc == DiscardBrowsingContext::Yes {
                window.discard_browsing_context();
            }

            debug!("{pipeline_id}: Clearing JavaScript runtime");
            window.clear_js_runtime();
        }

        // Prevent any further work for this Pipeline.
        self.closed_pipelines.borrow_mut().insert(pipeline_id);

        debug!("{pipeline_id}: Sending PipelineExited message to constellation");
        self.senders
            .pipeline_to_constellation_sender
            .send((
                webview_id,
                pipeline_id,
                ScriptToConstellationMessage::PipelineExited,
            ))
            .ok();

        self.paint_api
            .pipeline_exited(webview_id, pipeline_id, PipelineExitSource::Script);

        self.devtools_state.notify_pipeline_exited(pipeline_id);

        debug!("{pipeline_id}: Finished pipeline exit");
    }

    /// Handles a request to exit the script thread and shut down layout.
    fn handle_exit_script_thread_msg(&self, cx: &mut js::context::JSContext) {
        debug!("Exiting script thread.");

        let mut webview_and_pipeline_ids = Vec::new();
        webview_and_pipeline_ids.extend(
            self.incomplete_loads
                .borrow()
                .iter()
                .next()
                .map(|load| (load.webview_id, load.pipeline_id)),
        );
        webview_and_pipeline_ids.extend(
            self.documents
                .borrow()
                .iter()
                .next()
                .map(|(pipeline_id, document)| (document.webview_id(), pipeline_id)),
        );

        for (webview_id, pipeline_id) in webview_and_pipeline_ids {
            self.handle_exit_pipeline_msg(webview_id, pipeline_id, DiscardBrowsingContext::Yes, cx);
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
            document.add_restyle_reason(RestyleReason::PaintWorkletLoaded);
        }
    }

    /// Notify a window of a storage event
    fn handle_storage_event(
        &self,
        pipeline_id: PipelineId,
        storage_type: WebStorageType,
        url: ServoUrl,
        key: Option<String>,
        old_value: Option<String>,
        new_value: Option<String>,
    ) {
        let Some(window) = self.documents.borrow().find_window(pipeline_id) else {
            return warn!("Storage event sent to closed pipeline {pipeline_id}.");
        };

        let storage = match storage_type {
            WebStorageType::Local => window.LocalStorage(),
            WebStorageType::Session => window.SessionStorage(),
        };

        storage.queue_storage_event(url, key, old_value, new_value);
    }

    /// Notify the containing document of a child iframe that has completed loading.
    fn handle_iframe_load_event(
        &self,
        parent_id: PipelineId,
        browsing_context_id: BrowsingContextId,
        child_id: PipelineId,
        cx: &mut js::context::JSContext,
    ) {
        let iframe = self
            .documents
            .borrow()
            .find_iframe(parent_id, browsing_context_id);
        match iframe {
            Some(iframe) => iframe.iframe_load_event_steps(child_id, cx),
            None => warn!("Message sent to closed pipeline {}.", parent_id),
        }
    }

    fn ask_constellation_for_top_level_info(
        &self,
        sender_webview_id: WebViewId,
        sender_pipeline_id: PipelineId,
        browsing_context_id: BrowsingContextId,
    ) -> Option<WebViewId> {
        let (result_sender, result_receiver) = ipc::channel().unwrap();
        let msg = ScriptToConstellationMessage::GetTopForBrowsingContext(
            browsing_context_id,
            result_sender,
        );
        self.senders
            .pipeline_to_constellation_sender
            .send((sender_webview_id, sender_pipeline_id, msg))
            .expect("Failed to send to constellation.");
        result_receiver
            .recv()
            .expect("Failed to get top-level id from constellation.")
    }

    /// The entry point to document loading. Defines bindings, sets up the window and document
    /// objects, parses HTML and CSS, and kicks off initial layout.
    fn load(
        &self,
        metadata: Metadata,
        incomplete: InProgressLoad,
        cx: &mut js::context::JSContext,
    ) -> DomRoot<ServoParser> {
        let script_to_constellation_chan = ScriptToConstellationChan {
            sender: self.senders.pipeline_to_constellation_sender.clone(),
            webview_id: incomplete.webview_id,
            pipeline_id: incomplete.pipeline_id,
        };

        let final_url = metadata.final_url.clone();
        let _ = script_to_constellation_chan
            .send(ScriptToConstellationMessage::SetFinalUrl(final_url.clone()));

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

        let font_context = Arc::new(FontContext::new(
            self.system_font_service.clone(),
            self.paint_api.clone(),
            self.resource_threads.clone(),
        ));

        let image_cache = self.image_cache_factory.create(
            incomplete.webview_id,
            incomplete.pipeline_id,
            &self.paint_api,
        );

        let (user_contents, user_stylesheets) = incomplete
            .user_content_manager_id
            .and_then(|user_content_manager_id| {
                self.user_contents_for_manager_id
                    .borrow()
                    .get(&user_content_manager_id)
                    .map(|script_thread_user_contents| {
                        (
                            script_thread_user_contents.user_scripts.clone(),
                            script_thread_user_contents.user_stylesheets.clone(),
                        )
                    })
            })
            .unwrap_or_default();

        let layout_config = LayoutConfig {
            id: incomplete.pipeline_id,
            webview_id: incomplete.webview_id,
            url: final_url.clone(),
            is_iframe: incomplete.parent_info.is_some(),
            script_chan: self.senders.constellation_sender.clone(),
            image_cache: image_cache.clone(),
            font_context: font_context.clone(),
            time_profiler_chan: self.senders.time_profiler_sender.clone(),
            paint_api: self.paint_api.clone(),
            viewport_details: incomplete.viewport_details,
            user_stylesheets,
            theme: incomplete.theme,
            accessibility_active: self.accessibility_active.get(),
        };

        // Create the window and document objects.
        let window = Window::new(
            cx,
            incomplete.webview_id,
            self.js_runtime.clone(),
            self.senders.self_sender.clone(),
            self.layout_factory.create(layout_config),
            font_context,
            self.senders.image_cache_sender.clone(),
            image_cache.clone(),
            self.resource_threads.clone(),
            self.storage_threads.clone(),
            #[cfg(feature = "bluetooth")]
            self.senders.bluetooth_sender.clone(),
            self.senders.memory_profiler_sender.clone(),
            self.senders.time_profiler_sender.clone(),
            self.senders.devtools_server_sender.clone(),
            script_to_constellation_chan,
            self.senders.pipeline_to_embedder_sender.clone(),
            self.senders.constellation_sender.clone(),
            incomplete.pipeline_id,
            incomplete.parent_info,
            incomplete.viewport_details,
            origin.clone(),
            final_url.clone(),
            // TODO(37417): Set correct top-level URL here. Currently, we only specify the
            // url of the current window. However, in case this is an iframe, we should
            // pass in the URL from the frame that includes the iframe (which potentially
            // is another nested iframe in a frame).
            final_url.clone(),
            incomplete.navigation_start,
            self.webgl_chan.as_ref().map(|chan| chan.channel()),
            #[cfg(feature = "webxr")]
            self.webxr_registry.clone(),
            self.paint_api.clone(),
            self.unminify_js,
            self.unminify_css,
            self.local_script_source.clone(),
            user_contents,
            self.player_context.clone(),
            #[cfg(feature = "webgpu")]
            self.gpu_id_hub.clone(),
            incomplete.load_data.inherited_secure_context,
            incomplete.theme,
            self.this.clone(),
        );
        self.debugger_global.fire_add_debuggee(
            CanGc::from_cx(cx),
            window.upcast(),
            incomplete.pipeline_id,
            None,
        );

        let mut realm = enter_auto_realm(cx, &*window);
        let cx = &mut realm;

        // Initialize the browsing context for the window.
        let window_proxy = self.window_proxies.local_window_proxy(
            cx,
            &self.senders,
            &self.documents,
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
        let encoding_hint_from_content_type = content_type
            .as_ref()
            .and_then(|mime| mime.get_parameter(CHARSET))
            .and_then(|charset| Encoding::for_label(charset.as_bytes()));

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
            incomplete.load_data.about_base_url,
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
            self.custom_element_reaction_stack.clone(),
            incomplete.load_data.creation_sandboxing_flag_set,
            CanGc::from_cx(cx),
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

        document.set_ready_state(DocumentReadyState::Loading, CanGc::from_cx(cx));

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
                cx,
            );
        }

        self.senders
            .pipeline_to_constellation_sender
            .send((
                incomplete.webview_id,
                incomplete.pipeline_id,
                ScriptToConstellationMessage::ActivateDocument,
            ))
            .unwrap();

        // Notify devtools that a new script global exists.
        let incomplete_browsing_context_id: BrowsingContextId = incomplete.webview_id.into();
        let is_top_level_global = incomplete_browsing_context_id == incomplete.browsing_context_id;
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
            ServoParser::parse_xml_document(
                &document,
                None,
                final_url,
                encoding_hint_from_content_type,
                cx,
            );
        } else {
            ServoParser::parse_html_document(
                &document,
                None,
                final_url,
                encoding_hint_from_content_type,
                incomplete.load_data.container_document_encoding,
                cx,
            );
        }

        if incomplete.activity == DocumentActivity::FullyActive {
            window.resume(CanGc::from_cx(cx));
        } else {
            window.suspend(cx);
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

    /// Queue input events for later dispatching as part of a `update_the_rendering` task.
    fn handle_input_event(
        &self,
        webview_id: WebViewId,
        pipeline_id: PipelineId,
        event: ConstellationInputEvent,
    ) {
        let Some(document) = self.documents.borrow().find_document(pipeline_id) else {
            warn!("Input event sent to closed pipeline {pipeline_id}.");
            let _ = self
                .senders
                .pipeline_to_embedder_sender
                .send(EmbedderMsg::InputEventHandled(
                    webview_id,
                    event.event.id,
                    Default::default(),
                ));
            return;
        };
        document.event_handler().note_pending_input_event(event);
    }

    fn set_accessibility_active(&self, active: bool) {
        if !(pref!(accessibility_enabled)) {
            return;
        }

        let old_value = self.accessibility_active.replace(active);
        if active == old_value {
            return;
        }

        for (_, document) in self.documents.borrow().iter() {
            document.window().layout().set_accessibility_active(active);
        }
    }

    /// Handle a "navigate an iframe" message from the constellation.
    fn handle_navigate_iframe(
        &self,
        parent_pipeline_id: PipelineId,
        browsing_context_id: BrowsingContextId,
        load_data: LoadData,
        history_handling: NavigationHistoryBehavior,
        cx: &mut js::context::JSContext,
    ) {
        let iframe = self
            .documents
            .borrow()
            .find_iframe(parent_pipeline_id, browsing_context_id);
        if let Some(iframe) = iframe {
            iframe.navigate_or_reload_child_browsing_context(load_data, history_handling, cx);
        }
    }

    /// Turn javascript: URL into JS code to eval, according to the steps in
    /// <https://html.spec.whatwg.org/multipage/#javascript-protocol>
    fn eval_js_url(
        cx: &mut js::context::JSContext,
        global_scope: &GlobalScope,
        load_data: &mut LoadData,
    ) {
        // This slice of the URL’s serialization is equivalent to (5.) to (7.):
        // Start with the scheme data of the parsed URL;
        // append question mark and query component, if any;
        // append number sign and fragment component if any.
        let encoded = &load_data.url[Position::AfterScheme..][1..];

        // Percent-decode (8.) and UTF-8 decode (9.)
        let script_source = percent_decode(encoded.as_bytes()).decode_utf8_lossy();

        // Script source is ready to be evaluated (11.)
        let mut realm = enter_auto_realm(cx, global_scope);
        let cx = &mut realm.current_realm();

        rooted!(&in(cx) let mut jsval = UndefinedValue());
        _ = global_scope.evaluate_js_on_global(
            cx,
            script_source,
            "",
            Some(IntroductionType::JAVASCRIPT_URL),
            jsval.handle_mut(),
        );

        load_data.js_eval_result = if jsval.get().is_string() {
            let strval = DOMString::safe_from_jsval(
                cx.into(),
                jsval.handle(),
                StringificationBehavior::Empty,
                CanGc::from_cx(cx),
            );
            match strval {
                Ok(ConversionResult::Success(s)) => {
                    Some(JsEvalResult::Ok(String::from(s).as_bytes().to_vec()))
                },
                _ => None,
            }
        } else {
            Some(JsEvalResult::NoContent)
        };

        load_data.url = ServoUrl::parse("about:blank").unwrap();
    }

    /// Instructs the constellation to fetch the document that will be loaded. Stores the InProgressLoad
    /// argument until a notification is received that the fetch is complete.
    fn pre_page_load(&self, cx: &mut js::context::JSContext, mut incomplete: InProgressLoad) {
        let url_str = incomplete.load_data.url.as_str();
        if url_str == "about:blank" {
            self.start_page_load_about_blank(cx, incomplete);
            return;
        }
        if url_str == "about:srcdoc" {
            self.page_load_about_srcdoc(cx, incomplete);
            return;
        }

        let context = ParserContext::new(
            incomplete.webview_id,
            incomplete.pipeline_id,
            incomplete.load_data.url.clone(),
            incomplete.load_data.creation_sandboxing_flag_set,
        );
        self.incomplete_parser_contexts
            .0
            .borrow_mut()
            .push((incomplete.pipeline_id, context));

        let request_builder = incomplete.request_builder();
        incomplete.canceller = FetchCanceller::new(
            request_builder.id,
            false,
            self.resource_threads.core_thread.clone(),
        );
        NavigationListener::new(request_builder, self.senders.self_sender.clone())
            .initiate_fetch(&self.resource_threads.core_thread, None);
        self.incomplete_loads.borrow_mut().push(incomplete);
    }

    fn handle_navigation_response(
        &self,
        cx: &mut js::context::JSContext,
        pipeline_id: PipelineId,
        message: FetchResponseMsg,
    ) {
        if let Some(metadata) = NavigationListener::http_redirect_metadata(&message) {
            self.handle_navigation_redirect(pipeline_id, metadata);
            return;
        };

        match message {
            FetchResponseMsg::ProcessResponse(request_id, metadata) => {
                self.handle_fetch_metadata(pipeline_id, request_id, metadata)
            },
            FetchResponseMsg::ProcessResponseChunk(request_id, chunk) => {
                self.handle_fetch_chunk(pipeline_id, request_id, chunk.0)
            },
            FetchResponseMsg::ProcessResponseEOF(request_id, eof, timing) => {
                self.handle_fetch_eof(cx, pipeline_id, request_id, eof, timing)
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
        cx: &mut js::context::JSContext,
        id: PipelineId,
        request_id: RequestId,
        eof: Result<(), NetworkError>,
        timing: ResourceFetchTiming,
    ) {
        let idx = self
            .incomplete_parser_contexts
            .0
            .borrow()
            .iter()
            .position(|&(pipeline_id, _)| pipeline_id == id);

        if let Some(idx) = idx {
            let (_, context) = self.incomplete_parser_contexts.0.borrow_mut().remove(idx);

            // we need to register an iframe entry to the performance timeline if present
            if let Some(window_proxy) = context
                .get_document()
                .and_then(|document| document.browsing_context())
            {
                if let Some(frame_element) = window_proxy.frame_element() {
                    let iframe_ctx = IframeContext::new(
                        frame_element
                            .downcast::<HTMLIFrameElement>()
                            .expect("WindowProxy::frame_element should be an HTMLIFrameElement"),
                    );

                    // submit_timing will only accept timing that is of type ResourceTimingType::Resource
                    let mut resource_timing = timing.clone();
                    resource_timing.timing_type = ResourceTimingType::Resource;
                    submit_timing(&iframe_ctx, &eof, &resource_timing, CanGc::from_cx(cx));
                }
            }

            context.process_response_eof(cx, request_id, eof, timing);
        }
    }

    fn handle_csp_violations(&self, id: PipelineId, _: RequestId, violations: Vec<Violation>) {
        if let Some(global) = self.documents.borrow().find_global(id) {
            // TODO(https://github.com/w3c/webappsec-csp/issues/687): Update after spec is resolved
            global.report_csp_violations(violations, None, None);
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

        incomplete_load.canceller = FetchCanceller::new(
            request_builder.id,
            false,
            self.resource_threads.core_thread.clone(),
        );
        NavigationListener::new(request_builder, self.senders.self_sender.clone())
            .initiate_fetch(&self.resource_threads.core_thread, response_init);
    }

    /// Synchronously fetch `about:blank`. Stores the `InProgressLoad`
    /// argument until a notification is received that the fetch is complete.
    fn start_page_load_about_blank(
        &self,
        cx: &mut js::context::JSContext,
        mut incomplete: InProgressLoad,
    ) {
        let url = ServoUrl::parse("about:blank").unwrap();
        let mut context = ParserContext::new(
            incomplete.webview_id,
            incomplete.pipeline_id,
            url.clone(),
            incomplete.load_data.creation_sandboxing_flag_set,
        );

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

        let policy_container = incomplete.load_data.policy_container.clone();
        let about_base_url = incomplete.load_data.about_base_url.clone();
        self.incomplete_loads.borrow_mut().push(incomplete);

        let dummy_request_id = RequestId::default();
        context.process_response(dummy_request_id, Ok(FetchMetadata::Unfiltered(meta)));
        context.set_policy_container(policy_container.as_ref());
        context.set_about_base_url(about_base_url);
        context.process_response_chunk(dummy_request_id, chunk);
        context.process_response_eof(
            cx,
            dummy_request_id,
            Ok(()),
            ResourceFetchTiming::new(ResourceTimingType::None),
        );
    }

    /// Synchronously parse a srcdoc document from a giving HTML string.
    fn page_load_about_srcdoc(
        &self,
        cx: &mut js::context::JSContext,
        mut incomplete: InProgressLoad,
    ) {
        let url = ServoUrl::parse("about:srcdoc").unwrap();
        let mut meta = Metadata::default(url.clone());
        meta.set_content_type(Some(&mime::TEXT_HTML));
        meta.set_referrer_policy(incomplete.load_data.referrer_policy);

        let srcdoc = std::mem::take(&mut incomplete.load_data.srcdoc);
        let chunk = srcdoc.into_bytes();

        let policy_container = incomplete.load_data.policy_container.clone();
        let creation_sandboxing_flag_set = incomplete.load_data.creation_sandboxing_flag_set;

        let webview_id = incomplete.webview_id;
        let pipeline_id = incomplete.pipeline_id;
        let about_base_url = incomplete.load_data.about_base_url.clone();
        self.incomplete_loads.borrow_mut().push(incomplete);

        let mut context =
            ParserContext::new(webview_id, pipeline_id, url, creation_sandboxing_flag_set);
        let dummy_request_id = RequestId::default();

        context.process_response(dummy_request_id, Ok(FetchMetadata::Unfiltered(meta)));
        context.set_policy_container(policy_container.as_ref());
        context.set_about_base_url(about_base_url);
        context.process_response_chunk(dummy_request_id, chunk);
        context.process_response_eof(
            cx,
            dummy_request_id,
            Ok(()),
            ResourceFetchTiming::new(ResourceTimingType::None),
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
        let Some(ref sender) = self.senders.devtools_server_sender else {
            return;
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
        cx: &mut js::context::JSContext,
        pipeline_id: PipelineId,
        action: MediaSessionActionType,
    ) {
        if let Some(window) = self.documents.borrow().find_window(pipeline_id) {
            let media_session = window.Navigator().MediaSession();
            media_session.handle_action(cx, action);
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

    pub(crate) fn perform_a_microtask_checkpoint(&self, cx: &mut js::context::JSContext) {
        // Only perform the checkpoint if we're not shutting down.
        if self.can_continue_running_inner() {
            let globals = self
                .documents
                .borrow()
                .iter()
                .map(|(_id, document)| DomRoot::from_ref(document.window().upcast()))
                .collect();

            self.microtask_queue.checkpoint(
                cx,
                |id| self.documents.borrow().find_global(id),
                globals,
            )
        }
    }

    fn handle_evaluate_javascript(
        &self,
        webview_id: WebViewId,
        pipeline_id: PipelineId,
        evaluation_id: JavaScriptEvaluationId,
        script: String,
        cx: &mut js::context::JSContext,
    ) {
        let Some(window) = self.documents.borrow().find_window(pipeline_id) else {
            let _ = self.senders.pipeline_to_constellation_sender.send((
                webview_id,
                pipeline_id,
                ScriptToConstellationMessage::FinishJavaScriptEvaluation(
                    evaluation_id,
                    Err(JavaScriptEvaluationError::WebViewNotReady),
                ),
            ));
            return;
        };

        let global_scope = window.as_global_scope();
        let mut realm = enter_auto_realm(cx, global_scope);
        let cx = &mut realm.current_realm();

        rooted!(&in(cx) let mut return_value = UndefinedValue());
        if let Err(err) = global_scope.evaluate_js_on_global(
            cx,
            script.into(),
            "",
            None, // No known `introductionType` for JS code from embedder
            return_value.handle_mut(),
        ) {
            _ = self.senders.pipeline_to_constellation_sender.send((
                webview_id,
                pipeline_id,
                ScriptToConstellationMessage::FinishJavaScriptEvaluation(evaluation_id, Err(err)),
            ));
            return;
        };

        let result = jsval_to_webdriver(cx, global_scope, return_value.handle());
        let _ = self.senders.pipeline_to_constellation_sender.send((
            webview_id,
            pipeline_id,
            ScriptToConstellationMessage::FinishJavaScriptEvaluation(evaluation_id, result),
        ));
    }

    fn handle_refresh_cursor(&self, pipeline_id: PipelineId) {
        let Some(document) = self.documents.borrow().find_document(pipeline_id) else {
            return;
        };
        document.event_handler().handle_refresh_cursor();
    }

    pub(crate) fn is_servo_privileged(url: ServoUrl) -> bool {
        with_script_thread(|script_thread| script_thread.privileged_urls.contains(&url))
    }

    fn handle_request_screenshot_readiness(
        &self,
        webview_id: WebViewId,
        pipeline_id: PipelineId,
        can_gc: CanGc,
    ) {
        let Some(window) = self.documents.borrow().find_window(pipeline_id) else {
            let _ = self.senders.pipeline_to_constellation_sender.send((
                webview_id,
                pipeline_id,
                ScriptToConstellationMessage::RespondToScreenshotReadinessRequest(
                    ScreenshotReadinessResponse::NoLongerActive,
                ),
            ));
            return;
        };
        window.request_screenshot_readiness(can_gc);
    }

    fn handle_embedder_control_response(
        &self,
        id: EmbedderControlId,
        response: EmbedderControlResponse,
        can_gc: CanGc,
    ) {
        let Some(document) = self.documents.borrow().find_document(id.pipeline_id) else {
            return;
        };
        document
            .embedder_controls()
            .handle_embedder_control_response(id, response, can_gc);
    }

    pub(crate) fn handle_update_pinch_zoom_infos(
        &self,
        pipeline_id: PipelineId,
        pinch_zoom_infos: PinchZoomInfos,
        can_gc: CanGc,
    ) {
        let Some(window) = self.documents.borrow().find_window(pipeline_id) else {
            warn!("Visual viewport update for closed pipeline {pipeline_id}.");
            return;
        };

        window.maybe_update_visual_viewport(pinch_zoom_infos, can_gc);
    }

    pub(crate) fn devtools_want_updates_for_node(pipeline: PipelineId, node: &Node) -> bool {
        with_script_thread(|script_thread| {
            script_thread
                .devtools_state
                .wants_updates_for_node(pipeline, node)
        })
    }
}

impl Drop for ScriptThread {
    fn drop(&mut self) {
        SCRIPT_THREAD_ROOT.with(|root| {
            root.set(None);
        });
    }
}
