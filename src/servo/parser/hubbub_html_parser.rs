use dom::base::{Attr, Element, ElementData, ElementKind, HTMLDivElement, HTMLHeadElement};
use dom::base::{HTMLImageElement, HTMLScriptElement, Node, NodeScope, Text, UnknownElement};
use dom::style::Stylesheet;
use geom::size::Size2D;
use gfx::geometry::px_to_au;
use parser::html_builder::CSSMessage;
use resource::resource_task::{Done, Load, Payload, ResourceTask};
use CSSExitMessage = parser::html_builder::Exit;
use CSSFileMessage = parser::html_builder::File;
use JSExitMessage = parser::html_builder::js_exit;
use JSFileMessage = parser::html_builder::js_file;
use JSMessage = parser::html_builder::js_message;

use comm::{chan, port};
use str::from_slice;
use unsafe::reinterpret_cast;
use Error = result::err;
use OK = result::ok;
use Result = result::result;
use Url = std::net::url::url;

type JSResult = ~[~[u8]];

struct HtmlParserResult {
    root: Node;
    style_port: comm::Port<Stylesheet>;
    js_port: comm::Port<JSResult>;
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
fn css_link_listener(to_parent : comm::Chan<Stylesheet>, from_parent : comm::Port<CSSMessage>,
                     resource_task: ResourceTask) {
    let mut result_vec = ~[];

    loop {
        match from_parent.recv() {
          CSSFileMessage(url) => {
            let result_port = comm::port();
            let result_chan = comm::chan(result_port);
            // TODO: change copy to move once we have match move
            let url = copy url;
            task::spawn(|| {
                // TODO: change copy to move once we can move into closures
                let css_stream = css_lexer::spawn_css_lexer_task(copy url, resource_task);
                let mut css_rules = css_builder::build_stylesheet(css_stream);
                result_chan.send(css_rules);
            });

            vec::push(result_vec, result_port);
          }
          CSSExitMessage => {
            break;
          }
        }
    }

    let css_rules = vec::flat_map(result_vec, |result_port| { result_port.recv() });
    
    to_parent.send(css_rules);
}

fn js_script_listener(to_parent : comm::Chan<~[~[u8]]>, from_parent : comm::Port<JSMessage>,
                      resource_task: ResourceTask) {
    let mut result_vec = ~[];

    loop {
        match from_parent.recv() {
          JSFileMessage(url) => {
            let result_port = comm::port();
            let result_chan = comm::chan(result_port);
            // TODO: change copy to move once we have match move
            let url = copy url;
            do task::spawn || {
                let input_port = port();
                // TODO: change copy to move once we can move into closures
                resource_task.send(Load(copy url, input_port.chan()));

                let mut buf = ~[];
                loop {
                    match input_port.recv() {
                      Payload(data) => {
                        buf += data;
                      }
                      Done(OK(*)) => {
                        result_chan.send(buf);
                        break;
                      }
                      Done(Error(*)) => {
                        #error("error loading script %s", url.to_str());
                      }
                    }
                }
            }
            vec::push(result_vec, result_port);
          }
          JSExitMessage => {
            break;
          }  
        }
    }

    let js_scripts = vec::map(result_vec, |result_port| result_port.recv());
    to_parent.send(js_scripts);
}

fn build_element_kind(tag_name: &str) -> ~ElementKind {
    if tag_name == "div" {
        ~HTMLDivElement
    } else if tag_name == "img" {
        ~HTMLImageElement({ mut size: Size2D(px_to_au(100), px_to_au(100)) })
    } else if tag_name == "script" {
        ~HTMLScriptElement
    } else if tag_name == "head" {
        ~HTMLHeadElement
    } else {
        ~UnknownElement 
    }
}

fn parse_html(scope: NodeScope, url: Url, resource_task: ResourceTask) -> HtmlParserResult unsafe {
    // Spawn a CSS parser to receive links to CSS style sheets.
    let (css_port, css_chan): (comm::Port<Stylesheet>, comm::Chan<CSSMessage>) =
            do task::spawn_conversation |css_port: comm::Port<CSSMessage>,
                                         css_chan: comm::Chan<Stylesheet>| {
        css_link_listener(css_chan, css_port, resource_task);
    };

    // Spawn a JS parser to receive JavaScript.
    let (js_port, js_chan): (comm::Port<JSResult>, comm::Chan<JSMessage>) =
            do task::spawn_conversation |js_port: comm::Port<JSMessage>,
                                         js_chan: comm::Chan<JSResult>| {
        js_script_listener(js_chan, js_port, resource_task);
    };

    // Build the root node.
    let root = scope.new_node(Element(ElementData(~"html", ~HTMLDivElement)));
    debug!("created new node");
    let parser = hubbub::Parser("UTF-8", false);
    debug!("created parser");
    parser.set_document_node(reinterpret_cast(root));
    parser.enable_scripting(true);
    parser.set_tree_handler(@hubbub::TreeHandler {
        create_comment: |data| {
            debug!("create comment");
            0u
        },
        create_doctype: |doctype| {
            debug!("create doctype");
            let new_node = scope.new_node(Element(ElementData(~"doctype", ~UnknownElement)));
            reinterpret_cast(new_node)
        },
        create_element: |tag| {
            debug!("create element");
            let element_kind = build_element_kind(tag.name);
            let node = scope.new_node(Element(ElementData(from_slice(tag.name), element_kind)));
            for tag.attributes.each |attribute| {
                do scope.read(node) |node_contents| {
                    match *node_contents.kind {
                      Element(element) => {
                        element.attrs.push(~Attr(from_slice(attribute.name),
                                                 from_slice(attribute.value)));
                        match *element.kind {
                          HTMLImageElement(img) if attribute.name == "width" => {
                            match int::from_str(from_slice(attribute.value)) {
                              none => {} // Drop on the floor.
                              some(s) => img.size.width = px_to_au(s)
                            }
                          }
                          HTMLImageElement(img) if attribute.name == "height" => {
                            match int::from_str(from_slice(attribute.value)) {
                              none => {} // Drop on the floor.
                              some(s) => img.size.height = px_to_au(s)
                            }
                          }
                          HTMLDivElement | HTMLImageElement(*) | HTMLHeadElement |
                          HTMLScriptElement | UnknownElement => {} // Drop on the floor.
                        }
                      }

                      _ => fail ~"can't happen: unexpected node type"
                    }
                }
            }

            // Handle CSS style sheet links.
            do scope.read(node) |node_contents| {
                match *node_contents.kind {
                    Element(element) if element.tag_name == ~"link" => {
                        match (element.get_attr(~"rel"), element.get_attr(~"href")) {
                            (some(rel), some(href)) if rel == ~"stylesheet" => {
                                debug!("found CSS stylesheet: %s", href);
                                css_chan.send(CSSFileMessage(make_url(href, some(copy url))));
                            }
                            _ => {}
                        }
                    }
                    _ => {}
                }
            }

            reinterpret_cast(node)
        },
        create_text: |data| {
            debug!("create text");
            let new_node = scope.new_node(Text(from_slice(data)));
            reinterpret_cast(new_node)
        },
        ref_node: |_node| {},
        unref_node: |_node| {},
        append_child: |parent, child| unsafe {
            debug!("append child");
            scope.add_child(reinterpret_cast(parent), reinterpret_cast(child));
            child
        },
        insert_before: |_parent, _child| {
            debug!("insert before");
            0u
        },
        remove_child: |_parent, _child| {
            debug!("remove child");
            0u
        },
        clone_node: |_node, _deep| {
            debug!("clone node");
            0u
        },
        reparent_children: |_node, _new_parent| {
            debug!("reparent children");
            0u
        },
        get_parent: |_node, _element_only| {
            debug!("get parent");
            0u
        },
        has_children: |_node| {
            debug!("has children");
            false
        },
        form_associate: |_form, _node| {
            debug!("form associate");
        },
        add_attributes: |_node, _attributes| {
            debug!("add attributes");
        },
        set_quirks_mode: |_mode| {
            debug!("set quirks mode");
        },
        encoding_change: |_encname| {
            debug!("encoding change");
        },
        complete_script: |script| unsafe {
            do scope.read(reinterpret_cast(script)) |node_contents| {
                match *node_contents.kind {
                    Element(element) if element.tag_name == ~"script" => {
                        match element.get_attr(~"src") {
                            some(src) => {
                                debug!("found script: %s", src);
                                let new_url = make_url(src, some(copy url));
                                js_chan.send(JSFileMessage(new_url));
                            }
                            none => {}
                        }
                    }
                    _ => {}
                }
            }
            debug!("complete script");
        }
    });
    debug!("set tree handler");

    let input_port = port();
    resource_task.send(Load(copy url, input_port.chan()));
    debug!("loaded page");
    loop {
        match input_port.recv() {
            Payload(data) => {
                debug!("received data");
                parser.parse_chunk(data);
            }
            Done(*) => {
                break;
            }
        }
    }

    css_chan.send(CSSExitMessage);
    js_chan.send(JSExitMessage);

    return HtmlParserResult { root: root, style_port: css_port, js_port: js_port };
}

