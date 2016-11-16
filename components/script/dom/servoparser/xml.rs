/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

#![allow(unrooted_must_root)]

use dom::bindings::codegen::Bindings::NodeBinding::NodeMethods;
use dom::bindings::inheritance::Castable;
use dom::bindings::js::{JS, Root};
use dom::bindings::str::DOMString;
use dom::bindings::trace::JSTraceable;
use dom::comment::Comment;
use dom::document::Document;
use dom::documenttype::DocumentType;
use dom::element::{Element, ElementCreator};
use dom::htmlscriptelement::HTMLScriptElement;
use dom::node::Node;
use dom::processinginstruction::ProcessingInstruction;
use dom::text::Text;
use html5ever_atoms::{Prefix, QualName};
use js::jsapi::JSTracer;
use servo_url::ServoUrl;
use std::borrow::Cow;
use super::Sink;
use xml5ever::tendril::StrTendril;
use xml5ever::tokenizer::{Attribute, QName, XmlTokenizer};
use xml5ever::tree_builder::{NextParserState, NodeOrText};
use xml5ever::tree_builder::{Tracer as XmlTracer, TreeSink, XmlTreeBuilder};

#[derive(HeapSizeOf, JSTraceable)]
#[must_root]
pub struct Tokenizer {
    #[ignore_heap_size_of = "Defined in xml5ever"]
    inner: XmlTokenizer<XmlTreeBuilder<JS<Node>, Sink>>,
}

impl Tokenizer {
    pub fn new(document: &Document, url: ServoUrl) -> Self {
        let sink = Sink {
            base_url: url,
            document: JS::from_ref(document),
        };

        let tb = XmlTreeBuilder::new(sink);
        let tok = XmlTokenizer::new(tb, Default::default());

        Tokenizer {
            inner: tok,
        }
    }

    pub fn feed(&mut self, input: String) {
        self.inner.feed(input.into())
    }

    pub fn run(&mut self) {
        self.inner.run()
    }

    pub fn end(&mut self) {
        self.inner.end()
    }
}

impl JSTraceable for XmlTokenizer<XmlTreeBuilder<JS<Node>, Sink>> {
    fn trace(&self, trc: *mut JSTracer) {
        struct Tracer(*mut JSTracer);
        let tracer = Tracer(trc);

        impl XmlTracer for Tracer {
            type Handle = JS<Node>;
            #[allow(unrooted_must_root)]
            fn trace_handle(&self, node: JS<Node>) {
                node.trace(self.0);
            }
        }

        let tree_builder = self.sink();
        tree_builder.trace_handles(&tracer);
        tree_builder.sink().trace(trc);
    }
}

impl<'a> TreeSink for Sink {
    type Handle = JS<Node>;

    fn parse_error(&mut self, msg: Cow<'static, str>) {
        debug!("Parse error: {}", msg);
    }

    fn get_document(&mut self) -> JS<Node> {
        JS::from_ref(self.document.upcast())
    }

    fn elem_name(&self, target: &JS<Node>) -> QName {
        let elem = target.downcast::<Element>()
            .expect("tried to get name of non-Element in XML parsing");
        QName {
            prefix: elem.prefix().as_ref().map_or(namespace_prefix!(""), |p| Prefix::from(&**p)),
            namespace_url: elem.namespace().clone(),
            local: elem.local_name().clone(),
        }
    }

    fn create_element(&mut self, name: QName, attrs: Vec<Attribute>)
            -> JS<Node> {
        let prefix = if name.prefix == namespace_prefix!("") { None } else { Some(name.prefix) };
        let name = QualName {
            ns: name.namespace_url,
            local: name.local,
        };
        let elem = Element::create(name, prefix, &*self.document,
                                   ElementCreator::ParserCreated);

        for attr in attrs {
            let name = QualName {
                ns: attr.name.namespace_url,
                local: attr.name.local,
            };
            elem.set_attribute_from_parser(name, DOMString::from(String::from(attr.value)), None);
        }

        JS::from_ref(elem.upcast())
    }

    fn create_comment(&mut self, text: StrTendril) -> JS<Node> {
        let comment = Comment::new(DOMString::from(String::from(text)), &*self.document);
        JS::from_ref(comment.upcast())
    }

    fn append(&mut self, parent: JS<Node>, child: NodeOrText<JS<Node>>) {
        let child = match child {
            NodeOrText::AppendNode(n) => Root::from_ref(&*n),
            NodeOrText::AppendText(t) => {
                let s: String = t.into();
                let text = Text::new(DOMString::from(s), &self.document);
                Root::upcast(text)
            }
        };
        assert!(parent.AppendChild(&child).is_ok());
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

    fn mark_script_already_started(&mut self, node: Self::Handle) {
        let script = node.downcast::<HTMLScriptElement>();
        if let Some(script) = script {
            script.set_already_started(true);
        }
    }

    fn complete_script(&mut self, node: Self::Handle) -> NextParserState {
        let script = node.downcast::<HTMLScriptElement>();
        if let Some(script) = script {
            return match script.prepare() {
                true => NextParserState::Continue,
                false => NextParserState::Suspend,
            };
        }
        NextParserState::Continue
    }
}
