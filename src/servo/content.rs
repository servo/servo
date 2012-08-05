#[doc="
    The content task is the main task that runs JavaScript and spawns layout
    tasks.
"]

export Content;
export ControlMsg, ExecuteMsg, ParseMsg, ExitMsg;
export PingMsg, PongMsg;
export create_content;
export Document;

import comm::{port, chan, listen, select2};
import task::{spawn, spawn_listener};
import io::{read_whole_file, println};
import result::{ok, err};

import dom::base::{Node, NodeScope, define_bindings};
import dom::event::{Event, ResizeEvent};
import dom::rcu::WriterMethods;
import dom::style;
import dom::style::Stylesheet;
import gfx::renderer::Sink;
import parser::html_lexer::spawn_html_lexer_task;
import parser::html_builder::build_dom;
import layout::layout_task;
import layout_task::{Layout, BuildMsg};

import jsrt = js::rust::rt;
import js::rust::methods;
import js::global::{global_class, debug_fns};

import either::{either, left, right};
import result::extensions;

import dom::bindings::utils::rust_box;
import js::rust::compartment;

import resource::resource_task;
import resource_task::{ResourceTask};

import std::net::url::url;
import url_to_str = std::net::url::to_str;
import util::url::make_url;

type Content = chan<ControlMsg>;

enum ControlMsg {
    ParseMsg(url),
    ExecuteMsg(url),
    ExitMsg
}

enum PingMsg {
    PongMsg
}

#[doc="Sends a ping to layout and waits for the response."]
#[warn(no_non_implicitly_copyable_typarams)]
fn join_layout(scope: NodeScope, layout: Layout) {

    if scope.is_reader_forked() {
        listen(|response_from_layout| {
            layout.send(layout_task::PingMsg(response_from_layout));
            response_from_layout.recv();
        });
        scope.reader_joined();
    }
}

class Document {
    let root: Node;
    let css_rules: Stylesheet;

    new(root: Node, +css_rules: Stylesheet) {
        self.root = root;
        self.css_rules = css_rules;
    }
}

class Content<S:Sink send copy> {
    let sink: S;
    let layout: Layout;
    let from_master: comm::port<ControlMsg>;
    let event_port: comm::port<Event>;

    let scope: NodeScope;
    let jsrt: jsrt;

    let mut document: option<@Document>;

    let resource_task: ResourceTask;

    new(layout: Layout, sink: S, from_master: port<ControlMsg>, resource_task: ResourceTask) {
        self.layout = layout;
        self.sink = sink;
        self.from_master = from_master;
        self.event_port = port();

        self.scope = NodeScope();
        self.jsrt = jsrt();

        self.document = none;

        self.sink.add_event_listener(self.event_port.chan());

        self.resource_task = resource_task;
    }

    fn start() {
        while self.handle_msg(select2(self.from_master, self.event_port)) {
            // Go on...
        }
    }

    fn handle_msg(msg: either<ControlMsg,Event>) -> bool {
        alt msg {
            left(control_msg) {
                return self.handle_control_msg(control_msg);
            }
            right(event) {
                return self.handle_event(event);
            }
        }
    }

    fn handle_control_msg(control_msg: ControlMsg) -> bool {
        alt control_msg {
          ParseMsg(url) {
            #debug["content: Received url `%s` to parse", url_to_str(url)];

            // Note: we can parse the next document in parallel
            // with any previous documents.
            let stream = spawn_html_lexer_task(copy url, self.resource_task);
            let (root, style_port, js_port) = build_dom(self.scope, stream, url, self.resource_task);
            let css_rules = style_port.recv();
            let js_scripts = js_port.recv();

            // Apply the css rules to the dom tree:
            #debug["%?", css_rules];

            #debug["%?", js_scripts];

            let document = Document(root, css_rules);
            self.relayout(document);
            self.document = some(@document);

            //XXXjdm it was easier to duplicate the relevant ExecuteMsg code;
            //       they should be merged somehow in the future.
            for vec::each(js_scripts) |bytes| {
                let cx = self.jsrt.cx();
                cx.set_default_options_and_version();
                cx.set_logging_error_reporter();
                cx.new_compartment(global_class).chain(|compartment| {
                    compartment.define_functions(debug_fns);
                    define_bindings(*compartment, option::get(self.document));
                    cx.evaluate_script(compartment.global_obj, bytes, ~"???", 1u)
                });
            }

            return true;
          }

          ExecuteMsg(url) {
            #debug["content: Received url `%s` to execute", url_to_str(url)];

            alt read_whole_file(url.path) {
              err(msg) {
                println(#fmt["Error opening %s: %s", url_to_str(url), msg]);
              }
              ok(bytes) {
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

          ExitMsg {
            self.layout.send(layout_task::ExitMsg);
            return false;
          }
        }
    }

    fn relayout(document: Document) {
        #debug("content: performing relayout");

        // Now, join the layout so that they will see the latest
        // changes we have made.
        join_layout(self.scope, self.layout);

        // Send new document and relevant styles to layout
        // FIXME: Put CSS rules in an arc or something.
        self.layout.send(BuildMsg(document.root, document.css_rules));

        // Indicate that reader was forked so any further
        // changes will be isolated.
        self.scope.reader_forked();
    }

    fn handle_event(event: Event) -> bool {
        alt event {
          ResizeEvent(new_width, new_height) {
            #debug("content got resize event: %d, %d", new_width, new_height);
            alt copy self.document {
                none {
                    // Nothing to do.
                }
                some(document) {
                    self.relayout(*document);
                }
            }
            return true;
          }
        }
    }
}

fn create_content<S: Sink send copy>(layout: Layout, sink: S, resource_task: ResourceTask) -> chan<ControlMsg> {
    do spawn_listener::<ControlMsg> |from_master| {
        Content(layout, sink, from_master, resource_task).start();
    }
}

