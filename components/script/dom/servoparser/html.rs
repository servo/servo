/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

#![allow(unrooted_must_root)]

use dom::bindings::codegen::Bindings::HTMLTemplateElementBinding::HTMLTemplateElementMethods;
use dom::bindings::codegen::Bindings::NodeBinding::NodeMethods;
use dom::bindings::inheritance::{Castable, CharacterDataTypeId, NodeTypeId};
use dom::bindings::js::{JS, Root};
use dom::bindings::str::DOMString;
use dom::bindings::trace::JSTraceable;
use dom::characterdata::CharacterData;
use dom::comment::Comment;
use dom::document::Document;
use dom::documenttype::DocumentType;
use dom::element::{Element, ElementCreator};
use dom::htmlscriptelement::HTMLScriptElement;
use dom::htmltemplateelement::HTMLTemplateElement;
use dom::node::Node;
use dom::processinginstruction::ProcessingInstruction;
use dom::virtualmethods::vtable_for;
use html5ever::Attribute;
use html5ever::serialize::{AttrRef, Serializable, Serializer};
use html5ever::serialize::TraversalScope;
use html5ever::serialize::TraversalScope::{ChildrenOnly, IncludeNode};
use html5ever::tendril::StrTendril;
use html5ever::tokenizer::{Tokenizer as HtmlTokenizer, TokenizerOpts, TokenizerResult};
use html5ever::tokenizer::buffer_queue::BufferQueue;
use html5ever::tree_builder::{NodeOrText, QuirksMode};
use html5ever::tree_builder::{Tracer as HtmlTracer, TreeBuilder, TreeBuilderOpts, TreeSink};
use html5ever_atoms::QualName;
use js::jsapi::JSTracer;
use servo_url::ServoUrl;
use std::borrow::Cow;
use std::io::{self, Write};
use style::context::QuirksMode as ServoQuirksMode;

#[derive(HeapSizeOf, JSTraceable)]
#[must_root]
pub struct Tokenizer {
    #[ignore_heap_size_of = "Defined in html5ever"]
    inner: HtmlTokenizer<TreeBuilder<JS<Node>, Sink>>,
}

impl Tokenizer {
    pub fn new(
            document: &Document,
            url: ServoUrl,
            fragment_context: Option<super::FragmentContext>)
            -> Self {
        let sink = Sink {
            base_url: url,
            document: JS::from_ref(document),
            current_line: 1,
        };

        let options = TreeBuilderOpts {
            ignore_missing_rules: true,
            .. Default::default()
        };

        let inner = if let Some(fc) = fragment_context {
            let tb = TreeBuilder::new_for_fragment(
                sink,
                JS::from_ref(fc.context_elem),
                fc.form_elem.map(|n| JS::from_ref(n)),
                options);

            let tok_options = TokenizerOpts {
                initial_state: Some(tb.tokenizer_state_for_context_elem()),
                .. Default::default()
            };

            HtmlTokenizer::new(tb, tok_options)
        } else {
            HtmlTokenizer::new(TreeBuilder::new(sink, options), Default::default())
        };

        Tokenizer {
            inner: inner,
        }
    }

    pub fn feed(&mut self, input: &mut BufferQueue) -> Result<(), Root<HTMLScriptElement>> {
        match self.inner.feed(input) {
            TokenizerResult::Done => Ok(()),
            TokenizerResult::Script(script) => Err(Root::from_ref(script.downcast().unwrap())),
        }
    }

    pub fn end(&mut self) {
        self.inner.end();
    }

    pub fn url(&self) -> &ServoUrl {
        &self.inner.sink().sink().base_url
    }

    pub fn set_plaintext_state(&mut self) {
        self.inner.set_plaintext_state();
    }
}

#[allow(unsafe_code)]
unsafe impl JSTraceable for HtmlTokenizer<TreeBuilder<JS<Node>, Sink>> {
    unsafe fn trace(&self, trc: *mut JSTracer) {
        struct Tracer(*mut JSTracer);
        let tracer = Tracer(trc);

        impl HtmlTracer for Tracer {
            type Handle = JS<Node>;
            #[allow(unrooted_must_root)]
            fn trace_handle(&self, node: &JS<Node>) {
                unsafe { node.trace(self.0); }
            }
        }

        let tree_builder = self.sink();
        tree_builder.trace_handles(&tracer);
        tree_builder.sink().trace(trc);
    }
}

#[derive(JSTraceable, HeapSizeOf)]
#[must_root]
struct Sink {
    base_url: ServoUrl,
    document: JS<Document>,
    current_line: u64,
}

impl TreeSink for Sink {
    type Output = Self;
    fn finish(self) -> Self { self }

    type Handle = JS<Node>;

    fn get_document(&mut self) -> JS<Node> {
        JS::from_ref(self.document.upcast())
    }

    fn get_template_contents(&mut self, target: JS<Node>) -> JS<Node> {
        let template = target.downcast::<HTMLTemplateElement>()
            .expect("tried to get template contents of non-HTMLTemplateElement in HTML parsing");
        JS::from_ref(template.Content().upcast())
    }

    fn same_node(&self, x: JS<Node>, y: JS<Node>) -> bool {
        x == y
    }

    fn elem_name(&self, target: JS<Node>) -> QualName {
        let elem = target.downcast::<Element>()
            .expect("tried to get name of non-Element in HTML parsing");
        QualName {
            ns: elem.namespace().clone(),
            local: elem.local_name().clone(),
        }
    }

    fn create_element(&mut self, name: QualName, attrs: Vec<Attribute>)
            -> JS<Node> {
        let elem = Element::create(name, None, &*self.document,
                                   ElementCreator::ParserCreated(self.current_line));

        for attr in attrs {
            elem.set_attribute_from_parser(attr.name, DOMString::from(String::from(attr.value)), None);
        }

        JS::from_ref(elem.upcast())
    }

    fn create_comment(&mut self, text: StrTendril) -> JS<Node> {
        let comment = Comment::new(DOMString::from(String::from(text)), &*self.document);
        JS::from_ref(comment.upcast())
    }

    fn append_before_sibling(&mut self,
            sibling: JS<Node>,
            new_node: NodeOrText<JS<Node>>) -> Result<(), NodeOrText<JS<Node>>> {
        // If there is no parent, return the node to the parser.
        let parent = match sibling.GetParentNode() {
            Some(p) => p,
            None => return Err(new_node),
        };

        super::insert(&parent, Some(&*sibling), new_node);
        Ok(())
    }

    fn parse_error(&mut self, msg: Cow<'static, str>) {
        debug!("Parse error: {}", msg);
    }

    fn set_quirks_mode(&mut self, mode: QuirksMode) {
        let mode = match mode {
            QuirksMode::Quirks => ServoQuirksMode::Quirks,
            QuirksMode::LimitedQuirks => ServoQuirksMode::LimitedQuirks,
            QuirksMode::NoQuirks => ServoQuirksMode::NoQuirks,
        };
        self.document.set_quirks_mode(mode);
    }

    fn append(&mut self, parent: JS<Node>, child: NodeOrText<JS<Node>>) {
        super::insert(&parent, None, child);
    }

    fn append_doctype_to_document(&mut self, name: StrTendril, public_id: StrTendril,
                                  system_id: StrTendril) {
        let doc = &*self.document;
        let doctype = DocumentType::new(
            DOMString::from(String::from(name)), Some(DOMString::from(String::from(public_id))),
            Some(DOMString::from(String::from(system_id))), doc);
        doc.upcast::<Node>().AppendChild(doctype.upcast()).expect("Appending failed");
    }

    fn add_attrs_if_missing(&mut self, target: JS<Node>, attrs: Vec<Attribute>) {
        let elem = target.downcast::<Element>()
            .expect("tried to set attrs on non-Element in HTML parsing");
        for attr in attrs {
            elem.set_attribute_from_parser(attr.name, DOMString::from(String::from(attr.value)), None);
        }
    }

    fn remove_from_parent(&mut self, target: JS<Node>) {
        if let Some(ref parent) = target.GetParentNode() {
            parent.RemoveChild(&*target).unwrap();
        }
    }

    fn mark_script_already_started(&mut self, node: JS<Node>) {
        let script = node.downcast::<HTMLScriptElement>();
        script.map(|script| script.set_already_started(true));
    }

    fn reparent_children(&mut self, node: JS<Node>, new_parent: JS<Node>) {
        while let Some(ref child) = node.GetFirstChild() {
            new_parent.AppendChild(&child).unwrap();
        }
    }

    fn set_current_line(&mut self, line_number: u64) {
        self.current_line = line_number;
    }

    fn pop(&mut self, node: JS<Node>) {
        let node = Root::from_ref(&*node);
        vtable_for(&node).pop();
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
                    let attrs = elem.attrs().iter().map(|attr| {
                        let qname = QualName::new(attr.namespace().clone(),
                                                  attr.local_name().clone());
                        let value = attr.value().clone();
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
                    try!((&*handle).serialize(serializer, IncludeNode));
                }

                if traversal_scope == IncludeNode {
                    try!(serializer.end_elem(name.clone()));
                }
                Ok(())
            },

            (ChildrenOnly, NodeTypeId::Document(_)) => {
                for handle in node.children() {
                    try!((&*handle).serialize(serializer, IncludeNode));
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

            (IncludeNode, NodeTypeId::Document(_)) => panic!("Can't serialize Document node itself"),
        }
    }
}
