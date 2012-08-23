#[doc="
    The content task is the main task that runs JavaScript and spawns layout
    tasks.
"]

export ContentTask;
export ControlMsg, ExecuteMsg, ParseMsg, ExitMsg, Timer;
export PingMsg, PongMsg;

import std::arc::{arc, clone};
import comm::{Port, Chan, port, chan, listen, select2};
import task::{spawn, spawn_listener};
import io::{read_whole_file, println};
import result::{ok, err};

import dom::base::{Document, Node, NodeScope, Window, define_bindings};
import dom::event::{Event, ResizeEvent};
import dom::style;
import dom::style::Stylesheet;
import gfx::compositor::Compositor;
import parser::html_lexer::spawn_html_lexer_task;
import parser::html_builder::build_dom;
import layout::layout_task;
import layout_task::{LayoutTask, BuildMsg};

import jsrt = js::rust::rt;
import js::rust::{cx, methods};
import js::global::{global_class, debug_fns};

import either::{Either, Left, Right};

import dom::bindings::utils::rust_box;
import js::rust::compartment;

import resource::resource_task;
import resource_task::{ResourceTask};

import std::net::url::url;
import url_to_str = std::net::url::to_str;
import util::url::make_url;
import task::{task, SingleThreaded};

import js::glue::bindgen::RUST_JSVAL_TO_OBJECT;
import js::JSVAL_NULL;
import js::jsapi::jsval;
import js::jsapi::bindgen::JS_CallFunctionValue;
import ptr::null;

enum ControlMsg {
    ParseMsg(url),
    ExecuteMsg(url),
    Timer(~dom::bindings::window::TimerData),
    ExitMsg
}

enum PingMsg {
    PongMsg
}

type ContentTask = Chan<ControlMsg>;

fn ContentTask<S: Compositor send copy>(layout_task: LayoutTask, +compositor: S, resource_task: ResourceTask) -> ContentTask {
    do task().sched_mode(SingleThreaded).spawn_listener::<ControlMsg> |from_master| {
        Content(layout_task, compositor, from_master, resource_task).start();
    }
}

#[doc="Sends a ping to layout and waits for the response."]
#[allow(non_implicitly_copyable_typarams)]
fn join_layout(scope: NodeScope, layout_task: LayoutTask) {

    if scope.is_reader_forked() {
        listen(|response_from_layout| {
            layout_task.send(layout_task::PingMsg(response_from_layout));
            response_from_layout.recv();
        });
        scope.reader_joined();
    }
}

struct Content<C:Compositor> {
    let compositor: C;
    let layout_task: LayoutTask;
    let from_master: comm::Port<ControlMsg>;
    let event_port: comm::Port<Event>;

    let scope: NodeScope;
    let jsrt: jsrt;
    let cx: cx;

    let mut document: option<@Document>;
    let mut window:   option<@Window>;
    let mut doc_url: option<url>;

    let resource_task: ResourceTask;

    let compartment: option<compartment>;

    new(layout_task: LayoutTask, +compositor: C, from_master: Port<ControlMsg>,
        resource_task: ResourceTask) {
        self.layout_task = layout_task;
        self.compositor = compositor;
        self.from_master = from_master;
        self.event_port = port();

        self.scope = NodeScope();
        self.jsrt = jsrt();
        self.cx = self.jsrt.cx();

        self.document = none;
        self.window   = none;
        self.doc_url  = none;

        self.compositor.add_event_listener(self.event_port.chan());

        self.resource_task = resource_task;

        self.cx.set_default_options_and_version();
        self.cx.set_logging_error_reporter();
        self.compartment = match self.cx.new_compartment(global_class) {
          ok(c) => some(c),
          err(()) => none
        };
    }

    fn start() {
        while self.handle_msg(select2(self.from_master, self.event_port)) {
            // Go on...
        }
    }

    fn handle_msg(msg: Either<ControlMsg,Event>) -> bool {
        match msg {
            Left(control_msg) => self.handle_control_msg(control_msg),
            Right(event) => self.handle_event(event)
        }
    }

    fn handle_control_msg(control_msg: ControlMsg) -> bool {
        match control_msg {
          ParseMsg(url) => {
            #debug["content: Received url `%s` to parse", url_to_str(url)];

            // Note: we can parse the next document in parallel
            // with any previous documents.
            /*let stream = spawn_html_lexer_task(copy url, self.resource_task);
            let (root, style_port, js_port) = build_dom(self.scope, stream, url, 
                                                        self.resource_task);

            let css_rules = style_port.recv();
            let js_scripts = js_port.recv();*/

            let result = parser::hubbub_html_parser::parse_html(self.scope,
                                                                url,
                                                                self.resource_task);

            let root = result.root;
            let css_rules = result.style_port.recv();
            let js_scripts = result.js_port.recv();

            // Apply the css rules to the dom tree:
            #debug["css_rules: %?", css_rules];

            #debug["js_scripts: %?", js_scripts];

            let document = Document(root, self.scope, css_rules);
            let window   = Window(self.from_master);
            self.relayout(document, &url);
            self.document = some(@document);
            self.window   = some(@window);
            self.doc_url = some(copy url);

            let compartment = option::expect(self.compartment, ~"TODO error checking");
            compartment.define_functions(debug_fns);
            define_bindings(*compartment,
                            option::get(self.document),
                            option::get(self.window));

            for vec::each(js_scripts) |bytes| {
                self.cx.evaluate_script(compartment.global_obj, bytes, ~"???", 1u);
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
            let _rval = JSVAL_NULL;
            //TODO: support extra args. requires passing a *jsval argv
            JS_CallFunctionValue(self.cx.ptr, thisValue, timerData.funval,
                                 0, null(), ptr::addr_of(_rval));
            self.relayout(*option::get(self.document), &option::get(self.doc_url));
            return true;
          }


          ExecuteMsg(url) => {
            #debug["content: Received url `%s` to execute", url_to_str(url)];

            match read_whole_file(url.path) {
              err(msg) => {
                println(#fmt["Error opening %s: %s", url_to_str(url), msg]);
              }
              ok(bytes) => {
                let compartment = option::expect(self.compartment, ~"TODO error checking");
                compartment.define_functions(debug_fns);
                self.cx.evaluate_script(compartment.global_obj, bytes, url.path, 1u);
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

    fn relayout(document: Document, doc_url: &url) {
        #debug("content: performing relayout");

        // Now, join the layout so that they will see the latest
        // changes we have made.
        join_layout(self.scope, self.layout_task);

        // Send new document and relevant styles to layout
        // FIXME: Put CSS rules in an arc or something.
        self.layout_task.send(BuildMsg(document.root, clone(&document.css_rules), copy *doc_url, self.event_port.chan()));

        // Indicate that reader was forked so any further
        // changes will be isolated.
        self.scope.reader_forked();
    }

    fn handle_event(event: Event) -> bool {
        match event {
          ResizeEvent(new_width, new_height) => {
            #debug("content got resize event: %d, %d", new_width, new_height);
            match copy self.document {
                none => {
                    // Nothing to do.
                }
                some(document) => {
                    assert self.doc_url.is_some();
                    self.relayout(*document, &self.doc_url.get());
                }
            }
            return true;
          }
          ReflowEvent => {
            #debug("content got reflow event");
            match copy self.document {
                none => {
                    // Nothing to do.
                }
                some(document) => {
                    assert self.doc_url.is_some();
                    self.relayout(*document, &self.doc_url.get());
                }
            }
            return true;
          }
        }
    }
}
