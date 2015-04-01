/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

#![allow(unsafe_code, unrooted_must_root)]

use dom::attr::AttrHelpers;
use dom::bindings::codegen::Bindings::NodeBinding::NodeMethods;
use dom::bindings::codegen::InheritTypes::{NodeCast, ElementCast, HTMLScriptElementCast};
use dom::bindings::codegen::InheritTypes::{DocumentTypeCast, TextCast, CommentCast};
use dom::bindings::codegen::InheritTypes::ProcessingInstructionCast;
use dom::bindings::js::{JS, JSRef, Temporary, OptionalRootable, Root};
use dom::comment::Comment;
use dom::document::{Document, DocumentHelpers};
use dom::documenttype::DocumentType;
use dom::element::{Element, AttributeHandlers, ElementHelpers, ElementCreator};
use dom::htmlscriptelement::HTMLScriptElement;
use dom::htmlscriptelement::HTMLScriptElementHelpers;
use dom::node::{Node, NodeHelpers, NodeTypeId};
use dom::processinginstruction::ProcessingInstruction;
use dom::servohtmlparser;
use dom::servohtmlparser::{ServoHTMLParser, FragmentContext};
use dom::text::Text;
use parse::Parser;

use encoding::all::UTF_8;
use encoding::types::{Encoding, DecoderTrap};

use net::resource_task::{ProgressMsg, LoadResponse};
use util::task_state;
use util::task_state::IN_HTML_PARSER;
use std::ascii::AsciiExt;
use std::borrow::Cow;
use std::old_io::{Writer, IoResult};
use url::Url;
use html5ever::Attribute;
use html5ever::serialize::{Serializable, Serializer, AttrRef};
use html5ever::serialize::TraversalScope;
use html5ever::serialize::TraversalScope::{IncludeNode, ChildrenOnly};
use html5ever::tree_builder::{TreeSink, QuirksMode, NodeOrText, AppendNode, AppendText};
use string_cache::QualName;

pub enum HTMLInput {
    InputString(String),
    InputUrl(LoadResponse),
}

trait SinkHelpers {
    fn get_or_create(&self, child: NodeOrText<JS<Node>>) -> Temporary<Node>;
}

impl SinkHelpers for servohtmlparser::Sink {
    fn get_or_create(&self, child: NodeOrText<JS<Node>>) -> Temporary<Node> {
        match child {
            AppendNode(n) => Temporary::new(n),
            AppendText(t) => {
                let doc = self.document.root();
                let text = Text::new(t, doc.r());
                NodeCast::from_temporary(text)
            }
        }
    }
}

impl<'a> TreeSink for servohtmlparser::Sink {
    type Handle = JS<Node>;
    fn get_document(&mut self) -> JS<Node> {
        let doc = self.document.root();
        let node: JSRef<Node> = NodeCast::from_ref(doc.r());
        JS::from_rooted(node)
    }

    fn same_node(&self, x: JS<Node>, y: JS<Node>) -> bool {
        x == y
    }

    fn elem_name(&self, target: JS<Node>) -> QualName {
        let node: Root<Node> = target.root();
        let elem: JSRef<Element> = ElementCast::to_ref(node.r())
            .expect("tried to get name of non-Element in HTML parsing");
        QualName {
            ns: elem.namespace().clone(),
            local: elem.local_name().clone(),
        }
    }

    fn create_element(&mut self, name: QualName, attrs: Vec<Attribute>)
            -> JS<Node> {
        let doc = self.document.root();
        let elem = Element::create(name, None, doc.r(),
                                   ElementCreator::ParserCreated).root();

        for attr in attrs.into_iter() {
            elem.r().set_attribute_from_parser(attr.name, attr.value, None);
        }

        let node: JSRef<Node> = NodeCast::from_ref(elem.r());
        JS::from_rooted(node)
    }

    fn create_comment(&mut self, text: String) -> JS<Node> {
        let doc = self.document.root();
        let comment = Comment::new(text, doc.r());
        let node: Root<Node> = NodeCast::from_temporary(comment).root();
        JS::from_rooted(node.r())
    }

    fn append_before_sibling(&mut self,
            sibling: JS<Node>,
            new_node: NodeOrText<JS<Node>>) -> Result<(), NodeOrText<JS<Node>>> {
        // If there is no parent, return the node to the parser.
        let sibling: Root<Node> = sibling.root();
        let parent = match sibling.r().parent_node() {
            Some(p) => p.root(),
            None => return Err(new_node),
        };

        let child = self.get_or_create(new_node).root();
        assert!(parent.r().InsertBefore(child.r(), Some(sibling.r())).is_ok());
        Ok(())
    }

    fn parse_error(&mut self, msg: Cow<'static, str>) {
        debug!("Parse error: {}", msg);
    }

    fn set_quirks_mode(&mut self, mode: QuirksMode) {
        let doc = self.document.root();
        doc.r().set_quirks_mode(mode);
    }

    fn append(&mut self, parent: JS<Node>, child: NodeOrText<JS<Node>>) {
        let parent: Root<Node> = parent.root();
        let child = self.get_or_create(child).root();

        // FIXME(#3701): Use a simpler algorithm and merge adjacent text nodes
        assert!(parent.r().AppendChild(child.r()).is_ok());
    }

    fn append_doctype_to_document(&mut self, name: String, public_id: String, system_id: String) {
        let doc = self.document.root();
        let doc_node: JSRef<Node> = NodeCast::from_ref(doc.r());
        let doctype = DocumentType::new(name, Some(public_id), Some(system_id), doc.r());
        let node: Root<Node> = NodeCast::from_temporary(doctype).root();

        assert!(doc_node.AppendChild(node.r()).is_ok());
    }

    fn add_attrs_if_missing(&mut self, target: JS<Node>, attrs: Vec<Attribute>) {
        let node: Root<Node> = target.root();
        let elem: JSRef<Element> = ElementCast::to_ref(node.r())
            .expect("tried to set attrs on non-Element in HTML parsing");
        for attr in attrs.into_iter() {
            elem.set_attribute_from_parser(attr.name, attr.value, None);
        }
    }

    fn remove_from_parent(&mut self, target: JS<Node>) {
        let node: Root<Node> = target.root();
        let parent = node.r();
        while let Some(child) = parent.GetFirstChild() {
            parent.RemoveChild(child.root().r()).unwrap();
        }
    }

    fn mark_script_already_started(&mut self, node: JS<Node>) {
        let node: Root<Node> = node.root();
        let script: Option<JSRef<HTMLScriptElement>> = HTMLScriptElementCast::to_ref(node.r());
        script.map(|script| script.mark_already_started());
    }

    fn complete_script(&mut self, node: JS<Node>) {
        let node: Root<Node> = node.root();
        let script: Option<JSRef<HTMLScriptElement>> = HTMLScriptElementCast::to_ref(node.r());
        script.map(|script| script.prepare());
    }

    fn reparent_children(&mut self, node: JS<Node>, new_parent: JS<Node>) {
        let new_parent: Root<Node> = new_parent.root();
        let new_parent = new_parent.r();
        let old_parent: Root<Node> = node.root();
        let old_parent = old_parent.r();
        while let Some(child) = old_parent.GetFirstChild() {
           new_parent.AppendChild(child.root().r()).unwrap();
        }

    }
}

impl<'a> Serializable for JSRef<'a, Node> {
    fn serialize<'wr, Wr: Writer>(&self, serializer: &mut Serializer<'wr, Wr>,
                                  traversal_scope: TraversalScope) -> IoResult<()> {
        let node = *self;
        match (traversal_scope, node.type_id()) {
            (_, NodeTypeId::Element(..)) => {
                let elem: JSRef<Element> = ElementCast::to_ref(node).unwrap();
                let name = QualName::new(elem.namespace().clone(),
                                         elem.local_name().clone());
                if traversal_scope == IncludeNode {
                    let attrs = elem.attrs().iter().map(|at| {
                        let attr = at.root();
                        let qname = QualName::new(attr.r().namespace().clone(),
                                                  attr.r().local_name().clone());
                        let value = attr.r().value().clone();
                        (qname, value)
                    }).collect::<Vec<_>>();
                    let attr_refs = attrs.iter().map(|&(ref qname, ref value)| {
                        let ar: AttrRef = (&qname, value.as_slice());
                        ar
                    });
                    try!(serializer.start_elem(name.clone(), attr_refs));
                }

                for handle in node.children() {
                    try!(handle.serialize(serializer, IncludeNode));
                }

                if traversal_scope == IncludeNode {
                    try!(serializer.end_elem(name.clone()));
                }
                Ok(())
            },

            (ChildrenOnly, NodeTypeId::Document) => {
                for handle in node.children() {
                    try!(handle.serialize(serializer, IncludeNode));
                }
                Ok(())
            },

            (ChildrenOnly, _) => Ok(()),

            (IncludeNode, NodeTypeId::DocumentType) => {
                let doctype: JSRef<DocumentType> = DocumentTypeCast::to_ref(node).unwrap();
                serializer.write_doctype(doctype.name().as_slice())
            },

            (IncludeNode, NodeTypeId::Text) => {
                let text: JSRef<Text> = TextCast::to_ref(node).unwrap();
                let data = text.characterdata().data();
                serializer.write_text(data.as_slice())
            },

            (IncludeNode, NodeTypeId::Comment) => {
                let comment: JSRef<Comment> = CommentCast::to_ref(node).unwrap();
                let data = comment.characterdata().data();
                serializer.write_comment(data.as_slice())
            },

            (IncludeNode, NodeTypeId::ProcessingInstruction) => {
                let pi: JSRef<ProcessingInstruction> = ProcessingInstructionCast::to_ref(node).unwrap();
                let data = pi.characterdata().data();
                serializer.write_processing_instruction(pi.target().as_slice(),
                                                        data.as_slice())
            },

            (IncludeNode, NodeTypeId::DocumentFragment) => Ok(()),

            (IncludeNode, NodeTypeId::Document) => panic!("Can't serialize Document node itself"),
        }
    }
}

pub fn parse_html(document: JSRef<Document>,
                  input: HTMLInput,
                  url: &Url,
                  fragment_context: Option<FragmentContext>) {
    let parser = match fragment_context {
        None => ServoHTMLParser::new(Some(url.clone()), document).root(),
        Some(fc) => ServoHTMLParser::new_for_fragment(Some(url.clone()), document, fc).root(),
    };
    let parser: JSRef<ServoHTMLParser> = parser.r();

    let nested_parse = task_state::get().contains(task_state::IN_HTML_PARSER);
    if !nested_parse {
        task_state::enter(IN_HTML_PARSER);
    }

    fn parse_progress(parser: &JSRef<ServoHTMLParser>, url: &Url, load_response: &LoadResponse) {
        for msg in load_response.progress_port.iter() {
            match msg {
                ProgressMsg::Payload(data) => {
                    // FIXME: use Vec<u8> (html5ever #34)
                    let data = UTF_8.decode(data.as_slice(), DecoderTrap::Replace).unwrap();
                    parser.parse_chunk(data);
                }
                ProgressMsg::Done(Err(err)) => {
                    debug!("Failed to load page URL {}, error: {}", url.serialize(), err);
                    // TODO(Savago): we should send a notification to callers #5463.
                    break;
                }
                ProgressMsg::Done(Ok(())) => break,
            }
        }
    };

    match input {
        HTMLInput::InputString(s) => {
            parser.parse_chunk(s);
        }
        HTMLInput::InputUrl(load_response) => {
            match load_response.metadata.content_type {
                Some((ref t, _)) if t.as_slice().eq_ignore_ascii_case("image") => {
                    let page = format!("<html><body><img src='{}' /></body></html>", url.serialize());
                    parser.parse_chunk(page);
                },
                Some((ref t, ref st)) if t.as_slice().eq_ignore_ascii_case("text") &&
                                         st.as_slice().eq_ignore_ascii_case("plain") => {
                    // FIXME: When servo/html5ever#109 is fixed remove <plaintext> usage and
                    // replace with fix from that issue.

                    // text/plain documents require setting the tokenizer into PLAINTEXT mode.
                    // This is done by using a <plaintext> element as the html5ever tokenizer
                    // provides no other way to change to that state.
                    // Spec for text/plain handling is:
                    // https://html.spec.whatwg.org/multipage/browsers.html#read-text
                    let page = format!("<pre>\u{000A}<plaintext>");
                    parser.parse_chunk(page);
                    parse_progress(&parser, url, &load_response);
                },
                _ => {
                    parse_progress(&parser, url, &load_response);
                }
            }
        }
    }

    parser.finish();

    if !nested_parse {
        task_state::exit(IN_HTML_PARSER);
    }

    debug!("finished parsing");
}
