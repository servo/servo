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

use dom::bindings::cell::DOMRefCell;
use dom::bindings::codegen::Bindings::DocumentBinding::{DocumentMethods, DocumentReadyState};
use dom::bindings::codegen::InheritTypes::{ElementCast, EventTargetCast, HTMLIFrameElementCast, NodeCast, EventCast};
use dom::bindings::conversions::FromJSValConvertible;
use dom::bindings::conversions::StringificationBehavior;
use dom::bindings::js::{JS, JSRef, Temporary, OptionalRootable, RootedReference};
use dom::bindings::js::{RootCollection, RootCollectionPtr, Unrooted};
use dom::bindings::refcounted::{LiveDOMReferences, Trusted, TrustedReference};
use dom::bindings::structuredclone::StructuredCloneData;
use dom::bindings::trace::JSTraceable;
use dom::bindings::utils::{wrap_for_same_compartment, pre_wrap};
use dom::document::{Document, IsHTMLDocument, DocumentHelpers, DocumentProgressHandler, DocumentProgressTask, DocumentSource};
use dom::element::{Element, AttributeHandlers};
use dom::event::{Event, EventHelpers, EventBubbles, EventCancelable};
use dom::htmliframeelement::HTMLIFrameElementHelpers;
use dom::uievent::UIEvent;
use dom::eventtarget::EventTarget;
use dom::node::{self, Node, NodeHelpers, NodeDamage, window_from_node};
use dom::window::{Window, WindowHelpers, ScriptHelpers, ReflowReason};
use parse::html::{HTMLInput, parse_html};
use layout_interface::{ScriptLayoutChan, LayoutChan, ReflowGoal, ReflowQueryType};
use layout_interface;
use page::{Page, IterablePage, Frame};
use timers::TimerId;
use devtools;

use devtools_traits::{DevtoolsControlChan, DevtoolsControlPort, DevtoolsPageInfo};
use devtools_traits::{DevtoolsControlMsg, DevtoolScriptControlMsg};
use script_traits::CompositorEvent;
use script_traits::CompositorEvent::{ResizeEvent, ReflowEvent, ClickEvent};
use script_traits::CompositorEvent::{MouseDownEvent, MouseUpEvent};
use script_traits::CompositorEvent::{MouseMoveEvent, KeyEvent};
use script_traits::{NewLayoutInfo, OpaqueScriptLayoutChannel};
use script_traits::{ConstellationControlMsg, ScriptControlChan};
use script_traits::ScriptTaskFactory;
use msg::compositor_msg::ReadyState::{FinishedLoading, Loading, PerformingLayout};
use msg::compositor_msg::{LayerId, ScriptListener};
use msg::constellation_msg::{ConstellationChan};
use msg::constellation_msg::{LoadData, PipelineId, SubpageId, MozBrowserEvent};
use msg::constellation_msg::{Failure, WindowSizeData, PipelineExitType};
use msg::constellation_msg::Msg as ConstellationMsg;
use net::image_cache_task::ImageCacheTask;
use net::resource_task::{ResourceTask, ControlMsg, LoadResponse};
use net::resource_task::LoadData as NetLoadData;
use net::storage_task::StorageTask;
use string_cache::Atom;
use util::geometry::to_frac_px;
use util::smallvec::SmallVec;
use util::str::DOMString;
use util::task::{spawn_named, spawn_named_with_send_on_failure};
use util::task_state;

use geom::Rect;
use geom::point::Point2D;
use hyper::header::{LastModified, Headers};
use js::jsapi::{JS_SetWrapObjectCallbacks, JS_SetGCZeal, JS_SetExtraGCRootsTracer, JS_DEFAULT_ZEAL_FREQ};
use js::jsapi::{JSContext, JSRuntime, JSObject, JSTracer};
use js::jsapi::{JS_SetGCParameter, JSGC_MAX_BYTES};
use js::jsapi::{JS_SetGCCallback, JSGCStatus, JSGC_BEGIN, JSGC_END};
use js::rust::{Cx, RtUtils};
use js;
use url::Url;

use libc;
use std::ascii::AsciiExt;
use std::any::Any;
use std::borrow::ToOwned;
use std::cell::{Cell, RefCell};
use std::num::ToPrimitive;
use std::option::Option;
use std::ptr;
use std::rc::Rc;
use std::result::Result;
use std::sync::mpsc::{channel, Sender, Receiver, Select};
use std::u32;
use time::Tm;

thread_local!(pub static STACK_ROOTS: Cell<Option<RootCollectionPtr>> = Cell::new(None));
thread_local!(static SCRIPT_TASK_ROOT: RefCell<Option<*const ScriptTask>> = RefCell::new(None));

unsafe extern fn trace_script_tasks(tr: *mut JSTracer, _data: *mut libc::c_void) {
    SCRIPT_TASK_ROOT.with(|root| {
        if let Some(script_task) = *root.borrow() {
            (*script_task).trace(tr);
        }
    });
}

/// A document load that is in the process of fetching the requested resource. Contains
/// data that will need to be present when the document and frame tree entry are created,
/// but is only easily available at initiation of the load and on a push basis (so some
/// data will be updated according to future resize events, viewport changes, etc.)
#[jstraceable]
struct InProgressLoad {
    /// The pipeline which requested this load.
    pipeline_id: PipelineId,
    /// The parent pipeline and child subpage associated with this load, if any.
    parent_info: Option<(PipelineId, SubpageId)>,
    /// The current window size associated with this pipeline.
    window_size: Option<WindowSizeData>,
    /// Channel to the layout task associated with this pipeline.
    layout_chan: LayoutChan,
    /// The current viewport clipping rectangle applying to this pipelie, if any.
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

#[derive(Copy)]
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

/// Messages used to control script event loops, such as ScriptTask and
/// DedicatedWorkerGlobalScope.
pub enum ScriptMsg {
    /// Acts on a fragment URL load on the specified pipeline (only dispatched
    /// to ScriptTask).
    TriggerFragment(PipelineId, String),
    /// Begins a content-initiated load on the specified pipeline (only
    /// dispatched to ScriptTask).
    Navigate(PipelineId, LoadData),
    /// Fires a JavaScript timeout
    /// TimerSource must be FromWindow when dispatched to ScriptTask and
    /// must be FromWorker when dispatched to a DedicatedGlobalWorkerScope
    FireTimer(TimerSource, TimerId),
    /// Notifies the script that a window associated with a particular pipeline
    /// should be closed (only dispatched to ScriptTask).
    ExitWindow(PipelineId),
    /// Message sent through Worker.postMessage (only dispatched to
    /// DedicatedWorkerGlobalScope).
    DOMMessage(StructuredCloneData),
    /// Generic message that encapsulates event handling.
    RunnableMsg(Box<Runnable+Send>),
    /// Generic message for running tasks in the ScriptTask
    MainThreadRunnableMsg(Box<MainThreadRunnable+Send>),
    /// A DOM object's last pinned reference was removed (dispatched to all tasks).
    RefcountCleanup(TrustedReference),
    /// The final network response for a page has arrived.
    PageFetchComplete(PipelineId, Option<SubpageId>, LoadResponse),
}

/// A cloneable interface for communicating with an event loop.
pub trait ScriptChan {
    /// Send a message to the associated event loop.
    fn send(&self, msg: ScriptMsg) -> Result<(), ()>;
    /// Clone this handle.
    fn clone(&self) -> Box<ScriptChan+Send>;
}

/// Encapsulates internal communication within the script task.
#[jstraceable]
pub struct NonWorkerScriptChan(pub Sender<ScriptMsg>);

impl ScriptChan for NonWorkerScriptChan {
    fn send(&self, msg: ScriptMsg) -> Result<(), ()> {
        let NonWorkerScriptChan(ref chan) = *self;
        return chan.send(msg).map_err(|_| ());
    }

    fn clone(&self) -> Box<ScriptChan+Send> {
        let NonWorkerScriptChan(ref chan) = *self;
        box NonWorkerScriptChan((*chan).clone())
    }
}

impl NonWorkerScriptChan {
    /// Creates a new script chan.
    pub fn new() -> (Receiver<ScriptMsg>, Box<NonWorkerScriptChan>) {
        let (chan, port) = channel();
        (port, box NonWorkerScriptChan(chan))
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
///
/// FIXME: Rename to `Page`, following WebKit?
#[jstraceable]
pub struct ScriptTask {
    /// A handle to the information pertaining to page layout
    page: DOMRefCell<Option<Rc<Page>>>,
    /// A list of data pertaining to loads that have not yet received a network response
    incomplete_loads: DOMRefCell<Vec<InProgressLoad>>,
    /// A handle to the image cache task.
    image_cache_task: ImageCacheTask,
    /// A handle to the resource task.
    resource_task: ResourceTask,
    /// A handle to the storage task.
    storage_task: StorageTask,

    /// The port on which the script task receives messages (load URL, exit, etc.)
    port: Receiver<ScriptMsg>,
    /// A channel to hand out to script task-based entities that need to be able to enqueue
    /// events in the event queue.
    chan: NonWorkerScriptChan,

    /// A channel to hand out to tasks that need to respond to a message from the script task.
    control_chan: ScriptControlChan,

    /// The port on which the constellation and layout tasks can communicate with the
    /// script task.
    control_port: Receiver<ConstellationControlMsg>,

    /// For communicating load url messages to the constellation
    constellation_chan: ConstellationChan,
    /// A handle to the compositor for communicating ready state messages.
    compositor: DOMRefCell<Box<ScriptListener+'static>>,

    /// For providing instructions to an optional devtools server.
    devtools_chan: Option<DevtoolsControlChan>,
    /// For receiving commands from an optional devtools server. Will be ignored if
    /// no such server exists.
    devtools_port: DevtoolsControlPort,
    devtools_sender: Sender<DevtoolScriptControlMsg>,

    /// The JavaScript runtime.
    js_runtime: js::rust::rt,
    /// The JSContext.
    js_context: DOMRefCell<Option<Rc<Cx>>>,

    mouse_over_targets: DOMRefCell<Vec<JS<Node>>>
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

#[unsafe_destructor]
impl<'a> Drop for ScriptMemoryFailsafe<'a> {
    #[allow(unrooted_must_root)]
    fn drop(&mut self) {
        match self.owner {
            Some(owner) => {
                unsafe {
                    let page = owner.page.borrow_for_script_deallocation();
                    for page in page.iter() {
                        let window = Unrooted::from_temporary(page.window());
                        (*window.unsafe_get()).clear_js_context_for_script_deallocation();
                    }
                    *owner.js_context.borrow_for_script_deallocation() = None;
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

    fn clone_layout_channel(_phantom: Option<&mut ScriptTask>, pair: &OpaqueScriptLayoutChannel) -> Box<Any+Send> {
        box pair.sender() as Box<Any+Send>
    }

    fn create<C>(_phantom: Option<&mut ScriptTask>,
                 id: PipelineId,
                 parent_info: Option<(PipelineId, SubpageId)>,
                 compositor: C,
                 layout_chan: &OpaqueScriptLayoutChannel,
                 control_chan: ScriptControlChan,
                 control_port: Receiver<ConstellationControlMsg>,
                 constellation_chan: ConstellationChan,
                 failure_msg: Failure,
                 resource_task: ResourceTask,
                 storage_task: StorageTask,
                 image_cache_task: ImageCacheTask,
                 devtools_chan: Option<DevtoolsControlChan>,
                 window_size: Option<WindowSizeData>,
                 load_data: LoadData)
                 where C: ScriptListener + Send + 'static {
        let ConstellationChan(const_chan) = constellation_chan.clone();
        let (script_chan, script_port) = channel();
        let layout_chan = LayoutChan(layout_chan.sender());
        spawn_named_with_send_on_failure("ScriptTask", task_state::SCRIPT, move || {
            let script_task = ScriptTask::new(box compositor as Box<ScriptListener>,
                                              script_port,
                                              NonWorkerScriptChan(script_chan),
                                              control_chan,
                                              control_port,
                                              constellation_chan,
                                              resource_task,
                                              storage_task,
                                              image_cache_task,
                                              devtools_chan);

            SCRIPT_TASK_ROOT.with(|root| {
                *root.borrow_mut() = Some(&script_task as *const _);
            });
            let mut failsafe = ScriptMemoryFailsafe::new(&script_task);

            let new_load = InProgressLoad::new(id, parent_info, layout_chan, window_size,
                                               load_data.url.clone());
            script_task.start_page_load(new_load, load_data);

            script_task.start();

            // This must always be the very last operation performed before the task completes
            failsafe.neuter();
        }, ConstellationMsg::Failure(failure_msg), const_chan);
    }
}

unsafe extern "C" fn debug_gc_callback(_rt: *mut JSRuntime, status: JSGCStatus) {
    match status {
        JSGC_BEGIN => task_state::enter(task_state::IN_GC),
        JSGC_END   => task_state::exit(task_state::IN_GC),
        _ => (),
    }
}

impl ScriptTask {
    /// Creates a new script task.
    pub fn new(compositor: Box<ScriptListener+'static>,
               port: Receiver<ScriptMsg>,
               chan: NonWorkerScriptChan,
               control_chan: ScriptControlChan,
               control_port: Receiver<ConstellationControlMsg>,
               constellation_chan: ConstellationChan,
               resource_task: ResourceTask,
               storage_task: StorageTask,
               img_cache_task: ImageCacheTask,
               devtools_chan: Option<DevtoolsControlChan>)
               -> ScriptTask {
        let (js_runtime, js_context) = ScriptTask::new_rt_and_cx();
        let wrap_for_same_compartment = wrap_for_same_compartment as
            unsafe extern "C" fn(*mut JSContext, *mut JSObject) -> *mut JSObject;
        let pre_wrap = pre_wrap as
            unsafe extern fn(*mut JSContext, *mut JSObject, *mut JSObject,
                             libc::c_uint) -> *mut JSObject;

        unsafe {
            // JS_SetWrapObjectCallbacks clobbers the existing wrap callback,
            // and JSCompartment::wrap crashes if that happens. The only way
            // to retrieve the default callback is as the result of
            // JS_SetWrapObjectCallbacks, which is why we call it twice.
            let callback = JS_SetWrapObjectCallbacks((*js_runtime).ptr,
                                                     None,
                                                     Some(wrap_for_same_compartment),
                                                     None);
            JS_SetWrapObjectCallbacks((*js_runtime).ptr,
                                      callback,
                                      Some(wrap_for_same_compartment),
                                      Some(pre_wrap));
        }

        let (devtools_sender, devtools_receiver) = channel();
        ScriptTask {
            page: DOMRefCell::new(None),
            incomplete_loads: DOMRefCell::new(vec!()),

            image_cache_task: img_cache_task,
            resource_task: resource_task,
            storage_task: storage_task,

            port: port,
            chan: chan,
            control_chan: control_chan,
            control_port: control_port,
            constellation_chan: constellation_chan,
            compositor: DOMRefCell::new(compositor),
            devtools_chan: devtools_chan,
            devtools_port: devtools_receiver,
            devtools_sender: devtools_sender,

            js_runtime: js_runtime,
            js_context: DOMRefCell::new(Some(js_context)),
            mouse_over_targets: DOMRefCell::new(vec!())
        }
    }

    pub fn new_rt_and_cx() -> (js::rust::rt, Rc<Cx>) {
        LiveDOMReferences::initialize();
        let js_runtime = js::rust::rt();
        assert!({
            let ptr: *mut JSRuntime = (*js_runtime).ptr;
            !ptr.is_null()
        });

        // Unconstrain the runtime's threshold on nominal heap size, to avoid
        // triggering GC too often if operating continuously near an arbitrary
        // finite threshold. This leaves the maximum-JS_malloc-bytes threshold
        // still in effect to cause periodical, and we hope hygienic,
        // last-ditch GCs from within the GC's allocator.
        unsafe {
            JS_SetGCParameter(js_runtime.ptr, JSGC_MAX_BYTES, u32::MAX);
        }

        let js_context = js_runtime.cx();
        assert!({
            let ptr: *mut JSContext = (*js_context).ptr;
            !ptr.is_null()
        });
        js_context.set_default_options_and_version();
        js_context.set_logging_error_reporter();
        unsafe {
            JS_SetGCZeal((*js_context).ptr, 0, JS_DEFAULT_ZEAL_FREQ);
            JS_SetExtraGCRootsTracer((*js_runtime).ptr, Some(trace_script_tasks), ptr::null_mut());
        }

        // Needed for debug assertions about whether GC is running.
        if !cfg!(ndebug) {
            unsafe {
                JS_SetGCCallback(js_runtime.ptr,
                    Some(debug_gc_callback as unsafe extern "C" fn(*mut JSRuntime, JSGCStatus)));
            }
        }

        (js_runtime, js_context)
    }

    // Return the root page in the frame tree. Panics if it doesn't exist.
    fn root_page(&self) -> Rc<Page> {
        self.page.borrow().as_ref().unwrap().clone()
    }

    pub fn get_cx(&self) -> *mut JSContext {
        (**self.js_context.borrow().as_ref().unwrap()).ptr
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
        let roots = RootCollection::new();
        let _stack_roots_tls = StackRootTLS::new(&roots);

        // Handle pending resize events.
        // Gather them first to avoid a double mut borrow on self.
        let mut resizes = vec!();

        {
            let page = self.page.borrow();
            if let Some(page) = page.as_ref() {
                for page in page.iter() {
                    // Only process a resize if layout is idle.
                    let window = page.window().root();
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

        for (id, size) in resizes.into_iter() {
            self.handle_event(id, ResizeEvent(size));
        }

        enum MixedMessage {
            FromConstellation(ConstellationControlMsg),
            FromScript(ScriptMsg),
            FromDevtools(DevtoolScriptControlMsg),
        }

        // Store new resizes, and gather all other events.
        let mut sequential = vec!();

        // Receive at least one message so we don't spinloop.
        let mut event = {
            let sel = Select::new();
            let mut port1 = sel.handle(&self.port);
            let mut port2 = sel.handle(&self.control_port);
            let mut port3 = sel.handle(&self.devtools_port);
            unsafe {
                port1.add();
                port2.add();
                if self.devtools_chan.is_some() {
                    port3.add();
                }
            }
            let ret = sel.wait();
            if ret == port1.id() {
                MixedMessage::FromScript(self.port.recv().unwrap())
            } else if ret == port2.id() {
                MixedMessage::FromConstellation(self.control_port.recv().unwrap())
            } else if ret == port3.id() {
                MixedMessage::FromDevtools(self.devtools_port.recv().unwrap())
            } else {
                panic!("unexpected select result")
            }
        };

        // Squash any pending resize and reflow events in the queue.
        loop {
            match event {
                // This has to be handled before the ResizeMsg below,
                // otherwise the page may not have been added to the
                // child list yet, causing the find() to fail.
                MixedMessage::FromConstellation(ConstellationControlMsg::AttachLayout(new_layout_info)) => {
                    self.handle_new_layout(new_layout_info);
                }
                MixedMessage::FromConstellation(ConstellationControlMsg::Resize(id, size)) => {
                    self.handle_resize(id, size);
                }
                MixedMessage::FromConstellation(ConstellationControlMsg::Viewport(id, rect)) => {
                    self.handle_viewport(id, rect);
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
                        Err(_) => break,
                        Ok(ev) => event = MixedMessage::FromDevtools(ev),
                    },
                    Ok(ev) => event = MixedMessage::FromScript(ev),
                },
                Ok(ev) => event = MixedMessage::FromConstellation(ev),
            }
        }

        // Process the gathered events.
        for msg in sequential.into_iter() {
            match msg {
                MixedMessage::FromConstellation(ConstellationControlMsg::ExitPipeline(id, exit_type)) => {
                    if self.handle_exit_pipeline_msg(id, exit_type) {
                        return false
                    }
                },
                MixedMessage::FromConstellation(inner_msg) => self.handle_msg_from_constellation(inner_msg),
                MixedMessage::FromScript(inner_msg) => self.handle_msg_from_script(inner_msg),
                MixedMessage::FromDevtools(inner_msg) => self.handle_msg_from_devtools(inner_msg),
            }
        }

        true
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
            ConstellationControlMsg::MozBrowserEventMsg(parent_pipeline_id,
                                                        subpage_id,
                                                        event) =>
                self.handle_mozbrowser_event_msg(parent_pipeline_id,
                                                 subpage_id,
                                                 event),
            ConstellationControlMsg::UpdateSubpageId(containing_pipeline_id,
                                                     old_subpage_id,
                                                     new_subpage_id) =>
                self.handle_update_subpage_id(containing_pipeline_id, old_subpage_id, new_subpage_id),
        }
    }

    fn handle_msg_from_script(&self, msg: ScriptMsg) {
        match msg {
            ScriptMsg::Navigate(id, load_data) =>
                self.handle_navigate(id, None, load_data),
            ScriptMsg::TriggerFragment(id, fragment) =>
                self.trigger_fragment(id, fragment),
            ScriptMsg::FireTimer(TimerSource::FromWindow(id), timer_id) =>
                self.handle_fire_timer_msg(id, timer_id),
            ScriptMsg::FireTimer(TimerSource::FromWorker, _) =>
                panic!("Worker timeouts must not be sent to script task"),
            ScriptMsg::ExitWindow(id) =>
                self.handle_exit_window_msg(id),
            ScriptMsg::DOMMessage(..) =>
                panic!("unexpected message"),
            ScriptMsg::RunnableMsg(runnable) =>
                runnable.handler(),
            ScriptMsg::MainThreadRunnableMsg(runnable) =>
                runnable.handler(self),
            ScriptMsg::RefcountCleanup(addr) =>
                LiveDOMReferences::cleanup(self.get_cx(), addr),
            ScriptMsg::PageFetchComplete(id, subpage, response) =>
                self.handle_page_fetch_complete(id, subpage, response),
        }
    }

    fn handle_msg_from_devtools(&self, msg: DevtoolScriptControlMsg) {
        let page = self.root_page();
        match msg {
            DevtoolScriptControlMsg::EvaluateJS(id, s, reply) =>
                devtools::handle_evaluate_js(&page, id, s, reply),
            DevtoolScriptControlMsg::GetRootNode(id, reply) =>
                devtools::handle_get_root_node(&page, id, reply),
            DevtoolScriptControlMsg::GetDocumentElement(id, reply) =>
                devtools::handle_get_document_element(&page, id, reply),
            DevtoolScriptControlMsg::GetChildren(id, node_id, reply) =>
                devtools::handle_get_children(&page, id, node_id, reply),
            DevtoolScriptControlMsg::GetLayout(id, node_id, reply) =>
                devtools::handle_get_layout(&page, id, node_id, reply),
            DevtoolScriptControlMsg::ModifyAttribute(id, node_id, modifications) =>
                devtools::handle_modify_attribute(&page, id, node_id, modifications),
            DevtoolScriptControlMsg::WantsLiveNotifications(pipeline_id, to_send) =>
                devtools::handle_wants_live_notifications(&page, pipeline_id, to_send),
        }
    }

    fn handle_resize(&self, id: PipelineId, size: WindowSizeData) {
        let page = self.page.borrow();
        if let Some(ref page) = page.as_ref() {
            if let Some(ref page) = page.find(id) {
                let window = page.window().root();
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
                let window = inner_page.window().root();
                if window.r().set_page_clip_rect_with_new_viewport(rect) {
                    let page = get_page(page, id);
                    self.force_reflow(&*page, ReflowReason::Viewport);
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
    fn handle_new_layout(&self, new_layout_info: NewLayoutInfo) {
        let NewLayoutInfo {
            containing_pipeline_id,
            new_pipeline_id,
            subpage_id,
            layout_chan,
            load_data,
        } = new_layout_info;

        let page = self.root_page();
        let parent_page = page.find(containing_pipeline_id).expect("ScriptTask: received a layout
            whose parent has a PipelineId which does not correspond to a pipeline in the script
            task's page tree. This is a bug.");

        let parent_window = parent_page.window().root();
        let chan = layout_chan.downcast_ref::<Sender<layout_interface::Msg>>().unwrap();
        let layout_chan = LayoutChan(chan.clone());
        // Kick off the fetch for the new resource.
        let new_load = InProgressLoad::new(new_pipeline_id, Some((containing_pipeline_id, subpage_id)),
                                           layout_chan, parent_window.r().window_size(),
                                           load_data.url.clone());
        self.start_page_load(new_load, load_data);
    }

    /// Handles a timer that fired.
    fn handle_fire_timer_msg(&self, id: PipelineId, timer_id: TimerId) {
        let page = self.root_page();
        let page = page.find(id).expect("ScriptTask: received fire timer msg for a
            pipeline ID not associated with this script task. This is a bug.");
        let window = page.window().root();
        window.r().handle_fire_timer(timer_id);
    }

    /// Handles freeze message
    fn handle_freeze_msg(&self, id: PipelineId) {
        let page = self.root_page();
        let page = page.find(id).expect("ScriptTask: received freeze msg for a
                    pipeline ID not associated with this script task. This is a bug.");
        let window = page.window().root();
        window.r().freeze();
    }

    /// Handles thaw message
    fn handle_thaw_msg(&self, id: PipelineId) {
        // We should only get this message when moving in history, so all pages requested
        // should exist.
        let page = self.root_page().find(id).unwrap();

        let needed_reflow = page.set_reflow_status(false);
        if needed_reflow {
            self.force_reflow(&*page, ReflowReason::CachedPageNeededReflow);
        }

        let window = page.window().root();
        window.r().thaw();
    }

    /// Handles a mozbrowser event, for example see:
    /// https://developer.mozilla.org/en-US/docs/Web/Events/mozbrowserloadstart
    fn handle_mozbrowser_event_msg(&self,
                                   parent_pipeline_id: PipelineId,
                                   subpage_id: SubpageId,
                                   event: MozBrowserEvent) {
        let borrowed_page = self.root_page();

        let frame_element = borrowed_page.find(parent_pipeline_id).and_then(|page| {
            let doc = page.document().root();
            let doc: JSRef<Node> = NodeCast::from_ref(doc.r());

            doc.traverse_preorder()
               .filter_map(HTMLIFrameElementCast::to_ref)
               .find(|node| node.subpage_id() == Some(subpage_id))
               .map(Temporary::from_rooted)
        }).root();

        if let Some(frame_element) = frame_element {
            frame_element.r().dispatch_mozbrowser_event(event);
        }
    }

    fn handle_update_subpage_id(&self,
                                containing_pipeline_id: PipelineId,
                                old_subpage_id: SubpageId,
                                new_subpage_id: SubpageId) {
        let borrowed_page = self.root_page();

        let frame_element = borrowed_page.find(containing_pipeline_id).and_then(|page| {
            let doc = page.document().root();
            let doc: JSRef<Node> = NodeCast::from_ref(doc.r());

            doc.traverse_preorder()
               .filter_map(HTMLIFrameElementCast::to_ref)
               .find(|node| node.subpage_id() == Some(old_subpage_id))
               .map(Temporary::from_rooted)
        }).root();

        frame_element.unwrap().r().update_subpage_id(new_subpage_id);
    }

    /// Handles a notification that reflow completed.
    fn handle_reflow_complete_msg(&self, pipeline_id: PipelineId, reflow_id: uint) {
        debug!("Script: Reflow {:?} complete for {:?}", reflow_id, pipeline_id);
        let page = self.root_page();
        let page = page.find(pipeline_id).expect(
            "ScriptTask: received a load message for a layout channel that is not associated \
             with this script task. This is a bug.");
        let window = page.window().root();
        window.r().handle_reflow_complete_msg(reflow_id);

        let doc = page.document().root();
        let html_element = doc.r().GetDocumentElement().root();
        let reftest_wait = html_element.r().map_or(false, |elem| elem.has_class(&Atom::from_slice("reftest-wait")));

        if !reftest_wait {
            self.compositor.borrow_mut().set_ready_state(pipeline_id, FinishedLoading);
        }
    }

    /// Window was resized, but this script was not active, so don't reflow yet
    fn handle_resize_inactive_msg(&self, id: PipelineId, new_size: WindowSizeData) {
        let page = self.root_page();
        let page = page.find(id).expect("Received resize message for PipelineId not associated
            with a page in the page tree. This is a bug.");
        let window = page.window().root();
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
        self.compositor.borrow_mut().close();
    }

    /// We have received notification that the response associated with a load has completed.
    /// Kick off the document and frame tree creation process using the result.
    fn handle_page_fetch_complete(&self, id: PipelineId, subpage: Option<SubpageId>,
                                  response: LoadResponse) {
        // Any notification received should refer to an existing, in-progress load that is tracked.
        let idx = self.incomplete_loads.borrow().iter().position(|load| {
            load.pipeline_id == id && load.parent_info.map(|info| info.1) == subpage
        }).unwrap();
        let load = self.incomplete_loads.borrow_mut().remove(idx);
        self.load(response, load);
    }

    /// Handles a request for the window title.
    fn handle_get_title_msg(&self, pipeline_id: PipelineId) {
        let page = get_page(&self.root_page(), pipeline_id);
        let document = page.document().root();
        document.r().send_title_to_compositor();
    }

    /// Handles a request to exit the script task and shut down layout.
    /// Returns true if the script task should shut down and false otherwise.
    fn handle_exit_pipeline_msg(&self, id: PipelineId, exit_type: PipelineExitType) -> bool {
        // If root is being exited, shut down all pages
        let page = self.root_page();
        let window = page.window().root();
        if window.r().pipeline() == id {
            debug!("shutting down layout for root page {:?}", id);
            // To ensure the elements of the DOM tree remain usable (such as the window global),
            // don't free the JS context until all interactions with it are finished.
            shut_down_layout(&page, exit_type);
            *self.js_context.borrow_mut() = None;
            return true
        }

        // otherwise find just the matching page and exit all sub-pages
        if let Some(ref mut child_page) = page.remove(id) {
            shut_down_layout(&*child_page, exit_type);
        }
        return false;
    }

    /// The entry point to document loading. Defines bindings, sets up the window and document
    /// objects, parses HTML and CSS, and kicks off initial layout.
    fn load(&self, response: LoadResponse, incomplete: InProgressLoad) {
        let final_url = response.metadata.final_url.clone();
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
                    let doc = page.document().root();
                    let doc: JSRef<Node> = NodeCast::from_ref(doc.r());

                    doc.traverse_preorder()
                       .filter_map(HTMLIFrameElementCast::to_ref)
                       .find(|node| node.subpage_id() == Some(subpage_id))
                       .map(ElementCast::from_ref)
                       .map(Temporary::from_rooted)
                })
            })
        }).root();

        self.compositor.borrow_mut().set_ready_state(incomplete.pipeline_id, Loading);

        let cx = self.js_context.borrow();
        let cx = cx.as_ref().unwrap();

        // Create a new frame tree entry.
        let page = Rc::new(Page::new(incomplete.pipeline_id, final_url.clone()));
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
        #[unsafe_destructor]
        impl<'a> Drop for AutoPageRemover<'a> {
            fn drop(&mut self) {
                if !self.neutered {
                    match self.page {
                        PageToRemove::Root => *self.script_task.page.borrow_mut() = None,
                        PageToRemove::Child(id) => {
                            let _ = self.script_task.root_page().remove(id);
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

        // Create the window and document objects.
        let window = Window::new(cx.clone(),
                                 page.clone(),
                                 self.chan.clone(),
                                 self.control_chan.clone(),
                                 self.compositor.borrow_mut().dup(),
                                 self.image_cache_task.clone(),
                                 self.resource_task.clone(),
                                 self.storage_task.clone(),
                                 self.devtools_chan.clone(),
                                 self.constellation_chan.clone(),
                                 incomplete.layout_chan,
                                 incomplete.pipeline_id,
                                 incomplete.parent_info,
                                 incomplete.window_size).root();

        let last_modified: Option<DOMString> = response.metadata.headers.as_ref().and_then(|headers| {
            headers.get().map(|&LastModified(ref tm)| dom_last_modified(tm))
        });

        let content_type = match response.metadata.content_type {
            Some((ref t, ref st)) if t.as_slice().eq_ignore_ascii_case("text") &&
                                    st.as_slice().eq_ignore_ascii_case("plain") => {
                Some("text/plain".to_owned())
            }
            _ => None
        };

        let document = Document::new(window.r(),
                                     Some(final_url.clone()),
                                     IsHTMLDocument::HTMLDocument,
                                     content_type,
                                     last_modified,
                                     DocumentSource::FromParser).root();

        window.r().init_browser_context(document.r(), frame_element.r());

        // Create the root frame
        page.set_frame(Some(Frame {
            document: JS::from_rooted(document.r()),
            window: JS::from_rooted(window.r()),
        }));

        let is_javascript = incomplete.url.scheme.as_slice() == "javascript";
        let parse_input = if is_javascript {
            let evalstr = incomplete.url.non_relative_scheme_data().unwrap();
            let jsval = window.r().evaluate_js_on_global_with_result(evalstr);
            let strval = FromJSValConvertible::from_jsval(self.get_cx(), jsval,
                                                          StringificationBehavior::Empty);
            HTMLInput::InputString(strval.unwrap_or("".to_owned()))
        } else {
            HTMLInput::InputUrl(response)
        };

        parse_html(document.r(), parse_input, &final_url, None);

        document.r().set_ready_state(DocumentReadyState::Interactive);
        self.compositor.borrow_mut().set_ready_state(incomplete.pipeline_id, PerformingLayout);

        // Kick off the initial reflow of the page.
        debug!("kicking off initial reflow of {:?}", final_url);
        document.r().content_changed(NodeCast::from_ref(document.r()),
                                     NodeDamage::OtherNodeDamage);
        window.r().reflow(ReflowGoal::ForDisplay, ReflowQueryType::NoQuery, ReflowReason::FirstLoad);

        // No more reflow required
        page.set_reflow_status(false);

        // https://html.spec.whatwg.org/multipage/#the-end step 4
        let addr: Trusted<Document> = Trusted::new(self.get_cx(), document.r(), self.chan.clone());
        let handler = Box::new(DocumentProgressHandler::new(addr.clone(), DocumentProgressTask::DOMContentLoaded));
        self.chan.send(ScriptMsg::RunnableMsg(handler)).unwrap();

        // We have no concept of a document loader right now, so just dispatch the
        // "load" event as soon as we've finished executing all scripts parsed during
        // the initial load.

        // https://html.spec.whatwg.org/multipage/#the-end step 7
        let handler = Box::new(DocumentProgressHandler::new(addr, DocumentProgressTask::Load));
        self.chan.send(ScriptMsg::RunnableMsg(handler)).unwrap();

        window.r().set_fragment_name(final_url.fragment.clone());

        let ConstellationChan(ref chan) = self.constellation_chan;
        chan.send(ConstellationMsg::LoadComplete).unwrap();

        // Notify devtools that a new script global exists.
        match self.devtools_chan {
            None => {}
            Some(ref chan) => {
                let page_info = DevtoolsPageInfo {
                    title: document.r().Title(),
                    url: final_url
                };
                chan.send(DevtoolsControlMsg::NewGlobal(incomplete.pipeline_id,
                                                        self.devtools_sender.clone(),
                                                        page_info)).unwrap();
            }
        }

        page_remover.neuter();
    }

    fn scroll_fragment_point(&self, pipeline_id: PipelineId, node: JSRef<Element>) {
        let node: JSRef<Node> = NodeCast::from_ref(node);
        let rect = node.get_bounding_content_box();
        let point = Point2D(to_frac_px(rect.origin.x).to_f32().unwrap(),
                            to_frac_px(rect.origin.y).to_f32().unwrap());
        // FIXME(#2003, pcwalton): This is pretty bogus when multiple layers are involved.
        // Really what needs to happen is that this needs to go through layout to ask which
        // layer the element belongs to, and have it send the scroll message to the
        // compositor.
        self.compositor.borrow_mut().scroll_fragment_point(pipeline_id, LayerId::null(), point);
    }

    /// Reflows non-incrementally.
    fn force_reflow(&self, page: &Page, reason: ReflowReason) {
        let document = page.document().root();
        document.r().dirty_all_nodes();
        let window = window_from_node(document.r()).root();
        window.r().reflow(ReflowGoal::ForDisplay, ReflowQueryType::NoQuery, reason);
    }

    /// This is the main entry point for receiving and dispatching DOM events.
    ///
    /// TODO: Actually perform DOM event dispatch.
    fn handle_event(&self, pipeline_id: PipelineId, event: CompositorEvent) {
        match event {
            ResizeEvent(new_size) => {
                self.handle_resize_event(pipeline_id, new_size);
            }

            ReflowEvent(nodes) => {
                // FIXME(pcwalton): This event seems to only be used by the image cache task, and
                // the interaction between it and the image holder is really racy. I think that, in
                // order to fix this race, we need to rewrite the image cache task to make the
                // image holder responsible for the lifecycle of image loading instead of having
                // the image holder and layout task both be observers. Then we can have the DOM
                // image element observe the state of the image holder and have it send reflows
                // via the normal dirtying mechanism, and ultimately remove this event.
                //
                // See the implementation of `Width()` and `Height()` in `HTMLImageElement` for
                // fallout of this problem.
                for node in nodes.iter() {
                    let node_to_dirty = node::from_untrusted_node_address(self.js_runtime.ptr,
                                                                          *node).root();
                    let page = get_page(&self.root_page(), pipeline_id);
                    let document = page.document().root();
                    document.r().content_changed(node_to_dirty.r(),
                                                 NodeDamage::OtherNodeDamage);
                }

                self.handle_reflow_event(pipeline_id);
            }

            ClickEvent(_button, point) => {
                let page = get_page(&self.root_page(), pipeline_id);
                let document = page.document().root();
                document.r().handle_click_event(self.js_runtime.ptr, _button, point);
            }

            MouseDownEvent(..) => {}
            MouseUpEvent(..) => {}
            MouseMoveEvent(point) => {
                let page = get_page(&self.root_page(), pipeline_id);
                let document = page.document().root();
                let mouse_over_targets = &mut *self.mouse_over_targets.borrow_mut();

                if document.r().handle_mouse_move_event(self.js_runtime.ptr, point, mouse_over_targets) {
                    self.force_reflow(&page, ReflowReason::MouseEvent)
                }
            }

            KeyEvent(key, state, modifiers) => {
                let page = get_page(&self.root_page(), pipeline_id);
                let document = page.document().root();
                document.r().dispatch_key_event(
                    key, state, modifiers, &mut *self.compositor.borrow_mut());
            }
        }
    }

    /// https://html.spec.whatwg.org/multipage/browsers.html#navigating-across-documents
    /// The entry point for content to notify that a new load has been requested
    /// for the given pipeline (specifically the "navigate" algorithm).
    fn handle_navigate(&self, pipeline_id: PipelineId, subpage_id: Option<SubpageId>, load_data: LoadData) {
        match subpage_id {
            Some(subpage_id) => {
                let borrowed_page = self.root_page();
                let iframe = borrowed_page.find(pipeline_id).and_then(|page| {
                    let doc = page.document().root();
                    let doc: JSRef<Node> = NodeCast::from_ref(doc.r());

                    doc.traverse_preorder()
                       .filter_map(HTMLIFrameElementCast::to_ref)
                       .find(|node| node.subpage_id() == Some(subpage_id))
                       .map(Temporary::from_rooted)
                }).root();
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

    /// The entry point for content to notify that a fragment url has been requested
    /// for the given pipeline.
    fn trigger_fragment(&self, pipeline_id: PipelineId, fragment: String) {
        let page = get_page(&self.root_page(), pipeline_id);
        let document = page.document().root();
        match document.r().find_fragment_node(fragment).root() {
            Some(node) => {
                self.scroll_fragment_point(pipeline_id, node.r());
            }
            None => {}
        }
    }


    fn handle_resize_event(&self, pipeline_id: PipelineId, new_size: WindowSizeData) {
        let page = get_page(&self.root_page(), pipeline_id);
        let window = page.window().root();
        window.r().set_window_size(new_size);
        self.force_reflow(&*page, ReflowReason::WindowResize);

        let document = page.document().root();
        let fragment_node = window.r().steal_fragment_name()
                                      .and_then(|name| document.r().find_fragment_node(name))
                                      .root();
        match fragment_node {
            Some(node) => self.scroll_fragment_point(pipeline_id, node.r()),
            None => {}
        }

        // http://dev.w3.org/csswg/cssom-view/#resizing-viewports
        // https://dvcs.w3.org/hg/dom3events/raw-file/tip/html/DOM3-Events.html#event-type-resize
        let uievent = UIEvent::new(window.r(),
                                   "resize".to_owned(), EventBubbles::DoesNotBubble,
                                   EventCancelable::NotCancelable, Some(window.r()),
                                   0i32).root();
        let event: JSRef<Event> = EventCast::from_ref(uievent.r());

        let wintarget: JSRef<EventTarget> = EventTargetCast::from_ref(window.r());
        event.fire(wintarget);
    }

    fn handle_reflow_event(&self, pipeline_id: PipelineId) {
        debug!("script got reflow event");
        let page = get_page(&self.root_page(), pipeline_id);
        let document = page.document().root();
        let window = window_from_node(document.r()).root();
        window.r().reflow(ReflowGoal::ForDisplay,
                          ReflowQueryType::NoQuery,
                          ReflowReason::ReceivedReflowEvent);
    }

    /// Initiate a non-blocking fetch for a specified resource. Stores the InProgressLoad
    /// argument until a notification is received that the fetch is complete.
    fn start_page_load(&self, incomplete: InProgressLoad, mut load_data: LoadData) {
        let id = incomplete.pipeline_id.clone();
        let subpage = incomplete.parent_info.clone().map(|p| p.1);

        let script_chan = self.chan.clone();
        let resource_task = self.resource_task.clone();

        spawn_named(format!("fetch for {:?}", load_data.url.serialize()), move || {
            if load_data.url.scheme.as_slice() == "javascript" {
                load_data.url = Url::parse("about:blank").unwrap();
            }

            let (input_chan, input_port) = channel();
            resource_task.send(ControlMsg::Load(NetLoadData {
                url: load_data.url,
                method: load_data.method,
                headers: Headers::new(),
                preserved_headers: load_data.headers,
                data: load_data.data,
                cors: None,
                consumer: input_chan,
            })).unwrap();

            let load_response = input_port.recv().unwrap();
            script_chan.send(ScriptMsg::PageFetchComplete(id, subpage, load_response)).unwrap();
        });

        self.incomplete_loads.borrow_mut().push(incomplete);
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
fn shut_down_layout(page_tree: &Rc<Page>, exit_type: PipelineExitType) {
    let mut channels = vec!();

    for page in page_tree.iter() {
        // Tell the layout task to begin shutting down, and wait until it
        // processed this message.
        let (response_chan, response_port) = channel();
        let window = page.window().root();
        let LayoutChan(chan) = window.r().layout_chan();
        if chan.send(layout_interface::Msg::PrepareToExit(response_chan)).is_ok() {
            channels.push(chan);
            response_port.recv().unwrap();
        }
    }

    // Drop our references to the JSContext and DOM objects.
    for page in page_tree.iter() {
        let window = page.window().root();
        window.r().clear_js_context();
        // Sever the connection between the global and the DOM tree
        page.set_frame(None);
    }

    // Destroy the layout task. If there were node leaks, layout will now crash safely.
    for chan in channels.into_iter() {
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
