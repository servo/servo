/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

//! The script thread is the thread that owns the DOM in memory, runs JavaScript, and spawns parsing
//! and layout threads. It's in charge of processing events for all same-origin pages in a frame
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

use crate::devtools;
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
use crate::dom::bindings::reflector::DomObject;
use crate::dom::bindings::root::ThreadLocalStackRoots;
use crate::dom::bindings::root::{Dom, DomRoot, MutNullableDom, RootCollection};
use crate::dom::bindings::str::DOMString;
use crate::dom::bindings::trace::JSTraceable;
use crate::dom::bindings::utils::WRAP_CALLBACKS;
use crate::dom::customelementregistry::{
    CallbackReaction, CustomElementDefinition, CustomElementReactionStack,
};
use crate::dom::document::{
    Document, DocumentSource, FocusType, HasBrowsingContext, IsHTMLDocument, TouchEventResult,
};
use crate::dom::element::Element;
use crate::dom::event::{Event, EventBubbles, EventCancelable};
use crate::dom::globalscope::GlobalScope;
use crate::dom::htmlanchorelement::HTMLAnchorElement;
use crate::dom::htmliframeelement::{HTMLIFrameElement, NavigationType};
use crate::dom::identityhub::Identities;
use crate::dom::mutationobserver::MutationObserver;
use crate::dom::node::{window_from_node, Node, ShadowIncluding};
use crate::dom::performanceentry::PerformanceEntry;
use crate::dom::performancepainttiming::PerformancePaintTiming;
use crate::dom::serviceworker::TrustedServiceWorkerAddress;
use crate::dom::servoparser::{ParserContext, ServoParser};
use crate::dom::uievent::UIEvent;
use crate::dom::window::{ReflowReason, Window};
use crate::dom::windowproxy::{CreatorBrowsingContextInfo, WindowProxy};
use crate::dom::worker::TrustedWorkerAddress;
use crate::dom::worklet::WorkletThreadPool;
use crate::dom::workletglobalscope::WorkletGlobalScopeInit;
use crate::fetch::FetchCanceller;
use crate::microtask::{Microtask, MicrotaskQueue};
use crate::realms::enter_realm;
use crate::script_module::ScriptFetchOptions;
use crate::script_runtime::{
    get_reports, new_rt_and_cx, ContextForRequestInterrupt, JSContext, Runtime, ScriptPort,
};
use crate::script_runtime::{CommonScriptMsg, ScriptChan, ScriptThreadEventCategory};
use crate::task_manager::TaskManager;
use crate::task_queue::{QueuedTask, QueuedTaskConversion, TaskQueue};
use crate::task_source::dom_manipulation::DOMManipulationTaskSource;
use crate::task_source::file_reading::FileReadingTaskSource;
use crate::task_source::history_traversal::HistoryTraversalTaskSource;
use crate::task_source::media_element::MediaElementTaskSource;
use crate::task_source::networking::NetworkingTaskSource;
use crate::task_source::performance_timeline::PerformanceTimelineTaskSource;
use crate::task_source::port_message::PortMessageQueue;
use crate::task_source::remote_event::RemoteEventTaskSource;
use crate::task_source::timer::TimerTaskSource;
use crate::task_source::user_interaction::UserInteractionTaskSource;
use crate::task_source::websocket::WebsocketTaskSource;
use crate::task_source::TaskSource;
use crate::task_source::TaskSourceName;
use crate::webdriver_handlers;
use bluetooth_traits::BluetoothRequest;
use canvas_traits::webgl::WebGLPipeline;
use crossbeam_channel::{unbounded, Receiver, Sender};
use devtools_traits::CSSError;
use devtools_traits::{DevtoolScriptControlMsg, DevtoolsPageInfo};
use devtools_traits::{NavigationState, ScriptToDevtoolsControlMsg, WorkerId};
use embedder_traits::{EmbedderMsg, EventLoopWaker};
use euclid::default::{Point2D, Rect};
use euclid::Vector2D;
use headers::ReferrerPolicy as ReferrerPolicyHeader;
use headers::{HeaderMapExt, LastModified};
use hyper_serde::Serde;
use ipc_channel::ipc::{self, IpcSender};
use ipc_channel::router::ROUTER;
use js::glue::GetWindowProxyClass;
use js::jsapi::JS_SetWrapObjectCallbacks;
use js::jsapi::{
    JSContext as UnsafeJSContext, JSTracer, JS_AddInterruptCallback, SetWindowProxyClass,
};
use js::jsval::UndefinedValue;
use js::rust::ParentRuntime;
use media::WindowGLContext;
use metrics::{PaintTimeMetrics, MAX_TASK_NS};
use mime::{self, Mime};
use msg::constellation_msg::{
    BackgroundHangMonitor, BackgroundHangMonitorExitSignal, BackgroundHangMonitorRegister,
    ScriptHangAnnotation,
};
use msg::constellation_msg::{BrowsingContextId, HistoryStateId, PipelineId};
use msg::constellation_msg::{HangAnnotation, MonitoredComponentId, MonitoredComponentType};
use msg::constellation_msg::{PipelineNamespace, TopLevelBrowsingContextId};
use net_traits::image_cache::{ImageCache, PendingImageResponse};
use net_traits::request::{CredentialsMode, Destination, RedirectMode, RequestBuilder};
use net_traits::storage_thread::StorageType;
use net_traits::{FetchMetadata, FetchResponseListener, FetchResponseMsg};
use net_traits::{
    Metadata, NetworkError, ReferrerPolicy, ResourceFetchTiming, ResourceThreads,
    ResourceTimingType,
};
use parking_lot::Mutex;
use percent_encoding::percent_decode;
use profile_traits::mem::{self as profile_mem, OpaqueSender, ReportsChan};
use profile_traits::time::{self as profile_time, profile, ProfilerCategory};
use script_layout_interface::message::{self, LayoutThreadInit, Msg, ReflowGoal};
use script_traits::webdriver_msg::WebDriverScriptCommand;
use script_traits::CompositorEvent::{
    CompositionEvent, IMEDismissedEvent, KeyboardEvent, MouseButtonEvent, MouseMoveEvent,
    ResizeEvent, TouchEvent, WheelEvent,
};
use script_traits::{
    AnimationTickType, CompositorEvent, ConstellationControlMsg, DiscardBrowsingContext,
    DocumentActivity, EventResult, HistoryEntryReplacement, InitialScriptState, JsEvalResult,
    LayoutMsg, LoadData, LoadOrigin, MediaSessionActionType, MouseButton, MouseEventType,
    NewLayoutInfo, Painter, ProgressiveWebMetricType, ScriptMsg, ScriptThreadFactory,
    ScriptToConstellationChan, StructuredSerializedData, TimerSchedulerMsg, TouchEventType,
    TouchId, UntrustedNodeAddress, UpdatePipelineIdReason, WebrenderIpcSender, WheelDelta,
    WindowSizeData, WindowSizeType,
};
use servo_atoms::Atom;
use servo_config::opts;
use servo_url::{ImmutableOrigin, MutableOrigin, ServoUrl};
use std::borrow::Cow;
use std::cell::Cell;
use std::cell::RefCell;
use std::collections::{hash_map, HashMap, HashSet};
use std::default::Default;
use std::ops::Deref;
use std::option::Option;
use std::ptr;
use std::rc::Rc;
use std::result::Result;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::thread;
use std::time::{Duration, SystemTime};
use style::dom::OpaqueNode;
use style::thread_state::{self, ThreadState};
use time::{at_utc, get_time, precise_time_ns, Timespec};
use url::Position;
use webgpu::identity::WebGPUMsg;
use webrender_api::units::LayoutPixel;
use webrender_api::DocumentId;

pub type ImageCacheMsg = (PipelineId, PendingImageResponse);

thread_local!(static SCRIPT_THREAD_ROOT: Cell<Option<*const ScriptThread>> = Cell::new(None));

pub unsafe fn trace_thread(tr: *mut JSTracer) {
    SCRIPT_THREAD_ROOT.with(|root| {
        if let Some(script_thread) = root.get() {
            debug!("tracing fields of ScriptThread");
            (*script_thread).trace(tr);
        }
    });
}

/// A document load that is in the process of fetching the requested resource. Contains
/// data that will need to be present when the document and frame tree entry are created,
/// but is only easily available at initiation of the load and on a push basis (so some
/// data will be updated according to future resize events, viewport changes, etc.)
#[derive(JSTraceable)]
struct InProgressLoad {
    /// The pipeline which requested this load.
    pipeline_id: PipelineId,
    /// The browsing context being loaded into.
    browsing_context_id: BrowsingContextId,
    /// The top level ancestor browsing context.
    top_level_browsing_context_id: TopLevelBrowsingContextId,
    /// The parent pipeline and frame type associated with this load, if any.
    parent_info: Option<PipelineId>,
    /// The opener, if this is an auxiliary.
    opener: Option<BrowsingContextId>,
    /// The current window size associated with this pipeline.
    window_size: WindowSizeData,
    /// Channel to the layout thread associated with this pipeline.
    layout_chan: Sender<message::Msg>,
    /// The activity level of the document (inactive, active or fully active).
    activity: DocumentActivity,
    /// Window is visible.
    is_visible: bool,
    /// The requested URL of the load.
    url: ServoUrl,
    /// The origin for the document
    origin: MutableOrigin,
    /// Timestamp reporting the time when the browser started this load.
    navigation_start: u64,
    /// High res timestamp reporting the time when the browser started this load.
    navigation_start_precise: u64,
    /// For cancelling the fetch
    canceller: FetchCanceller,
    /// Flag for sharing with the layout thread that is not yet created.
    layout_is_busy: Arc<AtomicBool>,
    /// If inheriting the security context
    inherited_secure_context: Option<bool>,
}

impl InProgressLoad {
    /// Create a new InProgressLoad object.
    fn new(
        id: PipelineId,
        browsing_context_id: BrowsingContextId,
        top_level_browsing_context_id: TopLevelBrowsingContextId,
        parent_info: Option<PipelineId>,
        opener: Option<BrowsingContextId>,
        layout_chan: Sender<message::Msg>,
        window_size: WindowSizeData,
        url: ServoUrl,
        origin: MutableOrigin,
        layout_is_busy: Arc<AtomicBool>,
        inherited_secure_context: Option<bool>,
    ) -> InProgressLoad {
        let current_time = get_time();
        let navigation_start_precise = precise_time_ns();
        layout_chan
            .send(message::Msg::SetNavigationStart(navigation_start_precise))
            .unwrap();
        InProgressLoad {
            pipeline_id: id,
            browsing_context_id: browsing_context_id,
            top_level_browsing_context_id: top_level_browsing_context_id,
            parent_info: parent_info,
            opener: opener,
            layout_chan: layout_chan,
            window_size: window_size,
            activity: DocumentActivity::FullyActive,
            is_visible: true,
            url: url,
            origin: origin,
            navigation_start: (current_time.sec * 1000 + current_time.nsec as i64 / 1000000) as u64,
            navigation_start_precise: navigation_start_precise,
            canceller: Default::default(),
            layout_is_busy: layout_is_busy,
            inherited_secure_context: inherited_secure_context,
        }
    }
}

#[derive(Debug)]
enum MixedMessage {
    FromConstellation(ConstellationControlMsg),
    FromScript(MainThreadScriptMsg),
    FromDevtools(DevtoolScriptControlMsg),
    FromImageCache((PipelineId, PendingImageResponse)),
    FromWebGPUServer(WebGPUMsg),
}

/// Messages used to control the script event loop.
#[derive(Debug)]
pub enum MainThreadScriptMsg {
    /// Common variants associated with the script messages
    Common(CommonScriptMsg),
    /// Notifies the script thread that a new worklet has been loaded, and thus the page should be
    /// reflowed.
    WorkletLoaded(PipelineId),
    /// Notifies the script thread that a new paint worklet has been registered.
    RegisterPaintWorklet {
        pipeline_id: PipelineId,
        name: Atom,
        properties: Vec<Atom>,
        painter: Box<dyn Painter>,
    },
    /// A task related to a not fully-active document has been throttled.
    Inactive,
    /// Wake-up call from the task queue.
    WakeUp,
}

impl QueuedTaskConversion for MainThreadScriptMsg {
    fn task_source_name(&self) -> Option<&TaskSourceName> {
        let script_msg = match self {
            MainThreadScriptMsg::Common(script_msg) => script_msg,
            _ => return None,
        };
        match script_msg {
            CommonScriptMsg::Task(_category, _boxed, _pipeline_id, task_source) => {
                Some(&task_source)
            },
            _ => None,
        }
    }

    fn pipeline_id(&self) -> Option<PipelineId> {
        let script_msg = match self {
            MainThreadScriptMsg::Common(script_msg) => script_msg,
            _ => return None,
        };
        match script_msg {
            CommonScriptMsg::Task(_category, _boxed, pipeline_id, _task_source) => {
                pipeline_id.clone()
            },
            _ => None,
        }
    }

    fn into_queued_task(self) -> Option<QueuedTask> {
        let script_msg = match self {
            MainThreadScriptMsg::Common(script_msg) => script_msg,
            _ => return None,
        };
        let (category, boxed, pipeline_id, task_source) = match script_msg {
            CommonScriptMsg::Task(category, boxed, pipeline_id, task_source) => {
                (category, boxed, pipeline_id, task_source)
            },
            _ => return None,
        };
        Some((None, category, boxed, pipeline_id, task_source))
    }

    fn from_queued_task(queued_task: QueuedTask) -> Self {
        let (_worker, category, boxed, pipeline_id, task_source) = queued_task;
        let script_msg = CommonScriptMsg::Task(category, boxed, pipeline_id, task_source);
        MainThreadScriptMsg::Common(script_msg)
    }

    fn inactive_msg() -> Self {
        MainThreadScriptMsg::Inactive
    }

    fn wake_up_msg() -> Self {
        MainThreadScriptMsg::WakeUp
    }

    fn is_wake_up(&self) -> bool {
        match self {
            MainThreadScriptMsg::WakeUp => true,
            _ => false,
        }
    }
}

impl OpaqueSender<CommonScriptMsg> for Box<dyn ScriptChan + Send> {
    fn send(&self, msg: CommonScriptMsg) {
        ScriptChan::send(&**self, msg).unwrap();
    }
}

impl ScriptPort for Receiver<CommonScriptMsg> {
    fn recv(&self) -> Result<CommonScriptMsg, ()> {
        self.recv().map_err(|_| ())
    }
}

impl ScriptPort for Receiver<MainThreadScriptMsg> {
    fn recv(&self) -> Result<CommonScriptMsg, ()> {
        match self.recv() {
            Ok(MainThreadScriptMsg::Common(script_msg)) => Ok(script_msg),
            Ok(_) => panic!("unexpected main thread event message!"),
            Err(_) => Err(()),
        }
    }
}

impl ScriptPort for Receiver<(TrustedWorkerAddress, CommonScriptMsg)> {
    fn recv(&self) -> Result<CommonScriptMsg, ()> {
        self.recv().map(|(_, msg)| msg).map_err(|_| ())
    }
}

impl ScriptPort for Receiver<(TrustedWorkerAddress, MainThreadScriptMsg)> {
    fn recv(&self) -> Result<CommonScriptMsg, ()> {
        match self.recv().map(|(_, msg)| msg) {
            Ok(MainThreadScriptMsg::Common(script_msg)) => Ok(script_msg),
            Ok(_) => panic!("unexpected main thread event message!"),
            Err(_) => Err(()),
        }
    }
}

impl ScriptPort for Receiver<(TrustedServiceWorkerAddress, CommonScriptMsg)> {
    fn recv(&self) -> Result<CommonScriptMsg, ()> {
        self.recv().map(|(_, msg)| msg).map_err(|_| ())
    }
}

/// Encapsulates internal communication of shared messages within the script thread.
#[derive(JSTraceable)]
pub struct SendableMainThreadScriptChan(pub Sender<CommonScriptMsg>);

impl ScriptChan for SendableMainThreadScriptChan {
    fn send(&self, msg: CommonScriptMsg) -> Result<(), ()> {
        self.0.send(msg).map_err(|_| ())
    }

    fn clone(&self) -> Box<dyn ScriptChan + Send> {
        Box::new(SendableMainThreadScriptChan((&self.0).clone()))
    }
}

/// Encapsulates internal communication of main thread messages within the script thread.
#[derive(JSTraceable)]
pub struct MainThreadScriptChan(pub Sender<MainThreadScriptMsg>);

impl ScriptChan for MainThreadScriptChan {
    fn send(&self, msg: CommonScriptMsg) -> Result<(), ()> {
        self.0
            .send(MainThreadScriptMsg::Common(msg))
            .map_err(|_| ())
    }

    fn clone(&self) -> Box<dyn ScriptChan + Send> {
        Box::new(MainThreadScriptChan((&self.0).clone()))
    }
}

impl OpaqueSender<CommonScriptMsg> for Sender<MainThreadScriptMsg> {
    fn send(&self, msg: CommonScriptMsg) {
        self.send(MainThreadScriptMsg::Common(msg)).unwrap()
    }
}

/// The set of all documents managed by this script thread.
#[derive(JSTraceable)]
#[unrooted_must_root_lint::must_root]
pub struct Documents {
    map: HashMap<PipelineId, Dom<Document>>,
}

impl Documents {
    pub fn new() -> Documents {
        Documents {
            map: HashMap::new(),
        }
    }

    pub fn insert(&mut self, pipeline_id: PipelineId, doc: &Document) {
        self.map.insert(pipeline_id, Dom::from_ref(doc));
    }

    pub fn remove(&mut self, pipeline_id: PipelineId) -> Option<DomRoot<Document>> {
        self.map
            .remove(&pipeline_id)
            .map(|ref doc| DomRoot::from_ref(&**doc))
    }

    pub fn find_document(&self, pipeline_id: PipelineId) -> Option<DomRoot<Document>> {
        self.map
            .get(&pipeline_id)
            .map(|doc| DomRoot::from_ref(&**doc))
    }

    pub fn find_window(&self, pipeline_id: PipelineId) -> Option<DomRoot<Window>> {
        self.find_document(pipeline_id)
            .map(|doc| DomRoot::from_ref(doc.window()))
    }

    pub fn find_global(&self, pipeline_id: PipelineId) -> Option<DomRoot<GlobalScope>> {
        self.find_window(pipeline_id)
            .map(|window| DomRoot::from_ref(window.upcast()))
    }

    pub fn find_iframe(
        &self,
        pipeline_id: PipelineId,
        browsing_context_id: BrowsingContextId,
    ) -> Option<DomRoot<HTMLIFrameElement>> {
        self.find_document(pipeline_id)
            .and_then(|doc| doc.find_iframe(browsing_context_id))
    }

    pub fn iter<'a>(&'a self) -> DocumentsIter<'a> {
        DocumentsIter {
            iter: self.map.iter(),
        }
    }
}

#[allow(unrooted_must_root)]
pub struct DocumentsIter<'a> {
    iter: hash_map::Iter<'a, PipelineId, Dom<Document>>,
}

impl<'a> Iterator for DocumentsIter<'a> {
    type Item = (PipelineId, DomRoot<Document>);

    fn next(&mut self) -> Option<(PipelineId, DomRoot<Document>)> {
        self.iter
            .next()
            .map(|(id, doc)| (*id, DomRoot::from_ref(&**doc)))
    }
}

// We borrow the incomplete parser contexts mutably during parsing,
// which is fine except that parsing can trigger evaluation,
// which can trigger GC, and so we can end up tracing the script
// thread during parsing. For this reason, we don't trace the
// incomplete parser contexts during GC.
pub struct IncompleteParserContexts(RefCell<Vec<(PipelineId, ParserContext)>>);

unsafe_no_jsmanaged_fields!(TaskQueue<MainThreadScriptMsg>);
unsafe_no_jsmanaged_fields!(dyn BackgroundHangMonitorRegister);
unsafe_no_jsmanaged_fields!(dyn BackgroundHangMonitor);

#[derive(JSTraceable)]
// ScriptThread instances are rooted on creation, so this is okay
#[allow(unrooted_must_root)]
pub struct ScriptThread {
    /// The documents for pipelines managed by this thread
    documents: DomRefCell<Documents>,
    /// The window proxies known by this thread
    /// TODO: this map grows, but never shrinks. Issue #15258.
    window_proxies: DomRefCell<HashMap<BrowsingContextId, Dom<WindowProxy>>>,
    /// A list of data pertaining to loads that have not yet received a network response
    incomplete_loads: DomRefCell<Vec<InProgressLoad>>,
    /// A vector containing parser contexts which have not yet been fully processed
    incomplete_parser_contexts: IncompleteParserContexts,
    /// Image cache for this script thread.
    image_cache: Arc<dyn ImageCache>,
    /// A handle to the resource thread. This is an `Arc` to avoid running out of file descriptors if
    /// there are many iframes.
    resource_threads: ResourceThreads,
    /// A handle to the bluetooth thread.
    bluetooth_thread: IpcSender<BluetoothRequest>,

    /// A queue of tasks to be executed in this script-thread.
    task_queue: TaskQueue<MainThreadScriptMsg>,

    /// A handle to register associated layout threads for hang-monitoring.
    background_hang_monitor_register: Box<dyn BackgroundHangMonitorRegister>,
    /// The dedicated means of communication with the background-hang-monitor for this script-thread.
    background_hang_monitor: Box<dyn BackgroundHangMonitor>,
    /// A flag set to `true` by the BHM on exit, and checked from within the interrupt handler.
    closing: Arc<AtomicBool>,

    /// A channel to hand out to script thread-based entities that need to be able to enqueue
    /// events in the event queue.
    chan: MainThreadScriptChan,

    dom_manipulation_task_sender: Box<dyn ScriptChan>,

    media_element_task_sender: Sender<MainThreadScriptMsg>,

    user_interaction_task_sender: Sender<MainThreadScriptMsg>,

    networking_task_sender: Box<dyn ScriptChan>,

    history_traversal_task_sender: Sender<MainThreadScriptMsg>,

    file_reading_task_sender: Box<dyn ScriptChan>,

    performance_timeline_task_sender: Box<dyn ScriptChan>,

    port_message_sender: Box<dyn ScriptChan>,

    timer_task_sender: Box<dyn ScriptChan>,

    remote_event_task_sender: Box<dyn ScriptChan>,

    /// A channel to hand out to threads that need to respond to a message from the script thread.
    control_chan: IpcSender<ConstellationControlMsg>,

    /// The port on which the constellation and layout threads can communicate with the
    /// script thread.
    control_port: Receiver<ConstellationControlMsg>,

    /// For communicating load url messages to the constellation
    script_sender: IpcSender<(PipelineId, ScriptMsg)>,

    /// A sender for new layout threads to communicate to the constellation.
    layout_to_constellation_chan: IpcSender<LayoutMsg>,

    /// The port on which we receive messages from the image cache
    image_cache_port: Receiver<ImageCacheMsg>,

    /// The channel on which the image cache can send messages to ourself.
    image_cache_channel: Sender<ImageCacheMsg>,
    /// For providing contact with the time profiler.
    time_profiler_chan: profile_time::ProfilerChan,

    /// For providing contact with the memory profiler.
    mem_profiler_chan: profile_mem::ProfilerChan,

    /// For providing instructions to an optional devtools server.
    devtools_chan: Option<IpcSender<ScriptToDevtoolsControlMsg>>,
    /// For receiving commands from an optional devtools server. Will be ignored if
    /// no such server exists.
    devtools_port: Receiver<DevtoolScriptControlMsg>,
    devtools_sender: IpcSender<DevtoolScriptControlMsg>,

    /// The JavaScript runtime.
    js_runtime: Rc<Runtime>,

    /// The topmost element over the mouse.
    topmost_mouse_over_target: MutNullableDom<Element>,

    /// List of pipelines that have been owned and closed by this script thread.
    closed_pipelines: DomRefCell<HashSet<PipelineId>>,

    scheduler_chan: IpcSender<TimerSchedulerMsg>,

    content_process_shutdown_chan: Sender<()>,

    /// <https://html.spec.whatwg.org/multipage/#microtask-queue>
    microtask_queue: Rc<MicrotaskQueue>,

    /// Microtask Queue for adding support for mutation observer microtasks
    mutation_observer_microtask_queued: Cell<bool>,

    /// The unit of related similar-origin browsing contexts' list of MutationObserver objects
    mutation_observers: DomRefCell<Vec<Dom<MutationObserver>>>,

    /// A handle to the WebGL thread
    webgl_chan: Option<WebGLPipeline>,

    /// The WebXR device registry
    webxr_registry: webxr_api::Registry,

    /// The worklet thread pool
    worklet_thread_pool: DomRefCell<Option<Rc<WorkletThreadPool>>>,

    /// A list of pipelines containing documents that finished loading all their blocking
    /// resources during a turn of the event loop.
    docs_with_no_blocking_loads: DomRefCell<HashSet<Dom<Document>>>,

    /// <https://html.spec.whatwg.org/multipage/#custom-element-reactions-stack>
    custom_element_reaction_stack: CustomElementReactionStack,

    /// The Webrender Document ID associated with this thread.
    webrender_document: DocumentId,

    /// Webrender API sender.
    webrender_api_sender: WebrenderIpcSender,

    /// Periodically print out on which events script threads spend their processing time.
    profile_script_events: bool,

    /// Print Progressive Web Metrics to console.
    print_pwm: bool,

    /// Emits notifications when there is a relayout.
    relayout_event: bool,

    /// True if it is safe to write to the image.
    prepare_for_screenshot: bool,

    /// Unminify Javascript.
    unminify_js: bool,

    /// Directory with stored unminified scripts
    local_script_source: Option<String>,

    /// Where to load userscripts from, if any. An empty string will load from
    /// the resources/user-agent-js directory, and if the option isn't passed userscripts
    /// won't be loaded
    userscripts_path: Option<String>,

    /// True if headless mode.
    headless: bool,

    /// Replace unpaired surrogates in DOM strings with U+FFFD.
    /// See <https://github.com/servo/servo/issues/6564>
    replace_surrogates: bool,

    /// An optional string allowing the user agent to be set for testing.
    user_agent: Cow<'static, str>,

    /// Application window's GL Context for Media player
    player_context: WindowGLContext,

    /// A mechanism to force the compositor's event loop to process events.
    event_loop_waker: Option<Box<dyn EventLoopWaker>>,

    /// A set of all nodes ever created in this script thread
    node_ids: DomRefCell<HashSet<String>>,

    /// Code is running as a consequence of a user interaction
    is_user_interacting: Cell<bool>,

    /// Identity manager for WebGPU resources
    gpu_id_hub: Arc<Mutex<Identities>>,

    /// Receiver to receive commands from optional WebGPU server.
    webgpu_port: RefCell<Option<Receiver<WebGPUMsg>>>,

    // Secure context
    inherited_secure_context: Option<bool>,
}

struct BHMExitSignal {
    closing: Arc<AtomicBool>,
    js_context: ContextForRequestInterrupt,
}

impl BackgroundHangMonitorExitSignal for BHMExitSignal {
    fn signal_to_exit(&self) {
        self.closing.store(true, Ordering::SeqCst);
        self.js_context.request_interrupt();
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

impl<'a> Drop for ScriptMemoryFailsafe<'a> {
    #[allow(unrooted_must_root)]
    fn drop(&mut self) {
        if let Some(owner) = self.owner {
            for (_, document) in owner.documents.borrow().iter() {
                document.window().clear_js_runtime_for_script_deallocation();
            }
        }
    }
}

impl ScriptThreadFactory for ScriptThread {
    type Message = message::Msg;

    fn create(
        state: InitialScriptState,
        load_data: LoadData,
        profile_script_events: bool,
        print_pwm: bool,
        relayout_event: bool,
        prepare_for_screenshot: bool,
        unminify_js: bool,
        local_script_source: Option<String>,
        userscripts_path: Option<String>,
        headless: bool,
        replace_surrogates: bool,
        user_agent: Cow<'static, str>,
    ) -> (Sender<message::Msg>, Receiver<message::Msg>) {
        let (script_chan, script_port) = unbounded();

        let (sender, receiver) = unbounded();
        let layout_chan = sender.clone();
        thread::Builder::new()
            .name(format!("ScriptThread {:?}", state.id))
            .spawn(move || {
                thread_state::initialize(ThreadState::SCRIPT);
                PipelineNamespace::install(state.pipeline_namespace_id);
                TopLevelBrowsingContextId::install(state.top_level_browsing_context_id);
                let roots = RootCollection::new();
                let _stack_roots = ThreadLocalStackRoots::new(&roots);
                let id = state.id;
                let browsing_context_id = state.browsing_context_id;
                let top_level_browsing_context_id = state.top_level_browsing_context_id;
                let parent_info = state.parent_info;
                let opener = state.opener;
                let secure = load_data.inherited_secure_context.clone();
                let mem_profiler_chan = state.mem_profiler_chan.clone();
                let window_size = state.window_size;
                let layout_is_busy = state.layout_is_busy.clone();

                let script_thread = ScriptThread::new(
                    state,
                    script_port,
                    script_chan.clone(),
                    profile_script_events,
                    print_pwm,
                    relayout_event,
                    prepare_for_screenshot,
                    unminify_js,
                    local_script_source,
                    userscripts_path,
                    headless,
                    replace_surrogates,
                    user_agent,
                );

                SCRIPT_THREAD_ROOT.with(|root| {
                    root.set(Some(&script_thread as *const _));
                });

                let mut failsafe = ScriptMemoryFailsafe::new(&script_thread);

                let origin = MutableOrigin::new(load_data.url.origin());
                let new_load = InProgressLoad::new(
                    id,
                    browsing_context_id,
                    top_level_browsing_context_id,
                    parent_info,
                    opener,
                    layout_chan,
                    window_size,
                    load_data.url.clone(),
                    origin,
                    layout_is_busy,
                    secure,
                );
                script_thread.pre_page_load(new_load, load_data);

                let reporter_name = format!("script-reporter-{}", id);
                mem_profiler_chan.run_with_memory_reporting(
                    || {
                        script_thread.start();
                        let _ = script_thread.content_process_shutdown_chan.send(());
                    },
                    reporter_name,
                    script_chan,
                    CommonScriptMsg::CollectReports,
                );

                // This must always be the very last operation performed before the thread completes
                failsafe.neuter();
            })
            .expect("Thread spawning failed");

        (sender, receiver)
    }
}

impl ScriptThread {
    pub fn runtime_handle() -> ParentRuntime {
        SCRIPT_THREAD_ROOT.with(|root| {
            let script_thread = unsafe { &*root.get().unwrap() };
            script_thread.js_runtime.prepare_for_new_child()
        })
    }

    pub fn can_continue_running() -> bool {
        SCRIPT_THREAD_ROOT.with(|root| {
            let script_thread = unsafe { &*root.get().unwrap() };
            script_thread.can_continue_running_inner()
        })
    }

    pub fn prepare_for_shutdown() {
        SCRIPT_THREAD_ROOT.with(|root| {
            let script_thread = unsafe { &*root.get().unwrap() };
            script_thread.prepare_for_shutdown_inner();
        })
    }

    pub fn set_mutation_observer_microtask_queued(value: bool) {
        SCRIPT_THREAD_ROOT.with(|root| {
            let script_thread = unsafe { &*root.get().unwrap() };
            script_thread.mutation_observer_microtask_queued.set(value);
        })
    }

    pub fn is_mutation_observer_microtask_queued() -> bool {
        SCRIPT_THREAD_ROOT.with(|root| {
            let script_thread = unsafe { &*root.get().unwrap() };
            return script_thread.mutation_observer_microtask_queued.get();
        })
    }

    pub fn add_mutation_observer(observer: &MutationObserver) {
        SCRIPT_THREAD_ROOT.with(|root| {
            let script_thread = unsafe { &*root.get().unwrap() };
            script_thread
                .mutation_observers
                .borrow_mut()
                .push(Dom::from_ref(observer));
        })
    }

    pub fn get_mutation_observers() -> Vec<DomRoot<MutationObserver>> {
        SCRIPT_THREAD_ROOT.with(|root| {
            let script_thread = unsafe { &*root.get().unwrap() };
            script_thread
                .mutation_observers
                .borrow()
                .iter()
                .map(|o| DomRoot::from_ref(&**o))
                .collect()
        })
    }

    pub fn mark_document_with_no_blocked_loads(doc: &Document) {
        SCRIPT_THREAD_ROOT.with(|root| {
            if let Some(script_thread) = root.get() {
                let script_thread = unsafe { &*script_thread };
                script_thread
                    .docs_with_no_blocking_loads
                    .borrow_mut()
                    .insert(Dom::from_ref(doc));
            }
        })
    }

    pub fn page_headers_available(
        id: &PipelineId,
        metadata: Option<Metadata>,
    ) -> Option<DomRoot<ServoParser>> {
        SCRIPT_THREAD_ROOT.with(|root| {
            let script_thread = unsafe { &*root.get().unwrap() };
            script_thread.handle_page_headers_available(id, metadata)
        })
    }

    /// Process a single event as if it were the next event
    /// in the queue for this window event-loop.
    /// Returns a boolean indicating whether further events should be processed.
    pub fn process_event(msg: CommonScriptMsg) -> bool {
        SCRIPT_THREAD_ROOT.with(|root| {
            if let Some(script_thread) = root.get() {
                let script_thread = unsafe { &*script_thread };
                if !script_thread.can_continue_running_inner() {
                    return false;
                } else {
                    script_thread.handle_msg_from_script(MainThreadScriptMsg::Common(msg));
                    return true;
                }
            }
            false
        })
    }

    // https://html.spec.whatwg.org/multipage/#await-a-stable-state
    pub fn await_stable_state(task: Microtask) {
        SCRIPT_THREAD_ROOT.with(|root| {
            if let Some(script_thread) = root.get() {
                let script_thread = unsafe { &*script_thread };
                script_thread
                    .microtask_queue
                    .enqueue(task, script_thread.get_cx());
            }
        });
    }

    /// Check that two origins are "similar enough",
    /// for now only used to prevent cross-origin JS url evaluation.
    ///
    /// https://github.com/whatwg/html/issues/2591
    pub fn check_load_origin(source: &LoadOrigin, target: &ImmutableOrigin) -> bool {
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

    /// Step 13 of https://html.spec.whatwg.org/multipage/#navigate
    pub fn navigate(
        browsing_context: BrowsingContextId,
        pipeline_id: PipelineId,
        mut load_data: LoadData,
        replace: HistoryEntryReplacement,
    ) {
        SCRIPT_THREAD_ROOT.with(|root| {
            let script_thread = match root.get() {
                None => return,
                Some(script) => script,
            };
            let script_thread = unsafe { &*script_thread };
            let is_javascript = load_data.url.scheme() == "javascript";
            // If resource is a request whose url's scheme is "javascript"
            // https://html.spec.whatwg.org/multipage/#javascript-protocol
            if is_javascript {
                let window = match script_thread.documents.borrow().find_window(pipeline_id) {
                    None => return,
                    Some(window) => window,
                };
                let global = window.upcast::<GlobalScope>();
                let trusted_global = Trusted::new(global);
                let sender = script_thread.script_sender.clone();
                let task = task!(navigate_javascript: move || {
                    // Important re security. See https://github.com/servo/servo/issues/23373
                    // TODO: check according to https://w3c.github.io/webappsec-csp/#should-block-navigation-request
                    if let Some(window) = trusted_global.root().downcast::<Window>() {
                        if ScriptThread::check_load_origin(&load_data.load_origin, &window.get_url().origin()) {
                            ScriptThread::eval_js_url(&trusted_global.root(), &mut load_data);
                            sender
                                .send((pipeline_id, ScriptMsg::LoadUrl(load_data, replace)))
                                .unwrap();
                        }
                    }
                });
                global
                    .dom_manipulation_task_source()
                    .queue(task, global.upcast())
                    .expect("Enqueing navigate js task on the DOM manipulation task source failed");
            } else {
                if let Some(ref sender) = script_thread.devtools_chan {
                    let _ = sender.send(ScriptToDevtoolsControlMsg::Navigate(
                        browsing_context, NavigationState::Start(load_data.url.clone())
                    ));
                }

                script_thread
                    .script_sender
                    .send((pipeline_id, ScriptMsg::LoadUrl(load_data, replace)))
                    .expect("Sending a LoadUrl message to the constellation failed");
            }
        });
    }

    pub fn process_attach_layout(new_layout_info: NewLayoutInfo, origin: MutableOrigin) {
        SCRIPT_THREAD_ROOT.with(|root| {
            if let Some(script_thread) = root.get() {
                let script_thread = unsafe { &*script_thread };
                let pipeline_id = Some(new_layout_info.new_pipeline_id);
                script_thread.profile_event(
                    ScriptThreadEventCategory::AttachLayout,
                    pipeline_id,
                    || {
                        script_thread.handle_new_layout(new_layout_info, origin);
                    },
                )
            }
        });
    }

    pub fn get_top_level_for_browsing_context(
        sender_pipeline: PipelineId,
        browsing_context_id: BrowsingContextId,
    ) -> Option<TopLevelBrowsingContextId> {
        SCRIPT_THREAD_ROOT.with(|root| {
            root.get().and_then(|script_thread| {
                let script_thread = unsafe { &*script_thread };
                script_thread
                    .ask_constellation_for_top_level_info(sender_pipeline, browsing_context_id)
            })
        })
    }

    pub fn find_document(id: PipelineId) -> Option<DomRoot<Document>> {
        SCRIPT_THREAD_ROOT.with(|root| {
            root.get().and_then(|script_thread| {
                let script_thread = unsafe { &*script_thread };
                script_thread.documents.borrow().find_document(id)
            })
        })
    }

    pub fn set_user_interacting(interacting: bool) {
        SCRIPT_THREAD_ROOT.with(|root| {
            if let Some(script_thread) = root.get() {
                let script_thread = unsafe { &*script_thread };
                script_thread.is_user_interacting.set(interacting);
            }
        });
    }

    pub fn is_user_interacting() -> bool {
        SCRIPT_THREAD_ROOT.with(|root| {
            root.get().map_or(false, |script_thread| {
                let script_thread = unsafe { &*script_thread };
                script_thread.is_user_interacting.get()
            })
        })
    }

    pub fn get_fully_active_document_ids() -> HashSet<PipelineId> {
        SCRIPT_THREAD_ROOT.with(|root| {
            root.get().map_or(HashSet::new(), |script_thread| {
                let script_thread = unsafe { &*script_thread };
                script_thread
                    .documents
                    .borrow()
                    .iter()
                    .filter_map(|(id, document)| {
                        if document.is_fully_active() {
                            Some(id.clone())
                        } else {
                            None
                        }
                    })
                    .fold(HashSet::new(), |mut set, id| {
                        let _ = set.insert(id);
                        set
                    })
            })
        })
    }

    pub fn find_window_proxy(id: BrowsingContextId) -> Option<DomRoot<WindowProxy>> {
        SCRIPT_THREAD_ROOT.with(|root| {
            root.get().and_then(|script_thread| {
                let script_thread = unsafe { &*script_thread };
                script_thread
                    .window_proxies
                    .borrow()
                    .get(&id)
                    .map(|context| DomRoot::from_ref(&**context))
            })
        })
    }

    pub fn find_window_proxy_by_name(name: &DOMString) -> Option<DomRoot<WindowProxy>> {
        SCRIPT_THREAD_ROOT.with(|root| {
            root.get().and_then(|script_thread| {
                let script_thread = unsafe { &*script_thread };
                for (_, proxy) in script_thread.window_proxies.borrow().iter() {
                    if proxy.get_name() == *name {
                        return Some(DomRoot::from_ref(&**proxy));
                    }
                }
                None
            })
        })
    }

    pub fn worklet_thread_pool() -> Rc<WorkletThreadPool> {
        SCRIPT_THREAD_ROOT.with(|root| {
            let script_thread = unsafe { &*root.get().unwrap() };
            script_thread
                .worklet_thread_pool
                .borrow_mut()
                .get_or_insert_with(|| {
                    let init = WorkletGlobalScopeInit {
                        to_script_thread_sender: script_thread.chan.0.clone(),
                        resource_threads: script_thread.resource_threads.clone(),
                        mem_profiler_chan: script_thread.mem_profiler_chan.clone(),
                        time_profiler_chan: script_thread.time_profiler_chan.clone(),
                        devtools_chan: script_thread.devtools_chan.clone(),
                        to_constellation_sender: script_thread.script_sender.clone(),
                        scheduler_chan: script_thread.scheduler_chan.clone(),
                        image_cache: script_thread.image_cache.clone(),
                        is_headless: script_thread.headless,
                        user_agent: script_thread.user_agent.clone(),
                        gpu_id_hub: script_thread.gpu_id_hub.clone(),
                        inherited_secure_context: script_thread.inherited_secure_context.clone(),
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
        let window = self.documents.borrow().find_window(pipeline_id);
        let window = match window {
            Some(window) => window,
            None => {
                return warn!(
                    "Paint worklet registered after pipeline {} closed.",
                    pipeline_id
                );
            },
        };

        match window.layout_chan() {
            Some(chan) => chan
                .send(Msg::RegisterPaint(name, properties, painter))
                .unwrap(),
            None => warn!("Layout channel unavailable"),
        }
    }

    pub fn push_new_element_queue() {
        SCRIPT_THREAD_ROOT.with(|root| {
            if let Some(script_thread) = root.get() {
                let script_thread = unsafe { &*script_thread };
                script_thread
                    .custom_element_reaction_stack
                    .push_new_element_queue();
            }
        })
    }

    pub fn pop_current_element_queue() {
        SCRIPT_THREAD_ROOT.with(|root| {
            if let Some(script_thread) = root.get() {
                let script_thread = unsafe { &*script_thread };
                script_thread
                    .custom_element_reaction_stack
                    .pop_current_element_queue();
            }
        })
    }

    pub fn enqueue_callback_reaction(
        element: &Element,
        reaction: CallbackReaction,
        definition: Option<Rc<CustomElementDefinition>>,
    ) {
        SCRIPT_THREAD_ROOT.with(|root| {
            if let Some(script_thread) = root.get() {
                let script_thread = unsafe { &*script_thread };
                script_thread
                    .custom_element_reaction_stack
                    .enqueue_callback_reaction(element, reaction, definition);
            }
        })
    }

    pub fn enqueue_upgrade_reaction(element: &Element, definition: Rc<CustomElementDefinition>) {
        SCRIPT_THREAD_ROOT.with(|root| {
            if let Some(script_thread) = root.get() {
                let script_thread = unsafe { &*script_thread };
                script_thread
                    .custom_element_reaction_stack
                    .enqueue_upgrade_reaction(element, definition);
            }
        })
    }

    pub fn invoke_backup_element_queue() {
        SCRIPT_THREAD_ROOT.with(|root| {
            if let Some(script_thread) = root.get() {
                let script_thread = unsafe { &*script_thread };
                script_thread
                    .custom_element_reaction_stack
                    .invoke_backup_element_queue();
            }
        })
    }

    pub fn save_node_id(node_id: String) {
        SCRIPT_THREAD_ROOT.with(|root| {
            if let Some(script_thread) = root.get() {
                let script_thread = unsafe { &*script_thread };
                script_thread.node_ids.borrow_mut().insert(node_id);
            }
        })
    }

    pub fn has_node_id(node_id: &str) -> bool {
        SCRIPT_THREAD_ROOT.with(|root| match root.get() {
            Some(script_thread) => {
                let script_thread = unsafe { &*script_thread };
                script_thread.node_ids.borrow().contains(node_id)
            },
            None => false,
        })
    }

    /// Creates a new script thread.
    pub fn new(
        state: InitialScriptState,
        port: Receiver<MainThreadScriptMsg>,
        chan: Sender<MainThreadScriptMsg>,
        profile_script_events: bool,
        print_pwm: bool,
        relayout_event: bool,
        prepare_for_screenshot: bool,
        unminify_js: bool,
        local_script_source: Option<String>,
        userscripts_path: Option<String>,
        headless: bool,
        replace_surrogates: bool,
        user_agent: Cow<'static, str>,
    ) -> ScriptThread {
        let boxed_script_sender = Box::new(MainThreadScriptChan(chan.clone()));

        let runtime = new_rt_and_cx(Some(NetworkingTaskSource(
            boxed_script_sender.clone(),
            state.id,
        )));
        let cx = runtime.cx();

        unsafe {
            JS_SetWrapObjectCallbacks(cx, &WRAP_CALLBACKS);
            SetWindowProxyClass(cx, GetWindowProxyClass());
            JS_AddInterruptCallback(cx, Some(interrupt_callback));
        }

        // Ask the router to proxy IPC messages from the devtools to us.
        let (ipc_devtools_sender, ipc_devtools_receiver) = ipc::channel().unwrap();
        let devtools_port =
            ROUTER.route_ipc_receiver_to_new_crossbeam_receiver(ipc_devtools_receiver);

        let (image_cache_channel, image_cache_port) = unbounded();

        let task_queue = TaskQueue::new(port, chan.clone());

        let closing = Arc::new(AtomicBool::new(false));
        let background_hang_monitor_exit_signal = BHMExitSignal {
            closing: closing.clone(),
            js_context: ContextForRequestInterrupt::new(cx),
        };

        let background_hang_monitor = state.background_hang_monitor_register.register_component(
            MonitoredComponentId(state.id.clone(), MonitoredComponentType::Script),
            Duration::from_millis(1000),
            Duration::from_millis(5000),
            Some(Box::new(background_hang_monitor_exit_signal)),
        );

        // Ask the router to proxy IPC messages from the control port to us.
        let control_port = ROUTER.route_ipc_receiver_to_new_crossbeam_receiver(state.control_port);

        ScriptThread {
            documents: DomRefCell::new(Documents::new()),
            window_proxies: DomRefCell::new(HashMap::new()),
            incomplete_loads: DomRefCell::new(vec![]),
            incomplete_parser_contexts: IncompleteParserContexts(RefCell::new(vec![])),

            image_cache: state.image_cache.clone(),
            image_cache_channel: image_cache_channel,
            image_cache_port: image_cache_port,

            resource_threads: state.resource_threads,
            bluetooth_thread: state.bluetooth_thread,

            task_queue,

            background_hang_monitor_register: state.background_hang_monitor_register,
            background_hang_monitor,
            closing,

            chan: MainThreadScriptChan(chan.clone()),
            dom_manipulation_task_sender: boxed_script_sender.clone(),
            media_element_task_sender: chan.clone(),
            user_interaction_task_sender: chan.clone(),
            networking_task_sender: boxed_script_sender.clone(),
            port_message_sender: boxed_script_sender.clone(),
            file_reading_task_sender: boxed_script_sender.clone(),
            performance_timeline_task_sender: boxed_script_sender.clone(),
            timer_task_sender: boxed_script_sender.clone(),
            remote_event_task_sender: boxed_script_sender.clone(),

            history_traversal_task_sender: chan.clone(),

            control_chan: state.control_chan,
            control_port: control_port,
            script_sender: state.script_to_constellation_chan.sender.clone(),
            time_profiler_chan: state.time_profiler_chan.clone(),
            mem_profiler_chan: state.mem_profiler_chan,

            devtools_chan: state.devtools_chan,
            devtools_port: devtools_port,
            devtools_sender: ipc_devtools_sender,

            microtask_queue: runtime.microtask_queue.clone(),

            js_runtime: Rc::new(runtime),
            topmost_mouse_over_target: MutNullableDom::new(Default::default()),
            closed_pipelines: DomRefCell::new(HashSet::new()),

            scheduler_chan: state.scheduler_chan,

            content_process_shutdown_chan: state.content_process_shutdown_chan,

            mutation_observer_microtask_queued: Default::default(),

            mutation_observers: Default::default(),

            layout_to_constellation_chan: state.layout_to_constellation_chan,

            webgl_chan: state.webgl_chan,
            webxr_registry: state.webxr_registry,

            worklet_thread_pool: Default::default(),

            docs_with_no_blocking_loads: Default::default(),

            custom_element_reaction_stack: CustomElementReactionStack::new(),

            webrender_document: state.webrender_document,
            webrender_api_sender: state.webrender_api_sender,

            profile_script_events,
            print_pwm,

            relayout_event,
            prepare_for_screenshot,
            unminify_js,
            local_script_source,

            userscripts_path,
            headless,
            replace_surrogates,
            user_agent,
            player_context: state.player_context,
            event_loop_waker: state.event_loop_waker,

            node_ids: Default::default(),
            is_user_interacting: Cell::new(false),
            gpu_id_hub: Arc::new(Mutex::new(Identities::new())),
            webgpu_port: RefCell::new(None),
            inherited_secure_context: state.inherited_secure_context,
        }
    }

    #[allow(unsafe_code)]
    pub fn get_cx(&self) -> JSContext {
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
            let window = document.window();
            window.ignore_all_tasks();
        }
    }

    /// Starts the script thread. After calling this method, the script thread will loop receiving
    /// messages on its port.
    pub fn start(&self) {
        debug!("Starting script thread.");
        while self.handle_msgs() {
            // Go on...
            debug!("Running script thread.");
        }
        debug!("Stopped script thread.");
    }

    /// Handle incoming control messages.
    fn handle_msgs(&self) -> bool {
        use self::MixedMessage::FromScript;
        use self::MixedMessage::{
            FromConstellation, FromDevtools, FromImageCache, FromWebGPUServer,
        };

        // Handle pending resize events.
        // Gather them first to avoid a double mut borrow on self.
        let mut resizes = vec![];

        for (id, document) in self.documents.borrow().iter() {
            // Only process a resize if layout is idle.
            if let Some((size, size_type)) = document.window().steal_resize_event() {
                resizes.push((id, size, size_type));
            }
        }

        for (id, size, size_type) in resizes {
            self.handle_event(id, ResizeEvent(size, size_type));
        }

        // Store new resizes, and gather all other events.
        let mut sequential = vec![];

        // Notify the background-hang-monitor we are waiting for an event.
        self.background_hang_monitor.notify_wait();

        // Receive at least one message so we don't spinloop.
        debug!("Waiting for event.");
        let mut event = select! {
            recv(self.task_queue.select()) -> msg => {
                self.task_queue.take_tasks(msg.unwrap());
                let event = self
                    .task_queue
                    .recv()
                    .expect("Spurious wake-up of the event-loop, task-queue has no tasks available");
                FromScript(event)
            },
            recv(self.control_port) -> msg => FromConstellation(msg.unwrap()),
            recv(self.devtools_chan.as_ref().map(|_| &self.devtools_port).unwrap_or(&crossbeam_channel::never())) -> msg
                => FromDevtools(msg.unwrap()),
            recv(self.image_cache_port) -> msg => FromImageCache(msg.unwrap()),
            recv(self.webgpu_port.borrow().as_ref().unwrap_or(&crossbeam_channel::never())) -> msg
                => FromWebGPUServer(msg.unwrap()),
        };
        debug!("Got event.");

        // Squash any pending resize, reflow, animation tick, and mouse-move events in the queue.
        let mut mouse_move_event_index = None;
        let mut animation_ticks = HashSet::new();
        loop {
            // https://html.spec.whatwg.org/multipage/#event-loop-processing-model step 7
            match event {
                // This has to be handled before the ResizeMsg below,
                // otherwise the page may not have been added to the
                // child list yet, causing the find() to fail.
                FromConstellation(ConstellationControlMsg::AttachLayout(new_layout_info)) => {
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
                FromConstellation(ConstellationControlMsg::Resize(id, size, size_type)) => {
                    // step 7.7
                    self.profile_event(ScriptThreadEventCategory::Resize, Some(id), || {
                        self.handle_resize(id, size, size_type);
                    })
                },
                FromConstellation(ConstellationControlMsg::Viewport(id, rect)) => self
                    .profile_event(ScriptThreadEventCategory::SetViewport, Some(id), || {
                        self.handle_viewport(id, rect);
                    }),
                FromConstellation(ConstellationControlMsg::SetScrollState(id, scroll_state)) => {
                    self.profile_event(ScriptThreadEventCategory::SetScrollState, Some(id), || {
                        self.handle_set_scroll_state(id, &scroll_state);
                    })
                },
                FromConstellation(ConstellationControlMsg::TickAllAnimations(pipeline_id, _)) => {
                    if !animation_ticks.contains(&pipeline_id) {
                        animation_ticks.insert(pipeline_id);
                        sequential.push(event);
                    }
                },
                FromConstellation(ConstellationControlMsg::SendEvent(_, MouseMoveEvent(..))) => {
                    match mouse_move_event_index {
                        None => {
                            mouse_move_event_index = Some(sequential.len());
                            sequential.push(event);
                        },
                        Some(index) => sequential[index] = event,
                    }
                },
                FromScript(MainThreadScriptMsg::Inactive) => {
                    // An event came-in from a document that is not fully-active, it has been stored by the task-queue.
                    // Continue without adding it to "sequential".
                },
                FromConstellation(ConstellationControlMsg::ExitFullScreen(id)) => self
                    .profile_event(ScriptThreadEventCategory::ExitFullscreen, Some(id), || {
                        self.handle_exit_fullscreen(id);
                    }),
                _ => {
                    sequential.push(event);
                },
            }

            // If any of our input sources has an event pending, we'll perform another iteration
            // and check for more resize events. If there are no events pending, we'll move
            // on and execute the sequential non-resize events we've seen.
            match self.control_port.try_recv() {
                Err(_) => match self.task_queue.try_recv() {
                    Err(_) => match self.devtools_port.try_recv() {
                        Err(_) => match self.image_cache_port.try_recv() {
                            Err(_) => match &*self.webgpu_port.borrow() {
                                Some(p) => match p.try_recv() {
                                    Err(_) => break,
                                    Ok(ev) => event = FromWebGPUServer(ev),
                                },
                                None => break,
                            },
                            Ok(ev) => event = FromImageCache(ev),
                        },
                        Ok(ev) => event = FromDevtools(ev),
                    },
                    Ok(ev) => event = FromScript(ev),
                },
                Ok(ev) => event = FromConstellation(ev),
            }
        }

        // Step 11.10 from https://html.spec.whatwg.org/multipage/#event-loops.
        self.update_animations_and_send_events();

        // Process the gathered events.
        debug!("Processing events.");
        for msg in sequential {
            debug!("Processing event {:?}.", msg);

            let category = self.categorize_msg(&msg);
            let pipeline_id = self.message_to_pipeline(&msg);

            if self.closing.load(Ordering::SeqCst) {
                // If we've received the closed signal from the BHM, only handle exit messages.
                match msg {
                    FromConstellation(ConstellationControlMsg::ExitScriptThread) => {
                        self.handle_exit_script_thread_msg();
                        return false;
                    },
                    FromConstellation(ConstellationControlMsg::ExitPipeline(
                        pipeline_id,
                        discard_browsing_context,
                    )) => {
                        self.handle_exit_pipeline_msg(pipeline_id, discard_browsing_context);
                    },
                    _ => {},
                }
                continue;
            }

            let result = self.profile_event(category, pipeline_id, move || {
                match msg {
                    FromConstellation(ConstellationControlMsg::ExitScriptThread) => {
                        self.handle_exit_script_thread_msg();
                        return Some(false);
                    },
                    FromConstellation(inner_msg) => self.handle_msg_from_constellation(inner_msg),
                    FromScript(inner_msg) => self.handle_msg_from_script(inner_msg),
                    FromDevtools(inner_msg) => self.handle_msg_from_devtools(inner_msg),
                    FromImageCache(inner_msg) => self.handle_msg_from_image_cache(inner_msg),
                    FromWebGPUServer(inner_msg) => self.handle_msg_from_webgpu_server(inner_msg),
                }

                None
            });

            if let Some(retval) = result {
                return retval;
            }

            // https://html.spec.whatwg.org/multipage/#event-loop-processing-model step 6
            self.perform_a_microtask_checkpoint();
        }

        {
            // https://html.spec.whatwg.org/multipage/#the-end step 6
            let mut docs = self.docs_with_no_blocking_loads.borrow_mut();
            for document in docs.iter() {
                document.maybe_queue_document_completion();
            }
            docs.clear();
        }

        // https://html.spec.whatwg.org/multipage/#event-loop-processing-model step 7.12

        // Issue batched reflows on any pages that require it (e.g. if images loaded)
        // TODO(gw): In the future we could probably batch other types of reflows
        // into this loop too, but for now it's only images.
        debug!("Issuing batched reflows.");
        for (_, document) in self.documents.borrow().iter() {
            // Step 13
            if !document.is_fully_active() {
                continue;
            }
            let window = document.window();

            window
                .upcast::<GlobalScope>()
                .perform_a_dom_garbage_collection_checkpoint();

            let pending_reflows = window.get_pending_reflow_count();
            if pending_reflows > 0 {
                window.reflow(ReflowGoal::Full, ReflowReason::PendingReflow);
            } else {
                // Reflow currently happens when explicitly invoked by code that
                // knows the document could have been modified. This should really
                // be driven by the compositor on an as-needed basis instead, to
                // minimize unnecessary work.
                window.reflow(ReflowGoal::Full, ReflowReason::MissingExplicitReflow);
            }
        }

        true
    }

    // Perform step 11.10 from https://html.spec.whatwg.org/multipage/#event-loops.
    fn update_animations_and_send_events(&self) {
        for (_, document) in self.documents.borrow().iter() {
            document.animations().send_pending_events();
        }

        for (_, document) in self.documents.borrow().iter() {
            document.update_animation_timeline();
        }
    }

    fn categorize_msg(&self, msg: &MixedMessage) -> ScriptThreadEventCategory {
        match *msg {
            MixedMessage::FromConstellation(ref inner_msg) => match *inner_msg {
                ConstellationControlMsg::SendEvent(_, _) => ScriptThreadEventCategory::DomEvent,
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
            MixedMessage::FromWebGPUServer(_) => ScriptThreadEventCategory::WebGPUMsg,
        }
    }

    fn notify_activity_to_hang_monitor(&self, category: &ScriptThreadEventCategory) {
        let hang_annotation = match category {
            ScriptThreadEventCategory::AttachLayout => ScriptHangAnnotation::AttachLayout,
            ScriptThreadEventCategory::ConstellationMsg => ScriptHangAnnotation::ConstellationMsg,
            ScriptThreadEventCategory::DevtoolsMsg => ScriptHangAnnotation::DevtoolsMsg,
            ScriptThreadEventCategory::DocumentEvent => ScriptHangAnnotation::DocumentEvent,
            ScriptThreadEventCategory::DomEvent => ScriptHangAnnotation::DomEvent,
            ScriptThreadEventCategory::FileRead => ScriptHangAnnotation::FileRead,
            ScriptThreadEventCategory::FormPlannedNavigation => {
                ScriptHangAnnotation::FormPlannedNavigation
            },
            ScriptThreadEventCategory::HistoryEvent => ScriptHangAnnotation::HistoryEvent,
            ScriptThreadEventCategory::ImageCacheMsg => ScriptHangAnnotation::ImageCacheMsg,
            ScriptThreadEventCategory::InputEvent => ScriptHangAnnotation::InputEvent,
            ScriptThreadEventCategory::NetworkEvent => ScriptHangAnnotation::NetworkEvent,
            ScriptThreadEventCategory::Resize => ScriptHangAnnotation::Resize,
            ScriptThreadEventCategory::ScriptEvent => ScriptHangAnnotation::ScriptEvent,
            ScriptThreadEventCategory::SetScrollState => ScriptHangAnnotation::SetScrollState,
            ScriptThreadEventCategory::SetViewport => ScriptHangAnnotation::SetViewport,
            ScriptThreadEventCategory::StylesheetLoad => ScriptHangAnnotation::StylesheetLoad,
            ScriptThreadEventCategory::TimerEvent => ScriptHangAnnotation::TimerEvent,
            ScriptThreadEventCategory::UpdateReplacedElement => {
                ScriptHangAnnotation::UpdateReplacedElement
            },
            ScriptThreadEventCategory::WebSocketEvent => ScriptHangAnnotation::WebSocketEvent,
            ScriptThreadEventCategory::WorkerEvent => ScriptHangAnnotation::WorkerEvent,
            ScriptThreadEventCategory::WorkletEvent => ScriptHangAnnotation::WorkletEvent,
            ScriptThreadEventCategory::ServiceWorkerEvent => {
                ScriptHangAnnotation::ServiceWorkerEvent
            },
            ScriptThreadEventCategory::EnterFullscreen => ScriptHangAnnotation::EnterFullscreen,
            ScriptThreadEventCategory::ExitFullscreen => ScriptHangAnnotation::ExitFullscreen,
            ScriptThreadEventCategory::PerformanceTimelineTask => {
                ScriptHangAnnotation::PerformanceTimelineTask
            },
            ScriptThreadEventCategory::PortMessage => ScriptHangAnnotation::PortMessage,
            ScriptThreadEventCategory::WebGPUMsg => ScriptHangAnnotation::WebGPUMsg,
        };
        self.background_hang_monitor
            .notify_activity(HangAnnotation::Script(hang_annotation));
    }

    fn message_to_pipeline(&self, msg: &MixedMessage) -> Option<PipelineId> {
        use script_traits::ConstellationControlMsg::*;
        match *msg {
            MixedMessage::FromConstellation(ref inner_msg) => match *inner_msg {
                StopDelayingLoadEventsMode(id) => Some(id),
                NavigationResponse(id, _) => Some(id),
                AttachLayout(ref new_layout_info) => Some(new_layout_info.new_pipeline_id),
                Resize(id, ..) => Some(id),
                ResizeInactive(id, ..) => Some(id),
                UnloadDocument(id) => Some(id),
                ExitPipeline(id, ..) => Some(id),
                ExitScriptThread => None,
                SendEvent(id, ..) => Some(id),
                Viewport(id, ..) => Some(id),
                SetScrollState(id, ..) => Some(id),
                GetTitle(id) => Some(id),
                SetDocumentActivity(id, ..) => Some(id),
                ChangeFrameVisibilityStatus(id, ..) => Some(id),
                NotifyVisibilityChange(id, ..) => Some(id),
                NavigateIframe(id, ..) => Some(id),
                PostMessage { target: id, .. } => Some(id),
                UpdatePipelineId(_, _, _, id, _) => Some(id),
                UpdateHistoryState(id, ..) => Some(id),
                RemoveHistoryStates(id, ..) => Some(id),
                FocusIFrame(id, ..) => Some(id),
                WebDriverScriptCommand(id, ..) => Some(id),
                TickAllAnimations(id, ..) => Some(id),
                WebFontLoaded(id) => Some(id),
                DispatchIFrameLoadEvent {
                    target: _,
                    parent: id,
                    child: _,
                } => Some(id),
                DispatchStorageEvent(id, ..) => Some(id),
                ReportCSSError(id, ..) => Some(id),
                Reload(id, ..) => Some(id),
                PaintMetric(..) => None,
                ExitFullScreen(id, ..) => Some(id),
                MediaSessionAction(..) => None,
                SetWebGPUPort(..) => None,
            },
            MixedMessage::FromDevtools(_) => None,
            MixedMessage::FromScript(ref inner_msg) => match *inner_msg {
                MainThreadScriptMsg::Common(CommonScriptMsg::Task(_, _, pipeline_id, _)) => {
                    pipeline_id
                },
                MainThreadScriptMsg::Common(CommonScriptMsg::CollectReports(_)) => None,
                MainThreadScriptMsg::WorkletLoaded(pipeline_id) => Some(pipeline_id),
                MainThreadScriptMsg::RegisterPaintWorklet { pipeline_id, .. } => Some(pipeline_id),
                MainThreadScriptMsg::Inactive => None,
                MainThreadScriptMsg::WakeUp => None,
            },
            MixedMessage::FromImageCache((pipeline_id, _)) => Some(pipeline_id),
            MixedMessage::FromWebGPUServer(..) => None,
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
        self.notify_activity_to_hang_monitor(&category);
        let start = precise_time_ns();
        let value = if self.profile_script_events {
            let profiler_cat = match category {
                ScriptThreadEventCategory::AttachLayout => ProfilerCategory::ScriptAttachLayout,
                ScriptThreadEventCategory::ConstellationMsg => {
                    ProfilerCategory::ScriptConstellationMsg
                },
                ScriptThreadEventCategory::DevtoolsMsg => ProfilerCategory::ScriptDevtoolsMsg,
                ScriptThreadEventCategory::DocumentEvent => ProfilerCategory::ScriptDocumentEvent,
                ScriptThreadEventCategory::DomEvent => ProfilerCategory::ScriptDomEvent,
                ScriptThreadEventCategory::FileRead => ProfilerCategory::ScriptFileRead,
                ScriptThreadEventCategory::FormPlannedNavigation => {
                    ProfilerCategory::ScriptPlannedNavigation
                },
                ScriptThreadEventCategory::HistoryEvent => ProfilerCategory::ScriptHistoryEvent,
                ScriptThreadEventCategory::ImageCacheMsg => ProfilerCategory::ScriptImageCacheMsg,
                ScriptThreadEventCategory::InputEvent => ProfilerCategory::ScriptInputEvent,
                ScriptThreadEventCategory::NetworkEvent => ProfilerCategory::ScriptNetworkEvent,
                ScriptThreadEventCategory::PortMessage => ProfilerCategory::ScriptPortMessage,
                ScriptThreadEventCategory::Resize => ProfilerCategory::ScriptResize,
                ScriptThreadEventCategory::ScriptEvent => ProfilerCategory::ScriptEvent,
                ScriptThreadEventCategory::SetScrollState => ProfilerCategory::ScriptSetScrollState,
                ScriptThreadEventCategory::UpdateReplacedElement => {
                    ProfilerCategory::ScriptUpdateReplacedElement
                },
                ScriptThreadEventCategory::StylesheetLoad => ProfilerCategory::ScriptStylesheetLoad,
                ScriptThreadEventCategory::SetViewport => ProfilerCategory::ScriptSetViewport,
                ScriptThreadEventCategory::TimerEvent => ProfilerCategory::ScriptTimerEvent,
                ScriptThreadEventCategory::WebSocketEvent => ProfilerCategory::ScriptWebSocketEvent,
                ScriptThreadEventCategory::WorkerEvent => ProfilerCategory::ScriptWorkerEvent,
                ScriptThreadEventCategory::WorkletEvent => ProfilerCategory::ScriptWorkletEvent,
                ScriptThreadEventCategory::ServiceWorkerEvent => {
                    ProfilerCategory::ScriptServiceWorkerEvent
                },
                ScriptThreadEventCategory::EnterFullscreen => {
                    ProfilerCategory::ScriptEnterFullscreen
                },
                ScriptThreadEventCategory::ExitFullscreen => ProfilerCategory::ScriptExitFullscreen,
                ScriptThreadEventCategory::PerformanceTimelineTask => {
                    ProfilerCategory::ScriptPerformanceEvent
                },
                ScriptThreadEventCategory::WebGPUMsg => ProfilerCategory::ScriptWebGPUMsg,
            };
            profile(profiler_cat, None, self.time_profiler_chan.clone(), f)
        } else {
            f()
        };
        let end = precise_time_ns();
        for (doc_id, doc) in self.documents.borrow().iter() {
            if let Some(pipeline_id) = pipeline_id {
                if pipeline_id == doc_id && end - start > MAX_TASK_NS {
                    if self.print_pwm {
                        println!(
                            "Task took longer than max allowed ({:?}) {:?}",
                            category,
                            end - start
                        );
                    }
                    doc.start_tti();
                }
            }
            doc.record_tti_if_necessary();
        }
        value
    }

    fn handle_msg_from_constellation(&self, msg: ConstellationControlMsg) {
        match msg {
            ConstellationControlMsg::StopDelayingLoadEventsMode(pipeline_id) => {
                self.handle_stop_delaying_load_events_mode(pipeline_id)
            },
            ConstellationControlMsg::NavigationResponse(id, fetch_data) => {
                match fetch_data {
                    FetchResponseMsg::ProcessResponse(metadata) => {
                        self.handle_fetch_metadata(id, metadata)
                    },
                    FetchResponseMsg::ProcessResponseChunk(chunk) => {
                        self.handle_fetch_chunk(id, chunk)
                    },
                    FetchResponseMsg::ProcessResponseEOF(eof) => self.handle_fetch_eof(id, eof),
                    _ => unreachable!(),
                };
            },
            ConstellationControlMsg::NavigateIframe(
                parent_pipeline_id,
                browsing_context_id,
                load_data,
                replace,
            ) => self.handle_navigate_iframe(
                parent_pipeline_id,
                browsing_context_id,
                load_data,
                replace,
            ),
            ConstellationControlMsg::UnloadDocument(pipeline_id) => {
                self.handle_unload_document(pipeline_id)
            },
            ConstellationControlMsg::SendEvent(id, event) => self.handle_event(id, event),
            ConstellationControlMsg::ResizeInactive(id, new_size) => {
                self.handle_resize_inactive_msg(id, new_size)
            },
            ConstellationControlMsg::GetTitle(pipeline_id) => {
                self.handle_get_title_msg(pipeline_id)
            },
            ConstellationControlMsg::SetDocumentActivity(pipeline_id, activity) => {
                self.handle_set_document_activity_msg(pipeline_id, activity)
            },
            ConstellationControlMsg::ChangeFrameVisibilityStatus(pipeline_id, visible) => {
                self.handle_visibility_change_msg(pipeline_id, visible)
            },
            ConstellationControlMsg::NotifyVisibilityChange(
                parent_pipeline_id,
                browsing_context_id,
                visible,
            ) => self.handle_visibility_change_complete_msg(
                parent_pipeline_id,
                browsing_context_id,
                visible,
            ),
            ConstellationControlMsg::PostMessage {
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
            ConstellationControlMsg::UpdatePipelineId(
                parent_pipeline_id,
                browsing_context_id,
                top_level_browsing_context_id,
                new_pipeline_id,
                reason,
            ) => self.handle_update_pipeline_id(
                parent_pipeline_id,
                browsing_context_id,
                top_level_browsing_context_id,
                new_pipeline_id,
                reason,
            ),
            ConstellationControlMsg::UpdateHistoryState(pipeline_id, history_state_id, url) => {
                self.handle_update_history_state_msg(pipeline_id, history_state_id, url)
            },
            ConstellationControlMsg::RemoveHistoryStates(pipeline_id, history_states) => {
                self.handle_remove_history_states(pipeline_id, history_states)
            },
            ConstellationControlMsg::FocusIFrame(parent_pipeline_id, frame_id) => {
                self.handle_focus_iframe_msg(parent_pipeline_id, frame_id)
            },
            ConstellationControlMsg::WebDriverScriptCommand(pipeline_id, msg) => {
                self.handle_webdriver_msg(pipeline_id, msg)
            },
            ConstellationControlMsg::TickAllAnimations(pipeline_id, tick_type) => {
                self.handle_tick_all_animations(pipeline_id, tick_type)
            },
            ConstellationControlMsg::WebFontLoaded(pipeline_id) => {
                self.handle_web_font_loaded(pipeline_id)
            },
            ConstellationControlMsg::DispatchIFrameLoadEvent {
                target: browsing_context_id,
                parent: parent_id,
                child: child_id,
            } => self.handle_iframe_load_event(parent_id, browsing_context_id, child_id),
            ConstellationControlMsg::DispatchStorageEvent(
                pipeline_id,
                storage,
                url,
                key,
                old_value,
                new_value,
            ) => self.handle_storage_event(pipeline_id, storage, url, key, old_value, new_value),
            ConstellationControlMsg::ReportCSSError(pipeline_id, filename, line, column, msg) => {
                self.handle_css_error_reporting(pipeline_id, filename, line, column, msg)
            },
            ConstellationControlMsg::Reload(pipeline_id) => self.handle_reload(pipeline_id),
            ConstellationControlMsg::ExitPipeline(pipeline_id, discard_browsing_context) => {
                self.handle_exit_pipeline_msg(pipeline_id, discard_browsing_context)
            },
            ConstellationControlMsg::PaintMetric(pipeline_id, metric_type, metric_value) => {
                self.handle_paint_metric(pipeline_id, metric_type, metric_value)
            },
            ConstellationControlMsg::MediaSessionAction(pipeline_id, action) => {
                self.handle_media_session_action(pipeline_id, action)
            },
            ConstellationControlMsg::SetWebGPUPort(port) => {
                if self.webgpu_port.borrow().is_some() {
                    warn!("WebGPU port already exists for this content process");
                } else {
                    let p = ROUTER.route_ipc_receiver_to_new_crossbeam_receiver(port);
                    *self.webgpu_port.borrow_mut() = Some(p);
                }
            },
            msg @ ConstellationControlMsg::AttachLayout(..) |
            msg @ ConstellationControlMsg::Viewport(..) |
            msg @ ConstellationControlMsg::SetScrollState(..) |
            msg @ ConstellationControlMsg::Resize(..) |
            msg @ ConstellationControlMsg::ExitFullScreen(..) |
            msg @ ConstellationControlMsg::ExitScriptThread => {
                panic!("should have handled {:?} already", msg)
            },
        }
    }

    fn handle_msg_from_webgpu_server(&self, msg: WebGPUMsg) {
        match msg {
            WebGPUMsg::FreeAdapter(id) => self.gpu_id_hub.lock().kill_adapter_id(id),
            WebGPUMsg::FreeDevice(id) => self.gpu_id_hub.lock().kill_device_id(id),
            WebGPUMsg::FreeBuffer(id) => self.gpu_id_hub.lock().kill_buffer_id(id),
            WebGPUMsg::FreePipelineLayout(id) => self.gpu_id_hub.lock().kill_pipeline_layout_id(id),
            WebGPUMsg::FreeComputePipeline(id) => {
                self.gpu_id_hub.lock().kill_compute_pipeline_id(id)
            },
            WebGPUMsg::FreeBindGroup(id) => self.gpu_id_hub.lock().kill_bind_group_id(id),
            WebGPUMsg::FreeBindGroupLayout(id) => {
                self.gpu_id_hub.lock().kill_bind_group_layout_id(id)
            },
            WebGPUMsg::FreeCommandBuffer(id) => self.gpu_id_hub.lock().kill_command_buffer_id(id),
            WebGPUMsg::FreeSampler(id) => self.gpu_id_hub.lock().kill_sampler_id(id),
            WebGPUMsg::FreeShaderModule(id) => self.gpu_id_hub.lock().kill_shader_module_id(id),
            WebGPUMsg::FreeRenderBundle(id) => self.gpu_id_hub.lock().kill_render_bundle_id(id),
            WebGPUMsg::FreeRenderPipeline(id) => self.gpu_id_hub.lock().kill_render_pipeline_id(id),
            WebGPUMsg::FreeTexture(id) => self.gpu_id_hub.lock().kill_texture_id(id),
            WebGPUMsg::FreeTextureView(id) => self.gpu_id_hub.lock().kill_texture_view_id(id),
            WebGPUMsg::Exit => *self.webgpu_port.borrow_mut() = None,
            WebGPUMsg::WebGPUOpResult {
                device,
                scope_id,
                pipeline_id,
                result,
            } => {
                let global = self.documents.borrow().find_global(pipeline_id).unwrap();
                global.handle_wgpu_msg(device, scope_id, result);
            },
            WebGPUMsg::CleanDevice {
                pipeline_id,
                device,
            } => {
                let global = self.documents.borrow().find_global(pipeline_id).unwrap();
                global.remove_gpu_device(device);
            },
            _ => {},
        }
    }

    fn handle_msg_from_script(&self, msg: MainThreadScriptMsg) {
        match msg {
            MainThreadScriptMsg::Common(CommonScriptMsg::Task(_, task, _, _)) => task.run_box(),
            MainThreadScriptMsg::Common(CommonScriptMsg::CollectReports(chan)) => {
                self.collect_reports(chan)
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

    fn handle_msg_from_devtools(&self, msg: DevtoolScriptControlMsg) {
        let documents = self.documents.borrow();
        match msg {
            DevtoolScriptControlMsg::EvaluateJS(id, s, reply) => match documents.find_window(id) {
                Some(window) => devtools::handle_evaluate_js(window.upcast(), s, reply),
                None => return warn!("Message sent to closed pipeline {}.", id),
            },
            DevtoolScriptControlMsg::GetRootNode(id, reply) => {
                devtools::handle_get_root_node(&*documents, id, reply)
            },
            DevtoolScriptControlMsg::GetDocumentElement(id, reply) => {
                devtools::handle_get_document_element(&*documents, id, reply)
            },
            DevtoolScriptControlMsg::GetChildren(id, node_id, reply) => {
                devtools::handle_get_children(&*documents, id, node_id, reply)
            },
            DevtoolScriptControlMsg::GetLayout(id, node_id, reply) => {
                devtools::handle_get_layout(&*documents, id, node_id, reply)
            },
            DevtoolScriptControlMsg::ModifyAttribute(id, node_id, modifications) => {
                devtools::handle_modify_attribute(&*documents, id, node_id, modifications)
            },
            DevtoolScriptControlMsg::WantsLiveNotifications(id, to_send) => match documents
                .find_window(id)
            {
                Some(window) => devtools::handle_wants_live_notifications(window.upcast(), to_send),
                None => return warn!("Message sent to closed pipeline {}.", id),
            },
            DevtoolScriptControlMsg::SetTimelineMarkers(id, marker_types, reply) => {
                devtools::handle_set_timeline_markers(&*documents, id, marker_types, reply)
            },
            DevtoolScriptControlMsg::DropTimelineMarkers(id, marker_types) => {
                devtools::handle_drop_timeline_markers(&*documents, id, marker_types)
            },
            DevtoolScriptControlMsg::RequestAnimationFrame(id, name) => {
                devtools::handle_request_animation_frame(&*documents, id, name)
            },
            DevtoolScriptControlMsg::Reload(id) => devtools::handle_reload(&*documents, id),
        }
    }

    fn handle_msg_from_image_cache(&self, (id, response): (PipelineId, PendingImageResponse)) {
        let window = self.documents.borrow().find_window(id);
        if let Some(ref window) = window {
            window.pending_image_notification(response);
        }
    }

    fn handle_webdriver_msg(&self, pipeline_id: PipelineId, msg: WebDriverScriptCommand) {
        // https://github.com/servo/servo/issues/23535
        // These two messages need different treatment since the JS script might mutate
        // `self.documents`, which would conflict with the immutable borrow of it that
        // occurs for the rest of the messages
        match msg {
            WebDriverScriptCommand::ExecuteScript(script, reply) => {
                let window = self.documents.borrow().find_window(pipeline_id);
                return webdriver_handlers::handle_execute_script(window, script, reply);
            },
            WebDriverScriptCommand::ExecuteAsyncScript(script, reply) => {
                let window = self.documents.borrow().find_window(pipeline_id);
                return webdriver_handlers::handle_execute_async_script(window, script, reply);
            },
            _ => (),
        }

        let documents = self.documents.borrow();
        match msg {
            WebDriverScriptCommand::AddCookie(params, reply) => {
                webdriver_handlers::handle_add_cookie(&*documents, pipeline_id, params, reply)
            },
            WebDriverScriptCommand::DeleteCookies(reply) => {
                webdriver_handlers::handle_delete_cookies(&*documents, pipeline_id, reply)
            },
            WebDriverScriptCommand::FindElementCSS(selector, reply) => {
                webdriver_handlers::handle_find_element_css(
                    &*documents,
                    pipeline_id,
                    selector,
                    reply,
                )
            },
            WebDriverScriptCommand::FindElementLinkText(selector, partial, reply) => {
                webdriver_handlers::handle_find_element_link_text(
                    &*documents,
                    pipeline_id,
                    selector,
                    partial,
                    reply,
                )
            },
            WebDriverScriptCommand::FindElementTagName(selector, reply) => {
                webdriver_handlers::handle_find_element_tag_name(
                    &*documents,
                    pipeline_id,
                    selector,
                    reply,
                )
            },
            WebDriverScriptCommand::FindElementsCSS(selector, reply) => {
                webdriver_handlers::handle_find_elements_css(
                    &*documents,
                    pipeline_id,
                    selector,
                    reply,
                )
            },
            WebDriverScriptCommand::FindElementsLinkText(selector, partial, reply) => {
                webdriver_handlers::handle_find_elements_link_text(
                    &*documents,
                    pipeline_id,
                    selector,
                    partial,
                    reply,
                )
            },
            WebDriverScriptCommand::FindElementsTagName(selector, reply) => {
                webdriver_handlers::handle_find_elements_tag_name(
                    &*documents,
                    pipeline_id,
                    selector,
                    reply,
                )
            },
            WebDriverScriptCommand::FindElementElementCSS(selector, element_id, reply) => {
                webdriver_handlers::handle_find_element_element_css(
                    &*documents,
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
                &*documents,
                pipeline_id,
                element_id,
                selector,
                partial,
                reply,
            ),
            WebDriverScriptCommand::FindElementElementTagName(selector, element_id, reply) => {
                webdriver_handlers::handle_find_element_element_tag_name(
                    &*documents,
                    pipeline_id,
                    element_id,
                    selector,
                    reply,
                )
            },
            WebDriverScriptCommand::FindElementElementsCSS(selector, element_id, reply) => {
                webdriver_handlers::handle_find_element_elements_css(
                    &*documents,
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
                &*documents,
                pipeline_id,
                element_id,
                selector,
                partial,
                reply,
            ),
            WebDriverScriptCommand::FindElementElementsTagName(selector, element_id, reply) => {
                webdriver_handlers::handle_find_element_elements_tag_name(
                    &*documents,
                    pipeline_id,
                    element_id,
                    selector,
                    reply,
                )
            },
            WebDriverScriptCommand::FocusElement(element_id, reply) => {
                webdriver_handlers::handle_focus_element(
                    &*documents,
                    pipeline_id,
                    element_id,
                    reply,
                )
            },
            WebDriverScriptCommand::ElementClick(element_id, reply) => {
                webdriver_handlers::handle_element_click(
                    &*documents,
                    pipeline_id,
                    element_id,
                    reply,
                )
            },
            WebDriverScriptCommand::GetActiveElement(reply) => {
                webdriver_handlers::handle_get_active_element(&*documents, pipeline_id, reply)
            },
            WebDriverScriptCommand::GetPageSource(reply) => {
                webdriver_handlers::handle_get_page_source(&*documents, pipeline_id, reply)
            },
            WebDriverScriptCommand::GetCookies(reply) => {
                webdriver_handlers::handle_get_cookies(&*documents, pipeline_id, reply)
            },
            WebDriverScriptCommand::GetCookie(name, reply) => {
                webdriver_handlers::handle_get_cookie(&*documents, pipeline_id, name, reply)
            },
            WebDriverScriptCommand::GetElementTagName(node_id, reply) => {
                webdriver_handlers::handle_get_name(&*documents, pipeline_id, node_id, reply)
            },
            WebDriverScriptCommand::GetElementAttribute(node_id, name, reply) => {
                webdriver_handlers::handle_get_attribute(
                    &*documents,
                    pipeline_id,
                    node_id,
                    name,
                    reply,
                )
            },
            WebDriverScriptCommand::GetElementProperty(node_id, name, reply) => {
                webdriver_handlers::handle_get_property(
                    &*documents,
                    pipeline_id,
                    node_id,
                    name,
                    reply,
                )
            },
            WebDriverScriptCommand::GetElementCSS(node_id, name, reply) => {
                webdriver_handlers::handle_get_css(&*documents, pipeline_id, node_id, name, reply)
            },
            WebDriverScriptCommand::GetElementRect(node_id, reply) => {
                webdriver_handlers::handle_get_rect(&*documents, pipeline_id, node_id, reply)
            },
            WebDriverScriptCommand::GetBoundingClientRect(node_id, reply) => {
                webdriver_handlers::handle_get_bounding_client_rect(
                    &*documents,
                    pipeline_id,
                    node_id,
                    reply,
                )
            },
            WebDriverScriptCommand::GetElementText(node_id, reply) => {
                webdriver_handlers::handle_get_text(&*documents, pipeline_id, node_id, reply)
            },
            WebDriverScriptCommand::GetElementInViewCenterPoint(node_id, reply) => {
                webdriver_handlers::handle_get_element_in_view_center_point(
                    &*documents,
                    pipeline_id,
                    node_id,
                    reply,
                )
            },
            WebDriverScriptCommand::GetBrowsingContextId(webdriver_frame_id, reply) => {
                webdriver_handlers::handle_get_browsing_context_id(
                    &*documents,
                    pipeline_id,
                    webdriver_frame_id,
                    reply,
                )
            },
            WebDriverScriptCommand::GetUrl(reply) => {
                webdriver_handlers::handle_get_url(&*documents, pipeline_id, reply)
            },
            WebDriverScriptCommand::IsEnabled(element_id, reply) => {
                webdriver_handlers::handle_is_enabled(&*documents, pipeline_id, element_id, reply)
            },
            WebDriverScriptCommand::IsSelected(element_id, reply) => {
                webdriver_handlers::handle_is_selected(&*documents, pipeline_id, element_id, reply)
            },
            WebDriverScriptCommand::GetTitle(reply) => {
                webdriver_handlers::handle_get_title(&*documents, pipeline_id, reply)
            },
            _ => (),
        }
    }

    fn handle_resize(&self, id: PipelineId, size: WindowSizeData, size_type: WindowSizeType) {
        let window = self.documents.borrow().find_window(id);
        if let Some(ref window) = window {
            window.set_resize_event(size, size_type);
            return;
        }
        let mut loads = self.incomplete_loads.borrow_mut();
        if let Some(ref mut load) = loads.iter_mut().find(|load| load.pipeline_id == id) {
            load.window_size = size;
            return;
        }
        warn!("resize sent to nonexistent pipeline");
    }

    // exit_fullscreen creates a new JS promise object, so we need to have entered a realm
    fn handle_exit_fullscreen(&self, id: PipelineId) {
        let document = self.documents.borrow().find_document(id);
        if let Some(document) = document {
            let _ac = enter_realm(&*document);
            document.exit_fullscreen();
            return;
        }
    }

    fn handle_viewport(&self, id: PipelineId, rect: Rect<f32>) {
        let document = self.documents.borrow().find_document(id);
        if let Some(document) = document {
            if document.window().set_page_clip_rect_with_new_viewport(rect) {
                self.rebuild_and_force_reflow(&document, ReflowReason::Viewport);
            }
            return;
        }
        let loads = self.incomplete_loads.borrow();
        if loads.iter().any(|load| load.pipeline_id == id) {
            return;
        }
        warn!("Page rect message sent to nonexistent pipeline");
    }

    fn handle_set_scroll_state(
        &self,
        id: PipelineId,
        scroll_states: &[(UntrustedNodeAddress, Vector2D<f32, LayoutPixel>)],
    ) {
        let window = match self.documents.borrow().find_window(id) {
            Some(window) => window,
            None => {
                return warn!(
                    "Set scroll state message sent to nonexistent pipeline: {:?}",
                    id
                );
            },
        };

        let mut scroll_offsets = HashMap::new();
        for &(node_address, ref scroll_offset) in scroll_states {
            if node_address == UntrustedNodeAddress(ptr::null()) {
                window.update_viewport_for_scroll(-scroll_offset.x, -scroll_offset.y);
            } else {
                scroll_offsets.insert(OpaqueNode(node_address.0 as usize), -*scroll_offset);
            }
        }
        window.set_scroll_offsets(scroll_offsets)
    }

    fn handle_new_layout(&self, new_layout_info: NewLayoutInfo, origin: MutableOrigin) {
        let NewLayoutInfo {
            parent_info,
            new_pipeline_id,
            browsing_context_id,
            top_level_browsing_context_id,
            opener,
            load_data,
            window_size,
            pipeline_port,
        } = new_layout_info;

        let layout_pair = unbounded();
        let layout_chan = layout_pair.0.clone();

        let layout_is_busy = Arc::new(AtomicBool::new(false));

        let msg = message::Msg::CreateLayoutThread(LayoutThreadInit {
            id: new_pipeline_id,
            url: load_data.url.clone(),
            is_parent: false,
            layout_pair: layout_pair,
            pipeline_port: pipeline_port,
            background_hang_monitor_register: self.background_hang_monitor_register.clone(),
            constellation_chan: self.layout_to_constellation_chan.clone(),
            script_chan: self.control_chan.clone(),
            image_cache: self.image_cache.clone(),
            paint_time_metrics: PaintTimeMetrics::new(
                new_pipeline_id,
                self.time_profiler_chan.clone(),
                self.layout_to_constellation_chan.clone(),
                self.control_chan.clone(),
                load_data.url.clone(),
            ),
            layout_is_busy: layout_is_busy.clone(),
            window_size,
        });

        // Pick a layout thread, any layout thread
        let current_layout_chan: Option<Sender<Msg>> = self
            .documents
            .borrow()
            .iter()
            .next()
            .and_then(|(_, document)| document.window().layout_chan().cloned())
            .or_else(|| {
                self.incomplete_loads
                    .borrow()
                    .first()
                    .map(|load| load.layout_chan.clone())
            });

        match current_layout_chan {
            None => panic!("Layout attached to empty script thread."),
            // Tell the layout thread factory to actually spawn the thread.
            Some(layout_chan) => layout_chan.send(msg).unwrap(),
        };

        // Kick off the fetch for the new resource.
        let new_load = InProgressLoad::new(
            new_pipeline_id,
            browsing_context_id,
            top_level_browsing_context_id,
            parent_info,
            opener,
            layout_chan,
            window_size,
            load_data.url.clone(),
            origin,
            layout_is_busy.clone(),
            load_data.inherited_secure_context.clone(),
        );
        if load_data.url.as_str() == "about:blank" {
            self.start_page_load_about_blank(new_load, load_data.js_eval_result);
        } else if load_data.url.as_str() == "about:srcdoc" {
            self.page_load_about_srcdoc(new_load, load_data);
        } else {
            self.pre_page_load(new_load, load_data);
        }
    }

    fn collect_reports(&self, reports_chan: ReportsChan) {
        let documents = self.documents.borrow();
        let urls = itertools::join(documents.iter().map(|(_, d)| d.url().to_string()), ", ");
        let path_seg = format!("url({})", urls);

        let mut reports = vec![];
        reports.extend(get_reports(*self.get_cx(), path_seg));
        reports_chan.send(reports);
    }

    /// Updates iframe element after a change in visibility
    fn handle_visibility_change_complete_msg(
        &self,
        parent_pipeline_id: PipelineId,
        browsing_context_id: BrowsingContextId,
        visible: bool,
    ) {
        let iframe = self
            .documents
            .borrow()
            .find_iframe(parent_pipeline_id, browsing_context_id);
        if let Some(iframe) = iframe {
            iframe.change_visibility_status(visible);
        }
    }

    /// Handle visibility change message
    fn handle_visibility_change_msg(&self, id: PipelineId, visible: bool) {
        // Separate message sent since parent script thread could be different (Iframe of different
        // domain)
        self.script_sender
            .send((id, ScriptMsg::VisibilityChangeComplete(visible)))
            .unwrap();

        let window = self.documents.borrow().find_window(id);
        match window {
            Some(window) => {
                window.alter_resource_utilization(visible);
                return;
            },
            None => {
                let mut loads = self.incomplete_loads.borrow_mut();
                if let Some(ref mut load) = loads.iter_mut().find(|load| load.pipeline_id == id) {
                    load.is_visible = visible;
                    return;
                }
            },
        }

        warn!("change visibility message sent to nonexistent pipeline");
    }

    /// Handles activity change message
    fn handle_set_document_activity_msg(&self, id: PipelineId, activity: DocumentActivity) {
        debug!(
            "Setting activity of {} to be {:?} in {:?}.",
            id,
            activity,
            thread::current().name()
        );
        let document = self.documents.borrow().find_document(id);
        if let Some(document) = document {
            document.set_activity(activity);
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
    ) {
        let doc = self
            .documents
            .borrow()
            .find_document(parent_pipeline_id)
            .unwrap();
        let frame_element = doc.find_iframe(browsing_context_id);

        if let Some(ref frame_element) = frame_element {
            doc.request_focus(Some(frame_element.upcast()), FocusType::Parent);
        }
    }

    fn handle_post_message_msg(
        &self,
        pipeline_id: PipelineId,
        source_pipeline_id: PipelineId,
        source_browsing_context: TopLevelBrowsingContextId,
        origin: Option<ImmutableOrigin>,
        source_origin: ImmutableOrigin,
        data: StructuredSerializedData,
    ) {
        let window = self.documents.borrow().find_window(pipeline_id);
        match window {
            None => return warn!("postMessage after target pipeline {} closed.", pipeline_id),
            Some(window) => {
                // FIXME: synchronously talks to constellation.
                // send the required info as part of postmessage instead.
                let source = match self.remote_window_proxy(
                    &*window.global(),
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
                window.post_message(origin, source_origin, &*source, data)
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

    fn handle_unload_document(&self, pipeline_id: PipelineId) {
        let document = self.documents.borrow().find_document(pipeline_id);
        if let Some(document) = document {
            document.unload(false);
        }
    }

    fn handle_update_pipeline_id(
        &self,
        parent_pipeline_id: PipelineId,
        browsing_context_id: BrowsingContextId,
        top_level_browsing_context_id: TopLevelBrowsingContextId,
        new_pipeline_id: PipelineId,
        reason: UpdatePipelineIdReason,
    ) {
        let frame_element = self
            .documents
            .borrow()
            .find_iframe(parent_pipeline_id, browsing_context_id);
        if let Some(frame_element) = frame_element {
            frame_element.update_pipeline_id(new_pipeline_id, reason);
        }

        if let Some(window) = self.documents.borrow().find_window(new_pipeline_id) {
            // Ensure that the state of any local window proxies accurately reflects
            // the new pipeline.
            let _ = self.local_window_proxy(
                &*window,
                browsing_context_id,
                top_level_browsing_context_id,
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
    ) {
        let window = self.documents.borrow().find_window(pipeline_id);
        match window {
            None => {
                return warn!(
                    "update history state after pipeline {} closed.",
                    pipeline_id
                );
            },
            Some(window) => window.History().activate_state(history_state_id, url),
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
                return warn!(
                    "update history state after pipeline {} closed.",
                    pipeline_id
                );
            },
            Some(window) => window.History().remove_states(history_states),
        }
    }

    /// Window was resized, but this script was not active, so don't reflow yet
    fn handle_resize_inactive_msg(&self, id: PipelineId, new_size: WindowSizeData) {
        let window = self.documents.borrow().find_window(id)
            .expect("ScriptThread: received a resize msg for a pipeline not in this script thread. This is a bug.");
        window.set_window_size(new_size);
    }

    /// We have received notification that the response associated with a load has completed.
    /// Kick off the document and frame tree creation process using the result.
    fn handle_page_headers_available(
        &self,
        id: &PipelineId,
        metadata: Option<Metadata>,
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
                match metadata {
                    Some(Metadata {
                        status: Some((204..=205, _)),
                        ..
                    }) => {
                        // If we have an existing window that is being navigated:
                        if let Some(window) = self.documents.borrow().find_window(id.clone()) {
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
                        self.script_sender
                            .send((id.clone(), ScriptMsg::AbortLoadUrl))
                            .unwrap();
                        return None;
                    },
                    _ => (),
                };

                let load = self.incomplete_loads.borrow_mut().remove(idx);
                metadata.map(|meta| self.load(meta, load))
            },
            None => {
                assert!(self.closed_pipelines.borrow().contains(id));
                None
            },
        }
    }

    pub fn dom_manipulation_task_source(
        &self,
        pipeline_id: PipelineId,
    ) -> DOMManipulationTaskSource {
        DOMManipulationTaskSource(self.dom_manipulation_task_sender.clone(), pipeline_id)
    }

    pub fn media_element_task_source(&self, pipeline_id: PipelineId) -> MediaElementTaskSource {
        MediaElementTaskSource(self.media_element_task_sender.clone(), pipeline_id)
    }

    pub fn performance_timeline_task_source(
        &self,
        pipeline_id: PipelineId,
    ) -> PerformanceTimelineTaskSource {
        PerformanceTimelineTaskSource(self.performance_timeline_task_sender.clone(), pipeline_id)
    }

    pub fn history_traversal_task_source(
        &self,
        pipeline_id: PipelineId,
    ) -> HistoryTraversalTaskSource {
        HistoryTraversalTaskSource(self.history_traversal_task_sender.clone(), pipeline_id)
    }

    pub fn user_interaction_task_source(
        &self,
        pipeline_id: PipelineId,
    ) -> UserInteractionTaskSource {
        UserInteractionTaskSource(self.user_interaction_task_sender.clone(), pipeline_id)
    }

    pub fn networking_task_source(&self, pipeline_id: PipelineId) -> NetworkingTaskSource {
        NetworkingTaskSource(self.networking_task_sender.clone(), pipeline_id)
    }

    pub fn port_message_queue(&self, pipeline_id: PipelineId) -> PortMessageQueue {
        PortMessageQueue(self.port_message_sender.clone(), pipeline_id)
    }

    pub fn file_reading_task_source(&self, pipeline_id: PipelineId) -> FileReadingTaskSource {
        FileReadingTaskSource(self.file_reading_task_sender.clone(), pipeline_id)
    }

    pub fn remote_event_task_source(&self, pipeline_id: PipelineId) -> RemoteEventTaskSource {
        RemoteEventTaskSource(self.remote_event_task_sender.clone(), pipeline_id)
    }

    pub fn timer_task_source(&self, pipeline_id: PipelineId) -> TimerTaskSource {
        TimerTaskSource(self.timer_task_sender.clone(), pipeline_id)
    }

    pub fn websocket_task_source(&self, pipeline_id: PipelineId) -> WebsocketTaskSource {
        WebsocketTaskSource(self.remote_event_task_sender.clone(), pipeline_id)
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
    fn handle_exit_pipeline_msg(&self, id: PipelineId, discard_bc: DiscardBrowsingContext) {
        debug!("Exiting pipeline {}.", id);

        self.closed_pipelines.borrow_mut().insert(id);

        // Check if the exit message is for an in progress load.
        let idx = self
            .incomplete_loads
            .borrow()
            .iter()
            .position(|load| load.pipeline_id == id);

        let document = self.documents.borrow_mut().remove(id);

        // Abort the parser, if any,
        // to prevent any further incoming networking messages from being handled.
        if let Some(document) = document.as_ref() {
            if let Some(parser) = document.get_current_parser() {
                parser.abort();
            }
        }

        // We should never have a pipeline that's still an incomplete load,
        // but also has a Document.
        debug_assert!(idx.is_none() || document.is_none());

        // Remove any incomplete load.
        let chan = if let Some(idx) = idx {
            let load = self.incomplete_loads.borrow_mut().remove(idx);
            load.layout_chan.clone()
        } else if let Some(ref document) = document {
            match document.window().layout_chan() {
                Some(chan) => chan.clone(),
                None => return warn!("Layout channel unavailable"),
            }
        } else {
            return warn!("Exiting nonexistant pipeline {}.", id);
        };

        // We shut down layout before removing the document,
        // since layout might still be in the middle of laying it out.
        debug!("preparing to shut down layout for page {}", id);
        let (response_chan, response_port) = unbounded();
        chan.send(message::Msg::PrepareToExit(response_chan)).ok();
        let _ = response_port.recv();

        debug!("shutting down layout for page {}", id);
        chan.send(message::Msg::ExitNow).ok();
        self.script_sender
            .send((id, ScriptMsg::PipelineExited))
            .ok();

        // Now that layout is shut down, it's OK to remove the document.
        if let Some(document) = document {
            // Clear any active animations and unroot all of the associated DOM objects.
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
            window.clear_js_runtime();
        }

        debug!("Exited pipeline {}.", id);
    }

    /// Handles a request to exit the script thread and shut down layout.
    fn handle_exit_script_thread_msg(&self) {
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
            self.handle_exit_pipeline_msg(pipeline_id, DiscardBrowsingContext::Yes);
        }

        self.background_hang_monitor.unregister();

        // If we're in multiprocess mode, shut-down the IPC router for this process.
        if opts::multiprocess() {
            debug!("Exiting IPC router thread in script thread.");
            ROUTER.shutdown();
        }

        debug!("Exited script thread.");
    }

    /// Handles animation tick requested during testing.
    pub fn handle_tick_all_animations_for_testing(id: PipelineId) {
        SCRIPT_THREAD_ROOT.with(|root| {
            let script_thread = unsafe { &*root.get().unwrap() };
            script_thread
                .handle_tick_all_animations(id, AnimationTickType::CSS_ANIMATIONS_AND_TRANSITIONS);
        });
    }

    /// Handles when layout thread finishes all animation in one tick
    fn handle_tick_all_animations(&self, id: PipelineId, tick_type: AnimationTickType) {
        let document = match self.documents.borrow().find_document(id) {
            Some(document) => document,
            None => return warn!("Message sent to closed pipeline {}.", id),
        };
        if tick_type.contains(AnimationTickType::REQUEST_ANIMATION_FRAME) {
            document.run_the_animation_frame_callbacks();
        }
        if tick_type.contains(AnimationTickType::CSS_ANIMATIONS_AND_TRANSITIONS) {
            document.animations().mark_animating_nodes_as_dirty();
            document.window().add_pending_reflow();
        }
    }

    /// Handles a Web font being loaded. Does nothing if the page no longer exists.
    fn handle_web_font_loaded(&self, pipeline_id: PipelineId) {
        let document = self.documents.borrow().find_document(pipeline_id);
        if let Some(document) = document {
            self.rebuild_and_force_reflow(&document, ReflowReason::WebFontLoaded);
        }
    }

    /// Handles a worklet being loaded. Does nothing if the page no longer exists.
    fn handle_worklet_loaded(&self, pipeline_id: PipelineId) {
        let document = self.documents.borrow().find_document(pipeline_id);
        if let Some(document) = document {
            self.rebuild_and_force_reflow(&document, ReflowReason::WorkletLoaded);
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
    ) {
        let iframe = self
            .documents
            .borrow()
            .find_iframe(parent_id, browsing_context_id);
        match iframe {
            Some(iframe) => iframe.iframe_load_event_steps(child_id),
            None => warn!("Message sent to closed pipeline {}.", parent_id),
        }
    }

    fn ask_constellation_for_browsing_context_info(
        &self,
        pipeline_id: PipelineId,
    ) -> Option<(BrowsingContextId, Option<PipelineId>)> {
        let (result_sender, result_receiver) = ipc::channel().unwrap();
        let msg = ScriptMsg::GetBrowsingContextInfo(pipeline_id, result_sender);
        self.script_sender
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
    ) -> Option<TopLevelBrowsingContextId> {
        let (result_sender, result_receiver) = ipc::channel().unwrap();
        let msg = ScriptMsg::GetTopForBrowsingContext(browsing_context_id, result_sender);
        self.script_sender
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
        top_level_browsing_context_id: TopLevelBrowsingContextId,
        pipeline_id: PipelineId,
        opener: Option<BrowsingContextId>,
    ) -> Option<DomRoot<WindowProxy>> {
        let (browsing_context_id, parent_pipeline_id) =
            self.ask_constellation_for_browsing_context_info(pipeline_id)?;
        if let Some(window_proxy) = self.window_proxies.borrow().get(&browsing_context_id) {
            return Some(DomRoot::from_ref(window_proxy));
        }

        let parent_browsing_context = parent_pipeline_id.and_then(|parent_id| {
            self.remote_window_proxy(
                global_to_clone,
                top_level_browsing_context_id,
                parent_id,
                opener,
            )
        });

        let opener_browsing_context = opener.and_then(|id| ScriptThread::find_window_proxy(id));

        let creator = CreatorBrowsingContextInfo::from(
            parent_browsing_context.as_deref(),
            opener_browsing_context.as_deref(),
        );

        let window_proxy = WindowProxy::new_dissimilar_origin(
            global_to_clone,
            browsing_context_id,
            top_level_browsing_context_id,
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
        top_level_browsing_context_id: TopLevelBrowsingContextId,
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
            (_, Some(iframe)) => Some(window_from_node(&**iframe).window_proxy()),
            (Some(parent_id), _) => self.remote_window_proxy(
                window.upcast(),
                top_level_browsing_context_id,
                parent_id,
                opener,
            ),
            _ => None,
        };

        let opener_browsing_context = opener.and_then(|id| ScriptThread::find_window_proxy(id));

        let creator = CreatorBrowsingContextInfo::from(
            parent_browsing_context.as_deref(),
            opener_browsing_context.as_deref(),
        );

        let window_proxy = WindowProxy::new(
            &window,
            browsing_context_id,
            top_level_browsing_context_id,
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
    fn load(&self, metadata: Metadata, incomplete: InProgressLoad) -> DomRoot<ServoParser> {
        let final_url = metadata.final_url.clone();
        {
            // send the final url to the layout thread.
            incomplete
                .layout_chan
                .send(message::Msg::SetFinalUrl(final_url.clone()))
                .unwrap();

            // update the pipeline url
            self.script_sender
                .send((
                    incomplete.pipeline_id,
                    ScriptMsg::SetFinalUrl(final_url.clone()),
                ))
                .unwrap();
        }
        debug!(
            "ScriptThread: loading {} on pipeline {:?}",
            incomplete.url, incomplete.pipeline_id
        );

        let MainThreadScriptChan(ref sender) = self.chan;

        let origin = if final_url.as_str() == "about:blank" || final_url.as_str() == "about:srcdoc"
        {
            incomplete.origin.clone()
        } else {
            MutableOrigin::new(final_url.origin())
        };

        let script_to_constellation_chan = ScriptToConstellationChan {
            sender: self.script_sender.clone(),
            pipeline_id: incomplete.pipeline_id,
        };

        let task_manager = TaskManager::new(
            self.dom_manipulation_task_source(incomplete.pipeline_id),
            self.file_reading_task_source(incomplete.pipeline_id),
            self.history_traversal_task_source(incomplete.pipeline_id),
            self.media_element_task_source(incomplete.pipeline_id),
            self.networking_task_source(incomplete.pipeline_id),
            self.performance_timeline_task_source(incomplete.pipeline_id)
                .clone(),
            self.port_message_queue(incomplete.pipeline_id),
            self.user_interaction_task_source(incomplete.pipeline_id),
            self.remote_event_task_source(incomplete.pipeline_id),
            self.timer_task_source(incomplete.pipeline_id),
            self.websocket_task_source(incomplete.pipeline_id),
        );
        // Create the window and document objects.
        let window = Window::new(
            self.js_runtime.clone(),
            MainThreadScriptChan(sender.clone()),
            task_manager,
            self.image_cache_channel.clone(),
            self.image_cache.clone(),
            self.resource_threads.clone(),
            self.bluetooth_thread.clone(),
            self.mem_profiler_chan.clone(),
            self.time_profiler_chan.clone(),
            self.devtools_chan.clone(),
            script_to_constellation_chan,
            self.control_chan.clone(),
            self.scheduler_chan.clone(),
            incomplete.layout_chan,
            incomplete.pipeline_id,
            incomplete.parent_info,
            incomplete.window_size,
            origin.clone(),
            final_url.clone(),
            incomplete.navigation_start,
            incomplete.navigation_start_precise,
            self.webgl_chan.as_ref().map(|chan| chan.channel()),
            self.webxr_registry.clone(),
            self.microtask_queue.clone(),
            self.webrender_document,
            self.webrender_api_sender.clone(),
            incomplete.layout_is_busy,
            self.relayout_event,
            self.prepare_for_screenshot,
            self.unminify_js,
            self.local_script_source.clone(),
            self.userscripts_path.clone(),
            self.headless,
            self.replace_surrogates,
            self.user_agent.clone(),
            self.player_context.clone(),
            self.event_loop_waker.as_ref().map(|w| (*w).clone_box()),
            self.gpu_id_hub.clone(),
            incomplete.inherited_secure_context,
        );

        // Initialize the browsing context for the window.
        let window_proxy = self.local_window_proxy(
            &window,
            incomplete.browsing_context_id,
            incomplete.top_level_browsing_context_id,
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
            headers
                .typed_get::<LastModified>()
                .map(|tm| dom_last_modified(&tm.into()))
        });

        let loader = DocumentLoader::new_with_threads(
            self.resource_threads.clone(),
            Some(final_url.clone()),
        );

        let content_type: Option<Mime> =
            metadata.content_type.map(Serde::into_inner).map(Into::into);

        let is_html_document = match content_type {
            Some(ref mime)
                if mime.type_() == mime::APPLICATION && mime.suffix() == Some(mime::XML) =>
            {
                IsHTMLDocument::NonHTMLDocument
            },

            Some(ref mime)
                if (mime.type_() == mime::TEXT && mime.subtype() == mime::XML) ||
                    (mime.type_() == mime::APPLICATION && mime.subtype() == mime::XML) =>
            {
                IsHTMLDocument::NonHTMLDocument
            },
            _ => IsHTMLDocument::HTMLDocument,
        };

        let referrer = match metadata.referrer {
            Some(ref referrer) => Some(referrer.clone().into_string()),
            None => None,
        };

        let referrer_policy = metadata
            .headers
            .as_ref()
            .map(Serde::deref)
            .and_then(|h| h.typed_get::<ReferrerPolicyHeader>())
            .map(ReferrerPolicy::from);

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
            referrer_policy,
            incomplete.canceller,
        );
        document.set_ready_state(DocumentReadyState::Loading);

        self.documents
            .borrow_mut()
            .insert(incomplete.pipeline_id, &*document);

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
                window_proxy.top_level_browsing_context_id(),
                incomplete.pipeline_id,
                UpdatePipelineIdReason::Navigation,
            );
        }

        self.script_sender
            .send((incomplete.pipeline_id, ScriptMsg::ActivateDocument))
            .unwrap();

        // Notify devtools that a new script global exists.
        self.notify_devtools(
            document.Title(),
            final_url.clone(),
            (incomplete.browsing_context_id, incomplete.pipeline_id, None),
        );

        let parse_input = DOMString::new();

        document.set_https_state(metadata.https_state);
        document.set_navigation_start(incomplete.navigation_start_precise);

        if is_html_document == IsHTMLDocument::NonHTMLDocument {
            ServoParser::parse_xml_document(&document, parse_input, final_url);
        } else {
            ServoParser::parse_html_document(&document, parse_input, final_url);
        }

        if incomplete.activity == DocumentActivity::FullyActive {
            window.resume();
        } else {
            window.suspend();
        }

        if !incomplete.is_visible {
            window.alter_resource_utilization(false);
        }

        document.get_current_parser().unwrap()
    }

    fn notify_devtools(
        &self,
        title: DOMString,
        url: ServoUrl,
        (bc, p, w): (BrowsingContextId, PipelineId, Option<WorkerId>),
    ) {
        if let Some(ref chan) = self.devtools_chan {
            let page_info = DevtoolsPageInfo {
                title: String::from(title),
                url: url,
            };
            chan.send(ScriptToDevtoolsControlMsg::NewGlobal(
                (bc, p, w),
                self.devtools_sender.clone(),
                page_info.clone(),
            ))
            .unwrap();

            let state = NavigationState::Stop(p, page_info);
            let _ = chan.send(ScriptToDevtoolsControlMsg::Navigate(bc, state));
        }
    }

    /// Reflows non-incrementally, rebuilding the entire layout tree in the process.
    fn rebuild_and_force_reflow(&self, document: &Document, reason: ReflowReason) {
        let window = window_from_node(&*document);
        document.dirty_all_nodes();
        window.reflow(ReflowGoal::Full, reason);
    }

    /// This is the main entry point for receiving and dispatching DOM events.
    ///
    /// TODO: Actually perform DOM event dispatch.
    fn handle_event(&self, pipeline_id: PipelineId, event: CompositorEvent) {
        // Do not handle events if the pipeline exited.
        let window = match self.documents.borrow().find_window(pipeline_id) {
            Some(win) => win,
            None => {
                return warn!(
                    "Compositor event sent to a pipeline that already exited {}.",
                    pipeline_id
                )
            },
        };
        // Do not handle events if the BC has been, or is being, discarded
        if window.Closed() {
            return warn!(
                "Compositor event sent to a pipeline with a closed window {}.",
                pipeline_id
            );
        }

        // Assuming all CompositionEvent are generated by user interactions.
        ScriptThread::set_user_interacting(true);
        match event {
            ResizeEvent(new_size, size_type) => {
                self.handle_resize_event(pipeline_id, new_size, size_type);
            },

            MouseButtonEvent(
                event_type,
                button,
                point,
                node_address,
                point_in_node,
                pressed_mouse_buttons,
            ) => {
                self.handle_mouse_event(
                    pipeline_id,
                    event_type,
                    button,
                    point,
                    node_address,
                    point_in_node,
                    pressed_mouse_buttons,
                );
            },

            MouseMoveEvent(point, node_address, pressed_mouse_buttons) => {
                let document = match self.documents.borrow().find_document(pipeline_id) {
                    Some(document) => document,
                    None => return warn!("Message sent to closed pipeline {}.", pipeline_id),
                };
                let window = document.window();

                // Get the previous target temporarily
                let prev_mouse_over_target = self.topmost_mouse_over_target.get();

                document.handle_mouse_move_event(
                    self.js_runtime.rt(),
                    point,
                    &self.topmost_mouse_over_target,
                    node_address,
                    pressed_mouse_buttons,
                );

                // Short-circuit if nothing changed
                if self.topmost_mouse_over_target.get() == prev_mouse_over_target {
                    return;
                }

                let mut state_already_changed = false;

                // Notify Constellation about the topmost anchor mouse over target.
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
                        let event = EmbedderMsg::Status(status);
                        window.send_to_embedder(event);

                        state_already_changed = true;
                    }
                }

                // We might have to reset the anchor state
                if !state_already_changed {
                    if let Some(target) = prev_mouse_over_target {
                        if let Some(_) = target
                            .upcast::<Node>()
                            .inclusive_ancestors(ShadowIncluding::No)
                            .filter_map(DomRoot::downcast::<HTMLAnchorElement>)
                            .next()
                        {
                            let event = EmbedderMsg::Status(None);
                            window.send_to_embedder(event);
                        }
                    }
                }
            },

            TouchEvent(event_type, identifier, point, node_address) => {
                let touch_result = self.handle_touch_event(
                    pipeline_id,
                    event_type,
                    identifier,
                    point,
                    node_address,
                );
                match (event_type, touch_result) {
                    (TouchEventType::Down, TouchEventResult::Processed(handled)) => {
                        let result = if handled {
                            // TODO: Wait to see if preventDefault is called on the first touchmove event.
                            EventResult::DefaultAllowed
                        } else {
                            EventResult::DefaultPrevented
                        };
                        let message = ScriptMsg::TouchEventProcessed(result);
                        self.script_sender.send((pipeline_id, message)).unwrap();
                    },
                    _ => {
                        // TODO: Calling preventDefault on a touchup event should prevent clicks.
                    },
                }
            },

            WheelEvent(delta, point, node_address) => {
                self.handle_wheel_event(pipeline_id, delta, point, node_address);
            },

            KeyboardEvent(key_event) => {
                let document = match self.documents.borrow().find_document(pipeline_id) {
                    Some(document) => document,
                    None => return warn!("Message sent to closed pipeline {}.", pipeline_id),
                };
                document.dispatch_key_event(key_event);
            },

            IMEDismissedEvent => {
                let document = match self.documents.borrow().find_document(pipeline_id) {
                    Some(document) => document,
                    None => return warn!("Message sent to closed pipeline {}.", pipeline_id),
                };
                document.ime_dismissed();
            },

            CompositionEvent(composition_event) => {
                let document = match self.documents.borrow().find_document(pipeline_id) {
                    Some(document) => document,
                    None => return warn!("Message sent to closed pipeline {}.", pipeline_id),
                };
                document.dispatch_composition_event(composition_event);
            },
        }

        ScriptThread::set_user_interacting(false);
    }

    fn handle_mouse_event(
        &self,
        pipeline_id: PipelineId,
        mouse_event_type: MouseEventType,
        button: MouseButton,
        point: Point2D<f32>,
        node_address: Option<UntrustedNodeAddress>,
        point_in_node: Option<Point2D<f32>>,
        pressed_mouse_buttons: u16,
    ) {
        let document = match self.documents.borrow().find_document(pipeline_id) {
            Some(document) => document,
            None => return warn!("Message sent to closed pipeline {}.", pipeline_id),
        };
        document.handle_mouse_event(
            self.js_runtime.rt(),
            button,
            point,
            mouse_event_type,
            node_address,
            point_in_node,
            pressed_mouse_buttons,
        );
    }

    fn handle_touch_event(
        &self,
        pipeline_id: PipelineId,
        event_type: TouchEventType,
        identifier: TouchId,
        point: Point2D<f32>,
        node_address: Option<UntrustedNodeAddress>,
    ) -> TouchEventResult {
        let document = match self.documents.borrow().find_document(pipeline_id) {
            Some(document) => document,
            None => {
                warn!("Message sent to closed pipeline {}.", pipeline_id);
                return TouchEventResult::Processed(true);
            },
        };
        document.handle_touch_event(
            self.js_runtime.rt(),
            event_type,
            identifier,
            point,
            node_address,
        )
    }

    fn handle_wheel_event(
        &self,
        pipeline_id: PipelineId,
        wheel_delta: WheelDelta,
        point: Point2D<f32>,
        node_address: Option<UntrustedNodeAddress>,
    ) {
        let document = match self.documents.borrow().find_document(pipeline_id) {
            Some(document) => document,
            None => return warn!("Message sent to closed pipeline {}.", pipeline_id),
        };
        document.handle_wheel_event(self.js_runtime.rt(), wheel_delta, point, node_address);
    }

    /// Handle a "navigate an iframe" message from the constellation.
    fn handle_navigate_iframe(
        &self,
        parent_pipeline_id: PipelineId,
        browsing_context_id: BrowsingContextId,
        load_data: LoadData,
        replace: HistoryEntryReplacement,
    ) {
        let iframe = self
            .documents
            .borrow()
            .find_iframe(parent_pipeline_id, browsing_context_id);
        if let Some(iframe) = iframe {
            iframe.navigate_or_reload_child_browsing_context(
                load_data,
                NavigationType::Regular,
                replace,
            );
        }
    }

    /// Turn javascript: URL into JS code to eval, according to the steps in
    /// https://html.spec.whatwg.org/multipage/#javascript-protocol
    pub fn eval_js_url(global_scope: &GlobalScope, load_data: &mut LoadData) {
        // This slice of the URL’s serialization is equivalent to (5.) to (7.):
        // Start with the scheme data of the parsed URL;
        // append question mark and query component, if any;
        // append number sign and fragment component if any.
        let encoded = &load_data.url.clone()[Position::BeforePath..];

        // Percent-decode (8.) and UTF-8 decode (9.)
        let script_source = percent_decode(encoded.as_bytes()).decode_utf8_lossy();

        // Script source is ready to be evaluated (11.)
        let _ac = enter_realm(global_scope);
        rooted!(in(*global_scope.get_cx()) let mut jsval = UndefinedValue());
        global_scope.evaluate_js_on_global_with_result(
            &script_source,
            jsval.handle_mut(),
            ScriptFetchOptions::default_classic_script(&global_scope),
            global_scope.api_base_url(),
        );

        load_data.js_eval_result = if jsval.get().is_string() {
            unsafe {
                let strval = DOMString::from_jsval(
                    *global_scope.get_cx(),
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

    fn handle_resize_event(
        &self,
        pipeline_id: PipelineId,
        new_size: WindowSizeData,
        size_type: WindowSizeType,
    ) {
        let document = match self.documents.borrow().find_document(pipeline_id) {
            Some(document) => document,
            None => return warn!("Message sent to closed pipeline {}.", pipeline_id),
        };

        let window = document.window();
        if window.window_size() == new_size {
            return;
        }
        debug!(
            "resizing pipeline {:?} from {:?} to {:?}",
            pipeline_id,
            window.window_size(),
            new_size
        );
        window.set_window_size(new_size);
        window.force_reflow(ReflowGoal::Full, ReflowReason::WindowResize, None);

        // http://dev.w3.org/csswg/cssom-view/#resizing-viewports
        if size_type == WindowSizeType::Resize {
            let uievent = UIEvent::new(
                &window,
                DOMString::from("resize"),
                EventBubbles::DoesNotBubble,
                EventCancelable::NotCancelable,
                Some(&window),
                0i32,
            );
            uievent.upcast::<Event>().fire(window.upcast());
        }

        // https://html.spec.whatwg.org/multipage/#event-loop-processing-model
        // Step 7.7 - evaluate media queries and report changes
        // Since we have resized, we need to re-evaluate MQLs
        window.evaluate_media_queries_and_report_changes();
    }

    /// Instructs the constellation to fetch the document that will be loaded. Stores the InProgressLoad
    /// argument until a notification is received that the fetch is complete.
    fn pre_page_load(&self, mut incomplete: InProgressLoad, load_data: LoadData) {
        let id = incomplete.pipeline_id.clone();
        let req_init = RequestBuilder::new(load_data.url.clone(), load_data.referrer)
            .method(load_data.method)
            .destination(Destination::Document)
            .credentials_mode(CredentialsMode::Include)
            .use_url_credentials(true)
            .pipeline_id(Some(id))
            .referrer_policy(load_data.referrer_policy)
            .headers(load_data.headers)
            .body(load_data.data)
            .redirect_mode(RedirectMode::Manual)
            .origin(incomplete.origin.immutable().clone());

        let context = ParserContext::new(id, load_data.url);
        self.incomplete_parser_contexts
            .0
            .borrow_mut()
            .push((id, context));

        let cancel_chan = incomplete.canceller.initialize();

        self.script_sender
            .send((
                id,
                ScriptMsg::InitiateNavigateRequest(req_init, cancel_chan),
            ))
            .unwrap();
        self.incomplete_loads.borrow_mut().push(incomplete);
    }

    fn handle_fetch_metadata(
        &self,
        id: PipelineId,
        fetch_metadata: Result<FetchMetadata, NetworkError>,
    ) {
        match fetch_metadata {
            Ok(_) => (),
            Err(ref e) => {
                warn!("Network error: {:?}", e);
            },
        };

        let mut incomplete_parser_contexts = self.incomplete_parser_contexts.0.borrow_mut();
        let parser = incomplete_parser_contexts
            .iter_mut()
            .find(|&&mut (pipeline_id, _)| pipeline_id == id);
        if let Some(&mut (_, ref mut ctxt)) = parser {
            ctxt.process_response(fetch_metadata);
        }
    }

    fn handle_fetch_chunk(&self, id: PipelineId, chunk: Vec<u8>) {
        let mut incomplete_parser_contexts = self.incomplete_parser_contexts.0.borrow_mut();
        let parser = incomplete_parser_contexts
            .iter_mut()
            .find(|&&mut (pipeline_id, _)| pipeline_id == id);
        if let Some(&mut (_, ref mut ctxt)) = parser {
            ctxt.process_response_chunk(chunk);
        }
    }

    fn handle_fetch_eof(&self, id: PipelineId, eof: Result<ResourceFetchTiming, NetworkError>) {
        let idx = self
            .incomplete_parser_contexts
            .0
            .borrow()
            .iter()
            .position(|&(pipeline_id, _)| pipeline_id == id);

        if let Some(idx) = idx {
            let (_, mut ctxt) = self.incomplete_parser_contexts.0.borrow_mut().remove(idx);
            ctxt.process_response_eof(eof);
        }
    }

    /// Synchronously fetch `about:blank`. Stores the `InProgressLoad`
    /// argument until a notification is received that the fetch is complete.
    fn start_page_load_about_blank(
        &self,
        incomplete: InProgressLoad,
        js_eval_result: Option<JsEvalResult>,
    ) {
        let id = incomplete.pipeline_id;

        self.incomplete_loads.borrow_mut().push(incomplete);

        let url = ServoUrl::parse("about:blank").unwrap();
        let mut context = ParserContext::new(id, url.clone());

        let mut meta = Metadata::default(url);
        meta.set_content_type(Some(&mime::TEXT_HTML));

        // If this page load is the result of a javascript scheme url, map
        // the evaluation result into a response.
        let chunk = match js_eval_result {
            Some(JsEvalResult::Ok(content)) => content,
            Some(JsEvalResult::NoContent) => {
                meta.status = Some((204, b"No Content".to_vec()));
                vec![]
            },
            None => vec![],
        };

        context.process_response(Ok(FetchMetadata::Unfiltered(meta)));
        context.process_response_chunk(chunk);
        context.process_response_eof(Ok(ResourceFetchTiming::new(ResourceTimingType::None)));
    }

    /// Synchronously parse a srcdoc document from a giving HTML string.
    fn page_load_about_srcdoc(&self, incomplete: InProgressLoad, load_data: LoadData) {
        let id = incomplete.pipeline_id;

        self.incomplete_loads.borrow_mut().push(incomplete);

        let url = ServoUrl::parse("about:srcdoc").unwrap();
        let mut context = ParserContext::new(id, url.clone());

        let mut meta = Metadata::default(url);
        meta.set_content_type(Some(&mime::TEXT_HTML));
        meta.set_referrer_policy(load_data.referrer_policy);

        let chunk = load_data.srcdoc.into_bytes();

        context.process_response(Ok(FetchMetadata::Unfiltered(meta)));
        context.process_response_chunk(chunk);
        context.process_response_eof(Ok(ResourceFetchTiming::new(ResourceTimingType::None)));
    }

    fn handle_css_error_reporting(
        &self,
        pipeline_id: PipelineId,
        filename: String,
        line: u32,
        column: u32,
        msg: String,
    ) {
        let sender = match self.devtools_chan {
            Some(ref sender) => sender,
            None => return,
        };

        if let Some(global) = self.documents.borrow().find_global(pipeline_id) {
            if global.live_devtools_updates() {
                let css_error = CSSError {
                    filename: filename,
                    line: line,
                    column: column,
                    msg: msg,
                };
                let message = ScriptToDevtoolsControlMsg::ReportCSSError(pipeline_id, css_error);
                sender.send(message).unwrap();
            }
        }
    }

    fn handle_reload(&self, pipeline_id: PipelineId) {
        let window = self.documents.borrow().find_window(pipeline_id);
        if let Some(window) = window {
            window.Location().reload_without_origin_check();
        }
    }

    fn handle_paint_metric(
        &self,
        pipeline_id: PipelineId,
        metric_type: ProgressiveWebMetricType,
        metric_value: u64,
    ) {
        let window = self.documents.borrow().find_window(pipeline_id);
        if let Some(window) = window {
            let entry = PerformancePaintTiming::new(
                &window.upcast::<GlobalScope>(),
                metric_type,
                metric_value,
            );
            window
                .Performance()
                .queue_entry(&entry.upcast::<PerformanceEntry>());
        }
    }

    fn handle_media_session_action(&self, pipeline_id: PipelineId, action: MediaSessionActionType) {
        if let Some(window) = self.documents.borrow().find_window(pipeline_id) {
            let media_session = window.Navigator().MediaSession();
            media_session.handle_action(action);
        } else {
            warn!("No MediaSession for this pipeline ID");
        };
    }

    pub fn enqueue_microtask(job: Microtask) {
        SCRIPT_THREAD_ROOT.with(|root| {
            let script_thread = unsafe { &*root.get().unwrap() };
            script_thread
                .microtask_queue
                .enqueue(job, script_thread.get_cx());
        });
    }

    fn perform_a_microtask_checkpoint(&self) {
        // Only perform the checkpoint if we're not shutting down.
        if self.can_continue_running_inner() {
            let globals = self
                .documents
                .borrow()
                .iter()
                .map(|(_id, document)| document.global())
                .collect();

            self.microtask_queue.checkpoint(
                self.get_cx(),
                |id| self.documents.borrow().find_global(id),
                globals,
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

fn dom_last_modified(tm: &SystemTime) -> String {
    let tm = tm.duration_since(SystemTime::UNIX_EPOCH).unwrap();
    let tm = Timespec::new(tm.as_secs() as i64, 0);
    let tm = at_utc(tm);
    tm.to_local()
        .strftime("%m/%d/%Y %H:%M:%S")
        .unwrap()
        .to_string()
}
