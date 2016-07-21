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

use devtools;
use devtools_traits::CSSError;
use devtools_traits::{DevtoolScriptControlMsg, DevtoolsPageInfo};
use devtools_traits::{ScriptToDevtoolsControlMsg, WorkerId};
use document_loader::DocumentLoader;
use dom::bindings::cell::DOMRefCell;
use dom::bindings::codegen::Bindings::DocumentBinding::{DocumentMethods, DocumentReadyState};
use dom::bindings::codegen::Bindings::LocationBinding::LocationMethods;
use dom::bindings::codegen::Bindings::WindowBinding::WindowMethods;
use dom::bindings::conversions::{FromJSValConvertible, StringificationBehavior};
use dom::bindings::global::GlobalRef;
use dom::bindings::inheritance::Castable;
use dom::bindings::js::{JS, MutNullableHeap, Root, RootCollection};
use dom::bindings::js::{RootCollectionPtr, RootedReference};
use dom::bindings::refcounted::{LiveDOMReferences, Trusted};
use dom::bindings::reflector::Reflectable;
use dom::bindings::str::DOMString;
use dom::bindings::trace::JSTraceable;
use dom::bindings::utils::WRAP_CALLBACKS;
use dom::browsingcontext::BrowsingContext;
use dom::document::{Document, DocumentProgressHandler, DocumentSource, FocusType, IsHTMLDocument};
use dom::element::Element;
use dom::event::{Event, EventBubbles, EventCancelable};
use dom::htmlanchorelement::HTMLAnchorElement;
use dom::node::{Node, NodeDamage, window_from_node};
use dom::serviceworker::TrustedServiceWorkerAddress;
use dom::serviceworkerregistration::ServiceWorkerRegistration;
use dom::servohtmlparser::ParserContext;
use dom::uievent::UIEvent;
use dom::window::{ReflowReason, ScriptHelpers, Window};
use dom::worker::TrustedWorkerAddress;
use euclid::Rect;
use euclid::point::Point2D;
use gfx_traits::LayerId;
use hyper::header::{ContentType, Headers, HttpDate, LastModified};
use hyper::header::{ReferrerPolicy as ReferrerPolicyHeader};
use hyper::method::Method;
use hyper::mime::{Mime, SubLevel, TopLevel};
use ipc_channel::ipc::{self, IpcSender};
use ipc_channel::router::ROUTER;
use js::glue::GetWindowProxyClass;
use js::jsapi::{DOMProxyShadowsResult, HandleId, HandleObject};
use js::jsapi::{JSAutoCompartment, JSContext, JS_SetWrapObjectCallbacks};
use js::jsapi::{JSTracer, SetWindowProxyClass};
use js::jsval::UndefinedValue;
use js::rust::Runtime;
use mem::heap_size_of_self_and_children;
use msg::constellation_msg::{FrameType, LoadData, PipelineId, PipelineNamespace};
use msg::constellation_msg::{SubpageId, WindowSizeType, ReferrerPolicy};
use net_traits::bluetooth_thread::BluetoothMethodMsg;
use net_traits::image_cache_thread::{ImageCacheChan, ImageCacheResult, ImageCacheThread};
use net_traits::{AsyncResponseTarget, CoreResourceMsg, LoadConsumer, LoadContext, Metadata, ResourceThreads};
use net_traits::{IpcSend, LoadData as NetLoadData};
use network_listener::NetworkListener;
use parse::ParserRoot;
use parse::html::{ParseContext, parse_html};
use parse::xml::{self, parse_xml};
use profile_traits::mem::{self, OpaqueSender, Report, ReportKind, ReportsChan};
use profile_traits::time::{self, ProfilerCategory, profile};
use script_layout_interface::message::{self, NewLayoutThreadInfo, ReflowQueryType};
use script_runtime::{CommonScriptMsg, ScriptChan, ScriptThreadEventCategory};
use script_runtime::{ScriptPort, StackRootTLS, new_rt_and_cx, get_reports};
use script_traits::CompositorEvent::{KeyEvent, MouseButtonEvent, MouseMoveEvent, ResizeEvent};
use script_traits::CompositorEvent::{TouchEvent, TouchpadPressureEvent};
use script_traits::webdriver_msg::WebDriverScriptCommand;
use script_traits::{CompositorEvent, ConstellationControlMsg, EventResult};
use script_traits::{InitialScriptState, MouseButton, MouseEventType, MozBrowserEvent};
use script_traits::{NewLayoutInfo, ScriptMsg as ConstellationMsg};
use script_traits::{ScriptThreadFactory, TimerEvent, TimerEventRequest, TimerSource};
use script_traits::{TouchEventType, TouchId, UntrustedNodeAddress, WindowSizeData};
use std::borrow::ToOwned;
use std::cell::Cell;
use std::collections::{HashMap, HashSet};
use std::option::Option;
use std::ptr;
use std::rc::Rc;
use std::result::Result;
use std::sync::atomic::{Ordering, AtomicBool};
use std::sync::mpsc::{Receiver, Select, Sender, channel};
use std::sync::{Arc, Mutex};
use style::context::ReflowGoal;
use task_source::TaskSource;
use task_source::dom_manipulation::{DOMManipulationTaskSource, DOMManipulationTask};
use task_source::file_reading::FileReadingTaskSource;
use task_source::history_traversal::HistoryTraversalTaskSource;
use task_source::networking::NetworkingTaskSource;
use task_source::user_interaction::{UserInteractionTaskSource, UserInteractionTask};
use time::Tm;
use url::{Url, Position};
use util::opts;
use util::thread;
use util::thread_state;
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
    /// The parent pipeline and child subpage associated with this load, if any.
    parent_info: Option<(PipelineId, SubpageId, FrameType)>,
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
    url: Url,
}

impl InProgressLoad {
    /// Create a new InProgressLoad object.
    fn new(id: PipelineId,
           parent_info: Option<(PipelineId, SubpageId, FrameType)>,
           layout_chan: Sender<message::Msg>,
           window_size: Option<WindowSizeData>,
           url: Url) -> InProgressLoad {
        InProgressLoad {
            pipeline_id: id,
            parent_info: parent_info,
            layout_chan: layout_chan,
            window_size: window_size,
            clip_rect: None,
            is_frozen: false,
            is_visible: true,
            url: url,
        }
    }
}

/// Encapsulated state required to create cancellable runnables from non-script threads.
pub struct RunnableWrapper {
    pub cancelled: Arc<AtomicBool>,
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
    cancelled: Arc<AtomicBool>,
    inner: Box<T>,
}

impl<T: Runnable + Send> Runnable for CancellableRunnable<T> {
    fn name(&self) -> &'static str { self.inner.name() }

    fn is_cancelled(&self) -> bool {
        self.cancelled.load(Ordering::SeqCst)
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
    /// dispatched to ScriptThread).
    Navigate(PipelineId, LoadData),
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

/// Information for an entire page. Pages are top-level browsing contexts and can contain multiple
/// frames.
#[derive(JSTraceable)]
// ScriptThread instances are rooted on creation, so this is okay
#[allow(unrooted_must_root)]
pub struct ScriptThread {
    /// A handle to the information pertaining to page layout
    browsing_context: MutNullableHeap<JS<BrowsingContext>>,
    /// A list of data pertaining to loads that have not yet received a network response
    incomplete_loads: DOMRefCell<Vec<InProgressLoad>>,
    /// A map to store service worker registrations for a given origin
    registration_map: DOMRefCell<HashMap<Url, JS<ServiceWorkerRegistration>>>,
    /// A handle to the image cache thread.
    image_cache_thread: ImageCacheThread,
    /// A handle to the resource thread. This is an `Arc` to avoid running out of file descriptors if
    /// there are many iframes.
    resource_threads: ResourceThreads,
    /// A handle to the bluetooth thread.
    bluetooth_thread: IpcSender<BluetoothMethodMsg>,

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
    topmost_mouse_over_target: MutNullableHeap<JS<Element>>,

    /// List of pipelines that have been owned and closed by this script thread.
    closed_pipelines: DOMRefCell<HashSet<PipelineId>>,

    scheduler_chan: IpcSender<TimerEventRequest>,
    timer_event_chan: Sender<TimerEvent>,
    timer_event_port: Receiver<TimerEvent>,

    content_process_shutdown_chan: IpcSender<()>,
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
        match self.owner {
            Some(owner) => {
                let context = owner.browsing_context.get();
                for context in context.iter() {
                    if let Some(document) = context.maybe_active_document() {
                        let window = document.window();
                        window.clear_js_runtime_for_script_deallocation();
                    }
                }
            }
            None => (),
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
        let pipeline_id = state.id;
        thread::spawn_named(format!("ScriptThread {:?}", state.id),
                            move || {
            thread_state::initialize(thread_state::SCRIPT);
            PipelineId::install(pipeline_id);
            PipelineNamespace::install(state.pipeline_namespace_id);
            let roots = RootCollection::new();
            let _stack_roots_tls = StackRootTLS::new(&roots);
            let id = state.id;
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

            let new_load = InProgressLoad::new(id, parent_info, layout_chan, window_size,
                                               load_data.url.clone());
            script_thread.start_page_load(new_load, load_data);

            let reporter_name = format!("script-reporter-{}", id);
            mem_profiler_chan.run_with_memory_reporting(|| {
                script_thread.start();
                let _ = script_thread.content_process_shutdown_chan.send(());
            }, reporter_name, script_chan, CommonScriptMsg::CollectReports);

            // This must always be the very last operation performed before the thread completes
            failsafe.neuter();
        });

        (sender, receiver)
    }
}

pub unsafe extern "C" fn shadow_check_callback(_cx: *mut JSContext,
    _object: HandleObject, _id: HandleId) -> DOMProxyShadowsResult {
    // XXX implement me
    DOMProxyShadowsResult::ShadowCheckFailed
}

impl ScriptThread {
    pub fn page_headers_available(id: &PipelineId, subpage: Option<&SubpageId>, metadata: Option<Metadata>)
                                  -> Option<ParserRoot> {
        SCRIPT_THREAD_ROOT.with(|root| {
            let script_thread = unsafe { &*root.get().unwrap() };
            script_thread.handle_page_headers_available(id, subpage, metadata)
        })
    }

    // stores a service worker registration
    pub fn set_registration(scope_url: Url, registration:&ServiceWorkerRegistration, pipeline_id: PipelineId) {
        SCRIPT_THREAD_ROOT.with(|root| {
            let script_thread = unsafe { &*root.get().unwrap() };
            script_thread.handle_serviceworker_registration(scope_url, registration, pipeline_id);
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
            browsing_context: MutNullableHeap::new(None),
            incomplete_loads: DOMRefCell::new(vec!()),
            registration_map: DOMRefCell::new(HashMap::new()),

            image_cache_thread: state.image_cache_thread,
            image_cache_channel: ImageCacheChan(ipc_image_cache_channel),
            image_cache_port: image_cache_port,

            resource_threads: state.resource_threads,
            bluetooth_thread: state.bluetooth_thread,

            port: port,

            chan: MainThreadScriptChan(chan.clone()),
            dom_manipulation_task_source: DOMManipulationTaskSource(chan.clone()),
            user_interaction_task_source: UserInteractionTaskSource(chan.clone()),
            networking_task_source: NetworkingTaskSource(chan.clone()),
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
            topmost_mouse_over_target: MutNullableHeap::new(Default::default()),
            closed_pipelines: DOMRefCell::new(HashSet::new()),

            scheduler_chan: state.scheduler_chan,
            timer_event_chan: timer_event_chan,
            timer_event_port: timer_event_port,

            content_process_shutdown_chan: state.content_process_shutdown_chan,
        }
    }

    // Return the root browsing context in the frame tree. Panics if it doesn't exist.
    pub fn root_browsing_context(&self) -> Root<BrowsingContext> {
        self.browsing_context.get().unwrap()
    }

    fn root_browsing_context_exists(&self) -> bool {
        self.browsing_context.get().is_some()
    }

    /// Find a child browsing context of the root context by pipeline id. Returns `None` if the
    /// root context does not exist or the child context cannot be found.
    fn find_child_context(&self, pipeline_id: PipelineId) -> Option<Root<BrowsingContext>> {
        self.browsing_context.get().and_then(|context| context.find(pipeline_id))
    }

    pub fn get_cx(&self) -> *mut JSContext {
        self.js_runtime.cx()
    }

    /// Starts the script thread. After calling this method, the script thread will loop receiving
    /// messages on its port.
    pub fn start(&self) {
        while self.handle_msgs() {
            // Go on...
        }
    }

    /// Handle incoming control messages.
    fn handle_msgs(&self) -> bool {
        use self::MixedMessage::{FromConstellation, FromDevtools, FromImageCache};
        use self::MixedMessage::{FromScheduler, FromScript};

        // Handle pending resize events.
        // Gather them first to avoid a double mut borrow on self.
        let mut resizes = vec!();

        let context = self.browsing_context.get();
        if let Some(context) = context {
            for context in context.iter() {
                // Only process a resize if layout is idle.
                let window = context.active_window();
                let resize_event = window.steal_resize_event();
                match resize_event {
                    Some(size) => resizes.push((window.pipeline(), size)),
                    None => ()
                }
            }
        }

        for (id, (size, size_type)) in resizes {
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
                        self.handle_new_layout(new_layout_info);
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
                    FromConstellation(ConstellationControlMsg::ExitPipeline(id)) => {
                        if self.handle_exit_pipeline_msg(id) {
                            return Some(false)
                        }
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
        let context = self.browsing_context.get();
        if let Some(context) = context {
            for context in context.iter() {
                let window = context.active_window();
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
                ScriptThreadEventCategory::ServiceWorkerEvent => ProfilerCategory::ScriptServiceWorkerEvent
            };
            profile(profiler_cat, None, self.time_profiler_chan.clone(), f)
        } else {
            f()
        }
    }

    fn handle_msg_from_constellation(&self, msg: ConstellationControlMsg) {
        match msg {
            ConstellationControlMsg::Navigate(pipeline_id, subpage_id, load_data) =>
                self.handle_navigate(pipeline_id, Some(subpage_id), load_data),
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
            ConstellationControlMsg::NotifyVisibilityChange(containing_id, pipeline_id, visible) =>
                self.handle_visibility_change_complete_msg(containing_id, pipeline_id, visible),
            ConstellationControlMsg::MozBrowserEvent(parent_pipeline_id,
                                                     subpage_id,
                                                     event) =>
                self.handle_mozbrowser_event_msg(parent_pipeline_id,
                                                 subpage_id,
                                                 event),
            ConstellationControlMsg::UpdateSubpageId(containing_pipeline_id,
                                                     old_subpage_id,
                                                     new_subpage_id,
                                                     new_pipeline_id) =>
                self.handle_update_subpage_id(containing_pipeline_id,
                                              old_subpage_id,
                                              new_subpage_id,
                                              new_pipeline_id),
            ConstellationControlMsg::FocusIFrame(containing_pipeline_id, subpage_id) =>
                self.handle_focus_iframe_msg(containing_pipeline_id, subpage_id),
            ConstellationControlMsg::WebDriverScriptCommand(pipeline_id, msg) =>
                self.handle_webdriver_msg(pipeline_id, msg),
            ConstellationControlMsg::TickAllAnimations(pipeline_id) =>
                self.handle_tick_all_animations(pipeline_id),
            ConstellationControlMsg::WebFontLoaded(pipeline_id) =>
                self.handle_web_font_loaded(pipeline_id),
            ConstellationControlMsg::DispatchFrameLoadEvent {
                target: pipeline_id, parent: containing_id } =>
                self.handle_frame_load_event(containing_id, pipeline_id),
            ConstellationControlMsg::FramedContentChanged(containing_pipeline_id, subpage_id) =>
                self.handle_framed_content_changed(containing_pipeline_id, subpage_id),
            ConstellationControlMsg::ReportCSSError(pipeline_id, filename, line, column, msg) =>
                self.handle_css_error_reporting(pipeline_id, filename, line, column, msg),
            ConstellationControlMsg::Reload(pipeline_id) =>
                self.handle_reload(pipeline_id),
            msg @ ConstellationControlMsg::AttachLayout(..) |
            msg @ ConstellationControlMsg::Viewport(..) |
            msg @ ConstellationControlMsg::SetScrollState(..) |
            msg @ ConstellationControlMsg::Resize(..) |
            msg @ ConstellationControlMsg::ExitPipeline(..) =>
                      panic!("should have handled {:?} already", msg),
        }
    }

    fn handle_msg_from_script(&self, msg: MainThreadScriptMsg) {
        match msg {
            MainThreadScriptMsg::Navigate(id, load_data) =>
                self.handle_navigate(id, None, load_data),
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
            MainThreadScriptMsg::Common(CommonScriptMsg::RefcountCleanup(addr)) =>
                LiveDOMReferences::cleanup(addr),
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

        let context = self.root_browsing_context();
        let context = context.find(pipeline_id).expect("ScriptThread: received fire timer msg for a
            pipeline ID not associated with this script thread. This is a bug.");
        let window = context.active_window();

        window.handle_fire_timer(id);
    }

    fn handle_msg_from_devtools(&self, msg: DevtoolScriptControlMsg) {
        let context = self.root_browsing_context();
        match msg {
            DevtoolScriptControlMsg::EvaluateJS(id, s, reply) => {
                let window = match context.find(id) {
                    Some(browsing_context) => browsing_context.active_window(),
                    None => return warn!("Message sent to closed pipeline {}.", id),
                };
                let global_ref = GlobalRef::Window(window.r());
                devtools::handle_evaluate_js(&global_ref, s, reply)
            },
            DevtoolScriptControlMsg::GetRootNode(id, reply) =>
                devtools::handle_get_root_node(&context, id, reply),
            DevtoolScriptControlMsg::GetDocumentElement(id, reply) =>
                devtools::handle_get_document_element(&context, id, reply),
            DevtoolScriptControlMsg::GetChildren(id, node_id, reply) =>
                devtools::handle_get_children(&context, id, node_id, reply),
            DevtoolScriptControlMsg::GetLayout(id, node_id, reply) =>
                devtools::handle_get_layout(&context, id, node_id, reply),
            DevtoolScriptControlMsg::GetCachedMessages(pipeline_id, message_types, reply) =>
                devtools::handle_get_cached_messages(pipeline_id, message_types, reply),
            DevtoolScriptControlMsg::ModifyAttribute(id, node_id, modifications) =>
                devtools::handle_modify_attribute(&context, id, node_id, modifications),
            DevtoolScriptControlMsg::WantsLiveNotifications(id, to_send) => {
                let window = match context.find(id) {
                    Some(browsing_context) => browsing_context.active_window(),
                    None => return warn!("Message sent to closed pipeline {}.", id),
                };
                let global_ref = GlobalRef::Window(window.r());
                devtools::handle_wants_live_notifications(&global_ref, to_send)
            },
            DevtoolScriptControlMsg::SetTimelineMarkers(_pipeline_id, marker_types, reply) =>
                devtools::handle_set_timeline_markers(&context, marker_types, reply),
            DevtoolScriptControlMsg::DropTimelineMarkers(_pipeline_id, marker_types) =>
                devtools::handle_drop_timeline_markers(&context, marker_types),
            DevtoolScriptControlMsg::RequestAnimationFrame(pipeline_id, name) =>
                devtools::handle_request_animation_frame(&context, pipeline_id, name),
            DevtoolScriptControlMsg::Reload(pipeline_id) =>
                devtools::handle_reload(&context, pipeline_id),
        }
    }

    fn handle_msg_from_image_cache(&self, msg: ImageCacheResult) {
        msg.responder.unwrap().respond(msg.image_response);
    }

    fn handle_webdriver_msg(&self, pipeline_id: PipelineId, msg: WebDriverScriptCommand) {
        let context = self.root_browsing_context();
        match msg {
            WebDriverScriptCommand::AddCookie(params, reply) =>
                webdriver_handlers::handle_add_cookie(&context, pipeline_id, params, reply),
            WebDriverScriptCommand::ExecuteScript(script, reply) =>
                webdriver_handlers::handle_execute_script(&context, pipeline_id, script, reply),
            WebDriverScriptCommand::FindElementCSS(selector, reply) =>
                webdriver_handlers::handle_find_element_css(&context, pipeline_id, selector, reply),
            WebDriverScriptCommand::FindElementsCSS(selector, reply) =>
                webdriver_handlers::handle_find_elements_css(&context, pipeline_id, selector, reply),
            WebDriverScriptCommand::FocusElement(element_id, reply) =>
                webdriver_handlers::handle_focus_element(&context, pipeline_id, element_id, reply),
            WebDriverScriptCommand::GetActiveElement(reply) =>
                webdriver_handlers::handle_get_active_element(&context, pipeline_id, reply),
            WebDriverScriptCommand::GetCookies(reply) =>
                webdriver_handlers::handle_get_cookies(&context, pipeline_id, reply),
            WebDriverScriptCommand::GetCookie(name, reply) =>
                webdriver_handlers::handle_get_cookie(&context, pipeline_id, name, reply),
            WebDriverScriptCommand::GetElementTagName(node_id, reply) =>
                webdriver_handlers::handle_get_name(&context, pipeline_id, node_id, reply),
            WebDriverScriptCommand::GetElementAttribute(node_id, name, reply) =>
                webdriver_handlers::handle_get_attribute(&context, pipeline_id, node_id, name, reply),
            WebDriverScriptCommand::GetElementCSS(node_id, name, reply) =>
                webdriver_handlers::handle_get_css(&context, pipeline_id, node_id, name, reply),
            WebDriverScriptCommand::GetElementRect(node_id, reply) =>
                webdriver_handlers::handle_get_rect(&context, pipeline_id, node_id, reply),
            WebDriverScriptCommand::GetElementText(node_id, reply) =>
                webdriver_handlers::handle_get_text(&context, pipeline_id, node_id, reply),
            WebDriverScriptCommand::GetFrameId(frame_id, reply) =>
                webdriver_handlers::handle_get_frame_id(&context, pipeline_id, frame_id, reply),
            WebDriverScriptCommand::GetUrl(reply) =>
                webdriver_handlers::handle_get_url(&context, pipeline_id, reply),
            WebDriverScriptCommand::IsEnabled(element_id, reply) =>
                webdriver_handlers::handle_is_enabled(&context, pipeline_id, element_id, reply),
            WebDriverScriptCommand::IsSelected(element_id, reply) =>
                webdriver_handlers::handle_is_selected(&context, pipeline_id, element_id, reply),
            WebDriverScriptCommand::GetTitle(reply) =>
                webdriver_handlers::handle_get_title(&context, pipeline_id, reply),
            WebDriverScriptCommand::ExecuteAsyncScript(script, reply) =>
                webdriver_handlers::handle_execute_async_script(&context, pipeline_id, script, reply),
        }
    }

    fn handle_resize(&self, id: PipelineId, size: WindowSizeData, size_type: WindowSizeType) {
        if let Some(ref context) = self.find_child_context(id) {
            let window = context.active_window();
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
        let context = self.browsing_context.get();
        if let Some(context) = context {
            if let Some(inner_context) = context.find(id) {
                let window = inner_context.active_window();
                if window.set_page_clip_rect_with_new_viewport(rect) {
                    self.rebuild_and_force_reflow(&inner_context, ReflowReason::Viewport);
                }
                return;
            }
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
        let window = match self.browsing_context.get() {
            Some(context) => {
                match context.find(id) {
                    Some(inner_context) => inner_context.active_window(),
                    None => {
                        panic!("Set scroll state message sent to nonexistent pipeline: {:?}", id)
                    }
                }
            }
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

    fn handle_new_layout(&self, new_layout_info: NewLayoutInfo) {
        let NewLayoutInfo {
            containing_pipeline_id,
            new_pipeline_id,
            subpage_id,
            frame_type,
            load_data,
            paint_chan,
            pipeline_port,
            layout_to_constellation_chan,
            content_process_shutdown_chan,
        } = new_layout_info;

        let layout_pair = channel();
        let layout_chan = layout_pair.0.clone();

        let layout_creation_info = NewLayoutThreadInfo {
            id: new_pipeline_id,
            url: load_data.url.clone(),
            is_parent: false,
            layout_pair: layout_pair,
            pipeline_port: pipeline_port,
            constellation_chan: layout_to_constellation_chan,
            paint_chan: paint_chan,
            script_chan: self.control_chan.clone(),
            image_cache_thread: self.image_cache_thread.clone(),
            content_process_shutdown_chan: content_process_shutdown_chan,
        };

        let context = self.root_browsing_context();
        let parent_context = context.find(containing_pipeline_id).expect("ScriptThread: received a layout
            whose parent has a PipelineId which does not correspond to a pipeline in the script
            thread's browsing context tree. This is a bug.");
        let parent_window = parent_context.active_window();

        // Tell layout to actually spawn the thread.
        parent_window.layout_chan()
                     .send(message::Msg::CreateLayoutThread(layout_creation_info))
                     .unwrap();

        // Kick off the fetch for the new resource.
        let new_load = InProgressLoad::new(new_pipeline_id, Some((containing_pipeline_id, subpage_id, frame_type)),
                                           layout_chan, parent_window.window_size(),
                                           load_data.url.clone());
        self.start_page_load(new_load, load_data);
    }

    fn handle_loads_complete(&self, pipeline: PipelineId) {
        let doc = match self.root_browsing_context().find(pipeline) {
            Some(browsing_context) => browsing_context.active_document(),
            None => return warn!("Message sent to closed pipeline {}.", pipeline),
        };
        let doc = doc.r();
        if doc.loader().is_blocked() {
            return;
        }

        doc.mut_loader().inhibit_events();

        // https://html.spec.whatwg.org/multipage/#the-end step 7
        let handler = box DocumentProgressHandler::new(Trusted::new(doc));
        self.dom_manipulation_task_source.queue(handler, GlobalRef::Window(doc.window())).unwrap();

        self.constellation_chan.send(ConstellationMsg::LoadComplete(pipeline)).unwrap();
    }

    fn collect_reports(&self, reports_chan: ReportsChan) {
        let mut urls = vec![];
        let mut dom_tree_size = 0;
        let mut reports = vec![];

        if let Some(root_context) = self.browsing_context.get() {
            for it_context in root_context.iter() {
                let current_url = it_context.active_document().url().to_string();

                for child in it_context.active_document().upcast::<Node>().traverse_preorder() {
                    dom_tree_size += heap_size_of_self_and_children(&*child);
                }
                let window = it_context.active_window();
                dom_tree_size += heap_size_of_self_and_children(&*window);

                reports.push(Report {
                    path: path![format!("url({})", current_url), "dom-tree"],
                    kind: ReportKind::ExplicitJemallocHeapSize,
                    size: dom_tree_size,
                });
                urls.push(current_url);
            }
        }
        let path_seg = format!("url({})", urls.join(", "));
        reports.extend(get_reports(self.get_cx(), path_seg));
        reports_chan.send(reports);
    }

    /// To slow/speed up timers and manage any other script thread resource based on visibility.
    /// Returns true if successful.
    fn alter_resource_utilization(&self, id: PipelineId, visible: bool) -> bool {
        if let Some(root_context) = self.browsing_context.get() {
            if let Some(ref inner_context) = root_context.find(id) {
                let window = inner_context.active_window();
                if visible {
                    window.speed_up_timers();
                } else {
                    window.slow_down_timers();
                }
                return true;
            }
        }
        false
    }

    /// Updates iframe element after a change in visibility
    fn handle_visibility_change_complete_msg(&self, containing_id: PipelineId, id: PipelineId, visible: bool) {
        if let Some(root_context) = self.browsing_context.get() {
            if let Some(ref inner_context) = root_context.find(containing_id) {
                if let Some(iframe) = inner_context.active_document().find_iframe_by_pipeline(id) {
                    iframe.change_visibility_status(visible);
                }
            }
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
        if let Some(root_context) = self.browsing_context.get() {
            if let Some(ref inner_context) = root_context.find(id) {
                let window = inner_context.active_window();
                window.freeze();
                return;
            }
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
        if let Some(inner_context) = self.root_browsing_context().find(id) {
            let needed_reflow = inner_context.set_reflow_status(false);
            if needed_reflow {
                self.rebuild_and_force_reflow(&inner_context, ReflowReason::CachedPageNeededReflow);
            }
            let window = inner_context.active_window();
            window.thaw();
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
                               subpage_id: SubpageId) {
        let borrowed_context = self.root_browsing_context();
        let context = borrowed_context.find(parent_pipeline_id).unwrap();

        let doc = context.active_document();
        let frame_element = doc.find_iframe(subpage_id);

        if let Some(ref frame_element) = frame_element {
            doc.begin_focus_transaction();
            doc.request_focus(frame_element.upcast());
            doc.commit_focus_transaction(FocusType::Parent);
        }
    }

    fn handle_framed_content_changed(&self,
                                     parent_pipeline_id: PipelineId,
                                     subpage_id: SubpageId) {
        let root_context = self.root_browsing_context();
        let context = root_context.find(parent_pipeline_id).unwrap();
        let doc = context.active_document();
        let frame_element = doc.find_iframe(subpage_id);
        if let Some(ref frame_element) = frame_element {
            frame_element.upcast::<Node>().dirty(NodeDamage::OtherNodeDamage);
            let window = context.active_window();
            window.reflow(ReflowGoal::ForDisplay,
                          ReflowQueryType::NoQuery,
                          ReflowReason::FramedContentChanged);
        }
    }

    /// Handles a mozbrowser event, for example see:
    /// https://developer.mozilla.org/en-US/docs/Web/Events/mozbrowserloadstart
    fn handle_mozbrowser_event_msg(&self,
                                   parent_pipeline_id: PipelineId,
                                   subpage_id: Option<SubpageId>,
                                   event: MozBrowserEvent) {
        match self.root_browsing_context().find(parent_pipeline_id) {
            None => warn!("Mozbrowser event after pipeline {:?} closed.", parent_pipeline_id),
            Some(context) => match subpage_id {
                None => context.active_window().dispatch_mozbrowser_event(event),
                Some(subpage_id) => match context.active_document().find_iframe(subpage_id) {
                    None => warn!("Mozbrowser event after iframe {:?}/{:?} closed.", parent_pipeline_id, subpage_id),
                    Some(frame_element) => frame_element.dispatch_mozbrowser_event(event),
                },
            },
        }
    }

    fn handle_update_subpage_id(&self,
                                containing_pipeline_id: PipelineId,
                                old_subpage_id: SubpageId,
                                new_subpage_id: SubpageId,
                                new_pipeline_id: PipelineId) {
        let borrowed_context = self.root_browsing_context();

        let frame_element = borrowed_context.find(containing_pipeline_id).and_then(|context| {
            let doc = context.active_document();
            doc.find_iframe(old_subpage_id)
        });

        frame_element.unwrap().update_subpage_id(new_subpage_id, new_pipeline_id);
    }

    /// Window was resized, but this script was not active, so don't reflow yet
    fn handle_resize_inactive_msg(&self, id: PipelineId, new_size: WindowSizeData) {
        let context = self.root_browsing_context();
        let context = context.find(id).expect("Received resize message for PipelineId not associated
            with a browsing context in the browsing context tree. This is a bug.");
        let window = context.active_window();
        window.set_window_size(new_size);
        context.set_reflow_status(true);
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
    fn handle_page_headers_available(&self, id: &PipelineId, subpage: Option<&SubpageId>,
                                     metadata: Option<Metadata>) -> Option<ParserRoot> {
        let idx = self.incomplete_loads.borrow().iter().position(|load| {
            load.pipeline_id == *id && load.parent_info.as_ref().map(|info| &info.1) == subpage
        });
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

    fn handle_serviceworker_registration(&self,
                                         scope: Url,
                                         registration: &ServiceWorkerRegistration,
                                         pipeline_id: PipelineId) {
        {
            let ref mut reg_ref = *self.registration_map.borrow_mut();
            // according to spec we should replace if an older registration exists for
            // same scope otherwise just insert the new one
            let _ = reg_ref.remove(&scope);
            reg_ref.insert(scope.clone(), JS::from_ref(registration));
        }

        // send ScopeThings to sw-manager
        let ref maybe_registration_ref = *self.registration_map.borrow();
        let maybe_registration = match maybe_registration_ref.get(&scope) {
            Some(r) => r,
            None => return
        };
        if let Some(context) = self.root_browsing_context().find(pipeline_id) {
            let window = context.active_window();
            let global_ref = GlobalRef::Window(window.r());
            let script_url = maybe_registration.get_installed().get_script_url();
            let scope_things = ServiceWorkerRegistration::create_scope_things(global_ref, script_url);
            let _ = self.constellation_chan.send(ConstellationMsg::RegisterServiceWorker(scope_things, scope));
        } else {
            warn!("Registration failed for {}", pipeline_id);
        }
    }

    /// Handles a request for the window title.
    fn handle_get_title_msg(&self, pipeline_id: PipelineId) {
        let document = match self.root_browsing_context().find(pipeline_id) {
            Some(browsing_context) => browsing_context.active_document(),
            None => return warn!("Message sent to closed pipeline {}.", pipeline_id),
        };
        document.send_title_to_compositor();
    }

    /// Handles a request to exit the script thread and shut down layout.
    /// Returns true if the script thread should shut down and false otherwise.
    fn handle_exit_pipeline_msg(&self, id: PipelineId) -> bool {
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
                debug!("shutting down layout for page {:?}", id);
                response_port.recv().unwrap();
                chan.send(message::Msg::ExitNow).ok();
            }

            let has_pending_loads = self.incomplete_loads.borrow().len() > 0;
            let has_root_context = self.root_browsing_context_exists();

            // Exit if no pending loads and no root context
            return !has_pending_loads && !has_root_context;
        }

        // If root is being exited, shut down all contexts
        let context = self.root_browsing_context();
        let window = context.active_window();
        if window.pipeline() == id {
            debug!("shutting down layout for root context {:?}", id);
            shut_down_layout(&context);
            let _ = self.constellation_chan.send(ConstellationMsg::PipelineExited(id));
            return true
        }

        // otherwise find just the matching context and exit all sub-contexts
        if let Some(ref mut child_context) = context.remove(id) {
            shut_down_layout(&child_context);
        }
        let _ = self.constellation_chan.send(ConstellationMsg::PipelineExited(id));
        false
    }

    /// Handles when layout thread finishes all animation in one tick
    fn handle_tick_all_animations(&self, id: PipelineId) {
        let document = match self.root_browsing_context().find(id) {
            Some(browsing_context) => browsing_context.active_document(),
            None => return warn!("Message sent to closed pipeline {}.", id),
        };
        document.run_the_animation_frame_callbacks();
    }

    /// Handles a Web font being loaded. Does nothing if the page no longer exists.
    fn handle_web_font_loaded(&self, pipeline_id: PipelineId) {
        if let Some(context) = self.find_child_context(pipeline_id)  {
            self.rebuild_and_force_reflow(&context, ReflowReason::WebFontLoaded);
        }
    }

    /// Notify the containing document of a child frame that has completed loading.
    fn handle_frame_load_event(&self, containing_pipeline: PipelineId, id: PipelineId) {
        let document = match self.root_browsing_context().find(containing_pipeline) {
            Some(browsing_context) => browsing_context.active_document(),
            None => return warn!("Message sent to closed pipeline {}.", containing_pipeline),
        };
        if let Some(iframe) = document.find_iframe_by_pipeline(id) {
            iframe.iframe_load_event_steps(id);
        }
    }

    /// The entry point to document loading. Defines bindings, sets up the window and document
    /// objects, parses HTML and CSS, and kicks off initial layout.
    fn load(&self, metadata: Metadata, incomplete: InProgressLoad) -> ParserRoot {
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

        let frame_element = incomplete.parent_info.and_then(|(parent_id, subpage_id, _)| {
            // The root context may not exist yet, if the parent of this frame
            // exists in a different script thread.
            let root_context = self.browsing_context.get();

            // In the case a parent id exists but the matching context
            // cannot be found, this means the context exists in a different
            // script thread (due to origin) so it shouldn't be returned.
            // TODO: window.parent will continue to return self in that
            // case, which is wrong. We should be returning an object that
            // denies access to most properties (per
            // https://github.com/servo/servo/issues/3939#issuecomment-62287025).
            root_context.and_then(|root_context| {
                root_context.find(parent_id).and_then(|context| {
                    let doc = context.active_document();
                    doc.find_iframe(subpage_id)
                })
            })
        });

        let MainThreadScriptChan(ref sender) = self.chan;
        let DOMManipulationTaskSource(ref dom_sender) = self.dom_manipulation_task_source;
        let UserInteractionTaskSource(ref user_sender) = self.user_interaction_task_source;
        let NetworkingTaskSource(ref network_sender) = self.networking_task_source;
        let HistoryTraversalTaskSource(ref history_sender) = self.history_traversal_task_source;

        let (ipc_timer_event_chan, ipc_timer_event_port) = ipc::channel().unwrap();
        ROUTER.route_ipc_receiver_to_mpsc_sender(ipc_timer_event_port,
                                                 self.timer_event_chan.clone());

        // Create the window and document objects.
        let window = Window::new(self.js_runtime.clone(),
                                 MainThreadScriptChan(sender.clone()),
                                 DOMManipulationTaskSource(dom_sender.clone()),
                                 UserInteractionTaskSource(user_sender.clone()),
                                 NetworkingTaskSource(network_sender.clone()),
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

        enum ContextToRemove {
            Root,
            Child(PipelineId),
            None,
        }
        struct AutoContextRemover<'a> {
            context: ContextToRemove,
            script_thread: &'a ScriptThread,
            neutered: bool,
        }
        impl<'a> AutoContextRemover<'a> {
            fn new(script_thread: &'a ScriptThread, context: ContextToRemove) -> AutoContextRemover<'a> {
                AutoContextRemover {
                    context: context,
                    script_thread: script_thread,
                    neutered: false,
                }
            }

            fn neuter(&mut self) {
                self.neutered = true;
            }
        }

        impl<'a> Drop for AutoContextRemover<'a> {
            fn drop(&mut self) {
                if !self.neutered {
                    match self.context {
                        ContextToRemove::Root => {
                            self.script_thread.browsing_context.set(None)
                        },
                        ContextToRemove::Child(id) => {
                            self.script_thread.root_browsing_context().remove(id).unwrap();
                        },
                        ContextToRemove::None => {},
                    }
                }
            }
        }

        let mut using_new_context = true;

        let (browsing_context, context_to_remove) = if !self.root_browsing_context_exists() {
            // Create a new context tree entry. This will become the root context.
            let new_context = BrowsingContext::new(&window, frame_element, incomplete.pipeline_id);
            // We have a new root frame tree.
            self.browsing_context.set(Some(&new_context));
            (new_context, ContextToRemove::Root)
        } else if let Some((parent, _, _)) = incomplete.parent_info {
            // Create a new context tree entry. This will be a child context.
            let new_context = BrowsingContext::new(&window, frame_element, incomplete.pipeline_id);

            let root_context = self.root_browsing_context();
            // TODO(gw): This find will fail when we are sharing script threads
            // between cross origin iframes in the same TLD.
            let parent_context = root_context.find(parent)
                                             .expect("received load for child context with missing parent");
            parent_context.push_child_context(&*new_context);
            (new_context, ContextToRemove::Child(incomplete.pipeline_id))
        } else {
            using_new_context = false;
            (self.root_browsing_context(), ContextToRemove::None)
        };

        window.init_browsing_context(&browsing_context);
        let mut context_remover = AutoContextRemover::new(self, context_to_remove);

        let last_modified = metadata.headers.as_ref().and_then(|headers| {
            headers.get().map(|&LastModified(HttpDate(ref tm))| dom_last_modified(tm))
        });

        let content_type = metadata.content_type.as_ref().and_then(|&ContentType(ref mimetype)| {
            match *mimetype {
                Mime(TopLevel::Application, SubLevel::Xml, _) |
                Mime(TopLevel::Application, SubLevel::Ext(_), _) |
                Mime(TopLevel::Text, SubLevel::Xml, _) |
                Mime(TopLevel::Text, SubLevel::Plain, _) => Some(DOMString::from(mimetype.to_string())),
                _ => None,
            }
        });

        let loader = DocumentLoader::new_with_threads(self.resource_threads.clone(),
                                                      Some(browsing_context.pipeline()),
                                                      Some(incomplete.url.clone()));

        let is_html_document = match metadata.content_type {
            Some(ContentType(Mime(TopLevel::Application, SubLevel::Xml, _))) |
            Some(ContentType(Mime(TopLevel::Text, SubLevel::Xml, _))) =>
                IsHTMLDocument::NonHTMLDocument,
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
            })
        } else {
            None
        };

        let document = Document::new(window.r(),
                                     Some(&browsing_context),
                                     Some(final_url.clone()),
                                     is_html_document,
                                     content_type,
                                     last_modified,
                                     DocumentSource::FromParser,
                                     loader,
                                     referrer,
                                     referrer_policy);
        if using_new_context {
            browsing_context.init(&document);
        } else {
            browsing_context.push_history(&document);
        }
        document.set_ready_state(DocumentReadyState::Loading);

        self.constellation_chan
            .send(ConstellationMsg::ActivateDocument(incomplete.pipeline_id))
            .unwrap();

        // Notify devtools that a new script global exists.
        self.notify_devtools(document.Title(), final_url.clone(), (browsing_context.pipeline(), None));

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
                window.evaluate_js_on_global_with_result(&script_source, jsval.handle_mut());
                let strval = DOMString::from_jsval(self.get_cx(),
                                                   jsval.handle(),
                                                   StringificationBehavior::Empty);
                strval.unwrap_or(DOMString::new())
            }
        } else {
            DOMString::new()
        };

        document.set_https_state(metadata.https_state);

        let is_xml = match metadata.content_type {
            Some(ContentType(Mime(TopLevel::Application, SubLevel::Ext(ref sub_level), _)))
                if sub_level.ends_with("+xml") => true,

            Some(ContentType(Mime(TopLevel::Application, SubLevel::Xml, _))) |
            Some(ContentType(Mime(TopLevel::Text, SubLevel::Xml, _))) => true,

            _ => false,
        };

        if is_xml {
            parse_xml(document.r(),
                      parse_input,
                      final_url,
                      xml::ParseContext::Owner(Some(incomplete.pipeline_id)));
        } else {
            parse_html(document.r(),
                       parse_input,
                       final_url,
                       ParseContext::Owner(Some(incomplete.pipeline_id)));
        }

        if incomplete.is_frozen {
            window.freeze();
        }

        if !incomplete.is_visible {
            self.alter_resource_utilization(browsing_context.pipeline(), false);
        }

        context_remover.neuter();

        document.get_current_parser().unwrap()
    }

    fn notify_devtools(&self, title: DOMString, url: Url, ids: (PipelineId, Option<WorkerId>)) {
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

    fn scroll_fragment_point(&self, pipeline_id: PipelineId, element: &Element) {
        // FIXME(#8275, pcwalton): This is pretty bogus when multiple layers are involved.
        // Really what needs to happen is that this needs to go through layout to ask which
        // layer the element belongs to, and have it send the scroll message to the
        // compositor.
        let rect = element.upcast::<Node>().bounding_content_box();

        // In order to align with element edges, we snap to unscaled pixel boundaries, since the
        // paint thread currently does the same for drawing elements. This is important for pages
        // that require pixel perfect scroll positioning for proper display (like Acid2). Since we
        // don't have the device pixel ratio here, this might not be accurate, but should work as
        // long as the ratio is a whole number. Once #8275 is fixed this should actually take into
        // account the real device pixel ratio.
        let point = Point2D::new(rect.origin.x.to_nearest_px() as f32,
                                 rect.origin.y.to_nearest_px() as f32);

        let message = ConstellationMsg::ScrollFragmentPoint(pipeline_id,
                                                            LayerId::null(),
                                                            point,
                                                            false);
        self.constellation_chan.send(message).unwrap();
    }

    /// Reflows non-incrementally, rebuilding the entire layout tree in the process.
    fn rebuild_and_force_reflow(&self, context: &BrowsingContext, reason: ReflowReason) {
        let document = context.active_document();
        document.dirty_all_nodes();
        let window = window_from_node(document.r());
        window.reflow(ReflowGoal::ForDisplay, ReflowQueryType::NoQuery, reason);
    }

    /// This is the main entry point for receiving and dispatching DOM events.
    ///
    /// TODO: Actually perform DOM event dispatch.
    fn handle_event(&self, pipeline_id: PipelineId, event: CompositorEvent) {
        // DOM events can only be handled if there's a root browsing context.
        if !self.root_browsing_context_exists() {
            return;
        }

        match event {
            ResizeEvent(new_size, size_type) => {
                self.handle_resize_event(pipeline_id, new_size, size_type);
            }

            MouseButtonEvent(event_type, button, point) => {
                self.handle_mouse_event(pipeline_id, event_type, button, point);
            }

            MouseMoveEvent(point) => {
                let document = match self.root_browsing_context().find(pipeline_id) {
                    Some(browsing_context) => browsing_context.active_document(),
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
                                           .get_attribute(&ns!(), &atom!("href"))
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
                let handled = self.handle_touch_event(pipeline_id, event_type, identifier, point);
                match event_type {
                    TouchEventType::Down => {
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
                let document = match self.root_browsing_context().find(pipeline_id) {
                    Some(browsing_context) => browsing_context.active_document(),
                    None => return warn!("Message sent to closed pipeline {}.", pipeline_id),
                };
                document.r().handle_touchpad_pressure_event(self.js_runtime.rt(), point, pressure, phase);
            }

            KeyEvent(ch, key, state, modifiers) => {
                let document = match self.root_browsing_context().find(pipeline_id) {
                    Some(browsing_context) => browsing_context.active_document(),
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
        let document = match self.root_browsing_context().find(pipeline_id) {
            Some(browsing_context) => browsing_context.active_document(),
            None => return warn!("Message sent to closed pipeline {}.", pipeline_id),
        };
        document.handle_mouse_event(self.js_runtime.rt(), button, point, mouse_event_type);
    }

    fn handle_touch_event(&self,
                          pipeline_id: PipelineId,
                          event_type: TouchEventType,
                          identifier: TouchId,
                          point: Point2D<f32>)
                          -> bool {
        let document = match self.root_browsing_context().find(pipeline_id) {
            Some(browsing_context) => browsing_context.active_document(),
            None => { warn!("Message sent to closed pipeline {}.", pipeline_id); return true },
        };
        document.handle_touch_event(self.js_runtime.rt(), event_type, identifier, point)
    }

    /// https://html.spec.whatwg.org/multipage/#navigating-across-documents
    /// The entry point for content to notify that a new load has been requested
    /// for the given pipeline (specifically the "navigate" algorithm).
    fn handle_navigate(&self, pipeline_id: PipelineId, subpage_id: Option<SubpageId>, load_data: LoadData) {
        // Step 8.
        {
            let nurl = &load_data.url;
            if let Some(fragment) = nurl.fragment() {
                let document = match self.root_browsing_context().find(pipeline_id) {
                    Some(browsing_context) => browsing_context.active_document(),
                    None => return warn!("Message sent to closed pipeline {}.", pipeline_id),
                };
                let url = document.url();
                if &url[..Position::AfterQuery] == &nurl[..Position::AfterQuery] &&
                    load_data.method == Method::Get {
                    match document.find_fragment_node(fragment) {
                        Some(ref node) => {
                            self.scroll_fragment_point(pipeline_id, node.r());
                        }
                        None => {}
                    }
                    return;
                }
            }
        }

        match subpage_id {
            Some(subpage_id) => {
                let root_context = self.root_browsing_context();
                let iframe = root_context.find(pipeline_id).and_then(|context| {
                    let doc = context.active_document();
                    doc.find_iframe(subpage_id)
                });
                if let Some(iframe) = iframe.r() {
                    iframe.navigate_or_reload_child_browsing_context(Some(load_data));
                }
            }
            None => {
                self.constellation_chan
                    .send(ConstellationMsg::LoadUrl(pipeline_id, load_data))
                    .unwrap();
            }
        }
    }

    fn handle_resize_event(&self, pipeline_id: PipelineId, new_size: WindowSizeData, size_type: WindowSizeType) {
        let context = match self.root_browsing_context().find(pipeline_id) {
            Some(browsing_context) => browsing_context,
            None => return warn!("Message sent to closed pipeline {}.", pipeline_id),
        };
        let window = context.active_window();
        window.set_window_size(new_size);
        window.force_reflow(ReflowGoal::ForDisplay,
                            ReflowQueryType::NoQuery,
                            ReflowReason::WindowResize);

        let document = context.active_document();
        let fragment_node = window.steal_fragment_name()
                                  .and_then(|name| document.find_fragment_node(&*name));
        match fragment_node {
            Some(ref node) => self.scroll_fragment_point(pipeline_id, node.r()),
            None => {}
        }

        // http://dev.w3.org/csswg/cssom-view/#resizing-viewports
        if size_type == WindowSizeType::Resize {
            let uievent = UIEvent::new(window.r(),
                                       DOMString::from("resize"), EventBubbles::DoesNotBubble,
                                       EventCancelable::NotCancelable, Some(window.r()),
                                       0i32);
            uievent.upcast::<Event>().fire(window.upcast());
        }
    }

    /// Initiate a non-blocking fetch for a specified resource. Stores the InProgressLoad
    /// argument until a notification is received that the fetch is complete.
    fn start_page_load(&self, incomplete: InProgressLoad, mut load_data: LoadData) {
        let id = incomplete.pipeline_id.clone();
        let subpage = incomplete.parent_info.clone().map(|p| p.1);

        let context = Arc::new(Mutex::new(ParserContext::new(id, subpage, load_data.url.clone())));
        let (action_sender, action_receiver) = ipc::channel().unwrap();
        let listener = NetworkListener {
            context: context,
            script_chan: self.chan.clone(),
            wrapper: None,
        };
        ROUTER.add_route(action_receiver.to_opaque(), box move |message| {
            listener.notify_action(message.to().unwrap());
        });
        let response_target = AsyncResponseTarget {
            sender: action_sender,
        };

        if load_data.url.scheme() == "javascript" {
            load_data.url = Url::parse("about:blank").unwrap();
        }

        self.resource_threads.send(CoreResourceMsg::Load(NetLoadData {
            context: LoadContext::Browsing,
            url: load_data.url,
            method: load_data.method,
            headers: Headers::new(),
            preserved_headers: load_data.headers,
            data: load_data.data,
            cors: None,
            pipeline_id: Some(id),
            credentials_flag: true,
            referrer_policy: load_data.referrer_policy,
            referrer_url: load_data.referrer_url
        }, LoadConsumer::Listener(response_target), None)).unwrap();

        self.incomplete_loads.borrow_mut().push(incomplete);
    }

    fn handle_parsing_complete(&self, id: PipelineId) {
        let parent_context = self.root_browsing_context();
        let context = match parent_context.find(id) {
            Some(context) => context,
            None => return,
        };

        let document = context.active_document();
        let final_url = document.url();

        // https://html.spec.whatwg.org/multipage/#the-end step 1
        document.set_ready_state(DocumentReadyState::Interactive);

        // TODO: Execute step 2 here.

        // Kick off the initial reflow of the page.
        debug!("kicking off initial reflow of {:?}", final_url);
        document.disarm_reflow_timeout();
        document.upcast::<Node>().dirty(NodeDamage::OtherNodeDamage);
        let window = window_from_node(document.r());
        window.reflow(ReflowGoal::ForDisplay, ReflowQueryType::NoQuery, ReflowReason::FirstLoad);

        // No more reflow required
        context.set_reflow_status(false);

        // https://html.spec.whatwg.org/multipage/#the-end steps 3-4.
        document.process_deferred_scripts();

        window.set_fragment_name(final_url.fragment().map(str::to_owned));
    }

    fn handle_css_error_reporting(&self, pipeline_id: PipelineId, filename: String,
                                  line: usize, column: usize, msg: String) {
        let sender = match self.devtools_chan {
            Some(ref sender) => sender,
            None => return,
        };

        let parent_context = self.root_browsing_context();
        let context = match parent_context.find(pipeline_id) {
            Some(context) => context,
            None => return,
        };

        let window = context.active_window();
        if window.live_devtools_updates() {
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

    fn handle_reload(&self, pipeline_id: PipelineId) {
        if let Some(context) = self.find_child_context(pipeline_id) {
            let win = context.active_window();
            let location = win.Location();
            location.Reload();
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

/// Shuts down layout for the given browsing context tree.
fn shut_down_layout(context_tree: &BrowsingContext) {
    let mut channels = vec!();

    for context in context_tree.iter() {
        // Tell the layout thread to begin shutting down, and wait until it
        // processed this message.
        let (response_chan, response_port) = channel();
        let window = context.active_window();
        let chan = window.layout_chan().clone();
        if chan.send(message::Msg::PrepareToExit(response_chan)).is_ok() {
            channels.push(chan);
            response_port.recv().unwrap();
        }
    }

    // Drop our references to the JSContext and DOM objects.
    for context in context_tree.iter() {
        let window = context.active_window();
        window.clear_js_runtime();

        // Sever the connection between the global and the DOM tree
        context.clear_session_history();
    }

    // Destroy the layout thread. If there were node leaks, layout will now crash safely.
    for chan in channels {
        chan.send(message::Msg::ExitNow).ok();
    }
}

// TODO: remove this function, as it's a source of panic.
pub fn get_browsing_context(context: &BrowsingContext,
                            pipeline_id: PipelineId)
                            -> Root<BrowsingContext> {
    context.find(pipeline_id).expect("ScriptThread: received an event \
            message for a layout channel that is not associated with this script thread.\
            This is a bug.")
}

fn dom_last_modified(tm: &Tm) -> String {
    tm.to_local().strftime("%m/%d/%Y %H:%M:%S").unwrap().to_string()
}
