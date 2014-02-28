/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

//! The script task is the task that owns the DOM in memory, runs JavaScript, and spawns parsing
//! and layout tasks.

use dom::bindings::codegen::RegisterBindings;
use dom::bindings::codegen::InheritTypes::{EventTargetCast, NodeCast, ElementCast, EventCast};
use dom::bindings::js::JS;
use dom::bindings::utils::{Reflectable, GlobalStaticData, with_gc_enabled};
use dom::document::{Document, HTMLDocument};
use dom::element::Element;
use dom::event::{Event_, ResizeEvent, ReflowEvent, ClickEvent, MouseDownEvent, MouseMoveEvent, MouseUpEvent};
use dom::event::Event;
use dom::uievent::UIEvent;
use dom::eventtarget::EventTarget;
use dom::node::{Node, NodeHelpers};
use dom::window::{TimerData, TimerHandle, Window};
use dom::windowproxy::WindowProxy;
use html::hubbub_html_parser::HtmlParserResult;
use html::hubbub_html_parser::{HtmlDiscoveredStyle, HtmlDiscoveredIFrame, HtmlDiscoveredScript};
use html::hubbub_html_parser;
use layout_interface::{AddStylesheetMsg, DocumentDamage};
use layout_interface::{ContentBoxQuery, ContentBoxResponse};
use layout_interface::{DocumentDamageLevel, HitTestQuery, HitTestResponse, LayoutQuery, MouseOverQuery, MouseOverResponse};
use layout_interface::{LayoutChan, MatchSelectorsDocumentDamage, QueryMsg};
use layout_interface::{Reflow, ReflowDocumentDamage, ReflowForDisplay, ReflowGoal, ReflowMsg};
use layout_interface::ContentChangedDocumentDamage;
use layout_interface;

use extra::url::Url;
use geom::point::Point2D;
use geom::size::Size2D;
use js::JSVAL_NULL;
use js::global::DEBUG_FNS;
use js::glue::RUST_JSVAL_TO_OBJECT;
use js::jsapi::{JSContext, JSObject, JS_InhibitGC, JS_AllowGC};
use js::jsapi::{JS_CallFunctionValue, JS_GetContextPrivate};
use js::rust::{Compartment, Cx, CxUtils, RtUtils};
use js;
use servo_msg::compositor_msg::{FinishedLoading, Loading, PerformingLayout, ScriptListener};
use servo_msg::constellation_msg::{ConstellationChan, IFrameSandboxed, IFrameUnsandboxed};
use servo_msg::constellation_msg::{LoadIframeUrlMsg, LoadCompleteMsg, LoadUrlMsg, NavigationDirection};
use servo_msg::constellation_msg::{PipelineId, SubpageId, Failure, FailureMsg};
use servo_msg::constellation_msg;
use servo_net::image_cache_task::ImageCacheTask;
use servo_net::resource_task::ResourceTask;
use servo_util::geometry::to_frac_px;
use servo_util::url::parse_url;
use servo_util::task::send_on_failure;
use servo_util::namespace::Null;
use std::cast;
use std::cell::{RefCell, Ref, RefMut};
use std::comm::{Port, SharedChan};
use std::ptr;
use std::rc::Rc;
use std::task;
use std::util::replace;

use extra::serialize::{Encoder, Encodable};

/// Messages used to control the script task.
pub enum ScriptMsg {
    /// Loads a new URL on the specified pipeline.
    LoadMsg(PipelineId, Url),
    /// Gives a channel and ID to a layout task, as well as the ID of that layout's parent
    AttachLayoutMsg(NewLayoutInfo),
    /// Instructs the script task to send a navigate message to the constellation.
    NavigateMsg(NavigationDirection),
    /// Sends a DOM event.
    SendEventMsg(PipelineId, Event_),
    /// Window resized.  Sends a DOM event eventually, but first we combine events.
    ResizeMsg(PipelineId, Size2D<uint>),
    /// Fires a JavaScript timeout.
    FireTimerMsg(PipelineId, ~TimerData),
    /// Notifies script that reflow is finished.
    ReflowCompleteMsg(PipelineId, uint),
    /// Notifies script that window has been resized but to not take immediate action.
    ResizeInactiveMsg(PipelineId, Size2D<uint>),
    /// Notifies the script that a pipeline should be closed.
    ExitPipelineMsg(PipelineId),
    /// Notifies the script that a window associated with a particular pipeline should be closed.
    ExitWindowMsg(PipelineId),
}

pub struct NewLayoutInfo {
    old_id: PipelineId,
    new_id: PipelineId,
    layout_chan: LayoutChan,
}

/// Encapsulates external communication with the script task.
#[deriving(Clone)]
pub struct ScriptChan(SharedChan<ScriptMsg>);

impl<S: Encoder> Encodable<S> for ScriptChan {
    fn encode(&self, _s: &mut S) {
    }
}

impl ScriptChan {
    /// Creates a new script chan.
    pub fn new() -> (Port<ScriptMsg>, ScriptChan) {
        let (port, chan) = SharedChan::new();
        (port, ScriptChan(chan))
    }
}

/// Encapsulates a handle to a frame and its associated layout information.
pub struct Page {
    /// Pipeline id associated with this page.
    id: PipelineId,

    /// Unique id for last reflow request; used for confirming completion reply.
    last_reflow_id: RefCell<uint>,

    /// The outermost frame containing the document, window, and page URL.
    frame: RefCell<Option<Frame>>,

    /// A handle for communicating messages to the layout task.
    layout_chan: LayoutChan,

    /// The port that we will use to join layout. If this is `None`, then layout is not running.
    layout_join_port: RefCell<Option<Port<()>>>,

    /// What parts of the document are dirty, if any.
    damage: RefCell<Option<DocumentDamage>>,

    /// The current size of the window, in pixels.
    window_size: RefCell<Size2D<uint>>,

    js_info: RefCell<Option<JSPageInfo>>,

    /// Cached copy of the most recent url loaded by the script
    /// TODO(tkuehn): this currently does not follow any particular caching policy
    /// and simply caches pages forever (!). The bool indicates if reflow is required
    /// when reloading.
    url: RefCell<Option<(Url, bool)>>,

    next_subpage_id: RefCell<SubpageId>,

    /// Pending resize event, if any.
    resize_event: RefCell<Option<Size2D<uint>>>,

    /// Pending scroll to fragment event, if any
    fragment_node: RefCell<Option<JS<Element>>>
}

impl<S: Encoder> Encodable<S> for Page {
    fn encode(&self, s: &mut S) {
        let fragment_node = self.fragment_node.borrow();
        fragment_node.get().encode(s);
   }
}

pub struct PageTree {
    page: Rc<Page>,
    inner: ~[PageTree],
}

pub struct PageTreeIterator<'a> {
    priv stack: ~[&'a mut PageTree],
}

impl PageTree {
    fn new(id: PipelineId, layout_chan: LayoutChan, window_size: Size2D<uint>) -> PageTree {
        PageTree {
            page: unsafe { Rc::new_unchecked(Page {
                id: id,
                frame: RefCell::new(None),
                layout_chan: layout_chan,
                layout_join_port: RefCell::new(None),
                damage: RefCell::new(None),
                window_size: RefCell::new(window_size),
                js_info: RefCell::new(None),
                url: RefCell::new(None),
                next_subpage_id: RefCell::new(SubpageId(0)),
                resize_event: RefCell::new(None),
                fragment_node: RefCell::new(None),
                last_reflow_id: RefCell::new(0)
            }) },
            inner: ~[],
        }
    }

    fn id(&self) -> PipelineId {
        self.page().id
    }

    fn page<'a>(&'a self) -> &'a Page {
        self.page.borrow()
    }

    pub fn find<'a> (&'a mut self, id: PipelineId) -> Option<&'a mut PageTree> {
        if self.page().id == id { return Some(self); }
        for page_tree in self.inner.mut_iter() {
            let found = page_tree.find(id);
            if found.is_some() { return found; }
        }
        None
    }

    pub fn iter<'a>(&'a mut self) -> PageTreeIterator<'a> {
        PageTreeIterator {
            stack: ~[self],
        }
    }

    // must handle root case separately
    pub fn remove(&mut self, id: PipelineId) -> Option<PageTree> {
        let remove_idx = {
            self.inner.mut_iter()
                .enumerate()
                .find(|&(_idx, ref page_tree)| {
                    // FIXME: page_tree has a lifetime such that it's unusable for anything.
                    let page_tree = unsafe {
                        cast::transmute_region(page_tree)
                    };
                    page_tree.id() == id
                })
                .map(|(idx, _)| idx)
        };
        match remove_idx {
            Some(idx) => return Some(self.inner.remove(idx)),
            None => {
                for page_tree in self.inner.mut_iter() {
                    match page_tree.remove(id) {
                        found @ Some(_) => return found,
                        None => (), // keep going...
                    }
                }
            }
        }
        None
    }
}

impl<'a> Iterator<Rc<Page>> for PageTreeIterator<'a> {
    fn next(&mut self) -> Option<Rc<Page>> {
        if !self.stack.is_empty() {
            let next = self.stack.pop();
            {
                for child in next.inner.mut_iter() {
                    self.stack.push(child);
                }
            }
            Some(next.page.clone())
        } else {
            None
        }
    }
}

impl Page {
    pub fn mut_js_info<'a>(&'a self) -> RefMut<'a, Option<JSPageInfo>> {
        self.js_info.borrow_mut()
    }

    pub fn js_info<'a>(&'a self) -> Ref<'a, Option<JSPageInfo>> {
        self.js_info.borrow()
    }

    pub fn url<'a>(&'a self) -> Ref<'a, Option<(Url, bool)>> {
        self.url.borrow()
    }

    pub fn mut_url<'a>(&'a self) -> RefMut<'a, Option<(Url, bool)>> {
        self.url.borrow_mut()
    }

    pub fn frame<'a>(&'a self) -> Ref<'a, Option<Frame>> {
        self.frame.borrow()
    }

    pub fn mut_frame<'a>(&'a self) -> RefMut<'a, Option<Frame>> {
        self.frame.borrow_mut()
    }

    /// Adds the given damage.
    pub fn damage(&self, level: DocumentDamageLevel) {
        let frame = self.frame();
        let root = match *frame.get() {
            None => return,
            Some(ref frame) => frame.document.get().GetDocumentElement()
        };
        match root {
            None => {},
            Some(root) => {
                let root: JS<Node> = NodeCast::from(&root);
                let mut damage = self.damage.borrow_mut();
                match *damage.get() {
                    None => {}
                    Some(ref mut damage) => {
                        // FIXME(pcwalton): This is wrong. We should trace up to the nearest ancestor.
                        damage.root = root.to_trusted_node_address();
                        damage.level.add(level);
                        return
                    }
                }

                *damage.get() = Some(DocumentDamage {
                    root: root.to_trusted_node_address(),
                    level: level,
                })
            }
        };
    }

    pub fn get_url(&self) -> Url {
        let url = self.url();
        url.get().get_ref().first().clone()
    }

    /// Sends a ping to layout and waits for the response. The response will arrive when the
    /// layout task has finished any pending request messages.
    pub fn join_layout(&self) {
        let mut layout_join_port = self.layout_join_port.borrow_mut();
        if layout_join_port.get().is_some() {
            let join_port = replace(layout_join_port.get(), None);
            match join_port {
                Some(ref join_port) => {
                    match join_port.try_recv() {
                        None => {
                            info!("script: waiting on layout");
                            join_port.recv();
                        }
                        Some(_) => {}
                    }

                    debug!("script: layout joined")
                }
                None => fail!(~"reader forked but no join port?"),
            }
        }
    }

    /// Sends the given query to layout.
    pub fn query_layout<T: Send>(&self,
                                 query: LayoutQuery,
                                 response_port: Port<T>)
                                 -> T {
        self.join_layout();
        self.layout_chan.send(QueryMsg(query));
        response_port.recv()
    }

    /// Reflows the page if it's possible to do so. This method will wait until the layout task has
    /// completed its current action, join the layout task, and then request a new layout run. It
    /// won't wait for the new layout computation to finish.
    ///
    /// If there is no window size yet, the page is presumed invisible and no reflow is performed.
    ///
    /// This function fails if there is no root frame.
    pub fn reflow(&self,
                  goal: ReflowGoal,
                  script_chan: ScriptChan,
                  compositor: &ScriptListener) {
        let frame = self.frame();
        let root = match *frame.get() {
            None => return,
            Some(ref frame) => {
                frame.document.get().GetDocumentElement()
            }
        };

        match root {
            None => {},
            Some(root) => {
                debug!("script: performing reflow for goal {:?}", goal);

                // Now, join the layout so that they will see the latest changes we have made.
                self.join_layout();

                // Tell the user that we're performing layout.
                compositor.set_ready_state(PerformingLayout);

                // Layout will let us know when it's done.
                let (join_port, join_chan) = Chan::new();
                let mut layout_join_port = self.layout_join_port.borrow_mut();
                *layout_join_port.get() = Some(join_port);

                let mut last_reflow_id = self.last_reflow_id.borrow_mut();
                *last_reflow_id.get() += 1;

                let root: JS<Node> = NodeCast::from(&root);
                let mut damage = self.damage.borrow_mut();
                let window_size = self.window_size.borrow();

                // Send new document and relevant styles to layout.
                let reflow = ~Reflow {
                    document_root: root.to_trusted_node_address(),
                    url: self.get_url(),
                    goal: goal,
                    window_size: *window_size.get(),
                    script_chan: script_chan,
                    script_join_chan: join_chan,
                    damage: replace(damage.get(), None).unwrap(),
                    id: *last_reflow_id.get(),
                };

                self.layout_chan.send(ReflowMsg(reflow));

                debug!("script: layout forked")
            }
        }
    }

    pub fn initialize_js_info(&self, js_context: Rc<Cx>, global: *JSObject) {
        assert!(global.is_not_null());

        // Note that the order that these variables are initialized is _not_ arbitrary. Switching
        // them around can -- and likely will -- lead to things breaking.

        js_context.borrow().set_default_options_and_version();
        js_context.borrow().set_logging_error_reporter();

        let compartment = match js_context.new_compartment_with_global(global) {
              Ok(c) => c,
              Err(()) => fail!("Failed to create a compartment"),
        };

        // Indirection for Rust Issue #6248, dynamic freeze scope artifically extended
        let page_ptr = {
            let borrowed_page = &*self;
            borrowed_page as *Page
        };

        unsafe {
            js_context.borrow().set_cx_private(page_ptr as *());

            JS_InhibitGC(js_context.borrow().ptr);
        }

        let mut js_info = self.mut_js_info();
        *js_info.get() = Some(JSPageInfo {
            dom_static: GlobalStaticData(),
            js_compartment: compartment,
            js_context: js_context,
        });
    }
}

/// Information for one frame in the browsing context.
#[deriving(Encodable)]
pub struct Frame {
    /// The document for this frame.
    document: JS<Document>,
    /// The window object for this frame.
    window: JS<Window>,
}

/// Encapsulation of the javascript information associated with each frame.
pub struct JSPageInfo {
    /// Global static data related to the DOM.
    dom_static: GlobalStaticData,
    /// The JavaScript compartment for the origin associated with the script task.
    js_compartment: Rc<Compartment>,
    /// The JavaScript context.
    js_context: Rc<Cx>,
}

/// Information for an entire page. Pages are top-level browsing contexts and can contain multiple
/// frames.
///
/// FIXME: Rename to `Page`, following WebKit?
pub struct ScriptTask {
    /// A handle to the information pertaining to page layout
    page_tree: RefCell<PageTree>,
    /// A handle to the image cache task.
    image_cache_task: ImageCacheTask,
    /// A handle to the resource task.
    resource_task: ResourceTask,

    /// The port on which the script task receives messages (load URL, exit, etc.)
    port: Port<ScriptMsg>,
    /// A channel to hand out when some other task needs to be able to respond to a message from
    /// the script task.
    chan: ScriptChan,

    /// For communicating load url messages to the constellation
    constellation_chan: ConstellationChan,
    /// A handle to the compositor for communicating ready state messages.
    compositor: ~ScriptListener,

    /// The JavaScript runtime.
    js_runtime: js::rust::rt,

    mouse_over_targets: RefCell<Option<~[JS<Node>]>>
}

/// Returns the relevant page from the associated JS Context.
pub fn page_from_context(js_context: *JSContext) -> *Page {
    unsafe {
        JS_GetContextPrivate(js_context) as *Page
    }
}

impl ScriptTask {
    /// Creates a new script task.
    pub fn new(id: PipelineId,
               compositor: ~ScriptListener,
               layout_chan: LayoutChan,
               port: Port<ScriptMsg>,
               chan: ScriptChan,
               constellation_chan: ConstellationChan,
               resource_task: ResourceTask,
               img_cache_task: ImageCacheTask,
               window_size: Size2D<uint>)
               -> Rc<ScriptTask> {
        let js_runtime = js::rust::rt();

        unsafe {
          Rc::new_unchecked(ScriptTask {
            page_tree: RefCell::new(PageTree::new(id, layout_chan, window_size)),

            image_cache_task: img_cache_task,
            resource_task: resource_task,

            port: port,
            chan: chan,
            constellation_chan: constellation_chan,
            compositor: compositor,

            js_runtime: js_runtime,
            mouse_over_targets: RefCell::new(None)
          })
        }
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
                  compositor: ~C,
                  layout_chan: LayoutChan,
                  port: Port<ScriptMsg>,
                  chan: ScriptChan,
                  constellation_chan: ConstellationChan,
                  failure_msg: Failure,
                  resource_task: ResourceTask,
                  image_cache_task: ImageCacheTask,
                  window_size: Size2D<uint>) {
        let mut builder = task::task();
        send_on_failure(&mut builder, FailureMsg(failure_msg), (*constellation_chan).clone());
        builder.name("ScriptTask");
        builder.spawn(proc() {
            let script_task = ScriptTask::new(id,
                                              compositor as ~ScriptListener,
                                              layout_chan,
                                              port,
                                              chan,
                                              constellation_chan,
                                              resource_task,
                                              image_cache_task,
                                              window_size);
            script_task.borrow().start();
        });
    }

    /// Handle incoming control messages.
    fn handle_msgs(&self) -> bool {
        // Handle pending resize events.
        // Gather them first to avoid a double mut borrow on self.
        let mut resizes = ~[];

        {
            let mut page_tree = self.page_tree.borrow_mut();
            for page in page_tree.get().iter() {
                // Only process a resize if layout is idle.
                let page = page.borrow();
                let layout_join_port = page.layout_join_port.borrow();
                if layout_join_port.get().is_none() {
                    let mut resize_event = page.resize_event.borrow_mut();
                    match resize_event.get().take() {
                        Some(size) => resizes.push((page.id, size)),
                        None => ()
                    }
                }
            }
        }

        for (id, Size2D { width, height }) in resizes.move_iter() {
            self.handle_event(id, ResizeEvent(width, height));
        }

        // Store new resizes, and gather all other events.
        let mut sequential = ~[];

        // Receive at least one message so we don't spinloop.
        let mut event = self.port.recv();

        loop {
            match event {
                ResizeMsg(id, size) => {
                    let mut page_tree = self.page_tree.borrow_mut();
                    let page = page_tree.get().find(id).expect("resize sent to nonexistent pipeline").page();
                    let mut resize_event = page.resize_event.borrow_mut();
                    *resize_event.get() = Some(size);
                }
                _ => {
                    sequential.push(event);
                }
            }

            match self.port.try_recv() {
                None => break,
                Some(ev) => event = ev,
            }
        }

        // Process the gathered events.
        for msg in sequential.move_iter() {
            match msg {
                // TODO(tkuehn) need to handle auxiliary layouts for iframes
                AttachLayoutMsg(new_layout_info) => self.handle_new_layout(new_layout_info),
                LoadMsg(id, url) => self.load(id, url),
                SendEventMsg(id, event) => self.handle_event(id, event),
                FireTimerMsg(id, timer_data) => self.handle_fire_timer_msg(id, timer_data),
                NavigateMsg(direction) => self.handle_navigate_msg(direction),
                ReflowCompleteMsg(id, reflow_id) => self.handle_reflow_complete_msg(id, reflow_id),
                ResizeInactiveMsg(id, new_size) => self.handle_resize_inactive_msg(id, new_size),
                ExitPipelineMsg(id) => if self.handle_exit_pipeline_msg(id) { return false },
                ExitWindowMsg(id) => self.handle_exit_window_msg(id),
                ResizeMsg(..) => fail!("should have handled ResizeMsg already"),
            }
        }

        true
    }

    fn handle_new_layout(&self, new_layout_info: NewLayoutInfo) {
        debug!("Script: new layout: {:?}", new_layout_info);
        let NewLayoutInfo {
            old_id,
            new_id,
            layout_chan
        } = new_layout_info;

        let mut page_tree = self.page_tree.borrow_mut();
        let parent_page_tree = page_tree.get().find(old_id).expect("ScriptTask: received a layout
            whose parent has a PipelineId which does not correspond to a pipeline in the script
            task's page tree. This is a bug.");
        let new_page_tree = {
            let window_size = parent_page_tree.page().window_size.borrow();
            PageTree::new(new_id, layout_chan, *window_size.get())
        };
        parent_page_tree.inner.push(new_page_tree);
    }

    /// Handles a timer that fired.
    fn handle_fire_timer_msg(&self, id: PipelineId, timer_data: ~TimerData) {
        let mut page_tree = self.page_tree.borrow_mut();
        let page = page_tree.get().find(id).expect("ScriptTask: received fire timer msg for a
            pipeline ID not associated with this script task. This is a bug.").page();
        let frame = page.frame();
        let mut window = frame.get().get_ref().window.clone();
        if !window.get().active_timers.contains(&TimerHandle { handle: timer_data.handle, cancel_chan: None }) {
            return;
        }
        window.get_mut().active_timers.remove(&TimerHandle { handle: timer_data.handle, cancel_chan: None });
        let js_info = page.js_info();
        let this_value = if timer_data.args.len() > 0 {
            unsafe {
                RUST_JSVAL_TO_OBJECT(timer_data.args[0])
            }
        } else {
            js_info.get().get_ref().js_compartment.borrow().global_obj.borrow().ptr
        };

        // TODO: Support extra arguments. This requires passing a `*JSVal` array as `argv`.
        let rval = JSVAL_NULL;
        let cx = js_info.get().get_ref().js_context.borrow().ptr;
        with_gc_enabled(cx, || {
            unsafe {
                JS_CallFunctionValue(cx, this_value, timer_data.funval, 0, ptr::null(), &rval);
            }
        });
    }

    /// Handles a notification that reflow completed.
    fn handle_reflow_complete_msg(&self, pipeline_id: PipelineId, reflow_id: uint) {
        debug!("Script: Reflow {:?} complete for {:?}", reflow_id, pipeline_id);
        let mut page_tree = self.page_tree.borrow_mut();
        let page = page_tree.get().find(pipeline_id).expect(
            "ScriptTask: received a load message for a layout channel that is not associated \
             with this script task. This is a bug.").page();
        let last_reflow_id = page.last_reflow_id.borrow();
        if *last_reflow_id.get() == reflow_id {
            let mut layout_join_port = page.layout_join_port.borrow_mut();
            *layout_join_port.get() = None;
        }
        self.compositor.set_ready_state(FinishedLoading);
    }

    /// Handles a navigate forward or backward message.
    /// TODO(tkuehn): is it ever possible to navigate only on a subframe?
    fn handle_navigate_msg(&self, direction: NavigationDirection) {
        self.constellation_chan.send(constellation_msg::NavigateMsg(direction));
    }

    /// Window was resized, but this script was not active, so don't reflow yet
    fn handle_resize_inactive_msg(&self, id: PipelineId, new_size: Size2D<uint>) {
        let mut page_tree = self.page_tree.borrow_mut();
        let page = page_tree.get().find(id).expect("Received resize message for PipelineId not associated
            with a page in the page tree. This is a bug.").page();
        let mut window_size = page.window_size.borrow_mut();
        *window_size.get() = new_size;
        let mut page_url = page.mut_url();
        let last_loaded_url = replace(page_url.get(), None);
        for url in last_loaded_url.iter() {
            *page_url.get() = Some((url.first(), true));
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
        let mut page_tree = self.page_tree.borrow_mut();
        if page_tree.get().page().id == id {
            for page in page_tree.get().iter() {
                let page = page.borrow();
                debug!("shutting down layout for root page {:?}", page.id);
                shut_down_layout(page)
            }
            return true
        }

        // otherwise find just the matching page and exit all sub-pages
        match page_tree.get().remove(id) {
            Some(ref mut page_tree) => {
                for page in page_tree.iter() {
                    let page = page.borrow();
                    debug!("shutting down layout for page {:?}", page.id);
                    shut_down_layout(page)
                }
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

        let mut page_tree = self.page_tree.borrow_mut();
        let page_tree = page_tree.get().find(pipeline_id).expect("ScriptTask: received a load
            message for a layout channel that is not associated with this script task. This
            is a bug.");
        let page = page_tree.page();

        {
            let mut page_url = page.mut_url();
            let last_loaded_url = replace(page_url.get(), None);
            for loaded in last_loaded_url.iter() {
                let (ref loaded, needs_reflow) = *loaded;
                if *loaded == url {
                    *page_url.get() = Some((loaded.clone(), false));
                    if needs_reflow {
                        page.damage(ContentChangedDocumentDamage);
                        page.reflow(ReflowForDisplay, self.chan.clone(), self.compositor);
                    }
                    return;
                }
            }
        }

        let cx = self.js_runtime.cx();
        // Create the window and document objects.
        let window = Window::new(cx.borrow().ptr,
                                 page_tree.page.clone(),
                                 self.chan.clone(),
                                 self.compositor.dup(),
                                 self.image_cache_task.clone());
        page.initialize_js_info(cx.clone(), window.reflector().get_jsobject());

        {
            let mut js_info = page.mut_js_info();
            RegisterBindings::Register(js_info.get().get_mut_ref());
        }

        self.compositor.set_ready_state(Loading);
        // Parse HTML.
        //
        // Note: We can parse the next document in parallel with any previous documents.
        let mut document = Document::new(&window, Some(url.clone()), HTMLDocument, None);
        let next_subpage_id = page.next_subpage_id.borrow();
        let html_parsing_result = hubbub_html_parser::parse_html(page,
                                                                 &mut document,
                                                                 url.clone(),
                                                                 self.resource_task.clone(),
                                                                 next_subpage_id.get().clone());

        let HtmlParserResult {
            discovery_port
        } = html_parsing_result;

        {
            // Create the root frame.
            let mut frame = page.mut_frame();
            *frame.get() = Some(Frame {
                document: document.clone(),
                window: window.clone(),
            });
        }

        // Send style sheets over to layout.
        //
        // FIXME: These should be streamed to layout as they're parsed. We don't need to stop here
        // in the script task.

        let mut js_scripts = None;
        loop {
            match discovery_port.recv_opt() {
                Some(HtmlDiscoveredScript(scripts)) => {
                    assert!(js_scripts.is_none());
                    js_scripts = Some(scripts);
                }
                Some(HtmlDiscoveredStyle(sheet)) => {
                    page.layout_chan.send(AddStylesheetMsg(sheet));
                }
                Some(HtmlDiscoveredIFrame((iframe_url, subpage_id, sandboxed))) => {
                    let mut next_subpage_id = page.next_subpage_id.borrow_mut();
                    *next_subpage_id.get() = SubpageId(*subpage_id + 1);
                    let sandboxed = if sandboxed {
                        IFrameSandboxed
                    } else {
                        IFrameUnsandboxed
                    };
                    self.constellation_chan.send(LoadIframeUrlMsg(iframe_url,
                                                                  pipeline_id,
                                                                  subpage_id,
                                                                  sandboxed));
                }
                None => break
            }
        }

        // Kick off the initial reflow of the page.
        document.get().content_changed();

        let fragment = url.fragment.as_ref().map(|ref fragment| fragment.to_owned());

        {
            // No more reflow required
            let mut page_url = page.mut_url();
            *page_url.get() = Some((url.clone(), false));
        }

        // Receive the JavaScript scripts.
        assert!(js_scripts.is_some());
        let js_scripts = js_scripts.take_unwrap();
        debug!("js_scripts: {:?}", js_scripts);

        // Define debug functions.
        let cx = {
            let js_info = page.js_info();
            let js_info = js_info.get().get_ref();
            let compartment = js_info.js_compartment.borrow();
            compartment.define_functions(DEBUG_FNS);

            js_info.js_context.borrow().ptr
        };

        // Evaluate every script in the document.
        for file in js_scripts.iter() {
            with_gc_enabled(cx, || {
                let (cx, global_obj) = {
                    let js_info = page.js_info();
                    (js_info.get().get_ref().js_context.clone(),
                     js_info.get().get_ref().js_compartment.borrow().global_obj.clone())
                };
                cx.borrow().evaluate_script(global_obj,
                                            file.data.clone(),
                                            file.url.to_str(),
                                            1);
            });
        }

        // We have no concept of a document loader right now, so just dispatch the
        // "load" event as soon as we've finished executing all scripts parsed during
        // the initial load.
        let mut event = Event::new(&window);
        event.get_mut().InitEvent(~"load", false, false);
        let doctarget = EventTargetCast::from(&document);
        let mut wintarget: JS<EventTarget> = EventTargetCast::from(&window);
        let winclone = wintarget.clone();
        wintarget.get_mut().dispatch_event_with_target(&winclone, Some(doctarget), &mut event);

        let mut fragment_node = page.fragment_node.borrow_mut();
        *fragment_node.get() = fragment.map_default(None, |fragid| self.find_fragment_node(page, fragid));

        self.constellation_chan.send(LoadCompleteMsg(page.id, url));
    }

    fn find_fragment_node(&self, page: &Page, fragid: ~str) -> Option<JS<Element>> {
        let frame = page.frame();
        let document = frame.get().get_ref().document.clone();
        match document.get().GetElementById(fragid.to_owned()) {
            Some(node) => Some(node),
            None => {
                let doc_node: JS<Node> = NodeCast::from(&document);
                let mut anchors = doc_node.traverse_preorder().filter(|node| node.is_anchor_element());
                anchors.find(|node| {
                    let elem: JS<Element> = ElementCast::to(node);
                    elem.get().get_attribute(Null, "name").map_default(false, |attr| {
                        attr.get().value_ref() == fragid
                    })
                }).map(|node| ElementCast::to(&node))
            }
        }
    }

    fn scroll_fragment_point(&self, pipeline_id: PipelineId, page: &Page, node: JS<Element>) {
        let (port, chan) = Chan::new();
        let node: JS<Node> = NodeCast::from(&node);
        match page.query_layout(ContentBoxQuery(node.to_trusted_node_address(), chan), port) {
            ContentBoxResponse(rect) => {
                let point = Point2D(to_frac_px(rect.origin.x).to_f32().unwrap(), 
                                    to_frac_px(rect.origin.y).to_f32().unwrap());
                self.compositor.scroll_fragment_point(pipeline_id, point);
            }
        }
    }

    /// This is the main entry point for receiving and dispatching DOM events.
    ///
    /// TODO: Actually perform DOM event dispatch.
    fn handle_event(&self, pipeline_id: PipelineId, event: Event_) {
        let mut page_tree = self.page_tree.borrow_mut();
        let page = page_tree.get().find(pipeline_id).expect("ScriptTask: received an event
            message for a layout channel that is not associated with this script task. This
            is a bug.").page();

        match event {
            ResizeEvent(new_width, new_height) => {
                debug!("script got resize event: {:u}, {:u}", new_width, new_height);

                {
                    let mut window_size = page.window_size.borrow_mut();
                    *window_size.get() = Size2D(new_width, new_height);
                }

                {
                    let frame = page.frame();
                    if frame.get().is_some() {
                        page.damage(ReflowDocumentDamage);
                        page.reflow(ReflowForDisplay, self.chan.clone(), self.compositor)
                    }
                }

                let mut fragment_node = page.fragment_node.borrow_mut();
                match fragment_node.get().take() {
                    Some(node) => self.scroll_fragment_point(pipeline_id, page, node),
                    None => {}
                }

                let frame = page.frame();
                match *frame.get() {
                    Some(ref frame) => {
                        // http://dev.w3.org/csswg/cssom-view/#resizing-viewports
                        // https://dvcs.w3.org/hg/dom3events/raw-file/tip/html/DOM3-Events.html#event-type-resize
                        let window_proxy: JS<WindowProxy> = WindowProxy::new(frame.window.clone());
                        let mut uievent = UIEvent::new(&frame.window);
                        uievent.get_mut().InitUIEvent(~"resize", false, false, Some(window_proxy), 0i32);
                        let event: &mut JS<Event> = &mut EventCast::from(&uievent);

                        // FIXME: this event should be dispatch on WindowProxy. See #1715
                        let mut wintarget: JS<EventTarget> = EventTargetCast::from(&frame.window);
                        let winclone = wintarget.clone();
                        wintarget.get_mut().dispatch_event_with_target(&winclone, None, event);
                    }
                    None =>()
                }
            }

            // FIXME(pcwalton): This reflows the entire document and is not incremental-y.
            ReflowEvent => {
                debug!("script got reflow event");

                let frame = page.frame();
                if frame.get().is_some() {
                    page.damage(MatchSelectorsDocumentDamage);
                    page.reflow(ReflowForDisplay, self.chan.clone(), self.compositor)
                }
            }

            ClickEvent(_button, point) => {
                debug!("ClickEvent: clicked at {:?}", point);

                let frame = page.frame();
                let document = frame.get().get_ref().document.clone();
                let root = document.get().GetDocumentElement();
                if root.is_none() {
                    return;
                }
                let (port, chan) = Chan::new();
                let root: JS<Node> = NodeCast::from(&root.unwrap());
                match page.query_layout(HitTestQuery(root.to_trusted_node_address(), point, chan), port) {
                    Ok(HitTestResponse(node_address)) => {
                        debug!("node address is {:?}", node_address);
                        let mut node: JS<Node> =
                            NodeHelpers::from_untrusted_node_address(self.js_runtime.borrow().ptr,
                                                                     node_address);
                        debug!("clicked on {:s}", node.debug_str());

                        // Traverse node generations until a node that is an element is
                        // found.
                        while !node.is_element() {
                            match node.parent_node() {
                                Some(parent) => node = parent,
                                None => break,
                            }
                        }

                        if node.is_element() {
                            let element: JS<Element> = ElementCast::to(&node);
                            if "a" == element.get().tag_name {
                                self.load_url_from_element(page, element.get())
                            }
                        }
                    },
                    Err(()) => debug!("layout query error"),
                }
            }
            MouseDownEvent(..) => {}
            MouseUpEvent(..) => {}
            MouseMoveEvent(point) => {
                let frame = page.frame();
                let document = frame.get().get_ref().document.clone();
                let root = document.get().GetDocumentElement();
                if root.is_none() {
                    return;
                }
                let root: JS<Node> = NodeCast::from(&root.unwrap());
                let (port, chan) = Chan::new();
                match page.query_layout(MouseOverQuery(root.to_trusted_node_address(), point, chan), port) {
                    Ok(MouseOverResponse(node_address)) => {

                        let mut target_list: ~[JS<Node>] = ~[];
                        let mut target_compare = false;

                        let mut mouse_over_targets = self.mouse_over_targets.borrow_mut();
                        match *mouse_over_targets.get() {
                            Some(ref mut mouse_over_targets) => {
                                for node in mouse_over_targets.mut_iter() {
                                    node.set_hover_state(false);
                                }
                            }
                            None => {}
                        }

                        for node_address in node_address.iter() {
                            let mut node: JS<Node> =
                                NodeHelpers::from_untrusted_node_address(
                                    self.js_runtime.borrow().ptr, *node_address);
                            // Traverse node generations until a node that is an element is
                            // found.
                            while !node.is_element() {
                                match node.parent_node() {
                                    Some(parent) => node = parent,
                                    None => break,
                                }
                            }

                            if node.is_element() {
                                node.set_hover_state(true);

                                match *mouse_over_targets.get() {
                                    Some(ref mouse_over_targets) => {
                                        if !target_compare {
                                            target_compare = !mouse_over_targets.contains(&node);
                                        }
                                    }
                                    None => {}
                                }
                                target_list.push(node);
                            }
                        }
                        match *mouse_over_targets.get() {
                            Some(ref mouse_over_targets) => {
                                if mouse_over_targets.len() != target_list.len() {
                                    target_compare = true;
                                }
                            }
                            None => { target_compare = true; }
                        }
 
                        if target_compare {
                            if mouse_over_targets.get().is_some() {
                                page.damage(MatchSelectorsDocumentDamage);
                                page.reflow(ReflowForDisplay, self.chan.clone(), self.compositor);
                            }
                            *mouse_over_targets.get() = Some(target_list);
                        }
                    },
                    Err(()) => {},
              }
            }
        }
    }

    fn load_url_from_element(&self, page: &Page, element: &Element) {
        // if the node's element is "a," load url from href attr
        let attr = element.get_attribute(Null, "href");
        for href in attr.iter() {
            debug!("ScriptTask: clicked on link to {:s}", href.get().Value());
            let click_frag = href.get().value_ref().starts_with("#");
            let base_url = Some(page.get_url());
            debug!("ScriptTask: current url is {:?}", base_url);
            let url = parse_url(href.get().value_ref(), base_url);

            if click_frag {
                match self.find_fragment_node(page, url.fragment.unwrap()) {
                    Some(node) => self.scroll_fragment_point(page.id, page, node),
                    None => {}
                }
            } else {
                self.constellation_chan.send(LoadUrlMsg(page.id, url));
            } 
        }
    }
}

/// Shuts down layout for the given page.
fn shut_down_layout(page: &Page) {
    page.join_layout();

    // Tell the layout task to begin shutting down.
    let (response_port, response_chan) = Chan::new();
    page.layout_chan.send(layout_interface::PrepareToExitMsg(response_chan));
    response_port.recv();

    // Destroy all nodes. Setting frame and js_info to None will trigger our
    // compartment to shutdown, run GC, etc.

    let mut js_info = page.mut_js_info();
    unsafe {
        JS_AllowGC(js_info.get().get_ref().js_context.borrow().ptr);
    }

    let mut frame = page.mut_frame();
    *frame.get() = None;
    *js_info.get() = None;

    // Destroy the layout task. If there were node leaks, layout will now crash safely.
    page.layout_chan.send(layout_interface::ExitNowMsg);
}
