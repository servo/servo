/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

/// The script task is the task that owns the DOM in memory, runs JavaScript, and spawns parsing
/// and layout tasks.

use servo_msg::compositor_msg::{ScriptListener, Loading, PerformingLayout};
use servo_msg::compositor_msg::FinishedLoading;
use dom::bindings::utils::GlobalStaticData;
use dom::document::AbstractDocument;
use dom::element::Element;
use dom::event::{Event_, ResizeEvent, ReflowEvent, ClickEvent, MouseDownEvent, MouseUpEvent};
use dom::htmldocument::HTMLDocument;
use dom::node::{define_bindings};
use dom::window::Window;
use layout_interface::{AddStylesheetMsg, DocumentDamage};
use layout_interface::{DocumentDamageLevel, HitTestQuery, HitTestResponse, LayoutQuery};
use layout_interface::{LayoutChan, MatchSelectorsDocumentDamage, QueryMsg, Reflow};
use layout_interface::{ReflowDocumentDamage, ReflowForDisplay, ReflowGoal};
use layout_interface::ReflowMsg;
use layout_interface;
use servo_msg::constellation_msg::{ConstellationChan, LoadUrlMsg, NavigationDirection};
use servo_msg::constellation_msg::{PipelineId, SubpageId, RendererReadyMsg, ResizedWindowBroadcast};
use servo_msg::constellation_msg::{LoadIframeUrlMsg};
use servo_msg::constellation_msg;

use newcss::stylesheet::Stylesheet;

use std::cell::Cell;
use std::comm;
use std::comm::{Port, SharedChan, Select2};
use std::io::read_whole_file;
use std::ptr::null;
use std::task::{SingleThreaded, task};
use std::util::replace;
use dom::window::TimerData;
use geom::size::Size2D;
use html::hubbub_html_parser::HtmlParserResult;
use html::hubbub_html_parser::{HtmlDiscoveredStyle, HtmlDiscoveredIFrame, HtmlDiscoveredScript};
use html::hubbub_html_parser;
use js::JSVAL_NULL;
use js::global::{global_class, debug_fns};
use js::glue::RUST_JSVAL_TO_OBJECT;
use js::jsapi::JSContext;
use js::jsapi::{JS_CallFunctionValue, JS_GetContextPrivate};
use js::rust::{Compartment, Cx};
use js;
use servo_net::image_cache_task::ImageCacheTask;
use servo_net::resource_task::ResourceTask;
use servo_util::tree::TreeNodeRef;
use servo_util::url::make_url;
use extra::net::url::Url;
use extra::net::url;
use extra::future::{from_value, Future};

/// Messages used to control the script task.
pub enum ScriptMsg {
    /// Loads a new URL on the specified pipeline.
    LoadMsg(PipelineId, Url),
    /// Gives a channel and ID to a layout task, as well as the ID of that layout's parent
    AttachLayoutMsg(NewLayoutInfo),
    /// Executes a standalone script.
    ExecuteMsg(PipelineId, Url),
    /// Instructs the script task to send a navigate message to the constellation.
    NavigateMsg(NavigationDirection),
    /// Sends a DOM event.
    SendEventMsg(PipelineId, Event_),
    /// Fires a JavaScript timeout.
    FireTimerMsg(PipelineId, ~TimerData),
    /// Notifies script that reflow is finished.
    ReflowCompleteMsg(PipelineId),
    /// Notifies script that window has been resized but to not take immediate action.
    ResizeInactiveMsg(Size2D<uint>),
    /// Exits the constellation.
    ExitMsg,
}

pub struct NewLayoutInfo {
    old_id: PipelineId,
    new_id: PipelineId,
    layout_chan: LayoutChan,
    size_future: Future<Size2D<uint>>,
}

/// Encapsulates external communication with the script task.
#[deriving(Clone)]
pub struct ScriptChan {
    /// The channel used to send messages to the script task.
    chan: SharedChan<ScriptMsg>,
}

impl ScriptChan {
    /// Creates a new script chan.
    pub fn new(chan: Chan<ScriptMsg>) -> ScriptChan {
        ScriptChan {
            chan: SharedChan::new(chan)
        }
    }
    pub fn send(&self, msg: ScriptMsg) {
        self.chan.send(msg);
    }
}

/// Encapsulates a handle to a frame and its associate layout information
pub struct Page {
    /// Pipeline id associated with this page.
    id: PipelineId,

    /// The outermost frame containing the document, window, and page URL.
    frame: Option<Frame>,

    /// A handle for communicating messages to the layout task.
    layout_chan: LayoutChan,

    /// The port that we will use to join layout. If this is `None`, then layout is not running.
    layout_join_port: Option<Port<()>>,

    /// What parts of the document are dirty, if any.
    damage: Option<DocumentDamage>,

    /// The current size of the window, in pixels.
    window_size: Future<Size2D<uint>>,

    js_info: Option<JSPageInfo>,

    /// Cached copy of the most recent url loaded by the script
    /// TODO(tkuehn): this currently does not follow any particular caching policy
    /// and simply caches pages forever (!). The bool indicates if reflow is required
    /// when reloading.
    url: Option<(Url, bool)>,

    next_subpage_id: SubpageId,
}

pub struct PageTree {
    page: @mut Page,
    inner: ~[PageTree],
}

pub struct PageTreeIterator<'self> {
    priv stack: ~[&'self mut PageTree],
}

impl PageTree {
    fn new(id: PipelineId, layout_chan: LayoutChan, size_future: Future<Size2D<uint>>) -> PageTree {
        PageTree {
            page: @mut Page {
                id: id,
                frame: None,
                layout_chan: layout_chan,
                layout_join_port: None,
                damage: None,
                window_size: size_future,
                js_info: None,
                url: None,
                next_subpage_id: SubpageId(0),
            },
            inner: ~[],
        }
    }

    pub fn find<'a> (&'a mut self, id: PipelineId) -> Option<&'a mut PageTree> {
        if self.page.id == id { return Some(self); }
        for self.inner.mut_iter().advance |page_tree| {
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
}

impl<'self> Iterator<@mut Page> for PageTreeIterator<'self> {
    fn next(&mut self) -> Option<@mut Page> {
        if !self.stack.is_empty() {
            let next = self.stack.pop();
            {
                for next.inner.mut_iter().advance |child| {
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
    fn damage(&mut self, level: DocumentDamageLevel) {
        match self.damage {
            None => {}
            Some(ref mut damage) => {
                // FIXME(pcwalton): This is wrong. We should trace up to the nearest ancestor.
                damage.root = do self.frame.get_ref().document.with_base |doc| { doc.root };
                damage.level.add(level);
                return
            }
        }

        self.damage = Some(DocumentDamage {
            root: do self.frame.get_ref().document.with_base |doc| { doc.root },
            level: level,
        })
    }

    /// Sends a ping to layout and waits for the response. The response will arrive when the
    /// layout task has finished any pending request messages.
    fn join_layout(&mut self) {
        if self.layout_join_port.is_some() {
            let join_port = replace(&mut self.layout_join_port, None);
            match join_port {
                Some(ref join_port) => {
                    if !join_port.peek() {
                        info!("script: waiting on layout");
                    }

                    join_port.recv();

                    debug!("script: layout joined")
                }
                None => fail!(~"reader forked but no join port?"),
            }
        }
    }

    /// Sends the given query to layout.
    pub fn query_layout<T: Send>(&mut self,
                                 query: LayoutQuery,
                                 response_port: Port<Result<T, ()>>)
                                 -> Result<T,()> {
        self.join_layout();
        self.layout_chan.send(QueryMsg(query));
        response_port.recv()
    }

    /// This method will wait until the layout task has completed its current action, join the
    /// layout task, and then request a new layout run. It won't wait for the new layout
    /// computation to finish.
    ///
    /// This function fails if there is no root frame.
    fn reflow(&mut self, goal: ReflowGoal, script_chan: ScriptChan, compositor: @ScriptListener) {
    
        debug!("script: performing reflow for goal %?", goal);

        // Now, join the layout so that they will see the latest changes we have made.
        self.join_layout();

        // Tell the user that we're performing layout.
        compositor.set_ready_state(PerformingLayout);

        // Layout will let us know when it's done.
        let (join_port, join_chan) = comm::stream();
        self.layout_join_port = Some(join_port);

        match self.frame {
            None => fail!(~"Tried to relayout with no root frame!"),
            Some(ref frame) => {
                // Send new document and relevant styles to layout.
                let reflow = ~Reflow {
                    document_root: do frame.document.with_base |doc| { doc.root },
                    url: copy self.url.get_ref().first(),
                    goal: goal,
                    window_size: self.window_size.get(),
                    script_chan: script_chan,
                    script_join_chan: join_chan,
                    damage: replace(&mut self.damage, None).unwrap(),
                };

                self.layout_chan.send(ReflowMsg(reflow))
            }
        }

        debug!("script: layout forked")
    }

    /// Reflows the entire document.
    ///
    /// FIXME: This should basically never be used.
    pub fn reflow_all(&mut self, goal: ReflowGoal, script_chan: ScriptChan, compositor: @ScriptListener) {
        if self.frame.is_some() {
            self.damage(MatchSelectorsDocumentDamage);
        }

        self.reflow(goal, script_chan, compositor)
    }

    pub fn initialize_js_info(&mut self, js_context: @Cx) {
        // Note that the order that these variables are initialized is _not_ arbitrary. Switching them around
        // can -- and likely will -- lead to things breaking.

        js_context.set_default_options_and_version();
        js_context.set_logging_error_reporter();

        let compartment = match js_context.new_compartment(global_class) {
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
            bindings_initialized: false,
            js_compartment: compartment,
            js_context: js_context,
        });
    }

}

/// Information for one frame in the browsing context.
pub struct Frame {
    document: AbstractDocument,
    window: @mut Window,

}

/// Encapsulation of the javascript information associated with each frame.
pub struct JSPageInfo {
    /// Global static data related to the DOM.
    dom_static: GlobalStaticData,
    /// Flag indicating if the JS bindings have been initialized.
    bindings_initialized: bool,
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
               initial_size: Future<Size2D<uint>>)
               -> @mut ScriptTask {
        let js_runtime = js::rust::rt();

        let script_task = @mut ScriptTask {
            page_tree: PageTree::new(id, layout_chan, initial_size),

            image_cache_task: img_cache_task,
            resource_task: resource_task,

            port: port,
            chan: chan,
            constellation_chan: constellation_chan,
            compositor: compositor,

            js_runtime: js_runtime,
        };

        script_task.page_tree.page.initialize_js_info(script_task.js_runtime.cx());
        script_task
    }

    /// Starts the script task. After calling this method, the script task will loop receiving
    /// messages on its port.
    pub fn start(&mut self) {
        while self.handle_msg() {
            // Go on...
        }
    }

    pub fn create<C: ScriptListener + Send>(id: PipelineId,
                                            compositor: C,
                                            layout_chan: LayoutChan,
                                            port: Port<ScriptMsg>,
                                            chan: ScriptChan,
                                            constellation_chan: ConstellationChan,
                                            resource_task: ResourceTask,
                                            image_cache_task: ImageCacheTask,
                                            initial_size: Future<Size2D<uint>>) {
        let compositor = Cell::new(compositor);
        let port = Cell::new(port);
        let initial_size = Cell::new(initial_size);
        // FIXME: rust#6399
        let mut the_task = task();
        the_task.sched_mode(SingleThreaded);
        do spawn {
            let script_task = ScriptTask::new(id,
                                              @compositor.take() as @ScriptListener,
                                              layout_chan.clone(),
                                              port.take(),
                                              chan.clone(),
                                              constellation_chan.clone(),
                                              resource_task.clone(),
                                              image_cache_task.clone(),
                                              initial_size.take());
            script_task.start();
        }
    }

    /// Handles an incoming control message.
    fn handle_msg(&mut self) -> bool {
        match self.port.recv() {
            // TODO(tkuehn) need to handle auxiliary layouts for iframes
            AttachLayoutMsg(new_layout_info) => self.handle_new_layout(new_layout_info),
            LoadMsg(id, url) => self.load(id, url),
            ExecuteMsg(id, url) => self.handle_execute_msg(id, url),
            SendEventMsg(id, event) => self.handle_event(id, event),
            FireTimerMsg(id, timer_data) => self.handle_fire_timer_msg(id, timer_data),
            NavigateMsg(direction) => self.handle_navigate_msg(direction),
            ReflowCompleteMsg(id) => self.handle_reflow_complete_msg(id),
            ResizeInactiveMsg(new_size) => self.handle_resize_inactive_msg(new_size),
            ExitMsg => {
                self.handle_exit_msg();
                return false
            }
        }
        true
    }

    fn handle_new_layout(&mut self, new_layout_info: NewLayoutInfo) {
        let NewLayoutInfo {
            old_id,
            new_id,
            layout_chan,
            size_future
        } = new_layout_info;

        let parent_page_tree = self.page_tree.find(old_id).expect("ScriptTask: received a layout
            whose parent has a PipelineId which does not correspond to a pipeline in the script
            task's page tree. This is a bug.");
        let new_page_tree = PageTree::new(new_id, layout_chan, size_future);
        new_page_tree.page.initialize_js_info(self.js_runtime.cx());

        parent_page_tree.inner.push(new_page_tree);
    }

    /// Handles a request to execute a script.
    fn handle_execute_msg(&mut self, id: PipelineId, url: Url) {
        debug!("script: Received url `%s` to execute", url::to_str(&url));

        let page_tree = self.page_tree.find(id).expect("ScriptTask: received fire timer msg for a
            pipeline ID not associated with this script task. This is a bug.");
        let js_info = page_tree.page.js_info.get_ref();

        match read_whole_file(&Path(url.path)) {
            Err(msg) => println(fmt!("Error opening %s: %s", url::to_str(&url), msg)),

            Ok(bytes) => {
                js_info.js_compartment.define_functions(debug_fns);
                js_info.js_context.evaluate_script(js_info.js_compartment.global_obj,
                                                   bytes,
                                                   copy url.path,
                                                   1);
            }
        }
    }

    /// Handles a timer that fired.
    fn handle_fire_timer_msg(&mut self, id: PipelineId, timer_data: ~TimerData) {
        let page = self.page_tree.find(id).expect("ScriptTask: received fire timer msg for a
            pipeline ID not associated with this script task. This is a bug.").page;
        unsafe {
            let js_info = page.js_info.get_ref();
            let this_value = if timer_data.args.len() > 0 {
                RUST_JSVAL_TO_OBJECT(timer_data.args[0])
            } else {
                js_info.js_compartment.global_obj.ptr
            };

            // TODO: Support extra arguments. This requires passing a `*JSVal` array as `argv`.
            let rval = JSVAL_NULL;
            JS_CallFunctionValue(js_info.js_context.ptr,
                                 this_value,
                                 timer_data.funval,
                                 0,
                                 null(),
                                 &rval);

        }
        // We don't know what the script changed, so for now we will do a total redisplay.
        page.reflow_all(ReflowForDisplay, self.chan.clone(), self.compositor);
    }

    /// Handles a notification that reflow completed.
    fn handle_reflow_complete_msg(&mut self, pipeline_id: PipelineId) {
        self.page_tree.find(pipeline_id).expect("ScriptTask: received a load
            message for a layout channel that is not associated with this script task. This
            is a bug.").page.layout_join_port = None;
        self.constellation_chan.send(RendererReadyMsg(pipeline_id));
        self.compositor.set_ready_state(FinishedLoading);
    }

    /// Handles a navigate forward or backward message.
    /// TODO(tkuehn): is it ever possible to navigate only on a subframe?
    fn handle_navigate_msg(&self, direction: NavigationDirection) {
        self.constellation_chan.send(constellation_msg::NavigateMsg(direction));
    }

    /// Window was resized, but this script was not active, so don't reflow yet
    fn handle_resize_inactive_msg(&mut self, new_size: Size2D<uint>) {
        self.page_tree.page.window_size = from_value(new_size);
        let last_loaded_url = replace(&mut self.page_tree.page.url, None);
        for last_loaded_url.iter().advance |last_loaded_url| {
            self.page_tree.page.url = Some((last_loaded_url.first(), true));
        }
    }

    /// Handles a request to exit the script task and shut down layout.
    fn handle_exit_msg(&mut self) {
        for self.page_tree.iter().advance |page| {
            page.join_layout();
            do page.frame.get().document.with_mut_base |doc| {
                doc.teardown();
            }
            page.layout_chan.send(layout_interface::ExitMsg);
        }
    }

    /// The entry point to document loading. Defines bindings, sets up the window and document
    /// objects, parses HTML and CSS, and kicks off initial layout.
    fn load(&mut self, pipeline_id: PipelineId, url: Url) {
        debug!("ScriptTask: loading %?", url);

        let page = self.page_tree.find(pipeline_id).expect("ScriptTask: received a load
            message for a layout channel that is not associated with this script task. This
            is a bug.").page;
        let last_loaded_url = replace(&mut page.url, None);
        for last_loaded_url.iter().advance |last_loaded_url| {
            let (ref last_loaded_url, needs_reflow) = *last_loaded_url;
            if *last_loaded_url == url {
                page.url = Some((last_loaded_url.clone(), false));
                if needs_reflow {
                    page.reflow_all(ReflowForDisplay, self.chan.clone(), self.compositor);
                }
                return;
            }
        }
        
        {
            let js_info = page.js_info.get_mut_ref();
            // Define the script DOM bindings.
            //
            // FIXME: Can this be done earlier, to save the flag?
            if !js_info.bindings_initialized {
                define_bindings(js_info.js_compartment);
                js_info.bindings_initialized = true;
            }
        }

        self.compositor.set_ready_state(Loading);
        // Parse HTML.
        //
        // Note: We can parse the next document in parallel with any previous documents.
        let html_parsing_result = hubbub_html_parser::parse_html(page.js_info.get_ref().js_compartment.cx.ptr,
                                                                 url.clone(),
                                                                 self.resource_task.clone(),
                                                                 self.image_cache_task.clone(),
                                                                 page.next_subpage_id.clone());

        let HtmlParserResult {root, discovery_port} = html_parsing_result;

        // Create the window and document objects.
        let window = {
            // Need an extra block here due to Rust #6248
            //
            // FIXME(Servo #655): Unrelated to the Rust #6248 warning, this is fundamentally
            // unsafe because the box could go away or get moved while we're holding this raw
            // pointer.  We think it's safe here because the main task will hold onto the box,
            // and because the current refcounting implementation of @ doesn't move.
            let page = &mut *page;
            Window::new(page, self.chan.clone(), self.compositor)
        };
        let document = HTMLDocument::new(root, Some(window));

        // Tie the root into the document.
        do root.with_mut_base |base| {
            base.add_to_doc(document)
        }

        // Create the root frame.
        page.frame = Some(Frame {
            document: document,
            window: window,
        });
        page.url = Some((url.clone(), true));

        // Send style sheets over to layout.
        //
        // FIXME: These should be streamed to layout as they're parsed. We don't need to stop here
        // in the script task.

        let mut js_scripts = None;
        loop {
            match discovery_port.try_recv() {
                Some(HtmlDiscoveredScript(scripts)) => {
                    assert!(js_scripts.is_none());
                    js_scripts = Some(scripts);
                }
                Some(HtmlDiscoveredStyle(sheet)) => {
                    page.layout_chan.send(AddStylesheetMsg(sheet));
                }
                Some(HtmlDiscoveredIFrame((iframe_url, subpage_id, size_future))) => {
                    page.next_subpage_id = SubpageId(*subpage_id + 1);
                    self.constellation_chan.send(LoadIframeUrlMsg(iframe_url,
                                                                  pipeline_id,
                                                                  subpage_id,
                                                                  size_future));
                }
                None => break
            }
        }

        // Receive the JavaScript scripts.
        assert!(js_scripts.is_some());
        let js_scripts = js_scripts.swap_unwrap();
        debug!("js_scripts: %?", js_scripts);

        // Perform the initial reflow.
        page.damage = Some(DocumentDamage {
            root: root,
            level: MatchSelectorsDocumentDamage,
        });
        page.reflow(ReflowForDisplay, self.chan.clone(), self.compositor);
        page.url = Some((url, false));

        // Define debug functions.
        let js_info = page.js_info.get_ref();
        js_info.js_compartment.define_functions(debug_fns);

        // Evaluate every script in the document.
        for js_scripts.iter().advance |bytes| {
            let _ = js_info.js_context.evaluate_script(js_info.js_compartment.global_obj,
                                                       bytes.clone(),
                                                       ~"???",
                                                       1);
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
                debug!("script got resize event: %u, %u", new_width, new_height);

                page.window_size = from_value(Size2D(new_width, new_height));

                if page.frame.is_some() {
                    page.damage(ReflowDocumentDamage);
                    page.reflow(ReflowForDisplay, self.chan.clone(), self.compositor)
                }

                self.constellation_chan.send(ResizedWindowBroadcast(page.window_size.get().clone()));
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
                debug!("ClickEvent: clicked at %?", point);

                let root = do page.frame.expect("root frame is None").document.with_base |doc| {
                    doc.root
                };
                let (port, chan) = comm::stream();
                match page.query_layout(HitTestQuery(root, point, chan), port) {
                    Ok(node) => match node {
                        HitTestResponse(node) => {
                            debug!("clicked on %s", node.debug_str());
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
                                do node.with_imm_element |element| {
                                    if "a" == element.tag_name {
                                        self.load_url_from_element(page, element)
                                    }
                                }
                            }
                        }
                    },
                    Err(()) => {
                        debug!(fmt!("layout query error"));
                    }
                }
            }
            MouseDownEvent(*) => {}
            MouseUpEvent(*) => {}
        }
    }

    priv fn load_url_from_element(&self, page: @mut Page, element: &Element) {
        // if the node's element is "a," load url from href attr
        let href = element.get_attr("href");
        for href.iter().advance |href| {
            debug!("ScriptTask: clicked on link to %s", *href);
            let current_url = do page.url.map |&(ref url, _)| {
                url.clone()
            };
            debug!("ScriptTask: current url is %?", current_url);
            let url = make_url(href.to_owned(), current_url);
            self.constellation_chan.send(LoadUrlMsg(page.id, url, from_value(page.window_size.get())));
        }
    }
}

