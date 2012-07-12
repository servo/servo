#[doc="
    The content task is the main task that runs JavaScript and spawns layout
    tasks.
"]

export Content;
export ControlMsg, ExecuteMsg, ParseMsg, ExitMsg;
export PingMsg, PongMsg;
export create_content;

import comm::{port, chan, listen, select2};
import task::{spawn, spawn_listener};
import io::{read_whole_file, println};
import result::{ok, err};

import dom::base::NodeScope;
import dom::event::{Event, ResizeEvent};
import dom::rcu::WriterMethods;
import dom::style;
import gfx::renderer::Sink;
import parser::html_lexer::spawn_html_lexer_task;
import parser::css_builder::build_stylesheet;
import parser::html_builder::build_dom;
import layout::layout_task;
import layout_task::{Layout, BuildMsg};

import jsrt = js::rust::rt;
import js::rust::methods;
import js::global::{global_class, debug_fns};

import either::{either, left, right};
import result::extensions;

type Content = chan<ControlMsg>;

enum ControlMsg {
    ParseMsg(~str),
    ExecuteMsg(~str),
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

class Content<S:Sink send copy> {
    let sink: S;
    let layout: Layout;
    let from_master: port<ControlMsg>;
    let event_port: port<Event>;

    let scope: NodeScope;
    let jsrt: jsrt;

    new(layout: Layout, sink: S, from_master: port<ControlMsg>) {
        self.layout = layout;
        self.sink = sink;
        self.from_master = from_master;
        self.event_port = port();

        self.scope = NodeScope();
        self.jsrt = jsrt();

        self.sink.add_event_listener(self.event_port.chan());
    }

    fn start() {
        while self.handle_msg(select2(self.from_master, self.event_port)) {
            // Go on...
        }
    }

    fn handle_msg(msg: either<ControlMsg,Event>) -> bool {
        alt msg {
            left(control_msg) {
                ret self.handle_control_msg(control_msg);
            }
            right(event) {
                ret self.handle_event(event);
            }
        }
    }

    fn handle_control_msg(control_msg: ControlMsg) -> bool {
        alt control_msg {
          ParseMsg(filename) {
            #debug["content: Received filename `%s` to parse", *filename];

            // Note: we can parse the next document in parallel
            // with any previous documents.
            let stream = spawn_html_lexer_task(copy filename);
            let (root, style_port) = build_dom(self.scope, stream);

            // Collect the css stylesheet
            let css_rules = style_port.recv();

            // Apply the css rules to the dom tree:
            #debug["%?", css_rules];

            // Now, join the layout so that they will see the latest
            // changes we have made.
            join_layout(self.scope, self.layout);

            // Send new document and relevant styles to layout
            self.layout.send(BuildMsg(root, css_rules));

            // Indicate that reader was forked so any further
            // changes will be isolated.
            self.scope.reader_forked();
            ret true;
          }

          ExecuteMsg(filename) {
            #debug["content: Received filename `%s` to execute", *filename];

            alt read_whole_file(*filename) {
              err(msg) {
                println(#fmt["Error opening %s: %s", *filename, msg]);
              }
              ok(bytes) {
                let cx = self.jsrt.cx();
                cx.set_default_options_and_version();
                cx.set_logging_error_reporter();
                cx.new_compartment(global_class).chain(|compartment| {
                    compartment.define_functions(debug_fns);
                    cx.evaluate_script(compartment.global_obj, bytes, *filename, 1u)
                });
              }
            }
            ret true;
          }

          ExitMsg {
            self.layout.send(layout_task::ExitMsg);
            ret false;
          }
        }
    }

    fn handle_event(event: Event) -> bool {
        alt event {
          ResizeEvent(new_width, new_height) {
            #debug("content got resize event: %d, %d", new_width, new_height);
            ret true;
          }
        }
    }
}

fn create_content<S: Sink send copy>(layout: Layout, sink: S) -> chan<ControlMsg> {
    do spawn_listener::<ControlMsg> |from_master| {
        Content(layout, sink, from_master).start();
    }
}

