/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use dom::attr::AttrHelpers;
use dom::bindings::codegen::Bindings::NodeBinding::NodeMethods;
use dom::bindings::codegen::InheritTypes::{NodeCast, ElementCast, HTMLScriptElementCast};
use dom::bindings::js::{JS, JSRef, Temporary, OptionalRootable, Root};
use dom::document::{Document, DocumentHelpers};
use dom::element::{AttributeHandlers, ElementHelpers};
use dom::htmlelement::HTMLElement;
use dom::htmlheadingelement::{Heading1, Heading2, Heading3, Heading4, Heading5, Heading6};
use dom::htmlformelement::HTMLFormElement;
use dom::htmlscriptelement::HTMLScriptElementHelpers;
use dom::node::{Node, NodeHelpers, TrustedNodeAddress};
use dom::servohtmlparser;
use dom::servohtmlparser::ServoHTMLParser;
use dom::types::*;
use page::Page;
use parse::Parser;

use encoding::all::UTF_8;
use encoding::types::{Encoding, DecodeReplace};

use servo_net::resource_task::{Load, LoadData, Payload, Done, ResourceTask};
use servo_msg::constellation_msg::LoadData as MsgLoadData;
use servo_util::task_state;
use servo_util::task_state::InHTMLParser;
use servo_util::str::DOMString;
use std::ascii::StrAsciiExt;
use std::comm::channel;
use std::str::MaybeOwned;
use url::Url;
use http::headers::HeaderEnum;
use time;
use html5ever::Attribute;
use html5ever::tree_builder::{TreeSink, QuirksMode, NodeOrText, AppendNode, AppendText};
use string_cache::QualName;

pub enum HTMLInput {
    InputString(String),
    InputUrl(Url),
}

// Parses an RFC 2616 compliant date/time string, and returns a localized
// date/time string in a format suitable for document.lastModified.
fn parse_last_modified(timestamp: &str) -> String {
    let format = "%m/%d/%Y %H:%M:%S";

    // RFC 822, updated by RFC 1123
    match time::strptime(timestamp, "%a, %d %b %Y %T %Z") {
        Ok(t) => return t.to_local().strftime(format),
        Err(_) => ()
    }

    // RFC 850, obsoleted by RFC 1036
    match time::strptime(timestamp, "%A, %d-%b-%y %T %Z") {
        Ok(t) => return t.to_local().strftime(format),
        Err(_) => ()
    }

    // ANSI C's asctime() format
    match time::strptime(timestamp, "%c") {
        Ok(t) => t.to_local().strftime(format),
        Err(_) => String::from_str("")
    }
}

#[deriving(PartialEq)]
pub enum ElementCreator {
    ParserCreated,
    ScriptCreated,
}

pub fn build_element_from_tag(name: QualName,
                              prefix: Option<DOMString>,
                              document: JSRef<Document>,
                              creator: ElementCreator) -> Temporary<Element> {
    if name.ns != ns!(HTML) {
        return Element::new(name.local.as_slice().to_string(), name.ns, None, document);
    }

    macro_rules! make(
        ($ctor:ident $(, $arg:expr)*) => ({
            let obj = $ctor::new(name.local.as_slice().to_string(), prefix, document $(, $arg)*);
            ElementCast::from_temporary(obj)
        })
    )

    // This is a big match, and the IDs for inline-interned atoms are not very structured.
    // Perhaps we should build a perfect hash from those IDs instead.
    match name.local {
        atom!("a")          => make!(HTMLAnchorElement),
        atom!("abbr")       => make!(HTMLElement),
        atom!("acronym")    => make!(HTMLElement),
        atom!("address")    => make!(HTMLElement),
        atom!("applet")     => make!(HTMLAppletElement),
        atom!("area")       => make!(HTMLAreaElement),
        atom!("article")    => make!(HTMLElement),
        atom!("aside")      => make!(HTMLElement),
        atom!("audio")      => make!(HTMLAudioElement),
        atom!("b")          => make!(HTMLElement),
        atom!("base")       => make!(HTMLBaseElement),
        atom!("bdi")        => make!(HTMLElement),
        atom!("bdo")        => make!(HTMLElement),
        atom!("bgsound")    => make!(HTMLElement),
        atom!("big")        => make!(HTMLElement),
        atom!("blockquote") => make!(HTMLElement),
        atom!("body")       => make!(HTMLBodyElement),
        atom!("br")         => make!(HTMLBRElement),
        atom!("button")     => make!(HTMLButtonElement),
        atom!("canvas")     => make!(HTMLCanvasElement),
        atom!("caption")    => make!(HTMLTableCaptionElement),
        atom!("center")     => make!(HTMLElement),
        atom!("cite")       => make!(HTMLElement),
        atom!("code")       => make!(HTMLElement),
        atom!("col")        => make!(HTMLTableColElement),
        atom!("colgroup")   => make!(HTMLTableColElement),
        atom!("data")       => make!(HTMLDataElement),
        atom!("datalist")   => make!(HTMLDataListElement),
        atom!("dd")         => make!(HTMLElement),
        atom!("del")        => make!(HTMLModElement),
        atom!("details")    => make!(HTMLElement),
        atom!("dfn")        => make!(HTMLElement),
        atom!("dir")        => make!(HTMLDirectoryElement),
        atom!("div")        => make!(HTMLDivElement),
        atom!("dl")         => make!(HTMLDListElement),
        atom!("dt")         => make!(HTMLElement),
        atom!("em")         => make!(HTMLElement),
        atom!("embed")      => make!(HTMLEmbedElement),
        atom!("fieldset")   => make!(HTMLFieldSetElement),
        atom!("figcaption") => make!(HTMLElement),
        atom!("figure")     => make!(HTMLElement),
        atom!("font")       => make!(HTMLFontElement),
        atom!("footer")     => make!(HTMLElement),
        atom!("form")       => make!(HTMLFormElement),
        atom!("frame")      => make!(HTMLFrameElement),
        atom!("frameset")   => make!(HTMLFrameSetElement),
        atom!("h1")         => make!(HTMLHeadingElement, Heading1),
        atom!("h2")         => make!(HTMLHeadingElement, Heading2),
        atom!("h3")         => make!(HTMLHeadingElement, Heading3),
        atom!("h4")         => make!(HTMLHeadingElement, Heading4),
        atom!("h5")         => make!(HTMLHeadingElement, Heading5),
        atom!("h6")         => make!(HTMLHeadingElement, Heading6),
        atom!("head")       => make!(HTMLHeadElement),
        atom!("header")     => make!(HTMLElement),
        atom!("hgroup")     => make!(HTMLElement),
        atom!("hr")         => make!(HTMLHRElement),
        atom!("html")       => make!(HTMLHtmlElement),
        atom!("i")          => make!(HTMLElement),
        atom!("iframe")     => make!(HTMLIFrameElement),
        atom!("img")        => make!(HTMLImageElement),
        atom!("input")      => make!(HTMLInputElement),
        atom!("ins")        => make!(HTMLModElement),
        atom!("isindex")    => make!(HTMLElement),
        atom!("kbd")        => make!(HTMLElement),
        atom!("label")      => make!(HTMLLabelElement),
        atom!("legend")     => make!(HTMLLegendElement),
        atom!("li")         => make!(HTMLLIElement),
        atom!("link")       => make!(HTMLLinkElement),
        atom!("main")       => make!(HTMLElement),
        atom!("map")        => make!(HTMLMapElement),
        atom!("mark")       => make!(HTMLElement),
        atom!("marquee")    => make!(HTMLElement),
        atom!("meta")       => make!(HTMLMetaElement),
        atom!("meter")      => make!(HTMLMeterElement),
        atom!("nav")        => make!(HTMLElement),
        atom!("nobr")       => make!(HTMLElement),
        atom!("noframes")   => make!(HTMLElement),
        atom!("noscript")   => make!(HTMLElement),
        atom!("object")     => make!(HTMLObjectElement),
        atom!("ol")         => make!(HTMLOListElement),
        atom!("optgroup")   => make!(HTMLOptGroupElement),
        atom!("option")     => make!(HTMLOptionElement),
        atom!("output")     => make!(HTMLOutputElement),
        atom!("p")          => make!(HTMLParagraphElement),
        atom!("param")      => make!(HTMLParamElement),
        atom!("pre")        => make!(HTMLPreElement),
        atom!("progress")   => make!(HTMLProgressElement),
        atom!("q")          => make!(HTMLQuoteElement),
        atom!("rp")         => make!(HTMLElement),
        atom!("rt")         => make!(HTMLElement),
        atom!("ruby")       => make!(HTMLElement),
        atom!("s")          => make!(HTMLElement),
        atom!("samp")       => make!(HTMLElement),
        atom!("script")     => make!(HTMLScriptElement, creator),
        atom!("section")    => make!(HTMLElement),
        atom!("select")     => make!(HTMLSelectElement),
        atom!("small")      => make!(HTMLElement),
        atom!("source")     => make!(HTMLSourceElement),
        atom!("spacer")     => make!(HTMLElement),
        atom!("span")       => make!(HTMLSpanElement),
        atom!("strike")     => make!(HTMLElement),
        atom!("strong")     => make!(HTMLElement),
        atom!("style")      => make!(HTMLStyleElement),
        atom!("sub")        => make!(HTMLElement),
        atom!("summary")    => make!(HTMLElement),
        atom!("sup")        => make!(HTMLElement),
        atom!("table")      => make!(HTMLTableElement),
        atom!("tbody")      => make!(HTMLTableSectionElement),
        atom!("td")         => make!(HTMLTableDataCellElement),
        atom!("template")   => make!(HTMLTemplateElement),
        atom!("textarea")   => make!(HTMLTextAreaElement),
        atom!("th")         => make!(HTMLTableHeaderCellElement),
        atom!("time")       => make!(HTMLTimeElement),
        atom!("title")      => make!(HTMLTitleElement),
        atom!("tr")         => make!(HTMLTableRowElement),
        atom!("tt")         => make!(HTMLElement),
        atom!("track")      => make!(HTMLTrackElement),
        atom!("u")          => make!(HTMLElement),
        atom!("ul")         => make!(HTMLUListElement),
        atom!("var")        => make!(HTMLElement),
        atom!("video")      => make!(HTMLVideoElement),
        atom!("wbr")        => make!(HTMLElement),
        _                   => make!(HTMLUnknownElement),
    }
}

trait SinkHelpers {
    fn get_or_create(&self, child: NodeOrText<TrustedNodeAddress>) -> Temporary<Node>;
}

impl SinkHelpers for servohtmlparser::Sink {
    fn get_or_create(&self, child: NodeOrText<TrustedNodeAddress>) -> Temporary<Node> {
        match child {
            AppendNode(n) => Temporary::new(unsafe { JS::from_trusted_node_address(n) }),
            AppendText(t) => {
                let doc = self.document.root();
                let text = Text::new(t, *doc);
                NodeCast::from_temporary(text)
            }
        }
    }
}

impl<'a> TreeSink<TrustedNodeAddress> for servohtmlparser::Sink {
    fn get_document(&mut self) -> TrustedNodeAddress {
        let doc = self.document.root();
        let node: JSRef<Node> = NodeCast::from_ref(*doc);
        node.to_trusted_node_address()
    }

    fn same_node(&self, x: TrustedNodeAddress, y: TrustedNodeAddress) -> bool {
        x == y
    }

    fn elem_name(&self, target: TrustedNodeAddress) -> QualName {
        let node: Root<Node> = unsafe { JS::from_trusted_node_address(target).root() };
        let elem: JSRef<Element> = ElementCast::to_ref(*node)
            .expect("tried to get name of non-Element in HTML parsing");
        QualName {
            ns: elem.get_namespace().clone(),
            local: elem.get_local_name().clone(),
        }
    }

    fn create_element(&mut self, name: QualName, attrs: Vec<Attribute>)
            -> TrustedNodeAddress {
        let doc = self.document.root();
        let elem = build_element_from_tag(name, None, *doc, ParserCreated).root();

        for attr in attrs.into_iter() {
            elem.set_attribute_from_parser(attr.name, attr.value, None);
        }

        let node: JSRef<Node> = NodeCast::from_ref(*elem);
        node.to_trusted_node_address()
    }

    fn create_comment(&mut self, text: String) -> TrustedNodeAddress {
        let doc = self.document.root();
        let comment = Comment::new(text, *doc);
        let node: Root<Node> = NodeCast::from_temporary(comment).root();
        node.to_trusted_node_address()
    }

    fn append_before_sibling(&mut self,
            sibling: TrustedNodeAddress,
            new_node: NodeOrText<TrustedNodeAddress>) -> Result<(), NodeOrText<TrustedNodeAddress>> {
        // If there is no parent, return the node to the parser.
        let sibling: Root<Node> = unsafe { JS::from_trusted_node_address(sibling).root() };
        let parent = match sibling.parent_node() {
            Some(p) => p.root(),
            None => return Err(new_node),
        };

        let child = self.get_or_create(new_node).root();
        assert!(parent.InsertBefore(*child, Some(*sibling)).is_ok());
        Ok(())
    }

    fn parse_error(&mut self, msg: MaybeOwned<'static>) {
        error!("Parse error: {:s}", msg);
    }

    fn set_quirks_mode(&mut self, mode: QuirksMode) {
        let doc = self.document.root();
        doc.set_quirks_mode(mode);
    }

    fn append(&mut self, parent: TrustedNodeAddress, child: NodeOrText<TrustedNodeAddress>) {
        let parent: Root<Node> = unsafe { JS::from_trusted_node_address(parent).root() };
        let child = self.get_or_create(child).root();

        // FIXME(#3701): Use a simpler algorithm and merge adjacent text nodes
        assert!(parent.AppendChild(*child).is_ok());
    }

    fn append_doctype_to_document(&mut self, name: String, public_id: String, system_id: String) {
        let doc = self.document.root();
        let doc_node: JSRef<Node> = NodeCast::from_ref(*doc);
        let doctype = DocumentType::new(name, Some(public_id), Some(system_id), *doc);
        let node: Root<Node> = NodeCast::from_temporary(doctype).root();

        assert!(doc_node.AppendChild(*node).is_ok());
    }

    fn add_attrs_if_missing(&mut self, target: TrustedNodeAddress, attrs: Vec<Attribute>) {
        let node: Root<Node> = unsafe { JS::from_trusted_node_address(target).root() };
        let elem: JSRef<Element> = ElementCast::to_ref(*node)
            .expect("tried to set attrs on non-Element in HTML parsing");
        for attr in attrs.into_iter() {
            elem.set_attribute_from_parser(attr.name, attr.value, None);
        }
    }

    fn remove_from_parent(&mut self, _target: TrustedNodeAddress) {
        error!("remove_from_parent not implemented!");
    }

    fn mark_script_already_started(&mut self, node: TrustedNodeAddress) {
        let node: Root<Node> = unsafe { JS::from_trusted_node_address(node).root() };
        let script: Option<JSRef<HTMLScriptElement>> = HTMLScriptElementCast::to_ref(*node);
        script.map(|script| script.mark_already_started());
    }

    fn complete_script(&mut self, node: TrustedNodeAddress) {
        let node: Root<Node> = unsafe { JS::from_trusted_node_address(node).root() };
        let script: Option<JSRef<HTMLScriptElement>> = HTMLScriptElementCast::to_ref(*node);
        script.map(|script| script.prepare());
    }
}

// The url from msg_load_data is ignored here
pub fn parse_html(page: &Page,
                  document: JSRef<Document>,
                  input: HTMLInput,
                  resource_task: ResourceTask,
                  msg_load_data: Option<MsgLoadData>) {
    let (base_url, load_response) = match input {
        InputUrl(ref url) => {
            // Wait for the LoadResponse so that the parser knows the final URL.
            let (input_chan, input_port) = channel();
            let mut load_data = LoadData::new(url.clone());
            msg_load_data.map(|m| {
                load_data.headers = m.headers;
                load_data.method = m.method;
                load_data.data = m.data;
            });
            resource_task.send(Load(load_data, input_chan));

            let load_response = input_port.recv();

            debug!("Fetched page; metadata is {:?}", load_response.metadata);

            load_response.metadata.headers.as_ref().map(|headers| {
                let header = headers.iter().find(|h|
                    h.header_name().as_slice().to_ascii_lower() == "last-modified".to_string()
                );

                match header {
                    Some(h) => document.set_last_modified(
                        parse_last_modified(h.header_value().as_slice())),
                    None => {},
                };
            });

            let base_url = load_response.metadata.final_url.clone();

            {
                // Store the final URL before we start parsing, so that DOM routines
                // (e.g. HTMLImageElement::update_image) can resolve relative URLs
                // correctly.
                *page.mut_url() = Some((base_url.clone(), true));
            }

            (Some(base_url), Some(load_response))
        },
        InputString(_) => {
            match *page.url() {
                Some((ref page_url, _)) => (Some(page_url.clone()), None),
                None => (None, None),
            }
        },
    };

    let parser = ServoHTMLParser::new(base_url.clone(), document).root();
    let parser: JSRef<ServoHTMLParser> = *parser;

    task_state::enter(InHTMLParser);

    match input {
        InputString(s) => {
            parser.parse_chunk(s);
        }
        InputUrl(url) => {
            let load_response = load_response.unwrap();
            match load_response.metadata.content_type {
                Some((ref t, _)) if t.as_slice().eq_ignore_ascii_case("image") => {
                    let page = format!("<html><body><img src='{:s}' /></body></html>", base_url.as_ref().unwrap().serialize());
                    parser.parse_chunk(page);
                },
                _ => {
                    for msg in load_response.progress_port.iter() {
                        match msg {
                            Payload(data) => {
                                // FIXME: use Vec<u8> (html5ever #34)
                                let data = UTF_8.decode(data.as_slice(), DecodeReplace).unwrap();
                                parser.parse_chunk(data);
                            }
                            Done(Err(err)) => {
                                fail!("Failed to load page URL {:s}, error: {:s}", url.serialize(), err);
                            }
                            Done(Ok(())) => break,
                        }
                    }
                }
            }
        }
    }

    parser.finish();

    task_state::exit(InHTMLParser);

    debug!("finished parsing");
}
