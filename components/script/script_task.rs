/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

//! The script task is the task that owns the DOM in memory, runs JavaScript, and spawns parsing
//! and layout tasks.

use self::ScriptMsg::*;

use dom::bindings::cell::DOMRefCell;
use dom::bindings::codegen::Bindings::DocumentBinding::{DocumentMethods, DocumentReadyState};
use dom::bindings::codegen::Bindings::EventBinding::EventMethods;
use dom::bindings::codegen::Bindings::EventTargetBinding::EventTargetMethods;
use dom::bindings::codegen::Bindings::WindowBinding::WindowMethods;
use dom::bindings::codegen::InheritTypes::{ElementCast, EventTargetCast, NodeCast, EventCast};
use dom::bindings::conversions::FromJSValConvertible;
use dom::bindings::conversions::StringificationBehavior;
use dom::bindings::global;
use dom::bindings::js::{JS, JSRef, RootCollection, Temporary, OptionalRootable};
use dom::bindings::trace::JSTraceable;
use dom::bindings::utils::{wrap_for_same_compartment, pre_wrap};
use dom::document::{Document, IsHTMLDocument, DocumentHelpers, DocumentSource};
use dom::element::{Element, ElementTypeId, ActivationElementHelpers};
use dom::event::{Event, EventHelpers, EventBubbles, EventCancelable};
use dom::uievent::UIEvent;
use dom::eventtarget::{EventTarget, EventTargetHelpers};
use dom::keyboardevent::KeyboardEvent;
use dom::node::{mod, Node, NodeHelpers, NodeDamage, NodeTypeId};
use dom::window::{Window, WindowHelpers};
use dom::worker::{Worker, TrustedWorkerAddress};
use dom::xmlhttprequest::{TrustedXHRAddress, XMLHttpRequest, XHRProgress};
use parse::html::{HTMLInput, parse_html};
use layout_interface::{ScriptLayoutChan, LayoutChan, NoQuery, ReflowForDisplay};
use layout_interface;
use page::{Page, IterablePage, Frame};
use timers::TimerId;
use devtools;

use devtools_traits::{DevtoolsControlChan, DevtoolsControlPort, NewGlobal, GetRootNode, DevtoolsPageInfo};
use devtools_traits::{DevtoolScriptControlMsg, EvaluateJS, GetDocumentElement};
use devtools_traits::{GetChildren, GetLayout, ModifyAttribute};
use script_traits::{CompositorEvent, ResizeEvent, ReflowEvent, ClickEvent, MouseDownEvent};
use script_traits::{MouseMoveEvent, MouseUpEvent, ConstellationControlMsg, ScriptTaskFactory};
use script_traits::{ResizeMsg, AttachLayoutMsg, GetTitleMsg, KeyEvent, LoadMsg, ViewportMsg};
use script_traits::{ResizeInactiveMsg, ExitPipelineMsg, NewLayoutInfo, OpaqueScriptLayoutChannel};
use script_traits::{ScriptControlChan, ReflowCompleteMsg, SendEventMsg};
use servo_msg::compositor_msg::{FinishedLoading, LayerId, Loading, PerformingLayout};
use servo_msg::compositor_msg::{ScriptListener};
use servo_msg::constellation_msg::{ConstellationChan, LoadCompleteMsg};
use servo_msg::constellation_msg::{LoadData, LoadUrlMsg, NavigationDirection, PipelineId};
use servo_msg::constellation_msg::{Failure, FailureMsg, WindowSizeData, Key, KeyState};
use servo_msg::constellation_msg::{KeyModifiers, SUPER, SHIFT, CONTROL, ALT, Repeated, Pressed};
use servo_msg::constellation_msg::{Released};
use servo_msg::constellation_msg;
use servo_net::image_cache_task::ImageCacheTask;
use servo_net::resource_task::{ResourceTask, Load};
use servo_net::resource_task::LoadData as NetLoadData;
use servo_net::storage_task::StorageTask;
use servo_util::geometry::to_frac_px;
use servo_util::smallvec::SmallVec;
use servo_util::task::spawn_named_with_send_on_failure;
use servo_util::task_state;

use geom::point::Point2D;
use hyper::header::{Header, HeaderFormat};
use hyper::header::common::util as header_util;
use js::jsapi::{JS_SetWrapObjectCallbacks, JS_SetGCZeal, JS_DEFAULT_ZEAL_FREQ, JS_GC};
use js::jsapi::{JSContext, JSRuntime, JSTracer};
use js::jsapi::{JS_SetGCParameter, JSGC_MAX_BYTES};
use js::jsapi::{JS_SetGCCallback, JSGCStatus, JSGC_BEGIN, JSGC_END};
use js::rust::{Cx, RtUtils};
use js;
use url::Url;

use libc::size_t;
use std::any::{Any, AnyRefExt};
use std::comm::{channel, Sender, Receiver, Select};
use std::fmt::{mod, Show};
use std::mem::replace;
use std::rc::Rc;
use std::u32;
use time::{Tm, strptime};

local_data_key!(pub StackRoots: *const RootCollection)

pub enum TimerSource {
    FromWindow(PipelineId),
    FromWorker
}

/// Messages used to control script event loops, such as ScriptTask and
/// DedicatedWorkerGlobalScope.
pub enum ScriptMsg {
    /// Acts on a fragment URL load on the specified pipeline (only dispatched
    /// to ScriptTask).
    TriggerFragmentMsg(PipelineId, Url),
    /// Begins a content-initiated load on the specified pipeline (only
    /// dispatched to ScriptTask).
    TriggerLoadMsg(PipelineId, LoadData),
    /// Instructs the script task to send a navigate message to
    /// the constellation (only dispatched to ScriptTask).
    NavigateMsg(NavigationDirection),
    /// Fires a JavaScript timeout
    /// TimerSource must be FromWindow when dispatched to ScriptTask and
    /// must be FromWorker when dispatched to a DedicatedGlobalWorkerScope
    FireTimerMsg(TimerSource, TimerId),
    /// Notifies the script that a window associated with a particular pipeline
    /// should be closed (only dispatched to ScriptTask).
    ExitWindowMsg(PipelineId),
    /// Notifies the script of progress on a fetch (dispatched to all tasks).
    XHRProgressMsg(TrustedXHRAddress, XHRProgress),
    /// Releases one reference to the XHR object (dispatched to all tasks).
    XHRReleaseMsg(TrustedXHRAddress),
    /// Message sent through Worker.postMessage (only dispatched to
    /// DedicatedWorkerGlobalScope).
    DOMMessage(*mut u64, size_t),
    /// Posts a message to the Worker object (dispatched to all tasks).
    WorkerPostMessage(TrustedWorkerAddress, *mut u64, size_t),
    /// Releases one reference to the Worker object (dispatched to all tasks).
    WorkerRelease(TrustedWorkerAddress),
}

/// Encapsulates internal communication within the script task.
#[deriving(Clone)]
pub struct ScriptChan(pub Sender<ScriptMsg>);

no_jsmanaged_fields!(ScriptChan)

impl ScriptChan {
    /// Creates a new script chan.
    pub fn new() -> (Receiver<ScriptMsg>, ScriptChan) {
        let (chan, port) = channel();
        (port, ScriptChan(chan))
    }
}

pub struct StackRootTLS;

impl StackRootTLS {
    pub fn new(roots: &RootCollection) -> StackRootTLS {
        StackRoots.replace(Some(roots as *const RootCollection));
        StackRootTLS
    }
}

impl Drop for StackRootTLS {
    fn drop(&mut self) {
        let _ = StackRoots.replace(None);
    }
}

/// Information for an entire page. Pages are top-level browsing contexts and can contain multiple
/// frames.
///
/// FIXME: Rename to `Page`, following WebKit?
pub struct ScriptTask {
    /// A handle to the information pertaining to page layout
    page: DOMRefCell<Rc<Page>>,
    /// A handle to the image cache task.
    image_cache_task: ImageCacheTask,
    /// A handle to the resource task.
    resource_task: ResourceTask,

    /// The port on which the script task receives messages (load URL, exit, etc.)
    port: Receiver<ScriptMsg>,
    /// A channel to hand out to script task-based entities that need to be able to enqueue
    /// events in the event queue.
    chan: ScriptChan,

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

    mouse_over_targets: DOMRefCell<Option<Vec<JS<Node>>>>
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
    fn drop(&mut self) {
        match self.owner {
            Some(owner) => {
                let page = owner.page.borrow_mut();
                for page in page.iter() {
                    *page.mut_js_info() = None;
                }
                *owner.js_context.borrow_mut() = None;
            }
            None => (),
        }
    }
}

trait PrivateScriptTaskHelpers {
    fn click_event_filter_by_disabled_state(&self) -> bool;
}

impl<'a> PrivateScriptTaskHelpers for JSRef<'a, Node> {
    fn click_event_filter_by_disabled_state(&self) -> bool {
        match self.type_id() {
            NodeTypeId::Element(ElementTypeId::HTMLButtonElement) |
            NodeTypeId::Element(ElementTypeId::HTMLInputElement) |
            // NodeTypeId::Element(ElementTypeId::HTMLKeygenElement) |
            NodeTypeId::Element(ElementTypeId::HTMLOptionElement) |
            NodeTypeId::Element(ElementTypeId::HTMLSelectElement) |
            NodeTypeId::Element(ElementTypeId::HTMLTextAreaElement) if self.get_disabled_state() => true,
            _ => false
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
                 window_size: WindowSizeData)
                 where C: ScriptListener + Send + 'static {
        let ConstellationChan(const_chan) = constellation_chan.clone();
        let (script_chan, script_port) = channel();
        let layout_chan = LayoutChan(layout_chan.sender());
        spawn_named_with_send_on_failure("ScriptTask", task_state::SCRIPT, proc() {
            let script_task = ScriptTask::new(id,
                                              box compositor as Box<ScriptListener>,
                                              layout_chan,
                                              script_port,
                                              ScriptChan(script_chan),
                                              control_chan,
                                              control_port,
                                              constellation_chan,
                                              resource_task,
                                              storage_task,
                                              image_cache_task,
                                              devtools_chan,
                                              window_size);
            let mut failsafe = ScriptMemoryFailsafe::new(&script_task);
            script_task.start();

            // This must always be the very last operation performed before the task completes
            failsafe.neuter();
        }, FailureMsg(failure_msg), const_chan, false);
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
    pub fn new(id: PipelineId,
               compositor: Box<ScriptListener+'static>,
               layout_chan: LayoutChan,
               port: Receiver<ScriptMsg>,
               chan: ScriptChan,
               control_chan: ScriptControlChan,
               control_port: Receiver<ConstellationControlMsg>,
               constellation_chan: ConstellationChan,
               resource_task: ResourceTask,
               storage_task: StorageTask,
               img_cache_task: ImageCacheTask,
               devtools_chan: Option<DevtoolsControlChan>,
               window_size: WindowSizeData)
               -> ScriptTask {
        let (js_runtime, js_context) = ScriptTask::new_rt_and_cx();
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

        let page = Page::new(id, None, layout_chan, window_size,
                             resource_task.clone(),
                             storage_task,
                             constellation_chan.clone(),
                             js_context.clone());

        let (devtools_sender, devtools_receiver) = channel();
        ScriptTask {
            page: DOMRefCell::new(Rc::new(page)),

            image_cache_task: img_cache_task,
            resource_task: resource_task,

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
            mouse_over_targets: DOMRefCell::new(None)
        }
    }

    pub fn new_rt_and_cx() -> (js::rust::rt, Rc<Cx>) {
        let js_runtime = js::rust::rt();
        assert!({
            let ptr: *mut JSRuntime = (*js_runtime).ptr;
            ptr.is_not_null()
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
            ptr.is_not_null()
        });
        js_context.set_default_options_and_version();
        js_context.set_logging_error_reporter();
        unsafe {
            JS_SetGCZeal((*js_context).ptr, 0, JS_DEFAULT_ZEAL_FREQ);
        }

        // Needed for debug assertions about whether GC is running.
        if !cfg!(ndebug) {
            unsafe {
                JS_SetGCCallback(js_runtime.ptr, Some(debug_gc_callback));
            }
        }

        (js_runtime, js_context)
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
            let page = self.page.borrow_mut();
            for page in page.iter() {
                // Only process a resize if layout is idle.
                let layout_join_port = page.layout_join_port.borrow();
                if layout_join_port.is_none() {
                    let mut resize_event = page.resize_event.get();
                    match resize_event.take() {
                        Some(size) => resizes.push((page.id, size)),
                        None => ()
                    }
                    page.resize_event.set(None);
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
                MixedMessage::FromScript(self.port.recv())
            } else if ret == port2.id() {
                MixedMessage::FromConstellation(self.control_port.recv())
            } else if ret == port3.id() {
                MixedMessage::FromDevtools(self.devtools_port.recv())
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
                MixedMessage::FromConstellation(AttachLayoutMsg(new_layout_info)) => {
                    self.handle_new_layout(new_layout_info);
                }
                MixedMessage::FromConstellation(ResizeMsg(id, size)) => {
                    let page = self.page.borrow_mut();
                    let page = page.find(id).expect("resize sent to nonexistent pipeline");
                    page.resize_event.set(Some(size));
                }
                MixedMessage::FromConstellation(ViewportMsg(id, rect)) => {
                    let page = self.page.borrow_mut();
                    let inner_page = page.find(id).expect("Page rect message sent to nonexistent pipeline");
                    if inner_page.set_page_clip_rect_with_new_viewport(rect) {
                        let page = get_page(&*page, id);
                        self.force_reflow(&*page);
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
                MixedMessage::FromConstellation(ExitPipelineMsg(id)) =>
                    if self.handle_exit_pipeline_msg(id) { return false },
                MixedMessage::FromConstellation(inner_msg) => self.handle_msg_from_constellation(inner_msg),
                MixedMessage::FromScript(inner_msg) => self.handle_msg_from_script(inner_msg),
                MixedMessage::FromDevtools(inner_msg) => self.handle_msg_from_devtools(inner_msg),
            }
        }

        true
    }

    fn handle_msg_from_constellation(&self, msg: ConstellationControlMsg) {
        match msg {
            // TODO(tkuehn) need to handle auxiliary layouts for iframes
            AttachLayoutMsg(_) =>
                panic!("should have handled AttachLayoutMsg already"),
            LoadMsg(id, load_data) =>
                self.load(id, load_data),
            SendEventMsg(id, event) =>
                self.handle_event(id, event),
            ReflowCompleteMsg(id, reflow_id) =>
                self.handle_reflow_complete_msg(id, reflow_id),
            ResizeInactiveMsg(id, new_size) =>
                self.handle_resize_inactive_msg(id, new_size),
            ViewportMsg(..) =>
                panic!("should have handled ViewportMsg already"),
            ResizeMsg(..) =>
                panic!("should have handled ResizeMsg already"),
            ExitPipelineMsg(..) =>
                panic!("should have handled ExitPipelineMsg already"),
            GetTitleMsg(pipeline_id) =>
                self.handle_get_title_msg(pipeline_id),
        }
    }

    fn handle_msg_from_script(&self, msg: ScriptMsg) {
        match msg {
            TriggerLoadMsg(id, load_data) =>
                self.trigger_load(id, load_data),
            TriggerFragmentMsg(id, url) =>
                self.trigger_fragment(id, url),
            FireTimerMsg(TimerSource::FromWindow(id), timer_id) =>
                self.handle_fire_timer_msg(id, timer_id),
            FireTimerMsg(TimerSource::FromWorker, _) =>
                panic!("Worker timeouts must not be sent to script task"),
            NavigateMsg(direction) =>
                self.handle_navigate_msg(direction),
            ExitWindowMsg(id) =>
                self.handle_exit_window_msg(id),
            XHRProgressMsg(addr, progress) =>
                XMLHttpRequest::handle_progress(addr, progress),
            XHRReleaseMsg(addr) =>
                XMLHttpRequest::handle_release(addr),
            DOMMessage(..) =>
                panic!("unexpected message"),
            WorkerPostMessage(addr, data, nbytes) =>
                Worker::handle_message(addr, data, nbytes),
            WorkerRelease(addr) =>
                Worker::handle_release(addr),
        }
    }

    fn handle_msg_from_devtools(&self, msg: DevtoolScriptControlMsg) {
        match msg {
            EvaluateJS(id, s, reply) =>
                devtools::handle_evaluate_js(&*self.page.borrow(), id, s, reply),
            GetRootNode(id, reply) =>
                devtools::handle_get_root_node(&*self.page.borrow(), id, reply),
            GetDocumentElement(id, reply) =>
                devtools::handle_get_document_element(&*self.page.borrow(), id, reply),
            GetChildren(id, node_id, reply) =>
                devtools::handle_get_children(&*self.page.borrow(), id, node_id, reply),
            GetLayout(id, node_id, reply) =>
                devtools::handle_get_layout(&*self.page.borrow(), id, node_id, reply),
            ModifyAttribute(id, node_id, modifications) =>
                devtools::handle_modify_attribute(&*self.page.borrow(), id, node_id, modifications),
        }
    }

    fn handle_new_layout(&self, new_layout_info: NewLayoutInfo) {
        let NewLayoutInfo {
            old_pipeline_id,
            new_pipeline_id,
            subpage_id,
            layout_chan
        } = new_layout_info;

        let page = self.page.borrow_mut();
        let parent_page = page.find(old_pipeline_id).expect("ScriptTask: received a layout
            whose parent has a PipelineId which does not correspond to a pipeline in the script
            task's page tree. This is a bug.");
        let new_page = {
            let window_size = parent_page.window_size.get();
            Page::new(new_pipeline_id, Some(subpage_id),
                      LayoutChan(layout_chan.downcast_ref::<Sender<layout_interface::Msg>>().unwrap().clone()),
                      window_size,
                      parent_page.resource_task.clone(),
                      parent_page.storage_task.clone(),
                      self.constellation_chan.clone(),
                      self.js_context.borrow().as_ref().unwrap().clone())
        };
        parent_page.children.borrow_mut().push(Rc::new(new_page));
    }

    /// Handles a timer that fired.
    fn handle_fire_timer_msg(&self, id: PipelineId, timer_id: TimerId) {
        let page = self.page.borrow_mut();
        let page = page.find(id).expect("ScriptTask: received fire timer msg for a
            pipeline ID not associated with this script task. This is a bug.");
        let frame = page.frame();
        let window = frame.as_ref().unwrap().window.root();
        window.handle_fire_timer(timer_id);
    }

    /// Handles a notification that reflow completed.
    fn handle_reflow_complete_msg(&self, pipeline_id: PipelineId, reflow_id: uint) {
        debug!("Script: Reflow {} complete for {}", reflow_id, pipeline_id);
        let page = self.page.borrow_mut();
        let page = page.find(pipeline_id).expect(
            "ScriptTask: received a load message for a layout channel that is not associated \
             with this script task. This is a bug.");
        let last_reflow_id = page.last_reflow_id.get();
        if last_reflow_id == reflow_id {
            let mut layout_join_port = page.layout_join_port.borrow_mut();
            *layout_join_port = None;
        }

        self.compositor.borrow_mut().set_ready_state(pipeline_id, FinishedLoading);
    }

    /// Handles a navigate forward or backward message.
    /// TODO(tkuehn): is it ever possible to navigate only on a subframe?
    fn handle_navigate_msg(&self, direction: NavigationDirection) {
        let ConstellationChan(ref chan) = self.constellation_chan;
        chan.send(constellation_msg::NavigateMsg(direction));
    }

    /// Window was resized, but this script was not active, so don't reflow yet
    fn handle_resize_inactive_msg(&self, id: PipelineId, new_size: WindowSizeData) {
        let page = self.page.borrow_mut();
        let page = page.find(id).expect("Received resize message for PipelineId not associated
            with a page in the page tree. This is a bug.");
        page.window_size.set(new_size);
        match &mut *page.mut_url() {
            &Some((_, ref mut needs_reflow)) => *needs_reflow = true,
            &None => (),
        }
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

    /// Handles a request for the window title.
    fn handle_get_title_msg(&self, pipeline_id: PipelineId) {
        get_page(&*self.page.borrow(), pipeline_id).send_title_to_compositor();
    }

    /// Handles a request to exit the script task and shut down layout.
    /// Returns true if the script task should shut down and false otherwise.
    fn handle_exit_pipeline_msg(&self, id: PipelineId) -> bool {
        // If root is being exited, shut down all pages
        let page = self.page.borrow_mut();
        if page.id == id {
            debug!("shutting down layout for root page {}", id);
            *self.js_context.borrow_mut() = None;
            shut_down_layout(&*page, (*self.js_runtime).ptr);
            return true
        }

        // otherwise find just the matching page and exit all sub-pages
        match page.remove(id) {
            Some(ref mut page) => {
                shut_down_layout(&*page, (*self.js_runtime).ptr);
                false
            }
            // TODO(tkuehn): pipeline closing is currently duplicated across
            // script and constellation, which can cause this to happen. Constellation
            // needs to be smarter about exiting pipelines.
            None => false,
        }

    }

    /// The entry point to document loading. Defines bindings, sets up the window and document
    /// objects, parses HTML and CSS, and kicks off initial layout.
    fn load(&self, pipeline_id: PipelineId, load_data: LoadData) {
        let url = load_data.url.clone();
        debug!("ScriptTask: loading {} on page {}", url, pipeline_id);

        let page = self.page.borrow_mut();
        let page = page.find(pipeline_id).expect("ScriptTask: received a load
            message for a layout channel that is not associated with this script task. This
            is a bug.");

        let last_url = match &mut *page.mut_url() {
            &Some((ref mut loaded, ref mut needs_reflow)) if *loaded == url => {
                if replace(needs_reflow, false) {
                    self.force_reflow(&*page);
                }
                return;
            },
            url => replace(url, None).map(|(loaded, _)| loaded),
        };

        let is_javascript = url.scheme.as_slice() == "javascript";

        let cx = self.js_context.borrow();
        let cx = cx.as_ref().unwrap();
        // Create the window and document objects.
        let window = Window::new(cx.ptr,
                                 page.clone(),
                                 self.chan.clone(),
                                 self.control_chan.clone(),
                                 self.compositor.borrow_mut().dup(),
                                 self.image_cache_task.clone()).root();
        let doc_url = if is_javascript {
            let doc_url = last_url.unwrap_or_else(|| {
                Url::parse("about:blank").unwrap()
            });
            *page.mut_url() = Some((doc_url.clone(), true));
            doc_url
        } else {
            url.clone()
        };
        let document = Document::new(*window, Some(doc_url.clone()),
                                     IsHTMLDocument::HTMLDocument, None,
                                     DocumentSource::FromParser).root();

        window.init_browser_context(*document);

        self.compositor.borrow_mut().set_ready_state(pipeline_id, Loading);

        {
            // Create the root frame.
            let mut frame = page.mut_frame();
            *frame = Some(Frame {
                document: JS::from_rooted(*document),
                window: JS::from_rooted(*window),
            });
        }

        let (parser_input, final_url) = if !is_javascript {
            // Wait for the LoadResponse so that the parser knows the final URL.
            let (input_chan, input_port) = channel();
            self.resource_task.send(Load(NetLoadData {
                url: url,
                method: load_data.method,
                headers: load_data.headers,
                data: load_data.data,
                cors: None,
                consumer: input_chan,
            }));

            let load_response = input_port.recv();

            load_response.metadata.headers.as_ref().map(|headers| {
                headers.get().map(|&LastModified(ref tm)| {
                    document.set_last_modified(dom_last_modified(tm));
                });
            });

            let final_url = load_response.metadata.final_url.clone();

            {
                // Store the final URL before we start parsing, so that DOM routines
                // (e.g. HTMLImageElement::update_image) can resolve relative URLs
                // correctly.
                *page.mut_url() = Some((final_url.clone(), true));
            }

            (HTMLInput::InputUrl(load_response), final_url)
        } else {
            let evalstr = load_data.url.non_relative_scheme_data().unwrap();
            let jsval = window.evaluate_js_with_result(evalstr);
            let strval = FromJSValConvertible::from_jsval(self.get_cx(), jsval,
                                                          StringificationBehavior::Empty);
            (HTMLInput::InputString(strval.unwrap_or("".to_string())), doc_url)
        };

        parse_html(*document, parser_input, &final_url);

        document.set_ready_state(DocumentReadyState::Interactive);
        self.compositor.borrow_mut().set_ready_state(pipeline_id, PerformingLayout);

        // Kick off the initial reflow of the page.
        debug!("kicking off initial reflow of {}", final_url);
        {
            let document_js_ref = (&*document).clone();
            let document_as_node = NodeCast::from_ref(document_js_ref);
            document.content_changed(document_as_node, NodeDamage::OtherNodeDamage);
        }
        window.flush_layout(ReflowForDisplay, NoQuery);

        {
            // No more reflow required
            let mut page_url = page.mut_url();
            *page_url = Some((final_url.clone(), false));
        }

        // https://html.spec.whatwg.org/multipage/#the-end step 4
        let event = Event::new(global::Window(*window), "DOMContentLoaded".to_string(),
                               EventBubbles::DoesNotBubble,
                               EventCancelable::NotCancelable).root();
        let doctarget: JSRef<EventTarget> = EventTargetCast::from_ref(*document);
        let _ = doctarget.DispatchEvent(*event);

        // We have no concept of a document loader right now, so just dispatch the
        // "load" event as soon as we've finished executing all scripts parsed during
        // the initial load.

        // https://html.spec.whatwg.org/multipage/#the-end step 7
        document.set_ready_state(DocumentReadyState::Complete);

        let event = Event::new(global::Window(*window), "load".to_string(),
                               EventBubbles::DoesNotBubble,
                               EventCancelable::NotCancelable).root();
        let wintarget: JSRef<EventTarget> = EventTargetCast::from_ref(*window);
        let _ = wintarget.dispatch_event_with_target(Some(doctarget), *event);

        *page.fragment_name.borrow_mut() = final_url.fragment.clone();

        let ConstellationChan(ref chan) = self.constellation_chan;
        chan.send(LoadCompleteMsg);

        // Notify devtools that a new script global exists.
        match self.devtools_chan {
            None => {}
            Some(ref chan) => {
                let page_info = DevtoolsPageInfo {
                    title: document.Title(),
                    url: final_url
                };
                chan.send(NewGlobal(pipeline_id, self.devtools_sender.clone(),
                                    page_info));
            }
        }
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
    fn force_reflow(&self, page: &Page) {
        page.dirty_all_nodes();
        page.reflow(ReflowForDisplay,
                    self.control_chan.clone(),
                    &mut **self.compositor.borrow_mut(),
                    NoQuery);
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
                    let page = get_page(&*self.page.borrow(), pipeline_id);
                    let frame = page.frame();
                    let document = frame.as_ref().unwrap().document.root();
                    document.content_changed(*node_to_dirty, NodeDamage::OtherNodeDamage);
                }

                self.handle_reflow_event(pipeline_id);
            }

            ClickEvent(_button, point) => {
              self.handle_click_event(pipeline_id, _button, point);
            }

            MouseDownEvent(..) => {}
            MouseUpEvent(..) => {}
            MouseMoveEvent(point) => {
              self.handle_mouse_move_event(pipeline_id, point);
            }

            KeyEvent(key, state, modifiers) => {
                self.dispatch_key_event(key, state, modifiers, pipeline_id);
            }
        }
    }

    /// The entry point for all key processing for web content
    fn dispatch_key_event(&self, key: Key,
                          state: KeyState,
                          modifiers: KeyModifiers,
                          pipeline_id: PipelineId) {
        let page = get_page(&*self.page.borrow(), pipeline_id);
        let frame = page.frame();
        let window = frame.as_ref().unwrap().window.root();
        let doc = window.Document().root();
        let focused = doc.get_focused_element().root();
        let body = doc.GetBody().root();

        let target: JSRef<EventTarget> = match (&focused, &body) {
            (&Some(ref focused), _) => EventTargetCast::from_ref(**focused),
            (&None, &Some(ref body)) => EventTargetCast::from_ref(**body),
            (&None, &None) => EventTargetCast::from_ref(*window),
        };

        let ctrl = modifiers.contains(CONTROL);
        let alt = modifiers.contains(ALT);
        let shift = modifiers.contains(SHIFT);
        let meta = modifiers.contains(SUPER);

        let is_composing = false;
        let is_repeating = state == KeyState::Repeated;
        let ev_type = match state {
            KeyState::Pressed | KeyState::Repeated => "keydown",
            KeyState::Released => "keyup",
        }.to_string();

        let props = KeyboardEvent::key_properties(key, modifiers);

        let keyevent = KeyboardEvent::new(*window, ev_type, true, true, Some(*window), 0,
                                          props.key.to_string(), props.code.to_string(),
                                          props.location, is_repeating, is_composing,
                                          ctrl, alt, shift, meta,
                                          None, props.key_code).root();
        let event = EventCast::from_ref(*keyevent);
        let _ = target.DispatchEvent(event);
        let mut prevented = event.DefaultPrevented();

        // https://dvcs.w3.org/hg/dom3events/raw-file/tip/html/DOM3-Events.html#keys-cancelable-keys
        if state != KeyState::Released && props.is_printable() && !prevented {
            // https://dvcs.w3.org/hg/dom3events/raw-file/tip/html/DOM3-Events.html#keypress-event-order
            let event = KeyboardEvent::new(*window, "keypress".to_string(), true, true, Some(*window),
                                           0, props.key.to_string(), props.code.to_string(),
                                           props.location, is_repeating, is_composing,
                                           ctrl, alt, shift, meta,
                                           props.char_code, 0).root();
            let _ = target.DispatchEvent(EventCast::from_ref(*event));

            let ev = EventCast::from_ref(*event);
            prevented = ev.DefaultPrevented();
            // TODO: if keypress event is canceled, prevent firing input events
        }

        if !prevented {
            self.compositor.borrow_mut().send_key_event(key, state, modifiers);
        }

        // This behavior is unspecced
        // We are supposed to dispatch synthetic click activation for Space and/or Return,
        // however *when* we do it is up to us
        // I'm dispatching it after the key event so the script has a chance to cancel it
        // https://www.w3.org/Bugs/Public/show_bug.cgi?id=27337
        match key {
            Key::Space if !prevented && state == KeyState::Released => {
                let maybe_elem: Option<JSRef<Element>> = ElementCast::to_ref(target);
                maybe_elem.map(|el| el.as_maybe_activatable().map(|a| a.synthetic_click_activation(ctrl, alt, shift, meta)));
            }
            Key::Enter if !prevented && state == KeyState::Released => {
                let maybe_elem: Option<JSRef<Element>> = ElementCast::to_ref(target);
                maybe_elem.map(|el| el.as_maybe_activatable().map(|a| a.implicit_submission(ctrl, alt, shift, meta)));
            }
            _ => ()
        }

        window.flush_layout(ReflowForDisplay, NoQuery);
    }

    /// The entry point for content to notify that a new load has been requested
    /// for the given pipeline.
    fn trigger_load(&self, pipeline_id: PipelineId, load_data: LoadData) {
        let ConstellationChan(ref const_chan) = self.constellation_chan;
        const_chan.send(LoadUrlMsg(pipeline_id, load_data));
    }

    /// The entry point for content to notify that a fragment url has been requested
    /// for the given pipeline.
    fn trigger_fragment(&self, pipeline_id: PipelineId, url: Url) {
        let page = get_page(&*self.page.borrow(), pipeline_id);
        match page.find_fragment_node(url.fragment.unwrap()).root() {
            Some(node) => {
                self.scroll_fragment_point(pipeline_id, *node);
            }
            None => {}
        }
    }


    fn handle_resize_event(&self, pipeline_id: PipelineId, new_size: WindowSizeData) {
        let window = {
            let page = get_page(&*self.page.borrow(), pipeline_id);
            page.window_size.set(new_size);

            let frame = page.frame();
            if frame.is_some() {
                self.force_reflow(&*page);
            }

            let fragment_node =
                page.fragment_name
                    .borrow_mut()
                    .take()
                    .and_then(|name| page.find_fragment_node(name))
                    .root();
            match fragment_node {
                Some(node) => self.scroll_fragment_point(pipeline_id, *node),
                None => {}
            }

            frame.as_ref().map(|frame| Temporary::new(frame.window.clone()))
        };

        match window.root() {
            Some(window) => {
                // http://dev.w3.org/csswg/cssom-view/#resizing-viewports
                // https://dvcs.w3.org/hg/dom3events/raw-file/tip/html/DOM3-Events.html#event-type-resize
                let uievent = UIEvent::new(window.clone(),
                                           "resize".to_string(), false,
                                           false, Some(window.clone()),
                                           0i32).root();
                let event: JSRef<Event> = EventCast::from_ref(*uievent);

                let wintarget: JSRef<EventTarget> = EventTargetCast::from_ref(*window);
                wintarget.dispatch_event(event);
            }
            None => ()
        }
    }

    fn handle_reflow_event(&self, pipeline_id: PipelineId) {
        debug!("script got reflow event");
        let page = get_page(&*self.page.borrow(), pipeline_id);
        let frame = page.frame();
        if frame.is_some() {
            self.force_reflow(&*page);
        }
    }

    fn handle_click_event(&self, pipeline_id: PipelineId, _button: uint, point: Point2D<f32>) {
        debug!("ClickEvent: clicked at {}", point);
        let page = get_page(&*self.page.borrow(), pipeline_id);
        match page.hit_test(&point) {
            Some(node_address) => {
                debug!("node address is {}", node_address);

                let temp_node =
                        node::from_untrusted_node_address(
                            self.js_runtime.ptr, node_address).root();

                let maybe_node = if !temp_node.is_element() {
                    temp_node.ancestors().find(|node| node.is_element())
                } else {
                    Some(*temp_node)
                };

                match maybe_node {
                    Some(node) => {
                        debug!("clicked on {:s}", node.debug_str());
                        // Prevent click event if form control element is disabled.
                        if node.click_event_filter_by_disabled_state() { return; }
                        match *page.frame() {
                            Some(ref frame) => {
                                let window = frame.window.root();
                                let doc = window.Document().root();
                                doc.begin_focus_transaction();

                                let event =
                                    Event::new(global::Window(*window),
                                               "click".to_string(),
                                               EventBubbles::Bubbles,
                                               EventCancelable::Cancelable).root();
                                // https://dvcs.w3.org/hg/dom3events/raw-file/tip/html/DOM3-Events.html#trusted-events
                                event.set_trusted(true);
                                // https://html.spec.whatwg.org/multipage/interaction.html#run-authentic-click-activation-steps
                                let el = ElementCast::to_ref(node).unwrap(); // is_element() check already exists above
                                el.authentic_click_activation(*event);

                                doc.commit_focus_transaction();
                                window.flush_layout(ReflowForDisplay, NoQuery);
                            }
                            None => {}
                        }
                    }
                    None => {}
                }
            }

            None => {}
        }
    }


    fn handle_mouse_move_event(&self, pipeline_id: PipelineId, point: Point2D<f32>) {
        let page = get_page(&*self.page.borrow(), pipeline_id);
        match page.get_nodes_under_mouse(&point) {
            Some(node_address) => {

                let mut target_list = vec!();
                let mut target_compare = false;

                let mouse_over_targets = &mut *self.mouse_over_targets.borrow_mut();
                match *mouse_over_targets {
                    Some(ref mut mouse_over_targets) => {
                        for node in mouse_over_targets.iter_mut() {
                            let node = node.root();
                            node.set_hover_state(false);
                        }
                    }
                    None => {}
                }

                for node_address in node_address.iter() {

                    let temp_node =
                        node::from_untrusted_node_address(
                            self.js_runtime.ptr, *node_address);

                    let maybe_node = temp_node.root().ancestors().find(|node| node.is_element());
                    match maybe_node {
                        Some(node) => {
                            node.set_hover_state(true);

                            match *mouse_over_targets {
                                Some(ref mouse_over_targets) => {
                                    if !target_compare {
                                        target_compare = !mouse_over_targets.contains(&JS::from_rooted(node));
                                    }
                                }
                                None => {}
                            }
                            target_list.push(JS::from_rooted(node));
                        }
                        None => {}
                    }
                }
                match *mouse_over_targets {
                    Some(ref mouse_over_targets) => {
                        if mouse_over_targets.len() != target_list.len() {
                            target_compare = true;
                        }
                    }
                    None => { target_compare = true; }
                }

                if target_compare {
                    if mouse_over_targets.is_some() {
                        self.force_reflow(&*page);
                    }
                    *mouse_over_targets = Some(target_list);
                }
            }

            None => {}
        }
    }
}

/// Shuts down layout for the given page tree.
fn shut_down_layout(page_tree: &Rc<Page>, rt: *mut JSRuntime) {
    for page in page_tree.iter() {
        // Tell the layout task to begin shutting down, and wait until it
        // processed this message.
        let (response_chan, response_port) = channel();
        let LayoutChan(ref chan) = page.layout_chan;
        chan.send(layout_interface::PrepareToExitMsg(response_chan));
        response_port.recv();
    }

    // Remove our references to the DOM objects in this page tree.
    for page in page_tree.iter() {
        *page.mut_frame() = None;
    }

    // Drop our references to the JSContext, potentially triggering a GC.
    for page in page_tree.iter() {
        *page.mut_js_info() = None;
    }

    // Force a GC to make sure that our DOM reflectors are released before we tell
    // layout to exit.
    unsafe {
        JS_GC(rt);
    }

    // Destroy the layout task. If there were node leaks, layout will now crash safely.
    for page in page_tree.iter() {
        let LayoutChan(ref chan) = page.layout_chan;
        chan.send(layout_interface::ExitNowMsg);
    }
}


pub fn get_page(page: &Rc<Page>, pipeline_id: PipelineId) -> Rc<Page> {
    page.find(pipeline_id).expect("ScriptTask: received an event \
        message for a layout channel that is not associated with this script task.\
         This is a bug.")
}

//FIXME(seanmonstar): uplift to Hyper
#[deriving(Clone)]
struct LastModified(pub Tm);

impl Header for LastModified {
    #[inline]
    fn header_name(_: Option<LastModified>) -> &'static str {
        "Last-Modified"
    }

    // Parses an RFC 2616 compliant date/time string,
    fn parse_header(raw: &[Vec<u8>]) -> Option<LastModified> {
        header_util::from_one_raw_str(raw).and_then(|s: String| {
            let s = s.as_slice();
            strptime(s, "%a, %d %b %Y %T %Z").or_else(|_| {
                strptime(s, "%A, %d-%b-%y %T %Z")
            }).or_else(|_| {
                strptime(s, "%c")
            }).ok().map(|tm| LastModified(tm))
        })
    }
}

impl HeaderFormat for LastModified {
    // a localized date/time string in a format suitable
    // for document.lastModified.
    fn fmt_header(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let LastModified(ref tm) = *self;
        match tm.tm_utcoff {
            0 => tm.rfc822().fmt(f),
            _ => tm.to_utc().rfc822().fmt(f)
        }
    }
}

fn dom_last_modified(tm: &Tm) -> String {
    format!("{}", tm.to_local().strftime("%m/%d/%Y %H:%M:%S").unwrap())
}
