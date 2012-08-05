#[doc="Constructs a DOM tree from an incoming token stream."]

import dom::base::{Attr, Element, ElementData, ElementKind, HTMLDivElement, HTMLHeadElement,
                   HTMLScriptElement};
import dom::base::{HTMLImageElement, Node, NodeScope, Text, TreeReadMethods, TreeWriteMethods};
import dom::base::{UnknownElement};
import dom::rcu::WriterMethods;
import geom::size::Size2D;
import gfx::geometry;
import gfx::geometry::au;
import parser = parser::html_lexer;
import parser::Token;
import dom::style::Stylesheet;
import vec::{push, push_all_move, flat_map};
import std::net::url::url;
import resource::resource_task::{ResourceTask, Load, Payload, Done};
import dvec::extensions;
import result::{ok, err};
import to_str::to_str;

enum CSSMessage {
    File(url),
    Exit   
}

enum js_message {
    js_file(url),
    js_exit
}

#[warn(no_non_implicitly_copyable_typarams)]
fn link_up_attribute(scope: NodeScope, node: Node, -key: ~str, -value: ~str) {
    // TODO: Implement atoms so that we don't always perform string comparisons.
    scope.read(node, |node_contents| {
        alt *node_contents.kind {
          Element(element) {
            element.attrs.push(~Attr(copy key, copy value));
            alt *element.kind {
              HTMLImageElement(img) if key == ~"width" {
                alt int::from_str(value) {
                  none {
                    // Drop on the floor.
                  }
                  some(s) { img.size.width = geometry::px_to_au(s); }
                }
              }
              HTMLImageElement(img) if key == ~"height" {
                alt int::from_str(value) {
                  none {
                    // Drop on the floor.
                  }
                  some(s) {
                    img.size.height = geometry::px_to_au(s);
                  }
                }
              }
              HTMLDivElement | HTMLImageElement(*) | HTMLHeadElement |
              HTMLScriptElement | UnknownElement {
                // Drop on the floor.
              }
            }
          }

          Text(*) {
            fail ~"attempt to link up an attribute to a text node"
          }
        }
    })
}

fn build_element_kind(tag_name: ~str) -> ~ElementKind {
    alt tag_name {
      ~"div" { ~HTMLDivElement }
      ~"img" {
        ~HTMLImageElement({
            mut size: Size2D(geometry::px_to_au(100),
                             geometry::px_to_au(100))
        })
      }
      ~"script" { ~HTMLScriptElement }
      ~"head" { ~HTMLHeadElement }
      _ { ~UnknownElement  }
    }
}

#[doc="Runs a task that coordinates parsing links to css stylesheets.

This function should be spawned in a separate task and spins waiting
for the html builder to find links to css stylesheets and sends off
tasks to parse each link.  When the html process finishes, it notifies
the listener, who then collects the css rules from each task it
spawned, collates them, and sends them to the given result channel.

# Arguments

* `to_parent` - A channel on which to send back the full set of rules.
* `from_parent` - A port on which to receive new links.

"]
fn css_link_listener(to_parent : comm::chan<Stylesheet>, from_parent : comm::port<CSSMessage>,
                     resource_task: ResourceTask) {
    let mut result_vec = ~[];

    loop {
        alt from_parent.recv() {
          File(url) {
            let result_port = comm::port();
            let result_chan = comm::chan(result_port);
            task::spawn(|| {
                let css_stream = css_lexer::spawn_css_lexer_task(copy url, resource_task);
                let mut css_rules = css_builder::build_stylesheet(css_stream);
                result_chan.send(css_rules);
            });

            push(result_vec, result_port);
          }
          Exit {
            break;
          }
        }
    }

    let css_rules = flat_map(result_vec, |result_port| { result_port.recv() });
    
    to_parent.send(css_rules);
}

fn js_script_listener(to_parent : comm::chan<~[~[u8]]>, from_parent : comm::port<js_message>,
                      resource_task: ResourceTask) {
    let mut result_vec = ~[];

    loop {
        alt from_parent.recv() {
          js_file(url) {
            let result_port = comm::port();
            let result_chan = comm::chan(result_port);
            do task::spawn {
                let input_port = port();
                resource_task.send(Load(url, input_port.chan()));

                let mut buf = ~[];
                loop {
                    alt input_port.recv() {
                      Payload(data) {
                        buf += data;
                      }
                      Done(ok(*)) {
                        result_chan.send(buf);
                        break;
                      }
                      Done(err(*)) {
                        #error("error loading script %s", url.to_str());
                      }
                    }
                }
            }
            push(result_vec, result_port);
          }
          js_exit {
            break;
          }  
        }
    }

    let js_scripts = vec::map(result_vec, |result_port| result_port.recv());
    to_parent.send(js_scripts);
}

#[warn(no_non_implicitly_copyable_typarams)]
fn build_dom(scope: NodeScope, stream: comm::port<Token>, url: url,
             resource_task: ResourceTask) -> (Node, comm::port<Stylesheet>, comm::port<~[~[u8]]>) {
    // The current reference node.
    let mut cur_node = scope.new_node(Element(ElementData(~"html", ~HTMLDivElement)));
    // We will spawn a separate task to parse any css that is
    // encountered, each link to a stylesheet is sent to the waiting
    // task.  After the html sheet has been fully read, the spawned
    // task will collect the results of all linked style data and send
    // it along the returned port.
    let style_port = comm::port();
    let child_chan = comm::chan(style_port);
    let style_chan = task::spawn_listener(|child_port| {
        css_link_listener(child_chan, child_port, resource_task);
    });

    let js_port = comm::port();
    let child_chan = comm::chan(js_port);
    let js_chan = task::spawn_listener(|child_port| {
        js_script_listener(child_chan, child_port, resource_task);
    });

    loop {
        let token = stream.recv();
        alt token {
          parser::Eof { break; }
          parser::StartOpeningTag(tag_name) {
            #debug["starting tag %s", tag_name];
            let element_kind = build_element_kind(tag_name);
            let new_node = scope.new_node(Element(ElementData(copy tag_name, element_kind)));
            scope.add_child(cur_node, new_node);
            cur_node = new_node;
          }
          parser::Attr(key, value) {
            #debug["attr: %? = %?", key, value];
            link_up_attribute(scope, cur_node, copy key, copy value);
          }
          parser::EndOpeningTag {
            #debug("end opening tag");
          }
          // TODO: Fail more gracefully (i.e. according to the HTML5
          //       spec) if we close more tags than we open.
          parser::SelfCloseTag {
            //TODO: check for things other than the link tag
            scope.read(cur_node, |n| {
                alt *n.kind {
                  Element(elmt) if elmt.tag_name == ~"link" {
                    alt elmt.get_attr(~"rel") {
                      some(r) if r == ~"stylesheet" {
                        alt elmt.get_attr(~"href") {
                          some(filename) {
                            #debug["Linking to a css sheet named: %s", filename];
                            // FIXME: Need to base the new url on the current url
                            let new_url = make_url(filename, some(url));
                            style_chan.send(File(new_url));
                          }
                          none { /* fall through*/ }
                        }
                      }
                      _ { /* fall through*/ }
                    }
                  }
                  _ { /* fall through*/ }
                }                
            });
            cur_node = scope.get_parent(cur_node).get();
          }
          parser::EndTag(tag_name) {
            // TODO: Assert that the closing tag has the right name.
            scope.read(cur_node, |n| {
                alt *n.kind {
                  Element(elmt) if elmt.tag_name == ~"script" {
                    alt elmt.get_attr(~"src") {
                      some(filename) {
                        #debug["Linking to a js script named: %s", filename];
                        let new_url = make_url(filename, some(url));
                        js_chan.send(js_file(new_url));
                      }
                      none { /* fall through */ }
                    }
                  }
                  _ { /* fall though */ }
                }
            });
            cur_node = scope.get_parent(cur_node).get();
          }
          parser::Text(s) if !s.is_whitespace() {
            let new_node = scope.new_node(Text(copy s));
            scope.add_child(cur_node, new_node);
          }
          parser::Text(_) {
            // FIXME: Whitespace should not be ignored.
          }
          parser::Doctype {
            // TODO: Do something here...
          }
        }
    }

    style_chan.send(Exit);
    js_chan.send(js_exit);

    return (cur_node, style_port, js_port);
}
