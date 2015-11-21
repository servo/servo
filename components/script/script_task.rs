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

use devtools;
use devtools_traits::ScriptToDevtoolsControlMsg;
use devtools_traits::{DevtoolScriptControlMsg, DevtoolsPageInfo};
use document_loader::DocumentLoader;
use dom::bindings::cell::DOMRefCell;
use dom::bindings::codegen::Bindings::DocumentBinding::{DocumentMethods, DocumentReadyState};
use dom::bindings::conversions::{FromJSValConvertible, StringificationBehavior};
use dom::bindings::global::GlobalRef;
use dom::bindings::inheritance::Castable;
use dom::bindings::js::{JS, RootCollection, trace_roots};
use dom::bindings::js::{Root, RootCollectionPtr, RootedReference};
use dom::bindings::refcounted::{LiveDOMReferences, Trusted, TrustedReference, trace_refcounted_objects};
use dom::bindings::trace::{JSTraceable, RootedVec, trace_traceables};
use dom::bindings::utils::{DOM_CALLBACKS, WRAP_CALLBACKS};
use dom::document::{Document, DocumentProgressHandler, IsHTMLDocument};
use dom::document::{DocumentSource, MouseEventType};
use dom::element::Element;
use dom::event::{Event, EventBubbles, EventCancelable};
use dom::htmlanchorelement::HTMLAnchorElement;
use dom::node::{Node, NodeDamage, window_from_node};
use dom::servohtmlparser::{ParserContext, ServoHTMLParser};
use dom::uievent::UIEvent;
use dom::window::{ReflowReason, ScriptHelpers, Window};
use dom::worker::TrustedWorkerAddress;
use euclid::Rect;
use euclid::point::Point2D;
use hyper::header::{ContentType, HttpDate};
use hyper::header::{Headers, LastModified};
use hyper::method::Method;
use hyper::mime::{Mime, SubLevel, TopLevel};
use ipc_channel::ipc::{self, IpcSender};
use ipc_channel::router::ROUTER;
use js::glue::CollectServoSizes;
use js::jsapi::{DOMProxyShadowsResult, HandleId, HandleObject, RootedValue};
use js::jsapi::{DisableIncrementalGC, JS_AddExtraGCRootsTracer, JS_SetWrapObjectCallbacks};
use js::jsapi::{GCDescription, GCProgress, JSGCInvocationKind, SetGCSliceCallback};
use js::jsapi::{JSAutoRequest, JSGCStatus, JS_GetRuntime, JS_SetGCCallback, SetDOMCallbacks};
use js::jsapi::{JSContext, JSRuntime, JSTracer};
use js::jsapi::{JSObject, SetPreserveWrapperCallback};
use js::jsval::UndefinedValue;
use js::rust::Runtime;
use layout_interface::{ReflowQueryType};
use layout_interface::{self, LayoutChan, NewLayoutTaskInfo, ReflowGoal, ScriptLayoutChan};
use libc;
use mem::heap_size_of_self_and_children;
use msg::compositor_msg::{EventResult, LayerId, ScriptToCompositorMsg};
use msg::constellation_msg::ScriptMsg as ConstellationMsg;
use msg::constellation_msg::{ConstellationChan, FocusType, LoadData};
use msg::constellation_msg::{MozBrowserEvent, PipelineId};
use msg::constellation_msg::{PipelineNamespace};
use msg::constellation_msg::{SubpageId, WindowSizeData, WorkerId};
use msg::webdriver_msg::WebDriverScriptCommand;
use net_traits::LoadData as NetLoadData;
use net_traits::image_cache_task::{ImageCacheChan, ImageCacheResult, ImageCacheTask};
use net_traits::storage_task::StorageTask;
use net_traits::{AsyncResponseTarget, ControlMsg, LoadConsumer, Metadata, ResourceTask};
use network_listener::NetworkListener;
use page::{Frame, IterablePage, Page};
use parse::html::{ParseContext, parse_html};
use profile_traits::mem::{self, OpaqueSender, Report, ReportKind, ReportsChan};
use profile_traits::time::{self, ProfilerCategory, profile};
use script_traits::CompositorEvent::{ClickEvent, ResizeEvent};
use script_traits::CompositorEvent::{KeyEvent, MouseMoveEvent};
use script_traits::CompositorEvent::{MouseDownEvent, MouseUpEvent, TouchEvent};
use script_traits::{CompositorEvent, ConstellationControlMsg};
use script_traits::{InitialScriptState, MouseButton, NewLayoutInfo};
use script_traits::{OpaqueScriptLayoutChannel, ScriptState, ScriptTaskFactory};
use script_traits::{TimerEvent, TimerEventRequest, TimerSource};
use script_traits::{TouchEventType, TouchId};
use std::any::Any;
use std::borrow::ToOwned;
use std::cell::{Cell, RefCell};
use std::collections::HashSet;
use std::io::{Write, stdout};
use std::marker::PhantomData;
use std::mem as std_mem;
use std::option::Option;
use std::ptr;
use std::rc::Rc;
use std::result::Result;
use std::sync::atomic::{Ordering, AtomicBool};
use std::sync::mpsc::{Receiver, Select, Sender, channel};
use std::sync::{Arc, Mutex};
use string_cache::Atom;
use time::{Tm, now};
use url::{Url, UrlParser};
use util::opts;
use util::str::DOMString;
use util::task;
use util::task_state;
use webdriver_handlers;

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

/// Encapsulated state required to create cancellable runnables from non-script threads.
pub struct RunnableWrapper {
    pub cancelled: Arc<AtomicBool>,
}

impl RunnableWrapper {
    pub fn wrap_runnable<T: Runnable + Send + 'static>(&self, runnable: T) -> Box<Runnable + Send> {
        box CancellableRunnable {
            cancelled: self.cancelled.clone(),
            inner: box runnable,
        }
    }
}

/// A runnable that can be discarded by toggling a shared flag.
pub struct CancellableRunnable<T: Runnable + Send> {
    cancelled: Arc<AtomicBool>,
    inner: Box<T>,
}

impl<T: Runnable + Send> Runnable for CancellableRunnable<T> {
    fn is_cancelled(&self) -> bool {
        self.cancelled.load(Ordering::Relaxed)
    }

    fn handler(self: Box<CancellableRunnable<T>>) {
        self.inner.handler()
    }
}

pub trait Runnable {
    fn is_cancelled(&self) -> bool { false }
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
    FromScheduler(TimerEvent),
}

/// Common messages used to control the event loops in both the script and the worker
pub enum CommonScriptMsg {
    /// Requests that the script task measure its memory usage. The results are sent back via the
    /// supplied channel.
    CollectReports(ReportsChan),
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
    TimerEvent,
    SetViewport,
    StylesheetLoad,
    UpdateReplacedElement,
    WebSocketEvent,
    WorkerEvent,
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
        chan.send(MainThreadScriptMsg::Common(msg)).map_err(|_| ())
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

pub struct StackRootTLS<'a>(PhantomData<&'a u32>);

impl<'a> StackRootTLS<'a> {
    pub fn new(roots: &'a RootCollection) -> StackRootTLS<'a> {
        STACK_ROOTS.with(|ref r| {
            r.set(Some(RootCollectionPtr(roots as *const _)))
        });
        StackRootTLS(PhantomData)
    }
}

impl<'a> Drop for StackRootTLS<'a> {
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
    control_chan: IpcSender<ConstellationControlMsg>,

    /// The port on which the constellation and layout tasks can communicate with the
    /// script task.
    control_port: Receiver<ConstellationControlMsg>,

    /// For communicating load url messages to the constellation
    constellation_chan: ConstellationChan<ConstellationMsg>,

    /// A handle to the compositor for communicating ready state messages.
    compositor: DOMRefCell<IpcSender<ScriptToCompositorMsg>>,

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

    mouse_over_targets: DOMRefCell<Vec<JS<Element>>>,

    /// List of pipelines that have been owned and closed by this script task.
    closed_pipelines: DOMRefCell<HashSet<PipelineId>>,

    scheduler_chan: IpcSender<TimerEventRequest>,
    timer_event_chan: Sender<TimerEvent>,
    timer_event_port: Receiver<TimerEvent>,

    content_process_shutdown_chan: IpcSender<()>,
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
                        window.clear_js_runtime_for_script_deallocation();
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

    fn clone_layout_channel(_phantom: Option<&mut ScriptTask>, pair: &OpaqueScriptLayoutChannel)
                            -> Box<Any + Send> {
        box pair.sender() as Box<Any + Send>
    }

    fn create(_phantom: Option<&mut ScriptTask>,
              state: InitialScriptState,
              layout_chan: &OpaqueScriptLayoutChannel,
              load_data: LoadData) {
        let ConstellationChan(const_chan) = state.constellation_chan.clone();
        let (script_chan, script_port) = channel();
        let layout_chan = LayoutChan(layout_chan.sender());
        let failure_info = state.failure_info;
        task::spawn_named_with_send_on_failure(format!("ScriptTask {:?}", state.id),
                                               task_state::SCRIPT,
                                               move || {
            PipelineNamespace::install(state.pipeline_namespace_id);
            let roots = RootCollection::new();
            let _stack_roots_tls = StackRootTLS::new(&roots);
            let chan = MainThreadScriptChan(script_chan);
            let channel_for_reporter = chan.clone();
            let id = state.id;
            let parent_info = state.parent_info;
            let mem_profiler_chan = state.mem_profiler_chan.clone();
            let window_size = state.window_size;
            let script_task = ScriptTask::new(state,
                                              script_port,
                                              chan);

            SCRIPT_TASK_ROOT.with(|root| {
                *root.borrow_mut() = Some(&script_task as *const _);
            });

            let mut failsafe = ScriptMemoryFailsafe::new(&script_task);

            let new_load = InProgressLoad::new(id, parent_info, layout_chan, window_size,
                                               load_data.url.clone());
            script_task.start_page_load(new_load, load_data);

            let reporter_name = format!("script-reporter-{}", id);
            mem_profiler_chan.run_with_memory_reporting(|| {
                script_task.start();
                let _ = script_task.content_process_shutdown_chan.send(());
            }, reporter_name, channel_for_reporter, CommonScriptMsg::CollectReports);

            // This must always be the very last operation performed before the task completes
            failsafe.neuter();
        }, ConstellationMsg::Failure(failure_info), const_chan);
    }
}

thread_local!(static GC_CYCLE_START: Cell<Option<Tm>> = Cell::new(None));
thread_local!(static GC_SLICE_START: Cell<Option<Tm>> = Cell::new(None));

unsafe extern "C" fn gc_slice_callback(_rt: *mut JSRuntime, progress: GCProgress, desc: *const GCDescription) {
    match progress {
        GCProgress::GC_CYCLE_BEGIN => {
            GC_CYCLE_START.with(|start| {
                start.set(Some(now()));
                println!("GC cycle began");
            })
        },
        GCProgress::GC_SLICE_BEGIN => {
            GC_SLICE_START.with(|start| {
                start.set(Some(now()));
                println!("GC slice began");
            })
        },
        GCProgress::GC_SLICE_END => {
            GC_SLICE_START.with(|start| {
                let dur = now() - start.get().unwrap();
                start.set(None);
                println!("GC slice ended: duration={}", dur);
            })
        },
        GCProgress::GC_CYCLE_END => {
            GC_CYCLE_START.with(|start| {
                let dur = now() - start.get().unwrap();
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

pub unsafe extern "C" fn shadow_check_callback(_cx: *mut JSContext,
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
    pub fn new(state: InitialScriptState,
               port: Receiver<MainThreadScriptMsg>,
               chan: MainThreadScriptChan)
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

        let (timer_event_chan, timer_event_port) = channel();

        // Ask the router to proxy IPC messages from the control port to us.
        let control_port = ROUTER.route_ipc_receiver_to_new_mpsc_receiver(state.control_port);

        ScriptTask {
            page: DOMRefCell::new(None),
            incomplete_loads: DOMRefCell::new(vec!()),

            image_cache_task: state.image_cache_task,
            image_cache_channel: ImageCacheChan(ipc_image_cache_channel),
            image_cache_port: image_cache_port,

            resource_task: Arc::new(state.resource_task),
            storage_task: state.storage_task,

            port: port,
            chan: chan,
            control_chan: state.control_chan,
            control_port: control_port,
            constellation_chan: state.constellation_chan,
            compositor: DOMRefCell::new(state.compositor),
            time_profiler_chan: state.time_profiler_chan,
            mem_profiler_chan: state.mem_profiler_chan,

            devtools_chan: state.devtools_chan,
            devtools_port: devtools_port,
            devtools_sender: ipc_devtools_sender,

            js_runtime: Rc::new(runtime),
            mouse_over_targets: DOMRefCell::new(vec!()),
            closed_pipelines: DOMRefCell::new(HashSet::new()),

            scheduler_chan: state.scheduler_chan,
            timer_event_chan: timer_event_chan,
            timer_event_port: timer_event_port,

            content_process_shutdown_chan: state.content_process_shutdown_chan,
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
            unsafe extern "C" fn empty_wrapper_callback(_: *mut JSContext, _: *mut JSObject) -> bool { true }
            SetDOMCallbacks(runtime.rt(), &DOM_CALLBACKS);
            SetPreserveWrapperCallback(runtime.rt(), Some(empty_wrapper_callback));
            // Pre barriers aren't working correctly at the moment
            DisableIncrementalGC(runtime.rt());
        }

        runtime
    }

    // Return the root page in the frame tree. Panics if it doesn't exist.
    pub fn root_page(&self) -> Rc<Page> {
        self.page.borrow().as_ref().unwrap().clone()
    }

    /// Find a child page of the root page by pipeline id. Returns `None` if the root page does
    /// not exist or the subpage cannot be found.
    fn find_subpage(&self, pipeline_id: PipelineId) -> Option<Rc<Page>> {
        self.page.borrow().as_ref().and_then(|page| page.find(pipeline_id))
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
                    let resize_event = window.steal_resize_event();
                    match resize_event {
                        Some(size) => resizes.push((window.pipeline(), size)),
                        None => ()
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
                MixedMessage::FromScript(self.port.recv().unwrap())
            } else if ret == control_port.id() {
                MixedMessage::FromConstellation(self.control_port.recv().unwrap())
            } else if ret == timer_event_port.id() {
                MixedMessage::FromScheduler(self.timer_event_port.recv().unwrap())
            } else if ret == devtools_port.id() {
                MixedMessage::FromDevtools(self.devtools_port.recv().unwrap())
            } else if ret == image_cache_port.id() {
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
                    Err(_) => match self.timer_event_port.try_recv() {
                        Err(_) => match self.devtools_port.try_recv() {
                            Err(_) => match self.image_cache_port.try_recv() {
                                Err(_) => break,
                                Ok(ev) => event = MixedMessage::FromImageCache(ev),
                            },
                            Ok(ev) => event = MixedMessage::FromDevtools(ev),
                        },
                        Ok(ev) => event = MixedMessage::FromScheduler(ev),
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
                    MixedMessage::FromConstellation(ConstellationControlMsg::ExitPipeline(id)) => {
                        if self.handle_exit_pipeline_msg(id) {
                            return Some(false)
                        }
                    },
                    MixedMessage::FromConstellation(inner_msg) => self.handle_msg_from_constellation(inner_msg),
                    MixedMessage::FromScript(inner_msg) => self.handle_msg_from_script(inner_msg),
                    MixedMessage::FromScheduler(inner_msg) => self.handle_timer_event(inner_msg),
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
                let pending_reflows = window.get_pending_reflow_count();
                if pending_reflows > 0 {
                    window.reflow(ReflowGoal::ForDisplay,
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
            },
            MixedMessage::FromScheduler(_) => ScriptTaskEventCategory::TimerEvent,
        }
    }

    fn profile_event<F, R>(&self, category: ScriptTaskEventCategory, f: F) -> R
        where F: FnOnce() -> R {

        if opts::get().profile_script_events {
            let profiler_cat = match category {
                ScriptTaskEventCategory::AttachLayout => ProfilerCategory::ScriptAttachLayout,
                ScriptTaskEventCategory::ConstellationMsg => ProfilerCategory::ScriptConstellationMsg,
                ScriptTaskEventCategory::DevtoolsMsg => ProfilerCategory::ScriptDevtoolsMsg,
                ScriptTaskEventCategory::DocumentEvent => ProfilerCategory::ScriptDocumentEvent,
                ScriptTaskEventCategory::DomEvent => ProfilerCategory::ScriptDomEvent,
                ScriptTaskEventCategory::FileRead => ProfilerCategory::ScriptFileRead,
                ScriptTaskEventCategory::ImageCacheMsg => ProfilerCategory::ScriptImageCacheMsg,
                ScriptTaskEventCategory::InputEvent => ProfilerCategory::ScriptInputEvent,
                ScriptTaskEventCategory::NetworkEvent => ProfilerCategory::ScriptNetworkEvent,
                ScriptTaskEventCategory::Resize => ProfilerCategory::ScriptResize,
                ScriptTaskEventCategory::ScriptEvent => ProfilerCategory::ScriptEvent,
                ScriptTaskEventCategory::UpdateReplacedElement => {
                    ProfilerCategory::ScriptUpdateReplacedElement
                }
                ScriptTaskEventCategory::StylesheetLoad => ProfilerCategory::ScriptStylesheetLoad,
                ScriptTaskEventCategory::SetViewport => ProfilerCategory::ScriptSetViewport,
                ScriptTaskEventCategory::TimerEvent => ProfilerCategory::ScriptTimerEvent,
                ScriptTaskEventCategory::WebSocketEvent => ProfilerCategory::ScriptWebSocketEvent,
                ScriptTaskEventCategory::WorkerEvent => ProfilerCategory::ScriptWorkerEvent,
            };
            profile(profiler_cat, None, self.time_profiler_chan.clone(), f)
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
            ConstellationControlMsg::WebFontLoaded(pipeline_id) =>
                self.handle_web_font_loaded(pipeline_id),
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
        }
    }

    fn handle_timer_event(&self, timer_event: TimerEvent) {
        let TimerEvent(source, id) = timer_event;

        let pipeline_id = match source {
            TimerSource::FromWindow(pipeline_id) => pipeline_id,
            TimerSource::FromWorker => panic!("Worker timeouts must not be sent to script task"),
        };

        let page = self.root_page();
        let page = page.find(pipeline_id).expect("ScriptTask: received fire timer msg for a
            pipeline ID not associated with this script task. This is a bug.");
        let window = page.window();

        window.handle_fire_timer(id);
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
                devtools::handle_set_timeline_markers(&page, marker_types, reply),
            DevtoolScriptControlMsg::DropTimelineMarkers(_pipeline_id, marker_types) =>
                devtools::handle_drop_timeline_markers(&page, marker_types),
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
            WebDriverScriptCommand::FocusElement(element_id, reply) =>
                webdriver_handlers::handle_focus_element(&page, pipeline_id, element_id, reply),
            WebDriverScriptCommand::GetActiveElement(reply) =>
                webdriver_handlers::handle_get_active_element(&page, pipeline_id, reply),
            WebDriverScriptCommand::GetElementTagName(node_id, reply) =>
                webdriver_handlers::handle_get_name(&page, pipeline_id, node_id, reply),
            WebDriverScriptCommand::GetElementAttribute(node_id, name, reply) =>
                webdriver_handlers::handle_get_attribute(&page, pipeline_id, node_id, name, reply),
            WebDriverScriptCommand::GetElementCSS(node_id, name, reply) =>
                webdriver_handlers::handle_get_css(&page, pipeline_id, node_id, name, reply),
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
        if let Some(ref page) = self.find_subpage(id) {
            let window = page.window();
            window.set_resize_event(size);
            return;
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
                if window.set_page_clip_rect_with_new_viewport(rect) {
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
        let load_pending = doc.ReadyState() != DocumentReadyState::Complete;
        if load_pending {
            return ScriptState::DocumentLoading;
        }

        // Checks if the html element has reftest-wait attribute present.
        // See http://testthewebforward.org/docs/reftests.html
        let html_element = doc.GetDocumentElement();
        let reftest_wait = html_element.map_or(false, |elem| elem.has_class(&Atom::from_slice("reftest-wait")));
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
            content_process_shutdown_chan,
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
            content_process_shutdown_chan: content_process_shutdown_chan,
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
                                           layout_chan, parent_window.window_size(),
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
        let handler = box DocumentProgressHandler::new(addr.clone());
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

                for child in it_page.document().upcast::<Node>().traverse_preorder() {
                    dom_tree_size += heap_size_of_self_and_children(&*child);
                }
                let window = it_page.window();
                dom_tree_size += heap_size_of_self_and_children(&*window);

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

    /// Handles freeze message
    fn handle_freeze_msg(&self, id: PipelineId) {
        // Workaround for a race condition when navigating before the initial page has
        // been constructed c.f. https://github.com/servo/servo/issues/7677
        if self.page.borrow().is_none() {
            return
        };
        let page = self.root_page();
        let page = page.find(id).expect("ScriptTask: received freeze msg for a
                    pipeline ID not associated with this script task. This is a bug.");
        let window = page.window();
        window.freeze();
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
        window.thaw();
    }

    fn handle_focus_iframe_msg(&self,
                               parent_pipeline_id: PipelineId,
                               subpage_id: SubpageId) {
        let borrowed_page = self.root_page();
        let page = borrowed_page.find(parent_pipeline_id).unwrap();

        let doc = page.document();
        let frame_element = doc.find_iframe(subpage_id);

        if let Some(ref frame_element) = frame_element {
            doc.begin_focus_transaction();
            doc.request_focus(frame_element.upcast());
            doc.commit_focus_transaction(FocusType::Parent);
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
            frame_element.dispatch_mozbrowser_event(event);
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

        frame_element.unwrap().update_subpage_id(new_subpage_id);
    }

    /// Window was resized, but this script was not active, so don't reflow yet
    fn handle_resize_inactive_msg(&self, id: PipelineId, new_size: WindowSizeData) {
        let page = self.root_page();
        let page = page.find(id).expect("Received resize message for PipelineId not associated
            with a page in the page tree. This is a bug.");
        let window = page.window();
        window.set_window_size(new_size);
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
        document.send_title_to_compositor();
    }

    /// Handles a request to exit the script task and shut down layout.
    /// Returns true if the script task should shut down and false otherwise.
    fn handle_exit_pipeline_msg(&self, id: PipelineId) -> bool {
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
                chan.send(layout_interface::Msg::ExitNow).ok();
            }

            let has_pending_loads = self.incomplete_loads.borrow().len() > 0;
            let has_root_page = self.page.borrow().is_some();

            // Exit if no pending loads and no root page
            return !has_pending_loads && !has_root_page;
        }

        // If root is being exited, shut down all pages
        let page = self.root_page();
        let window = page.window();
        if window.pipeline() == id {
            debug!("shutting down layout for root page {:?}", id);
            shut_down_layout(&page);
            return true
        }

        // otherwise find just the matching page and exit all sub-pages
        if let Some(ref mut child_page) = page.remove(id) {
            shut_down_layout(&*child_page);
        }
        false
    }

    /// Handles when layout task finishes all animation in one tick
    fn handle_tick_all_animations(&self, id: PipelineId) {
        let page = get_page(&self.root_page(), id);
        let document = page.document();
        document.run_the_animation_frame_callbacks();
    }

    /// Handles a Web font being loaded. Does nothing if the page no longer exists.
    fn handle_web_font_loaded(&self, pipeline_id: PipelineId) {
        if let Some(ref page) = self.find_subpage(pipeline_id)  {
            self.rebuild_and_force_reflow(page, ReflowReason::WebFontLoaded);
        }
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

        let (ipc_timer_event_chan, ipc_timer_event_port) = ipc::channel().unwrap();
        ROUTER.route_ipc_receiver_to_mpsc_sender(ipc_timer_event_port,
                                                 self.timer_event_chan.clone());

        // Create the window and document objects.
        let window = Window::new(self.js_runtime.clone(),
                                 page.clone(),
                                 MainThreadScriptChan(sender.clone()),
                                 self.image_cache_channel.clone(),
                                 self.compositor.borrow_mut().clone(),
                                 self.image_cache_task.clone(),
                                 self.resource_task.clone(),
                                 self.storage_task.clone(),
                                 self.mem_profiler_chan.clone(),
                                 self.devtools_chan.clone(),
                                 self.constellation_chan.clone(),
                                 self.scheduler_chan.clone(),
                                 ipc_timer_event_chan,
                                 incomplete.layout_chan,
                                 incomplete.pipeline_id,
                                 incomplete.parent_info,
                                 incomplete.window_size);

        let last_modified = metadata.headers.as_ref().and_then(|headers| {
            headers.get().map(|&LastModified(HttpDate(ref tm))| dom_last_modified(tm))
        });

        let content_type = match metadata.content_type {
            Some(ContentType(Mime(TopLevel::Text, SubLevel::Plain, _))) => {
                Some(DOMString::from("text/plain"))
            }
            _ => None
        };

        let loader = DocumentLoader::new_with_task(self.resource_task.clone(),
                                                   Some(page.pipeline()),
                                                   Some(incomplete.url.clone()));
        let document = Document::new(window.r(),
                                     Some(final_url.clone()),
                                     IsHTMLDocument::HTMLDocument,
                                     content_type,
                                     last_modified,
                                     DocumentSource::FromParser,
                                     loader);

        let frame_element = frame_element.r().map(Castable::upcast);
        window.init_browsing_context(document.r(), frame_element);

        document.set_ready_state(DocumentReadyState::Loading);

        // Create the root frame
        page.set_frame(Some(Frame {
            document: JS::from_rooted(&document),
            window: JS::from_rooted(&window),
        }));

        let is_javascript = incomplete.url.scheme == "javascript";
        let parse_input = if is_javascript {
            use url::percent_encoding::percent_decode_to;

            // Turn javascript: URL into JS code to eval, according to the steps in
            // https://html.spec.whatwg.org/multipage/#javascript-protocol
            let _ar = JSAutoRequest::new(self.get_cx());
            let mut script_source_bytes = Vec::new();
            // Start with the scheme data of the parsed URL (5.), while percent-decoding (8.)
            percent_decode_to(incomplete.url.non_relative_scheme_data().unwrap().as_bytes(),
                              &mut script_source_bytes);
            // Append question mark and query component, if any (6.), while percent-decoding (8.)
            if let Some(ref query) = incomplete.url.query {
                script_source_bytes.push(b'?');
                percent_decode_to(query.as_bytes(), &mut script_source_bytes);
            }
            // Append number sign and fragment component if any (7.), while percent-decoding (8.)
            if let Some(ref fragment) = incomplete.url.fragment {
                script_source_bytes.push(b'#');
                percent_decode_to(fragment.as_bytes(), &mut script_source_bytes);
            }

            // UTF-8 decode (9.)
            let script_source = String::from_utf8_lossy(&script_source_bytes);

            // Script source is ready to be evaluated (11.)
            unsafe {
                let mut jsval = RootedValue::new(self.get_cx(), UndefinedValue());
                window.evaluate_js_on_global_with_result(&script_source, jsval.handle_mut());
                let strval = DOMString::from_jsval(self.get_cx(), jsval.handle(),
                                                   StringificationBehavior::Empty);
                strval.unwrap_or(DOMString::new())
            }
        } else {
            DOMString::new()
        };

        parse_html(document.r(), parse_input, final_url,
                   ParseContext::Owner(Some(incomplete.pipeline_id)));

        page_remover.neuter();

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
        let rect = element.upcast::<Node>().get_bounding_content_box();

        // In order to align with element edges, we snap to unscaled pixel boundaries, since the
        // paint task currently does the same for drawing elements. This is important for pages
        // that require pixel perfect scroll positioning for proper display (like Acid2). Since we
        // don't have the device pixel ratio here, this might not be accurate, but should work as
        // long as the ratio is a whole number. Once #8275 is fixed this should actually take into
        // account the real device pixel ratio.
        let point = Point2D::new(rect.origin.x.to_nearest_px() as f32,
                                 rect.origin.y.to_nearest_px() as f32);

        self.compositor.borrow_mut().send(ScriptToCompositorMsg::ScrollFragmentPoint(
                                                 pipeline_id, LayerId::null(), point, false)).unwrap();
    }

    /// Reflows non-incrementally, rebuilding the entire layout tree in the process.
    fn rebuild_and_force_reflow(&self, page: &Page, reason: ReflowReason) {
        let document = page.document();
        document.dirty_all_nodes();
        let window = window_from_node(document.r());
        window.reflow(ReflowGoal::ForDisplay, ReflowQueryType::NoQuery, reason);
    }

    /// This is the main entry point for receiving and dispatching DOM events.
    ///
    /// TODO: Actually perform DOM event dispatch.
    fn handle_event(&self, pipeline_id: PipelineId, event: CompositorEvent) {

        match event {
            ResizeEvent(new_size) => {
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
                let page = get_page(&self.root_page(), pipeline_id);
                let document = page.document();

                let mut prev_mouse_over_targets: RootedVec<JS<Element>> = RootedVec::new();
                for target in &*self.mouse_over_targets.borrow_mut() {
                    prev_mouse_over_targets.push(target.clone());
                }

                // We temporarily steal the list of targets over which the mouse is to pass it to
                // handle_mouse_move_event() in a safe RootedVec container.
                let mut mouse_over_targets = RootedVec::new();
                std_mem::swap(&mut *self.mouse_over_targets.borrow_mut(), &mut *mouse_over_targets);
                document.handle_mouse_move_event(self.js_runtime.rt(), point, &mut mouse_over_targets);

                // Notify Constellation about anchors that are no longer mouse over targets.
                for target in &*prev_mouse_over_targets {
                    if !mouse_over_targets.contains(target) {
                        if target.is::<HTMLAnchorElement>() {
                            let event = ConstellationMsg::NodeStatus(None);
                            let ConstellationChan(ref chan) = self.constellation_chan;
                            chan.send(event).unwrap();
                            break;
                        }
                    }
                }

                // Notify Constellation about the topmost anchor mouse over target.
                for target in &*mouse_over_targets {
                    if target.is::<HTMLAnchorElement>() {
                        let status = target.get_attribute(&ns!(""), &atom!("href"))
                            .and_then(|href| {
                                let value = href.value();
                                let url = document.url();
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

            TouchEvent(event_type, identifier, point) => {
                let handled = self.handle_touch_event(pipeline_id, event_type, identifier, point);
                match event_type {
                    TouchEventType::Down => {
                        if handled {
                            // TODO: Wait to see if preventDefault is called on the first touchmove event.
                            self.compositor.borrow_mut()
                                .send(ScriptToCompositorMsg::TouchEventProcessed(
                                        EventResult::DefaultAllowed)).unwrap();
                        } else {
                            self.compositor.borrow_mut()
                                .send(ScriptToCompositorMsg::TouchEventProcessed(
                                        EventResult::DefaultPrevented)).unwrap();
                        }
                    }
                    _ => {
                        // TODO: Calling preventDefault on a touchup event should prevent clicks.
                    }
                }
            }

            KeyEvent(key, state, modifiers) => {
                let page = get_page(&self.root_page(), pipeline_id);
                let document = page.document();
                document.dispatch_key_event(
                    key, state, modifiers, &mut self.compositor.borrow_mut());
            }
        }
    }

    fn handle_mouse_event(&self,
                          pipeline_id: PipelineId,
                          mouse_event_type: MouseEventType,
                          button: MouseButton,
                          point: Point2D<f32>) {
        let page = get_page(&self.root_page(), pipeline_id);
        let document = page.document();
        document.handle_mouse_event(self.js_runtime.rt(), button, point, mouse_event_type);
    }

    fn handle_touch_event(&self,
                          pipeline_id: PipelineId,
                          event_type: TouchEventType,
                          identifier: TouchId,
                          point: Point2D<f32>)
                          -> bool {
        let page = get_page(&self.root_page(), pipeline_id);
        let document = page.document();
        document.handle_touch_event(self.js_runtime.rt(), event_type, identifier, point)
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
        window.set_window_size(new_size);
        window.force_reflow(ReflowGoal::ForDisplay,
                            ReflowQueryType::NoQuery,
                            ReflowReason::WindowResize);

        let document = page.document();
        let fragment_node = window.steal_fragment_name()
                                  .and_then(|name| document.find_fragment_node(&*name));
        match fragment_node {
            Some(ref node) => self.scroll_fragment_point(pipeline_id, node.r()),
            None => {}
        }

        // http://dev.w3.org/csswg/cssom-view/#resizing-viewports
        // https://dvcs.w3.org/hg/dom3events/raw-file/tip/html/DOM3-Events.html#event-type-resize
        let uievent = UIEvent::new(window.r(),
                                   DOMString::from("resize"), EventBubbles::DoesNotBubble,
                                   EventCancelable::NotCancelable, Some(window.r()),
                                   0i32);
        uievent.upcast::<Event>().fire(window.upcast());
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
        }, LoadConsumer::Listener(response_target), None)).unwrap();

        self.incomplete_loads.borrow_mut().push(incomplete);
    }

    fn handle_parsing_complete(&self, id: PipelineId) {
        let parent_page = self.root_page();
        let page = match parent_page.find(id) {
            Some(page) => page,
            None => return,
        };

        let document = page.document();
        let final_url = document.url();

        // https://html.spec.whatwg.org/multipage/#the-end step 1
        document.set_ready_state(DocumentReadyState::Interactive);

        // TODO: Execute step 2 here.

        // Kick off the initial reflow of the page.
        debug!("kicking off initial reflow of {:?}", final_url);
        document.disarm_reflow_timeout();
        document.content_changed(document.upcast(),
                                 NodeDamage::OtherNodeDamage);
        let window = window_from_node(document.r());
        window.reflow(ReflowGoal::ForDisplay, ReflowQueryType::NoQuery, ReflowReason::FirstLoad);

        // No more reflow required
        page.set_reflow_status(false);

        // https://html.spec.whatwg.org/multipage/#the-end steps 3-4.
        document.process_deferred_scripts();

        window.set_fragment_name(final_url.fragment.clone());

        // Notify devtools that a new script global exists.
        //TODO: should this happen as soon as the global is created, or at least once the first
        // script runs?
        self.notify_devtools(document.Title(), (*final_url).clone(), (id, None));
    }
}

impl Drop for ScriptTask {
    fn drop(&mut self) {
        SCRIPT_TASK_ROOT.with(|root| {
            *root.borrow_mut() = None;
        });
    }
}

/// Shuts down layout for the given page tree.
fn shut_down_layout(page_tree: &Rc<Page>) {
    let mut channels = vec!();

    for page in page_tree.iter() {
        // Tell the layout task to begin shutting down, and wait until it
        // processed this message.
        let (response_chan, response_port) = channel();
        let window = page.window();
        let LayoutChan(chan) = window.layout_chan();
        if chan.send(layout_interface::Msg::PrepareToExit(response_chan)).is_ok() {
            channels.push(chan);
            response_port.recv().unwrap();
        }
    }

    // Drop our references to the JSContext and DOM objects.
    for page in page_tree.iter() {
        let window = page.window();
        window.clear_js_runtime();
        // Sever the connection between the global and the DOM tree
        page.set_frame(None);
    }

    // Destroy the layout task. If there were node leaks, layout will now crash safely.
    for chan in channels {
        chan.send(layout_interface::Msg::ExitNow).ok();
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
