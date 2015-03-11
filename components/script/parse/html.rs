/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

#![allow(unsafe_blocks, unrooted_must_root)]

use dom::attr::AttrHelpers;
use dom::bindings::codegen::Bindings::NodeBinding::NodeMethods;
use dom::bindings::codegen::InheritTypes::{NodeCast, ElementCast, HTMLScriptElementCast};
use dom::bindings::js::{JS, JSRef, Temporary, OptionalRootable, Root};
use dom::comment::Comment;
use dom::document::{Document, DocumentHelpers};
use dom::documenttype::DocumentType;
use dom::element::{Element, AttributeHandlers, ElementHelpers, ElementCreator};
use dom::htmlscriptelement::HTMLScriptElement;
use dom::htmlscriptelement::HTMLScriptElementHelpers;
use dom::node::{Node, NodeHelpers};
use dom::servohtmlparser;
use dom::servohtmlparser::ServoHTMLParser;
use dom::text::Text;
use parse::Parser;

use encoding::types::Encoding;

use msg::constellation_msg::PipelineId;
use std::string::CowString;
use url::Url;
use html5ever::Attribute;
use html5ever::tree_builder::{TreeSink, QuirksMode, NodeOrText, AppendNode, AppendText, NextParserState};
use string_cache::QualName;

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

    fn parse_error(&mut self, msg: CowString<'static>) {
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

    fn remove_from_parent(&mut self, _target: JS<Node>) {
        error!("remove_from_parent not implemented!");
    }

    fn mark_script_already_started(&mut self, node: JS<Node>) {
        let node: Root<Node> = node.root();
        let script: Option<JSRef<HTMLScriptElement>> = HTMLScriptElementCast::to_ref(node.r());
        script.map(|script| script.mark_already_started());
    }

    fn complete_script(&mut self, node: JS<Node>) -> NextParserState {
        let node: Root<Node> = node.root();
        let script: Option<JSRef<HTMLScriptElement>> = HTMLScriptElementCast::to_ref(node.r());
        if let Some(script) = script {
            return script.prepare();
        }
        NextParserState::Continue
    }

    fn reparent_children(&mut self, _node: JS<Node>, _new_parent: JS<Node>) {
        panic!("unimplemented")
    }
}

pub fn parse_html(owning_pipeline: Option<PipelineId>,
                  document: JSRef<Document>,
                  input: String,
                  url: &Url) {
    let parser = ServoHTMLParser::new(Some(url.clone()), document, owning_pipeline).root();
    parser.r().parse_chunk(input);
}
