/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

//! The script task is the task that owns the DOM in memory, runs JavaScript, and spawns parsing
//! and layout tasks.

use dom::bindings::codegen::RegisterBindings;
use dom::bindings::utils::{Reflectable, GlobalStaticData};
use dom::document::AbstractDocument;
use dom::element::Element;
use dom::event::{Event_, ResizeEvent, ReflowEvent, ClickEvent, MouseDownEvent, MouseUpEvent};
use dom::event::Event;
use dom::eventtarget::AbstractEventTarget;
use dom::htmldocument::HTMLDocument;
use dom::namespace::Null;
use dom::node::AbstractNode;
use dom::window::{TimerData, TimerHandle, Window};
use html::hubbub_html_parser::HtmlParserResult;
use html::hubbub_html_parser::{HtmlDiscoveredStyle, HtmlDiscoveredIFrame, HtmlDiscoveredScript};
use html::hubbub_html_parser;
use layout_interface::{AddStylesheetMsg, DocumentDamage};
use layout_interface::{ContentBoxQuery, ContentBoxResponse};
use layout_interface::{DocumentDamageLevel, HitTestQuery, HitTestResponse, LayoutQuery};
use layout_interface::{LayoutChan, MatchSelectorsDocumentDamage, QueryMsg};
use layout_interface::{Reflow, ReflowDocumentDamage, ReflowForDisplay, ReflowGoal, ReflowMsg};
use layout_interface::ContentChangedDocumentDamage;
use layout_interface;

use extra::url::Url;
use geom::point::Point2D;
use geom::size::Size2D;
use js::JSVAL_NULL;
use js::global::debug_fns;
use js::glue::RUST_JSVAL_TO_OBJECT;
use js::jsapi::{JSContext, JSObject};
use js::jsapi::{JS_CallFunctionValue, JS_GetContextPrivate};
use js::rust::{Compartment, Cx};
use js;
use servo_msg::compositor_msg::{FinishedLoading, Loading, PerformingLayout, ScriptListener};
use servo_msg::constellation_msg::{ConstellationChan, IFrameSandboxed, IFrameUnsandboxed};
use servo_msg::constellation_msg::{LoadIframeUrlMsg, LoadUrlMsg, NavigationDirection, PipelineId};
use servo_msg::constellation_msg::{SubpageId};
use servo_msg::constellation_msg;
use servo_net::image_cache_task::ImageCacheTask;
use servo_net::resource_task::ResourceTask;
use servo_util::geometry::to_frac_px;
use servo_util::url::make_url;
use std::comm::{Port, SharedChan};
use std::ptr;
use std::str::eq_slice;
use std::util::replace;

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
    last_reflow_id: uint,

    /// The outermost frame containing the document, window, and page URL.
    frame: Option<Frame>,

    /// A handle for communicating messages to the layout task.
    layout_chan: LayoutChan,

    /// The port that we will use to join layout. If this is `None`, then layout is not running.
    layout_join_port: Option<Port<()>>,

    /// What parts of the document are dirty, if any.
    damage: Option<DocumentDamage>,

    /// The current size of the window, in pixels.
    window_size: Size2D<uint>,

    js_info: Option<JSPageInfo>,

    /// Cached copy of the most recent url loaded by the script
    /// TODO(tkuehn): this currently does not follow any particular caching policy
    /// and simply caches pages forever (!). The bool indicates if reflow is required
    /// when reloading.
    url: Option<(Url, bool)>,

    next_subpage_id: SubpageId,

    /// Pending resize event, if any.
    resize_event: Option<Size2D<uint>>,

    /// Pending scroll to fragment event, if any
    fragment_node: Option<AbstractNode>
}

pub struct PageTree {
    page: @mut Page,
    inner: ~[PageTree],
}

pub struct PageTreeIterator<'a> {
    priv stack: ~[&'a mut PageTree],
}

impl PageTree {
    fn new(id: PipelineId, layout_chan: LayoutChan, window_size: Size2D<uint>) -> PageTree {
        PageTree {
            page: @mut Page {
                id: id,
                frame: None,
                layout_chan: layout_chan,
                layout_join_port: None,
                damage: None,
                window_size: window_size,
                js_info: None,
                url: None,
                next_subpage_id: SubpageId(0),
                resize_event: None,
                fragment_node: None,
                last_reflow_id: 0
            },
            inner: ~[],
        }
    }

    pub fn find<'a> (&'a mut self, id: PipelineId) -> Option<&'a mut PageTree> {
        if self.page.id == id { return Some(self); }
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
                .find(|&(_idx, ref page_tree)| page_tree.page.id == id)
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

impl<'a> Iterator<@mut Page> for PageTreeIterator<'a> {
    fn next(&mut self) -> Option<@mut Page> {
        if !self.stack.is_empty() {
            let next = self.stack.pop();
            {
                for child in next.inner.mut_iter() {
                    self.stack.push(child);
                }
            }
            Some(next.page)
        } else {
            None
        }
    }
}

impl Page {
    /// Adds the given damage.
    pub fn damage(&mut self, level: DocumentDamageLevel) {
        let root = match self.frame {
            None => return,
            Some(ref frame) => frame.document.document().GetDocumentElement()
        };
        match root {
            None => {},
            Some(root) => {
                match self.damage {
                    None => {}
                    Some(ref mut damage) => {
                        // FIXME(pcwalton): This is wrong. We should trace up to the nearest ancestor.
                        damage.root = root;
                        damage.level.add(level);
                        return
                    }
                }

                self.damage = Some(DocumentDamage {
                    root: root,
                    level: level,
                })
            }
        };
    }

    /// Sends a ping to layout and waits for the response. The response will arrive when the
    /// layout task has finished any pending request messages.
    pub fn join_layout(&mut self) {
        if self.layout_join_port.is_some() {
            let join_port = replace(&mut self.layout_join_port, None);
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
    pub fn query_layout<T: Send>(&mut self,
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
    pub fn reflow(&mut self,
                  goal: ReflowGoal,
                  script_chan: ScriptChan,
                  compositor: @ScriptListener) {
        let root = match self.frame {
            None => return,
            Some(ref frame) => {
                frame.document.document().GetDocumentElement()
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
                self.layout_join_port = Some(join_port);

                self.last_reflow_id += 1;

                // Send new document and relevant styles to layout.
                let reflow = ~Reflow {
                    document_root: root,
                    url: self.url.get_ref().first().clone(),
                    goal: goal,
                    window_size: self.window_size,
                    script_chan: script_chan,
                    script_join_chan: join_chan,
                    damage: replace(&mut self.damage, None).unwrap(),
                    id: self.last_reflow_id,
                };

                self.layout_chan.send(ReflowMsg(reflow));

                debug!("script: layout forked")
            }
        }
    }

    pub fn initialize_js_info(&mut self, js_context: @Cx, global: *JSObject) {
        // Note that the order that these variables are initialized is _not_ arbitrary. Switching
        // them around can -- and likely will -- lead to things breaking.

        js_context.set_default_options_and_version();
        js_context.set_logging_error_reporter();

        let compartment = match js_context.new_compartment_with_global(global) {
              Ok(c) => c,
              Err(()) => fail!("Failed to create a compartment"),
        };

        // Indirection for Rust Issue #6248, dynamic freeze scope artifically extended
        let page_ptr = {
            let borrowed_page = &mut *self;
            borrowed_page as *mut Page
        };

        unsafe {
            js_context.set_cx_private(page_ptr as *());
        }

        self.js_info = Some(JSPageInfo {
            dom_static: GlobalStaticData(),
            js_compartment: compartment,
            js_context: js_context,
        });
    }
}

/// Information for one frame in the browsing context.
pub struct Frame {
    /// The document for this frame.
    document: AbstractDocument,
    /// The window object for this frame.
    window: @mut Window,
}

/// Encapsulation of the javascript information associated with each frame.
pub struct JSPageInfo {
    /// Global static data related to the DOM.
    dom_static: GlobalStaticData,
    /// The JavaScript compartment for the origin associated with the script task.
    js_compartment: @mut Compartment,
    /// The JavaScript context.
    js_context: @Cx,
}

/// Information for an entire page. Pages are top-level browsing contexts and can contain multiple
/// frames.
///
/// FIXME: Rename to `Page`, following WebKit?
pub struct ScriptTask {
    /// A handle to the information pertaining to page layout
    page_tree: PageTree,
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
    compositor: @ScriptListener,

    /// The JavaScript runtime.
    js_runtime: js::rust::rt,
}

/// Returns the relevant page from the associated JS Context.
pub fn page_from_context(js_context: *JSContext) -> *mut Page {
    unsafe {
        JS_GetContextPrivate(js_context) as *mut Page
    }
}

impl ScriptTask {
    /// Creates a new script task.
    pub fn new(id: PipelineId,
               compositor: @ScriptListener,
               layout_chan: LayoutChan,
               port: Port<ScriptMsg>,
               chan: ScriptChan,
               constellation_chan: ConstellationChan,
               resource_task: ResourceTask,
               img_cache_task: ImageCacheTask,
               window_size: Size2D<uint>)
               -> @mut ScriptTask {
        let js_runtime = js::rust::rt();

        let script_task = @mut ScriptTask {
            page_tree: PageTree::new(id, layout_chan, window_size),

            image_cache_task: img_cache_task,
            resource_task: resource_task,

            port: port,
            chan: chan,
            constellation_chan: constellation_chan,
            compositor: compositor,

            js_runtime: js_runtime,
        };

        script_task
    }

    /// Starts the script task. After calling this method, the script task will loop receiving
    /// messages on its port.
    pub fn start(&mut self) {
        while self.handle_msgs() {
            // Go on...
        }
    }

    pub fn create<C:ScriptListener + Send>(
                  id: PipelineId,
                  compositor: C,
                  layout_chan: LayoutChan,
                  port: Port<ScriptMsg>,
                  chan: ScriptChan,
                  constellation_chan: ConstellationChan,
                  resource_task: ResourceTask,
                  image_cache_task: ImageCacheTask,
                  window_size: Size2D<uint>) {
        spawn(proc() {
            let script_task = ScriptTask::new(id,
                                              @compositor as @ScriptListener,
                                              layout_chan,
                                              port,
                                              chan,
                                              constellation_chan,
                                              resource_task,
                                              image_cache_task,
                                              window_size);
            script_task.start();
        });
    }

    /// Handle incoming control messages.
    fn handle_msgs(&mut self) -> bool {
        // Handle pending resize events.
        // Gather them first to avoid a double mut borrow on self.
        let mut resizes = ~[];
        for page in self.page_tree.iter() {
            // Only process a resize if layout is idle.
            if page.layout_join_port.is_none() {
                match page.resize_event.take() {
                    Some(size) => resizes.push((page.id, size)),
                    None => ()
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
                    let page = self.page_tree.find(id).expect("resize sent to nonexistent pipeline").page;
                    page.resize_event = Some(size);
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
                ExitWindowMsg(id) => {
                    self.handle_exit_window_msg(id);
                    return false
                },
                ResizeMsg(..) => fail!("should have handled ResizeMsg already"),
            }
        }

        true
    }

    fn handle_new_layout(&mut self, new_layout_info: NewLayoutInfo) {
        debug!("Script: new layout: {:?}", new_layout_info);
        let NewLayoutInfo {
            old_id,
            new_id,
            layout_chan
        } = new_layout_info;

        let parent_page_tree = self.page_tree.find(old_id).expect("ScriptTask: received a layout
            whose parent has a PipelineId which does not correspond to a pipeline in the script
            task's page tree. This is a bug.");
        let new_page_tree = PageTree::new(new_id, layout_chan, parent_page_tree.page.window_size);
        parent_page_tree.inner.push(new_page_tree);
    }

    /// Handles a timer that fired.
    fn handle_fire_timer_msg(&mut self, id: PipelineId, timer_data: ~TimerData) {
        let page = self.page_tree.find(id).expect("ScriptTask: received fire timer msg for a
            pipeline ID not associated with this script task. This is a bug.").page;
        let window = page.frame.expect("ScriptTask: Expect a timeout to have a document").window;
        if !window.active_timers.contains(&TimerHandle { handle: timer_data.handle, cancel_chan: None }) {
            return;
        }
        window.active_timers.remove(&TimerHandle { handle: timer_data.handle, cancel_chan: None });
        unsafe {
            let this_value = if timer_data.args.len() > 0 {
                RUST_JSVAL_TO_OBJECT(timer_data.args[0])
            } else {
                page.js_info.get_ref().js_compartment.global_obj.ptr
            };

            // TODO: Support extra arguments. This requires passing a `*JSVal` array as `argv`.
            let rval = JSVAL_NULL;
            JS_CallFunctionValue(page.js_info.get_ref().js_context.ptr,
                                 this_value,
                                 timer_data.funval,
                                 0,
                                 ptr::null(),
                                 &rval);

        }
    }

    /// Handles a notification that reflow completed.
    fn handle_reflow_complete_msg(&mut self, pipeline_id: PipelineId, reflow_id: uint) {
        debug!("Script: Reflow {:?} complete for {:?}", reflow_id, pipeline_id);
        let page_tree = self.page_tree.find(pipeline_id).expect(
            "ScriptTask: received a load message for a layout channel that is not associated \
             with this script task. This is a bug.");
        if page_tree.page.last_reflow_id == reflow_id {
            page_tree.page.layout_join_port = None;
        }
        self.compositor.set_ready_state(FinishedLoading);
    }

    /// Handles a navigate forward or backward message.
    /// TODO(tkuehn): is it ever possible to navigate only on a subframe?
    fn handle_navigate_msg(&self, direction: NavigationDirection) {
        self.constellation_chan.send(constellation_msg::NavigateMsg(direction));
    }

    /// Window was resized, but this script was not active, so don't reflow yet
    fn handle_resize_inactive_msg(&mut self, id: PipelineId, new_size: Size2D<uint>) {
        let page = self.page_tree.find(id).expect("Received resize message for PipelineId not associated
            with a page in the page tree. This is a bug.").page;
        page.window_size = new_size;
        let last_loaded_url = replace(&mut page.url, None);
        for url in last_loaded_url.iter() {
            page.url = Some((url.first(), true));
        }
    }

    fn handle_exit_window_msg(&mut self, id: PipelineId) {
        debug!("script task handling exit window msg");
        self.handle_exit_pipeline_msg(id);

        // TODO(tkuehn): currently there is only one window,
        // so this can afford to be naive and just shut down the
        // compositor. In the future it'll need to be smarter.
        self.compositor.close();
    }

    /// Handles a request to exit the script task and shut down layout.
    /// Returns true if the script task should shut down and false otherwise.
    fn handle_exit_pipeline_msg(&mut self, id: PipelineId) -> bool {
        // If root is being exited, shut down all pages
        if self.page_tree.page.id == id {
            for page in self.page_tree.iter() {
                debug!("shutting down layout for root page {:?}", page.id);
                shut_down_layout(page)
            }
            return true
        }

        // otherwise find just the matching page and exit all sub-pages
        match self.page_tree.remove(id) {
            Some(ref mut page_tree) => {
                for page in page_tree.iter() {
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
    fn load(&mut self, pipeline_id: PipelineId, url: Url) {
        debug!("ScriptTask: loading {:?} on page {:?}", url, pipeline_id);

        let page = self.page_tree.find(pipeline_id).expect("ScriptTask: received a load
            message for a layout channel that is not associated with this script task. This
            is a bug.").page;
        let last_loaded_url = replace(&mut page.url, None);
        for loaded in last_loaded_url.iter() {
            let (ref loaded, needs_reflow) = *loaded;
            if *loaded == url {
                page.url = Some((loaded.clone(), false));
                if needs_reflow {
                    page.damage(ContentChangedDocumentDamage);
                    page.reflow(ReflowForDisplay, self.chan.clone(), self.compositor);
                }
                return;
            }
        }

        let cx = self.js_runtime.cx();
        // Create the window and document objects.
        let window = Window::new(cx.ptr,
                                 page,
                                 self.chan.clone(),
                                 self.compositor,
                                 self.image_cache_task.clone());
        page.initialize_js_info(cx, window.reflector().get_jsobject());

        RegisterBindings::Register(page.js_info.get_ref().js_compartment);

        self.compositor.set_ready_state(Loading);
        // Parse HTML.
        //
        // Note: We can parse the next document in parallel with any previous documents.
        let document = HTMLDocument::new(window);
        let html_parsing_result = hubbub_html_parser::parse_html(cx.ptr,
                                                                 document,
                                                                 url.clone(),
                                                                 self.resource_task.clone(),
                                                                 self.image_cache_task.clone(),
                                                                 page.next_subpage_id.clone());

        let HtmlParserResult {
            discovery_port
        } = html_parsing_result;

        // Create the root frame.
        page.frame = Some(Frame {
            document: document,
            window: window,
        });

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
                    page.next_subpage_id = SubpageId(*subpage_id + 1);
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
        document.document().content_changed();

        let fragment = url.fragment.as_ref().map(|ref fragment| fragment.to_owned());

        // No more reflow required
        page.url = Some((url, false));

        // Receive the JavaScript scripts.
        assert!(js_scripts.is_some());
        let js_scripts = js_scripts.take_unwrap();
        debug!("js_scripts: {:?}", js_scripts);

        // Define debug functions.
        let compartment = page.js_info.get_ref().js_compartment;
        let cx = page.js_info.get_ref().js_context;
        compartment.define_functions(debug_fns);

        // Evaluate every script in the document.
        for file in js_scripts.iter() {
            let _ = cx.evaluate_script(compartment.global_obj,
                                       file.data.clone(),
                                       file.url.to_str(),
                                       1);
        }

        // We have no concept of a document loader right now, so just dispatch the
        // "load" event as soon as we've finished executing all scripts parsed during
        // the initial load.
        let event = Event::new(window);
        event.mut_event().InitEvent(~"load", false, false);
        let doctarget = AbstractEventTarget::from_document(document);
        let wintarget = AbstractEventTarget::from_window(window);
        window.eventtarget.dispatch_event_with_target(wintarget, Some(doctarget), event);

        page.fragment_node = fragment.map_default(None, |fragid| self.find_fragment_node(page, fragid));
    }

    fn find_fragment_node(&self, page: &mut Page, fragid: ~str) -> Option<AbstractNode> {
        let document = page.frame.expect("root frame is None").document; 
        match document.document().GetElementById(fragid.to_owned()) {
            Some(node) => Some(node),
            None => {
                let doc_node = AbstractNode::from_document(document);
                let mut anchors = doc_node.traverse_preorder().filter(|node| node.is_anchor_element());
                anchors.find(|node| {
                    node.with_imm_element(|elem| {
                        match elem.get_attr(Null, "name") {
                            Some(name) => eq_slice(name, fragid),
                            None => false
                        }
                    })
                })
            }
        }
    }

    fn scroll_fragment_point(&self, pipeline_id: PipelineId, page: &mut Page, node: AbstractNode) {
        let (port, chan) = Chan::new();
        match page.query_layout(ContentBoxQuery(node, chan), port) {
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
    fn handle_event(&mut self, pipeline_id: PipelineId, event: Event_) {
        let page = self.page_tree.find(pipeline_id).expect("ScriptTask: received an event
            message for a layout channel that is not associated with this script task. This
            is a bug.").page;

        match event {
            ResizeEvent(new_width, new_height) => {
                debug!("script got resize event: {:u}, {:u}", new_width, new_height);

                page.window_size = Size2D(new_width, new_height);

                if page.frame.is_some() {
                    page.damage(ReflowDocumentDamage);
                    page.reflow(ReflowForDisplay, self.chan.clone(), self.compositor)
                }
                match page.fragment_node.take() {
                    Some(node) => self.scroll_fragment_point(pipeline_id, page, node),
                    None => {}
                }
            }

            // FIXME(pcwalton): This reflows the entire document and is not incremental-y.
            ReflowEvent => {
                debug!("script got reflow event");

                if page.frame.is_some() {
                    page.damage(MatchSelectorsDocumentDamage);
                    page.reflow(ReflowForDisplay, self.chan.clone(), self.compositor)
                }
            }

            ClickEvent(_button, point) => {
                debug!("ClickEvent: clicked at {:?}", point);

                let document = page.frame.expect("root frame is None").document;
                let root = document.document().GetDocumentElement();
                if root.is_none() {
                    return;
                }
                let (port, chan) = Chan::new();
                match page.query_layout(HitTestQuery(root.unwrap(), point, chan), port) {
                    Ok(node) => match node {
                        HitTestResponse(node) => {
                            debug!("clicked on {:s}", node.debug_str());
                            let mut node = node;
                            // traverse node generations until a node that is an element is found
                            while !node.is_element() {
                                match node.parent_node() {
                                    Some(parent) => {
                                        node = parent;
                                    }
                                    None => break
                                }
                            }
                            if node.is_element() {
                                node.with_imm_element(|element| {
                                    if "a" == element.tag_name {
                                        self.load_url_from_element(page, element)
                                    }
                                })
                            }
                        }
                    },
                    Err(()) => {
                        debug!("layout query error");
                    }
                }
            }
            MouseDownEvent(..) => {}
            MouseUpEvent(..) => {}
        }
    }

    fn load_url_from_element(&self, page: @mut Page, element: &Element) {
        // if the node's element is "a," load url from href attr
        let attr = element.get_attr(Null, "href");
        for href in attr.iter() {
            debug!("ScriptTask: clicked on link to {:s}", *href);
            let click_frag = href.starts_with("#");
            let current_url = page.url.as_ref().map(|&(ref url, _)| {
                url.clone()
            });
            debug!("ScriptTask: current url is {:?}", current_url);
            let url = make_url(href.to_owned(), current_url);

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
fn shut_down_layout(page: @mut Page) {
    page.join_layout();

    // Tell the layout task to begin shutting down.
    let (response_port, response_chan) = Chan::new();
    page.layout_chan.send(layout_interface::PrepareToExitMsg(response_chan));
    response_port.recv();

    // Destroy all nodes. Setting frame and js_info to None will trigger our
    // compartment to shutdown, run GC, etc.
    page.frame = None;
    page.js_info = None;

    // Destroy the layout task. If there were node leaks, layout will now crash safely.
    page.layout_chan.send(layout_interface::ExitNowMsg);
}
