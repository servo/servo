/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

/// The script task is the task that owns the DOM in memory, runs JavaScript, and spawns parsing
/// and layout tasks.

use dom::bindings::utils::GlobalStaticData;
use dom::document::Document;
use dom::event::{Event, ResizeEvent, ReflowEvent};
use dom::node::define_bindings;
use dom::window::Window;
use layout::layout_task::{AddStylesheet, BuildData, BuildMsg, Damage, LayoutQuery};
use layout::layout_task::{LayoutQueryResponse, LayoutTask, MatchSelectorsDamage, NoDamage};
use layout::layout_task::{QueryMsg, ReflowDamage};
use layout::layout_task;

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
               layout_task: LayoutTask,
               resource_task: ResourceTask,
               image_cache_task: ImageCacheTask)
               -> ScriptTask {
        let (script_chan_copy, script_port) = (script_chan.clone(), Cell(script_port));

        // FIXME: rust#6399
        let mut the_task = task();
        the_task.sched_mode(SingleThreaded);
        do the_task.spawn {
            let script_context = ScriptContext::new(layout_task.clone(),
                                                    script_port.take(),
                                                    script_chan_copy.clone(),
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
    /// What parts of layout are dirty.
    damage: Damage,
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

            js_runtime: js_runtime,
            js_context: js_context,
            js_compartment: compartment,

            dom_static: GlobalStaticData(),
            bindings_initialized: false,

            root_frame: None,

            window_size: Size2D(800, 600),
            damage: MatchSelectorsDamage,
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

        self.relayout()
    }

    /// Handles a request to exit the script task and shut down layout.
    fn handle_exit_msg(&mut self) {
        self.join_layout();
        for self.root_frame.each |frame| {
            frame.document.teardown();
        }
        self.layout_task.send(layout_task::ExitMsg)
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
                  Some(sheet) => self.layout_task.send(AddStylesheet(sheet)),
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
        self.damage.add(MatchSelectorsDamage);
        self.relayout();

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

    /// Initiate an asynchronous relayout operation
    pub fn trigger_relayout(&mut self, damage: Damage) {
        self.damage.add(damage);
        self.relayout();
    }

    /// This method will wait until the layout task has completed its current action, join the
    /// layout task, and then request a new layout run. It won't wait for the new layout
    /// computation to finish.
    ///
    /// This function fails if there is no root frame.
    fn relayout(&mut self) {
        debug!("script: performing relayout");

        // Now, join the layout so that they will see the latest changes we have made.
        self.join_layout();

        // Layout will let us know when it's done.
        let (join_port, join_chan) = comm::stream();
        self.layout_join_port = Some(join_port);

        match self.root_frame {
            None => fail!(~"Tried to relayout with no root frame!"),
            Some(ref root_frame) => {
                // Send new document and relevant styles to layout.
                let data = ~BuildData {
                    node: root_frame.document.root,
                    url: copy root_frame.url,
                    script_chan: self.script_chan.clone(),
                    window_size: self.window_size,
                    script_join_chan: join_chan,
                    damage: replace(&mut self.damage, NoDamage),
                };

                self.layout_task.send(BuildMsg(data))
            }
        }

        debug!("script: layout forked")
    }

    /// Sends the given query to layout.
    pub fn query_layout(&mut self, query: LayoutQuery) -> LayoutQueryResponse {
         self.join_layout();

         let (response_port, response_chan) = comm::stream();
         self.layout_task.send(QueryMsg(query, response_chan));
         response_port.recv()
    }

    /// This is the main entry point for receiving and dispatching DOM events.
    ///
    /// TODO: Actually perform DOM event dispatch.
    fn handle_event(&mut self, event: Event) {
        match event {
            ResizeEvent(new_width, new_height, response_chan) => {
                debug!("script got resize event: %u, %u", new_width, new_height);

                self.damage.add(ReflowDamage);
                self.window_size = Size2D(new_width, new_height);

                if self.root_frame.is_some() {
                    self.relayout()
                }

                response_chan.send(())
            }

            ReflowEvent => {
                debug!("script got reflow event");

                self.damage.add(MatchSelectorsDamage);

                if self.root_frame.is_some() {
                    self.relayout()
                }
            }
        }
    }
}

