/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

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

extern crate itertools;

use bluetooth_traits::BluetoothRequest;
use canvas_traits::webgl::WebGLPipeline;
use devtools;
use devtools_traits::{DevtoolScriptControlMsg, DevtoolsPageInfo};
use devtools_traits::{ScriptToDevtoolsControlMsg, WorkerId};
use devtools_traits::CSSError;
use document_loader::DocumentLoader;
use dom::bindings::cell::DomRefCell;
use dom::bindings::codegen::Bindings::DocumentBinding::{DocumentMethods, DocumentReadyState};
use dom::bindings::codegen::Bindings::EventBinding::EventInit;
use dom::bindings::codegen::Bindings::TransitionEventBinding::TransitionEventInit;
use dom::bindings::codegen::Bindings::WindowBinding::WindowMethods;
use dom::bindings::conversions::{ConversionResult, FromJSValConvertible, StringificationBehavior};
use dom::bindings::inheritance::Castable;
use dom::bindings::num::Finite;
use dom::bindings::reflector::DomObject;
use dom::bindings::root::{Dom, DomRoot, MutNullableDom, RootCollection};
use dom::bindings::root::{RootedReference, ThreadLocalStackRoots};
use dom::bindings::str::DOMString;
use dom::bindings::structuredclone::StructuredCloneData;
use dom::bindings::trace::JSTraceable;
use dom::bindings::utils::WRAP_CALLBACKS;
use dom::customelementregistry::{CallbackReaction, CustomElementDefinition, CustomElementReactionStack};
use dom::document::{Document, DocumentSource, FocusType, HasBrowsingContext, IsHTMLDocument, TouchEventResult};
use dom::element::Element;
use dom::event::{Event, EventBubbles, EventCancelable};
use dom::globalscope::GlobalScope;
use dom::htmlanchorelement::HTMLAnchorElement;
use dom::htmliframeelement::{HTMLIFrameElement, NavigationType};
use dom::mutationobserver::MutationObserver;
use dom::node::{Node, NodeDamage, window_from_node, from_untrusted_node_address};
use dom::performanceentry::PerformanceEntry;
use dom::performancepainttiming::PerformancePaintTiming;
use dom::serviceworker::TrustedServiceWorkerAddress;
use dom::serviceworkerregistration::ServiceWorkerRegistration;
use dom::servoparser::{ParserContext, ServoParser};
use dom::transitionevent::TransitionEvent;
use dom::uievent::UIEvent;
use dom::window::{ReflowReason, Window};
use dom::windowproxy::WindowProxy;
use dom::worker::TrustedWorkerAddress;
use dom::worklet::WorkletThreadPool;
use dom::workletglobalscope::WorkletGlobalScopeInit;
use embedder_traits::EmbedderMsg;
use euclid::{Point2D, Vector2D, Rect};
use fetch::FetchCanceller;
use hyper::header::{ContentType, HttpDate, Headers, LastModified};
use hyper::header::ReferrerPolicy as ReferrerPolicyHeader;
use hyper::mime::{Mime, SubLevel, TopLevel};
use hyper_serde::Serde;
use ipc_channel::ipc::{self, IpcSender};
use ipc_channel::router::ROUTER;
use js::glue::GetWindowProxyClass;
use js::jsapi::{JSAutoCompartment, JSContext, JS_SetWrapObjectCallbacks};
use js::jsapi::{JSTracer, SetWindowProxyClass};
use js::jsval::UndefinedValue;
use metrics::{MAX_TASK_NS, PaintTimeMetrics};
use microtask::{MicrotaskQueue, Microtask};
use msg::constellation_msg::{BrowsingContextId, HistoryStateId, PipelineId};
use msg::constellation_msg::{PipelineNamespace, TopLevelBrowsingContextId};
use net_traits::{FetchMetadata, FetchResponseListener, FetchResponseMsg};
use net_traits::{Metadata, NetworkError, ReferrerPolicy, ResourceThreads};
use net_traits::image_cache::{ImageCache, PendingImageResponse};
use net_traits::request::{CredentialsMode, Destination, RedirectMode, RequestInit};
use net_traits::storage_thread::StorageType;
use profile_traits::mem::{self, OpaqueSender, ReportsChan};
use profile_traits::time::{self, ProfilerCategory, profile};
use script_layout_interface::message::{self, Msg, NewLayoutThreadInfo, ReflowGoal};
use script_runtime::{CommonScriptMsg, ScriptChan, ScriptThreadEventCategory};
use script_runtime::{ScriptPort, get_reports, new_rt_and_cx, Runtime};
use script_traits::{CompositorEvent, ConstellationControlMsg};
use script_traits::{DiscardBrowsingContext, DocumentActivity, EventResult};
use script_traits::{InitialScriptState, JsEvalResult, LayoutMsg, LoadData};
use script_traits::{MouseButton, MouseEventType, NewLayoutInfo};
use script_traits::{ProgressiveWebMetricType, Painter, ScriptMsg, ScriptThreadFactory};
use script_traits::{ScriptToConstellationChan, TimerEvent, TimerSchedulerMsg};
use script_traits::{TimerSource, TouchEventType, TouchId, UntrustedNodeAddress};
use script_traits::{UpdatePipelineIdReason, WindowSizeData, WindowSizeType};
use script_traits::CompositorEvent::{KeyEvent, MouseButtonEvent, MouseMoveEvent, ResizeEvent, TouchEvent};
use script_traits::webdriver_msg::WebDriverScriptCommand;
use serviceworkerjob::{Job, JobQueue};
use servo_atoms::Atom;
use servo_config::opts;
use servo_url::{ImmutableOrigin, MutableOrigin, ServoUrl};
use std::cell::Cell;
use std::cell::RefCell;
use std::collections::{hash_map, HashMap, HashSet};
use std::default::Default;
use std::ops::Deref;
use std::option::Option;
use std::ptr;
use std::rc::Rc;
use std::result::Result;
use std::sync::Arc;
use std::sync::mpsc::{Receiver, Select, Sender, channel};
use std::thread;
use style::thread_state::{self, ThreadState};
use task_source::dom_manipulation::DOMManipulationTaskSource;
use task_source::file_reading::FileReadingTaskSource;
use task_source::history_traversal::HistoryTraversalTaskSource;
use task_source::networking::NetworkingTaskSource;
use task_source::performance_timeline::PerformanceTimelineTaskSource;
use task_source::remote_event::RemoteEventTaskSource;
use task_source::user_interaction::UserInteractionTaskSource;
use time::{get_time, precise_time_ns, Tm};
use url::Position;
use url::percent_encoding::percent_decode;
use webdriver_handlers;
use webrender_api::DocumentId;
use webvr_traits::{WebVREvent, WebVRMsg};

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
    window_size: Option<WindowSizeData>,
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
}

impl InProgressLoad {
    /// Create a new InProgressLoad object.
    fn new(id: PipelineId,
           browsing_context_id: BrowsingContextId,
           top_level_browsing_context_id: TopLevelBrowsingContextId,
           parent_info: Option<PipelineId>,
           opener: Option<BrowsingContextId>,
           layout_chan: Sender<message::Msg>,
           window_size: Option<WindowSizeData>,
           url: ServoUrl,
           origin: MutableOrigin) -> InProgressLoad {
        let current_time = get_time();
        let navigation_start_precise = precise_time_ns();
        layout_chan.send(message::Msg::SetNavigationStart(navigation_start_precise)).unwrap();
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
        }
    }
}

#[derive(Debug)]
enum MixedMessage {
    FromConstellation(ConstellationControlMsg),
    FromScript(MainThreadScriptMsg),
    FromDevtools(DevtoolScriptControlMsg),
    FromImageCache((PipelineId, PendingImageResponse)),
    FromScheduler(TimerEvent),
}

/// Messages used to control the script event loop.
#[derive(Debug)]
pub enum MainThreadScriptMsg {
    /// Common variants associated with the script messages
    Common(CommonScriptMsg),
    /// Begins a content-initiated load on the specified pipeline (only
    /// dispatched to ScriptThread). Allows for a replace bool to be passed. If true,
    /// the current entry will be replaced instead of a new entry being added.
    Navigate(PipelineId, LoadData, bool),
    /// Notifies the script thread that a new worklet has been loaded, and thus the page should be
    /// reflowed.
    WorkletLoaded(PipelineId),
    /// Notifies the script thread that a new paint worklet has been registered.
    RegisterPaintWorklet {
        pipeline_id: PipelineId,
        name: Atom,
        properties: Vec<Atom>,
        painter: Box<Painter>
    },
    /// Dispatches a job queue.
    DispatchJobQueue { scope_url: ServoUrl },
}

impl OpaqueSender<CommonScriptMsg> for Box<ScriptChan + Send> {
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
            _ => Err(()),
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
            _ => Err(()),
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

    fn clone(&self) -> Box<ScriptChan + Send> {
        Box::new(SendableMainThreadScriptChan((&self.0).clone()))
    }
}

/// Encapsulates internal communication of main thread messages within the script thread.
#[derive(JSTraceable)]
pub struct MainThreadScriptChan(pub Sender<MainThreadScriptMsg>);

impl ScriptChan for MainThreadScriptChan {
    fn send(&self, msg: CommonScriptMsg) -> Result<(), ()> {
        self.0.send(MainThreadScriptMsg::Common(msg)).map_err(|_| ())
    }

    fn clone(&self) -> Box<ScriptChan + Send> {
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
#[must_root]
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
        self.map.remove(&pipeline_id).map(|ref doc| DomRoot::from_ref(&**doc))
    }

    pub fn is_empty(&self) -> bool {
        self.map.is_empty()
    }

    pub fn find_document(&self, pipeline_id: PipelineId) -> Option<DomRoot<Document>> {
        self.map.get(&pipeline_id).map(|doc| DomRoot::from_ref(&**doc))
    }

    pub fn len(&self) -> usize {
        self.map.len()
    }

    pub fn find_window(&self, pipeline_id: PipelineId) -> Option<DomRoot<Window>> {
        self.find_document(pipeline_id).map(|doc| DomRoot::from_ref(doc.window()))
    }

    pub fn find_global(&self, pipeline_id: PipelineId) -> Option<DomRoot<GlobalScope>> {
        self.find_window(pipeline_id).map(|window| DomRoot::from_ref(window.upcast()))
    }

    pub fn find_iframe(&self, pipeline_id: PipelineId, browsing_context_id: BrowsingContextId)
                       -> Option<DomRoot<HTMLIFrameElement>>
    {
        self.find_document(pipeline_id).and_then(|doc| doc.find_iframe(browsing_context_id))
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
        self.iter.next().map(|(id, doc)| (*id, DomRoot::from_ref(&**doc)))
    }
}

// We borrow the incomplete parser contexts mutably during parsing,
// which is fine except that parsing can trigger evaluation,
// which can trigger GC, and so we can end up tracing the script
// thread during parsing. For this reason, we don't trace the
// incomplete parser contexts during GC.
type IncompleteParserContexts = Vec<(PipelineId, ParserContext)>;
unsafe_no_jsmanaged_fields!(RefCell<IncompleteParserContexts>);

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
    incomplete_parser_contexts: RefCell<IncompleteParserContexts>,
    /// A map to store service worker registrations for a given origin
    registration_map: DomRefCell<HashMap<ServoUrl, Dom<ServiceWorkerRegistration>>>,
    /// A job queue for Service Workers keyed by their scope url
    job_queue_map: Rc<JobQueue>,
    /// Image cache for this script thread.
    image_cache: Arc<ImageCache>,
    /// A handle to the resource thread. This is an `Arc` to avoid running out of file descriptors if
    /// there are many iframes.
    resource_threads: ResourceThreads,
    /// A handle to the bluetooth thread.
    bluetooth_thread: IpcSender<BluetoothRequest>,

    /// The port on which the script thread receives messages (load URL, exit, etc.)
    port: Receiver<MainThreadScriptMsg>,
    /// A channel to hand out to script thread-based entities that need to be able to enqueue
    /// events in the event queue.
    chan: MainThreadScriptChan,

    dom_manipulation_task_sender: Sender<MainThreadScriptMsg>,

    user_interaction_task_sender: Sender<MainThreadScriptMsg>,

    networking_task_sender: Box<ScriptChan>,

    history_traversal_task_source: HistoryTraversalTaskSource,

    file_reading_task_sender: Box<ScriptChan>,

    performance_timeline_task_sender: Box<ScriptChan>,

    remote_event_task_sender: Box<ScriptChan>,

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
    time_profiler_chan: time::ProfilerChan,

    /// For providing contact with the memory profiler.
    mem_profiler_chan: mem::ProfilerChan,

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
    timer_event_chan: Sender<TimerEvent>,
    timer_event_port: Receiver<TimerEvent>,

    content_process_shutdown_chan: IpcSender<()>,

    /// <https://html.spec.whatwg.org/multipage/#microtask-queue>
    microtask_queue: Rc<MicrotaskQueue>,

    /// Microtask Queue for adding support for mutation observer microtasks
    mutation_observer_compound_microtask_queued: Cell<bool>,

    /// The unit of related similar-origin browsing contexts' list of MutationObserver objects
    mutation_observers: DomRefCell<Vec<Dom<MutationObserver>>>,

    /// A handle to the WebGL thread
    webgl_chan: Option<WebGLPipeline>,

    /// A handle to the webvr thread, if available
    webvr_chan: Option<IpcSender<WebVRMsg>>,

    /// The worklet thread pool
    worklet_thread_pool: DomRefCell<Option<Rc<WorkletThreadPool>>>,

    /// A list of pipelines containing documents that finished loading all their blocking
    /// resources during a turn of the event loop.
    docs_with_no_blocking_loads: DomRefCell<HashSet<Dom<Document>>>,

    /// A list of nodes with in-progress CSS transitions, which roots them for the duration
    /// of the transition.
    transitioning_nodes: DomRefCell<Vec<Dom<Node>>>,

    /// <https://html.spec.whatwg.org/multipage/#custom-element-reactions-stack>
    custom_element_reaction_stack: CustomElementReactionStack,

    /// The Webrender Document ID associated with this thread.
    webrender_document: DocumentId,
}

/// In the event of thread panic, all data on the stack runs its destructor. However, there
/// are no reachable, owning pointers to the DOM memory, so it never gets freed by default
/// when the script thread fails. The ScriptMemoryFailsafe uses the destructor bomb pattern
/// to forcibly tear down the JS compartments for pages associated with the failing ScriptThread.
struct ScriptMemoryFailsafe<'a> {
    owner: Option<&'a ScriptThread>,
}

impl<'a> ScriptMemoryFailsafe<'a> {
    fn neuter(&mut self) {
        self.owner = None;
    }

    fn new(owner: &'a ScriptThread) -> ScriptMemoryFailsafe<'a> {
        ScriptMemoryFailsafe {
            owner: Some(owner),
        }
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

    fn create(state: InitialScriptState,
              load_data: LoadData)
              -> (Sender<message::Msg>, Receiver<message::Msg>) {
        let (script_chan, script_port) = channel();

        let (sender, receiver) = channel();
        let layout_chan = sender.clone();
        thread::Builder::new().name(format!("ScriptThread {:?}", state.id)).spawn(move || {
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
            let mem_profiler_chan = state.mem_profiler_chan.clone();
            let window_size = state.window_size;
            let script_thread = ScriptThread::new(state,
                                                  script_port,
                                                  script_chan.clone());

            SCRIPT_THREAD_ROOT.with(|root| {
                root.set(Some(&script_thread as *const _));
            });

            let mut failsafe = ScriptMemoryFailsafe::new(&script_thread);

            let origin = MutableOrigin::new(load_data.url.origin());
            let new_load = InProgressLoad::new(id, browsing_context_id, top_level_browsing_context_id, parent_info,
                                               opener, layout_chan, window_size, load_data.url.clone(), origin);
            script_thread.pre_page_load(new_load, load_data);

            let reporter_name = format!("script-reporter-{}", id);
            mem_profiler_chan.run_with_memory_reporting(|| {
                script_thread.start();
                let _ = script_thread.content_process_shutdown_chan.send(());
            }, reporter_name, script_chan, CommonScriptMsg::CollectReports);

            // This must always be the very last operation performed before the thread completes
            failsafe.neuter();
        }).expect("Thread spawning failed");

        (sender, receiver)
    }
}

impl ScriptThread {
    pub unsafe fn note_newly_transitioning_nodes(nodes: Vec<UntrustedNodeAddress>) {
        SCRIPT_THREAD_ROOT.with(|root| {
            let script_thread = &*root.get().unwrap();
            let js_runtime = script_thread.js_runtime.rt();
            let new_nodes = nodes
                .into_iter()
                .map(|n| Dom::from_ref(&*from_untrusted_node_address(js_runtime, n)));
            script_thread.transitioning_nodes.borrow_mut().extend(new_nodes);
        })
    }

    pub fn set_mutation_observer_compound_microtask_queued(value: bool) {
        SCRIPT_THREAD_ROOT.with(|root| {
            let script_thread = unsafe { &*root.get().unwrap() };
            script_thread.mutation_observer_compound_microtask_queued.set(value);
        })
    }

    pub fn is_mutation_observer_compound_microtask_queued() -> bool {
        SCRIPT_THREAD_ROOT.with(|root| {
            let script_thread = unsafe { &*root.get().unwrap() };
            return script_thread.mutation_observer_compound_microtask_queued.get();
        })
    }

    pub fn add_mutation_observer(observer: &MutationObserver) {
        SCRIPT_THREAD_ROOT.with(|root| {
            let script_thread = unsafe { &*root.get().unwrap() };
            script_thread.mutation_observers
                .borrow_mut()
                .push(Dom::from_ref(observer));
        })
    }

    pub fn get_mutation_observers() -> Vec<DomRoot<MutationObserver>> {
        SCRIPT_THREAD_ROOT.with(|root| {
            let script_thread = unsafe { &*root.get().unwrap() };
            script_thread.mutation_observers.borrow().iter().map(|o| DomRoot::from_ref(&**o)).collect()
        })
    }

    pub fn mark_document_with_no_blocked_loads(doc: &Document) {
        SCRIPT_THREAD_ROOT.with(|root| {
            let script_thread = unsafe { &*root.get().unwrap() };
            script_thread.docs_with_no_blocking_loads
                .borrow_mut()
                .insert(Dom::from_ref(doc));
        })
    }

    pub fn invoke_perform_a_microtask_checkpoint() {
        SCRIPT_THREAD_ROOT.with(|root| {
            let script_thread = unsafe { &*root.get().unwrap() };
            script_thread.perform_a_microtask_checkpoint()
        })
    }

    pub fn page_headers_available(id: &PipelineId, metadata: Option<Metadata>)
                                  -> Option<DomRoot<ServoParser>> {
        SCRIPT_THREAD_ROOT.with(|root| {
            let script_thread = unsafe { &*root.get().unwrap() };
            script_thread.handle_page_headers_available(id, metadata)
        })
    }

    #[allow(unrooted_must_root)]
    pub fn schedule_job(job: Job) {
        SCRIPT_THREAD_ROOT.with(|root| {
            let script_thread = unsafe { &*root.get().unwrap() };
            let job_queue = &*script_thread.job_queue_map;
            job_queue.schedule_job(job, &script_thread);
        });
    }

    pub fn process_event(msg: CommonScriptMsg) {
        SCRIPT_THREAD_ROOT.with(|root| {
            if let Some(script_thread) = root.get() {
                let script_thread = unsafe { &*script_thread };
                script_thread.handle_msg_from_script(MainThreadScriptMsg::Common(msg));
            }
        });
    }

    // https://html.spec.whatwg.org/multipage/#await-a-stable-state
    pub fn await_stable_state(task: Microtask) {
        SCRIPT_THREAD_ROOT.with(|root| {
            if let Some(script_thread) = root.get() {
                let script_thread = unsafe { &*script_thread };
                script_thread.microtask_queue.enqueue(task);
            }
        });
    }

    pub fn process_attach_layout(new_layout_info: NewLayoutInfo, origin: MutableOrigin) {
        SCRIPT_THREAD_ROOT.with(|root| {
            if let Some(script_thread) = root.get() {
                let script_thread = unsafe { &*script_thread };
                let pipeline_id = Some(new_layout_info.new_pipeline_id);
                script_thread.profile_event(ScriptThreadEventCategory::AttachLayout, pipeline_id, || {
                    script_thread.handle_new_layout(new_layout_info, origin);
                })
            }
        });
    }

    pub fn get_top_level_for_browsing_context(sender_pipeline: PipelineId,
                                              browsing_context_id: BrowsingContextId)
                                              -> Option<TopLevelBrowsingContextId> {
        SCRIPT_THREAD_ROOT.with(|root| root.get().and_then(|script_thread| {
            let script_thread = unsafe { &*script_thread };
            script_thread.ask_constellation_for_top_level_info(sender_pipeline, browsing_context_id)
        }))
    }

    pub fn find_document(id: PipelineId) -> Option<DomRoot<Document>> {
        SCRIPT_THREAD_ROOT.with(|root| root.get().and_then(|script_thread| {
            let script_thread = unsafe { &*script_thread };
            script_thread.documents.borrow().find_document(id)
        }))
    }

    pub fn find_window_proxy(id: BrowsingContextId) -> Option<DomRoot<WindowProxy>> {
        SCRIPT_THREAD_ROOT.with(|root| root.get().and_then(|script_thread| {
            let script_thread = unsafe { &*script_thread };
            script_thread.window_proxies.borrow().get(&id)
                .map(|context| DomRoot::from_ref(&**context))
        }))
    }

    pub fn find_window_proxy_by_name(name: &DOMString) -> Option<DomRoot<WindowProxy>> {
        SCRIPT_THREAD_ROOT.with(|root| root.get().and_then(|script_thread| {
            let script_thread = unsafe { &*script_thread };
            for (_, proxy) in script_thread.window_proxies.borrow().iter() {
                if proxy.get_name() == *name {
                    return Some(DomRoot::from_ref(&**proxy))
                }
            }
            None
        }))
    }

    pub fn worklet_thread_pool() -> Rc<WorkletThreadPool> {
        SCRIPT_THREAD_ROOT.with(|root| {
            let script_thread = unsafe { &*root.get().unwrap() };
            script_thread.worklet_thread_pool.borrow_mut().get_or_insert_with(|| {
                let init = WorkletGlobalScopeInit {
                    to_script_thread_sender: script_thread.chan.0.clone(),
                    resource_threads: script_thread.resource_threads.clone(),
                    mem_profiler_chan: script_thread.mem_profiler_chan.clone(),
                    time_profiler_chan: script_thread.time_profiler_chan.clone(),
                    devtools_chan: script_thread.devtools_chan.clone(),
                    to_constellation_sender: script_thread.script_sender.clone(),
                    scheduler_chan: script_thread.scheduler_chan.clone(),
                    image_cache: script_thread.image_cache.clone(),
                };
                Rc::new(WorkletThreadPool::spawn(init))
            }).clone()
        })
    }

    fn handle_register_paint_worklet(
        &self,
        pipeline_id: PipelineId,
        name: Atom,
        properties: Vec<Atom>,
        painter: Box<Painter>)
    {
        let window = self.documents.borrow().find_window(pipeline_id);
        let window = match window {
            Some(window) => window,
            None => return warn!("Paint worklet registered after pipeline {} closed.", pipeline_id),
        };
        let _ = window.layout_chan().send(
            Msg::RegisterPaint(name, properties, painter),
        );
    }

    pub fn push_new_element_queue() {
        SCRIPT_THREAD_ROOT.with(|root| {
            if let Some(script_thread) = root.get() {
                let script_thread = unsafe { &*script_thread };
                script_thread.custom_element_reaction_stack.push_new_element_queue();
            }
        })
    }

    pub fn pop_current_element_queue() {
        SCRIPT_THREAD_ROOT.with(|root| {
            if let Some(script_thread) = root.get() {
                let script_thread = unsafe { &*script_thread };
                script_thread.custom_element_reaction_stack.pop_current_element_queue();
            }
        })
    }

    pub fn enqueue_callback_reaction(element: &Element,
                                     reaction: CallbackReaction,
                                     definition: Option<Rc<CustomElementDefinition>>) {
        SCRIPT_THREAD_ROOT.with(|root| {
            if let Some(script_thread) = root.get() {
                let script_thread = unsafe { &*script_thread };
                script_thread.custom_element_reaction_stack.enqueue_callback_reaction(element, reaction, definition);
            }
        })
    }

    pub fn enqueue_upgrade_reaction(element: &Element, definition: Rc<CustomElementDefinition>) {
        SCRIPT_THREAD_ROOT.with(|root| {
            if let Some(script_thread) = root.get() {
                let script_thread = unsafe { &*script_thread };
                script_thread.custom_element_reaction_stack.enqueue_upgrade_reaction(element, definition);
            }
        })
    }

    pub fn invoke_backup_element_queue() {
        SCRIPT_THREAD_ROOT.with(|root| {
            if let Some(script_thread) = root.get() {
                let script_thread = unsafe { &*script_thread };
                script_thread.custom_element_reaction_stack.invoke_backup_element_queue();
            }
        })
    }

    /// Creates a new script thread.
    pub fn new(state: InitialScriptState,
               port: Receiver<MainThreadScriptMsg>,
               chan: Sender<MainThreadScriptMsg>)
               -> ScriptThread {
        let runtime = unsafe { new_rt_and_cx() };
        let cx = runtime.cx();

        unsafe {
            JS_SetWrapObjectCallbacks(cx, &WRAP_CALLBACKS);
            SetWindowProxyClass(cx, GetWindowProxyClass());
        }

        // Ask the router to proxy IPC messages from the devtools to us.
        let (ipc_devtools_sender, ipc_devtools_receiver) = ipc::channel().unwrap();
        let devtools_port = ROUTER.route_ipc_receiver_to_new_mpsc_receiver(ipc_devtools_receiver);

        let (timer_event_chan, timer_event_port) = channel();

        // Ask the router to proxy IPC messages from the control port to us.
        let control_port = ROUTER.route_ipc_receiver_to_new_mpsc_receiver(state.control_port);

        let boxed_script_sender = Box::new(MainThreadScriptChan(chan.clone()));

        let (image_cache_channel, image_cache_port) = channel();

        ScriptThread {
            documents: DomRefCell::new(Documents::new()),
            window_proxies: DomRefCell::new(HashMap::new()),
            incomplete_loads: DomRefCell::new(vec!()),
            incomplete_parser_contexts: RefCell::new(vec!()),
            registration_map: DomRefCell::new(HashMap::new()),
            job_queue_map: Rc::new(JobQueue::new()),

            image_cache: state.image_cache.clone(),
            image_cache_channel: image_cache_channel,
            image_cache_port: image_cache_port,

            resource_threads: state.resource_threads,
            bluetooth_thread: state.bluetooth_thread,

            port: port,

            chan: MainThreadScriptChan(chan.clone()),
            dom_manipulation_task_sender: chan.clone(),
            user_interaction_task_sender: chan.clone(),
            networking_task_sender: boxed_script_sender.clone(),
            file_reading_task_sender: boxed_script_sender.clone(),
            performance_timeline_task_sender: boxed_script_sender.clone(),
            remote_event_task_sender: boxed_script_sender.clone(),

            history_traversal_task_source: HistoryTraversalTaskSource(chan),

            control_chan: state.control_chan,
            control_port: control_port,
            script_sender: state.script_to_constellation_chan.sender.clone(),
            time_profiler_chan: state.time_profiler_chan.clone(),
            mem_profiler_chan: state.mem_profiler_chan,

            devtools_chan: state.devtools_chan,
            devtools_port: devtools_port,
            devtools_sender: ipc_devtools_sender,

            js_runtime: Rc::new(runtime),
            topmost_mouse_over_target: MutNullableDom::new(Default::default()),
            closed_pipelines: DomRefCell::new(HashSet::new()),

            scheduler_chan: state.scheduler_chan,
            timer_event_chan: timer_event_chan,
            timer_event_port: timer_event_port,

            content_process_shutdown_chan: state.content_process_shutdown_chan,

            microtask_queue: Default::default(),

            mutation_observer_compound_microtask_queued: Default::default(),

            mutation_observers: Default::default(),

            layout_to_constellation_chan: state.layout_to_constellation_chan,

            webgl_chan: state.webgl_chan,
            webvr_chan: state.webvr_chan,

            worklet_thread_pool: Default::default(),

            docs_with_no_blocking_loads: Default::default(),

            transitioning_nodes: Default::default(),

            custom_element_reaction_stack: CustomElementReactionStack::new(),

            webrender_document: state.webrender_document,
        }
    }

    pub fn get_cx(&self) -> *mut JSContext {
        self.js_runtime.cx()
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
        use self::MixedMessage::{FromConstellation, FromDevtools, FromImageCache};
        use self::MixedMessage::{FromScheduler, FromScript};

        // Handle pending resize events.
        // Gather them first to avoid a double mut borrow on self.
        let mut resizes = vec!();

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

        // Receive at least one message so we don't spinloop.
        debug!("Waiting for event.");
        let mut event = {
            let sel = Select::new();
            let mut script_port = sel.handle(&self.port);
            let mut control_port = sel.handle(&self.control_port);
            let mut timer_event_port = sel.handle(&self.timer_event_port);
            let mut devtools_port = sel.handle(&self.devtools_port);
            let mut image_cache_port = sel.handle(&self.image_cache_port);
            unsafe {
                script_port.add();
                control_port.add();
                timer_event_port.add();
                if self.devtools_chan.is_some() {
                    devtools_port.add();
                }
                image_cache_port.add();
            }
            let ret = sel.wait();
            if ret == script_port.id() {
                FromScript(self.port.recv().unwrap())
            } else if ret == control_port.id() {
                FromConstellation(self.control_port.recv().unwrap())
            } else if ret == timer_event_port.id() {
                FromScheduler(self.timer_event_port.recv().unwrap())
            } else if ret == devtools_port.id() {
                FromDevtools(self.devtools_port.recv().unwrap())
            } else if ret == image_cache_port.id() {
                FromImageCache(self.image_cache_port.recv().unwrap())
            } else {
                panic!("unexpected select result")
            }
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
                FromConstellation(ConstellationControlMsg::AttachLayout(
                        new_layout_info)) => {
                    let pipeline_id = new_layout_info.new_pipeline_id;
                    self.profile_event(ScriptThreadEventCategory::AttachLayout, Some(pipeline_id), || {
                        // If this is an about:blank load, it must share the creator's origin.
                        // This must match the logic in the constellation when creating a new pipeline
                        let origin = if new_layout_info.load_data.url.as_str() != "about:blank" {
                            MutableOrigin::new(new_layout_info.load_data.url.origin())
                        } else if let Some(parent) = new_layout_info.parent_info
                                .and_then(|pipeline_id| self.documents.borrow()
                                .find_document(pipeline_id)) {
                            parent.origin().clone()
                        } else if let Some(creator) = new_layout_info.load_data.creator_pipeline_id
                                .and_then(|pipeline_id| self.documents.borrow()
                                .find_document(pipeline_id)) {
                            creator.origin().clone()
                        } else {
                            MutableOrigin::new(ImmutableOrigin::new_opaque())
                        };

                        self.handle_new_layout(new_layout_info, origin);
                    })
                }
                FromConstellation(ConstellationControlMsg::Resize(id, size, size_type)) => {
                    // step 7.7
                    self.profile_event(ScriptThreadEventCategory::Resize, Some(id), || {
                        self.handle_resize(id, size, size_type);
                    })
                }
                FromConstellation(ConstellationControlMsg::Viewport(id, rect)) => {
                    self.profile_event(ScriptThreadEventCategory::SetViewport, Some(id), || {
                        self.handle_viewport(id, rect);
                    })
                }
                FromConstellation(ConstellationControlMsg::SetScrollState(id, scroll_state)) => {
                    self.profile_event(ScriptThreadEventCategory::SetScrollState, Some(id), || {
                        self.handle_set_scroll_state(id, &scroll_state);
                    })
                }
                FromConstellation(ConstellationControlMsg::TickAllAnimations(
                        pipeline_id)) => {
                    // step 7.8
                    if !animation_ticks.contains(&pipeline_id) {
                        animation_ticks.insert(pipeline_id);
                        sequential.push(event);
                    }
                }
                FromConstellation(ConstellationControlMsg::SendEvent(
                        _,
                        MouseMoveEvent(..))) => {
                    match mouse_move_event_index {
                        None => {
                            mouse_move_event_index = Some(sequential.len());
                            sequential.push(event);
                        }
                        Some(index) => {
                            sequential[index] = event
                        }
                    }
                }
                _ => {
                    sequential.push(event);
                }
            }

            // If any of our input sources has an event pending, we'll perform another iteration
            // and check for more resize events. If there are no events pending, we'll move
            // on and execute the sequential non-resize events we've seen.
            match self.control_port.try_recv() {
                Err(_) => match self.port.try_recv() {
                    Err(_) => match self.timer_event_port.try_recv() {
                        Err(_) => match self.devtools_port.try_recv() {
                            Err(_) => match self.image_cache_port.try_recv() {
                                Err(_) => break,
                                Ok(ev) => event = FromImageCache(ev),
                            },
                            Ok(ev) => event = FromDevtools(ev),
                        },
                        Ok(ev) => event = FromScheduler(ev),
                    },
                    Ok(ev) => event = FromScript(ev),
                },
                Ok(ev) => event = FromConstellation(ev),
            }
        }

        // Process the gathered events.
        debug!("Processing events.");
        for msg in sequential {
            debug!("Processing event {:?}.", msg);

            let category = self.categorize_msg(&msg);
            let pipeline_id = self.message_to_pipeline(&msg);

            let result = self.profile_event(category, pipeline_id, move || {
                match msg {
                    FromConstellation(ConstellationControlMsg::ExitScriptThread) => {
                        self.handle_exit_script_thread_msg();
                        return Some(false);
                    },
                    FromConstellation(inner_msg) => self.handle_msg_from_constellation(inner_msg),
                    FromScript(inner_msg) => self.handle_msg_from_script(inner_msg),
                    FromScheduler(inner_msg) => self.handle_timer_event(inner_msg),
                    FromDevtools(inner_msg) => self.handle_msg_from_devtools(inner_msg),
                    FromImageCache(inner_msg) => self.handle_msg_from_image_cache(inner_msg),
                }

                None
            });

            // https://html.spec.whatwg.org/multipage/#event-loop-processing-model step 6
            self.perform_a_microtask_checkpoint();

            if let Some(retval) = result {
                return retval
            }
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
            let window = document.window();
            let pending_reflows = window.get_pending_reflow_count();
            if pending_reflows > 0 {
                window.reflow(ReflowGoal::Full, ReflowReason::ImageLoaded);
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

    fn categorize_msg(&self, msg: &MixedMessage) -> ScriptThreadEventCategory {
        match *msg {
            MixedMessage::FromConstellation(ref inner_msg) => {
                match *inner_msg {
                    ConstellationControlMsg::SendEvent(_, _) =>
                        ScriptThreadEventCategory::DomEvent,
                    _ => ScriptThreadEventCategory::ConstellationMsg
                }
            },
            // TODO https://github.com/servo/servo/issues/18998
            MixedMessage::FromDevtools(_) => ScriptThreadEventCategory::DevtoolsMsg,
            MixedMessage::FromImageCache(_) => ScriptThreadEventCategory::ImageCacheMsg,
            MixedMessage::FromScript(ref inner_msg) => {
                match *inner_msg {
                    MainThreadScriptMsg::Common(CommonScriptMsg::Task(category, ..)) => {
                        category
                    },
                    MainThreadScriptMsg::RegisterPaintWorklet { .. } => {
                        ScriptThreadEventCategory::WorkletEvent
                    },
                    _ => ScriptThreadEventCategory::ScriptEvent,
                }
            },
            MixedMessage::FromScheduler(_) => ScriptThreadEventCategory::TimerEvent
        }
    }

    fn message_to_pipeline(&self, msg: &MixedMessage) -> Option<PipelineId> {
        use script_traits::ConstellationControlMsg::*;
        match *msg {
            MixedMessage::FromConstellation(ref inner_msg) => {
                match *inner_msg {
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
                    Navigate(id, ..) => Some(id),
                    PostMessage(id, ..) => Some(id),
                    UpdatePipelineId(_, _, id, _) => Some(id),
                    UpdateHistoryState(id, ..) => Some(id),
                    RemoveHistoryStates(id, ..) => Some(id),
                    FocusIFrame(id, ..) => Some(id),
                    WebDriverScriptCommand(id, ..) => Some(id),
                    TickAllAnimations(id) => Some(id),
                    // FIXME https://github.com/servo/servo/issues/15079
                    TransitionEnd(..) => None,
                    WebFontLoaded(id) => Some(id),
                    DispatchIFrameLoadEvent { target: _, parent: id, child: _ } => Some(id),
                    DispatchStorageEvent(id, ..) => Some(id),
                    ReportCSSError(id, ..) => Some(id),
                    Reload(id, ..) => Some(id),
                    WebVREvents(id, ..) => Some(id),
                    PaintMetric(..) => None,
                }
            },
            MixedMessage::FromDevtools(_) => None,
            MixedMessage::FromScript(ref inner_msg) => {
                match *inner_msg {
                    MainThreadScriptMsg::Common(CommonScriptMsg::Task(_, _, pipeline_id)) =>
                        pipeline_id,
                    MainThreadScriptMsg::Common(CommonScriptMsg::CollectReports(_)) => None,
                    MainThreadScriptMsg::Navigate(pipeline_id, ..) => Some(pipeline_id),
                    MainThreadScriptMsg::WorkletLoaded(pipeline_id) => Some(pipeline_id),
                    MainThreadScriptMsg::RegisterPaintWorklet { pipeline_id, .. } => Some(pipeline_id),
                    MainThreadScriptMsg::DispatchJobQueue { .. }  => None,
                }
            },
            MixedMessage::FromImageCache((pipeline_id, _)) => Some(pipeline_id),
            MixedMessage::FromScheduler(ref timer_event) => {
                let TimerEvent(source, _) = *timer_event;
                match source {
                    TimerSource::FromWindow(pipeline_id) => Some(pipeline_id),
                    _ => None
                }
            }
        }
    }

    fn profile_event<F, R>(&self, category: ScriptThreadEventCategory, pipeline_id: Option<PipelineId>, f: F) -> R
        where F: FnOnce() -> R {
        let start = precise_time_ns();
        let value = if opts::get().profile_script_events {
            let profiler_cat = match category {
                ScriptThreadEventCategory::AttachLayout => ProfilerCategory::ScriptAttachLayout,
                ScriptThreadEventCategory::ConstellationMsg => ProfilerCategory::ScriptConstellationMsg,
                ScriptThreadEventCategory::DevtoolsMsg => ProfilerCategory::ScriptDevtoolsMsg,
                ScriptThreadEventCategory::DocumentEvent => ProfilerCategory::ScriptDocumentEvent,
                ScriptThreadEventCategory::DomEvent => ProfilerCategory::ScriptDomEvent,
                ScriptThreadEventCategory::FileRead => ProfilerCategory::ScriptFileRead,
                ScriptThreadEventCategory::FormPlannedNavigation => ProfilerCategory::ScriptPlannedNavigation,
                ScriptThreadEventCategory::ImageCacheMsg => ProfilerCategory::ScriptImageCacheMsg,
                ScriptThreadEventCategory::InputEvent => ProfilerCategory::ScriptInputEvent,
                ScriptThreadEventCategory::NetworkEvent => ProfilerCategory::ScriptNetworkEvent,
                ScriptThreadEventCategory::Resize => ProfilerCategory::ScriptResize,
                ScriptThreadEventCategory::ScriptEvent => ProfilerCategory::ScriptEvent,
                ScriptThreadEventCategory::SetScrollState => {
                    ProfilerCategory::ScriptSetScrollState
                }
                ScriptThreadEventCategory::UpdateReplacedElement => {
                    ProfilerCategory::ScriptUpdateReplacedElement
                }
                ScriptThreadEventCategory::StylesheetLoad => ProfilerCategory::ScriptStylesheetLoad,
                ScriptThreadEventCategory::SetViewport => ProfilerCategory::ScriptSetViewport,
                ScriptThreadEventCategory::TimerEvent => ProfilerCategory::ScriptTimerEvent,
                ScriptThreadEventCategory::WebSocketEvent => ProfilerCategory::ScriptWebSocketEvent,
                ScriptThreadEventCategory::WebVREvent => ProfilerCategory::ScriptWebVREvent,
                ScriptThreadEventCategory::WorkerEvent => ProfilerCategory::ScriptWorkerEvent,
                ScriptThreadEventCategory::WorkletEvent => ProfilerCategory::ScriptWorkletEvent,
                ScriptThreadEventCategory::ServiceWorkerEvent => ProfilerCategory::ScriptServiceWorkerEvent,
                ScriptThreadEventCategory::EnterFullscreen => ProfilerCategory::ScriptEnterFullscreen,
                ScriptThreadEventCategory::ExitFullscreen => ProfilerCategory::ScriptExitFullscreen,
                ScriptThreadEventCategory::PerformanceTimelineTask => ProfilerCategory::ScriptPerformanceEvent,
            };
            profile(profiler_cat, None, self.time_profiler_chan.clone(), f)
        } else {
            f()
        };
        let end = precise_time_ns();
        for (doc_id, doc) in self.documents.borrow().iter() {
           if let Some(pipeline_id) = pipeline_id {
                if pipeline_id == doc_id && end - start > MAX_TASK_NS {
                    if opts::get().print_pwm {
                        println!("Task took longer than max allowed ({:?}) {:?}", category, end - start);
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
            ConstellationControlMsg::NavigationResponse(id, fetch_data) => {
                match fetch_data {
                    FetchResponseMsg::ProcessResponse(metadata) => self.handle_fetch_metadata(id, metadata),
                    FetchResponseMsg::ProcessResponseChunk(chunk) => self.handle_fetch_chunk(id, chunk),
                    FetchResponseMsg::ProcessResponseEOF(eof) => self.handle_fetch_eof(id, eof),
                    _ => unreachable!(),
                };
            },
            ConstellationControlMsg::Navigate(parent_pipeline_id, browsing_context_id, load_data, replace) =>
                self.handle_navigate(parent_pipeline_id, Some(browsing_context_id), load_data, replace),
            ConstellationControlMsg::UnloadDocument(pipeline_id) =>
                self.handle_unload_document(pipeline_id),
            ConstellationControlMsg::SendEvent(id, event) =>
                self.handle_event(id, event),
            ConstellationControlMsg::ResizeInactive(id, new_size) =>
                self.handle_resize_inactive_msg(id, new_size),
            ConstellationControlMsg::GetTitle(pipeline_id) =>
                self.handle_get_title_msg(pipeline_id),
            ConstellationControlMsg::SetDocumentActivity(pipeline_id, activity) =>
                self.handle_set_document_activity_msg(pipeline_id, activity),
            ConstellationControlMsg::ChangeFrameVisibilityStatus(pipeline_id, visible) =>
                self.handle_visibility_change_msg(pipeline_id, visible),
            ConstellationControlMsg::NotifyVisibilityChange(parent_pipeline_id, browsing_context_id, visible) =>
                self.handle_visibility_change_complete_msg(parent_pipeline_id, browsing_context_id, visible),
            ConstellationControlMsg::PostMessage(pipeline_id, origin, data) =>
                self.handle_post_message_msg(pipeline_id, origin, data),
            ConstellationControlMsg::UpdatePipelineId(parent_pipeline_id,
                                                      browsing_context_id,
                                                      new_pipeline_id,
                                                      reason) =>
                self.handle_update_pipeline_id(parent_pipeline_id,
                                               browsing_context_id,
                                               new_pipeline_id,
                                               reason),
            ConstellationControlMsg::UpdateHistoryState(pipeline_id, history_state_id, url) =>
                self.handle_update_history_state_msg(pipeline_id, history_state_id, url),
            ConstellationControlMsg::RemoveHistoryStates(pipeline_id, history_states) =>
                self.handle_remove_history_states(pipeline_id, history_states),
            ConstellationControlMsg::FocusIFrame(parent_pipeline_id, frame_id) =>
                self.handle_focus_iframe_msg(parent_pipeline_id, frame_id),
            ConstellationControlMsg::WebDriverScriptCommand(pipeline_id, msg) =>
                self.handle_webdriver_msg(pipeline_id, msg),
            ConstellationControlMsg::TickAllAnimations(pipeline_id) =>
                self.handle_tick_all_animations(pipeline_id),
            ConstellationControlMsg::TransitionEnd(unsafe_node, name, duration) =>
                self.handle_transition_event(unsafe_node, name, duration),
            ConstellationControlMsg::WebFontLoaded(pipeline_id) =>
                self.handle_web_font_loaded(pipeline_id),
            ConstellationControlMsg::DispatchIFrameLoadEvent {
                target: browsing_context_id, parent: parent_id, child: child_id } =>
                self.handle_iframe_load_event(parent_id, browsing_context_id, child_id),
            ConstellationControlMsg::DispatchStorageEvent(pipeline_id, storage, url, key, old_value, new_value) =>
                self.handle_storage_event(pipeline_id, storage, url, key, old_value, new_value),
            ConstellationControlMsg::ReportCSSError(pipeline_id, filename, line, column, msg) =>
                self.handle_css_error_reporting(pipeline_id, filename, line, column, msg),
            ConstellationControlMsg::Reload(pipeline_id) =>
                self.handle_reload(pipeline_id),
            ConstellationControlMsg::ExitPipeline(pipeline_id, discard_browsing_context) =>
                self.handle_exit_pipeline_msg(pipeline_id, discard_browsing_context),
            ConstellationControlMsg::WebVREvents(pipeline_id, events) =>
                self.handle_webvr_events(pipeline_id, events),
            ConstellationControlMsg::PaintMetric(pipeline_id, metric_type, metric_value) =>
                self.handle_paint_metric(pipeline_id, metric_type, metric_value),
            msg @ ConstellationControlMsg::AttachLayout(..) |
            msg @ ConstellationControlMsg::Viewport(..) |
            msg @ ConstellationControlMsg::SetScrollState(..) |
            msg @ ConstellationControlMsg::Resize(..) |
            msg @ ConstellationControlMsg::ExitScriptThread =>
                      panic!("should have handled {:?} already", msg),
        }
    }

    fn handle_msg_from_script(&self, msg: MainThreadScriptMsg) {
        match msg {
            MainThreadScriptMsg::Navigate(parent_pipeline_id, load_data, replace) => {
                self.handle_navigate(parent_pipeline_id, None, load_data, replace)
            },
            MainThreadScriptMsg::Common(CommonScriptMsg::Task(_, task, _)) => {
                task.run_box()
            }
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
            } => {
                self.handle_register_paint_worklet(
                    pipeline_id,
                    name,
                    properties,
                    painter,
                )
            },
            MainThreadScriptMsg::DispatchJobQueue { scope_url } => {
                self.job_queue_map.run_job(scope_url, self)
            }
        }
    }

    fn handle_timer_event(&self, timer_event: TimerEvent) {
        let TimerEvent(source, id) = timer_event;

        let pipeline_id = match source {
            TimerSource::FromWindow(pipeline_id) => pipeline_id,
            TimerSource::FromWorker => panic!("Worker timeouts must not be sent to script thread"),
        };

        let window = self.documents.borrow().find_window(pipeline_id);
        let window = match window {
            Some(w) => w,
            None => return warn!("Received fire timer msg for a closed pipeline {}.", pipeline_id),
        };

        window.handle_fire_timer(id);
    }

    fn handle_msg_from_devtools(&self, msg: DevtoolScriptControlMsg) {
        let documents = self.documents.borrow();
        match msg {
            DevtoolScriptControlMsg::EvaluateJS(id, s, reply) => {
                match documents.find_window(id) {
                    Some(window) => devtools::handle_evaluate_js(window.upcast(), s, reply),
                    None => return warn!("Message sent to closed pipeline {}.", id),
                }
            },
            DevtoolScriptControlMsg::GetRootNode(id, reply) =>
                devtools::handle_get_root_node(&*documents, id, reply),
            DevtoolScriptControlMsg::GetDocumentElement(id, reply) =>
                devtools::handle_get_document_element(&*documents, id, reply),
            DevtoolScriptControlMsg::GetChildren(id, node_id, reply) =>
                devtools::handle_get_children(&*documents, id, node_id, reply),
            DevtoolScriptControlMsg::GetLayout(id, node_id, reply) =>
                devtools::handle_get_layout(&*documents, id, node_id, reply),
            DevtoolScriptControlMsg::GetCachedMessages(id, message_types, reply) =>
                devtools::handle_get_cached_messages(id, message_types, reply),
            DevtoolScriptControlMsg::ModifyAttribute(id, node_id, modifications) =>
                devtools::handle_modify_attribute(&*documents, id, node_id, modifications),
            DevtoolScriptControlMsg::WantsLiveNotifications(id, to_send) => {
                match documents.find_window(id) {
                    Some(window) => devtools::handle_wants_live_notifications(window.upcast(), to_send),
                    None => return warn!("Message sent to closed pipeline {}.", id),
                }
            },
            DevtoolScriptControlMsg::SetTimelineMarkers(id, marker_types, reply) =>
                devtools::handle_set_timeline_markers(&*documents, id, marker_types, reply),
            DevtoolScriptControlMsg::DropTimelineMarkers(id, marker_types) =>
                devtools::handle_drop_timeline_markers(&*documents, id, marker_types),
            DevtoolScriptControlMsg::RequestAnimationFrame(id, name) =>
                devtools::handle_request_animation_frame(&*documents, id, name),
            DevtoolScriptControlMsg::Reload(id) =>
                devtools::handle_reload(&*documents, id),
        }
    }

    fn handle_msg_from_image_cache(&self, (id, response): (PipelineId, PendingImageResponse)) {
        let window = self.documents.borrow().find_window(id);
        if let Some(ref window) = window {
            window.pending_image_notification(response);
        }
    }

    fn handle_webdriver_msg(&self, pipeline_id: PipelineId, msg: WebDriverScriptCommand) {
        let documents = self.documents.borrow();
        match msg {
            WebDriverScriptCommand::AddCookie(params, reply) =>
                webdriver_handlers::handle_add_cookie(&*documents, pipeline_id, params, reply),
            WebDriverScriptCommand::ExecuteScript(script, reply) =>
                webdriver_handlers::handle_execute_script(&*documents, pipeline_id, script, reply),
            WebDriverScriptCommand::FindElementCSS(selector, reply) =>
                webdriver_handlers::handle_find_element_css(&*documents, pipeline_id, selector, reply),
            WebDriverScriptCommand::FindElementsCSS(selector, reply) =>
                webdriver_handlers::handle_find_elements_css(&*documents, pipeline_id, selector, reply),
            WebDriverScriptCommand::FocusElement(element_id, reply) =>
                webdriver_handlers::handle_focus_element(&*documents, pipeline_id, element_id, reply),
            WebDriverScriptCommand::GetActiveElement(reply) =>
                webdriver_handlers::handle_get_active_element(&*documents, pipeline_id, reply),
            WebDriverScriptCommand::GetCookies(reply) =>
                webdriver_handlers::handle_get_cookies(&*documents, pipeline_id, reply),
            WebDriverScriptCommand::GetCookie(name, reply) =>
                webdriver_handlers::handle_get_cookie(&*documents, pipeline_id, name, reply),
            WebDriverScriptCommand::GetElementTagName(node_id, reply) =>
                webdriver_handlers::handle_get_name(&*documents, pipeline_id, node_id, reply),
            WebDriverScriptCommand::GetElementAttribute(node_id, name, reply) =>
                webdriver_handlers::handle_get_attribute(&*documents, pipeline_id, node_id, name, reply),
            WebDriverScriptCommand::GetElementCSS(node_id, name, reply) =>
                webdriver_handlers::handle_get_css(&*documents, pipeline_id, node_id, name, reply),
            WebDriverScriptCommand::GetElementRect(node_id, reply) =>
                webdriver_handlers::handle_get_rect(&*documents, pipeline_id, node_id, reply),
            WebDriverScriptCommand::GetElementText(node_id, reply) =>
                webdriver_handlers::handle_get_text(&*documents, pipeline_id, node_id, reply),
            WebDriverScriptCommand::GetBrowsingContextId(webdriver_frame_id, reply) =>
                webdriver_handlers::handle_get_browsing_context_id(&*documents, pipeline_id, webdriver_frame_id, reply),
            WebDriverScriptCommand::GetUrl(reply) =>
                webdriver_handlers::handle_get_url(&*documents, pipeline_id, reply),
            WebDriverScriptCommand::IsEnabled(element_id, reply) =>
                webdriver_handlers::handle_is_enabled(&*documents, pipeline_id, element_id, reply),
            WebDriverScriptCommand::IsSelected(element_id, reply) =>
                webdriver_handlers::handle_is_selected(&*documents, pipeline_id, element_id, reply),
            WebDriverScriptCommand::GetTitle(reply) =>
                webdriver_handlers::handle_get_title(&*documents, pipeline_id, reply),
            WebDriverScriptCommand::ExecuteAsyncScript(script, reply) =>
                webdriver_handlers::handle_execute_async_script(&*documents, pipeline_id, script, reply),
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
            load.window_size = Some(size);
            return;
        }
        warn!("resize sent to nonexistent pipeline");
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

    fn handle_set_scroll_state(&self,
                               id: PipelineId,
                               scroll_states: &[(UntrustedNodeAddress, Vector2D<f32>)]) {
        let window = match { self.documents.borrow().find_window(id) } {
            Some(window) => window,
            None => return warn!("Set scroll state message sent to nonexistent pipeline: {:?}", id),
        };

        let mut scroll_offsets = HashMap::new();
        for &(node_address, ref scroll_offset) in scroll_states {
            if node_address == UntrustedNodeAddress(ptr::null()) {
                window.update_viewport_for_scroll(-scroll_offset.x, -scroll_offset.y);
            } else {
                scroll_offsets.insert(node_address, -*scroll_offset);
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
            content_process_shutdown_chan,
            layout_threads,
        } = new_layout_info;

        let layout_pair = channel();
        let layout_chan = layout_pair.0.clone();

        let msg = message::Msg::CreateLayoutThread(NewLayoutThreadInfo {
            id: new_pipeline_id,
            url: load_data.url.clone(),
            is_parent: false,
            layout_pair: layout_pair,
            pipeline_port: pipeline_port,
            constellation_chan: self.layout_to_constellation_chan.clone(),
            script_chan: self.control_chan.clone(),
            image_cache: self.image_cache.clone(),
            content_process_shutdown_chan: content_process_shutdown_chan,
            layout_threads: layout_threads,
            paint_time_metrics: PaintTimeMetrics::new(new_pipeline_id,
                                                      self.time_profiler_chan.clone(),
                                                      self.layout_to_constellation_chan.clone(),
                                                      self.control_chan.clone(),
                                                      load_data.url.clone()),
        });

        // Pick a layout thread, any layout thread
        let current_layout_chan = self.documents.borrow().iter().next()
            .map(|(_, document)| document.window().layout_chan().clone())
            .or_else(|| self.incomplete_loads.borrow().first().map(|load| load.layout_chan.clone()));

        match current_layout_chan {
            None => panic!("Layout attached to empty script thread."),
            // Tell the layout thread factory to actually spawn the thread.
            Some(layout_chan) => layout_chan.send(msg).unwrap(),
        };

        // Kick off the fetch for the new resource.
        let new_load = InProgressLoad::new(new_pipeline_id,
                                           browsing_context_id,
                                           top_level_browsing_context_id,
                                           parent_info,
                                           opener,
                                           layout_chan,
                                           window_size,
                                           load_data.url.clone(),
                                           origin);
        if load_data.url.as_str() == "about:blank" {
            self.start_page_load_about_blank(new_load, load_data.js_eval_result);
        } else {
            self.pre_page_load(new_load, load_data);
        }
    }

    fn collect_reports(&self, reports_chan: ReportsChan) {
        let documents = self.documents.borrow();
        let urls = itertools::join(documents.iter().map(|(_, d)| d.url().to_string()), ", ");
        let path_seg = format!("url({})", urls);

        let mut reports = vec![];
        reports.extend(get_reports(self.get_cx(), path_seg));
        reports_chan.send(reports);
    }

    /// Updates iframe element after a change in visibility
    fn handle_visibility_change_complete_msg(&self,
                                             parent_pipeline_id: PipelineId,
                                             browsing_context_id: BrowsingContextId,
                                             visible: bool)
    {
        let iframe = self.documents.borrow().find_iframe(parent_pipeline_id, browsing_context_id);
        if let Some(iframe) = iframe {
            iframe.change_visibility_status(visible);
        }
    }

    /// Handle visibility change message
    fn handle_visibility_change_msg(&self, id: PipelineId, visible: bool) {
        // Separate message sent since parent script thread could be different (Iframe of different
        // domain)
        self.script_sender.send((id, ScriptMsg::VisibilityChangeComplete(visible))).unwrap();

        let window = self.documents.borrow().find_window(id);
        match window {
            Some(window) => {
                window.alter_resource_utilization(visible);
                return;
            }
            None => {
                let mut loads = self.incomplete_loads.borrow_mut();
                if let Some(ref mut load) = loads.iter_mut().find(|load| load.pipeline_id == id) {
                    load.is_visible = visible;
                    return;
                }
            }
        }

        warn!("change visibility message sent to nonexistent pipeline");
    }

    /// Handles activity change message
    fn handle_set_document_activity_msg(&self, id: PipelineId, activity: DocumentActivity) {
        debug!("Setting activity of {} to be {:?} in {:?}.", id, activity, thread::current().name());
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

    fn handle_focus_iframe_msg(&self,
                               parent_pipeline_id: PipelineId,
                               browsing_context_id: BrowsingContextId) {
        let doc = self.documents.borrow().find_document(parent_pipeline_id).unwrap();
        let frame_element = doc.find_iframe(browsing_context_id);

        if let Some(ref frame_element) = frame_element {
            doc.begin_focus_transaction();
            doc.request_focus(frame_element.upcast());
            doc.commit_focus_transaction(FocusType::Parent);
        }
    }

    fn handle_post_message_msg(&self, pipeline_id: PipelineId, origin: Option<ImmutableOrigin>, data: Vec<u8>) {
        match { self.documents.borrow().find_window(pipeline_id) } {
            None => return warn!("postMessage after pipeline {} closed.", pipeline_id),
            Some(window) => window.post_message(origin, StructuredCloneData::Vector(data)),
        }
    }

    fn handle_unload_document(&self, pipeline_id: PipelineId) {
        let document = self.documents.borrow().find_document(pipeline_id);
        if let Some(document) = document {
            document.unload(false, false);
        }
    }

    fn handle_update_pipeline_id(&self,
                                 parent_pipeline_id: PipelineId,
                                 browsing_context_id: BrowsingContextId,
                                 new_pipeline_id: PipelineId,
                                 reason: UpdatePipelineIdReason) {
        let frame_element = self.documents.borrow().find_iframe(parent_pipeline_id, browsing_context_id);
        if let Some(frame_element) = frame_element {
            frame_element.update_pipeline_id(new_pipeline_id, reason);
        }
    }

    fn handle_update_history_state_msg(
        &self, pipeline_id: PipelineId,
        history_state_id: Option<HistoryStateId>,
        url: ServoUrl,
    ) {
        match { self.documents.borrow().find_window(pipeline_id) } {
            None => return warn!("update history state after pipeline {} closed.", pipeline_id),
            Some(window) => window.History().r().activate_state(history_state_id, url),
        }
    }

    fn handle_remove_history_states(&self, pipeline_id: PipelineId, history_states: Vec<HistoryStateId>) {
        match { self.documents.borrow().find_window(pipeline_id) } {
            None => return warn!("update history state after pipeline {} closed.", pipeline_id),
            Some(window) => window.History().r().remove_states(history_states),
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
    fn handle_page_headers_available(&self, id: &PipelineId,
                                     metadata: Option<Metadata>) -> Option<DomRoot<ServoParser>> {
        let idx = self.incomplete_loads.borrow().iter().position(|load| { load.pipeline_id == *id });
        // The matching in progress load structure may not exist if
        // the pipeline exited before the page load completed.
        match idx {
            Some(idx) => {
                // https://html.spec.whatwg.org/multipage/#process-a-navigate-response
                // 2. If response's status is 204 or 205, then abort these steps.
                match metadata {
                    Some(Metadata { status: Some((204 ... 205, _)), .. }) => {
                        self.script_sender
                            .send((id.clone(), ScriptMsg::AbortLoadUrl))
                            .unwrap();
                        return None;
                    },
                    _ => ()
                };

                let load = self.incomplete_loads.borrow_mut().remove(idx);
                metadata.map(|meta| self.load(meta, load))
            }
            None => {
                assert!(self.closed_pipelines.borrow().contains(id));
                None
            }
        }
    }

    pub fn handle_get_registration(&self, scope_url: &ServoUrl) -> Option<DomRoot<ServiceWorkerRegistration>> {
        let maybe_registration_ref = self.registration_map.borrow();
        maybe_registration_ref.get(scope_url).map(|x| DomRoot::from_ref(&**x))
    }

    pub fn handle_serviceworker_registration(&self,
                                         scope: &ServoUrl,
                                         registration: &ServiceWorkerRegistration,
                                         pipeline_id: PipelineId) {
        {
            let ref mut reg_ref = *self.registration_map.borrow_mut();
            // according to spec we should replace if an older registration exists for
            // same scope otherwise just insert the new one
            let _ = reg_ref.remove(scope);
            reg_ref.insert(scope.clone(), Dom::from_ref(registration));
        }

        // send ScopeThings to sw-manager
        let ref maybe_registration_ref = *self.registration_map.borrow();
        let maybe_registration = match maybe_registration_ref.get(scope) {
            Some(r) => r,
            None => return
        };
        let window = match { self.documents.borrow().find_window(pipeline_id) } {
            Some(window) => window,
            None => return warn!("Registration failed for {}", scope),
        };

        let script_url = maybe_registration.get_installed().get_script_url();
        let scope_things = ServiceWorkerRegistration::create_scope_things(window.upcast(), script_url);
        let _ = self.script_sender.send((pipeline_id, ScriptMsg::RegisterServiceWorker(scope_things, scope.clone())));
    }

    pub fn schedule_job_queue(&self, scope_url: ServoUrl) {
        let _ = self.chan.0.send(MainThreadScriptMsg::DispatchJobQueue { scope_url });
    }

    pub fn dom_manipulation_task_source(&self, pipeline_id: PipelineId) -> DOMManipulationTaskSource {
        DOMManipulationTaskSource(self.dom_manipulation_task_sender.clone(), pipeline_id)
    }

    pub fn performance_timeline_task_source(&self, pipeline_id: PipelineId) -> PerformanceTimelineTaskSource {
        PerformanceTimelineTaskSource(self.performance_timeline_task_sender.clone(), pipeline_id)
    }

    pub fn user_interaction_task_source(&self, pipeline_id: PipelineId) -> UserInteractionTaskSource {
        UserInteractionTaskSource(self.user_interaction_task_sender.clone(), pipeline_id)
    }

    pub fn networking_task_source(&self, pipeline_id: PipelineId) -> NetworkingTaskSource {
        NetworkingTaskSource(self.networking_task_sender.clone(), pipeline_id)
    }

    pub fn file_reading_task_source(&self, pipeline_id: PipelineId) -> FileReadingTaskSource {
        FileReadingTaskSource(self.file_reading_task_sender.clone(), pipeline_id)
    }

    pub fn remote_event_task_source(&self, pipeline_id: PipelineId) -> RemoteEventTaskSource {
        RemoteEventTaskSource(self.remote_event_task_sender.clone(), pipeline_id)
    }

    /// Handles a request for the window title.
    fn handle_get_title_msg(&self, pipeline_id: PipelineId) {
        let document = match { self.documents.borrow().find_document(pipeline_id) } {
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
        let idx = self.incomplete_loads.borrow().iter().position(|load| {
            load.pipeline_id == id
        });

        let document = self.documents.borrow_mut().remove(id);

        // We should never have a pipeline that's still an incomplete load,
        // but also has a Document.
        debug_assert!(idx.is_none() || document.is_none());

        // Remove any incomplete load.
        let chan = if let Some(idx) = idx {
            let load = self.incomplete_loads.borrow_mut().remove(idx);
            load.layout_chan.clone()
        } else if let Some(ref document) = document {
            document.window().layout_chan().clone()
        } else {
            return warn!("Exiting nonexistant pipeline {}.", id);
        };

        // We shut down layout before removing the document,
        // since layout might still be in the middle of laying it out.
        debug!("preparing to shut down layout for page {}", id);
        let (response_chan, response_port) = channel();
        chan.send(message::Msg::PrepareToExit(response_chan)).ok();
        let _ = response_port.recv();

        debug!("shutting down layout for page {}", id);
        chan.send(message::Msg::ExitNow).ok();
        self.script_sender.send((id, ScriptMsg::PipelineExited)).ok();

        // Now that layout is shut down, it's OK to remove the document.
        if let Some(document) = document {
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
                window.window_proxy().discard_browsing_context();
            }
            window.clear_js_runtime();
        }

        debug!("Exited pipeline {}.", id);
    }

    /// Handles a request to exit the script thread and shut down layout.
    fn handle_exit_script_thread_msg(&self) {
        debug!("Exiting script thread.");

        let mut pipeline_ids = Vec::new();
        pipeline_ids.extend(self.incomplete_loads.borrow().iter().next().map(|load| load.pipeline_id));
        pipeline_ids.extend(self.documents.borrow().iter().next().map(|(pipeline_id, _)| pipeline_id));

        for pipeline_id in pipeline_ids {
            self.handle_exit_pipeline_msg(pipeline_id, DiscardBrowsingContext::Yes);
        }

        debug!("Exited script thread.");
    }

    /// Handles when layout thread finishes all animation in one tick
    fn handle_tick_all_animations(&self, id: PipelineId) {
        let document = match { self.documents.borrow().find_document(id) } {
            Some(document) => document,
            None => return warn!("Message sent to closed pipeline {}.", id),
        };
        document.run_the_animation_frame_callbacks();
    }

    /// Handles firing of transition events.
    fn handle_transition_event(&self, unsafe_node: UntrustedNodeAddress, name: String, duration: f64) {
        let js_runtime = self.js_runtime.rt();
        let node = unsafe {
            from_untrusted_node_address(js_runtime, unsafe_node)
        };

        let idx = self.transitioning_nodes
            .borrow()
            .iter()
            .position(|n| &**n as *const _ == &*node as *const _);
        match idx {
            Some(idx) => {
                self.transitioning_nodes.borrow_mut().remove(idx);
            }
            None => {
                // If no index is found, we can't know whether this node is safe to use.
                // It's better not to fire a DOM event than crash.
                warn!("Ignoring transition end notification for unknown node.");
                return;
            }
        }

        let window = window_from_node(&*node);

        // Not quite the right thing - see #13865.
        node.dirty(NodeDamage::NodeStyleDamaged);

        if let Some(el) = node.downcast::<Element>() {
            if !el.has_css_layout_box() {
                return;
            }
        }

        let init = TransitionEventInit {
            parent: EventInit {
                bubbles: true,
                cancelable: false,
            },
            propertyName: DOMString::from(name),
            elapsedTime: Finite::new(duration as f32).unwrap(),
            // FIXME: Handle pseudo-elements properly
            pseudoElement: DOMString::new()
        };
        let transition_event = TransitionEvent::new(&window,
                                                    atom!("transitionend"),
                                                    &init);
        transition_event.upcast::<Event>().fire(node.upcast());
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
    fn handle_storage_event(&self, pipeline_id: PipelineId, storage_type: StorageType, url: ServoUrl,
                            key: Option<String>, old_value: Option<String>, new_value: Option<String>) {
        let window = match { self.documents.borrow().find_window(pipeline_id) } {
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
    fn handle_iframe_load_event(&self,
                                parent_id: PipelineId,
                                browsing_context_id: BrowsingContextId,
                                child_id: PipelineId) {
        let iframe = self.documents.borrow().find_iframe(parent_id, browsing_context_id);
        match iframe {
            Some(iframe) => iframe.iframe_load_event_steps(child_id),
            None => warn!("Message sent to closed pipeline {}.", parent_id),
        }
    }

    fn ask_constellation_for_browsing_context_id(&self, pipeline_id: PipelineId) -> Option<BrowsingContextId> {
        let (result_sender, result_receiver) = ipc::channel().unwrap();
        let msg = ScriptMsg::GetBrowsingContextId(pipeline_id, result_sender);
        self.script_sender.send((pipeline_id, msg)).expect("Failed to send to constellation.");
        result_receiver.recv().expect("Failed to get frame id from constellation.")
    }

    fn ask_constellation_for_parent_info(&self, pipeline_id: PipelineId) -> Option<PipelineId> {
        let (result_sender, result_receiver) = ipc::channel().unwrap();
        let msg = ScriptMsg::GetParentInfo(pipeline_id, result_sender);
        self.script_sender.send((pipeline_id, msg)).expect("Failed to send to constellation.");
        result_receiver.recv().expect("Failed to get frame id from constellation.")
    }

    fn ask_constellation_for_top_level_info(&self,
                                            sender_pipeline: PipelineId,
                                            browsing_context_id: BrowsingContextId)
                                            -> Option<TopLevelBrowsingContextId> {
        let (result_sender, result_receiver) = ipc::channel().unwrap();
        let msg = ScriptMsg::GetTopForBrowsingContext(browsing_context_id, result_sender);
        self.script_sender.send((sender_pipeline, msg)).expect("Failed to send to constellation.");
        result_receiver.recv().expect("Failed to get top-level id from constellation.")
    }

    // Get the browsing context for a pipeline that may exist in another
    // script thread.  If the browsing context already exists in the
    // `window_proxies` map, we return it, otherwise we recursively
    // get the browsing context for the parent if there is one,
    // construct a new dissimilar-origin browsing context, add it
    // to the `window_proxies` map, and return it.
    fn remote_window_proxy(&self,
                           global_to_clone: &GlobalScope,
                           top_level_browsing_context_id: TopLevelBrowsingContextId,
                           pipeline_id: PipelineId,
                           opener: Option<BrowsingContextId>)
                           -> Option<DomRoot<WindowProxy>>
    {
        let browsing_context_id = self.ask_constellation_for_browsing_context_id(pipeline_id)?;
        if let Some(window_proxy) = self.window_proxies.borrow().get(&browsing_context_id) {
            return Some(DomRoot::from_ref(window_proxy));
        }
        let parent = self.ask_constellation_for_parent_info(pipeline_id).and_then(|parent_id| {
            self.remote_window_proxy(global_to_clone, top_level_browsing_context_id, parent_id, opener)
        });
        let window_proxy = WindowProxy::new_dissimilar_origin(global_to_clone,
                                                              browsing_context_id,
                                                              top_level_browsing_context_id,
                                                              parent.r(),
                                                              opener);
        self.window_proxies.borrow_mut().insert(browsing_context_id, Dom::from_ref(&*window_proxy));
        Some(window_proxy)
    }

    // Get the browsing context for a pipeline that exists in this
    // script thread.  If the browsing context already exists in the
    // `window_proxies` map, we return it, otherwise we recursively
    // get the browsing context for the parent if there is one,
    // construct a new similar-origin browsing context, add it
    // to the `window_proxies` map, and return it.
    fn local_window_proxy(&self,
                          window: &Window,
                          browsing_context_id: BrowsingContextId,
                          top_level_browsing_context_id: TopLevelBrowsingContextId,
                          parent_info: Option<PipelineId>,
                          opener: Option<BrowsingContextId>)
                          -> DomRoot<WindowProxy>
    {
        if let Some(window_proxy) = self.window_proxies.borrow().get(&browsing_context_id) {
            window_proxy.set_currently_active(&*window);
            return DomRoot::from_ref(window_proxy);
        }
        let iframe = parent_info.and_then(|parent_id| {
            self.documents.borrow().find_iframe(parent_id, browsing_context_id)
        });
        let parent = match (parent_info, iframe.as_ref()) {
            (_, Some(iframe)) => Some(window_from_node(&**iframe).window_proxy()),
            (Some(parent_id), _) => self.remote_window_proxy(window.upcast(),
                                                             top_level_browsing_context_id,
                                                             parent_id,
                                                             opener),
            _ => None,
        };
        let window_proxy = WindowProxy::new(&window,
                                            browsing_context_id,
                                            top_level_browsing_context_id,
                                            iframe.r().map(Castable::upcast),
                                            parent.r(),
                                            opener);
        self.window_proxies.borrow_mut().insert(browsing_context_id, Dom::from_ref(&*window_proxy));
        window_proxy
    }

    /// The entry point to document loading. Defines bindings, sets up the window and document
    /// objects, parses HTML and CSS, and kicks off initial layout.
    fn load(&self, metadata: Metadata, incomplete: InProgressLoad) -> DomRoot<ServoParser> {
        let final_url = metadata.final_url.clone();
        {
            // send the final url to the layout thread.
            incomplete.layout_chan
                      .send(message::Msg::SetFinalUrl(final_url.clone()))
                      .unwrap();

            // update the pipeline url
            self.script_sender
                .send((incomplete.pipeline_id, ScriptMsg::SetFinalUrl(final_url.clone())))
                .unwrap();
        }
        debug!("ScriptThread: loading {} on pipeline {:?}", incomplete.url, incomplete.pipeline_id);

        let MainThreadScriptChan(ref sender) = self.chan;
        let HistoryTraversalTaskSource(ref history_sender) = self.history_traversal_task_source;

        let (ipc_timer_event_chan, ipc_timer_event_port) = ipc::channel().unwrap();
        ROUTER.route_ipc_receiver_to_mpsc_sender(ipc_timer_event_port,
                                                 self.timer_event_chan.clone());

        let origin = if final_url.as_str() == "about:blank" {
            incomplete.origin.clone()
        } else {
            MutableOrigin::new(final_url.origin())
        };

        let script_to_constellation_chan = ScriptToConstellationChan {
            sender: self.script_sender.clone(),
            pipeline_id: incomplete.pipeline_id,
        };

        // Create the window and document objects.
        let window = Window::new(
            self.js_runtime.clone(),
            MainThreadScriptChan(sender.clone()),
            self.dom_manipulation_task_source(incomplete.pipeline_id),
            self.user_interaction_task_source(incomplete.pipeline_id),
            self.networking_task_source(incomplete.pipeline_id),
            HistoryTraversalTaskSource(history_sender.clone()),
            self.file_reading_task_source(incomplete.pipeline_id),
            self.performance_timeline_task_source(incomplete.pipeline_id).clone(),
            self.remote_event_task_source(incomplete.pipeline_id),
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
            ipc_timer_event_chan,
            incomplete.layout_chan,
            incomplete.pipeline_id,
            incomplete.parent_info,
            incomplete.window_size,
            origin,
            incomplete.navigation_start,
            incomplete.navigation_start_precise,
            self.webgl_chan.as_ref().map(|chan| chan.channel()),
            self.webvr_chan.clone(),
            self.microtask_queue.clone(),
            self.webrender_document,
        );

        // Initialize the browsing context for the window.
        let window_proxy = self.local_window_proxy(&window,
                                                   incomplete.browsing_context_id,
                                                   incomplete.top_level_browsing_context_id,
                                                   incomplete.parent_info,
                                                   incomplete.opener);
        window.init_window_proxy(&window_proxy);

        let last_modified = metadata.headers.as_ref().and_then(|headers| {
            headers.get().map(|&LastModified(HttpDate(ref tm))| dom_last_modified(tm))
        });

        let content_type = metadata.content_type
                                   .as_ref()
                                   .map(|&Serde(ContentType(ref mimetype))| mimetype.clone());

        let loader = DocumentLoader::new_with_threads(self.resource_threads.clone(),
                                                      Some(final_url.clone()));

        let is_html_document = match metadata.content_type {
            Some(Serde(ContentType(Mime(TopLevel::Application, SubLevel::Ext(ref sub_level), _))))
                if sub_level.ends_with("+xml") => IsHTMLDocument::NonHTMLDocument,

            Some(Serde(ContentType(Mime(TopLevel::Application, SubLevel::Xml, _)))) |
            Some(Serde(ContentType(Mime(TopLevel::Text, SubLevel::Xml, _)))) => IsHTMLDocument::NonHTMLDocument,

            _ => IsHTMLDocument::HTMLDocument,
        };

        let referrer = match metadata.referrer {
            Some(ref referrer) => Some(referrer.clone().into_string()),
            None => None,
        };

        let referrer_policy = metadata.headers
                                      .as_ref()
                                      .map(Serde::deref)
                                      .and_then(Headers::get::<ReferrerPolicyHeader>)
                                      .map(ReferrerPolicy::from);

        let document = Document::new(&window,
                                     HasBrowsingContext::Yes,
                                     Some(final_url.clone()),
                                     incomplete.origin,
                                     is_html_document,
                                     content_type,
                                     last_modified,
                                     incomplete.activity,
                                     DocumentSource::FromParser,
                                     loader,
                                     referrer,
                                     referrer_policy,
                                     incomplete.canceller);
        document.set_ready_state(DocumentReadyState::Loading);

        self.documents.borrow_mut().insert(incomplete.pipeline_id, &*document);

        window.init_document(&document);

        self.script_sender
            .send((incomplete.pipeline_id, ScriptMsg::ActivateDocument))
            .unwrap();

        // Notify devtools that a new script global exists.
        self.notify_devtools(document.Title(), final_url.clone(), (incomplete.pipeline_id, None));

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

    fn notify_devtools(&self, title: DOMString, url: ServoUrl, ids: (PipelineId, Option<WorkerId>)) {
        if let Some(ref chan) = self.devtools_chan {
            let page_info = DevtoolsPageInfo {
                title: String::from(title),
                url: url,
            };
            chan.send(ScriptToDevtoolsControlMsg::NewGlobal(
                        ids,
                        self.devtools_sender.clone(),
                        page_info)).unwrap();
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
        match event {
            ResizeEvent(new_size, size_type) => {
                self.handle_resize_event(pipeline_id, new_size, size_type);
            }

            MouseButtonEvent(event_type, button, point, node_address, point_in_node) => {
                self.handle_mouse_event(
                    pipeline_id,
                    event_type,
                    button,
                    point,
                    node_address,
                    point_in_node
                );
            }

            MouseMoveEvent(point, node_address) => {
                let document = match { self.documents.borrow().find_document(pipeline_id) } {
                    Some(document) => document,
                    None => return warn!("Message sent to closed pipeline {}.", pipeline_id),
                };
                let window = document.window();

                // Get the previous target temporarily
                let prev_mouse_over_target = self.topmost_mouse_over_target.get();

                document.handle_mouse_move_event(self.js_runtime.rt(), point,
                                                 &self.topmost_mouse_over_target,
                                                 node_address);

                // Short-circuit if nothing changed
                if self.topmost_mouse_over_target.get() == prev_mouse_over_target {
                    return;
                }

                let mut state_already_changed = false;

                // Notify Constellation about the topmost anchor mouse over target.
                if let Some(target) = self.topmost_mouse_over_target.get() {
                    if let Some(anchor) = target.upcast::<Node>()
                                                .inclusive_ancestors()
                                                .filter_map(DomRoot::downcast::<HTMLAnchorElement>)
                                                .next() {
                        let status = anchor.upcast::<Element>()
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
                        if let Some(_) = target.upcast::<Node>()
                                               .inclusive_ancestors()
                                               .filter_map(DomRoot::downcast::<HTMLAnchorElement>)
                                               .next() {
                            let event = EmbedderMsg::Status(None);
                            window.send_to_embedder(event);
                        }
                    }
                }
            }
            TouchEvent(event_type, identifier, point, node_address) => {
                let touch_result = self.handle_touch_event(
                    pipeline_id,
                    event_type,
                    identifier,
                    point,
                    node_address
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
                    }
                    _ => {
                        // TODO: Calling preventDefault on a touchup event should prevent clicks.
                    }
                }
            }

            KeyEvent(ch, key, state, modifiers) => {
                let document = match { self.documents.borrow().find_document(pipeline_id) } {
                    Some(document) => document,
                    None => return warn!("Message sent to closed pipeline {}.", pipeline_id),
                };
                document.dispatch_key_event(ch, key, state, modifiers);
            }
        }
    }

    fn handle_mouse_event(
        &self,
        pipeline_id: PipelineId,
        mouse_event_type: MouseEventType,
        button: MouseButton,
        point: Point2D<f32>,
        node_address: Option<UntrustedNodeAddress>,
        point_in_node: Option<Point2D<f32>>
    ) {
        let document = match { self.documents.borrow().find_document(pipeline_id) } {
            Some(document) => document,
            None => return warn!("Message sent to closed pipeline {}.", pipeline_id),
        };
        document.handle_mouse_event(
            self.js_runtime.rt(),
            button,
            point,
            mouse_event_type,
            node_address,
            point_in_node
        );
    }

    fn handle_touch_event(
        &self,
        pipeline_id: PipelineId,
        event_type: TouchEventType,
        identifier: TouchId,
        point: Point2D<f32>,
        node_address: Option<UntrustedNodeAddress>
    ) -> TouchEventResult {
        let document = match { self.documents.borrow().find_document(pipeline_id) } {
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
            node_address
        )
    }

    /// <https://html.spec.whatwg.org/multipage/#navigating-across-documents>
    /// The entry point for content to notify that a new load has been requested
    /// for the given pipeline (specifically the "navigate" algorithm).
    fn handle_navigate(&self, parent_pipeline_id: PipelineId,
                              browsing_context_id: Option<BrowsingContextId>,
                              mut load_data: LoadData,
                              replace: bool) {
        let is_javascript = load_data.url.scheme() == "javascript";
        if is_javascript {
            let window = self.documents.borrow().find_window(parent_pipeline_id);
            if let Some(window) = window {
                ScriptThread::eval_js_url(window.upcast::<GlobalScope>(), &mut load_data);
            }
        }

        match browsing_context_id {
            Some(browsing_context_id) => {
                let iframe = self.documents.borrow().find_iframe(parent_pipeline_id, browsing_context_id);
                if let Some(iframe) = iframe {
                    iframe.navigate_or_reload_child_browsing_context(Some(load_data), NavigationType::Regular, replace);
                }
            }
            None => {
                self.script_sender
                    .send((parent_pipeline_id, ScriptMsg::LoadUrl(load_data, replace)))
                    .unwrap();
            }
        }
    }

    pub fn eval_js_url(global_scope: &GlobalScope, load_data: &mut LoadData) {
        // Turn javascript: URL into JS code to eval, according to the steps in
        // https://html.spec.whatwg.org/multipage/#javascript-protocol

        // This slice of the URL’s serialization is equivalent to (5.) to (7.):
        // Start with the scheme data of the parsed URL;
        // append question mark and query component, if any;
        // append number sign and fragment component if any.
        let encoded = &load_data.url.clone()[Position::BeforePath..];

        // Percent-decode (8.) and UTF-8 decode (9.)
        let script_source = percent_decode(encoded.as_bytes()).decode_utf8_lossy();

        // Script source is ready to be evaluated (11.)
        let _ac = JSAutoCompartment::new(global_scope.get_cx(), global_scope.reflector().get_jsobject().get());
        rooted!(in(global_scope.get_cx()) let mut jsval = UndefinedValue());
        global_scope.evaluate_js_on_global_with_result(&script_source, jsval.handle_mut());

        load_data.js_eval_result = if jsval.get().is_string() {
            unsafe {
                let strval = DOMString::from_jsval(global_scope.get_cx(),
                                                   jsval.handle(),
                                                   StringificationBehavior::Empty);
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

    fn handle_resize_event(&self, pipeline_id: PipelineId, new_size: WindowSizeData, size_type: WindowSizeType) {
        let document = match { self.documents.borrow().find_document(pipeline_id) } {
            Some(document) => document,
            None => return warn!("Message sent to closed pipeline {}.", pipeline_id),
        };

        let window = document.window();
        window.set_window_size(new_size);
        window.force_reflow(ReflowGoal::Full, ReflowReason::WindowResize);

        // http://dev.w3.org/csswg/cssom-view/#resizing-viewports
        if size_type == WindowSizeType::Resize {
            let uievent = UIEvent::new(&window,
                                       DOMString::from("resize"), EventBubbles::DoesNotBubble,
                                       EventCancelable::NotCancelable, Some(&window),
                                       0i32);
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
        let req_init = RequestInit {
            url: load_data.url.clone(),
            method: load_data.method,
            destination: Destination::Document,
            credentials_mode: CredentialsMode::Include,
            use_url_credentials: true,
            pipeline_id: Some(id),
            referrer_url: load_data.referrer_url,
            referrer_policy: load_data.referrer_policy,
            headers: load_data.headers,
            body: load_data.data,
            redirect_mode: RedirectMode::Manual,
            origin: incomplete.origin.immutable().clone(),
            .. RequestInit::default()
        };

        let context = ParserContext::new(id, load_data.url);
        self.incomplete_parser_contexts.borrow_mut().push((id, context));

        let cancel_chan = incomplete.canceller.initialize();

        self.script_sender.send((id, ScriptMsg::InitiateNavigateRequest(req_init, cancel_chan))).unwrap();
        self.incomplete_loads.borrow_mut().push(incomplete);
    }

    fn handle_fetch_metadata(&self, id: PipelineId, fetch_metadata: Result<FetchMetadata, NetworkError>) {
        match fetch_metadata {
            Ok(_) => {},
            Err(ref e) => warn!("Network error: {:?}", e),
        };
        let mut incomplete_parser_contexts = self.incomplete_parser_contexts.borrow_mut();
        let parser = incomplete_parser_contexts.iter_mut().find(|&&mut (pipeline_id, _)| pipeline_id == id);
        if let Some(&mut (_, ref mut ctxt)) = parser {
            ctxt.process_response(fetch_metadata);
        }
    }

    fn handle_fetch_chunk(&self, id: PipelineId, chunk: Vec<u8>) {
        let mut incomplete_parser_contexts = self.incomplete_parser_contexts.borrow_mut();
        let parser = incomplete_parser_contexts.iter_mut().find(|&&mut (pipeline_id, _)| pipeline_id == id);
        if let Some(&mut (_, ref mut ctxt)) = parser {
            ctxt.process_response_chunk(chunk);
        }
    }

    fn handle_fetch_eof(&self, id: PipelineId, eof: Result<(), NetworkError>) {
        let idx = self.incomplete_parser_contexts.borrow().iter().position(|&(pipeline_id, _)| {
            pipeline_id == id
        });
        if let Some(idx) = idx {
            let (_, mut ctxt) = self.incomplete_parser_contexts.borrow_mut().remove(idx);
            ctxt.process_response_eof(eof);
        }
    }

    /// Synchronously fetch `about:blank`. Stores the `InProgressLoad`
    /// argument until a notification is received that the fetch is complete.
    fn start_page_load_about_blank(&self, incomplete: InProgressLoad, js_eval_result: Option<JsEvalResult>) {
        let id = incomplete.pipeline_id;

        self.incomplete_loads.borrow_mut().push(incomplete);

        let url = ServoUrl::parse("about:blank").unwrap();
        let mut context = ParserContext::new(id, url.clone());

        let mut meta = Metadata::default(url);
        meta.set_content_type(Some(&mime!(Text / Html)));

        // If this page load is the result of a javascript scheme url, map
        // the evaluation result into a response.
        let chunk = match js_eval_result {
            Some(JsEvalResult::Ok(content)) => content,
            Some(JsEvalResult::NoContent) => {
                meta.status = Some((204, b"No Content".to_vec()));
                vec![]
            },
            None => vec![]
        };

        context.process_response(Ok(FetchMetadata::Unfiltered(meta)));
        context.process_response_chunk(chunk);
        context.process_response_eof(Ok(()));
    }

    fn handle_css_error_reporting(&self, pipeline_id: PipelineId, filename: String,
                                  line: u32, column: u32, msg: String) {
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
                    msg: msg
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

    fn handle_webvr_events(&self, pipeline_id: PipelineId, events: Vec<WebVREvent>) {
        let window = self.documents.borrow().find_window(pipeline_id);
        if let Some(window) = window {
            let vr = window.Navigator().Vr();
            vr.handle_webvr_events(events);
        }
    }

    fn handle_paint_metric(&self,
                           pipeline_id: PipelineId,
                           metric_type: ProgressiveWebMetricType,
                           metric_value: u64) {
        let window = self.documents.borrow().find_window(pipeline_id);
        if let Some(window) = window {
            let entry = PerformancePaintTiming::new(&window.upcast::<GlobalScope>(),
                                                    metric_type, metric_value);
            window.Performance().queue_entry(&entry.upcast::<PerformanceEntry>(),
                                             true /* buffer performance entry */);
        }
    }

    pub fn enqueue_microtask(job: Microtask) {
        SCRIPT_THREAD_ROOT.with(|root| {
            let script_thread = unsafe { &*root.get().unwrap() };
            script_thread.microtask_queue.enqueue(job);
        });
    }

    fn perform_a_microtask_checkpoint(&self) {
        self.microtask_queue.checkpoint(|id| self.documents.borrow().find_global(id))
    }
}

impl Drop for ScriptThread {
    fn drop(&mut self) {
        SCRIPT_THREAD_ROOT.with(|root| {
            root.set(None);
        });
    }
}

fn dom_last_modified(tm: &Tm) -> String {
    tm.to_local().strftime("%m/%d/%Y %H:%M:%S").unwrap().to_string()
}
