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

use bluetooth_traits::BluetoothRequest;
use devtools;
use devtools_traits::{DevtoolScriptControlMsg, DevtoolsPageInfo};
use devtools_traits::{ScriptToDevtoolsControlMsg, WorkerId};
use devtools_traits::CSSError;
use document_loader::DocumentLoader;
use dom::bindings::cell::DOMRefCell;
use dom::bindings::codegen::Bindings::CSSStyleDeclarationBinding::CSSStyleDeclarationMethods;
use dom::bindings::codegen::Bindings::DocumentBinding::{DocumentMethods, DocumentReadyState};
use dom::bindings::codegen::Bindings::EventBinding::EventInit;
use dom::bindings::codegen::Bindings::LocationBinding::LocationMethods;
use dom::bindings::codegen::Bindings::TransitionEventBinding::TransitionEventInit;
use dom::bindings::codegen::Bindings::WindowBinding::WindowMethods;
use dom::bindings::conversions::{ConversionResult, FromJSValConvertible, StringificationBehavior};
use dom::bindings::inheritance::Castable;
use dom::bindings::js::{JS, MutNullableJS, Root, RootCollection};
use dom::bindings::js::{RootCollectionPtr, RootedReference};
use dom::bindings::num::Finite;
use dom::bindings::refcounted::Trusted;
use dom::bindings::reflector::DomObject;
use dom::bindings::str::DOMString;
use dom::bindings::trace::JSTraceable;
use dom::bindings::utils::WRAP_CALLBACKS;
use dom::browsingcontext::BrowsingContext;
use dom::document::{Document, DocumentProgressHandler, DocumentSource, FocusType, IsHTMLDocument, TouchEventResult};
use dom::element::Element;
use dom::event::{Event, EventBubbles, EventCancelable};
use dom::globalscope::GlobalScope;
use dom::htmlanchorelement::HTMLAnchorElement;
use dom::htmliframeelement::HTMLIFrameElement;
use dom::node::{Node, NodeDamage, window_from_node};
use dom::serviceworker::TrustedServiceWorkerAddress;
use dom::serviceworkerregistration::ServiceWorkerRegistration;
use dom::servoparser::{ParserContext, ServoParser};
use dom::transitionevent::TransitionEvent;
use dom::uievent::UIEvent;
use dom::window::{ReflowReason, Window};
use dom::worker::TrustedWorkerAddress;
use euclid::Rect;
use euclid::point::Point2D;
use hyper::header::{ContentType, HttpDate, LastModified};
use hyper::header::ReferrerPolicy as ReferrerPolicyHeader;
use hyper::mime::{Mime, SubLevel, TopLevel};
use hyper_serde::Serde;
use ipc_channel::ipc::{self, IpcSender};
use ipc_channel::router::ROUTER;
use js::glue::GetWindowProxyClass;
use js::jsapi::{JSAutoCompartment, JSContext, JS_SetWrapObjectCallbacks};
use js::jsapi::{JSTracer, SetWindowProxyClass};
use js::jsval::UndefinedValue;
use js::rust::Runtime;
use layout_wrapper::ServoLayoutNode;
use mem::heap_size_of_self_and_children;
use msg::constellation_msg::{FrameId, FrameType, PipelineId, PipelineNamespace};
use net_traits::{CoreResourceMsg, FetchMetadata, FetchResponseListener};
use net_traits::{IpcSend, Metadata, ReferrerPolicy, ResourceThreads};
use net_traits::image_cache_thread::{ImageCacheChan, ImageCacheResult, ImageCacheThread};
use net_traits::request::{CredentialsMode, Destination, RequestInit};
use net_traits::storage_thread::StorageType;
use network_listener::NetworkListener;
use origin::Origin;
use profile_traits::mem::{self, OpaqueSender, Report, ReportKind, ReportsChan};
use profile_traits::time::{self, ProfilerCategory, profile};
use script_layout_interface::message::{self, NewLayoutThreadInfo, ReflowQueryType};
use script_runtime::{CommonScriptMsg, ScriptChan, ScriptThreadEventCategory, EnqueuedPromiseCallback};
use script_runtime::{ScriptPort, StackRootTLS, get_reports, new_rt_and_cx, PromiseJobQueue};
use script_traits::{CompositorEvent, ConstellationControlMsg, DiscardBrowsingContext, EventResult};
use script_traits::{InitialScriptState, LayoutMsg, LoadData, MouseButton, MouseEventType, MozBrowserEvent};
use script_traits::{NewLayoutInfo, ScriptMsg as ConstellationMsg};
use script_traits::{ScriptThreadFactory, TimerEvent, TimerEventRequest, TimerSource};
use script_traits::{TouchEventType, TouchId, UntrustedNodeAddress, WindowSizeData, WindowSizeType};
use script_traits::CompositorEvent::{KeyEvent, MouseButtonEvent, MouseMoveEvent, ResizeEvent};
use script_traits::CompositorEvent::{TouchEvent, TouchpadPressureEvent};
use script_traits::webdriver_msg::WebDriverScriptCommand;
use serviceworkerjob::{Job, JobQueue, AsyncJobHandler, FinishJobHandler, InvokeType, SettleType};
use servo_config::opts;
use servo_url::ServoUrl;
use std::cell::Cell;
use std::collections::{hash_map, HashMap, HashSet};
use std::option::Option;
use std::ptr;
use std::rc::Rc;
use std::result::Result;
use std::sync::{Arc, Mutex};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::mpsc::{Receiver, Select, Sender, channel};
use std::thread;
use style::context::ReflowGoal;
use style::dom::{TNode, UnsafeNode};
use style::thread_state;
use task_source::TaskSource;
use task_source::dom_manipulation::{DOMManipulationTask, DOMManipulationTaskSource};
use task_source::file_reading::FileReadingTaskSource;
use task_source::history_traversal::HistoryTraversalTaskSource;
use task_source::networking::NetworkingTaskSource;
use task_source::user_interaction::{UserInteractionTask, UserInteractionTaskSource};
use time::Tm;
use url::Position;
use webdriver_handlers;

thread_local!(pub static STACK_ROOTS: Cell<Option<RootCollectionPtr>> = Cell::new(None));
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
    /// The frame being loaded into.
    frame_id: FrameId,
    /// The parent pipeline and frame type associated with this load, if any.
    parent_info: Option<(PipelineId, FrameType)>,
    /// The current window size associated with this pipeline.
    window_size: Option<WindowSizeData>,
    /// Channel to the layout thread associated with this pipeline.
    layout_chan: Sender<message::Msg>,
    /// The current viewport clipping rectangle applying to this pipeline, if any.
    clip_rect: Option<Rect<f32>>,
    /// Window is frozen (navigated away while loading for example).
    is_frozen: bool,
    /// Window is visible.
    is_visible: bool,
    /// The requested URL of the load.
    url: ServoUrl,
    origin: Origin,
}

impl InProgressLoad {
    /// Create a new InProgressLoad object.
    fn new(id: PipelineId,
           frame_id: FrameId,
           parent_info: Option<(PipelineId, FrameType)>,
           layout_chan: Sender<message::Msg>,
           window_size: Option<WindowSizeData>,
           url: ServoUrl,
           origin: Origin) -> InProgressLoad {
        InProgressLoad {
            pipeline_id: id,
            frame_id: frame_id,
            parent_info: parent_info,
            layout_chan: layout_chan,
            window_size: window_size,
            clip_rect: None,
            is_frozen: false,
            is_visible: true,
            url: url,
            origin: origin,
        }
    }
}

/// Encapsulated state required to create cancellable runnables from non-script threads.
pub struct RunnableWrapper {
    pub cancelled: Option<Arc<AtomicBool>>,
}

impl RunnableWrapper {
    pub fn wrap_runnable<T: Runnable + Send + 'static>(&self, runnable: Box<T>) -> Box<Runnable + Send> {
        box CancellableRunnable {
            cancelled: self.cancelled.clone(),
            inner: runnable,
        }
    }
}

/// A runnable that can be discarded by toggling a shared flag.
pub struct CancellableRunnable<T: Runnable + Send> {
    cancelled: Option<Arc<AtomicBool>>,
    inner: Box<T>,
}

impl<T: Runnable + Send> Runnable for CancellableRunnable<T> {
    fn name(&self) -> &'static str { self.inner.name() }

    fn is_cancelled(&self) -> bool {
        self.cancelled.as_ref()
            .map(|cancelled| cancelled.load(Ordering::SeqCst))
            .unwrap_or(false)
    }

    fn main_thread_handler(self: Box<CancellableRunnable<T>>, script_thread: &ScriptThread) {
        self.inner.main_thread_handler(script_thread);
    }

    fn handler(self: Box<CancellableRunnable<T>>) {
        self.inner.handler()
    }
}

pub trait Runnable {
    fn is_cancelled(&self) -> bool { false }
    fn name(&self) -> &'static str { "generic runnable" }
    fn handler(self: Box<Self>) {}
    fn main_thread_handler(self: Box<Self>, _script_thread: &ScriptThread) { self.handler(); }
}

enum MixedMessage {
    FromConstellation(ConstellationControlMsg),
    FromScript(MainThreadScriptMsg),
    FromDevtools(DevtoolScriptControlMsg),
    FromImageCache(ImageCacheResult),
    FromScheduler(TimerEvent)
}

/// Messages used to control the script event loop
pub enum MainThreadScriptMsg {
    /// Common variants associated with the script messages
    Common(CommonScriptMsg),
    /// Notify a document that all pending loads are complete.
    DocumentLoadsComplete(PipelineId),
    /// Notifies the script that a window associated with a particular pipeline
    /// should be closed (only dispatched to ScriptThread).
    ExitWindow(PipelineId),
    /// Begins a content-initiated load on the specified pipeline (only
    /// dispatched to ScriptThread). Allows for a replace bool to be passed. If true,
    /// the current entry will be replaced instead of a new entry being added.
    Navigate(PipelineId, LoadData, bool),
    /// Tasks that originate from the DOM manipulation task source
    DOMManipulation(DOMManipulationTask),
    /// Tasks that originate from the user interaction task source
    UserInteraction(UserInteractionTask),
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
        box SendableMainThreadScriptChan((&self.0).clone())
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
        box MainThreadScriptChan((&self.0).clone())
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
    map: HashMap<PipelineId, JS<Document>>,
}

impl Documents {
    pub fn new() -> Documents {
        Documents {
            map: HashMap::new(),
        }
    }

    pub fn insert(&mut self, pipeline_id: PipelineId, doc: &Document) {
        self.map.insert(pipeline_id, JS::from_ref(doc));
    }

    pub fn remove(&mut self, pipeline_id: PipelineId) -> Option<Root<Document>> {
        self.map.remove(&pipeline_id).map(|ref doc| Root::from_ref(&**doc))
    }

    pub fn is_empty(&self) -> bool {
        self.map.is_empty()
    }

    pub fn find_document(&self, pipeline_id: PipelineId) -> Option<Root<Document>> {
        self.map.get(&pipeline_id).map(|doc| Root::from_ref(&**doc))
    }

    pub fn find_window(&self, pipeline_id: PipelineId) -> Option<Root<Window>> {
        self.find_document(pipeline_id).map(|doc| Root::from_ref(doc.window()))
    }

    pub fn find_global(&self, pipeline_id: PipelineId) -> Option<Root<GlobalScope>> {
        self.find_window(pipeline_id).map(|window| Root::from_ref(window.upcast()))
    }

    pub fn find_iframe(&self, pipeline_id: PipelineId, frame_id: FrameId) -> Option<Root<HTMLIFrameElement>> {
        self.find_document(pipeline_id).and_then(|doc| doc.find_iframe(frame_id))
    }

    pub fn iter<'a>(&'a self) -> DocumentsIter<'a> {
        DocumentsIter {
            iter: self.map.iter(),
        }
    }
}

#[allow(unrooted_must_root)]
pub struct DocumentsIter<'a> {
    iter: hash_map::Iter<'a, PipelineId, JS<Document>>,
}

impl<'a> Iterator for DocumentsIter<'a> {
    type Item = (PipelineId, Root<Document>);

    fn next(&mut self) -> Option<(PipelineId, Root<Document>)> {
        self.iter.next().map(|(id, doc)| (*id, Root::from_ref(&**doc)))
    }
}


#[derive(JSTraceable)]
// ScriptThread instances are rooted on creation, so this is okay
#[allow(unrooted_must_root)]
pub struct ScriptThread {
    /// The documents for pipelines managed by this thread
    documents: DOMRefCell<Documents>,
    /// A list of data pertaining to loads that have not yet received a network response
    incomplete_loads: DOMRefCell<Vec<InProgressLoad>>,
    /// A map to store service worker registrations for a given origin
    registration_map: DOMRefCell<HashMap<ServoUrl, JS<ServiceWorkerRegistration>>>,
    /// A job queue for Service Workers keyed by their scope url
    job_queue_map: Rc<JobQueue>,
    /// A handle to the image cache thread.
    image_cache_thread: ImageCacheThread,
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

    dom_manipulation_task_source: DOMManipulationTaskSource,

    user_interaction_task_source: UserInteractionTaskSource,

    networking_task_source: NetworkingTaskSource,

    history_traversal_task_source: HistoryTraversalTaskSource,

    file_reading_task_source: FileReadingTaskSource,

    /// A channel to hand out to threads that need to respond to a message from the script thread.
    control_chan: IpcSender<ConstellationControlMsg>,

    /// The port on which the constellation and layout threads can communicate with the
    /// script thread.
    control_port: Receiver<ConstellationControlMsg>,

    /// For communicating load url messages to the constellation
    constellation_chan: IpcSender<ConstellationMsg>,

    /// A sender for new layout threads to communicate to the constellation.
    layout_to_constellation_chan: IpcSender<LayoutMsg>,

    /// The port on which we receive messages from the image cache
    image_cache_port: Receiver<ImageCacheResult>,

    /// The channel on which the image cache can send messages to ourself.
    image_cache_channel: ImageCacheChan,

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
    topmost_mouse_over_target: MutNullableJS<Element>,

    /// List of pipelines that have been owned and closed by this script thread.
    closed_pipelines: DOMRefCell<HashSet<PipelineId>>,

    scheduler_chan: IpcSender<TimerEventRequest>,
    timer_event_chan: Sender<TimerEvent>,
    timer_event_port: Receiver<TimerEvent>,

    content_process_shutdown_chan: IpcSender<()>,

    promise_job_queue: PromiseJobQueue,
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
            thread_state::initialize(thread_state::SCRIPT);
            PipelineNamespace::install(state.pipeline_namespace_id);
            FrameId::install(state.top_level_frame_id);
            let roots = RootCollection::new();
            let _stack_roots_tls = StackRootTLS::new(&roots);
            let id = state.id;
            let frame_id = state.frame_id;
            let parent_info = state.parent_info;
            let mem_profiler_chan = state.mem_profiler_chan.clone();
            let window_size = state.window_size;
            let script_thread = ScriptThread::new(state,
                                                  script_port,
                                                  script_chan.clone());

            SCRIPT_THREAD_ROOT.with(|root| {
                root.set(Some(&script_thread as *const _));
            });

            let mut failsafe = ScriptMemoryFailsafe::new(&script_thread);

            let origin = Origin::new(&load_data.url);
            let new_load = InProgressLoad::new(id, frame_id, parent_info, layout_chan, window_size,
                                               load_data.url.clone(), origin);
            script_thread.start_page_load(new_load, load_data);

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
    pub fn page_headers_available(id: &PipelineId, metadata: Option<Metadata>)
                                  -> Option<Root<ServoParser>> {
        SCRIPT_THREAD_ROOT.with(|root| {
            let script_thread = unsafe { &*root.get().unwrap() };
            script_thread.handle_page_headers_available(id, metadata)
        })
    }

    #[allow(unrooted_must_root)]
    pub fn schedule_job(job: Job, global: &GlobalScope) {
        SCRIPT_THREAD_ROOT.with(|root| {
            let script_thread = unsafe { &*root.get().unwrap() };
            let job_queue = &*script_thread.job_queue_map;
            job_queue.schedule_job(job, global, &script_thread);
        });
    }

    pub fn parsing_complete(id: PipelineId) {
        SCRIPT_THREAD_ROOT.with(|root| {
            let script_thread = unsafe { &*root.get().unwrap() };
            script_thread.handle_parsing_complete(id);
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
    pub fn await_stable_state<T: Runnable + Send + 'static>(task: T) {
        //TODO use microtasks when they exist
        SCRIPT_THREAD_ROOT.with(|root| {
            if let Some(script_thread) = root.get() {
                let script_thread = unsafe { &*script_thread };
                let _ = script_thread.chan.send(CommonScriptMsg::RunnableMsg(
                    ScriptThreadEventCategory::DomEvent,
                    box task));
            }
        });
    }

    pub fn process_attach_layout(new_layout_info: NewLayoutInfo, origin: Origin) {
        SCRIPT_THREAD_ROOT.with(|root| {
            if let Some(script_thread) = root.get() {
                let script_thread = unsafe { &*script_thread };
                script_thread.profile_event(ScriptThreadEventCategory::AttachLayout, || {
                    script_thread.handle_new_layout(new_layout_info, origin);
                })
            }
        });
    }

    pub fn find_document(id: PipelineId) -> Option<Root<Document>> {
        SCRIPT_THREAD_ROOT.with(|root| root.get().and_then(|script_thread| {
            let script_thread = unsafe { &*script_thread };
            script_thread.documents.borrow().find_document(id)
        }))
    }

    /// Creates a new script thread.
    pub fn new(state: InitialScriptState,
               port: Receiver<MainThreadScriptMsg>,
               chan: Sender<MainThreadScriptMsg>)
               -> ScriptThread {
        let runtime = unsafe { new_rt_and_cx() };

        unsafe {
            JS_SetWrapObjectCallbacks(runtime.rt(),
                                      &WRAP_CALLBACKS);
            SetWindowProxyClass(runtime.rt(), GetWindowProxyClass());
        }

        // Ask the router to proxy IPC messages from the devtools to us.
        let (ipc_devtools_sender, ipc_devtools_receiver) = ipc::channel().unwrap();
        let devtools_port = ROUTER.route_ipc_receiver_to_new_mpsc_receiver(ipc_devtools_receiver);

        // Ask the router to proxy IPC messages from the image cache thread to us.
        let (ipc_image_cache_channel, ipc_image_cache_port) = ipc::channel().unwrap();
        let image_cache_port =
            ROUTER.route_ipc_receiver_to_new_mpsc_receiver(ipc_image_cache_port);

        let (timer_event_chan, timer_event_port) = channel();

        // Ask the router to proxy IPC messages from the control port to us.
        let control_port = ROUTER.route_ipc_receiver_to_new_mpsc_receiver(state.control_port);

        let boxed_script_sender = MainThreadScriptChan(chan.clone()).clone();

        ScriptThread {
            documents: DOMRefCell::new(Documents::new()),
            incomplete_loads: DOMRefCell::new(vec!()),
            registration_map: DOMRefCell::new(HashMap::new()),
            job_queue_map: Rc::new(JobQueue::new()),

            image_cache_thread: state.image_cache_thread,
            image_cache_channel: ImageCacheChan(ipc_image_cache_channel),
            image_cache_port: image_cache_port,

            resource_threads: state.resource_threads,
            bluetooth_thread: state.bluetooth_thread,

            port: port,

            chan: MainThreadScriptChan(chan.clone()),
            dom_manipulation_task_source: DOMManipulationTaskSource(chan.clone()),
            user_interaction_task_source: UserInteractionTaskSource(chan.clone()),
            networking_task_source: NetworkingTaskSource(boxed_script_sender.clone()),
            history_traversal_task_source: HistoryTraversalTaskSource(chan),
            file_reading_task_source: FileReadingTaskSource(boxed_script_sender),

            control_chan: state.control_chan,
            control_port: control_port,
            constellation_chan: state.constellation_chan,
            time_profiler_chan: state.time_profiler_chan,
            mem_profiler_chan: state.mem_profiler_chan,

            devtools_chan: state.devtools_chan,
            devtools_port: devtools_port,
            devtools_sender: ipc_devtools_sender,

            js_runtime: Rc::new(runtime),
            topmost_mouse_over_target: MutNullableJS::new(Default::default()),
            closed_pipelines: DOMRefCell::new(HashSet::new()),

            scheduler_chan: state.scheduler_chan,
            timer_event_chan: timer_event_chan,
            timer_event_port: timer_event_port,

            content_process_shutdown_chan: state.content_process_shutdown_chan,

            promise_job_queue: PromiseJobQueue::new(),

            layout_to_constellation_chan: state.layout_to_constellation_chan,
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

        // Squash any pending resize, reflow, animation tick, and mouse-move events in the queue.
        let mut mouse_move_event_index = None;
        let mut animation_ticks = HashSet::new();
        loop {
            match event {
                // This has to be handled before the ResizeMsg below,
                // otherwise the page may not have been added to the
                // child list yet, causing the find() to fail.
                FromConstellation(ConstellationControlMsg::AttachLayout(
                        new_layout_info)) => {
                    self.profile_event(ScriptThreadEventCategory::AttachLayout, || {
                        let origin = Origin::new(&new_layout_info.load_data.url);
                        self.handle_new_layout(new_layout_info, origin);
                    })
                }
                FromConstellation(ConstellationControlMsg::Resize(id, size, size_type)) => {
                    self.profile_event(ScriptThreadEventCategory::Resize, || {
                        self.handle_resize(id, size, size_type);
                    })
                }
                FromConstellation(ConstellationControlMsg::Viewport(id, rect)) => {
                    self.profile_event(ScriptThreadEventCategory::SetViewport, || {
                        self.handle_viewport(id, rect);
                    })
                }
                FromConstellation(ConstellationControlMsg::SetScrollState(id, scroll_state)) => {
                    self.profile_event(ScriptThreadEventCategory::SetScrollState, || {
                        self.handle_set_scroll_state(id, &scroll_state);
                    })
                }
                FromConstellation(ConstellationControlMsg::TickAllAnimations(
                        pipeline_id)) => {
                    if !animation_ticks.contains(&pipeline_id) {
                        animation_ticks.insert(pipeline_id);
                        sequential.push(event);
                    }
                }
                FromConstellation(ConstellationControlMsg::SendEvent(
                        _,
                        MouseMoveEvent(_))) => {
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
        for msg in sequential {
            let category = self.categorize_msg(&msg);

            let result = self.profile_event(category, move || {
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

            if let Some(retval) = result {
                return retval
            }
        }

        // Issue batched reflows on any pages that require it (e.g. if images loaded)
        // TODO(gw): In the future we could probably batch other types of reflows
        // into this loop too, but for now it's only images.
        for (_, document) in self.documents.borrow().iter() {
            let window = document.window();
            let pending_reflows = window.get_pending_reflow_count();
            if pending_reflows > 0 {
                window.reflow(ReflowGoal::ForDisplay,
                              ReflowQueryType::NoQuery,
                              ReflowReason::ImageLoaded);
            } else {
                // Reflow currently happens when explicitly invoked by code that
                // knows the document could have been modified. This should really
                // be driven by the compositor on an as-needed basis instead, to
                // minimize unnecessary work.
                window.reflow(ReflowGoal::ForDisplay,
                              ReflowQueryType::NoQuery,
                              ReflowReason::MissingExplicitReflow);
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
            MixedMessage::FromDevtools(_) => ScriptThreadEventCategory::DevtoolsMsg,
            MixedMessage::FromImageCache(_) => ScriptThreadEventCategory::ImageCacheMsg,
            MixedMessage::FromScript(ref inner_msg) => {
                match *inner_msg {
                    MainThreadScriptMsg::Common(CommonScriptMsg::RunnableMsg(ref category, _)) =>
                        *category,
                    _ => ScriptThreadEventCategory::ScriptEvent
                }
            },
            MixedMessage::FromScheduler(_) => ScriptThreadEventCategory::TimerEvent
        }
    }

    fn profile_event<F, R>(&self, category: ScriptThreadEventCategory, f: F) -> R
        where F: FnOnce() -> R {
        if opts::get().profile_script_events {
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
                ScriptThreadEventCategory::WorkerEvent => ProfilerCategory::ScriptWorkerEvent,
                ScriptThreadEventCategory::ServiceWorkerEvent => ProfilerCategory::ScriptServiceWorkerEvent,
                ScriptThreadEventCategory::EnterFullscreen => ProfilerCategory::ScriptEnterFullscreen,
                ScriptThreadEventCategory::ExitFullscreen => ProfilerCategory::ScriptExitFullscreen,
            };
            profile(profiler_cat, None, self.time_profiler_chan.clone(), f)
        } else {
            f()
        }
    }

    fn handle_msg_from_constellation(&self, msg: ConstellationControlMsg) {
        match msg {
            ConstellationControlMsg::Navigate(parent_pipeline_id, frame_id, load_data, replace) =>
                self.handle_navigate(parent_pipeline_id, Some(frame_id), load_data, replace),
            ConstellationControlMsg::SendEvent(id, event) =>
                self.handle_event(id, event),
            ConstellationControlMsg::ResizeInactive(id, new_size) =>
                self.handle_resize_inactive_msg(id, new_size),
            ConstellationControlMsg::GetTitle(pipeline_id) =>
                self.handle_get_title_msg(pipeline_id),
            ConstellationControlMsg::Freeze(pipeline_id) =>
                self.handle_freeze_msg(pipeline_id),
            ConstellationControlMsg::Thaw(pipeline_id) =>
                self.handle_thaw_msg(pipeline_id),
            ConstellationControlMsg::ChangeFrameVisibilityStatus(pipeline_id, visible) =>
                self.handle_visibility_change_msg(pipeline_id, visible),
            ConstellationControlMsg::NotifyVisibilityChange(parent_pipeline_id, frame_id, visible) =>
                self.handle_visibility_change_complete_msg(parent_pipeline_id, frame_id, visible),
            ConstellationControlMsg::MozBrowserEvent(parent_pipeline_id,
                                                     frame_id,
                                                     event) =>
                self.handle_mozbrowser_event_msg(parent_pipeline_id,
                                                 frame_id,
                                                 event),
            ConstellationControlMsg::UpdatePipelineId(parent_pipeline_id,
                                                      frame_id,
                                                      new_pipeline_id) =>
                self.handle_update_pipeline_id(parent_pipeline_id,
                                               frame_id,
                                               new_pipeline_id),
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
            ConstellationControlMsg::DispatchFrameLoadEvent {
                target: frame_id, parent: parent_id, child: child_id } =>
                self.handle_frame_load_event(parent_id, frame_id, child_id),
            ConstellationControlMsg::DispatchStorageEvent(pipeline_id, storage, url, key, old_value, new_value) =>
                self.handle_storage_event(pipeline_id, storage, url, key, old_value, new_value),
            ConstellationControlMsg::FramedContentChanged(parent_pipeline_id, frame_id) =>
                self.handle_framed_content_changed(parent_pipeline_id, frame_id),
            ConstellationControlMsg::ReportCSSError(pipeline_id, filename, line, column, msg) =>
                self.handle_css_error_reporting(pipeline_id, filename, line, column, msg),
            ConstellationControlMsg::Reload(pipeline_id) =>
                self.handle_reload(pipeline_id),
            ConstellationControlMsg::ExitPipeline(pipeline_id, discard_browsing_context) =>
                self.handle_exit_pipeline_msg(pipeline_id, discard_browsing_context),
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
            MainThreadScriptMsg::Navigate(parent_pipeline_id, load_data, replace) =>
                self.handle_navigate(parent_pipeline_id, None, load_data, replace),
            MainThreadScriptMsg::ExitWindow(id) =>
                self.handle_exit_window_msg(id),
            MainThreadScriptMsg::DocumentLoadsComplete(id) =>
                self.handle_loads_complete(id),
            MainThreadScriptMsg::Common(CommonScriptMsg::RunnableMsg(_, runnable)) => {
                // The category of the runnable is ignored by the pattern, however
                // it is still respected by profiling (see categorize_msg).
                if !runnable.is_cancelled() {
                    runnable.handler()
                }
            }
            MainThreadScriptMsg::Common(CommonScriptMsg::CollectReports(reports_chan)) =>
                self.collect_reports(reports_chan),
            MainThreadScriptMsg::DOMManipulation(task) =>
                task.handle_task(self),
            MainThreadScriptMsg::UserInteraction(task) =>
                task.handle_task(self),
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

    fn handle_msg_from_image_cache(&self, msg: ImageCacheResult) {
        msg.responder.unwrap().respond(msg.image_response);
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
            WebDriverScriptCommand::GetFrameId(frame_id, reply) =>
                webdriver_handlers::handle_get_frame_id(&*documents, pipeline_id, frame_id, reply),
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
        let mut loads = self.incomplete_loads.borrow_mut();
        if let Some(ref mut load) = loads.iter_mut().find(|load| load.pipeline_id == id) {
            load.clip_rect = Some(rect);
            return;
        }
        warn!("Page rect message sent to nonexistent pipeline");
    }

    fn handle_set_scroll_state(&self,
                               id: PipelineId,
                               scroll_states: &[(UntrustedNodeAddress, Point2D<f32>)]) {
        let window = match { self.documents.borrow().find_window(id) } {
            Some(window) => window,
            None => return warn!("Set scroll state message sent to nonexistent pipeline: {:?}", id),
        };

        let mut scroll_offsets = HashMap::new();
        for &(node_address, ref scroll_offset) in scroll_states {
            if node_address == UntrustedNodeAddress(ptr::null()) {
                window.update_viewport_for_scroll(-scroll_offset.x, -scroll_offset.y);
            } else {
                scroll_offsets.insert(node_address,
                                      Point2D::new(-scroll_offset.x, -scroll_offset.y));
            }
        }
        window.set_scroll_offsets(scroll_offsets)
    }

    fn handle_new_layout(&self, new_layout_info: NewLayoutInfo, origin: Origin) {
        let NewLayoutInfo {
            parent_info,
            new_pipeline_id,
            frame_id,
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
            image_cache_thread: self.image_cache_thread.clone(),
            content_process_shutdown_chan: content_process_shutdown_chan,
            layout_threads: layout_threads,
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
        let new_load = InProgressLoad::new(new_pipeline_id, frame_id, parent_info,
                                           layout_chan, window_size,
                                           load_data.url.clone(), origin);
        if load_data.url.as_str() == "about:blank" {
            self.start_page_load_about_blank(new_load);
        } else {
            self.start_page_load(new_load, load_data);
        }
    }

    fn handle_loads_complete(&self, pipeline: PipelineId) {
        let doc = match { self.documents.borrow().find_document(pipeline) } {
            Some(doc) => doc,
            None => return warn!("Message sent to closed pipeline {}.", pipeline),
        };
        if doc.loader().is_blocked() {
            debug!("Script thread got loads complete while loader is blocked.");
            return;
        }

        doc.mut_loader().inhibit_events();

        // https://html.spec.whatwg.org/multipage/#the-end step 7
        // Schedule a task to fire a "load" event (if no blocking loads have arrived in the mean time)
        // NOTE: we can end up executing this code more than once, in case more blocking loads arrive.
        let handler = box DocumentProgressHandler::new(Trusted::new(&doc));
        self.dom_manipulation_task_source.queue(handler, doc.window().upcast()).unwrap();

        if let Some(fragment) = doc.url().fragment() {
            doc.check_and_scroll_fragment(fragment);
        };
    }

    fn collect_reports(&self, reports_chan: ReportsChan) {
        let mut path_seg = String::from("url(");
        let mut dom_tree_size = 0;
        let mut reports = vec![];

        for (_, document) in self.documents.borrow().iter() {
            let current_url = document.url();

            for child in document.upcast::<Node>().traverse_preorder() {
                dom_tree_size += heap_size_of_self_and_children(&*child);
            }
            dom_tree_size += heap_size_of_self_and_children(document.window());

            if reports.len() > 0 {
                path_seg.push_str(", ");
            }
            path_seg.push_str(current_url.as_str());

            reports.push(Report {
                path: path![format!("url({})", current_url.as_str()), "dom-tree"],
                kind: ReportKind::ExplicitJemallocHeapSize,
                size: dom_tree_size,
            });
        }

        path_seg.push_str(")");
        reports.extend(get_reports(self.get_cx(), path_seg));
        reports_chan.send(reports);
    }

    /// To slow/speed up timers and manage any other script thread resource based on visibility.
    /// Returns true if successful.
    fn alter_resource_utilization(&self, id: PipelineId, visible: bool) -> bool {
        let window = self.documents.borrow().find_window(id);
        if let Some(window) = window {
            if visible {
                window.upcast::<GlobalScope>().speed_up_timers();
            } else {
                window.upcast::<GlobalScope>().slow_down_timers();
            }
            return true;
        }
        false
    }

    /// Updates iframe element after a change in visibility
    fn handle_visibility_change_complete_msg(&self, parent_pipeline_id: PipelineId, id: FrameId, visible: bool) {
        let iframe = self.documents.borrow().find_iframe(parent_pipeline_id, id);
        if let Some(iframe) = iframe {
            iframe.change_visibility_status(visible);
        }
    }

    /// Handle visibility change message
    fn handle_visibility_change_msg(&self, id: PipelineId, visible: bool) {
        let resources_altered = self.alter_resource_utilization(id, visible);

        // Separate message sent since parent script thread could be different (Iframe of different
        // domain)
        self.constellation_chan.send(ConstellationMsg::VisibilityChangeComplete(id, visible)).unwrap();

        if !resources_altered {
            let mut loads = self.incomplete_loads.borrow_mut();
            if let Some(ref mut load) = loads.iter_mut().find(|load| load.pipeline_id == id) {
                load.is_visible = visible;
                return;
            }
        } else {
            return;
        }

        warn!("change visibility message sent to nonexistent pipeline");
    }

    /// Handles freeze message
    fn handle_freeze_msg(&self, id: PipelineId) {
        let window = self.documents.borrow().find_window(id);
        if let Some(window) = window {
            window.freeze();
            return;
        }
        let mut loads = self.incomplete_loads.borrow_mut();
        if let Some(ref mut load) = loads.iter_mut().find(|load| load.pipeline_id == id) {
            load.is_frozen = true;
            return;
        }
        warn!("freeze sent to nonexistent pipeline");
    }

    /// Handles thaw message
    fn handle_thaw_msg(&self, id: PipelineId) {
        let document = self.documents.borrow().find_document(id);
        if let Some(document) = document {
            if let Some(context) = document.browsing_context() {
                let needed_reflow = context.set_reflow_status(false);
                if needed_reflow {
                    self.rebuild_and_force_reflow(&document, ReflowReason::CachedPageNeededReflow);
                }
            }
            document.window().thaw();
            document.fully_activate();
            return;
        }
        let mut loads = self.incomplete_loads.borrow_mut();
        if let Some(ref mut load) = loads.iter_mut().find(|load| load.pipeline_id == id) {
            load.is_frozen = false;
            return;
        }
        warn!("thaw sent to nonexistent pipeline");
    }

    fn handle_focus_iframe_msg(&self,
                               parent_pipeline_id: PipelineId,
                               frame_id: FrameId) {
        let doc = self.documents.borrow().find_document(parent_pipeline_id).unwrap();
        let frame_element = doc.find_iframe(frame_id);

        if let Some(ref frame_element) = frame_element {
            doc.begin_focus_transaction();
            doc.request_focus(frame_element.upcast());
            doc.commit_focus_transaction(FocusType::Parent);
        }
    }

    fn handle_framed_content_changed(&self,
                                     parent_pipeline_id: PipelineId,
                                     frame_id: FrameId) {
        let doc = self.documents.borrow().find_document(parent_pipeline_id).unwrap();
        let frame_element = doc.find_iframe(frame_id);
        if let Some(ref frame_element) = frame_element {
            frame_element.upcast::<Node>().dirty(NodeDamage::OtherNodeDamage);
            let window = doc.window();
            window.reflow(ReflowGoal::ForDisplay,
                          ReflowQueryType::NoQuery,
                          ReflowReason::FramedContentChanged);
        }
    }

    /// Handles a mozbrowser event, for example see:
    /// https://developer.mozilla.org/en-US/docs/Web/Events/mozbrowserloadstart
    fn handle_mozbrowser_event_msg(&self,
                                   parent_pipeline_id: PipelineId,
                                   frame_id: Option<FrameId>,
                                   event: MozBrowserEvent) {
        let doc = match { self.documents.borrow().find_document(parent_pipeline_id) } {
            None => return warn!("Mozbrowser event after pipeline {:?} closed.", parent_pipeline_id),
            Some(doc) => doc,
        };

        match frame_id {
            None => doc.window().dispatch_mozbrowser_event(event),
            Some(frame_id) => match doc.find_iframe(frame_id) {
                None => warn!("Mozbrowser event after iframe {:?}/{:?} closed.", parent_pipeline_id, frame_id),
                Some(frame_element) => frame_element.dispatch_mozbrowser_event(event),
            },
        }
    }

    fn handle_update_pipeline_id(&self,
                                 parent_pipeline_id: PipelineId,
                                 frame_id: FrameId,
                                 new_pipeline_id: PipelineId) {
        let frame_element = self.documents.borrow().find_iframe(parent_pipeline_id, frame_id);
        if let Some(frame_element) = frame_element {
            frame_element.update_pipeline_id(new_pipeline_id);
        }
    }

    /// Window was resized, but this script was not active, so don't reflow yet
    fn handle_resize_inactive_msg(&self, id: PipelineId, new_size: WindowSizeData) {
        let window = self.documents.borrow().find_window(id)
            .expect("ScriptThread: received a resize msg for a pipeline not in this script thread. This is a bug.");
        window.set_window_size(new_size);
        window.browsing_context().set_reflow_status(true);
    }

    /// We have gotten a window.close from script, which we pass on to the compositor.
    /// We do not shut down the script thread now, because the compositor will ask the
    /// constellation to shut down the pipeline, which will clean everything up
    /// normally. If we do exit, we will tear down the DOM nodes, possibly at a point
    /// where layout is still accessing them.
    fn handle_exit_window_msg(&self, _: PipelineId) {
        debug!("script thread handling exit window msg");

        // TODO(tkuehn): currently there is only one window,
        // so this can afford to be naive and just shut down the
        // constellation. In the future it'll need to be smarter.
        self.constellation_chan.send(ConstellationMsg::Exit).unwrap();
    }

    /// We have received notification that the response associated with a load has completed.
    /// Kick off the document and frame tree creation process using the result.
    fn handle_page_headers_available(&self, id: &PipelineId,
                                     metadata: Option<Metadata>) -> Option<Root<ServoParser>> {
        let idx = self.incomplete_loads.borrow().iter().position(|load| { load.pipeline_id == *id });
        // The matching in progress load structure may not exist if
        // the pipeline exited before the page load completed.
        match idx {
            Some(idx) => {
                let load = self.incomplete_loads.borrow_mut().remove(idx);
                metadata.map(|meta| self.load(meta, load))
            }
            None => {
                assert!(self.closed_pipelines.borrow().contains(id));
                None
            }
        }
    }

    pub fn handle_get_registration(&self, scope_url: &ServoUrl) -> Option<Root<ServiceWorkerRegistration>> {
        let maybe_registration_ref = self.registration_map.borrow();
        maybe_registration_ref.get(scope_url).map(|x| Root::from_ref(&**x))
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
            reg_ref.insert(scope.clone(), JS::from_ref(registration));
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
        let _ = self.constellation_chan.send(ConstellationMsg::RegisterServiceWorker(scope_things, scope.clone()));
    }

    pub fn dispatch_job_queue(&self, job_handler: Box<AsyncJobHandler>) {
        let scope_url = job_handler.scope_url.clone();
        let queue_ref = self.job_queue_map.0.borrow();
        let front_job = {
            let job_vec = queue_ref.get(&scope_url);
            job_vec.unwrap().first().unwrap()
        };
        match job_handler.invoke_type {
            InvokeType::Run => (&*self.job_queue_map).run_job(job_handler, self),
            InvokeType::Register => self.job_queue_map.run_register(front_job, job_handler, self),
            InvokeType::Update => self.job_queue_map.update(front_job, &*front_job.client.global(), self),
            InvokeType::Settle(settle_type) => {
                let promise = &front_job.promise;
                let global = &*front_job.client.global();
                let trusted_global = Trusted::new(global);
                let _ac = JSAutoCompartment::new(global.get_cx(), promise.reflector().get_jsobject().get());
                match settle_type {
                    SettleType::Resolve(reg) => promise.resolve_native(global.get_cx(), &*reg.root()),
                    SettleType::Reject(err) => promise.reject_error(global.get_cx(), err)
                }
                let finish_job_handler = box FinishJobHandler::new(scope_url, trusted_global);
                self.queue_finish_job(finish_job_handler, global);
            }
        }
    }

    pub fn queue_serviceworker_job(&self, async_job_handler: Box<AsyncJobHandler>, global: &GlobalScope) {
        let _ = self.dom_manipulation_task_source.queue(async_job_handler, &*global);
    }

    pub fn queue_finish_job(&self, finish_job_handler: Box<FinishJobHandler>, global: &GlobalScope) {
        let _ = self.dom_manipulation_task_source.queue(finish_job_handler, global);
    }

    pub fn invoke_finish_job(&self, finish_job_handler: Box<FinishJobHandler>) {
        let job_queue = &*self.job_queue_map;
        let global = &*finish_job_handler.global.root();
        let scope_url = (*finish_job_handler).scope_url;
        job_queue.finish_job(scope_url, global, self);
    }

    pub fn invoke_job_update(&self, job: &Job, global: &GlobalScope) {
        let job_queue = &*self.job_queue_map;
        job_queue.update(job, global, self);
    }

    /// Handles a request for the window title.
    fn handle_get_title_msg(&self, pipeline_id: PipelineId) {
        let document = match { self.documents.borrow().find_document(pipeline_id) } {
            Some(document) => document,
            None => return warn!("Message sent to closed pipeline {}.", pipeline_id),
        };
        document.send_title_to_compositor();
    }

    /// Handles a request to exit a pipeline and shut down layout.
    fn handle_exit_pipeline_msg(&self, id: PipelineId, discard_bc: DiscardBrowsingContext) {
        debug!("Exiting pipeline {}.", id);

        self.closed_pipelines.borrow_mut().insert(id);

        // Check if the exit message is for an in progress load.
        let idx = self.incomplete_loads.borrow().iter().position(|load| {
            load.pipeline_id == id
        });

        if let Some(idx) = idx {
            let load = self.incomplete_loads.borrow_mut().remove(idx);

            // Tell the layout thread to begin shutting down, and wait until it
            // processed this message.
            let (response_chan, response_port) = channel();
            let chan = &load.layout_chan;
            if chan.send(message::Msg::PrepareToExit(response_chan)).is_ok() {
                debug!("shutting down layout for page {}", id);
                response_port.recv().unwrap();
                chan.send(message::Msg::ExitNow).ok();
            }
        }

        if let Some(document) = self.documents.borrow_mut().remove(id) {
            shut_down_layout(document.window());
            if discard_bc == DiscardBrowsingContext::Yes {
                if let Some(context) = document.browsing_context() {
                    context.discard();
                }
            }
            let _ = self.constellation_chan.send(ConstellationMsg::PipelineExited(id));
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
    #[allow(unsafe_code)]
    fn handle_transition_event(&self, unsafe_node: UnsafeNode, name: String, duration: f64) {
        let node = unsafe { ServoLayoutNode::from_unsafe(&unsafe_node) };
        let node = unsafe { node.get_jsmanaged().get_for_script() };
        let window = window_from_node(node);

        // Not quite the right thing - see #13865.
        node.dirty(NodeDamage::NodeStyleDamaged);

        if let Some(el) = node.downcast::<Element>() {
            if &*window.GetComputedStyle(el, None).Display() == "none" {
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
        let transition_event = TransitionEvent::new(window.upcast(),
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

    /// Notify the containing document of a child frame that has completed loading.
    fn handle_frame_load_event(&self, parent_id: PipelineId, frame_id: FrameId, child_id: PipelineId) {
        let iframe = self.documents.borrow().find_iframe(parent_id, frame_id);
        match iframe {
            Some(iframe) => iframe.iframe_load_event_steps(child_id),
            None => warn!("Message sent to closed pipeline {}.", parent_id),
        }
    }

    /// The entry point to document loading. Defines bindings, sets up the window and document
    /// objects, parses HTML and CSS, and kicks off initial layout.
    fn load(&self, metadata: Metadata, incomplete: InProgressLoad) -> Root<ServoParser> {
        let final_url = metadata.final_url.clone();
        {
            // send the final url to the layout thread.
            incomplete.layout_chan
                      .send(message::Msg::SetFinalUrl(final_url.clone()))
                      .unwrap();

            // update the pipeline url
            self.constellation_chan
                .send(ConstellationMsg::SetFinalUrl(incomplete.pipeline_id, final_url.clone()))
                .unwrap();
        }
        debug!("ScriptThread: loading {} on pipeline {:?}", incomplete.url, incomplete.pipeline_id);

        let frame_element = incomplete.parent_info.and_then(|(parent_id, _)|
            // In the case a parent id exists but the matching context
            // cannot be found, this means the context exists in a different
            // script thread (due to origin) so it shouldn't be returned.
            // TODO: window.parent will continue to return self in that
            // case, which is wrong. We should be returning an object that
            // denies access to most properties (per
            // https://github.com/servo/servo/issues/3939#issuecomment-62287025).
            self.documents.borrow().find_iframe(parent_id, incomplete.frame_id)
        );

        let MainThreadScriptChan(ref sender) = self.chan;
        let DOMManipulationTaskSource(ref dom_sender) = self.dom_manipulation_task_source;
        let UserInteractionTaskSource(ref user_sender) = self.user_interaction_task_source;
        let HistoryTraversalTaskSource(ref history_sender) = self.history_traversal_task_source;

        let (ipc_timer_event_chan, ipc_timer_event_port) = ipc::channel().unwrap();
        ROUTER.route_ipc_receiver_to_mpsc_sender(ipc_timer_event_port,
                                                 self.timer_event_chan.clone());

        // Create the window and document objects.
        let window = Window::new(self.js_runtime.clone(),
                                 MainThreadScriptChan(sender.clone()),
                                 DOMManipulationTaskSource(dom_sender.clone()),
                                 UserInteractionTaskSource(user_sender.clone()),
                                 self.networking_task_source.clone(),
                                 HistoryTraversalTaskSource(history_sender.clone()),
                                 self.file_reading_task_source.clone(),
                                 self.image_cache_channel.clone(),
                                 self.image_cache_thread.clone(),
                                 self.resource_threads.clone(),
                                 self.bluetooth_thread.clone(),
                                 self.mem_profiler_chan.clone(),
                                 self.time_profiler_chan.clone(),
                                 self.devtools_chan.clone(),
                                 self.constellation_chan.clone(),
                                 self.control_chan.clone(),
                                 self.scheduler_chan.clone(),
                                 ipc_timer_event_chan,
                                 incomplete.layout_chan,
                                 incomplete.pipeline_id,
                                 incomplete.parent_info,
                                 incomplete.window_size);
        let frame_element = frame_element.r().map(Castable::upcast);

        let browsing_context = BrowsingContext::new(&window, frame_element);
        window.init_browsing_context(&browsing_context);

        let last_modified = metadata.headers.as_ref().and_then(|headers| {
            headers.get().map(|&LastModified(HttpDate(ref tm))| dom_last_modified(tm))
        });

        let content_type = metadata.content_type.as_ref().and_then(|&Serde(ContentType(ref mimetype))| {
            match *mimetype {
                Mime(TopLevel::Application, SubLevel::Xml, _) |
                Mime(TopLevel::Application, SubLevel::Ext(_), _) |
                Mime(TopLevel::Text, SubLevel::Xml, _) |
                Mime(TopLevel::Text, SubLevel::Plain, _) => Some(DOMString::from(mimetype.to_string())),
                _ => None,
            }
        });

        let loader = DocumentLoader::new_with_threads(self.resource_threads.clone(),
                                                      Some(incomplete.url.clone()));

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

        let referrer_policy = if let Some(headers) = metadata.headers {
            headers.get::<ReferrerPolicyHeader>().map(|h| match *h {
                ReferrerPolicyHeader::NoReferrer =>
                    ReferrerPolicy::NoReferrer,
                ReferrerPolicyHeader::NoReferrerWhenDowngrade =>
                    ReferrerPolicy::NoReferrerWhenDowngrade,
                ReferrerPolicyHeader::SameOrigin =>
                    ReferrerPolicy::SameOrigin,
                ReferrerPolicyHeader::Origin =>
                    ReferrerPolicy::Origin,
                ReferrerPolicyHeader::OriginWhenCrossOrigin =>
                    ReferrerPolicy::OriginWhenCrossOrigin,
                ReferrerPolicyHeader::UnsafeUrl =>
                    ReferrerPolicy::UnsafeUrl,
                ReferrerPolicyHeader::StrictOrigin =>
                    ReferrerPolicy::StrictOrigin,
                ReferrerPolicyHeader::StrictOriginWhenCrossOrigin =>
                    ReferrerPolicy::StrictOriginWhenCrossOrigin,
            })
        } else {
            None
        };

        let document = Document::new(&window,
                                     Some(&browsing_context),
                                     Some(final_url.clone()),
                                     incomplete.origin,
                                     is_html_document,
                                     content_type,
                                     last_modified,
                                     DocumentSource::FromParser,
                                     loader,
                                     referrer,
                                     referrer_policy);
        document.set_ready_state(DocumentReadyState::Loading);

        if !incomplete.is_frozen {
            document.fully_activate();
        }

        self.documents.borrow_mut().insert(incomplete.pipeline_id, &*document);

        window.init_document(&document);

        self.constellation_chan
            .send(ConstellationMsg::ActivateDocument(incomplete.pipeline_id))
            .unwrap();

        // Notify devtools that a new script global exists.
        self.notify_devtools(document.Title(), final_url.clone(), (incomplete.pipeline_id, None));

        let is_javascript = incomplete.url.scheme() == "javascript";
        let parse_input = if is_javascript {
            use url::percent_encoding::percent_decode;

            // Turn javascript: URL into JS code to eval, according to the steps in
            // https://html.spec.whatwg.org/multipage/#javascript-protocol

            // This slice of the URL’s serialization is equivalent to (5.) to (7.):
            // Start with the scheme data of the parsed URL;
            // append question mark and query component, if any;
            // append number sign and fragment component if any.
            let encoded = &incomplete.url[Position::BeforePath..];

            // Percent-decode (8.) and UTF-8 decode (9.)
            let script_source = percent_decode(encoded.as_bytes()).decode_utf8_lossy();

            // Script source is ready to be evaluated (11.)
            unsafe {
                let _ac = JSAutoCompartment::new(self.get_cx(), window.reflector().get_jsobject().get());
                rooted!(in(self.get_cx()) let mut jsval = UndefinedValue());
                window.upcast::<GlobalScope>().evaluate_js_on_global_with_result(
                    &script_source, jsval.handle_mut());
                let strval = DOMString::from_jsval(self.get_cx(),
                                                   jsval.handle(),
                                                   StringificationBehavior::Empty);
                match strval {
                    Ok(ConversionResult::Success(s)) => s,
                    _ => DOMString::new(),
                }
            }
        } else {
            DOMString::new()
        };

        document.set_https_state(metadata.https_state);

        if is_html_document == IsHTMLDocument::NonHTMLDocument {
            ServoParser::parse_xml_document(
                &document,
                parse_input,
                final_url,
                Some(incomplete.pipeline_id));
        } else {
            ServoParser::parse_html_document(
                &document,
                parse_input,
                final_url,
                Some(incomplete.pipeline_id));
        }

        if incomplete.is_frozen {
            window.upcast::<GlobalScope>().suspend();
        }

        if !incomplete.is_visible {
            self.alter_resource_utilization(incomplete.pipeline_id, false);
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
        window.reflow(ReflowGoal::ForDisplay, ReflowQueryType::NoQuery, reason);
    }

    /// This is the main entry point for receiving and dispatching DOM events.
    ///
    /// TODO: Actually perform DOM event dispatch.
    fn handle_event(&self, pipeline_id: PipelineId, event: CompositorEvent) {
        match event {
            ResizeEvent(new_size, size_type) => {
                self.handle_resize_event(pipeline_id, new_size, size_type);
            }

            MouseButtonEvent(event_type, button, point) => {
                self.handle_mouse_event(pipeline_id, event_type, button, point);
            }

            MouseMoveEvent(point) => {
                let document = match { self.documents.borrow().find_document(pipeline_id) } {
                    Some(document) => document,
                    None => return warn!("Message sent to closed pipeline {}.", pipeline_id),
                };

                // Get the previous target temporarily
                let prev_mouse_over_target = self.topmost_mouse_over_target.get();

                document.handle_mouse_move_event(self.js_runtime.rt(), point,
                                                 &self.topmost_mouse_over_target);

                // Short-circuit if nothing changed
                if self.topmost_mouse_over_target.get() == prev_mouse_over_target {
                    return;
                }

                let mut state_already_changed = false;

                // Notify Constellation about the topmost anchor mouse over target.
                if let Some(target) = self.topmost_mouse_over_target.get() {
                    if let Some(anchor) = target.upcast::<Node>()
                                                .inclusive_ancestors()
                                                .filter_map(Root::downcast::<HTMLAnchorElement>)
                                                .next() {
                        let status = anchor.upcast::<Element>()
                                           .get_attribute(&ns!(), &local_name!("href"))
                                           .and_then(|href| {
                                               let value = href.value();
                                               let url = document.url();
                                               url.join(&value).map(|url| url.to_string()).ok()
                                           });

                        let event = ConstellationMsg::NodeStatus(status);
                        self.constellation_chan.send(event).unwrap();

                        state_already_changed = true;
                    }
                }

                // We might have to reset the anchor state
                if !state_already_changed {
                    if let Some(target) = prev_mouse_over_target {
                        if let Some(_) = target.upcast::<Node>()
                                               .inclusive_ancestors()
                                               .filter_map(Root::downcast::<HTMLAnchorElement>)
                                               .next() {
                            let event = ConstellationMsg::NodeStatus(None);
                            self.constellation_chan.send(event).unwrap();
                        }
                    }
                }
            }
            TouchEvent(event_type, identifier, point) => {
                let touch_result = self.handle_touch_event(pipeline_id, event_type, identifier, point);
                match (event_type, touch_result) {
                    (TouchEventType::Down, TouchEventResult::Processed(handled)) => {
                        let result = if handled {
                            // TODO: Wait to see if preventDefault is called on the first touchmove event.
                            EventResult::DefaultAllowed
                        } else {
                            EventResult::DefaultPrevented
                        };
                        let message = ConstellationMsg::TouchEventProcessed(result);
                        self.constellation_chan.send(message).unwrap();
                    }
                    _ => {
                        // TODO: Calling preventDefault on a touchup event should prevent clicks.
                    }
                }
            }

            TouchpadPressureEvent(point, pressure, phase) => {
                let doc = match { self.documents.borrow().find_document(pipeline_id) } {
                    Some(doc) => doc,
                    None => return warn!("Message sent to closed pipeline {}.", pipeline_id),
                };
                doc.handle_touchpad_pressure_event(self.js_runtime.rt(), point, pressure, phase);
            }

            KeyEvent(ch, key, state, modifiers) => {
                let document = match { self.documents.borrow().find_document(pipeline_id) } {
                    Some(document) => document,
                    None => return warn!("Message sent to closed pipeline {}.", pipeline_id),
                };
                document.dispatch_key_event(ch, key, state, modifiers, &self.constellation_chan);
            }
        }
    }

    fn handle_mouse_event(&self,
                          pipeline_id: PipelineId,
                          mouse_event_type: MouseEventType,
                          button: MouseButton,
                          point: Point2D<f32>) {
        let document = match { self.documents.borrow().find_document(pipeline_id) } {
            Some(document) => document,
            None => return warn!("Message sent to closed pipeline {}.", pipeline_id),
        };
        document.handle_mouse_event(self.js_runtime.rt(), button, point, mouse_event_type);
    }

    fn handle_touch_event(&self,
                          pipeline_id: PipelineId,
                          event_type: TouchEventType,
                          identifier: TouchId,
                          point: Point2D<f32>)
                          -> TouchEventResult {
        let document = match { self.documents.borrow().find_document(pipeline_id) } {
            Some(document) => document,
            None => {
                warn!("Message sent to closed pipeline {}.", pipeline_id);
                return TouchEventResult::Processed(true);
            },
        };
        document.handle_touch_event(self.js_runtime.rt(), event_type, identifier, point)
    }

    /// https://html.spec.whatwg.org/multipage/#navigating-across-documents
    /// The entry point for content to notify that a new load has been requested
    /// for the given pipeline (specifically the "navigate" algorithm).
    fn handle_navigate(&self, parent_pipeline_id: PipelineId,
                              frame_id: Option<FrameId>,
                              load_data: LoadData,
                              replace: bool) {
        match frame_id {
            Some(frame_id) => {
                let iframe = self.documents.borrow().find_iframe(parent_pipeline_id, frame_id);
                if let Some(iframe) = iframe {
                    iframe.navigate_or_reload_child_browsing_context(Some(load_data), replace);
                }
            }
            None => {
                self.constellation_chan
                    .send(ConstellationMsg::LoadUrl(parent_pipeline_id, load_data, replace))
                    .unwrap();
            }
        }
    }

    fn handle_resize_event(&self, pipeline_id: PipelineId, new_size: WindowSizeData, size_type: WindowSizeType) {
        let document = match { self.documents.borrow().find_document(pipeline_id) } {
            Some(document) => document,
            None => return warn!("Message sent to closed pipeline {}.", pipeline_id),
        };

        let window = document.window();
        window.set_window_size(new_size);
        window.force_reflow(ReflowGoal::ForDisplay,
                            ReflowQueryType::NoQuery,
                            ReflowReason::WindowResize);

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

    /// Initiate a non-blocking fetch for a specified resource. Stores the InProgressLoad
    /// argument until a notification is received that the fetch is complete.
    fn start_page_load(&self, incomplete: InProgressLoad, mut load_data: LoadData) {
        let id = incomplete.pipeline_id.clone();

        let context = Arc::new(Mutex::new(ParserContext::new(id, load_data.url.clone())));
        let (action_sender, action_receiver) = ipc::channel().unwrap();
        let listener = NetworkListener {
            context: context,
            task_source: self.networking_task_source.clone(),
            wrapper: None,
        };
        ROUTER.add_route(action_receiver.to_opaque(), box move |message| {
            listener.notify_fetch(message.to().unwrap());
        });

        if load_data.url.scheme() == "javascript" {
            load_data.url = ServoUrl::parse("about:blank").unwrap();
        }

        let request = RequestInit {
            url: load_data.url.clone(),
            method: load_data.method,
            destination: Destination::Document,
            credentials_mode: CredentialsMode::Include,
            use_url_credentials: true,
            origin: load_data.url,
            pipeline_id: Some(id),
            referrer_url: load_data.referrer_url,
            referrer_policy: load_data.referrer_policy,
            headers: load_data.headers,
            body: load_data.data,
            .. RequestInit::default()
        };

        self.resource_threads.send(CoreResourceMsg::Fetch(request, action_sender)).unwrap();
        self.incomplete_loads.borrow_mut().push(incomplete);
    }

    /// Synchronously fetch `about:blank`. Stores the `InProgressLoad`
    /// argument until a notification is received that the fetch is complete.
    fn start_page_load_about_blank(&self, incomplete: InProgressLoad) {
        let id = incomplete.pipeline_id;

        self.incomplete_loads.borrow_mut().push(incomplete);

        let url = ServoUrl::parse("about:blank").unwrap();
        let mut context = ParserContext::new(id, url.clone());

        let mut meta = Metadata::default(url);
        meta.set_content_type(Some(&mime!(Text / Html)));
        context.process_response(Ok(FetchMetadata::Unfiltered(meta)));
        context.process_response_chunk(vec![]);
        context.process_response_eof(Ok(()));
    }

    fn handle_parsing_complete(&self, id: PipelineId) {
        let document = match { self.documents.borrow().find_document(id) } {
            Some(document) => document,
            None => return,
        };

        let final_url = document.url();

        // https://html.spec.whatwg.org/multipage/#the-end step 1
        document.set_ready_state(DocumentReadyState::Interactive);

        // TODO: Execute step 2 here.

        // Kick off the initial reflow of the page.
        debug!("kicking off initial reflow of {:?}", final_url);
        document.disarm_reflow_timeout();
        document.upcast::<Node>().dirty(NodeDamage::OtherNodeDamage);
        let window = window_from_node(&*document);
        window.reflow(ReflowGoal::ForDisplay, ReflowQueryType::NoQuery, ReflowReason::FirstLoad);

        // No more reflow required
        window.browsing_context().set_reflow_status(false);

        // https://html.spec.whatwg.org/multipage/#the-end steps 3-4.
        document.process_deferred_scripts();
    }

    fn handle_css_error_reporting(&self, pipeline_id: PipelineId, filename: String,
                                  line: usize, column: usize, msg: String) {
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
            window.Location().Reload();
        }
    }

    pub fn enqueue_promise_job(job: EnqueuedPromiseCallback, global: &GlobalScope) {
        SCRIPT_THREAD_ROOT.with(|root| {
            let script_thread = unsafe { &*root.get().unwrap() };
            script_thread.promise_job_queue.enqueue(job, global);
        });
    }

    pub fn flush_promise_jobs(global: &GlobalScope) {
        SCRIPT_THREAD_ROOT.with(|root| {
            let script_thread = unsafe { &*root.get().unwrap() };
            let _ = script_thread.dom_manipulation_task_source.queue(
                box FlushPromiseJobs, global);
        })
    }

    fn do_flush_promise_jobs(&self) {
        self.promise_job_queue.flush_promise_jobs(|id| self.documents.borrow().find_global(id))
    }
}

struct FlushPromiseJobs;
impl Runnable for FlushPromiseJobs {
    fn main_thread_handler(self: Box<FlushPromiseJobs>, script_thread: &ScriptThread) {
        script_thread.do_flush_promise_jobs();
    }
}

impl Drop for ScriptThread {
    fn drop(&mut self) {
        SCRIPT_THREAD_ROOT.with(|root| {
            root.set(None);
        });
    }
}

/// Shuts down layout for the given window.
fn shut_down_layout(window: &Window) {
    // Tell the layout thread to begin shutting down, and wait until it
    // processed this message.
    let (response_chan, response_port) = channel();
    let chan = window.layout_chan().clone();
    if chan.send(message::Msg::PrepareToExit(response_chan)).is_ok() {
        let _ = response_port.recv();
    }

    // The browsing context is cleared by window.clear_js_runtime(), so we need to save a copy
    let browsing_context = window.browsing_context();

    // Drop our references to the JSContext and DOM objects.
    window.clear_js_runtime();

    // Discard the browsing context.
    browsing_context.discard();

    // Destroy the layout thread. If there were node leaks, layout will now crash safely.
    chan.send(message::Msg::ExitNow).ok();
}

fn dom_last_modified(tm: &Tm) -> String {
    tm.to_local().strftime("%m/%d/%Y %H:%M:%S").unwrap().to_string()
}
