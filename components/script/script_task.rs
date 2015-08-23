/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

//! The script task is the task that owns the DOM in memory, runs JavaScript, and spawns parsing
//! and layout tasks. It's in charge of processing events for all same-origin pages in a frame
//! tree, and manages the entire lifetime of pages in the frame tree from initial request to
//! teardown.
//!
//! Page loads follow a two-step process. When a request for a new page load is received, the
//! network request is initiated and the relevant data pertaining to the new page is stashed.
//! While the non-blocking request is ongoing, the script task is free to process further events,
//! noting when they pertain to ongoing loads (such as resizes/viewport adjustments). When the
//! initial response is received for an ongoing load, the second phase starts - the frame tree
//! entry is created, along with the Window and Document objects, and the appropriate parser
//! takes over the response body. Once parsing is complete, the document lifecycle for loading
//! a page runs its course and the script task returns to processing events in the main event
//! loop.

#![allow(unsafe_code)]

use devtools;
use document_loader::{LoadType, DocumentLoader, NotifierData};
use dom::bindings::cell::DOMRefCell;
use dom::bindings::codegen::Bindings::AttrBinding::AttrMethods;
use dom::bindings::codegen::Bindings::DocumentBinding::{DocumentMethods, DocumentReadyState};
use dom::bindings::codegen::InheritTypes::{ElementCast, EventTargetCast, NodeCast, EventCast};
use dom::bindings::conversions::FromJSValConvertible;
use dom::bindings::conversions::StringificationBehavior;
use dom::bindings::global::GlobalRef;
use dom::bindings::js::{JS, RootCollection, trace_roots};
use dom::bindings::js::{RootCollectionPtr, Root, RootedReference};
use dom::bindings::refcounted::{LiveDOMReferences, Trusted, TrustedReference, trace_refcounted_objects};
use dom::bindings::trace::{JSTraceable, trace_traceables, RootedVec};
use dom::bindings::utils::{WRAP_CALLBACKS, DOM_CALLBACKS};
use dom::document::{Document, IsHTMLDocument, DocumentProgressHandler};
use dom::document::{DocumentProgressTask, DocumentSource, MouseEventType};
use dom::element::Element;
use dom::event::{EventBubbles, EventCancelable};
use dom::node::{Node, NodeDamage, window_from_node};
use dom::servohtmlparser::{ServoHTMLParser, ParserContext};
use dom::uievent::UIEvent;
use dom::window::{Window, ScriptHelpers, ReflowReason};
use dom::worker::TrustedWorkerAddress;
use layout_interface::{ReflowQueryType};
use layout_interface::{self, NewLayoutTaskInfo, ScriptLayoutChan, LayoutChan, ReflowGoal};
use mem::heap_size_of_eventtarget;
use network_listener::NetworkListener;
use page::{Page, IterablePage, Frame};
use parse::html::{ParseContext, parse_html};
use timers::TimerId;
use webdriver_handlers;

use devtools_traits::{DevtoolsPageInfo, DevtoolScriptControlMsg};
use devtools_traits::{ScriptToDevtoolsControlMsg, TimelineMarker, TimelineMarkerType};
use devtools_traits::{TracingMetadata};
use msg::compositor_msg::{LayerId, ScriptToCompositorMsg};
use msg::constellation_msg::Msg as ConstellationMsg;
use msg::constellation_msg::{ConstellationChan, FocusType};
use msg::constellation_msg::{Failure, WindowSizeData, PipelineExitType};
use msg::constellation_msg::{LoadData, PipelineId, SubpageId, MozBrowserEvent, WorkerId};
use msg::webdriver_msg::WebDriverScriptCommand;
use net_traits::LoadData as NetLoadData;
use net_traits::image_cache_task::{ImageCacheChan, ImageCacheTask, ImageCacheResult};
use net_traits::storage_task::StorageTask;
use net_traits::{AsyncResponseTarget, ResourceTask, LoadConsumer, ControlMsg, Metadata};
use profile_traits::mem::{self, Report, ReportKind, ReportsChan, OpaqueSender};
use script_traits::CompositorEvent::{MouseDownEvent, MouseUpEvent};
use script_traits::CompositorEvent::{MouseMoveEvent, KeyEvent};
use script_traits::CompositorEvent::{ResizeEvent, ClickEvent};
use script_traits::ConstellationControlMsg;
use script_traits::{CompositorEvent, MouseButton};
use script_traits::{NewLayoutInfo, OpaqueScriptLayoutChannel};
use script_traits::{ScriptState, ScriptTaskFactory};
use string_cache::Atom;
use util::opts;
use util::str::DOMString;
use util::task::spawn_named_with_send_on_failure;
use util::task_state;

use euclid::Rect;
use euclid::point::Point2D;
use hyper::header::{LastModified, Headers};
use hyper::method::Method;
use ipc_channel::ipc::{self, IpcSender};
use ipc_channel::router::ROUTER;
use js::glue::CollectServoSizes;
use js::jsapi::{JSContext, JSRuntime, JSTracer};
use js::jsapi::{JSGCInvocationKind, GCDescription, SetGCSliceCallback, GCProgress};
use js::jsapi::{JS_GetRuntime, JS_SetGCCallback, JSGCStatus, JSAutoRequest, SetDOMCallbacks};
use js::jsapi::{JS_SetWrapObjectCallbacks, JS_AddExtraGCRootsTracer, DisableIncrementalGC};
use js::jsapi::{SetDOMProxyInformation, DOMProxyShadowsResult, HandleObject, HandleId, RootedValue};
use js::jsval::UndefinedValue;
use js::rust::Runtime;
use url::{Url, UrlParser};

use core::ops::Deref;
use libc;
use std::any::Any;
use std::borrow::ToOwned;
use std::cell::{Cell, RefCell};
use std::collections::{HashMap, HashSet};
use std::io::{stdout, Write};
use std::mem as std_mem;
use std::option::Option;
use std::ptr;
use std::rc::Rc;
use std::result::Result;
use std::sync::mpsc::{channel, Sender, Receiver, Select};
use std::sync::{Arc, Mutex};
use time::{self, Tm};

use hyper::header::{ContentType, HttpDate};
use hyper::mime::{Mime, TopLevel, SubLevel};

thread_local!(pub static STACK_ROOTS: Cell<Option<RootCollectionPtr>> = Cell::new(None));
thread_local!(static SCRIPT_TASK_ROOT: RefCell<Option<*const ScriptTask>> = RefCell::new(None));

unsafe extern fn trace_rust_roots(tr: *mut JSTracer, _data: *mut libc::c_void) {
    SCRIPT_TASK_ROOT.with(|root| {
        if let Some(script_task) = *root.borrow() {
            (*script_task).trace(tr);
        }
    });

    trace_traceables(tr);
    trace_roots(tr);
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
    parent_info: Option<(PipelineId, SubpageId)>,
    /// The current window size associated with this pipeline.
    window_size: Option<WindowSizeData>,
    /// Channel to the layout task associated with this pipeline.
    layout_chan: LayoutChan,
    /// The current viewport clipping rectangle applying to this pipeline, if any.
    clip_rect: Option<Rect<f32>>,
    /// The requested URL of the load.
    url: Url,
}

impl InProgressLoad {
    /// Create a new InProgressLoad object.
    fn new(id: PipelineId,
           parent_info: Option<(PipelineId, SubpageId)>,
           layout_chan: LayoutChan,
           window_size: Option<WindowSizeData>,
           url: Url) -> InProgressLoad {
        InProgressLoad {
            pipeline_id: id,
            parent_info: parent_info,
            layout_chan: layout_chan,
            window_size: window_size,
            clip_rect: None,
            url: url,
        }
    }
}

#[derive(Copy, Clone)]
pub enum TimerSource {
    FromWindow(PipelineId),
    FromWorker
}

pub trait Runnable {
    fn handler(self: Box<Self>);
}

pub trait MainThreadRunnable {
    fn handler(self: Box<Self>, script_task: &ScriptTask);
}

enum MixedMessage {
    FromConstellation(ConstellationControlMsg),
    FromScript(MainThreadScriptMsg),
    FromDevtools(DevtoolScriptControlMsg),
    FromImageCache(ImageCacheResult),
}

/// Common messages used to control the event loops in both the script and the worker
pub enum CommonScriptMsg {
    /// Requests that the script task measure its memory usage. The results are sent back via the
    /// supplied channel.
    CollectReports(ReportsChan),
    /// Fires a JavaScript timeout
    /// TimerSource must be FromWindow when dispatched to ScriptTask and
    /// must be FromWorker when dispatched to a DedicatedGlobalWorkerScope
    FireTimer(TimerSource, TimerId),
    /// A DOM object's last pinned reference was removed (dispatched to all tasks).
    RefcountCleanup(TrustedReference),
    /// Generic message that encapsulates event handling.
    RunnableMsg(ScriptTaskEventCategory, Box<Runnable + Send>),
}

#[derive(Clone, Copy, Debug, Eq, Hash, JSTraceable, PartialEq)]
pub enum ScriptTaskEventCategory {
    AttachLayout,
    ConstellationMsg,
    DevtoolsMsg,
    DocumentEvent,
    DomEvent,
    FileRead,
    ImageCacheMsg,
    InputEvent,
    NetworkEvent,
    Resize,
    ScriptEvent,
    UpdateReplacedElement,
    SetViewport,
    WebSocketEvent,
    WorkerEvent,
    XhrEvent,
}

/// Messages used to control the script event loop
pub enum MainThreadScriptMsg {
    /// Common variants associated with the script messages
    Common(CommonScriptMsg),
    /// Notify a document that all pending loads are complete.
    DocumentLoadsComplete(PipelineId),
    /// Notifies the script that a window associated with a particular pipeline
    /// should be closed (only dispatched to ScriptTask).
    ExitWindow(PipelineId),
    /// Generic message for running tasks in the ScriptTask
    MainThreadRunnableMsg(Box<MainThreadRunnable + Send>),
    /// Begins a content-initiated load on the specified pipeline (only
    /// dispatched to ScriptTask).
    Navigate(PipelineId, LoadData),
}

/// A cloneable interface for communicating with an event loop.
pub trait ScriptChan {
    /// Send a message to the associated event loop.
    fn send(&self, msg: CommonScriptMsg) -> Result<(), ()>;
    /// Clone this handle.
    fn clone(&self) -> Box<ScriptChan + Send>;
}

impl OpaqueSender<CommonScriptMsg> for Box<ScriptChan + Send> {
    fn send(&self, msg: CommonScriptMsg) {
        ScriptChan::send(&**self, msg).unwrap();
    }
}

/// An interface for receiving ScriptMsg values in an event loop. Used for synchronous DOM
/// APIs that need to abstract over multiple kinds of event loops (worker/main thread) with
/// different Receiver interfaces.
pub trait ScriptPort {
    fn recv(&self) -> CommonScriptMsg;
}

impl ScriptPort for Receiver<CommonScriptMsg> {
    fn recv(&self) -> CommonScriptMsg {
        self.recv().unwrap()
    }
}

impl ScriptPort for Receiver<MainThreadScriptMsg> {
    fn recv(&self) -> CommonScriptMsg {
        match self.recv().unwrap() {
            MainThreadScriptMsg::Common(script_msg) => script_msg,
            _ => panic!("unexpected main thread event message!")
        }
    }
}

impl ScriptPort for Receiver<(TrustedWorkerAddress, CommonScriptMsg)> {
    fn recv(&self) -> CommonScriptMsg {
        self.recv().unwrap().1
    }
}

impl ScriptPort for Receiver<(TrustedWorkerAddress, MainThreadScriptMsg)> {
    fn recv(&self) -> CommonScriptMsg {
        match self.recv().unwrap().1 {
            MainThreadScriptMsg::Common(script_msg) => script_msg,
            _ => panic!("unexpected main thread event message!")
        }
    }
}

/// Encapsulates internal communication of shared messages within the script task.
#[derive(JSTraceable)]
pub struct SendableMainThreadScriptChan(pub Sender<CommonScriptMsg>);

impl ScriptChan for SendableMainThreadScriptChan {
    fn send(&self, msg: CommonScriptMsg) -> Result<(), ()> {
        let SendableMainThreadScriptChan(ref chan) = *self;
        chan.send(msg).map_err(|_| ())
    }

    fn clone(&self) -> Box<ScriptChan + Send> {
        let SendableMainThreadScriptChan(ref chan) = *self;
        box SendableMainThreadScriptChan((*chan).clone())
    }
}

impl SendableMainThreadScriptChan {
    /// Creates a new script chan.
    pub fn new() -> (Receiver<CommonScriptMsg>, Box<SendableMainThreadScriptChan>) {
        let (chan, port) = channel();
        (port, box SendableMainThreadScriptChan(chan))
    }
}

/// Encapsulates internal communication of main thread messages within the script task.
#[derive(JSTraceable)]
pub struct MainThreadScriptChan(pub Sender<MainThreadScriptMsg>);

impl ScriptChan for MainThreadScriptChan {
    fn send(&self, msg: CommonScriptMsg) -> Result<(), ()> {
        let MainThreadScriptChan(ref chan) = *self;
        return chan.send(MainThreadScriptMsg::Common(msg)).map_err(|_| ());
    }

    fn clone(&self) -> Box<ScriptChan + Send> {
        let MainThreadScriptChan(ref chan) = *self;
        box MainThreadScriptChan((*chan).clone())
    }
}

impl MainThreadScriptChan {
    /// Creates a new script chan.
    pub fn new() -> (Receiver<MainThreadScriptMsg>, Box<MainThreadScriptChan>) {
        let (chan, port) = channel();
        (port, box MainThreadScriptChan(chan))
    }
}

pub struct StackRootTLS;

impl StackRootTLS {
    pub fn new(roots: &RootCollection) -> StackRootTLS {
        STACK_ROOTS.with(|ref r| {
            r.set(Some(RootCollectionPtr(roots as *const _)))
        });
        StackRootTLS
    }
}

impl Drop for StackRootTLS {
    fn drop(&mut self) {
        STACK_ROOTS.with(|ref r| r.set(None));
    }
}


/// Information for an entire page. Pages are top-level browsing contexts and can contain multiple
/// frames.
#[derive(JSTraceable)]
// ScriptTask instances are rooted on creation, so this is okay
#[allow(unrooted_must_root)]
pub struct ScriptTask {
    /// A handle to the information pertaining to page layout
    page: DOMRefCell<Option<Rc<Page>>>,
    /// A list of data pertaining to loads that have not yet received a network response
    incomplete_loads: DOMRefCell<Vec<InProgressLoad>>,
    /// A handle to the image cache task.
    image_cache_task: ImageCacheTask,
    /// A handle to the resource task. This is an `Arc` to avoid running out of file descriptors if
    /// there are many iframes.
    resource_task: Arc<ResourceTask>,
    /// A handle to the storage task.
    storage_task: StorageTask,

    /// The port on which the script task receives messages (load URL, exit, etc.)
    port: Receiver<MainThreadScriptMsg>,
    /// A channel to hand out to script task-based entities that need to be able to enqueue
    /// events in the event queue.
    chan: MainThreadScriptChan,

    /// A channel to hand out to tasks that need to respond to a message from the script task.
    control_chan: Sender<ConstellationControlMsg>,

    /// The port on which the constellation and layout tasks can communicate with the
    /// script task.
    control_port: Receiver<ConstellationControlMsg>,

    /// For communicating load url messages to the constellation
    constellation_chan: ConstellationChan,

    /// A handle to the compositor for communicating ready state messages.
    compositor: DOMRefCell<IpcSender<ScriptToCompositorMsg>>,

    /// The port on which we receive messages from the image cache
    image_cache_port: Receiver<ImageCacheResult>,

    /// The channel on which the image cache can send messages to ourself.
    image_cache_channel: ImageCacheChan,

    /// For providing contact with the memory profiler.
    mem_profiler_chan: mem::ProfilerChan,

    /// For providing instructions to an optional devtools server.
    devtools_chan: Option<IpcSender<ScriptToDevtoolsControlMsg>>,
    /// For receiving commands from an optional devtools server. Will be ignored if
    /// no such server exists.
    devtools_port: Receiver<DevtoolScriptControlMsg>,
    devtools_sender: IpcSender<DevtoolScriptControlMsg>,
    /// For sending timeline markers. Will be ignored if
    /// no devtools server
    devtools_markers: RefCell<HashSet<TimelineMarkerType>>,
    devtools_marker_sender: RefCell<Option<IpcSender<TimelineMarker>>>,

    /// The JavaScript runtime.
    js_runtime: Rc<Runtime>,

    mouse_over_targets: DOMRefCell<Vec<JS<Node>>>,

    /// List of pipelines that have been owned and closed by this script task.
    closed_pipelines: RefCell<HashSet<PipelineId>>,

    /// When profiling data should be written out to stdout.
    perf_profiler_next_report: Cell<Option<u64>>,
    /// How much time was spent on what since the last report.
    perf_profiler_times: RefCell<HashMap<ScriptTaskEventCategory, u64>>,
}

/// In the event of task failure, all data on the stack runs its destructor. However, there
/// are no reachable, owning pointers to the DOM memory, so it never gets freed by default
/// when the script task fails. The ScriptMemoryFailsafe uses the destructor bomb pattern
/// to forcibly tear down the JS compartments for pages associated with the failing ScriptTask.
struct ScriptMemoryFailsafe<'a> {
    owner: Option<&'a ScriptTask>,
}

impl<'a> ScriptMemoryFailsafe<'a> {
    fn neuter(&mut self) {
        self.owner = None;
    }

    fn new(owner: &'a ScriptTask) -> ScriptMemoryFailsafe<'a> {
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
                unsafe {
                    let page = owner.page.borrow_for_script_deallocation();
                    for page in page.iter() {
                        let window = page.window();
                        window.r().clear_js_runtime_for_script_deallocation();
                    }
                }
            }
            None => (),
        }
    }
}

impl ScriptTaskFactory for ScriptTask {
    fn create_layout_channel(_phantom: Option<&mut ScriptTask>) -> OpaqueScriptLayoutChannel {
        let (chan, port) = channel();
        ScriptLayoutChan::new(chan, port)
    }

    fn clone_layout_channel(_phantom: Option<&mut ScriptTask>, pair: &OpaqueScriptLayoutChannel) -> Box<Any + Send> {
        box pair.sender() as Box<Any + Send>
    }

    fn create(_phantom: Option<&mut ScriptTask>,
              id: PipelineId,
              parent_info: Option<(PipelineId, SubpageId)>,
              compositor: IpcSender<ScriptToCompositorMsg>,
              layout_chan: &OpaqueScriptLayoutChannel,
              control_chan: Sender<ConstellationControlMsg>,
              control_port: Receiver<ConstellationControlMsg>,
              constellation_chan: ConstellationChan,
              failure_msg: Failure,
              resource_task: ResourceTask,
              storage_task: StorageTask,
              image_cache_task: ImageCacheTask,
              mem_profiler_chan: mem::ProfilerChan,
              devtools_chan: Option<IpcSender<ScriptToDevtoolsControlMsg>>,
              window_size: Option<WindowSizeData>,
              load_data: LoadData) {
        let ConstellationChan(const_chan) = constellation_chan.clone();
        let (script_chan, script_port) = channel();
        let layout_chan = LayoutChan(layout_chan.sender());
        spawn_named_with_send_on_failure(format!("ScriptTask {:?}", id), task_state::SCRIPT, move || {
            let roots = RootCollection::new();
            let _stack_roots_tls = StackRootTLS::new(&roots);
            let chan = MainThreadScriptChan(script_chan);
            let channel_for_reporter = chan.clone();
            let script_task = ScriptTask::new(compositor,
                                              script_port,
                                              chan,
                                              control_chan,
                                              control_port,
                                              constellation_chan,
                                              Arc::new(resource_task),
                                              storage_task,
                                              image_cache_task,
                                              mem_profiler_chan.clone(),
                                              devtools_chan);

            SCRIPT_TASK_ROOT.with(|root| {
                *root.borrow_mut() = Some(&script_task as *const _);
            });

            let mut failsafe = ScriptMemoryFailsafe::new(&script_task);

            let new_load = InProgressLoad::new(id, parent_info, layout_chan, window_size,
                                               load_data.url.clone());
            script_task.start_page_load(new_load, load_data);

            let reporter_name = format!("script-reporter-{}", id.0);
            mem_profiler_chan.run_with_memory_reporting(|| {
                script_task.start();
            }, reporter_name, channel_for_reporter, CommonScriptMsg::CollectReports);

            // This must always be the very last operation performed before the task completes
            failsafe.neuter();
        }, ConstellationMsg::Failure(failure_msg), const_chan);
    }
}

thread_local!(static GC_CYCLE_START: Cell<Option<Tm>> = Cell::new(None));
thread_local!(static GC_SLICE_START: Cell<Option<Tm>> = Cell::new(None));

unsafe extern "C" fn gc_slice_callback(_rt: *mut JSRuntime, progress: GCProgress, desc: *const GCDescription) {
    match progress {
        GCProgress::GC_CYCLE_BEGIN => {
            GC_CYCLE_START.with(|start| {
                start.set(Some(time::now()));
                println!("GC cycle began");
            })
        },
        GCProgress::GC_SLICE_BEGIN => {
            GC_SLICE_START.with(|start| {
                start.set(Some(time::now()));
                println!("GC slice began");
            })
        },
        GCProgress::GC_SLICE_END => {
            GC_SLICE_START.with(|start| {
                let dur = time::now() - start.get().unwrap();
                start.set(None);
                println!("GC slice ended: duration={}", dur);
            })
        },
        GCProgress::GC_CYCLE_END => {
            GC_CYCLE_START.with(|start| {
                let dur = time::now() - start.get().unwrap();
                start.set(None);
                println!("GC cycle ended: duration={}", dur);
            })
        },
    };
    if !desc.is_null() {
        let desc: &GCDescription = &*desc;
        let invocationKind = match desc.invocationKind_ {
            JSGCInvocationKind::GC_NORMAL => "GC_NORMAL",
            JSGCInvocationKind::GC_SHRINK => "GC_SHRINK",
        };
        println!("  isCompartment={}, invocationKind={}", desc.isCompartment_, invocationKind);
    }
    let _ = stdout().flush();
}

unsafe extern "C" fn debug_gc_callback(_rt: *mut JSRuntime, status: JSGCStatus, _data: *mut libc::c_void) {
    match status {
        JSGCStatus::JSGC_BEGIN => task_state::enter(task_state::IN_GC),
        JSGCStatus::JSGC_END   => task_state::exit(task_state::IN_GC),
    }
}

unsafe extern "C" fn shadow_check_callback(_cx: *mut JSContext,
    _object: HandleObject, _id: HandleId) -> DOMProxyShadowsResult {
    // XXX implement me
    DOMProxyShadowsResult::ShadowCheckFailed
}

impl ScriptTask {
    pub fn page_fetch_complete(id: PipelineId, subpage: Option<SubpageId>, metadata: Metadata)
                               -> Option<Root<ServoHTMLParser>> {
        SCRIPT_TASK_ROOT.with(|root| {
            let script_task = unsafe { &*root.borrow().unwrap() };
            script_task.handle_page_fetch_complete(id, subpage, metadata)
        })
    }

    pub fn parsing_complete(id: PipelineId) {
        SCRIPT_TASK_ROOT.with(|root| {
            let script_task = unsafe { &*root.borrow().unwrap() };
            script_task.handle_parsing_complete(id);
        });
    }

    pub fn process_event(msg: CommonScriptMsg) {
        SCRIPT_TASK_ROOT.with(|root| {
            if let Some(script_task) = *root.borrow() {
                let script_task = unsafe { &*script_task };
                script_task.handle_msg_from_script(MainThreadScriptMsg::Common(msg));
            }
        });
    }

    /// Creates a new script task.
    pub fn new(compositor: IpcSender<ScriptToCompositorMsg>,
               port: Receiver<MainThreadScriptMsg>,
               chan: MainThreadScriptChan,
               control_chan: Sender<ConstellationControlMsg>,
               control_port: Receiver<ConstellationControlMsg>,
               constellation_chan: ConstellationChan,
               resource_task: Arc<ResourceTask>,
               storage_task: StorageTask,
               image_cache_task: ImageCacheTask,
               mem_profiler_chan: mem::ProfilerChan,
               devtools_chan: Option<IpcSender<ScriptToDevtoolsControlMsg>>)
               -> ScriptTask {
        let runtime = ScriptTask::new_rt_and_cx();

        unsafe {
            JS_SetWrapObjectCallbacks(runtime.rt(),
                                      &WRAP_CALLBACKS);
        }

        // Ask the router to proxy IPC messages from the devtools to us.
        let (ipc_devtools_sender, ipc_devtools_receiver) = ipc::channel().unwrap();
        let devtools_port = ROUTER.route_ipc_receiver_to_new_mpsc_receiver(ipc_devtools_receiver);

        // Ask the router to proxy IPC messages from the image cache task to us.
        let (ipc_image_cache_channel, ipc_image_cache_port) = ipc::channel().unwrap();
        let image_cache_port =
            ROUTER.route_ipc_receiver_to_new_mpsc_receiver(ipc_image_cache_port);

        ScriptTask {
            page: DOMRefCell::new(None),
            incomplete_loads: DOMRefCell::new(vec!()),

            image_cache_task: image_cache_task,
            image_cache_channel: ImageCacheChan(ipc_image_cache_channel),
            image_cache_port: image_cache_port,

            resource_task: resource_task,
            storage_task: storage_task,

            port: port,
            chan: chan,
            control_chan: control_chan,
            control_port: control_port,
            constellation_chan: constellation_chan,
            compositor: DOMRefCell::new(compositor),
            mem_profiler_chan: mem_profiler_chan,

            devtools_chan: devtools_chan,
            devtools_port: devtools_port,
            devtools_sender: ipc_devtools_sender,
            devtools_markers: RefCell::new(HashSet::new()),
            devtools_marker_sender: RefCell::new(None),

            js_runtime: Rc::new(runtime),
            mouse_over_targets: DOMRefCell::new(vec!()),
            closed_pipelines: RefCell::new(HashSet::new()),

            perf_profiler_next_report: Cell::new(None),
            perf_profiler_times: RefCell::new(HashMap::new()),
        }
    }

    pub fn new_rt_and_cx() -> Runtime {
        LiveDOMReferences::initialize();
        let runtime = Runtime::new();

        unsafe {
            JS_AddExtraGCRootsTracer(runtime.rt(), Some(trace_rust_roots), ptr::null_mut());
            JS_AddExtraGCRootsTracer(runtime.rt(), Some(trace_refcounted_objects), ptr::null_mut());
        }

        // Needed for debug assertions about whether GC is running.
        if cfg!(debug_assertions) {
            unsafe {
                JS_SetGCCallback(runtime.rt(), Some(debug_gc_callback), ptr::null_mut());
            }
        }
        if opts::get().gc_profile {
            unsafe {
                SetGCSliceCallback(runtime.rt(), Some(gc_slice_callback));
            }
        }

        unsafe {
            SetDOMProxyInformation(ptr::null(), 0, Some(shadow_check_callback));
            SetDOMCallbacks(runtime.rt(), &DOM_CALLBACKS);
            // Pre barriers aren't working correctly at the moment
            DisableIncrementalGC(runtime.rt());
        }

        runtime
    }

    // Return the root page in the frame tree. Panics if it doesn't exist.
    pub fn root_page(&self) -> Rc<Page> {
        self.page.borrow().as_ref().unwrap().clone()
    }

    pub fn get_cx(&self) -> *mut JSContext {
        self.js_runtime.cx()
    }

    /// Starts the script task. After calling this method, the script task will loop receiving
    /// messages on its port.
    pub fn start(&self) {
        while self.handle_msgs() {
            // Go on...
        }
    }

    /// Handle incoming control messages.
    fn handle_msgs(&self) -> bool {
        // Handle pending resize events.
        // Gather them first to avoid a double mut borrow on self.
        let mut resizes = vec!();

        {
            let page = self.page.borrow();
            if let Some(page) = page.as_ref() {
                for page in page.iter() {
                    // Only process a resize if layout is idle.
                    let window = page.window();
                    if window.r().layout_is_idle() {
                        let resize_event = window.r().steal_resize_event();
                        match resize_event {
                            Some(size) => resizes.push((window.r().pipeline(), size)),
                            None => ()
                        }
                    }
                }
            }
        }

        for (id, size) in resizes {
            self.handle_event(id, ResizeEvent(size));
        }

        // Store new resizes, and gather all other events.
        let mut sequential = vec!();

        // Receive at least one message so we don't spinloop.
        let mut event = {
            let sel = Select::new();
            let mut port1 = sel.handle(&self.port);
            let mut port2 = sel.handle(&self.control_port);
            let mut port3 = sel.handle(&self.devtools_port);
            let mut port4 = sel.handle(&self.image_cache_port);
            unsafe {
                port1.add();
                port2.add();
                if self.devtools_chan.is_some() {
                    port3.add();
                }
                port4.add();
            }
            let ret = sel.wait();
            if ret == port1.id() {
                MixedMessage::FromScript(self.port.recv().unwrap())
            } else if ret == port2.id() {
                MixedMessage::FromConstellation(self.control_port.recv().unwrap())
            } else if ret == port3.id() {
                MixedMessage::FromDevtools(self.devtools_port.recv().unwrap())
            } else if ret == port4.id() {
                MixedMessage::FromImageCache(self.image_cache_port.recv().unwrap())
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
                MixedMessage::FromConstellation(ConstellationControlMsg::AttachLayout(
                        new_layout_info)) => {
                    self.profile_event(ScriptTaskEventCategory::AttachLayout, || {
                        self.handle_new_layout(new_layout_info);
                    })
                }
                MixedMessage::FromConstellation(ConstellationControlMsg::Resize(id, size)) => {
                    self.profile_event(ScriptTaskEventCategory::Resize, || {
                        self.handle_resize(id, size);
                    })
                }
                MixedMessage::FromConstellation(ConstellationControlMsg::Viewport(id, rect)) => {
                    self.profile_event(ScriptTaskEventCategory::SetViewport, || {
                        self.handle_viewport(id, rect);
                    })
                }
                MixedMessage::FromConstellation(ConstellationControlMsg::TickAllAnimations(
                        pipeline_id)) => {
                    if !animation_ticks.contains(&pipeline_id) {
                        animation_ticks.insert(pipeline_id);
                        sequential.push(event);
                    }
                }
                MixedMessage::FromConstellation(ConstellationControlMsg::SendEvent(
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
                    Err(_) => match self.devtools_port.try_recv() {
                        Err(_) => match self.image_cache_port.try_recv() {
                            Err(_) => break,
                            Ok(ev) => event = MixedMessage::FromImageCache(ev),
                        },
                        Ok(ev) => event = MixedMessage::FromDevtools(ev),
                    },
                    Ok(ev) => event = MixedMessage::FromScript(ev),
                },
                Ok(ev) => event = MixedMessage::FromConstellation(ev),
            }
        }

        // Process the gathered events.
        for msg in sequential {
            let category = self.categorize_msg(&msg);

            let result = self.profile_event(category, move || {
                match msg {
                    MixedMessage::FromConstellation(ConstellationControlMsg::ExitPipeline(id, exit_type)) => {
                        if self.handle_exit_pipeline_msg(id, exit_type) {
                            return Some(false)
                        }
                    },
                    MixedMessage::FromConstellation(inner_msg) => self.handle_msg_from_constellation(inner_msg),
                    MixedMessage::FromScript(inner_msg) => self.handle_msg_from_script(inner_msg),
                    MixedMessage::FromDevtools(inner_msg) => self.handle_msg_from_devtools(inner_msg),
                    MixedMessage::FromImageCache(inner_msg) => self.handle_msg_from_image_cache(inner_msg),
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
        let page = self.page.borrow();
        if let Some(page) = page.as_ref() {
            for page in page.iter() {
                let window = page.window();
                let pending_reflows = window.r().get_pending_reflow_count();
                if pending_reflows > 0 {
                    window.r().reflow(ReflowGoal::ForDisplay,
                                      ReflowQueryType::NoQuery,
                                      ReflowReason::ImageLoaded);
                }
            }
        }

        true
    }

    fn categorize_msg(&self, msg: &MixedMessage) -> ScriptTaskEventCategory {
        match *msg {
            MixedMessage::FromConstellation(ref inner_msg) => {
                match *inner_msg {
                    ConstellationControlMsg::SendEvent(_, _) =>
                        ScriptTaskEventCategory::DomEvent,
                    _ => ScriptTaskEventCategory::ConstellationMsg
                }
            },
            MixedMessage::FromDevtools(_) => ScriptTaskEventCategory::DevtoolsMsg,
            MixedMessage::FromImageCache(_) => ScriptTaskEventCategory::ImageCacheMsg,
            MixedMessage::FromScript(ref inner_msg) => {
                match *inner_msg {
                    MainThreadScriptMsg::Common(CommonScriptMsg::RunnableMsg(ref category, _)) =>
                        *category,
                    _ => ScriptTaskEventCategory::ScriptEvent
                }
            }
        }
    }

    fn profile_event<F, R>(&self, category: ScriptTaskEventCategory, f: F) -> R
        where F: FnOnce() -> R {

        if opts::get().profile_script_events {
            let start = time::precise_time_ns();
            let result = f();
            let end = time::precise_time_ns();

            let duration = end - start;

            let aggregate = {
                let zero = 0;
                let perf_profiler_times = self.perf_profiler_times.borrow();
                let so_far = perf_profiler_times.get(&category).unwrap_or(&zero);

                so_far + duration
            };

            self.perf_profiler_times.borrow_mut().insert(category, aggregate);

            const NANO: u64 = 1000 * 1000 * 1000;
            const REPORT_INTERVAL: u64 = 10 * NANO;

            match self.perf_profiler_next_report.get() {
                None => self.perf_profiler_next_report.set(Some(start + REPORT_INTERVAL)),
                Some(time) if time <= end => {
                    self.perf_profiler_next_report.set(Some(end + REPORT_INTERVAL));

                    let stdout = stdout();
                    let mut stdout = stdout.lock();
                    writeln!(&mut stdout, "Script task time distribution:").unwrap();
                    for (c, t) in self.perf_profiler_times.borrow().deref() {
                        let secs = t / NANO;
                        let nanos = t % NANO;
                        writeln!(&mut stdout, "  {:?}: {}.{}s", c, secs, nanos).unwrap();
                    }
                    stdout.flush().unwrap();

                    self.perf_profiler_times.borrow_mut().clear();
                },
                Some(_) => {}
            }

            result
        } else {
            f()
        }
    }

    fn handle_msg_from_constellation(&self, msg: ConstellationControlMsg) {
        match msg {
            ConstellationControlMsg::AttachLayout(_) =>
                panic!("should have handled AttachLayout already"),
            ConstellationControlMsg::Navigate(pipeline_id, subpage_id, load_data) =>
                self.handle_navigate(pipeline_id, Some(subpage_id), load_data),
            ConstellationControlMsg::SendEvent(id, event) =>
                self.handle_event(id, event),
            ConstellationControlMsg::ReflowComplete(id, reflow_id) =>
                self.handle_reflow_complete_msg(id, reflow_id),
            ConstellationControlMsg::ResizeInactive(id, new_size) =>
                self.handle_resize_inactive_msg(id, new_size),
            ConstellationControlMsg::Viewport(..) =>
                panic!("should have handled Viewport already"),
            ConstellationControlMsg::Resize(..) =>
                panic!("should have handled Resize already"),
            ConstellationControlMsg::ExitPipeline(..) =>
                panic!("should have handled ExitPipeline already"),
            ConstellationControlMsg::GetTitle(pipeline_id) =>
                self.handle_get_title_msg(pipeline_id),
            ConstellationControlMsg::Freeze(pipeline_id) =>
                self.handle_freeze_msg(pipeline_id),
            ConstellationControlMsg::Thaw(pipeline_id) =>
                self.handle_thaw_msg(pipeline_id),
            ConstellationControlMsg::MozBrowserEvent(parent_pipeline_id,
                                                     subpage_id,
                                                     event) =>
                self.handle_mozbrowser_event_msg(parent_pipeline_id,
                                                 subpage_id,
                                                 event),
            ConstellationControlMsg::UpdateSubpageId(containing_pipeline_id,
                                                     old_subpage_id,
                                                     new_subpage_id) =>
                self.handle_update_subpage_id(containing_pipeline_id, old_subpage_id, new_subpage_id),
            ConstellationControlMsg::FocusIFrame(containing_pipeline_id, subpage_id) =>
                self.handle_focus_iframe_msg(containing_pipeline_id, subpage_id),
            ConstellationControlMsg::WebDriverScriptCommand(pipeline_id, msg) =>
                self.handle_webdriver_msg(pipeline_id, msg),
            ConstellationControlMsg::TickAllAnimations(pipeline_id) =>
                self.handle_tick_all_animations(pipeline_id),
            ConstellationControlMsg::StylesheetLoadComplete(id, url, responder) => {
                responder.respond();
                self.handle_resource_loaded(id, LoadType::Stylesheet(url));
            }
            ConstellationControlMsg::GetCurrentState(sender, pipeline_id) => {
                let state = self.handle_get_current_state(pipeline_id);
                sender.send(state).unwrap();
            }
        }
    }

    fn handle_msg_from_script(&self, msg: MainThreadScriptMsg) {
        match msg {
            MainThreadScriptMsg::Navigate(id, load_data) =>
                self.handle_navigate(id, None, load_data),
            MainThreadScriptMsg::ExitWindow(id) =>
                self.handle_exit_window_msg(id),
            MainThreadScriptMsg::MainThreadRunnableMsg(runnable) =>
                runnable.handler(self),
            MainThreadScriptMsg::DocumentLoadsComplete(id) =>
                self.handle_loads_complete(id),
            MainThreadScriptMsg::Common(
                CommonScriptMsg::FireTimer(TimerSource::FromWindow(id), timer_id)) =>
                self.handle_fire_timer_msg(id, timer_id),
            MainThreadScriptMsg::Common(
                CommonScriptMsg::FireTimer(TimerSource::FromWorker, _)) =>
                panic!("Worker timeouts must not be sent to script task"),
            MainThreadScriptMsg::Common(CommonScriptMsg::RunnableMsg(_, runnable)) =>
                // The category of the runnable is ignored by the pattern, however
                // it is still respected by profiling (see categorize_msg).
                runnable.handler(),
            MainThreadScriptMsg::Common(CommonScriptMsg::RefcountCleanup(addr)) =>
                LiveDOMReferences::cleanup(addr),
            MainThreadScriptMsg::Common(CommonScriptMsg::CollectReports(reports_chan)) =>
                self.collect_reports(reports_chan),
        }
    }

    fn handle_msg_from_devtools(&self, msg: DevtoolScriptControlMsg) {
        let page = self.root_page();
        match msg {
            DevtoolScriptControlMsg::EvaluateJS(id, s, reply) => {
                let window = get_page(&page, id).window();
                let global_ref = GlobalRef::Window(window.r());
                devtools::handle_evaluate_js(&global_ref, s, reply)
            },
            DevtoolScriptControlMsg::GetRootNode(id, reply) =>
                devtools::handle_get_root_node(&page, id, reply),
            DevtoolScriptControlMsg::GetDocumentElement(id, reply) =>
                devtools::handle_get_document_element(&page, id, reply),
            DevtoolScriptControlMsg::GetChildren(id, node_id, reply) =>
                devtools::handle_get_children(&page, id, node_id, reply),
            DevtoolScriptControlMsg::GetLayout(id, node_id, reply) =>
                devtools::handle_get_layout(&page, id, node_id, reply),
            DevtoolScriptControlMsg::GetCachedMessages(pipeline_id, message_types, reply) =>
                devtools::handle_get_cached_messages(pipeline_id, message_types, reply),
            DevtoolScriptControlMsg::ModifyAttribute(id, node_id, modifications) =>
                devtools::handle_modify_attribute(&page, id, node_id, modifications),
            DevtoolScriptControlMsg::WantsLiveNotifications(id, to_send) => {
                let window = get_page(&page, id).window();
                let global_ref = GlobalRef::Window(window.r());
                devtools::handle_wants_live_notifications(&global_ref, to_send)
            },
            DevtoolScriptControlMsg::SetTimelineMarkers(_pipeline_id, marker_types, reply) =>
                devtools::handle_set_timeline_markers(&page, self, marker_types, reply),
            DevtoolScriptControlMsg::DropTimelineMarkers(_pipeline_id, marker_types) =>
                devtools::handle_drop_timeline_markers(&page, self, marker_types),
            DevtoolScriptControlMsg::RequestAnimationFrame(pipeline_id, name) =>
                devtools::handle_request_animation_frame(&page, pipeline_id, name),
        }
    }

    fn handle_msg_from_image_cache(&self, msg: ImageCacheResult) {
        msg.responder.unwrap().respond(msg.image_response);
    }

    fn handle_webdriver_msg(&self, pipeline_id: PipelineId, msg: WebDriverScriptCommand) {
        let page = self.root_page();
        match msg {
            WebDriverScriptCommand::ExecuteScript(script, reply) =>
                webdriver_handlers::handle_execute_script(&page, pipeline_id, script, reply),
            WebDriverScriptCommand::FindElementCSS(selector, reply) =>
                webdriver_handlers::handle_find_element_css(&page, pipeline_id, selector, reply),
            WebDriverScriptCommand::FindElementsCSS(selector, reply) =>
                webdriver_handlers::handle_find_elements_css(&page, pipeline_id, selector, reply),
            WebDriverScriptCommand::GetActiveElement(reply) =>
                webdriver_handlers::handle_get_active_element(&page, pipeline_id, reply),
            WebDriverScriptCommand::GetElementTagName(node_id, reply) =>
                webdriver_handlers::handle_get_name(&page, pipeline_id, node_id, reply),
            WebDriverScriptCommand::GetElementText(node_id, reply) =>
                webdriver_handlers::handle_get_text(&page, pipeline_id, node_id, reply),
            WebDriverScriptCommand::GetFrameId(frame_id, reply) =>
                webdriver_handlers::handle_get_frame_id(&page, pipeline_id, frame_id, reply),
            WebDriverScriptCommand::GetUrl(reply) =>
                webdriver_handlers::handle_get_url(&page, pipeline_id, reply),
            WebDriverScriptCommand::GetTitle(reply) =>
                webdriver_handlers::handle_get_title(&page, pipeline_id, reply),
            WebDriverScriptCommand::ExecuteAsyncScript(script, reply) =>
                webdriver_handlers::handle_execute_async_script(&page, pipeline_id, script, reply),
        }
    }

    fn handle_resize(&self, id: PipelineId, size: WindowSizeData) {
        let page = self.page.borrow();
        if let Some(ref page) = page.as_ref() {
            if let Some(ref page) = page.find(id) {
                let window = page.window();
                window.r().set_resize_event(size);
                return;
            }
        }
        let mut loads = self.incomplete_loads.borrow_mut();
        if let Some(ref mut load) = loads.iter_mut().find(|load| load.pipeline_id == id) {
            load.window_size = Some(size);
            return;
        }
        panic!("resize sent to nonexistent pipeline");
    }

    fn handle_viewport(&self, id: PipelineId, rect: Rect<f32>) {
        let page = self.page.borrow();
        if let Some(page) = page.as_ref() {
            if let Some(ref inner_page) = page.find(id) {
                let window = inner_page.window();
                if window.r().set_page_clip_rect_with_new_viewport(rect) {
                    let page = get_page(page, id);
                    self.rebuild_and_force_reflow(&*page, ReflowReason::Viewport);
                }
                return;
            }
        }
        let mut loads = self.incomplete_loads.borrow_mut();
        if let Some(ref mut load) = loads.iter_mut().find(|load| load.pipeline_id == id) {
            load.clip_rect = Some(rect);
            return;
        }
        panic!("Page rect message sent to nonexistent pipeline");
    }

    /// Handle a request to load a page in a new child frame of an existing page.
    fn handle_resource_loaded(&self, pipeline: PipelineId, load: LoadType) {
        let page = get_page(&self.root_page(), pipeline);
        let doc = page.document();
        doc.r().finish_load(load);
    }

    /// Get the current state of a given pipeline.
    fn handle_get_current_state(&self, pipeline_id: PipelineId) -> ScriptState {
        // Check if the main page load is still pending
        let loads = self.incomplete_loads.borrow();
        if let Some(_) = loads.iter().find(|load| load.pipeline_id == pipeline_id) {
            return ScriptState::DocumentLoading;
        }

        // If not in pending loads, the page should exist by now.
        let page = self.root_page();
        let page = page.find(pipeline_id).expect("GetCurrentState sent to nonexistent pipeline");
        let doc = page.document();

        // Check if document load event has fired. If the document load
        // event has fired, this also guarantees that the first reflow
        // has been kicked off. Since the script task does a join with
        // layout, this ensures there are no race conditions that can occur
        // between load completing and the first layout completing.
        let load_pending = doc.r().ReadyState() != DocumentReadyState::Complete;
        if load_pending {
            return ScriptState::DocumentLoading;
        }

        // Checks if the html element has reftest-wait attribute present.
        // See http://testthewebforward.org/docs/reftests.html
        let html_element = doc.r().GetDocumentElement();
        let reftest_wait = html_element.r().map_or(false, |elem| elem.has_class(&Atom::from_slice("reftest-wait")));
        if reftest_wait {
            return ScriptState::DocumentLoading;
        }

        ScriptState::DocumentLoaded
    }

    fn handle_new_layout(&self, new_layout_info: NewLayoutInfo) {
        let NewLayoutInfo {
            containing_pipeline_id,
            new_pipeline_id,
            subpage_id,
            load_data,
            paint_chan,
            failure,
            pipeline_port,
            layout_shutdown_chan,
        } = new_layout_info;

        let layout_pair = ScriptTask::create_layout_channel(None::<&mut ScriptTask>);
        let layout_chan = LayoutChan(*ScriptTask::clone_layout_channel(
            None::<&mut ScriptTask>,
            &layout_pair).downcast::<Sender<layout_interface::Msg>>().unwrap());

        let layout_creation_info = NewLayoutTaskInfo {
            id: new_pipeline_id,
            url: load_data.url.clone(),
            is_parent: false,
            layout_pair: layout_pair,
            pipeline_port: pipeline_port,
            constellation_chan: self.constellation_chan.clone(),
            failure: failure,
            paint_chan: paint_chan,
            script_chan: self.control_chan.clone(),
            image_cache_task: self.image_cache_task.clone(),
            layout_shutdown_chan: layout_shutdown_chan,
        };

        let page = self.root_page();
        let parent_page = page.find(containing_pipeline_id).expect("ScriptTask: received a layout
            whose parent has a PipelineId which does not correspond to a pipeline in the script
            task's page tree. This is a bug.");
        let parent_window = parent_page.window();

        // Tell layout to actually spawn the task.
        parent_window.layout_chan()
                     .0
                     .send(layout_interface::Msg::CreateLayoutTask(layout_creation_info))
                     .unwrap();

        // Kick off the fetch for the new resource.
        let new_load = InProgressLoad::new(new_pipeline_id, Some((containing_pipeline_id, subpage_id)),
                                           layout_chan, parent_window.r().window_size(),
                                           load_data.url.clone());
        self.start_page_load(new_load, load_data);
    }

    fn handle_loads_complete(&self, pipeline: PipelineId) {
        let page = get_page(&self.root_page(), pipeline);
        let doc = page.document();
        let doc = doc.r();
        if doc.loader().is_blocked() {
            return;
        }

        doc.mut_loader().inhibit_events();

        // https://html.spec.whatwg.org/multipage/#the-end step 7
        let addr: Trusted<Document> = Trusted::new(self.get_cx(), doc, self.chan.clone());
        let handler = box DocumentProgressHandler::new(addr.clone(), DocumentProgressTask::Load);
        self.chan.send(CommonScriptMsg::RunnableMsg(ScriptTaskEventCategory::DocumentEvent, handler)).unwrap();

        let ConstellationChan(ref chan) = self.constellation_chan;
        chan.send(ConstellationMsg::LoadComplete(pipeline)).unwrap();
    }

    pub fn get_reports(cx: *mut JSContext, path_seg: String) -> Vec<Report> {
        let mut reports = vec![];

        unsafe {
            let rt = JS_GetRuntime(cx);
            let mut stats = ::std::mem::zeroed();
            if CollectServoSizes(rt, &mut stats) {
                let mut report = |mut path_suffix, kind, size| {
                    let mut path = path![path_seg, "js"];
                    path.append(&mut path_suffix);
                    reports.push(Report {
                        path: path,
                        kind: kind,
                        size: size as usize,
                    })
                };

                // A note about possibly confusing terminology: the JS GC "heap" is allocated via
                // mmap/VirtualAlloc, which means it's not on the malloc "heap", so we use
                // `ExplicitNonHeapSize` as its kind.

                report(path!["gc-heap", "used"],
                       ReportKind::ExplicitNonHeapSize,
                       stats.gcHeapUsed);

                report(path!["gc-heap", "unused"],
                       ReportKind::ExplicitNonHeapSize,
                       stats.gcHeapUnused);

                report(path!["gc-heap", "admin"],
                       ReportKind::ExplicitNonHeapSize,
                       stats.gcHeapAdmin);

                report(path!["gc-heap", "decommitted"],
                       ReportKind::ExplicitNonHeapSize,
                       stats.gcHeapDecommitted);

                // SpiderMonkey uses the system heap, not jemalloc.
                report(path!["malloc-heap"],
                       ReportKind::ExplicitSystemHeapSize,
                       stats.mallocHeap);

                report(path!["non-heap"],
                       ReportKind::ExplicitNonHeapSize,
                       stats.nonHeap);
            }
        }
        reports
    }

    fn collect_reports(&self, reports_chan: ReportsChan) {
        let mut urls = vec![];
        let mut dom_tree_size = 0;
        let mut reports = vec![];

        if let Some(root_page) = self.page.borrow().as_ref() {
            for it_page in root_page.iter() {
                let current_url = it_page.document().url().serialize();
                urls.push(current_url.clone());

                for child in NodeCast::from_ref(&*it_page.document()).traverse_preorder() {
                    let target = EventTargetCast::from_ref(&*child);
                    dom_tree_size += heap_size_of_eventtarget(target);
                }
                let window = it_page.window();
                let target = EventTargetCast::from_ref(&*window);
                dom_tree_size += heap_size_of_eventtarget(target);

                reports.push(Report {
                    path: path![format!("url({})", current_url), "dom-tree"],
                    kind: ReportKind::ExplicitJemallocHeapSize,
                    size: dom_tree_size,
                })
            }
        }
        let path_seg = format!("url({})", urls.join(", "));
        reports.extend(ScriptTask::get_reports(self.get_cx(), path_seg));
        reports_chan.send(reports);
    }

    /// Handles a timer that fired.
    fn handle_fire_timer_msg(&self, id: PipelineId, timer_id: TimerId) {
        let page = self.root_page();
        let page = page.find(id).expect("ScriptTask: received fire timer msg for a
            pipeline ID not associated with this script task. This is a bug.");
        let window = page.window();
        window.r().handle_fire_timer(timer_id);
    }

    /// Handles freeze message
    fn handle_freeze_msg(&self, id: PipelineId) {
        let page = self.root_page();
        let page = page.find(id).expect("ScriptTask: received freeze msg for a
                    pipeline ID not associated with this script task. This is a bug.");
        let window = page.window();
        window.r().freeze();
    }

    /// Handles thaw message
    fn handle_thaw_msg(&self, id: PipelineId) {
        // We should only get this message when moving in history, so all pages requested
        // should exist.
        let page = self.root_page().find(id).unwrap();

        let needed_reflow = page.set_reflow_status(false);
        if needed_reflow {
            self.rebuild_and_force_reflow(&*page, ReflowReason::CachedPageNeededReflow);
        }

        let window = page.window();
        window.r().thaw();
    }

    fn handle_focus_iframe_msg(&self,
                               parent_pipeline_id: PipelineId,
                               subpage_id: SubpageId) {
        let borrowed_page = self.root_page();
        let page = borrowed_page.find(parent_pipeline_id).unwrap();

        let doc = page.document();
        let frame_element = doc.find_iframe(subpage_id);

        if let Some(ref frame_element) = frame_element {
            let element = ElementCast::from_ref(frame_element.r());
            doc.r().begin_focus_transaction();
            doc.r().request_focus(element);
            doc.r().commit_focus_transaction(FocusType::Parent);
        }
    }

    /// Handles a mozbrowser event, for example see:
    /// https://developer.mozilla.org/en-US/docs/Web/Events/mozbrowserloadstart
    fn handle_mozbrowser_event_msg(&self,
                                   parent_pipeline_id: PipelineId,
                                   subpage_id: SubpageId,
                                   event: MozBrowserEvent) {
        let borrowed_page = self.root_page();

        let frame_element = borrowed_page.find(parent_pipeline_id).and_then(|page| {
            let doc = page.document();
            doc.find_iframe(subpage_id)
        });

        if let Some(ref frame_element) = frame_element {
            frame_element.r().dispatch_mozbrowser_event(event);
        }
    }

    fn handle_update_subpage_id(&self,
                                containing_pipeline_id: PipelineId,
                                old_subpage_id: SubpageId,
                                new_subpage_id: SubpageId) {
        let borrowed_page = self.root_page();

        let frame_element = borrowed_page.find(containing_pipeline_id).and_then(|page| {
            let doc = page.document();
            doc.find_iframe(old_subpage_id)
        });

        frame_element.r().unwrap().update_subpage_id(new_subpage_id);
    }

    /// Handles a notification that reflow completed.
    fn handle_reflow_complete_msg(&self, pipeline_id: PipelineId, reflow_id: u32) {
        debug!("Script: Reflow {:?} complete for {:?}", reflow_id, pipeline_id);
        let page = self.root_page();
        match page.find(pipeline_id) {
            Some(page) => {
                let window = page.window();
                window.r().handle_reflow_complete_msg(reflow_id);
            }
            None => {
                assert!(self.closed_pipelines.borrow().contains(&pipeline_id));
            }
        }
    }

    /// Window was resized, but this script was not active, so don't reflow yet
    fn handle_resize_inactive_msg(&self, id: PipelineId, new_size: WindowSizeData) {
        let page = self.root_page();
        let page = page.find(id).expect("Received resize message for PipelineId not associated
            with a page in the page tree. This is a bug.");
        let window = page.window();
        window.r().set_window_size(new_size);
        page.set_reflow_status(true);
    }

    /// We have gotten a window.close from script, which we pass on to the compositor.
    /// We do not shut down the script task now, because the compositor will ask the
    /// constellation to shut down the pipeline, which will clean everything up
    /// normally. If we do exit, we will tear down the DOM nodes, possibly at a point
    /// where layout is still accessing them.
    fn handle_exit_window_msg(&self, _: PipelineId) {
        debug!("script task handling exit window msg");

        // TODO(tkuehn): currently there is only one window,
        // so this can afford to be naive and just shut down the
        // compositor. In the future it'll need to be smarter.
        self.compositor.borrow_mut().send(ScriptToCompositorMsg::Exit).unwrap();
    }

    /// We have received notification that the response associated with a load has completed.
    /// Kick off the document and frame tree creation process using the result.
    fn handle_page_fetch_complete(&self, id: PipelineId, subpage: Option<SubpageId>,
                                  metadata: Metadata) -> Option<Root<ServoHTMLParser>> {
        let idx = self.incomplete_loads.borrow().iter().position(|load| {
            load.pipeline_id == id && load.parent_info.map(|info| info.1) == subpage
        });
        // The matching in progress load structure may not exist if
        // the pipeline exited before the page load completed.
        match idx {
            Some(idx) => {
                let load = self.incomplete_loads.borrow_mut().remove(idx);
                Some(self.load(metadata, load))
            }
            None => {
                assert!(self.closed_pipelines.borrow().contains(&id));
                None
            }
        }
    }

    /// Handles a request for the window title.
    fn handle_get_title_msg(&self, pipeline_id: PipelineId) {
        let page = get_page(&self.root_page(), pipeline_id);
        let document = page.document();
        document.r().send_title_to_compositor();
    }

    /// Handles a request to exit the script task and shut down layout.
    /// Returns true if the script task should shut down and false otherwise.
    fn handle_exit_pipeline_msg(&self, id: PipelineId, exit_type: PipelineExitType) -> bool {
        self.closed_pipelines.borrow_mut().insert(id);

        // Check if the exit message is for an in progress load.
        let idx = self.incomplete_loads.borrow().iter().position(|load| {
            load.pipeline_id == id
        });

        if let Some(idx) = idx {
            let load = self.incomplete_loads.borrow_mut().remove(idx);

            // Tell the layout task to begin shutting down, and wait until it
            // processed this message.
            let (response_chan, response_port) = channel();
            let LayoutChan(chan) = load.layout_chan;
            if chan.send(layout_interface::Msg::PrepareToExit(response_chan)).is_ok() {
                debug!("shutting down layout for page {:?}", id);
                response_port.recv().unwrap();
                chan.send(layout_interface::Msg::ExitNow(exit_type)).ok();
            }

            let has_pending_loads = self.incomplete_loads.borrow().len() > 0;
            let has_root_page = self.page.borrow().is_some();

            // Exit if no pending loads and no root page
            return !has_pending_loads && !has_root_page;
        }

        // If root is being exited, shut down all pages
        let page = self.root_page();
        let window = page.window();
        if window.r().pipeline() == id {
            debug!("shutting down layout for root page {:?}", id);
            shut_down_layout(&page, exit_type);
            return true
        }

        // otherwise find just the matching page and exit all sub-pages
        if let Some(ref mut child_page) = page.remove(id) {
            shut_down_layout(&*child_page, exit_type);
        }
        false
    }

    /// Handles when layout task finishes all animation in one tick
    fn handle_tick_all_animations(&self, id: PipelineId) {
        let page = get_page(&self.root_page(), id);
        let document = page.document();
        document.r().run_the_animation_frame_callbacks();
    }

    /// The entry point to document loading. Defines bindings, sets up the window and document
    /// objects, parses HTML and CSS, and kicks off initial layout.
    fn load(&self, metadata: Metadata, incomplete: InProgressLoad) -> Root<ServoHTMLParser> {
        let final_url = metadata.final_url.clone();
        debug!("ScriptTask: loading {} on page {:?}", incomplete.url.serialize(), incomplete.pipeline_id);

        // We should either be initializing a root page or loading a child page of an
        // existing one.
        let root_page_exists = self.page.borrow().is_some();

        let frame_element = incomplete.parent_info.and_then(|(parent_id, subpage_id)| {
            // The root page may not exist yet, if the parent of this frame
            // exists in a different script task.
            let borrowed_page = self.page.borrow();

            // In the case a parent id exists but the matching page
            // cannot be found, this means the page exists in a different
            // script task (due to origin) so it shouldn't be returned.
            // TODO: window.parent will continue to return self in that
            // case, which is wrong. We should be returning an object that
            // denies access to most properties (per
            // https://github.com/servo/servo/issues/3939#issuecomment-62287025).
            borrowed_page.as_ref().and_then(|borrowed_page| {
                borrowed_page.find(parent_id).and_then(|page| {
                    let doc = page.document();
                    doc.find_iframe(subpage_id)
                })
            })
        });

        // Create a new frame tree entry.
        let page = Rc::new(Page::new(incomplete.pipeline_id));
        if !root_page_exists {
            // We have a new root frame tree.
            *self.page.borrow_mut() = Some(page.clone());
        } else if let Some((parent, _)) = incomplete.parent_info {
            // We have a new child frame.
            let parent_page = self.root_page();
            // TODO(gw): This find will fail when we are sharing script tasks
            // between cross origin iframes in the same TLD.
            parent_page.find(parent).expect("received load for subpage with missing parent");
            parent_page.children.borrow_mut().push(page.clone());
        }

        enum PageToRemove {
            Root,
            Child(PipelineId),
        }
        struct AutoPageRemover<'a> {
            page: PageToRemove,
            script_task: &'a ScriptTask,
            neutered: bool,
        }
        impl<'a> AutoPageRemover<'a> {
            fn new(script_task: &'a ScriptTask, page: PageToRemove) -> AutoPageRemover<'a> {
                AutoPageRemover {
                    page: page,
                    script_task: script_task,
                    neutered: false,
                }
            }

            fn neuter(&mut self) {
                self.neutered = true;
            }
        }
        impl<'a> Drop for AutoPageRemover<'a> {
            fn drop(&mut self) {
                if !self.neutered {
                    match self.page {
                        PageToRemove::Root => *self.script_task.page.borrow_mut() = None,
                        PageToRemove::Child(id) => {
                            self.script_task.root_page().remove(id).unwrap();
                        }
                    }
                }
            }
        }

        let page_to_remove = if !root_page_exists {
            PageToRemove::Root
        } else {
            PageToRemove::Child(incomplete.pipeline_id)
        };
        let mut page_remover = AutoPageRemover::new(self, page_to_remove);
        let MainThreadScriptChan(ref sender) = self.chan;

        // Create the window and document objects.
        let window = Window::new(self.js_runtime.clone(),
                                 page.clone(),
                                 MainThreadScriptChan(sender.clone()),
                                 self.image_cache_channel.clone(),
                                 self.control_chan.clone(),
                                 self.compositor.borrow_mut().clone(),
                                 self.image_cache_task.clone(),
                                 self.resource_task.clone(),
                                 self.storage_task.clone(),
                                 self.mem_profiler_chan.clone(),
                                 self.devtools_chan.clone(),
                                 self.constellation_chan.clone(),
                                 incomplete.layout_chan,
                                 incomplete.pipeline_id,
                                 incomplete.parent_info,
                                 incomplete.window_size);

        let last_modified: Option<DOMString> = metadata.headers.as_ref().and_then(|headers| {
            headers.get().map(|&LastModified(HttpDate(ref tm))| dom_last_modified(tm))
        });

        let content_type = match metadata.content_type {
            Some(ContentType(Mime(TopLevel::Text, SubLevel::Plain, _))) => {
                Some("text/plain".to_owned())
            }
            _ => None
        };

        let notifier_data =  {
            let MainThreadScriptChan(ref sender) = self.chan;
            NotifierData {
                script_chan: sender.clone(),
                pipeline: page.pipeline(),
            }
        };
        let loader = DocumentLoader::new_with_task(self.resource_task.clone(),
                                                   Some(notifier_data),
                                                   Some(incomplete.url.clone()));
        let document = Document::new(window.r(),
                                     Some(final_url.clone()),
                                     IsHTMLDocument::HTMLDocument,
                                     content_type,
                                     last_modified,
                                     DocumentSource::FromParser,
                                     loader);

        let frame_element = frame_element.r().map(ElementCast::from_ref);
        window.r().init_browsing_context(document.r(), frame_element);

        // Create the root frame
        page.set_frame(Some(Frame {
            document: JS::from_rooted(&document),
            window: JS::from_rooted(&window),
        }));

        let is_javascript = incomplete.url.scheme == "javascript";
        let parse_input = if is_javascript {
            let _ar = JSAutoRequest::new(self.get_cx());
            let evalstr = incomplete.url.non_relative_scheme_data().unwrap();
            let mut jsval = RootedValue::new(self.get_cx(), UndefinedValue());
            window.r().evaluate_js_on_global_with_result(evalstr, jsval.handle_mut());
            let strval = FromJSValConvertible::from_jsval(self.get_cx(), jsval.handle(),
                                                          StringificationBehavior::Empty);
            strval.unwrap_or("".to_owned())
        } else {
            "".to_owned()
        };

        parse_html(document.r(), parse_input, &final_url,
                   ParseContext::Owner(Some(incomplete.pipeline_id)));

        page_remover.neuter();

        document.r().get_current_parser().unwrap()
    }

    fn notify_devtools(&self, title: DOMString, url: Url, ids: (PipelineId, Option<WorkerId>)) {
        match self.devtools_chan {
            None => {}
            Some(ref chan) => {
                let page_info = DevtoolsPageInfo {
                    title: title,
                    url: url,
                };
                chan.send(ScriptToDevtoolsControlMsg::NewGlobal(
                            ids,
                            self.devtools_sender.clone(),
                            page_info)).unwrap();
            }
        }
    }

    fn scroll_fragment_point(&self, pipeline_id: PipelineId, node: &Element) {
        let node = NodeCast::from_ref(node);
        let rect = node.get_bounding_content_box();
        let point = Point2D::new(rect.origin.x.to_f32_px(), rect.origin.y.to_f32_px());
        // FIXME(#2003, pcwalton): This is pretty bogus when multiple layers are involved.
        // Really what needs to happen is that this needs to go through layout to ask which
        // layer the element belongs to, and have it send the scroll message to the
        // compositor.
        self.compositor.borrow_mut().send(ScriptToCompositorMsg::ScrollFragmentPoint(
                                                 pipeline_id, LayerId::null(), point)).unwrap();
    }

    /// Reflows non-incrementally, rebuilding the entire layout tree in the process.
    fn rebuild_and_force_reflow(&self, page: &Page, reason: ReflowReason) {
        let document = page.document();
        document.r().dirty_all_nodes();
        let window = window_from_node(document.r());
        window.r().reflow(ReflowGoal::ForDisplay, ReflowQueryType::NoQuery, reason);
    }

    /// This is the main entry point for receiving and dispatching DOM events.
    ///
    /// TODO: Actually perform DOM event dispatch.
    fn handle_event(&self, pipeline_id: PipelineId, event: CompositorEvent) {

        match event {
            ResizeEvent(new_size) => {
                let _marker;
                if self.need_emit_timeline_marker(TimelineMarkerType::DOMEvent) {
                   _marker = AutoDOMEventMarker::new(self);
                }

                self.handle_resize_event(pipeline_id, new_size);
            }

            ClickEvent(button, point) => {
                self.handle_mouse_event(pipeline_id, MouseEventType::Click, button, point);
            }

            MouseDownEvent(button, point) => {
                self.handle_mouse_event(pipeline_id, MouseEventType::MouseDown, button, point);
            }

            MouseUpEvent(button, point) => {
                self.handle_mouse_event(pipeline_id, MouseEventType::MouseUp, button, point);
            }

            MouseMoveEvent(point) => {
                let _marker;
                if self.need_emit_timeline_marker(TimelineMarkerType::DOMEvent) {
                    _marker = AutoDOMEventMarker::new(self);
                }
                let page = get_page(&self.root_page(), pipeline_id);
                let document = page.document();

                let mut prev_mouse_over_targets: RootedVec<JS<Node>> = RootedVec::new();
                for target in &*self.mouse_over_targets.borrow_mut() {
                    prev_mouse_over_targets.push(target.clone());
                }

                // We temporarily steal the list of targets over which the mouse is to pass it to
                // handle_mouse_move_event() in a safe RootedVec container.
                let mut mouse_over_targets = RootedVec::new();
                std_mem::swap(&mut *self.mouse_over_targets.borrow_mut(), &mut *mouse_over_targets);
                document.r().handle_mouse_move_event(self.js_runtime.rt(), point, &mut mouse_over_targets);

                // Notify Constellation about anchors that are no longer mouse over targets.
                for target in &*prev_mouse_over_targets {
                    if !mouse_over_targets.contains(target) {
                        if target.root().r().is_anchor_element() {
                            let event = ConstellationMsg::NodeStatus(None);
                            let ConstellationChan(ref chan) = self.constellation_chan;
                            chan.send(event).unwrap();
                            break;
                        }
                    }
                }

                // Notify Constellation about the topmost anchor mouse over target.
                for target in &*mouse_over_targets {
                    let target = target.root();
                    if target.r().is_anchor_element() {
                        let element = ElementCast::to_ref(target.r()).unwrap();
                        let status = element.get_attribute(&ns!(""), &atom!("href"))
                            .and_then(|href| {
                                let value = href.r().Value();
                                let url = document.r().url();
                                UrlParser::new().base_url(&url).parse(&value).map(|url| url.serialize()).ok()
                            });
                        let event = ConstellationMsg::NodeStatus(status);
                        let ConstellationChan(ref chan) = self.constellation_chan;
                        chan.send(event).unwrap();
                        break;
                    }
                }

                std_mem::swap(&mut *self.mouse_over_targets.borrow_mut(), &mut *mouse_over_targets);
            }

            KeyEvent(key, state, modifiers) => {
                let _marker;
                if self.need_emit_timeline_marker(TimelineMarkerType::DOMEvent) {
                    _marker = AutoDOMEventMarker::new(self);
                }
                let page = get_page(&self.root_page(), pipeline_id);
                let document = page.document();
                document.r().dispatch_key_event(
                    key, state, modifiers, &mut self.compositor.borrow_mut());
            }
        }
    }

    fn handle_mouse_event(&self,
                          pipeline_id: PipelineId,
                          mouse_event_type: MouseEventType,
                          button: MouseButton,
                          point: Point2D<f32>) {
        let _marker;
        if self.need_emit_timeline_marker(TimelineMarkerType::DOMEvent) {
            _marker = AutoDOMEventMarker::new(self);
        }
        let page = get_page(&self.root_page(), pipeline_id);
        let document = page.document();
        document.r().handle_mouse_event(self.js_runtime.rt(), button, point, mouse_event_type);
    }

    /// https://html.spec.whatwg.org/multipage/#navigating-across-documents
    /// The entry point for content to notify that a new load has been requested
    /// for the given pipeline (specifically the "navigate" algorithm).
    fn handle_navigate(&self, pipeline_id: PipelineId, subpage_id: Option<SubpageId>, load_data: LoadData) {
        // Step 8.
        {
            let nurl = &load_data.url;
            if let Some(ref fragment) = nurl.fragment {
                let page = get_page(&self.root_page(), pipeline_id);
                let document = page.document();
                let document = document.r();
                let url = document.url();
                if url.scheme == nurl.scheme && url.scheme_data == nurl.scheme_data &&
                    url.query == nurl.query && load_data.method == Method::Get {
                    match document.find_fragment_node(&*fragment) {
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
                let borrowed_page = self.root_page();
                let iframe = borrowed_page.find(pipeline_id).and_then(|page| {
                    let doc = page.document();
                    doc.find_iframe(subpage_id)
                });
                if let Some(iframe) = iframe.r() {
                    iframe.navigate_child_browsing_context(load_data.url);
                }
            }
            None => {
                let ConstellationChan(ref const_chan) = self.constellation_chan;
                const_chan.send(ConstellationMsg::LoadUrl(pipeline_id, load_data)).unwrap();
            }
        }
    }

    fn handle_resize_event(&self, pipeline_id: PipelineId, new_size: WindowSizeData) {
        let page = get_page(&self.root_page(), pipeline_id);
        let window = page.window();
        window.r().set_window_size(new_size);
        window.r().force_reflow(ReflowGoal::ForDisplay,
                                ReflowQueryType::NoQuery,
                                ReflowReason::WindowResize);

        let document = page.document();
        let fragment_node = window.r().steal_fragment_name()
                                      .and_then(|name| document.r().find_fragment_node(&*name));
        match fragment_node {
            Some(ref node) => self.scroll_fragment_point(pipeline_id, node.r()),
            None => {}
        }

        // http://dev.w3.org/csswg/cssom-view/#resizing-viewports
        // https://dvcs.w3.org/hg/dom3events/raw-file/tip/html/DOM3-Events.html#event-type-resize
        let uievent = UIEvent::new(window.r(),
                                   "resize".to_owned(), EventBubbles::DoesNotBubble,
                                   EventCancelable::NotCancelable, Some(window.r()),
                                   0i32);
        let event = EventCast::from_ref(uievent.r());

        let wintarget = EventTargetCast::from_ref(window.r());
        event.fire(wintarget);
    }

    /// Initiate a non-blocking fetch for a specified resource. Stores the InProgressLoad
    /// argument until a notification is received that the fetch is complete.
    fn start_page_load(&self, incomplete: InProgressLoad, mut load_data: LoadData) {
        let id = incomplete.pipeline_id.clone();
        let subpage = incomplete.parent_info.clone().map(|p| p.1);

        let script_chan = self.chan.clone();
        let resource_task = self.resource_task.clone();

        let context = Arc::new(Mutex::new(ParserContext::new(id, subpage, script_chan.clone(),
                                                             load_data.url.clone())));
        let (action_sender, action_receiver) = ipc::channel().unwrap();
        let listener = box NetworkListener {
            context: context,
            script_chan: script_chan.clone(),
        };
        ROUTER.add_route(action_receiver.to_opaque(), box move |message| {
            listener.notify(message.to().unwrap());
        });
        let response_target = AsyncResponseTarget {
            sender: action_sender,
        };

        if load_data.url.scheme == "javascript" {
            load_data.url = Url::parse("about:blank").unwrap();
        }

        resource_task.send(ControlMsg::Load(NetLoadData {
            url: load_data.url,
            method: load_data.method,
            headers: Headers::new(),
            preserved_headers: load_data.headers,
            data: load_data.data,
            cors: None,
            pipeline_id: Some(id),
        }, LoadConsumer::Listener(response_target))).unwrap();

        self.incomplete_loads.borrow_mut().push(incomplete);
    }

    fn need_emit_timeline_marker(&self, timeline_type: TimelineMarkerType) -> bool {
        self.devtools_markers.borrow().contains(&timeline_type)
    }

    fn emit_timeline_marker(&self, marker: TimelineMarker) {
        let sender = self.devtools_marker_sender.borrow();
        let sender = sender.as_ref().expect("There is no marker sender");
        sender.send(marker).unwrap();
    }

    pub fn set_devtools_timeline_marker(&self,
                                        marker: TimelineMarkerType,
                                        reply: IpcSender<TimelineMarker>) {
        *self.devtools_marker_sender.borrow_mut() = Some(reply);
        self.devtools_markers.borrow_mut().insert(marker);
    }

    pub fn drop_devtools_timeline_markers(&self) {
        self.devtools_markers.borrow_mut().clear();
        *self.devtools_marker_sender.borrow_mut() = None;
    }

    fn handle_parsing_complete(&self, id: PipelineId) {
        let parent_page = self.root_page();
        let page = match parent_page.find(id) {
            Some(page) => page,
            None => return,
        };

        let document = page.document();
        let final_url = document.r().url();

        document.r().set_ready_state(DocumentReadyState::Interactive);

        // Kick off the initial reflow of the page.
        debug!("kicking off initial reflow of {:?}", final_url);
        document.r().disarm_reflow_timeout();
        document.r().content_changed(NodeCast::from_ref(document.r()),
                                     NodeDamage::OtherNodeDamage);
        let window = window_from_node(document.r());
        window.r().reflow(ReflowGoal::ForDisplay, ReflowQueryType::NoQuery, ReflowReason::FirstLoad);

        // No more reflow required
        page.set_reflow_status(false);

        // https://html.spec.whatwg.org/multipage/#the-end step 4
        let addr: Trusted<Document> = Trusted::new(self.get_cx(), document.r(), self.chan.clone());
        let handler = box DocumentProgressHandler::new(addr, DocumentProgressTask::DOMContentLoaded);
        self.chan.send(CommonScriptMsg::RunnableMsg(ScriptTaskEventCategory::DocumentEvent, handler)).unwrap();

        window.r().set_fragment_name(final_url.fragment.clone());

        // Notify devtools that a new script global exists.
        self.notify_devtools(document.r().Title(), final_url, (id, None));
    }
}

impl Drop for ScriptTask {
    fn drop(&mut self) {
        SCRIPT_TASK_ROOT.with(|root| {
            *root.borrow_mut() = None;
        });
    }
}

struct AutoDOMEventMarker<'a> {
    script_task: &'a ScriptTask
}

impl<'a> AutoDOMEventMarker<'a> {
    fn new(script_task: &'a ScriptTask) -> AutoDOMEventMarker<'a> {
        let marker = TimelineMarker::new("DOMEvent".to_owned(), TracingMetadata::IntervalStart);
        script_task.emit_timeline_marker(marker);
        AutoDOMEventMarker {
            script_task: script_task
        }
    }
}

impl<'a> Drop for AutoDOMEventMarker<'a> {
    fn drop(&mut self) {
        let marker = TimelineMarker::new("DOMEvent".to_owned(), TracingMetadata::IntervalEnd);
        self.script_task.emit_timeline_marker(marker);
    }
}

/// Shuts down layout for the given page tree.
fn shut_down_layout(page_tree: &Rc<Page>, exit_type: PipelineExitType) {
    let mut channels = vec!();

    for page in page_tree.iter() {
        // Tell the layout task to begin shutting down, and wait until it
        // processed this message.
        let (response_chan, response_port) = channel();
        let window = page.window();
        let LayoutChan(chan) = window.r().layout_chan();
        if chan.send(layout_interface::Msg::PrepareToExit(response_chan)).is_ok() {
            channels.push(chan);
            response_port.recv().unwrap();
        }
    }

    // Drop our references to the JSContext and DOM objects.
    for page in page_tree.iter() {
        let window = page.window();
        window.r().clear_js_runtime();
        // Sever the connection between the global and the DOM tree
        page.set_frame(None);
    }

    // Destroy the layout task. If there were node leaks, layout will now crash safely.
    for chan in channels {
        chan.send(layout_interface::Msg::ExitNow(exit_type)).ok();
    }
}

pub fn get_page(page: &Rc<Page>, pipeline_id: PipelineId) -> Rc<Page> {
    page.find(pipeline_id).expect("ScriptTask: received an event \
        message for a layout channel that is not associated with this script task.\
         This is a bug.")
}

fn dom_last_modified(tm: &Tm) -> String {
    tm.to_local().strftime("%m/%d/%Y %H:%M:%S").unwrap().to_string()
}
