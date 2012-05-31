export msg, ping;
export content;

import result::extensions;
import dom::rcu::writer_methods;
import dom=dom::base;
import layout::layout;
import js::rust::methods;

enum msg {
    parse(str),
    execute(str),
    exit
}

enum ping {
    pong
}

// sends a ping to layout and awaits the response.
fn join_layout(scope: dom::node_scope,
               to_layout: chan<layout::msg>) {
    if scope.is_reader_forked() {
        comm::listen { |ch|
            to_layout.send(layout::ping(ch));
            ch.recv();
        }
        scope.reader_joined();
    }
}

fn content(to_layout: chan<layout::msg>) -> chan<msg> {
    task::spawn_listener::<msg> {|from_master|
        let scope = dom::node_scope();
        let rt = js::rust::rt();
        loop {
            alt from_master.recv() {
              parse(filename) {
                #debug["content: Received filename `%s` to parse", filename];

                // Note: we can parse the next document in parallel
                // with any previous documents.
                let stream = lexer::spawn_html_parser_task(filename);
                let root = parser::html_builder::build_dom(scope, stream);

                // Now, join the layout so that they will see the latest
                // changes we have made.
                join_layout(scope, to_layout);

                // Send new document to layout.
                to_layout.send(layout::build(root));

                // Indicate that reader was forked so any further
                // changes will be isolated.
                scope.reader_forked();
              }
              execute(filename) {
                #debug["content: Received filename `%s` to execute", filename];

                alt io::read_whole_file(filename) {
                  result::err(msg) {
                    io::println(#fmt["Error opening %s: %s", filename, msg]);
                  }
                  result::ok(bytes) {
                    let cx = rt.cx();
                    cx.set_default_options_and_version();
                    cx.set_logging_error_reporter();
                    cx.new_compartment(js::global::global_class).chain { |comp|
                        comp.define_functions(js::global::debug_fns);
                        cx.evaluate_script(comp.global_obj, bytes, filename, 1u)
                    };
                  }
                }
              }
              exit {
                to_layout.send(layout::exit);
                break;
              }
            }
        }
    }
}
