/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

/// The script task is the task that owns the DOM in memory, runs JavaScript, and spawns parsing
/// and layout tasks.

use servo_msg::compositor_msg::{ScriptListener, Loading, PerformingLayout};
use servo_msg::compositor_msg::FinishedLoading;
use dom::bindings::utils::GlobalStaticData;
use dom::document::Document;
use dom::element::Element;
use dom::event::{Event, ResizeEvent, ReflowEvent, ClickEvent, MouseDownEvent, MouseUpEvent};
use dom::node::{AbstractNode, ScriptView, define_bindings};
use dom::window::Window;
use layout_interface::{AddStylesheetMsg, DocumentDamage};
use layout_interface::{DocumentDamageLevel, HitTestQuery, HitTestResponse, LayoutQuery};
use layout_interface::{LayoutChan, MatchSelectorsDocumentDamage, QueryMsg, Reflow};
use layout_interface::{ReflowDocumentDamage, ReflowForDisplay, ReflowForScriptQuery, ReflowGoal};
use layout_interface::ReflowMsg;
use layout_interface;
use servo_msg::constellation_msg::{ConstellationChan, LoadUrlMsg, NavigationDirection};
use servo_msg::constellation_msg::{RendererReadyMsg, ResizedWindowBroadcast};
use servo_msg::constellation_msg;

use std::cast::transmute;
use std::cell::Cell;
use std::comm;
use std::comm::{Port, SharedChan};
use std::io::read_whole_file;
use std::local_data;
use std::ptr::null;
use std::task::{SingleThreaded, task};
use std::util::replace;
use dom::window::TimerData;
use geom::size::Size2D;
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

/// Messages used to control the script task.
pub enum ScriptMsg {
    /// Loads a new URL.
    LoadMsg(Url),
    /// Executes a standalone script.
    ExecuteMsg(Url),
    /// Instructs the script task to send a navigate message to the constellation.
    NavigateMsg(NavigationDirection),
    /// Sends a DOM event.
    SendEventMsg(Event),
    /// Fires a JavaScript timeout.
    FireTimerMsg(~TimerData),
    /// Notifies script that reflow is finished.
    ReflowCompleteMsg,
    /// Notifies script that window has been resized but to not take immediate action.
    ResizeInactiveMsg(Size2D<uint>),
    /// Exits the constellation.
    ExitMsg,
}

/// Encapsulates external communication with the script task.
#[deriving(Clone)]
pub struct ScriptChan {
    /// The channel used to send messages to the script task.
    chan: SharedChan<ScriptMsg>,
}

impl ScriptChan {
    /// Creates a new script task.
    pub fn new(chan: Chan<ScriptMsg>) -> ScriptChan {
        ScriptChan {
            chan: SharedChan::new(chan)
        }
    }
    pub fn send(&self, msg: ScriptMsg) {
        self.chan.send(msg);
    }
}

/// Information for one frame in the browsing context.
pub struct Frame {
    document: @mut Document,
    window: @mut Window,
    url: Url,
}

/// Information for an entire page. Pages are top-level browsing contexts and can contain multiple
/// frames.
///
/// FIXME: Rename to `Page`, following WebKit?
pub struct ScriptTask {
    /// A unique identifier to the script's pipeline
    id: uint,
    /// A handle to the layout task.
    layout_chan: LayoutChan,
    /// A handle to the image cache task.
    image_cache_task: ImageCacheTask,
    /// A handle to the resource task.
    resource_task: ResourceTask,

    /// The port that we will use to join layout. If this is `None`, then layout is not currently
    /// running.
    layout_join_port: Option<Port<()>>,
    /// The port on which we receive messages (load URL, exit, etc.)
    script_port: Port<ScriptMsg>,
    /// A channel for us to hand out when we want some other task to be able to send us script
    /// messages.
    script_chan: ScriptChan,

    /// For communicating load url messages to the constellation
    constellation_chan: ConstellationChan,
    /// For permission to communicate ready state messages to the compositor
    compositor: @ScriptListener,

    /// The JavaScript runtime.
    js_runtime: js::rust::rt,
    /// The JavaScript context.
    js_context: @Cx,
    /// The JavaScript compartment.
    js_compartment: @mut Compartment,

    /// Global static data related to the DOM.
    dom_static: GlobalStaticData,
    /// Whether the JS bindings have been initialized.
    bindings_initialized: bool,

    /// The outermost frame. This frame contains the document, window, and page URL.
    root_frame: Option<Frame>,

    /// The current size of the window, in pixels.
    window_size: Size2D<uint>,
    /// What parts of the document are dirty, if any.
    damage: Option<DocumentDamage>,

    /// Cached copy of the most recent url loaded by the script
    /// TODO(tkuehn): this currently does not follow any particular caching policy
    /// and simply caches pages forever (!). The bool indicates if reflow is required
    last_loaded_url: Option<(Url, bool)>,
}

fn global_script_context_key(_: @ScriptTask) {}

/// Returns this task's script context singleton.
pub fn global_script_context() -> @ScriptTask {
    unsafe {
        local_data::local_data_get(global_script_context_key).get()
    }
}

/// Returns the script task from the JS Context.
///
/// FIXME: Rename to `script_context_from_js_context`.
pub fn task_from_context(js_context: *JSContext) -> *mut ScriptTask {
    unsafe {
        JS_GetContextPrivate(js_context) as *mut ScriptTask
    }
}

#[unsafe_destructor]
impl Drop for ScriptTask {
    fn drop(&self) {
        unsafe {
            let _ = local_data::local_data_pop(global_script_context_key);
        }
    }
}

impl ScriptTask {
    /// Creates a new script task.
    pub fn new(id: uint,
               compositor: @ScriptListener,
               layout_chan: LayoutChan,
               script_port: Port<ScriptMsg>,
               script_chan: ScriptChan,
               constellation_chan: ConstellationChan,
               resource_task: ResourceTask,
               img_cache_task: ImageCacheTask,
               initial_size: Size2D<int>)
               -> @mut ScriptTask {
        let js_runtime = js::rust::rt();
        let js_context = js_runtime.cx();

        js_context.set_default_options_and_version();
        js_context.set_logging_error_reporter();

        let compartment = match js_context.new_compartment(global_class) {
              Ok(c) => c,
              Err(()) => fail!("Failed to create a compartment"),
        };

        let script_task = @mut ScriptTask {
            id: id,
            compositor: compositor,

            layout_chan: layout_chan,
            image_cache_task: img_cache_task,
            resource_task: resource_task,

            layout_join_port: None,
            script_port: script_port,
            script_chan: script_chan,

            constellation_chan: constellation_chan,

            js_runtime: js_runtime,
            js_context: js_context,
            js_compartment: compartment,

            dom_static: GlobalStaticData(),
            bindings_initialized: false,

            root_frame: None,

            window_size: Size2D(initial_size.width as uint, initial_size.height as uint),
            damage: None,

            last_loaded_url: None,
        };
        // Indirection for Rust Issue #6248, dynamic freeze scope artifically extended
        let script_task_ptr = {
            let borrowed_ctx= &mut *script_task;
            borrowed_ctx as *mut ScriptTask
        };

        unsafe {
            js_context.set_cx_private(script_task_ptr as *());
            local_data::local_data_set(global_script_context_key, transmute(script_task))
        }

        script_task
    }

    /// Starts the script task. After calling this method, the script task will loop receiving
    /// messages on its port.
    pub fn start(&mut self) {
        while self.handle_msg() {
            // Go on...
        }
    }

    pub fn create<C: ScriptListener + Send>(id: uint,
                  compositor: C,
                  layout_chan: LayoutChan,
                  script_port: Port<ScriptMsg>,
                  script_chan: ScriptChan,
                  constellation_chan: ConstellationChan,
                  resource_task: ResourceTask,
                  image_cache_task: ImageCacheTask,
                  initial_size: Size2D<int>) {
        let compositor = Cell::new(compositor);
        let script_port = Cell::new(script_port);
        // FIXME: rust#6399
        let mut the_task = task();
        the_task.sched_mode(SingleThreaded);
        do spawn {
            let script_task = ScriptTask::new(id,
                                              @compositor.take() as @ScriptListener,
                                              layout_chan.clone(),
                                              script_port.take(),
                                              script_chan.clone(),
                                              constellation_chan.clone(),
                                              resource_task.clone(),
                                              image_cache_task.clone(),
                                              initial_size);
            script_task.start();
        }
    }

    /// Handles an incoming control message.
    fn handle_msg(&mut self) -> bool {
        match self.script_port.recv() {
            LoadMsg(url) => self.load(url),
            ExecuteMsg(url) => self.handle_execute_msg(url),
            SendEventMsg(event) => self.handle_event(event),
            FireTimerMsg(timer_data) => self.handle_fire_timer_msg(timer_data),
            NavigateMsg(direction) => self.handle_navigate_msg(direction),
            ReflowCompleteMsg => self.handle_reflow_complete_msg(),
            ResizeInactiveMsg(new_size) => self.handle_resize_inactive_msg(new_size),
            ExitMsg => {
                self.handle_exit_msg();
                return false
            }
        }
        true
    }

    /// Handles a request to execute a script.
    fn handle_execute_msg(&self, url: Url) {
        debug!("script: Received url `%s` to execute", url::to_str(&url));

        match read_whole_file(&Path(url.path)) {
            Err(msg) => println(fmt!("Error opening %s: %s", url::to_str(&url), msg)),

            Ok(bytes) => {
                self.js_compartment.define_functions(debug_fns);
                let _ = self.js_context.evaluate_script(self.js_compartment.global_obj,
                                                        bytes,
                                                        copy url.path,
                                                        1);
            }
        }
    }

    /// Handles a timer that fired.
    fn handle_fire_timer_msg(&mut self, timer_data: ~TimerData) {
        unsafe {
            let this_value = if timer_data.args.len() > 0 {
                RUST_JSVAL_TO_OBJECT(timer_data.args[0])
            } else {
                self.js_compartment.global_obj.ptr
            };

            // TODO: Support extra arguments. This requires passing a `*JSVal` array as `argv`.
            let rval = JSVAL_NULL;
            JS_CallFunctionValue(self.js_context.ptr,
                                 this_value,
                                 timer_data.funval,
                                 0,
                                 null(),
                                 &rval);

            self.reflow(ReflowForScriptQuery)
        }
    }

    /// Handles a notification that reflow completed.
    fn handle_reflow_complete_msg(&mut self) {
        self.layout_join_port = None;
        self.constellation_chan.send(RendererReadyMsg(self.id));
        self.compositor.set_ready_state(FinishedLoading);
    }

    /// Handles a navigate forward or backward message.
    fn handle_navigate_msg(&self, direction: NavigationDirection) {
        self.constellation_chan.send(constellation_msg::NavigateMsg(direction));
    }

    /// Window was resized, but this script was not active, so don't reflow yet
    fn handle_resize_inactive_msg(&mut self, new_size: Size2D<uint>) {
        self.window_size = new_size;
        let last_loaded_url = replace(&mut self.last_loaded_url, None);
        for last_loaded_url.iter().advance |last_loaded_url| {
            self.last_loaded_url = Some((last_loaded_url.first(), true));
        }
    }

    /// Handles a request to exit the script task and shut down layout.
    fn handle_exit_msg(&mut self) {
        self.join_layout();
        for self.root_frame.iter().advance |frame| {
            frame.document.teardown();
        }

        self.layout_chan.send(layout_interface::ExitMsg)
    }

    /// The entry point to document loading. Defines bindings, sets up the window and document
    /// objects, parses HTML and CSS, and kicks off initial layout.
    fn load(&mut self, url: Url) {
        let last_loaded_url = replace(&mut self.last_loaded_url, None);
        for last_loaded_url.iter().advance |last_loaded_url| {
            let (ref last_loaded_url, needs_reflow) = *last_loaded_url;
            if *last_loaded_url == url {
                if needs_reflow {
                    self.reflow_all(ReflowForDisplay);
                    self.last_loaded_url = Some((last_loaded_url.clone(), false));
                }
                return;
            }
        }
        
        // Define the script DOM bindings.
        //
        // FIXME: Can this be done earlier, to save the flag?
        if !self.bindings_initialized {
            define_bindings(self.js_compartment);
            self.bindings_initialized = true
        }

        self.compositor.set_ready_state(Loading);
        // Parse HTML.
        //
        // Note: We can parse the next document in parallel with any previous documents.
        let html_parsing_result = hubbub_html_parser::parse_html(url.clone(),
                                                                 self.resource_task.clone(),
                                                                 self.image_cache_task.clone());

        let root_node = html_parsing_result.root;

        // Send style sheets over to layout.
        //
        // FIXME: These should be streamed to layout as they're parsed. We don't need to stop here
        // in the script task.
        loop {
              match html_parsing_result.style_port.recv() {
                  Some(sheet) => self.layout_chan.send(AddStylesheetMsg(sheet)),
                  None => break,
              }
        }

        // Receive the JavaScript scripts.
        let js_scripts = html_parsing_result.js_port.recv();
        debug!("js_scripts: %?", js_scripts);

        // Create the window and document objects.
        let window = Window::new(self.script_chan.clone(), &mut *self);
        let document = Document(root_node, Some(window));

        // Tie the root into the document.
        do root_node.with_mut_base |base| {
            base.add_to_doc(document)
        }

        // Create the root frame.
        self.root_frame = Some(Frame {
            document: document,
            window: window,
            url: url.clone(),
        });

        // Perform the initial reflow.
        self.damage = Some(DocumentDamage {
            root: root_node,
            level: MatchSelectorsDocumentDamage,
        });
        self.reflow(ReflowForDisplay);

        // Define debug functions.
        self.js_compartment.define_functions(debug_fns);

        // Evaluate every script in the document.
        for js_scripts.iter().advance |bytes| {
            let _ = self.js_context.evaluate_script(self.js_compartment.global_obj,
                                                    bytes.clone(),
                                                    ~"???",
                                                    1);
        }
        self.last_loaded_url = Some((url, false));
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

    /// This method will wait until the layout task has completed its current action, join the
    /// layout task, and then request a new layout run. It won't wait for the new layout
    /// computation to finish.
    ///
    /// This function fails if there is no root frame.
    fn reflow(&mut self, goal: ReflowGoal) {
        debug!("script: performing reflow for goal %?", goal);

        // Now, join the layout so that they will see the latest changes we have made.
        self.join_layout();

        // Tell the user that we're performing layout.
        self.compositor.set_ready_state(PerformingLayout);

        // Layout will let us know when it's done.
        let (join_port, join_chan) = comm::stream();
        self.layout_join_port = Some(join_port);

        match self.root_frame {
            None => fail!(~"Tried to relayout with no root frame!"),
            Some(ref root_frame) => {
                // Send new document and relevant styles to layout.
                let reflow = ~Reflow {
                    document_root: root_frame.document.root,
                    url: copy root_frame.url,
                    goal: goal,
                    window_size: self.window_size,
                    script_chan: self.script_chan.clone(),
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
    pub fn reflow_all(&mut self, goal: ReflowGoal) {
        for self.root_frame.iter().advance |root_frame| {
            ScriptTask::damage(&mut self.damage,
                               root_frame.document.root,
                               MatchSelectorsDocumentDamage)
        }

        self.reflow(goal)
    }

    /// Sends the given query to layout.
    pub fn query_layout<T: Send>(&mut self, query: LayoutQuery, response_port: Port<Result<T, ()>>) -> Result<T,()> {
        self.join_layout();
        self.layout_chan.send(QueryMsg(query));
        response_port.recv()
    }

    /// Adds the given damage.
    fn damage(damage: &mut Option<DocumentDamage>,
              root: AbstractNode<ScriptView>,
              level: DocumentDamageLevel) {
        match *damage {
            None => {}
            Some(ref mut damage) => {
                // FIXME(pcwalton): This is wrong. We should trace up to the nearest ancestor.
                damage.root = root;
                damage.level.add(level);
                return
            }
        }

        *damage = Some(DocumentDamage {
            root: root,
            level: level,
        })
    }

    /// This is the main entry point for receiving and dispatching DOM events.
    ///
    /// TODO: Actually perform DOM event dispatch.
    fn handle_event(&mut self, event: Event) {
        match event {
            ResizeEvent(new_width, new_height) => {
                debug!("script got resize event: %u, %u", new_width, new_height);

                self.window_size = Size2D(new_width, new_height);

                for self.root_frame.iter().advance |root_frame| {
                    ScriptTask::damage(&mut self.damage,
                                       root_frame.document.root,
                                       ReflowDocumentDamage);
                }

                if self.root_frame.is_some() {
                    self.reflow(ReflowForDisplay)
                }
                self.constellation_chan.send(ResizedWindowBroadcast(self.window_size));
            }

            // FIXME(pcwalton): This reflows the entire document and is not incremental-y.
            ReflowEvent => {
                debug!("script got reflow event");

                for self.root_frame.iter().advance |root_frame| {
                    ScriptTask::damage(&mut self.damage,
                                       root_frame.document.root,
                                       MatchSelectorsDocumentDamage);
                }

                if self.root_frame.is_some() {
                    self.reflow(ReflowForDisplay)
                }
            }

            ClickEvent(_button, point) => {
                debug!("ClickEvent: clicked at %?", point);
                let root = match self.root_frame {
                    Some(ref frame) => frame.document.root,
                    None => fail!("root frame is None")
                };
                let (port, chan) = comm::stream();
                match self.query_layout(HitTestQuery(root, point, chan), port) {
                    Ok(node) => match node {
                        HitTestResponse(node) => {
                            debug!("clicked on %?", node.debug_str());
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
                                    match element.tag_name {
                                        ~"a" => self.load_url_from_element(element),
                                        _ => {}
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

    priv fn load_url_from_element(&self, element: &Element) {
        // if the node's element is "a," load url from href attr
        for element.attrs.iter().advance |attr| {
            if attr.name == ~"href" {
                debug!("clicked on link to %?", attr.value); 
                let current_url = match self.root_frame {
                    Some(ref frame) => Some(frame.url.clone()),
                    None => None
                };
                let url = make_url(attr.value.clone(), current_url);
                self.constellation_chan.send(LoadUrlMsg(url));
            }
        }
    }
}

