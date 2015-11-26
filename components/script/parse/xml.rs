/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

#![allow(unrooted_must_root)]

use dom::bindings::codegen::Bindings::NodeBinding::NodeMethods;
use dom::bindings::inheritance::Castable;
use dom::bindings::js::{JS, RootedReference};
use dom::comment::Comment;
use dom::document::Document;
use dom::documenttype::DocumentType;
use dom::element::{Element, ElementCreator};
use dom::node::Node;
use dom::processinginstruction::ProcessingInstruction;
use dom::servoxmlparser;
use std::borrow::Cow;
use string_cache::QualName;
use tendril::StrTendril;
use url::Url;
use util::str::DOMString;
use xml5ever::tokenizer::Attribute;
use xml5ever::tree_builder::{NodeOrText, TreeSink};

impl<'a> TreeSink for servoxmlparser::Sink {
    type Handle = JS<Node>;

    fn parse_error(&mut self, msg: Cow<'static, str>) {
        debug!("Parse error: {}", msg);
    }

    fn get_document(&mut self) -> JS<Node> {
        JS::from_ref(self.document.upcast())
    }

    fn elem_name(&self, target: &JS<Node>) -> QualName {
        let elem = target.downcast::<Element>()
            .expect("tried to get name of non-Element in XML parsing");
        QualName {
            ns: elem.namespace().clone(),
            local: elem.local_name().clone(),
        }
    }

    fn create_element(&mut self, name: QualName, attrs: Vec<Attribute>)
            -> JS<Node> {
        let elem = Element::create(name, None, &*self.document,
                                   ElementCreator::ParserCreated);

        for attr in attrs {
            elem.set_attribute_from_parser(attr.name, DOMString::from(String::from(attr.value)), None);
        }

        JS::from_ref(elem.upcast())
    }

    fn create_comment(&mut self, text: StrTendril) -> JS<Node> {
        let comment = Comment::new(DOMString::from(String::from(text)), &*self.document);
        JS::from_ref(comment.upcast())
    }

    fn append(&mut self, parent: JS<Node>, child: NodeOrText<JS<Node>>) {
        let child = self.get_or_create(child);

        assert!(parent.AppendChild(child.r()).is_ok());
    }

    fn append_doctype_to_document(&mut self, name: StrTendril, public_id: StrTendril,
                                  system_id: StrTendril) {
        let doc = &*self.document;
        let doctype = DocumentType::new(
            DOMString::from(String::from(name)), Some(DOMString::from(String::from(public_id))),
            Some(DOMString::from(String::from(system_id))), doc);
        doc.upcast::<Node>().AppendChild(doctype.upcast()).expect("Appending failed");
    }

    fn create_pi(&mut self, target: StrTendril,  data: StrTendril) -> JS<Node> {
        let doc = &*self.document;
        let pi = ProcessingInstruction::new(
            DOMString::from(String::from(target)), DOMString::from(String::from(data)),
            doc);
        JS::from_ref(pi.upcast())
    }
}


pub enum ParseContext {
    Owner(Option<i32>)
}


pub fn parse_xml(_document: &Document,
                 _input: DOMString,
                 _url: Url,
                 _context: ParseContext) {
}
