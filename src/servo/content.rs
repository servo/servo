export msg, ping;
export content;

import dom::rcu::writer_methods;
import dom=dom::base;
import layout::layout;

enum msg {
    parse(str),
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
        loop {
            alt from_master.recv() {
              parse(filename) {
                #debug["content: Received filename `%s`", filename];

                // Note: we can parse the next document in parallel
                // with any previous documents.
                let stream = html::spawn_parser_task(filename);
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
              exit {
                to_layout.send(layout::exit);
                break;
              }
            }
        }
    }
}
