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
use layout::layout_task::{AddStylesheet, BuildData, BuildMsg, Damage, LayoutTask};
use layout::layout_task::{MatchSelectorsDamage, NoDamage, ReflowDamage};
use layout::layout_task;

use core::cell::Cell;
use core::comm::{Port, SharedChan};
use core::either;
use core::io::read_whole_file;
use core::local_data;
use core::pipes::select2i;
use core::ptr::null;
use core::task::{SingleThreaded, task};
use core::util::replace;
use dom;
use geom::size::Size2D;
use html;
use js::JSVAL_NULL;
use js::global::{global_class, debug_fns};
use js::glue::bindgen::RUST_JSVAL_TO_OBJECT;
use js::jsapi::JSContext;
use js::jsapi::bindgen::{JS_CallFunctionValue, JS_GetContextPrivate};
use js::rust::{Compartment, Cx};
use jsrt = js::rust::rt;
use servo_net::image_cache_task::ImageCacheTask;
use servo_net::resource_task::ResourceTask;
use servo_util::tree::TreeNodeRef;
use std::net::url::Url;
use url_to_str = std::net::url::to_str;

pub enum ControlMsg {
    ParseMsg(Url),
    ExecuteMsg(Url),
    Timer(~dom::window::TimerData),
    ExitMsg
}

pub enum PingMsg {
    PongMsg
}

pub type ScriptTask = SharedChan<ControlMsg>;

pub fn ScriptTask(layout_task: LayoutTask,
                  dom_event_port: Port<Event>,
                  dom_event_chan: SharedChan<Event>,
                  resource_task: ResourceTask,
                  img_cache_task: ImageCacheTask)
                  -> ScriptTask {
    let (control_port, control_chan) = comm::stream();

    let control_chan = SharedChan::new(control_chan);
    let control_chan_copy = control_chan.clone();
    let control_port = Cell(control_port);
    let dom_event_port = Cell(dom_event_port);
    let dom_event_chan = Cell(dom_event_chan);

    // FIXME: rust#6399
    let mut the_task = task();
    the_task.sched_mode(SingleThreaded);
    do the_task.spawn {
        let script_context = ScriptContext(layout_task.clone(),
                                    control_port.take(),
                                    control_chan_copy.clone(),
                                    resource_task.clone(),
                                    img_cache_task.clone(),
                                    dom_event_port.take(),
                                    dom_event_chan.take());
        script_context.start();
    }

    return control_chan;
}

pub struct ScriptContext {
    layout_task: LayoutTask,
    layout_join_port: Option<comm::Port<()>>,

    image_cache_task: ImageCacheTask,
    control_port: comm::Port<ControlMsg>,
    control_chan: comm::SharedChan<ControlMsg>,
    event_port: comm::Port<Event>,
    event_chan: comm::SharedChan<Event>,

    jsrt: jsrt,
    cx: @Cx,
    dom_static: GlobalStaticData,

    document: Option<@mut Document>,
    window:   Option<@mut Window>,
    doc_url: Option<Url>,
    window_size: Size2D<uint>,

    resource_task: ResourceTask,

    compartment: Option<@mut Compartment>,

    // What parts of layout are dirty.
    damage: Damage,
}

pub fn ScriptContext(layout_task: LayoutTask, 
                     control_port: comm::Port<ControlMsg>,
                     control_chan: comm::SharedChan<ControlMsg>,
                     resource_task: ResourceTask,
                     img_cache_task: ImageCacheTask,
                     event_port: comm::Port<Event>,
                     event_chan: comm::SharedChan<Event>)
                     -> @mut ScriptContext {
    let jsrt = jsrt();
    let cx = jsrt.cx();

    cx.set_default_options_and_version();
    cx.set_logging_error_reporter();

    let compartment = match cx.new_compartment(global_class) {
          Ok(c) => Some(c),
          Err(()) => None
    };

    let script_context = @mut ScriptContext {
        layout_task: layout_task,
        layout_join_port: None,
        image_cache_task: img_cache_task,
        control_port: control_port,
        control_chan: control_chan,
        event_port: event_port,
        event_chan: event_chan,

        jsrt: jsrt,
        cx: cx,
        dom_static: GlobalStaticData(),

        document: None,
        window: None,
        doc_url: None,
        window_size: Size2D(800u, 600u),

        resource_task: resource_task,
        compartment: compartment,

        damage: MatchSelectorsDamage,
    };

    cx.set_cx_private(ptr::to_unsafe_ptr(&*script_context) as *());
    unsafe {
        local_data::local_data_set(global_script_context_key, cast::transmute(script_context));
    }

    script_context
}

fn global_script_context_key(_: @ScriptContext) {}

pub fn global_script_context() -> @ScriptContext {
    unsafe {
        return local_data::local_data_get(global_script_context_key).get();
    }
}

pub fn task_from_context(cx: *JSContext) -> *mut ScriptContext {
    JS_GetContextPrivate(cx) as *mut ScriptContext
}

#[unsafe_destructor]
impl Drop for ScriptContext {
    fn finalize(&self) {
        unsafe {
            let _ = local_data::local_data_pop(global_script_context_key);
        }
    }
}

#[allow(non_implicitly_copyable_typarams)]
pub impl ScriptContext {
    fn start(&mut self) {
        while self.handle_msg() {
            // Go on...
        }
    }

    fn handle_msg(&mut self) -> bool {
        match select2i(&mut self.control_port, &mut self.event_port) {
            either::Left(*) => {
                let msg = self.control_port.recv();
                self.handle_control_msg(msg)
            }
            either::Right(*) => {
                let ev = self.event_port.recv();
                self.handle_event(ev)
            }
        }
    }

    fn handle_control_msg(&mut self, control_msg: ControlMsg) -> bool {
        match control_msg {
          ParseMsg(url) => {
            debug!("script: Received url `%s` to parse", url_to_str(&url));

            define_bindings(self.compartment.get());

            // Note: we can parse the next document in parallel
            // with any previous documents.

            let result = html::hubbub_html_parser::parse_html(copy url,
                                                              self.resource_task.clone(),
                                                              self.image_cache_task.clone());

            let root = result.root;

              // Send stylesheets over to layout
              // FIXME: Need these should be streamed to layout as they are parsed
              // and do not need to stop here in the script task
              loop {
                  match result.style_port.recv() {
                      Some(sheet) => {
                          self.layout_task.send(AddStylesheet(sheet));
                      }
                      None => break
                  }
              }

            let js_scripts = result.js_port.recv();
            debug!("js_scripts: %?", js_scripts);

            let window   = Window(self.control_chan.clone(),
                                  self.event_chan.clone(),
                                  ptr::to_mut_unsafe_ptr(&mut *self)); //FIXME store this safely
            let document = Document(root, Some(window));

            do root.with_mut_base |base| {
                base.add_to_doc(document);
            }

            self.damage.add(MatchSelectorsDamage);
            self.relayout(document, &url);

            self.document = Some(document);
            self.window   = Some(window);
            self.doc_url = Some(url);

            let compartment = self.compartment.expect(~"TODO error checking");
            compartment.define_functions(debug_fns);

            do vec::consume(js_scripts) |_i, bytes| {
                self.cx.evaluate_script(compartment.global_obj, bytes, ~"???", 1u);
            }

            return true;
          }

          Timer(timerData) => {
            let compartment = self.compartment.expect(~"TODO error checking");
            let thisValue = if timerData.args.len() > 0 {
                RUST_JSVAL_TO_OBJECT(timerData.args[0])
            } else {
                compartment.global_obj.ptr
            };
            let rval = JSVAL_NULL;
            //TODO: support extra args. requires passing a *JSVal argv
            JS_CallFunctionValue(self.cx.ptr, thisValue, timerData.funval,
                                 0, null(), ptr::to_unsafe_ptr(&rval));
            self.relayout(self.document.get(), &(copy self.doc_url).get());
            return true;
          }


          ExecuteMsg(url) => {
            debug!("script: Received url `%s` to execute", url_to_str(&url));

            match read_whole_file(&Path(url.path)) {
              Err(msg) => {
                println(fmt!("Error opening %s: %s", url_to_str(&url), msg));
              }
              Ok(bytes) => {
                let compartment = self.compartment.expect(~"TODO error checking");
                compartment.define_functions(debug_fns);
                self.cx.evaluate_script(compartment.global_obj, bytes, copy url.path, 1u);
              }
            }
            return true;
          }

          ExitMsg => {
            self.layout_task.send(layout_task::ExitMsg);
            return false;
          }
        }
    }

    /**
       Sends a ping to layout and waits for the response (i.e., it has finished any
       pending layout request messages).
    */
    fn join_layout(&mut self) {
        if self.layout_join_port.is_some() {
            let join_port = replace(&mut self.layout_join_port, None);
            match join_port {
                Some(ref join_port) => {
                    if !join_port.peek() {
                        warn!("script: waiting on layout");
                    }
                    join_port.recv();
                    debug!("script: layout joined");
                }
                None => fail!(~"reader forked but no join port?")
            }
        }
    }

    /// This method will wait until the layout task has completed its current action, join the
    /// layout task, and then request a new layout run. It won't wait for the new layout
    /// computation to finish.
    fn relayout(&mut self, document: &Document, doc_url: &Url) {
        debug!("script: performing relayout");

        // Now, join the layout so that they will see the latest changes we have made.
        self.join_layout();

        // Layout will let us know when it's done.
        let (join_port, join_chan) = comm::stream();
        self.layout_join_port = Some(join_port);

        // Send new document and relevant styles to layout.
        let data = ~BuildData {
            node: document.root,
            url: copy *doc_url,
            dom_event_chan: self.event_chan.clone(),
            window_size: self.window_size,
            script_join_chan: join_chan,
            damage: replace(&mut self.damage, NoDamage),
        };

        self.layout_task.send(BuildMsg(data));

        debug!("script: layout forked");
    }

     fn query_layout(&mut self, query: layout_task::LayoutQuery)
                     -> layout_task::LayoutQueryResponse {
         //self.relayout(self.document.get(), &(copy self.doc_url).get());
         self.join_layout();

         let (response_port, response_chan) = comm::stream();
         self.layout_task.send(layout_task::QueryMsg(query, response_chan));
         return response_port.recv()
    }

    /**
       This is the main entry point for receiving and dispatching DOM events.
    */
    // TODO: actually perform DOM event dispatch.
    fn handle_event(&mut self, event: Event) -> bool {
        match event {
          ResizeEvent(new_width, new_height, response_chan) => {
            debug!("script got resize event: %u, %u", new_width, new_height);
            self.damage.add(ReflowDamage);
            self.window_size = Size2D(new_width, new_height);
            match copy self.document {
                None => {
                    // Nothing to do.
                }
                Some(document) => {
                    assert!(self.doc_url.is_some());
                    self.relayout(document, &(copy self.doc_url).get());
                }
            }
            response_chan.send(());
            return true;
          }
          ReflowEvent => {
            debug!("script got reflow event");
            self.damage.add(MatchSelectorsDamage);
            match /*bad*/ copy self.document {
                None => {
                    // Nothing to do.
                }
                Some(document) => {
                    assert!(self.doc_url.is_some());
                    self.relayout(document, &(copy self.doc_url).get());
                }
            }
            return true;
          }
        }
    }
}
