/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use dom::element::{HTMLAnchorElementTypeId, HTMLAsideElementTypeId, HTMLBRElementTypeId,
                   HTMLBodyElementTypeId, HTMLBoldElementTypeId, HTMLDivElementTypeId,
                   HTMLFontElementTypeId, HTMLFormElementTypeId, HTMLHRElementTypeId,
                   HTMLHeadElementTypeId, HTMLHtmlElementTypeId,
                   HTMLImageElementTypeId, HTMLIframeElementTypeId, HTMLInputElementTypeId,
                   HTMLItalicElementTypeId, HTMLLinkElementTypeId, HTMLListItemElementTypeId,
                   HTMLMetaElementTypeId, HTMLOListElementTypeId, HTMLOptionElementTypeId,
                   HTMLParagraphElementTypeId, HTMLScriptElementTypeId,
                   HTMLSectionElementTypeId, HTMLSelectElementTypeId, HTMLSmallElementTypeId,
                   HTMLSpanElementTypeId, HTMLStyleElementTypeId, HTMLTableBodyElementTypeId,
                   HTMLTableCellElementTypeId, HTMLTableElementTypeId,
                   HTMLTableRowElementTypeId, HTMLTitleElementTypeId, HTMLUListElementTypeId,
                   UnknownElementTypeId};
use dom::element::{HTMLAnchorElement, HTMLAsideElement, HTMLBRElement, HTMLBodyElement,
                   HTMLBoldElement, HTMLDivElement, HTMLFontElement, HTMLFormElement,
                   HTMLHRElement, HTMLHeadElement, HTMLHeadingElement, HTMLHtmlElement,
                   HTMLInputElement, HTMLImageElement, HTMLIframeElement,
                   HTMLItalicElement, HTMLLinkElement, HTMLListItemElement, HTMLMetaElement,
                   HTMLOListElement, HTMLOptionElement, HTMLParagraphElement,
                   HTMLScriptElement, HTMLSectionElement, HTMLSelectElement, HTMLSmallElement,
                   HTMLSpanElement, HTMLStyleElement, HTMLTableBodyElement,
                   HTMLTableCellElement, HTMLTableElement, HTMLTableRowElement,
                   HTMLTitleElement, HTMLUListElement};
use dom::element::{HTMLHeadingElementTypeId, Heading1, Heading2, Heading3, Heading4, Heading5,
                   Heading6};
use dom::element::{Element, Attr};
use dom::node::{AbstractNode, Comment, Doctype, ElementNodeTypeId, Node, ScriptView};
use dom::node::{Text};
use html::cssparse::{InlineProvenance, StylesheetProvenance, UrlProvenance, spawn_css_parser};
use js::jsapi::JSContext;
use newcss::stylesheet::Stylesheet;

use std::cast;
use std::cell::Cell;
use std::comm;
use std::comm::{Chan, Port, SharedChan};
use std::str::eq_slice;
use std::result;
use std::task;
use hubbub::hubbub;
use servo_msg::constellation_msg::SubpageId;
use servo_net::image_cache_task::ImageCacheTask;
use servo_net::image_cache_task;
use servo_net::resource_task::{Done, Load, Payload, ResourceTask};
use servo_util::tree::TreeUtils;
use servo_util::url::make_url;
use extra::net::url::Url;
use extra::net::url;
use extra::future::{Future, from_port};
use geom::size::Size2D;

macro_rules! handle_element(
    ($cx: expr, $tag:expr, $string:expr, $type_id:expr, $ctor:ident, [ $(($field:ident : $field_init:expr)),* ]) => (
        if eq_slice($tag, $string) {
            let _element = ~$ctor {
                parent: Element::new($type_id, ($tag).to_str()),
                $(
                    $field: $field_init,
                )*
            };
            unsafe {
                return Node::as_abstract_node(cx, _element);
            }
        }
    )
)

type JSResult = ~[~[u8]];

enum CSSMessage {
    CSSTaskNewFile(StylesheetProvenance),
    CSSTaskExit   
}

enum JSMessage {
    JSTaskNewFile(Url),
    JSTaskExit
}

pub struct HtmlParserResult {
    root: AbstractNode<ScriptView>,
    style_port: Port<Stylesheet>,
    iframe_port: Port<(Url, SubpageId, Future<Size2D<uint>>)>,
    js_port: Port<JSResult>,
}

trait NodeWrapping {
    unsafe fn to_hubbub_node(self) -> hubbub::NodeDataPtr;
    unsafe fn from_hubbub_node(n: hubbub::NodeDataPtr) -> Self;
}

impl NodeWrapping for AbstractNode<ScriptView> {
    unsafe fn to_hubbub_node(self) -> hubbub::NodeDataPtr {
        cast::transmute(self)
    }
    unsafe fn from_hubbub_node(n: hubbub::NodeDataPtr) -> AbstractNode<ScriptView> {
        cast::transmute(n)
    }
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
fn css_link_listener(to_parent: Chan<Stylesheet>,
                     from_parent: Port<CSSMessage>,
                     resource_task: ResourceTask) {
    let mut result_vec = ~[];

    loop {
        match from_parent.recv() {
            CSSTaskNewFile(provenance) => {
                result_vec.push(spawn_css_parser(provenance, resource_task.clone()));
            }
            CSSTaskExit => {
                break;
            }
        }
    }

    // Send the sheets back in order
    // FIXME: Shouldn't wait until after we've recieved CSSTaskExit to start sending these
    for result_vec.iter().advance |port| {
        to_parent.send(port.recv());
    }
}

fn js_script_listener(to_parent: Chan<~[~[u8]]>,
                      from_parent: Port<JSMessage>,
                      resource_task: ResourceTask) {
    let mut result_vec = ~[];

    loop {
        match from_parent.recv() {
            JSTaskNewFile(url) => {
                let (result_port, result_chan) = comm::stream();
                let resource_task = resource_task.clone();
                do task::spawn {
                    let (input_port, input_chan) = comm::stream();
                    // TODO: change copy to move once we can move into closures
                    resource_task.send(Load(copy url, input_chan));

                    let mut buf = ~[];
                    loop {
                        match input_port.recv() {
                            Payload(data) => {
                                buf.push_all(data);
                            }
                            Done(Ok(*)) => {
                                result_chan.send(Some(buf));
                                break;
                            }
                            Done(Err(*)) => {
                                error!("error loading script %s", url.to_str());
                                result_chan.send(None);
                                break;
                            }
                        }
                    }
                }
                result_vec.push(result_port);
            }
            JSTaskExit => {
                break;
            }
        }
    }

    let js_scripts = result_vec.iter().filter_map(|result_port| result_port.recv()).collect();
    to_parent.send(js_scripts);
}

// Silly macros to handle constructing DOM nodes. This produces bad code and should be optimized
// via atomization (issue #85).

fn build_element_from_tag(cx: *JSContext, tag: &str) -> AbstractNode<ScriptView> {
    // TODO (Issue #85): use atoms
    handle_element!(cx, tag, "a",        HTMLAnchorElementTypeId, HTMLAnchorElement, []);
    handle_element!(cx, tag, "aside",   HTMLAsideElementTypeId, HTMLAsideElement, []);
    handle_element!(cx, tag, "br",      HTMLBRElementTypeId, HTMLBRElement, []);
    handle_element!(cx, tag, "body",    HTMLBodyElementTypeId, HTMLBodyElement, []);
    handle_element!(cx, tag, "bold",    HTMLBoldElementTypeId, HTMLBoldElement, []);
    handle_element!(cx, tag, "div",     HTMLDivElementTypeId, HTMLDivElement, []);
    handle_element!(cx, tag, "font",    HTMLFontElementTypeId, HTMLFontElement, []);
    handle_element!(cx, tag, "form",    HTMLFormElementTypeId, HTMLFormElement, []);
    handle_element!(cx, tag, "hr",      HTMLHRElementTypeId, HTMLHRElement, []);
    handle_element!(cx, tag, "head",    HTMLHeadElementTypeId, HTMLHeadElement, []);
    handle_element!(cx, tag, "html",    HTMLHtmlElementTypeId, HTMLHtmlElement, []);
    handle_element!(cx, tag, "input",   HTMLInputElementTypeId, HTMLInputElement, []);
    handle_element!(cx, tag, "i",       HTMLItalicElementTypeId, HTMLItalicElement, []);
    handle_element!(cx, tag, "link",    HTMLLinkElementTypeId, HTMLLinkElement, []);
    handle_element!(cx, tag, "li",      HTMLListItemElementTypeId, HTMLListItemElement, []);
    handle_element!(cx, tag, "meta",    HTMLMetaElementTypeId, HTMLMetaElement, []);
    handle_element!(cx, tag, "ol",      HTMLOListElementTypeId, HTMLOListElement, []);
    handle_element!(cx, tag, "option",  HTMLOptionElementTypeId, HTMLOptionElement, []);
    handle_element!(cx, tag, "p",       HTMLParagraphElementTypeId, HTMLParagraphElement, []);
    handle_element!(cx, tag, "script",  HTMLScriptElementTypeId, HTMLScriptElement, []);
    handle_element!(cx, tag, "section", HTMLSectionElementTypeId, HTMLSectionElement, []);
    handle_element!(cx, tag, "select",  HTMLSelectElementTypeId, HTMLSelectElement, []);
    handle_element!(cx, tag, "small",   HTMLSmallElementTypeId, HTMLSmallElement, []);
    handle_element!(cx, tag, "span",    HTMLSpanElementTypeId, HTMLSpanElement, []);
    handle_element!(cx, tag, "style",   HTMLStyleElementTypeId, HTMLStyleElement, []);
    handle_element!(cx, tag, "tbody",   HTMLTableBodyElementTypeId, HTMLTableBodyElement, []);
    handle_element!(cx, tag, "td",      HTMLTableCellElementTypeId, HTMLTableCellElement, []);
    handle_element!(cx, tag, "table",   HTMLTableElementTypeId, HTMLTableElement, []);
    handle_element!(cx, tag, "tr",      HTMLTableRowElementTypeId, HTMLTableRowElement, []);
    handle_element!(cx, tag, "title",   HTMLTitleElementTypeId, HTMLTitleElement, []);
    handle_element!(cx, tag, "ul",      HTMLUListElementTypeId, HTMLUListElement, []);

    handle_element!(cx, tag, "img", HTMLImageElementTypeId, HTMLImageElement, [(image: None)]);
    handle_element!(cx, tag, "iframe",  HTMLIframeElementTypeId, HTMLIframeElement, [(frame: None), (size_future_chan: None), (subpage_id: None)]);

    handle_element!(cx, tag, "h1", HTMLHeadingElementTypeId, HTMLHeadingElement, [(level: Heading1)]);
    handle_element!(cx, tag, "h2", HTMLHeadingElementTypeId, HTMLHeadingElement, [(level: Heading2)]);
    handle_element!(cx, tag, "h3", HTMLHeadingElementTypeId, HTMLHeadingElement, [(level: Heading3)]);
    handle_element!(cx, tag, "h4", HTMLHeadingElementTypeId, HTMLHeadingElement, [(level: Heading4)]);
    handle_element!(cx, tag, "h5", HTMLHeadingElementTypeId, HTMLHeadingElement, [(level: Heading5)]);
    handle_element!(cx, tag, "h6", HTMLHeadingElementTypeId, HTMLHeadingElement, [(level: Heading6)]);

    unsafe {
        Node::as_abstract_node(cx, ~Element::new(UnknownElementTypeId, tag.to_str()))
    }
}

#[allow(non_implicitly_copyable_typarams)]
pub fn parse_html(cx: *JSContext,
                  url: Url,
                  resource_task: ResourceTask,
                  image_cache_task: ImageCacheTask,
                  next_subpage_id: SubpageId) -> HtmlParserResult {
    debug!("Hubbub: parsing %?", url);
    // Spawn a CSS parser to receive links to CSS style sheets.
    let resource_task2 = resource_task.clone();

    let (stylesheet_port, stylesheet_chan) = comm::stream();
    let stylesheet_chan = Cell::new(stylesheet_chan);
    let (css_msg_port, css_msg_chan) = comm::stream();
    let css_msg_port = Cell::new(css_msg_port);
    do spawn {
        css_link_listener(stylesheet_chan.take(), css_msg_port.take(), resource_task2.clone());
    }

    let css_chan = SharedChan::new(css_msg_chan);

    // Spawn a JS parser to receive JavaScript.
    let resource_task2 = resource_task.clone();
    let (js_result_port, js_result_chan) = comm::stream();
    let js_result_chan = Cell::new(js_result_chan);
    let (js_msg_port, js_msg_chan) = comm::stream();
    let js_msg_port = Cell::new(js_msg_port);
    do spawn {
        js_script_listener(js_result_chan.take(), js_msg_port.take(), resource_task2.clone());
    }
    let js_chan = SharedChan::new(js_msg_chan);

    let url2 = url.clone();
    let url3 = url.clone();

    // Build the root node.
    let root = ~HTMLHtmlElement { parent: Element::new(HTMLHtmlElementTypeId, ~"html") };
    let root = unsafe { Node::as_abstract_node(cx, root) };
    debug!("created new node");
    let mut parser = hubbub::Parser("UTF-8", false);
    debug!("created parser");
    parser.set_document_node(unsafe { root.to_hubbub_node() });
    parser.enable_scripting(true);
    parser.enable_styling(true);

    let (css_chan2, css_chan3, js_chan2) = (css_chan.clone(), css_chan.clone(), js_chan.clone());
    let (iframe_port, iframe_chan) = comm::stream();
    let next_subpage_id = Cell::new(next_subpage_id);
    
    parser.set_tree_handler(~hubbub::TreeHandler {
        create_comment: |data: ~str| {
            debug!("create comment");
            unsafe {
                Node::as_abstract_node(cx, ~Comment::new(data)).to_hubbub_node()
            }
        },
        create_doctype: |doctype: ~hubbub::Doctype| {
            debug!("create doctype");
            let ~hubbub::Doctype {name: name,
                                public_id: public_id,
                                system_id: system_id,
                                force_quirks: force_quirks } = doctype;
            let node = ~Doctype::new(name,
                                     public_id,
                                     system_id,
                                     force_quirks);
            unsafe {
                Node::as_abstract_node(cx, node).to_hubbub_node()
            }
        },
        create_element: |tag: ~hubbub::Tag| {
            debug!("create element");
            let node = build_element_from_tag(cx, tag.name);

            debug!("-- attach attrs");
            do node.as_mut_element |element| {
                for tag.attributes.iter().advance |attr| {
                    element.attrs.push(Attr::new(attr.name.clone(), attr.value.clone()));
                }
            }

            // Spawn additional parsing, network loads, etc. from tag and attrs
            match node.type_id() {
                // Handle CSS style sheets from <link> elements
                ElementNodeTypeId(HTMLLinkElementTypeId) => {
                    do node.with_imm_element |element| {
                        match (element.get_attr("rel"), element.get_attr("href")) {
                            (Some(rel), Some(href)) => {
                                if rel == "stylesheet" {
                                    debug!("found CSS stylesheet: %s", href);
                                    let url = make_url(href.to_str(), Some(url2.clone()));
                                    css_chan2.send(CSSTaskNewFile(UrlProvenance(url)));
                                }
                            }
                            _ => {}
                        }
                    }
                }

                ElementNodeTypeId(HTMLIframeElementTypeId) => {
                    do node.with_mut_iframe_element |iframe_element| {
                        let src_opt = iframe_element.parent.get_attr("src").map(|x| x.to_str());
                        for src_opt.iter().advance |src| {
                            let iframe_url = make_url(src.clone(), Some(url2.clone()));
                            iframe_element.frame = Some(iframe_url.clone());
                            
                            // Size future
                            let (port, chan) = comm::oneshot();
                            iframe_element.size_future_chan = Some(chan);
                            let size_future = from_port(port);

                            // Subpage Id
                            let subpage_id = next_subpage_id.take();
                            iframe_element.subpage_id = Some(subpage_id);
                            next_subpage_id.put_back(SubpageId(*subpage_id + 1));

                            iframe_chan.send((iframe_url, subpage_id, size_future));
                        }
                    }
                }

                ElementNodeTypeId(HTMLImageElementTypeId) => {
                    do node.with_mut_image_element |image_element| {
                        let src_opt = image_element.parent.get_attr("src").map(|x| x.to_str());
                        match src_opt {
                            None => {}
                            Some(src) => {
                                let img_url = make_url(src, Some(url2.clone()));
                                image_element.image = Some(copy img_url);
                                // inform the image cache to load this, but don't store a handle.
                                // TODO (Issue #84): don't prefetch if we are within a <noscript>
                                // tag.
                                image_cache_task.send(image_cache_task::Prefetch(img_url));
                            }
                        }
                    }
                }

                //TODO (Issue #86): handle inline styles ('style' attr)
                _ => {}
            }

            unsafe { node.to_hubbub_node() }
        },
        create_text: |data: ~str| {
            debug!("create text");
            unsafe {
                Node::as_abstract_node(cx, ~Text::new(data)).to_hubbub_node()
            }
        },
        ref_node: |_| {},
        unref_node: |_| {},
        append_child: |parent: hubbub::NodeDataPtr, child: hubbub::NodeDataPtr| {
            unsafe {
                debug!("append child %x %x", cast::transmute(parent), cast::transmute(child));
                let parent: AbstractNode<ScriptView> = NodeWrapping::from_hubbub_node(parent);
                let child: AbstractNode<ScriptView> = NodeWrapping::from_hubbub_node(child);
                parent.add_child(child);
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
        clone_node: |_node, deep| {
            debug!("clone node");
            if deep { error!("-- deep clone unimplemented"); }
            fail!(~"clone node unimplemented")
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
            fn complete_script(script: hubbub::NodeDataPtr,
                               url: Url,
                               js_chan: SharedChan<JSMessage>) {
                unsafe {
                    let script: AbstractNode<ScriptView> = NodeWrapping::from_hubbub_node(script);
                    do script.with_imm_element |script| {
                        match script.get_attr("src") {
                            Some(src) => {
                                debug!("found script: %s", src);
                                let new_url = make_url(src.to_str(), Some(url.clone()));
                                js_chan.send(JSTaskNewFile(new_url));
                            }
                            None => {}
                        }
                    }
                }
            }
            complete_script(script, url3.clone(), js_chan2.clone());
            debug!("complete script");
        },
        complete_style: |style| {
            // We've reached the end of a <style> so we can submit all the text to the parser.
            unsafe {
                let style: AbstractNode<ScriptView> = NodeWrapping::from_hubbub_node(style);
                let url = url::from_str("http://example.com/"); // FIXME
                let url_cell = Cell::new(url);

                let mut data = ~[];
                debug!("iterating over children %?", style.first_child());
                for style.children().advance |child| {
                    debug!("child = %?", child);
                    do child.with_imm_text() |text| {
                        data.push(text.parent.data.to_str());  // FIXME: Bad copy.
                    }
                }

                debug!("data = %?", data);
                let provenance = InlineProvenance(result::unwrap(url_cell.take()), data.concat());
                css_chan3.send(CSSTaskNewFile(provenance));
            }
        },
    });
    debug!("set tree handler");

    let (input_port, input_chan) = comm::stream();
    resource_task.send(Load(url.clone(), input_chan));
    debug!("loaded page");
    loop {
        match input_port.recv() {
            Payload(data) => {
                debug!("received data");
                parser.parse_chunk(data);
            }
            Done(Err(*)) => {
                fail!("Failed to load page URL %s", url.to_str());
            }
            Done(*) => {
                break;
            }
        }
    }

    css_chan.send(CSSTaskExit);
    js_chan.send(JSTaskExit);

    HtmlParserResult {
        root: root,
        style_port: stylesheet_port,
        iframe_port: iframe_port,
        js_port: js_result_port,
    }
}

