#[doc="
    The content task is the main task that runs JavaScript and spawns layout
    tasks.
"]

export ContentTask;
export ControlMsg, ExecuteMsg, ParseMsg, ExitMsg;
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
import js::rust::methods;
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

enum ControlMsg {
    ParseMsg(url),
    ExecuteMsg(url),
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

    let mut document: option<@Document>;
    let mut window:   option<@Window>;
    let mut doc_url: option<url>;

    let resource_task: ResourceTask;

    new(layout_task: LayoutTask, +compositor: C, from_master: Port<ControlMsg>,
        resource_task: ResourceTask) {
        self.layout_task = layout_task;
        self.compositor = compositor;
        self.from_master = from_master;
        self.event_port = port();

        self.scope = NodeScope();
        self.jsrt = jsrt();

        self.document = none;
        self.window   = none;
        self.doc_url  = none;

        self.compositor.add_event_listener(self.event_port.chan());

        self.resource_task = resource_task;
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
            let stream = spawn_html_lexer_task(copy url, self.resource_task);
            let (root, style_port, js_port) = build_dom(self.scope, stream, url, 
                                                        self.resource_task);
            let css_rules = style_port.recv();
            let js_scripts = js_port.recv();

            // Apply the css rules to the dom tree:
            #debug["css_rules: %?", css_rules];

            #debug["js_scripts: %?", js_scripts];

            let document = Document(root, css_rules);
            let window   = Window();
            self.relayout(document, &url);
            self.document = some(@document);
            self.window   = some(@window);
            self.doc_url = some(copy url);

            //XXXjdm it was easier to duplicate the relevant ExecuteMsg code;
            //       they should be merged somehow in the future.
            for vec::each(js_scripts) |bytes| {
                let cx = self.jsrt.cx();
                cx.set_default_options_and_version();
                cx.set_logging_error_reporter();
                cx.new_compartment(global_class).chain(|compartment| {
                    compartment.define_functions(debug_fns);
                    define_bindings(*compartment, option::get(self.document),
                                    option::get(self.window));
                    cx.evaluate_script(compartment.global_obj, bytes, ~"???", 1u)
                });
            }

            return true;
          }

          ExecuteMsg(url) => {
            #debug["content: Received url `%s` to execute", url_to_str(url)];

            match read_whole_file(url.path) {
              err(msg) => {
                println(#fmt["Error opening %s: %s", url_to_str(url), msg]);
              }
              ok(bytes) => {
                let cx = self.jsrt.cx();
                cx.set_default_options_and_version();
                cx.set_logging_error_reporter();
                cx.new_compartment(global_class).chain(|compartment| {
                    compartment.define_functions(debug_fns);
                    cx.evaluate_script(compartment.global_obj, bytes, url.path, 1u)
                });
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
