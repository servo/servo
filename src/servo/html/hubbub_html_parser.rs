use au = gfx::geometry;
use dom::base::{Comment, Doctype, DoctypeData, Element};
use dom::element::*;
use dom::base::{Node, NodeScope, Text, UnknownElement};
use css::values::Stylesheet;
use geom::size::Size2D;
use resource::resource_task::{Done, Load, Payload, ResourceTask};

use comm::{Chan, Port};
use str::from_slice;
use cast::reinterpret_cast;
use std::net::url::Url;

type JSResult = ~[~[u8]];

enum CSSMessage {
    CSSTaskNewFile(Url),
    CSSTaskExit   
}

enum JSMessage {
    JSTaskNewFile(Url),
    JSTaskExit
}

struct HtmlParserResult {
    root: Node,
    style_port: comm::Port<Stylesheet>,
    js_port: comm::Port<JSResult>,
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
          CSSTaskNewFile(url) => {
            let result_port = comm::Port();
            let result_chan = comm::Chan(result_port);
            // TODO: change copy to move once we have match move
            let url = copy url;
            task::spawn(|| {
                // TODO: change copy to move once we can move into closures
                let css_stream = css::lexer::spawn_css_lexer_task(copy url, resource_task);
                let mut css_rules = css::parser::build_stylesheet(css_stream);
                result_chan.send(css_rules);
            });

            vec::push(result_vec, result_port);
          }
          CSSTaskExit => {
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
          JSTaskNewFile(url) => {
            let result_port = comm::Port();
            let result_chan = comm::Chan(result_port);
            // TODO: change copy to move once we have match move
            let url = copy url;
            do task::spawn || {
                let input_port = Port();
                // TODO: change copy to move once we can move into closures
                resource_task.send(Load(copy url, input_port.chan()));

                let mut buf = ~[];
                loop {
                    match input_port.recv() {
                      Payload(data) => {
                        buf += data;
                      }
                      Done(Ok(*)) => {
                        result_chan.send(buf);
                        break;
                      }
                      Done(Err(*)) => {
                        #error("error loading script %s", url.to_str());
                      }
                    }
                }
            }
            vec::push(result_vec, result_port);
          }
          JSTaskExit => {
            break;
          }  
        }
    }

    let js_scripts = vec::map(result_vec, |result_port| result_port.recv());
    to_parent.send(js_scripts);
}

fn build_element_kind(tag: &str) -> ~ElementKind {
    // TODO: use atoms
    if      tag == ~"a" { ~HTMLAnchorElement }
    else if tag == ~"aside" { ~HTMLAsideElement }
    else if tag == ~"br" { ~HTMLBRElement }
    else if tag == ~"body" { ~HTMLBodyElement }
    else if tag == ~"bold" { ~HTMLBoldElement }
    else if tag == ~"div" { ~HTMLDivElement }
    else if tag == ~"font" { ~HTMLFontElement }
    else if tag == ~"form" { ~HTMLFormElement }
    else if tag == ~"hr" { ~HTMLHRElement }
    else if tag == ~"head" { ~HTMLHeadElement }
    else if tag == ~"h1" { ~HTMLHeadingElement(Heading1) }
    else if tag == ~"h2" { ~HTMLHeadingElement(Heading2) }
    else if tag == ~"h3" { ~HTMLHeadingElement(Heading3) }
    else if tag == ~"h4" { ~HTMLHeadingElement(Heading4) }
    else if tag == ~"h5" { ~HTMLHeadingElement(Heading5) }
    else if tag == ~"h6" { ~HTMLHeadingElement(Heading6) }
    else if tag == ~"html" { ~HTMLHtmlElement }
    else if tag == ~"img" { ~HTMLImageElement({ mut size: au::zero_size() }) }
    else if tag == ~"input" { ~HTMLInputElement }
    else if tag == ~"i" { ~HTMLItalicElement }
    else if tag == ~"link" { ~HTMLLinkElement }
    else if tag == ~"li" { ~HTMLListItemElement }
    else if tag == ~"meta" { ~HTMLMetaElement }
    else if tag == ~"ol" { ~HTMLOListElement }
    else if tag == ~"option" { ~HTMLOptionElement }
    else if tag == ~"p" { ~HTMLParagraphElement }
    else if tag == ~"script" { ~HTMLScriptElement }
    else if tag == ~"section" { ~HTMLSectionElement }
    else if tag == ~"select" { ~HTMLSelectElement }
    else if tag == ~"small" { ~HTMLSmallElement }
    else if tag == ~"span" { ~HTMLSpanElement }
    else if tag == ~"style" { ~HTMLStyleElement }
    else if tag == ~"tbody" { ~HTMLTableBodyElement }
    else if tag == ~"td" { ~HTMLTableCellElement }
    else if tag == ~"table" { ~HTMLTableElement }
    else if tag == ~"tr" { ~HTMLTableRowElement }
    else if tag == ~"title" { ~HTMLTitleElement }
    else if tag == ~"ul" { ~HTMLUListElement }
    else { ~UnknownElement }
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

    let (scope, url) = (@copy scope, @copy url);

    // Build the root node.
    let root = scope.new_node(Element(ElementData(~"html", ~HTMLDivElement)));
    debug!("created new node");
    let parser = hubbub::Parser("UTF-8", false);
    debug!("created parser");
    parser.set_document_node(reinterpret_cast(&root));
    parser.enable_scripting(true);
    parser.set_tree_handler(@hubbub::TreeHandler {
        create_comment: |data: &str| {
            debug!("create comment");
            let new_node = scope.new_node(Comment(from_slice(data)));
            unsafe { reinterpret_cast(&new_node) }
        },
        create_doctype: |doctype: &hubbub::Doctype| {
            debug!("create doctype");
            let name = from_slice(doctype.name);
            let public_id = match doctype.public_id {
                None => None,
                Some(id) => Some(from_slice(id))
            };
            let system_id = match doctype.system_id {
                None => None,
                Some(id) => Some(from_slice(id))
            };
            let data = DoctypeData(name, public_id, system_id, doctype.force_quirks);
            let new_node = scope.new_node(Doctype(data));
            unsafe { reinterpret_cast(&new_node) }
        },
        create_element: |tag: &hubbub::Tag| {
            debug!("create element");
            let elem_kind = build_element_kind(tag.name);
            let elem = ElementData(from_slice(tag.name), elem_kind);
            debug!("attach attrs");
            for tag.attributes.each |attribute| {
                elem.attrs.push(~Attr(from_slice(attribute.name),
                                      from_slice(attribute.value)));
            }

            // Spawn additional parsing, network loads, etc. from opening tag
            match elem.tag_name {
                //Handle CSS style sheets from <link> elements
                ~"link" => {
                    match (elem.get_attr(~"rel"), elem.get_attr(~"href")) {
                        (Some(rel), Some(href)) if rel == ~"stylesheet" => {
                            debug!("found CSS stylesheet: %s", href);
                            css_chan.send(CSSTaskNewFile(make_url(href, Some(copy *url))));
                        }
                        _ => {}
                    }
                }
                //TODO: handle inline styles ('style' attr)
                _ => {}
            }
            let node = scope.new_node(Element(elem));
            unsafe { reinterpret_cast(&node) }
        },
        create_text: |data| {
            debug!("create text");
            let new_node = scope.new_node(Text(from_slice(data)));
            unsafe { reinterpret_cast(&new_node) }
        },
        ref_node: |_node| {},
        unref_node: |_node| {},
        append_child: |parent, child| unsafe {
            debug!("append child");
            scope.add_child(reinterpret_cast(&parent), reinterpret_cast(&child));
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
            do scope.read(reinterpret_cast(&script)) |node_contents| {
                match *node_contents.kind {
                    Element(element) if element.tag_name == ~"script" => {
                        match element.get_attr(~"src") {
                            Some(src) => {
                                debug!("found script: %s", src);
                                let new_url = make_url(src, Some(copy *url));
                                js_chan.send(JSTaskNewFile(new_url));
                            }
                            None => {}
                        }
                    }
                    _ => {}
                }
            }
            debug!("complete script");
        }
    });
    debug!("set tree handler");

    let input_port = Port();
    resource_task.send(Load(copy *url, input_port.chan()));
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

    css_chan.send(CSSTaskExit);
    js_chan.send(JSTaskExit);

    return HtmlParserResult { root: root, style_port: css_port, js_port: js_port };
}

