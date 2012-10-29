/*!
The content task is the main task that runs JavaScript and spawns layout
tasks.
*/

export Content, ContentTask;
export ControlMsg, ExecuteMsg, ParseMsg, ExitMsg, Timer;
export PingMsg, PongMsg;
export task_from_context;

use core::util::replace;
use std::arc::{ARC, clone};
use comm::{Port, Chan, listen, select2};
use task::{spawn, spawn_listener};
use io::{read_whole_file, println};

use dom::document::Document;
use dom::node::{Node, NodeScope, define_bindings};
use dom::event::{Event, ResizeEvent, ReflowEvent};
use dom::window::Window;
use geom::size::Size2D;
use layout::layout_task;
use layout_task::{LayoutTask, BuildMsg, BuildData};
use resource::image_cache_task::ImageCacheTask;

use newcss::values::Stylesheet;

use jsrt = js::rust::rt;
use js::rust::{cx, methods};
use js::global::{global_class, debug_fns};

use either::{Either, Left, Right};

use dom::bindings::utils::rust_box;
use js::rust::compartment;

use resource::resource_task;
use resource_task::{ResourceTask};

use std::net::url::Url;
use url_to_str = std::net::url::to_str;
use util::url::make_url;
use task::{task, SingleThreaded};
use std::cell::Cell;

use js::glue::bindgen::RUST_JSVAL_TO_OBJECT;
use js::JSVAL_NULL;
use js::jsapi::{JSContext, JSVal};
use js::jsapi::bindgen::{JS_CallFunctionValue, JS_GetContextPrivate};
use ptr::null;

pub enum ControlMsg {
    ParseMsg(Url),
    ExecuteMsg(Url),
    Timer(~dom::window::TimerData),
    ExitMsg
}

pub enum PingMsg {
    PongMsg
}

pub type ContentTask = pipes::SharedChan<ControlMsg>;

fn ContentTask(layout_task: LayoutTask,
               dom_event_port: pipes::Port<Event>,
               dom_event_chan: pipes::SharedChan<Event>,
               resource_task: ResourceTask,
               img_cache_task: ImageCacheTask) -> ContentTask {

    let (control_chan, control_port) = pipes::stream();

    let control_chan = pipes::SharedChan(move control_chan);
    let control_chan_copy = control_chan.clone();
    let control_port = Cell(move control_port);
    let dom_event_port = Cell(move dom_event_port);
    let dom_event_chan = Cell(move dom_event_chan);

    do task().sched_mode(SingleThreaded).spawn |move layout_task, move control_port,
                                                move control_chan_copy, move resource_task,
                                                move img_cache_task, move dom_event_port,
                                                move dom_event_chan| {
        let content = Content(layout_task, control_port.take(), control_chan_copy.clone(),
                              resource_task, img_cache_task.clone(),
                              dom_event_port.take(), dom_event_chan.take());
        content.start();
    }

    return move control_chan;
}

struct Content {
    layout_task: LayoutTask,
    mut layout_join_port: Option<pipes::Port<()>>,

    image_cache_task: ImageCacheTask,
    control_port: pipes::Port<ControlMsg>,
    control_chan: pipes::SharedChan<ControlMsg>,
    event_port: pipes::Port<Event>,
    event_chan: pipes::SharedChan<Event>,

    scope: NodeScope,
    jsrt: jsrt,
    cx: cx,

    mut document: Option<@Document>,
    mut window:   Option<@Window>,
    mut doc_url: Option<Url>,
    mut window_size: Size2D<uint>,

    resource_task: ResourceTask,

    compartment: Option<compartment>,
}

fn Content(layout_task: LayoutTask, 
           control_port: pipes::Port<ControlMsg>,
           control_chan: pipes::SharedChan<ControlMsg>,
           resource_task: ResourceTask,
           img_cache_task: ImageCacheTask,
           event_port: pipes::Port<Event>,
           event_chan: pipes::SharedChan<Event>) -> @Content {

    let jsrt = jsrt();
    let cx = jsrt.cx();

    cx.set_default_options_and_version();
    cx.set_logging_error_reporter();

    let compartment = match cx.new_compartment(global_class) {
          Ok(c) => Some(c),
          Err(()) => None
    };

    let content = @Content {
        layout_task : move layout_task,
        layout_join_port : None,
        image_cache_task : move img_cache_task,
        control_port : move control_port,
        control_chan : move control_chan,
        event_port : move event_port,
        event_chan : move event_chan,

        scope : NodeScope(),
        jsrt : jsrt,
        cx : cx,

        document    : None,
        window      : None,
        doc_url     : None,
        window_size : Size2D(800u, 600u),

        resource_task : resource_task,
        compartment : compartment
    };

    cx.set_cx_private(ptr::to_unsafe_ptr(&*content) as *());

    content
}

fn task_from_context(cx: *JSContext) -> *Content unsafe {
    cast::reinterpret_cast(&JS_GetContextPrivate(cx))
}

#[allow(non_implicitly_copyable_typarams)]
impl Content {

    fn start() {
        while self.handle_msg() {
            // Go on ...
        }
    }

    fn handle_msg() -> bool {
        match pipes::select2i(&self.control_port, &self.event_port) {
            either::Left(*) => self.handle_control_msg(self.control_port.recv()),
            either::Right(*) => self.handle_event(self.event_port.recv())
        }
    }

    fn handle_control_msg(control_msg: ControlMsg) -> bool {
        match move control_msg {
          ParseMsg(move url) => {
            debug!("content: Received url `%s` to parse", url_to_str(copy url));

            // Note: we can parse the next document in parallel
            // with any previous documents.

            let result = html::hubbub_html_parser::parse_html(self.scope,
                                                              copy url,
                                                              self.resource_task,
                                                              self.image_cache_task.clone());

            let root = result.root;
            let css_rules = result.style_port.recv();
            let js_scripts = result.js_port.recv();

            // Apply the css rules to the dom tree:
            debug!("css_rules: %?", css_rules);

            debug!("js_scripts: %?", js_scripts);

            let document = Document(root, self.scope, move css_rules);
            let window   = Window(self.control_chan.clone());
            self.relayout(&document, &url);
            self.document = Some(@move document);
            self.window   = Some(@move window);
            self.doc_url = Some(move url);

            let compartment = option::expect(self.compartment, ~"TODO error checking");
            compartment.define_functions(debug_fns);
            define_bindings(compartment,
                            option::get(self.document),
                            option::get(self.window));

            do vec::consume(move js_scripts) |_i, bytes| {
                self.cx.evaluate_script(compartment.global_obj, move bytes, ~"???", 1u);
            }

            return true;
          }

          Timer(timerData) => {
            let compartment = option::expect(self.compartment, ~"TODO error checking");
            let thisValue = if timerData.args.len() > 0 {
                RUST_JSVAL_TO_OBJECT(unsafe { timerData.args.shift() })
            } else {
                compartment.global_obj.ptr
            };
            let rval = JSVAL_NULL;
            //TODO: support extra args. requires passing a *JSVal argv
            JS_CallFunctionValue(self.cx.ptr, thisValue, timerData.funval,
                                 0, null(), ptr::to_unsafe_ptr(&rval));
            self.relayout(self.document.get(), &self.doc_url.get());
            return true;
          }


          ExecuteMsg(url) => {
            debug!("content: Received url `%s` to execute", url_to_str(copy url));

            match read_whole_file(&Path(url.path)) {
              Err(msg) => {
                println(fmt!("Error opening %s: %s", url_to_str(copy url), msg));
              }
              Ok(move bytes) => {
                let compartment = option::expect(self.compartment, ~"TODO error checking");
                compartment.define_functions(debug_fns);
                self.cx.evaluate_script(compartment.global_obj, move bytes, copy url.path, 1u);
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
    fn join_layout() {
        assert self.scope.is_reader_forked() == self.layout_join_port.is_some();

        if self.scope.is_reader_forked() {

            let join_port = replace(&mut self.layout_join_port, None);

            match join_port {
                Some(ref join_port) => {
                    if !join_port.peek() {
                        warn!("content: waiting on layout");
                    }
                    join_port.recv();
                    debug!("content: layout joined");
                }
                None => fail ~"reader forked but no join port?"
            }

            self.scope.reader_joined();
        }
    }

    /**
       This method will wait until the layout task has completed its current action,
       join the layout task, and then request a new layout run. It won't wait for the
       new layout computation to finish.
    */
    fn relayout(document: &Document, doc_url: &Url) {
        debug!("content: performing relayout");

        // Now, join the layout so that they will see the latest
        // changes we have made.
        self.join_layout();

        // Layout will let us know when it's done
        let (join_chan, join_port) = pipes::stream();
        self.layout_join_port = move Some(move join_port);

        // Send new document and relevant styles to layout

        let data = BuildData {
            node: document.root,
            style: clone(&document.css_rules),
            url: copy *doc_url,
            dom_event_chan: self.event_chan.clone(),
            window_size: self.window_size,
            content_join_chan: move join_chan
        };

        self.layout_task.send(BuildMsg(move data));

        // Indicate that reader was forked so any further
        // changes will be isolated.
        self.scope.reader_forked();

        debug!("content: layout forked");
    }

     fn query_layout(query: layout_task::LayoutQuery) -> layout_task::LayoutQueryResponse {
         self.relayout(self.document.get(), &self.doc_url.get());
         self.join_layout();
         
         let response_port = Port();
         self.layout_task.send(layout_task::QueryMsg(query, response_port.chan()));
         return response_port.recv()
    }

    /**
       This is the main entry point for receiving and dispatching DOM events.
    */
    // TODO: actually perform DOM event dispatch.
    fn handle_event(event: Event) -> bool {
        match event {
          ResizeEvent(new_width, new_height, response_chan) => {
            debug!("content got resize event: %u, %u", new_width, new_height);
            self.window_size = Size2D(new_width, new_height);
            match copy self.document {
                None => {
                    // Nothing to do.
                }
                Some(document) => {
                    assert self.doc_url.is_some();
                    self.relayout(document, &self.doc_url.get());
                }
            }
            response_chan.send(());
            return true;
          }
          ReflowEvent => {
            debug!("content got reflow event");
            match copy self.document {
                None => {
                    // Nothing to do.
                }
                Some(document) => {
                    assert self.doc_url.is_some();
                    self.relayout(document, &self.doc_url.get());
                }
            }
            return true;
          }
        }
    }
}
