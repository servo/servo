/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

#![allow(unrooted_must_root)]

use document_loader::DocumentLoader;
use dom::bindings::codegen::Bindings::DocumentBinding::DocumentMethods;
use dom::bindings::codegen::Bindings::HTMLTemplateElementBinding::HTMLTemplateElementMethods;
use dom::bindings::codegen::Bindings::NodeBinding::NodeMethods;
use dom::bindings::codegen::InheritTypes::{CharacterDataTypeId, NodeTypeId};
use dom::bindings::conversions::Castable;
use dom::bindings::js::{JS, Root};
use dom::bindings::js::{RootedReference};
use dom::characterdata::CharacterData;
use dom::comment::Comment;
use dom::document::Document;
use dom::document::{DocumentSource, IsHTMLDocument};
use dom::documenttype::DocumentType;
use dom::element::{Element, ElementCreator};
use dom::htmlformelement::HTMLFormElement;
use dom::htmlscriptelement::HTMLScriptElement;
use dom::htmltemplateelement::HTMLTemplateElement;
use dom::node::Node;
use dom::node::{document_from_node, window_from_node};
use dom::processinginstruction::ProcessingInstruction;
use dom::servohtmlparser;
use dom::servohtmlparser::{FragmentContext, ServoHTMLParser};
use encoding::types::Encoding;
use html5ever::Attribute;
use html5ever::serialize::TraversalScope;
use html5ever::serialize::TraversalScope::{ChildrenOnly, IncludeNode};
use html5ever::serialize::{AttrRef, Serializable, Serializer};
use html5ever::tree_builder::{NextParserState, NodeOrText, QuirksMode, TreeSink};
use msg::constellation_msg::PipelineId;
use parse::Parser;
use std::borrow::Cow;
use std::io::{self, Write};
use string_cache::QualName;
use tendril::StrTendril;
use url::Url;
use util::str::DOMString;

impl<'a> TreeSink for servohtmlparser::Sink {
    type Handle = JS<Node>;

    fn get_document(&mut self) -> JS<Node> {
        JS::from_ref(self.document.root().upcast())
    }

    fn get_template_contents(&self, target: JS<Node>) -> JS<Node> {
        let target = target.root();
        let template = target.downcast::<HTMLTemplateElement>()
            .expect("tried to get template contents of non-HTMLTemplateElement in HTML parsing");
        JS::from_ref(template.Content().upcast())
    }

    fn same_node(&self, x: JS<Node>, y: JS<Node>) -> bool {
        x == y
    }

    fn elem_name(&self, target: JS<Node>) -> QualName {
        let node: Root<Node> = target.root();
        let elem = node.downcast::<Element>()
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
                                   ElementCreator::ParserCreated);

        for attr in attrs {
            elem.r().set_attribute_from_parser(attr.name, attr.value.into(), None);
        }

        JS::from_ref(elem.upcast())
    }

    fn create_comment(&mut self, text: StrTendril) -> JS<Node> {
        let doc = self.document.root();
        let comment = Comment::new(text.into(), doc.r());
        JS::from_ref(comment.upcast())
    }

    fn append_before_sibling(&mut self,
            sibling: JS<Node>,
            new_node: NodeOrText<JS<Node>>) -> Result<(), NodeOrText<JS<Node>>> {
        // If there is no parent, return the node to the parser.
        let sibling: Root<Node> = sibling.root();
        let parent = match sibling.r().GetParentNode() {
            Some(p) => p,
            None => return Err(new_node),
        };

        let child = self.get_or_create(new_node);
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
        let child = self.get_or_create(child);

        // FIXME(#3701): Use a simpler algorithm and merge adjacent text nodes
        assert!(parent.r().AppendChild(child.r()).is_ok());
    }

    fn append_doctype_to_document(&mut self, name: StrTendril, public_id: StrTendril,
                                  system_id: StrTendril) {
        let doc = self.document.root();
        let doctype = DocumentType::new(
            name.into(), Some(public_id.into()), Some(system_id.into()), doc.r());
        doc.upcast::<Node>().AppendChild(doctype.upcast()).expect("Appending failed");
    }

    fn add_attrs_if_missing(&mut self, target: JS<Node>, attrs: Vec<Attribute>) {
        let node: Root<Node> = target.root();
        let elem = node.downcast::<Element>()
            .expect("tried to set attrs on non-Element in HTML parsing");
        for attr in attrs {
            elem.set_attribute_from_parser(attr.name, attr.value.into(), None);
        }
    }

    fn remove_from_parent(&mut self, target: JS<Node>) {
        let node = target.root();
        if let Some(ref parent) = node.r().GetParentNode() {
            parent.r().RemoveChild(node.r()).unwrap();
        }
    }

    fn mark_script_already_started(&mut self, node: JS<Node>) {
        let node: Root<Node> = node.root();
        let script = node.downcast::<HTMLScriptElement>();
        script.map(|script| script.mark_already_started());
    }

    fn complete_script(&mut self, node: JS<Node>) -> NextParserState {
        let node: Root<Node> = node.root();
        let script = node.downcast::<HTMLScriptElement>();
        if let Some(script) = script {
            return script.prepare();
        }
        NextParserState::Continue
    }

    fn reparent_children(&mut self, node: JS<Node>, new_parent: JS<Node>) {
        let new_parent = new_parent.root();
        let new_parent = new_parent.r();
        let old_parent = node.root();
        let old_parent = old_parent.r();
        while let Some(ref child) = old_parent.GetFirstChild() {
            new_parent.AppendChild(child.r()).unwrap();
        }

    }
}

impl<'a> Serializable for &'a Node {
    fn serialize<'wr, Wr: Write>(&self, serializer: &mut Serializer<'wr, Wr>,
                                 traversal_scope: TraversalScope) -> io::Result<()> {
        let node = *self;
        match (traversal_scope, node.type_id()) {
            (_, NodeTypeId::Element(..)) => {
                let elem = node.downcast::<Element>().unwrap();
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
                        let ar: AttrRef = (&qname, &**value);
                        ar
                    });
                    try!(serializer.start_elem(name.clone(), attr_refs));
                }

                let children = if let Some(tpl) = node.downcast::<HTMLTemplateElement>() {
                    // https://github.com/w3c/DOM-Parsing/issues/1
                    tpl.Content().upcast::<Node>().children()
                } else {
                    node.children()
                };

                for handle in children {
                    try!(handle.r().serialize(serializer, IncludeNode));
                }

                if traversal_scope == IncludeNode {
                    try!(serializer.end_elem(name.clone()));
                }
                Ok(())
            },

            (ChildrenOnly, NodeTypeId::Document) => {
                for handle in node.children() {
                    try!(handle.r().serialize(serializer, IncludeNode));
                }
                Ok(())
            },

            (ChildrenOnly, _) => Ok(()),

            (IncludeNode, NodeTypeId::DocumentType) => {
                let doctype = node.downcast::<DocumentType>().unwrap();
                serializer.write_doctype(&doctype.name())
            },

            (IncludeNode, NodeTypeId::CharacterData(CharacterDataTypeId::Text)) => {
                let cdata = node.downcast::<CharacterData>().unwrap();
                serializer.write_text(&cdata.data())
            },

            (IncludeNode, NodeTypeId::CharacterData(CharacterDataTypeId::Comment)) => {
                let cdata = node.downcast::<CharacterData>().unwrap();
                serializer.write_comment(&cdata.data())
            },

            (IncludeNode, NodeTypeId::CharacterData(CharacterDataTypeId::ProcessingInstruction)) => {
                let pi = node.downcast::<ProcessingInstruction>().unwrap();
                let data = pi.upcast::<CharacterData>().data();
                serializer.write_processing_instruction(&pi.target(), &data)
            },

            (IncludeNode, NodeTypeId::DocumentFragment) => Ok(()),

            (IncludeNode, NodeTypeId::Document) => panic!("Can't serialize Document node itself"),
        }
    }
}

pub enum ParseContext<'a> {
    Fragment(FragmentContext<'a>),
    Owner(Option<PipelineId>),
}

pub fn parse_html(document: &Document,
                  input: String,
                  url: Url,
                  context: ParseContext) {
    let parser = match context {
        ParseContext::Owner(owner) =>
            ServoHTMLParser::new(Some(url), document, owner),
        ParseContext::Fragment(fc) =>
            ServoHTMLParser::new_for_fragment(Some(url), document, fc),
    };
    parser.r().parse_chunk(input.into());
}

// https://html.spec.whatwg.org/multipage/#parsing-html-fragments
pub fn parse_html_fragment(context_node: &Node,
                           input: DOMString,
                           output: &Node) {
    let window = window_from_node(context_node);
    let context_document = document_from_node(context_node);
    let context_document = context_document.r();
    let url = context_document.url();

    // Step 1.
    let loader = DocumentLoader::new(&*context_document.loader());
    let document = Document::new(window.r(), Some(url.clone()),
                                 IsHTMLDocument::HTMLDocument,
                                 None, None,
                                 DocumentSource::FromParser,
                                 loader);

    // Step 2.
    document.r().set_quirks_mode(context_document.quirks_mode());

    // Step 11.
    let form = context_node.inclusive_ancestors()
                           .find(|element| element.r().is::<HTMLFormElement>());
    let fragment_context = FragmentContext {
        context_elem: context_node,
        form_elem: form.r(),
    };
    parse_html(document.r(), input, url.clone(), ParseContext::Fragment(fragment_context));

    // Step 14.
    let root_element = document.r().GetDocumentElement().expect("no document element");
    for child in root_element.upcast::<Node>().children() {
        output.AppendChild(child.r()).unwrap();
    }
}
