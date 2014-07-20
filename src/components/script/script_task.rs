/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

//! The script task is the task that owns the DOM in memory, runs JavaScript, and spawns parsing
//! and layout tasks.

use dom::bindings::codegen::InheritTypes::{EventTargetCast, NodeCast, EventCast};
use dom::bindings::global::Window;
use dom::bindings::js::{JS, JSRef, RootCollection, Temporary, OptionalSettable};
use dom::bindings::js::OptionalRootable;
use dom::bindings::utils::Reflectable;
use dom::bindings::utils::{wrap_for_same_compartment, pre_wrap};
use dom::document::{Document, HTMLDocument, DocumentHelpers};
use dom::element::{Element};
use dom::event::{Event_, ResizeEvent, ReflowEvent, ClickEvent, MouseDownEvent, MouseMoveEvent, MouseUpEvent};
use dom::event::Event;
use dom::uievent::UIEvent;
use dom::eventtarget::{EventTarget, EventTargetHelpers};
use dom::node;
use dom::node::{Node, NodeHelpers};
use dom::window::{TimerId, Window, WindowHelpers};
use dom::xmlhttprequest::{TrustedXHRAddress, XMLHttpRequest, XHRProgress};
use html::hubbub_html_parser::HtmlParserResult;
use html::hubbub_html_parser::{HtmlDiscoveredStyle, HtmlDiscoveredScript};
use html::hubbub_html_parser;
use layout_interface::AddStylesheetMsg;
use layout_interface::{LayoutChan, MatchSelectorsDocumentDamage};
use layout_interface::{ReflowDocumentDamage, ReflowForDisplay};
use layout_interface::ContentChangedDocumentDamage;
use layout_interface;
use page::{Page, IterablePage, Frame};

use geom::point::Point2D;
use js::jsapi::JS_CallFunctionValue;
use js::jsapi::{JS_SetWrapObjectCallbacks, JS_SetGCZeal, JS_DEFAULT_ZEAL_FREQ, JS_GC};
use js::jsapi::{JSContext, JSRuntime};
use js::jsval::NullValue;
use js::rust::{Cx, RtUtils};
use js::rust::with_compartment;
use js;
use servo_msg::compositor_msg::{FinishedLoading, LayerId, Loading};
use servo_msg::compositor_msg::{ScriptListener};
use servo_msg::constellation_msg::{ConstellationChan, LoadCompleteMsg, LoadUrlMsg, NavigationDirection};
use servo_msg::constellation_msg::{PipelineId, SubpageId, Failure, FailureMsg, WindowSizeData};
use servo_msg::constellation_msg;
use servo_net::image_cache_task::ImageCacheTask;
use servo_net::resource_task::ResourceTask;
use servo_util::geometry::to_frac_px;
use servo_util::task::send_on_failure;
use std::cell::RefCell;
use std::comm::{channel, Sender, Receiver};
use std::mem::replace;
use std::ptr;
use std::rc::Rc;
use std::task::TaskBuilder;
use url::Url;

use serialize::{Encoder, Encodable};

local_data_key!(pub StackRoots: *RootCollection)

/// Messages used to control the script task.
pub enum ScriptMsg {
    /// Loads a new URL on the specified pipeline.
    LoadMsg(PipelineId, Url),
    /// Acts on a fragment URL load on the specified pipeline.
    TriggerFragmentMsg(PipelineId, Url),
    /// Begins a content-initiated load on the specified pipeline.
    TriggerLoadMsg(PipelineId, Url),
    /// Gives a channel and ID to a layout task, as well as the ID of that layout's parent
    AttachLayoutMsg(NewLayoutInfo),
    /// Instructs the script task to send a navigate message to the constellation.
    NavigateMsg(NavigationDirection),
    /// Sends a DOM event.
    SendEventMsg(PipelineId, Event_),
    /// Window resized.  Sends a DOM event eventually, but first we combine events.
    ResizeMsg(PipelineId, WindowSizeData),
    /// Fires a JavaScript timeout.
    FireTimerMsg(PipelineId, TimerId),
    /// Notifies script that reflow is finished.
    ReflowCompleteMsg(PipelineId, uint),
    /// Notifies script that window has been resized but to not take immediate action.
    ResizeInactiveMsg(PipelineId, WindowSizeData),
    /// Notifies the script that a pipeline should be closed.
    ExitPipelineMsg(PipelineId),
    /// Notifies the script that a window associated with a particular pipeline should be closed.
    ExitWindowMsg(PipelineId),
    /// Notifies the script of progress on a fetch
    XHRProgressMsg(TrustedXHRAddress, XHRProgress)
}

pub struct NewLayoutInfo {
    pub old_pipeline_id: PipelineId,
    pub new_pipeline_id: PipelineId,
    pub subpage_id: SubpageId,
    pub layout_chan: LayoutChan,
}

/// Encapsulates external communication with the script task.
#[deriving(Clone)]
pub struct ScriptChan(pub Sender<ScriptMsg>);

impl<S: Encoder<E>, E> Encodable<S, E> for ScriptChan {
    fn encode(&self, _s: &mut S) -> Result<(), E> {
        Ok(())
    }
}

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
        StackRoots.replace(Some(roots as *RootCollection));
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
    page: RefCell<Rc<Page>>,
    /// A handle to the image cache task.
    image_cache_task: ImageCacheTask,
    /// A handle to the resource task.
    resource_task: ResourceTask,

    /// The port on which the script task receives messages (load URL, exit, etc.)
    port: Receiver<ScriptMsg>,
    /// A channel to hand out when some other task needs to be able to respond to a message from
    /// the script task.
    chan: ScriptChan,

    /// For communicating load url messages to the constellation
    constellation_chan: ConstellationChan,
    /// A handle to the compositor for communicating ready state messages.
    compositor: Box<ScriptListener>,

    /// The JavaScript runtime.
    js_runtime: js::rust::rt,
    /// The JSContext.
    js_context: RefCell<Option<Rc<Cx>>>,

    mouse_over_targets: RefCell<Option<Vec<JS<Node>>>>
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
                let mut page = owner.page.borrow_mut();
                for page in page.iter() {
                    *page.mut_js_info() = None;
                }
                *owner.js_context.borrow_mut() = None;
            }
            None => (),
        }
    }
}

impl ScriptTask {
    /// Creates a new script task.
    pub fn new(id: PipelineId,
               compositor: Box<ScriptListener>,
               layout_chan: LayoutChan,
               port: Receiver<ScriptMsg>,
               chan: ScriptChan,
               constellation_chan: ConstellationChan,
               resource_task: ResourceTask,
               img_cache_task: ImageCacheTask,
               window_size: WindowSizeData)
               -> Rc<ScriptTask> {
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
                             constellation_chan.clone(),
                             js_context.clone());
        Rc::new(ScriptTask {
            page: RefCell::new(Rc::new(page)),

            image_cache_task: img_cache_task,
            resource_task: resource_task,

            port: port,
            chan: chan,
            constellation_chan: constellation_chan,
            compositor: compositor,

            js_runtime: js_runtime,
            js_context: RefCell::new(Some(js_context)),
            mouse_over_targets: RefCell::new(None)
        })
    }

    pub fn new_rt_and_cx() -> (js::rust::rt, Rc<Cx>) {
        let js_runtime = js::rust::rt();
        assert!({
            let ptr: *mut JSRuntime = (*js_runtime).ptr;
            ptr.is_not_null()
        });

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

        (js_runtime, js_context)
    }

    pub fn get_cx(&self) -> *mut JSContext {
        (**self.js_context.borrow().get_ref()).ptr
    }

    /// Starts the script task. After calling this method, the script task will loop receiving
    /// messages on its port.
    pub fn start(&self) {
        while self.handle_msgs() {
            // Go on...
        }
    }

    pub fn create<C:ScriptListener + Send>(
                  id: PipelineId,
                  compositor: Box<C>,
                  layout_chan: LayoutChan,
                  port: Receiver<ScriptMsg>,
                  chan: ScriptChan,
                  constellation_chan: ConstellationChan,
                  failure_msg: Failure,
                  resource_task: ResourceTask,
                  image_cache_task: ImageCacheTask,
                  window_size: WindowSizeData) {
        let mut builder = TaskBuilder::new().named("ScriptTask");
        let ConstellationChan(const_chan) = constellation_chan.clone();
        send_on_failure(&mut builder, FailureMsg(failure_msg), const_chan);
        builder.spawn(proc() {
            let script_task = ScriptTask::new(id,
                                              compositor as Box<ScriptListener>,
                                              layout_chan,
                                              port,
                                              chan,
                                              constellation_chan,
                                              resource_task,
                                              image_cache_task,
                                              window_size);
            let mut failsafe = ScriptMemoryFailsafe::new(&*script_task);
            script_task.start();

            // This must always be the very last operation performed before the task completes
            failsafe.neuter();
        });
    }

    /// Handle incoming control messages.
    fn handle_msgs(&self) -> bool {
        let roots = RootCollection::new();
        let _stack_roots_tls = StackRootTLS::new(&roots);

        // Handle pending resize events.
        // Gather them first to avoid a double mut borrow on self.
        let mut resizes = vec!();

        {
            let mut page = self.page.borrow_mut();
            for page in page.iter() {
                // Only process a resize if layout is idle.
                let layout_join_port = page.layout_join_port.deref().borrow();
                if layout_join_port.is_none() {
                    let mut resize_event = page.resize_event.deref().get();
                    match resize_event.take() {
                        Some(size) => resizes.push((page.id, size)),
                        None => ()
                    }
                    page.resize_event.deref().set(None);
                }
            }
        }

        for (id, size) in resizes.move_iter() {
            self.handle_event(id, ResizeEvent(size));
        }

        // Store new resizes, and gather all other events.
        let mut sequential = vec!();

        // Receive at least one message so we don't spinloop.
        let mut event = self.port.recv();

        loop {
            match event {
                ResizeMsg(id, size) => {
                    let mut page = self.page.borrow_mut();
                    let page = page.find(id).expect("resize sent to nonexistent pipeline");
                    page.resize_event.deref().set(Some(size));
                }
                _ => {
                    sequential.push(event);
                }
            }

            match self.port.try_recv() {
                Err(_) => break,
                Ok(ev) => event = ev,
            }
        }

        // Process the gathered events.
        for msg in sequential.move_iter() {
            match msg {
                // TODO(tkuehn) need to handle auxiliary layouts for iframes
                AttachLayoutMsg(new_layout_info) => self.handle_new_layout(new_layout_info),
                LoadMsg(id, url) => self.load(id, url),
                TriggerLoadMsg(id, url) => self.trigger_load(id, url),
                TriggerFragmentMsg(id, url) => self.trigger_fragment(id, url),
                SendEventMsg(id, event) => self.handle_event(id, event),
                FireTimerMsg(id, timer_id) => self.handle_fire_timer_msg(id, timer_id),
                NavigateMsg(direction) => self.handle_navigate_msg(direction),
                ReflowCompleteMsg(id, reflow_id) => self.handle_reflow_complete_msg(id, reflow_id),
                ResizeInactiveMsg(id, new_size) => self.handle_resize_inactive_msg(id, new_size),
                ExitPipelineMsg(id) => if self.handle_exit_pipeline_msg(id) { return false },
                ExitWindowMsg(id) => self.handle_exit_window_msg(id),
                ResizeMsg(..) => fail!("should have handled ResizeMsg already"),
                XHRProgressMsg(addr, progress) => XMLHttpRequest::handle_xhr_progress(addr, progress),
            }
        }

        true
    }

    fn handle_new_layout(&self, new_layout_info: NewLayoutInfo) {
        debug!("Script: new layout: {:?}", new_layout_info);
        let NewLayoutInfo {
            old_pipeline_id,
            new_pipeline_id,
            subpage_id,
            layout_chan
        } = new_layout_info;

        let mut page = self.page.borrow_mut();
        let parent_page = page.find(old_pipeline_id).expect("ScriptTask: received a layout
            whose parent has a PipelineId which does not correspond to a pipeline in the script
            task's page tree. This is a bug.");
        let new_page = {
            let window_size = parent_page.window_size.deref().get();
            Page::new(new_pipeline_id, Some(subpage_id), layout_chan, window_size,
                      parent_page.resource_task.deref().clone(),
                      self.constellation_chan.clone(),
                      self.js_context.borrow().get_ref().clone())
        };
        parent_page.children.deref().borrow_mut().push(Rc::new(new_page));
    }

    /// Handles a timer that fired.
    fn handle_fire_timer_msg(&self, id: PipelineId, timer_id: TimerId) {
        let mut page = self.page.borrow_mut();
        let page = page.find(id).expect("ScriptTask: received fire timer msg for a
            pipeline ID not associated with this script task. This is a bug.");
        let frame = page.frame();
        let window = frame.get_ref().window.root();

        let this_value = window.deref().reflector().get_jsobject();

        let data = match window.deref().active_timers.deref().borrow().find(&timer_id) {
            None => return,
            Some(timer_handle) => timer_handle.data,
        };

        // TODO: Support extra arguments. This requires passing a `*JSVal` array as `argv`.
        let cx = self.get_cx();
        with_compartment(cx, this_value, || {
            let mut rval = NullValue();
            unsafe {
                JS_CallFunctionValue(cx, this_value, *data.funval,
                                     0, ptr::mut_null(), &mut rval);
            }
        });

        if !data.is_interval {
            window.deref().active_timers.deref().borrow_mut().remove(&timer_id);
        }
    }

    /// Handles a notification that reflow completed.
    fn handle_reflow_complete_msg(&self, pipeline_id: PipelineId, reflow_id: uint) {
        debug!("Script: Reflow {:?} complete for {:?}", reflow_id, pipeline_id);
        let mut page = self.page.borrow_mut();
        let page = page.find(pipeline_id).expect(
            "ScriptTask: received a load message for a layout channel that is not associated \
             with this script task. This is a bug.");
        let last_reflow_id = page.last_reflow_id.deref().get();
        if last_reflow_id == reflow_id {
            let mut layout_join_port = page.layout_join_port.deref().borrow_mut();
            *layout_join_port = None;
        }
        self.compositor.set_ready_state(FinishedLoading);
    }

    /// Handles a navigate forward or backward message.
    /// TODO(tkuehn): is it ever possible to navigate only on a subframe?
    fn handle_navigate_msg(&self, direction: NavigationDirection) {
        let ConstellationChan(ref chan) = self.constellation_chan;
        chan.send(constellation_msg::NavigateMsg(direction));
    }

    /// Window was resized, but this script was not active, so don't reflow yet
    fn handle_resize_inactive_msg(&self, id: PipelineId, new_size: WindowSizeData) {
        let mut page = self.page.borrow_mut();
        let page = page.find(id).expect("Received resize message for PipelineId not associated
            with a page in the page tree. This is a bug.");
        page.window_size.deref().set(new_size);
        let mut page_url = page.mut_url();
        let last_loaded_url = replace(&mut *page_url, None);
        for url in last_loaded_url.iter() {
            *page_url = Some((url.ref0().clone(), true));
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
        self.compositor.close();
    }

    /// Handles a request to exit the script task and shut down layout.
    /// Returns true if the script task should shut down and false otherwise.
    fn handle_exit_pipeline_msg(&self, id: PipelineId) -> bool {
        // If root is being exited, shut down all pages
        let mut page = self.page.borrow_mut();
        if page.id == id {
            debug!("shutting down layout for root page {:?}", id);
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
    fn load(&self, pipeline_id: PipelineId, url: Url) {
        debug!("ScriptTask: loading {:?} on page {:?}", url, pipeline_id);

        let mut page = self.page.borrow_mut();
        let page = page.find(pipeline_id).expect("ScriptTask: received a load
            message for a layout channel that is not associated with this script task. This
            is a bug.");

        let last_loaded_url = replace(&mut *page.mut_url(), None);
        match last_loaded_url {
            Some((ref loaded, needs_reflow)) if *loaded == url => {
                *page.mut_url() = Some((loaded.clone(), false));
                if needs_reflow {
                    page.damage(ContentChangedDocumentDamage);
                    page.reflow(ReflowForDisplay, self.chan.clone(), self.compositor);
                }
                return;
            },
            _ => (),
        }

        let cx = self.js_context.borrow();
        let cx = cx.get_ref();
        // Create the window and document objects.
        let window = Window::new(cx.deref().ptr,
                                 page.clone(),
                                 self.chan.clone(),
                                 self.compositor.dup(),
                                 self.image_cache_task.clone()).root();
        let document = Document::new(&*window, Some(url.clone()), HTMLDocument, None).root();
        window.deref().init_browser_context(&*document);

        self.compositor.set_ready_state(Loading);
        // Parse HTML.
        //
        // Note: We can parse the next document in parallel with any previous documents.
        let html_parsing_result = hubbub_html_parser::parse_html(&*page,
                                                                 &*document,
                                                                 url.clone(),
                                                                 self.resource_task.clone());

        let HtmlParserResult {
            discovery_port
        } = html_parsing_result;

        {
            // Create the root frame.
            let mut frame = page.mut_frame();
            *frame = Some(Frame {
                document: JS::from_rooted(document.deref()),
                window: JS::from_rooted(window.deref()),
            });
        }

        // Send style sheets over to layout.
        //
        // FIXME: These should be streamed to layout as they're parsed. We don't need to stop here
        // in the script task.

        let mut js_scripts = None;
        loop {
            match discovery_port.recv_opt() {
                Ok(HtmlDiscoveredScript(scripts)) => {
                    assert!(js_scripts.is_none());
                    js_scripts = Some(scripts);
                }
                Ok(HtmlDiscoveredStyle(sheet)) => {
                    let LayoutChan(ref chan) = *page.layout_chan;
                    chan.send(AddStylesheetMsg(sheet));
                }
                Err(()) => break
            }
        }

        // Kick off the initial reflow of the page.
        document.deref().content_changed();

        let fragment = url.fragment.as_ref().map(|ref fragment| fragment.to_string());

        {
            // No more reflow required
            let mut page_url = page.mut_url();
            *page_url = Some((url.clone(), false));
        }

        // Receive the JavaScript scripts.
        assert!(js_scripts.is_some());
        let js_scripts = js_scripts.take_unwrap();
        debug!("js_scripts: {:?}", js_scripts);

        with_compartment((**cx).ptr, window.reflector().get_jsobject(), || {
            // Evaluate every script in the document.
            for file in js_scripts.iter() {
                let global_obj = window.reflector().get_jsobject();
                //FIXME: this should have some kind of error handling, or explicitly
                //       drop an exception on the floor.
                match cx.evaluate_script(global_obj, file.data.clone(), file.url.to_str(), 1) {
                    Ok(_) => (),
                    Err(_) => println!("evaluate_script failed")
                }
            }
        });

        // We have no concept of a document loader right now, so just dispatch the
        // "load" event as soon as we've finished executing all scripts parsed during
        // the initial load.
        let event = Event::new(&Window(*window), "load".to_string(), false, false).root();
        let doctarget: &JSRef<EventTarget> = EventTargetCast::from_ref(&*document);
        let wintarget: &JSRef<EventTarget> = EventTargetCast::from_ref(&*window);
        let _ = wintarget.dispatch_event_with_target(Some((*doctarget).clone()),
                                                     &*event);

        page.fragment_node.assign(fragment.map_or(None, |fragid| page.find_fragment_node(fragid)));

        let ConstellationChan(ref chan) = self.constellation_chan;
        chan.send(LoadCompleteMsg(page.id, url));
    }

    fn scroll_fragment_point(&self, pipeline_id: PipelineId, node: &JSRef<Element>) {
        let node: &JSRef<Node> = NodeCast::from_ref(node);
        let rect = node.get_bounding_content_box();
        let point = Point2D(to_frac_px(rect.origin.x).to_f32().unwrap(),
                            to_frac_px(rect.origin.y).to_f32().unwrap());
        // FIXME(#2003, pcwalton): This is pretty bogus when multiple layers are involved.
        // Really what needs to happen is that this needs to go through layout to ask which
        // layer the element belongs to, and have it send the scroll message to the
        // compositor.
        self.compositor.scroll_fragment_point(pipeline_id, LayerId::null(), point);
    }

    /// This is the main entry point for receiving and dispatching DOM events.
    ///
    /// TODO: Actually perform DOM event dispatch.
    fn handle_event(&self, pipeline_id: PipelineId, event: Event_) {
        match event {
            ResizeEvent(new_size) => {
                debug!("script got resize event: {:?}", new_size);

                let window = {
                    let page = get_page(&*self.page.borrow(), pipeline_id);
                    page.window_size.deref().set(new_size);

                    let frame = page.frame();
                    if frame.is_some() {
                        page.damage(ReflowDocumentDamage);
                        page.reflow(ReflowForDisplay, self.chan.clone(), self.compositor)
                    }

                    let mut fragment_node = page.fragment_node.get();
                    match fragment_node.take().map(|node| node.root()) {
                        Some(node) => self.scroll_fragment_point(pipeline_id, &*node),
                        None => {}
                    }

                    frame.as_ref().map(|frame| Temporary::new(frame.window.clone()))
                };

                match window.root() {
                    Some(window) => {
                        // http://dev.w3.org/csswg/cssom-view/#resizing-viewports
                        // https://dvcs.w3.org/hg/dom3events/raw-file/tip/html/DOM3-Events.html#event-type-resize
                        let uievent = UIEvent::new(&window.clone(),
                                                   "resize".to_string(), false,
                                                   false, Some(window.clone()),
                                                   0i32).root();
                        let event: &JSRef<Event> = EventCast::from_ref(&*uievent);

                        let wintarget: &JSRef<EventTarget> = EventTargetCast::from_ref(&*window);
                        let _ = wintarget.dispatch_event_with_target(None, event);
                    }
                    None => ()
                }
            }

            // FIXME(pcwalton): This reflows the entire document and is not incremental-y.
            ReflowEvent => {
                debug!("script got reflow event");
                let page = get_page(&*self.page.borrow(), pipeline_id);
                let frame = page.frame();
                if frame.is_some() {
                    page.damage(MatchSelectorsDocumentDamage);
                    page.reflow(ReflowForDisplay, self.chan.clone(), self.compositor)
                }
            }

            ClickEvent(_button, point) => {
                debug!("ClickEvent: clicked at {:?}", point);
                let page = get_page(&*self.page.borrow(), pipeline_id);
                match page.hit_test(&point) {
                    Some(node_address) => {
                        debug!("node address is {:?}", node_address);

                        let temp_node =
                                node::from_untrusted_node_address(
                                    self.js_runtime.deref().ptr, node_address);

                        let maybe_node = temp_node.root().ancestors().find(|node| node.is_element());
                        match maybe_node {
                            Some(node) => {
                                debug!("clicked on {:s}", node.debug_str());
                                match *page.frame() {
                                    Some(ref frame) => {
                                        let window = frame.window.root();
                                        let event =
                                            Event::new(&Window(*window),
                                                       "click".to_string(),
                                                       true, true).root();
                                        let eventtarget: &JSRef<EventTarget> = EventTargetCast::from_ref(&node);
                                        let _ = eventtarget.dispatch_event_with_target(None, &*event);
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
            MouseDownEvent(..) => {}
            MouseUpEvent(..) => {}
            MouseMoveEvent(point) => {
                let page = get_page(&*self.page.borrow(), pipeline_id);
                match page.get_nodes_under_mouse(&point) {
                    Some(node_address) => {

                        let mut target_list = vec!();
                        let mut target_compare = false;

                        let mouse_over_targets = &mut *self.mouse_over_targets.borrow_mut();
                        match *mouse_over_targets {
                            Some(ref mut mouse_over_targets) => {
                                for node in mouse_over_targets.mut_iter() {
                                    let node = node.root();
                                    node.deref().set_hover_state(false);
                                }
                            }
                            None => {}
                        }

                        for node_address in node_address.iter() {

                            let temp_node =
                                node::from_untrusted_node_address(
                                    self.js_runtime.deref().ptr, *node_address);

                            let maybe_node = temp_node.root().ancestors().find(|node| node.is_element());
                            match maybe_node {
                                Some(node) => {
                                    node.set_hover_state(true);

                                    match *mouse_over_targets {
                                        Some(ref mouse_over_targets) => {
                                            if !target_compare {
                                                target_compare = !mouse_over_targets.contains(&JS::from_rooted(&node));
                                            }
                                        }
                                        None => {}
                                    }
                                    target_list.push(JS::from_rooted(&node));
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
                                page.damage(MatchSelectorsDocumentDamage);
                                page.reflow(ReflowForDisplay, self.chan.clone(), self.compositor);
                            }
                            *mouse_over_targets = Some(target_list);
                        }
                    }

                    None => {}
              }
            }
        }
    }

    /// The entry point for content to notify that a new load has been requested
    /// for the given pipeline.
    fn trigger_load(&self, pipeline_id: PipelineId, url: Url) {
        let ConstellationChan(ref const_chan) = self.constellation_chan;
        const_chan.send(LoadUrlMsg(pipeline_id, url));
    }

    /// The entry point for content to notify that a fragment url has been requested
    /// for the given pipeline.
    fn trigger_fragment(&self, pipeline_id: PipelineId, url: Url) {
        let page = get_page(&*self.page.borrow(), pipeline_id);
        match page.find_fragment_node(url.fragment.unwrap()).root() {
            Some(node) => {
                self.scroll_fragment_point(pipeline_id, &*node);
            }
            None => {}
         }
     }
}

/// Shuts down layout for the given page tree.
fn shut_down_layout(page_tree: &Rc<Page>, rt: *mut JSRuntime) {
    for page in page_tree.iter() {
        page.join_layout();

        // Tell the layout task to begin shutting down, and wait until it
        // processed this message.
        let (response_chan, response_port) = channel();
        let LayoutChan(ref chan) = *page.layout_chan;
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
        let LayoutChan(ref chan) = *page.layout_chan;
        chan.send(layout_interface::ExitNowMsg);
    }
}


fn get_page(page: &Rc<Page>, pipeline_id: PipelineId) -> Rc<Page> {
    page.find(pipeline_id).expect("ScriptTask: received an event \
        message for a layout channel that is not associated with this script task.\
         This is a bug.")
}
