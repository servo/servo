#[doc="
    The content task is the main task that runs JavaScript and spawns layout
    tasks.
"]

export Content;
export ControlMsg, ExecuteMsg, ParseMsg, ExitMsg;
export PingMsg, PongMsg;

import comm::{port, chan, listen, select2};
import task::{spawn, spawn_listener};
import io::{read_whole_file, println};
import result::{ok, err};

import dom::base::NodeScope;
import dom::event::ResizeEvent;
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

import either::{left, right};
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

#[warn(no_non_implicitly_copyable_typarams)]
fn Content<S:Sink send copy>(layout: Layout, sink: S) -> Content {
    spawn_listener::<ControlMsg>(|from_master| {
        let event_port = port();
        sink.add_event_listener(event_port.chan());

        let scope = NodeScope();
        let rt = jsrt();
        loop {
            alt select2(from_master, event_port) {
              left(ParseMsg(filename)) {
                #debug["content: Received filename `%s` to parse", *filename];

                // Note: we can parse the next document in parallel
                // with any previous documents.
                let stream = spawn_html_lexer_task(copy filename);
                let (root, style_port) = build_dom(scope, stream);
           
                // Collect the css stylesheet
                let css_rules = style_port.recv();
                
                // Apply the css rules to the dom tree:
                #debug["%?", css_rules];

                // Now, join the layout so that they will see the latest
                // changes we have made.
                join_layout(scope, layout);

                // Send new document and relevant styles to layout
                layout.send(BuildMsg(root, css_rules));

                // Indicate that reader was forked so any further
                // changes will be isolated.
                scope.reader_forked();
              }

              left(ExecuteMsg(filename)) {
                #debug["content: Received filename `%s` to execute", *filename];

                alt read_whole_file(*filename) {
                  err(msg) {
                    println(#fmt["Error opening %s: %s", *filename, msg]);
                  }
                  ok(bytes) {
                    let cx = rt.cx();
                    cx.set_default_options_and_version();
                    cx.set_logging_error_reporter();
                    cx.new_compartment(global_class).chain(|compartment| {
                        compartment.define_functions(debug_fns);
                        cx.evaluate_script(compartment.global_obj, bytes, *filename, 1u)
                    });
                  }
                }
              }

              left(ExitMsg) {
                layout.send(layout_task::ExitMsg);
                break;
              }

              right(ResizeEvent(new_width, new_height)) {
                #debug("content got resize event: %d, %d", new_width, new_height);
              }
            }
        }
    })
}
