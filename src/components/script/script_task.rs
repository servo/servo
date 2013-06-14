/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

/// The script task is the task that owns the DOM in memory, runs JavaScript, and spawns parsing
/// and layout tasks.

use compositor_interface::{ReadyState, Loading, PerformingLayout, FinishedLoading};
use dom::bindings::utils::GlobalStaticData;
use dom::document::Document;
use dom::element::Element;
use dom::event::{Event, ResizeEvent, ReflowEvent, ClickEvent, MouseDownEvent, MouseUpEvent};
use dom::node::{AbstractNode, ScriptView, define_bindings};
use dom::window::Window;
use layout_interface::{AddStylesheetMsg, DocumentDamage, DocumentDamageLevel, HitTestQuery};
use layout_interface::{HitTestResponse, LayoutQuery, LayoutResponse, LayoutTask};
use layout_interface::{MatchSelectorsDocumentDamage, QueryMsg, Reflow, ReflowDocumentDamage};
use layout_interface::{ReflowForDisplay, ReflowForScriptQuery, ReflowGoal, ReflowMsg};
use layout_interface;
use engine_interface::{EngineTask, LoadUrlMsg};

use core::cast::transmute;
use core::cell::Cell;
use core::comm::{Port, SharedChan};
use core::io::read_whole_file;
use core::local_data;
use core::ptr::null;
use core::task::{SingleThreaded, task};
use core::util::replace;
use dom::window::TimerData;
use geom::size::Size2D;
use html::hubbub_html_parser;
use js::JSVAL_NULL;
use js::global::{global_class, debug_fns};
use js::glue::bindgen::RUST_JSVAL_TO_OBJECT;
use js::jsapi::JSContext;
use js::jsapi::bindgen::{JS_CallFunctionValue, JS_GetContextPrivate};
use js::rust::{Compartment, Cx};
use js;
use servo_net::image_cache_task::ImageCacheTask;
use servo_net::resource_task::ResourceTask;
use servo_util::tree::TreeNodeRef;
use servo_util::url::make_url;
use std::net::url::Url;
use std::net::url;

/// Messages used to control the script task.
pub enum ScriptMsg {
    /// Loads a new URL.
    LoadMsg(Url),
    /// Executes a standalone script.
    ExecuteMsg(Url),
    /// Sends a DOM event.
    SendEventMsg(Event),
    /// Fires a JavaScript timeout.
    FireTimerMsg(~TimerData),
    /// Notifies script that reflow is finished.
    ReflowCompleteMsg,
    /// Exits the engine.
    ExitMsg,
}

/// Encapsulates external communication with the script task.
pub struct ScriptTask {
    /// The channel used to send messages to the script task.
    chan: SharedChan<ScriptMsg>,
}

impl ScriptTask {
    /// Creates a new script task.
    pub fn new(script_port: Port<ScriptMsg>,
               script_chan: SharedChan<ScriptMsg>,
               engine_task: EngineTask,
               //FIXME(rust #5192): workaround for lack of working ~Trait
               compositor_task: ~fn(ReadyState),
               layout_task: LayoutTask,
               resource_task: ResourceTask,
               image_cache_task: ImageCacheTask)
               -> ScriptTask {
        let (script_chan_copy, script_port) = (script_chan.clone(), Cell(script_port));
        let compositor_task = Cell(compositor_task);
        // FIXME: rust#6399
        let mut the_task = task();
        the_task.sched_mode(SingleThreaded);
        do the_task.spawn {
            let script_context = ScriptContext::new(layout_task.clone(),
                                                    script_port.take(),
                                                    script_chan_copy.clone(),
                                                    engine_task.clone(),
                                                    compositor_task.take(),
                                                    resource_task.clone(),
                                                    image_cache_task.clone());
            script_context.start();
        }

        ScriptTask {
            chan: script_chan
        }
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
pub struct ScriptContext {
    /// A handle to the layout task.
    layout_task: LayoutTask,
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
    script_chan: SharedChan<ScriptMsg>,

    /// For communicating load url messages to the engine
    engine_task: EngineTask,
    /// For communicating loading messages to the compositor
    compositor_task: ~fn(ReadyState),

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
}

fn global_script_context_key(_: @ScriptContext) {}

/// Returns this task's script context singleton.
pub fn global_script_context() -> @ScriptContext {
    unsafe {
        local_data::local_data_get(global_script_context_key).get()
    }
}

/// Returns the script context from the JS Context.
///
/// FIXME: Rename to `script_context_from_js_context`.
pub fn task_from_context(js_context: *JSContext) -> *mut ScriptContext {
    JS_GetContextPrivate(js_context) as *mut ScriptContext
}

#[unsafe_destructor]
impl Drop for ScriptContext {
    fn finalize(&self) {
        unsafe {
            let _ = local_data::local_data_pop(global_script_context_key);
        }
    }
}

impl ScriptContext {
    /// Creates a new script context.
    pub fn new(layout_task: LayoutTask,
               script_port: Port<ScriptMsg>,
               script_chan: SharedChan<ScriptMsg>,
               engine_task: EngineTask,
               compositor_task: ~fn(ReadyState),
               resource_task: ResourceTask,
               img_cache_task: ImageCacheTask)
               -> @mut ScriptContext {
        let js_runtime = js::rust::rt();
        let js_context = js_runtime.cx();

        js_context.set_default_options_and_version();
        js_context.set_logging_error_reporter();

        let compartment = match js_context.new_compartment(global_class) {
              Ok(c) => c,
              Err(()) => fail!("Failed to create a compartment"),
        };

        let script_context = @mut ScriptContext {
            layout_task: layout_task,
            image_cache_task: img_cache_task,
            resource_task: resource_task,

            layout_join_port: None,
            script_port: script_port,
            script_chan: script_chan,

            engine_task: engine_task,
            compositor_task: compositor_task,

            js_runtime: js_runtime,
            js_context: js_context,
            js_compartment: compartment,

            dom_static: GlobalStaticData(),
            bindings_initialized: false,

            root_frame: None,

            window_size: Size2D(800, 600),
            damage: None,
        };
        // Indirection for Rust Issue #6248, dynamic freeze scope artifically extended
        let script_context_ptr = {
            let borrowed_ctx= &mut *script_context;
            borrowed_ctx as *mut ScriptContext
        };
        js_context.set_cx_private(script_context_ptr as *());

        unsafe {
            local_data::local_data_set(global_script_context_key, transmute(script_context))
        }

        script_context
    }

    /// Starts the script task. After calling this method, the script task will loop receiving
    /// messages on its port.
    pub fn start(&mut self) {
        while self.handle_msg() {
            // Go on...
        }
    }

    /// Handles an incoming control message.
    fn handle_msg(&mut self) -> bool {
        match self.script_port.recv() {
            LoadMsg(url) => {
                self.load(url);
                true
            }
            ExecuteMsg(url) => {
                self.handle_execute_msg(url);
                true
            }
            SendEventMsg(event) => {
                self.handle_event(event);
                true
            }
            FireTimerMsg(timer_data) => {
                self.handle_fire_timer_msg(timer_data);
                true
            }
            ReflowCompleteMsg => {
                self.handle_reflow_complete_msg();
                true
            }
            ExitMsg => {
                self.handle_exit_msg();
                false
            }
        }
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

    /// Handles a notification that reflow completed.
    fn handle_reflow_complete_msg(&mut self) {
        self.layout_join_port = None;
        self.set_ready_state(FinishedLoading)
    }

    /// Handles a request to exit the script task and shut down layout.
    fn handle_exit_msg(&mut self) {
        self.join_layout();
        for self.root_frame.each |frame| {
            frame.document.teardown();
        }

        self.layout_task.chan.send(layout_interface::ExitMsg)
    }

    // tells the compositor when loading starts and finishes
    // FIXME ~compositor_interface doesn't work right now, which is why this is necessary
    fn set_ready_state(&self, msg: ReadyState) {
        (self.compositor_task)(msg);
    }

    /// The entry point to document loading. Defines bindings, sets up the window and document
    /// objects, parses HTML and CSS, and kicks off initial layout.
    fn load(&mut self, url: Url) {
        // Define the script DOM bindings.
        //
        // FIXME: Can this be done earlier, to save the flag?
        if !self.bindings_initialized {
            define_bindings(self.js_compartment);
            self.bindings_initialized = true
        }

        self.set_ready_state(Loading);
        // Parse HTML.
        //
        // Note: We can parse the next document in parallel with any previous documents.
        let html_parsing_result = hubbub_html_parser::parse_html(copy url,
                                                                 self.resource_task.clone(),
                                                                 self.image_cache_task.clone());

        let root_node = html_parsing_result.root;

        // Send style sheets over to layout.
        //
        // FIXME: These should be streamed to layout as they're parsed. We don't need to stop here
        // in the script task.
        loop {
              match html_parsing_result.style_port.recv() {
                  Some(sheet) => self.layout_task.chan.send(AddStylesheetMsg(sheet)),
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
            url: url
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
        do vec::consume(js_scripts) |_, bytes| {
            let _ = self.js_context.evaluate_script(self.js_compartment.global_obj,
                                                    bytes,
                                                    ~"???",
                                                    1);
        }
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
        debug!("script: performing reflow");

        // Now, join the layout so that they will see the latest changes we have made.
        self.join_layout();

        // Tell the user that we're performing layout.
        self.set_ready_state(PerformingLayout);

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

                self.layout_task.chan.send(ReflowMsg(reflow))
            }
        }

        debug!("script: layout forked")
    }

    /// Reflows the entire document.
    ///
    /// FIXME: This should basically never be used.
    pub fn reflow_all(&mut self, goal: ReflowGoal) {
        for self.root_frame.each |root_frame| {
            ScriptContext::damage(&mut self.damage,
                                  root_frame.document.root,
                                  MatchSelectorsDocumentDamage)
        }

        self.reflow(goal)
    }

    /// Sends the given query to layout.
    pub fn query_layout(&mut self, query: LayoutQuery) -> Result<LayoutResponse,()> {
         self.join_layout();

         let (response_port, response_chan) = comm::stream();
         self.layout_task.chan.send(QueryMsg(query, response_chan));
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
            ResizeEvent(new_width, new_height, response_chan) => {
                debug!("script got resize event: %u, %u", new_width, new_height);

                self.window_size = Size2D(new_width, new_height);

                for self.root_frame.each |root_frame| {
                    ScriptContext::damage(&mut self.damage,
                                          root_frame.document.root,
                                          ReflowDocumentDamage);
                }

                if self.root_frame.is_some() {
                    self.reflow(ReflowForDisplay)
                }

                response_chan.send(())
            }

            // FIXME(pcwalton): This reflows the entire document and is not incremental-y.
            ReflowEvent => {
                debug!("script got reflow event");

                for self.root_frame.each |root_frame| {
                    ScriptContext::damage(&mut self.damage,
                                          root_frame.document.root,
                                          MatchSelectorsDocumentDamage);
                }

                if self.root_frame.is_some() {
                    self.reflow(ReflowForDisplay)
                }
            }

            ClickEvent(button, point) => {
                debug!("ClickEvent: clicked at %?", point);
                let root = match self.root_frame {
                    Some(ref frame) => frame.document.root,
                    None => fail!("root frame is None")
                };
                match self.query_layout(HitTestQuery(root, point)) {
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
                        _ => fail!(~"unexpected layout reply")
                    },
                    Err(()) => {
                        debug!(fmt!("layout query error"));
                    }
                };
            }
            MouseDownEvent(*) => {}
            MouseUpEvent(*) => {}
        }
    }

    priv fn load_url_from_element(&self, element: &Element) {
        // if the node's element is "a," load url from href attr
        for element.attrs.each |attr| {
            if attr.name == ~"href" {
                debug!("clicked on link to %?", attr.value); 
                let current_url = match self.root_frame {
                    Some(ref frame) => Some(frame.url.clone()),
                    None => None
                };
                let url = make_url(attr.value.clone(), current_url);
                self.engine_task.send(LoadUrlMsg(url));
            }
        }
    }
}

