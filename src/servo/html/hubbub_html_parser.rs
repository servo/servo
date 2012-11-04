use au = gfx::geometry;
use content::content_task::ContentTask;
use newcss::stylesheet::Stylesheet;
use dom::cow;
use dom::element::*;
use dom::event::{Event, ReflowEvent};
use dom::node::{Comment, Doctype, DoctypeData, Text,
                Element, Node, NodeScope};
use resource::image_cache_task::ImageCacheTask;
use resource::image_cache_task;
use resource::resource_task::{Done, Load, Payload, ResourceTask};

use hubbub::Attribute;

use comm::{Chan, Port};
use std::net::url::Url;
use cssparse::spawn_css_parser;

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
    style_port: comm::Port<Option<Stylesheet>>,
    js_port: comm::Port<JSResult>,
}

/**
Runs a task that coordinates parsing links to css stylesheets.

This function should be spawned in a separate task and spins waiting
for the html builder to find links to css stylesheets and sends off
tasks to parse each link.  When the html process finishes, it notifies
the listener, who then collects the css rules from each task it
spawned, collates them, and sends them to the given result channel.

# Arguments

* `to_parent` - A channel on which to send back the full set of rules.
* `from_parent` - A port on which to receive new links.

*/
fn css_link_listener(to_parent : comm::Chan<Option<Stylesheet>>, from_parent : comm::Port<CSSMessage>,
                     resource_task: ResourceTask) {
    let mut result_vec = ~[];

    loop {
        match from_parent.recv() {
            CSSTaskNewFile(move url) => {
                result_vec.push(spawn_css_parser(move url, copy resource_task));
            }
            CSSTaskExit => {
                break;
            }
        }
    }

    // Send the sheets back in order
    // FIXME: Shouldn't wait until after we've recieved CSSTaskExit to start sending these
    do vec::consume(move result_vec) |_i, port| {
        to_parent.send(Some(port.recv()));
    }
    to_parent.send(None);
}

fn js_script_listener(to_parent : comm::Chan<~[~[u8]]>, from_parent : comm::Port<JSMessage>,
                      resource_task: ResourceTask) {
    let mut result_vec = ~[];

    loop {
        match from_parent.recv() {
            JSTaskNewFile(move url) => {
                let result_port = comm::Port();
                let result_chan = comm::Chan(&result_port);
                do task::spawn |move url| {
                    let input_port = Port();
                    // TODO: change copy to move once we can move into closures
                    resource_task.send(Load(copy url, input_port.chan()));

                    let mut buf = ~[];
                    loop {
                        match input_port.recv() {
                            Payload(move data) => {
                                buf += data;
                            }
                            Done(Ok(*)) => {
                                result_chan.send(move buf);
                                break;
                            }
                            Done(Err(*)) => {
                                #error("error loading script %s", url.to_str());
                            }
                        }
                    }
                }
                vec::push(&mut result_vec, result_port);
            }
            JSTaskExit => {
                break;
            }
        }
    }

    let js_scripts = vec::map(result_vec, |result_port| result_port.recv());
    to_parent.send(move js_scripts);
}

fn build_element_kind(tag: &str) -> ~ElementKind {
    // TODO (Issue #85): use atoms
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
    else if tag == ~"img" { ~HTMLImageElement(HTMLImageData()) }
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

pub fn parse_html(scope: NodeScope,
                  url: Url,
                  resource_task: ResourceTask,
                  image_cache_task: ImageCacheTask) -> HtmlParserResult unsafe {
    // Spawn a CSS parser to receive links to CSS style sheets.
    let (css_port, css_chan): (comm::Port<Option<Stylesheet>>, comm::Chan<CSSMessage>) =
            do task::spawn_conversation |css_port: comm::Port<CSSMessage>,
                                         css_chan: comm::Chan<Option<Stylesheet>>| {
        css_link_listener(css_chan, css_port, resource_task);
    };

    // Spawn a JS parser to receive JavaScript.
    let (js_port, js_chan): (comm::Port<JSResult>, comm::Chan<JSMessage>) =
            do task::spawn_conversation |js_port: comm::Port<JSMessage>,
                                         js_chan: comm::Chan<JSResult>| {
        js_script_listener(js_chan, js_port, resource_task);
    };

    let (scope, url) = (@copy scope, @move url);

    // Build the root node.
    let root = scope.new_node(Element(ElementData(~"html", ~HTMLDivElement)));
    debug!("created new node");
    let parser = hubbub::Parser("UTF-8", false);
    debug!("created parser");
    parser.set_document_node(cast::transmute(cow::unwrap(root)));
    parser.enable_scripting(true);
    parser.set_tree_handler(@hubbub::TreeHandler {
        create_comment: |data: ~str| {
            debug!("create comment");
            let new_node = scope.new_node(Comment(move data));
            unsafe { cast::transmute(cow::unwrap(new_node)) }
        },
        create_doctype: |doctype: ~hubbub::Doctype| {
            debug!("create doctype");
            // TODO: remove copying here by using struct pattern matching to 
            // move all ~strs at once (blocked on Rust #3845, #3846, #3847)
            let public_id = match doctype.public_id {
                None => None,
                Some(id) => Some(copy id)
            };
            let system_id = match doctype.system_id {
                None => None,
                Some(id) => Some(copy id)
            };
            let data = DoctypeData(copy doctype.name, move public_id, move system_id,
                                   copy doctype.force_quirks);
            let new_node = scope.new_node(Doctype(move data));
            unsafe { cast::transmute(cow::unwrap(new_node)) }
        },
        create_element: |tag: ~hubbub::Tag, move image_cache_task| {
            debug!("create element");
            // TODO: remove copying here by using struct pattern matching to 
            // move all ~strs at once (blocked on Rust #3845, #3846, #3847)
            let elem_kind = build_element_kind(tag.name);
            let elem = ElementData(copy tag.name, move elem_kind);

            debug!("attach attrs");
            for tag.attributes.each |attr| {
                elem.attrs.push(~Attr(copy attr.name, copy attr.value));
            }

            // Spawn additional parsing, network loads, etc. from tag and attrs
            match elem.kind {
                //Handle CSS style sheets from <link> elements
                ~HTMLLinkElement => {
                    match (elem.get_attr(~"rel"), elem.get_attr(~"href")) {
                        (Some(move rel), Some(move href)) => {
                            if rel == ~"stylesheet" {
                                debug!("found CSS stylesheet: %s", href);
                                css_chan.send(CSSTaskNewFile(make_url(move href,
                                                                      Some(copy *url))));
                            }
                        }
                        _ => {}
                    }
                },
                ~HTMLImageElement(copy d) => {  // FIXME: Bad copy.
                    do elem.get_attr(~"src").iter |img_url_str| {
                        let img_url = make_url(copy *img_url_str, Some(copy *url));
                        d.image = Some(copy img_url);
                        // inform the image cache to load this, but don't store a handle.
                        // TODO (Issue #84): don't prefetch if we are within a <noscript> tag.
                        image_cache_task.send(image_cache_task::Prefetch(move img_url));
                    }
                }
                //TODO (Issue #86): handle inline styles ('style' attr)
                _ => {}
            }
            let node = scope.new_node(Element(move elem));
            unsafe { cast::transmute(cow::unwrap(node)) }
        },
        create_text: |data: ~str| {
            debug!("create text");
            let new_node = scope.new_node(Text(move data));
            unsafe { cast::transmute(cow::unwrap(new_node)) }
        },
        ref_node: |_node| {},
        unref_node: |_node| {},
        append_child: |parent: hubbub::NodeDataPtr, child: hubbub::NodeDataPtr| unsafe {
            debug!("append child");
            unsafe {
                let p: Node = cow::wrap(cast::transmute(parent));
                let c: Node = cow::wrap(cast::transmute(child));
                scope.add_child(p, c);
            }
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
        complete_script: |script| {
            // A little function for holding this lint attr
            #[allow(non_implicitly_copyable_typarams)]
            fn complete_script(scope: &NodeScope, script: hubbub::NodeDataPtr, url: &Url, js_chan: &comm::Chan<JSMessage>) unsafe {
                do scope.read(&cow::wrap(cast::transmute(script))) |node_contents| {
                    match *node_contents.kind {
                        Element(element) if element.tag_name == ~"script" => {
                            match element.get_attr(~"src") {
                                Some(move src) => {
                                    debug!("found script: %s", src);
                                    let new_url = make_url(move src, Some(copy *url));
                                    js_chan.send(JSTaskNewFile(move new_url));
                                }
                                None => {}
                            }
                        }
                        _ => {}
                    }
                }
            }
            complete_script(scope, script, url, &js_chan);
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

