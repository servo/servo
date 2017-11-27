/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

#![allow(unrooted_must_root)]

use dom::bindings::codegen::Bindings::HTMLTemplateElementBinding::HTMLTemplateElementMethods;
use dom::bindings::inheritance::{Castable, CharacterDataTypeId, NodeTypeId};
use dom::bindings::root::{Dom, DomRoot};
use dom::bindings::trace::JSTraceable;
use dom::characterdata::CharacterData;
use dom::document::Document;
use dom::documenttype::DocumentType;
use dom::element::Element;
use dom::htmlscriptelement::HTMLScriptElement;
use dom::htmltemplateelement::HTMLTemplateElement;
use dom::node::Node;
use dom::processinginstruction::ProcessingInstruction;
use dom::servoparser::{ParsingAlgorithm, Sink};
use html5ever::QualName;
use html5ever::buffer_queue::BufferQueue;
use html5ever::serialize::{AttrRef, Serialize, Serializer};
use html5ever::serialize::TraversalScope;
use html5ever::serialize::TraversalScope::IncludeNode;
use html5ever::tokenizer::{Tokenizer as HtmlTokenizer, TokenizerOpts, TokenizerResult};
use html5ever::tree_builder::{Tracer as HtmlTracer, TreeBuilder, TreeBuilderOpts};
use js::jsapi::JSTracer;
use servo_url::ServoUrl;
use std::io;

#[derive(JSTraceable, MallocSizeOf)]
#[must_root]
pub struct Tokenizer {
    #[ignore_malloc_size_of = "Defined in html5ever"]
    inner: HtmlTokenizer<TreeBuilder<Dom<Node>, Sink>>,
}

impl Tokenizer {
    pub fn new(
            document: &Document,
            url: ServoUrl,
            fragment_context: Option<super::FragmentContext>,
            parsing_algorithm: ParsingAlgorithm)
            -> Self {
        let sink = Sink {
            base_url: url,
            document: Dom::from_ref(document),
            current_line: 1,
            script: Default::default(),
            parsing_algorithm: parsing_algorithm,
        };

        let options = TreeBuilderOpts {
            ignore_missing_rules: true,
            .. Default::default()
        };

        let inner = if let Some(fc) = fragment_context {
            let tb = TreeBuilder::new_for_fragment(
                sink,
                Dom::from_ref(fc.context_elem),
                fc.form_elem.map(|n| Dom::from_ref(n)),
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

    pub fn feed(&mut self, input: &mut BufferQueue) -> Result<(), DomRoot<HTMLScriptElement>> {
        match self.inner.feed(input) {
            TokenizerResult::Done => Ok(()),
            TokenizerResult::Script(script) => Err(DomRoot::from_ref(script.downcast().unwrap())),
        }
    }

    pub fn end(&mut self) {
        self.inner.end();
    }

    pub fn url(&self) -> &ServoUrl {
        &self.inner.sink.sink.base_url
    }

    pub fn set_plaintext_state(&mut self) {
        self.inner.set_plaintext_state();
    }
}

#[allow(unsafe_code)]
unsafe impl JSTraceable for HtmlTokenizer<TreeBuilder<Dom<Node>, Sink>> {
    unsafe fn trace(&self, trc: *mut JSTracer) {
        struct Tracer(*mut JSTracer);
        let tracer = Tracer(trc);

        impl HtmlTracer for Tracer {
            type Handle = Dom<Node>;
            #[allow(unrooted_must_root)]
            fn trace_handle(&self, node: &Dom<Node>) {
                unsafe { node.trace(self.0); }
            }
        }

        let tree_builder = &self.sink;
        tree_builder.trace_handles(&tracer);
        tree_builder.sink.trace(trc);
    }
}

fn start_element<S: Serializer>(node: &Element, serializer: &mut S) -> io::Result<()> {
    let name = QualName::new(None, node.namespace().clone(),
                             node.local_name().clone());
    let attrs = node.attrs().iter().map(|attr| {
        let qname = QualName::new(None, attr.namespace().clone(),
                                  attr.local_name().clone());
        let value = attr.value().clone();
        (qname, value)
    }).collect::<Vec<_>>();
    let attr_refs = attrs.iter().map(|&(ref qname, ref value)| {
        let ar: AttrRef = (&qname, &**value);
        ar
    });
    serializer.start_elem(name, attr_refs)?;
    Ok(())
}

fn end_element<S: Serializer>(node: &Element, serializer: &mut S) -> io::Result<()> {
    let name = QualName::new(None, node.namespace().clone(),
                             node.local_name().clone());
    serializer.end_elem(name)
}


enum SerializationCommand {
    OpenElement(DomRoot<Element>),
    CloseElement(DomRoot<Element>),
    SerializeNonelement(DomRoot<Node>),
}

struct SerializationIterator {
    stack: Vec<SerializationCommand>,
}

fn rev_children_iter(n: &Node) -> impl Iterator<Item=DomRoot<Node>>{
    match n.downcast::<HTMLTemplateElement>() {
        Some(t) => t.Content().upcast::<Node>().rev_children(),
        None => n.rev_children(),
    }
}

impl SerializationIterator {
    fn new(node: &Node, skip_first: bool) -> SerializationIterator {
        let mut ret = SerializationIterator {
            stack: vec![],
        };
        if skip_first {
            for c in rev_children_iter(node) {
                ret.push_node(&*c);
            }
        } else {
            ret.push_node(node);
        }
        ret
    }

    fn push_node(&mut self, n: &Node) {
        match n.downcast::<Element>() {
            Some(e) => self.stack.push(SerializationCommand::OpenElement(DomRoot::from_ref(e))),
            None => self.stack.push(SerializationCommand::SerializeNonelement(DomRoot::from_ref(n))),
        }
    }
}

impl Iterator for SerializationIterator {
    type Item = SerializationCommand;

    fn next(&mut self) -> Option<SerializationCommand> {
        let res = self.stack.pop();

        if let Some(SerializationCommand::OpenElement(ref e)) = res {
            self.stack.push(SerializationCommand::CloseElement(e.clone()));
            for c in rev_children_iter(&*e.upcast::<Node>()) {
                self.push_node(&c);
            }
        }

        res
    }
}

impl<'a> Serialize for &'a Node {
    fn serialize<S: Serializer>(&self, serializer: &mut S,
                                traversal_scope: TraversalScope) -> io::Result<()> {
        let node = *self;


        let iter = SerializationIterator::new(node, traversal_scope != IncludeNode);

        for cmd in iter {
            match cmd {
                SerializationCommand::OpenElement(n) => {
                    start_element(&n, serializer)?;
                }

                SerializationCommand::CloseElement(n) => {
                    end_element(&&n, serializer)?;
                }

                SerializationCommand::SerializeNonelement(n) => {
                    match n.type_id() {
                        NodeTypeId::DocumentType => {
                            let doctype = n.downcast::<DocumentType>().unwrap();
                            serializer.write_doctype(&doctype.name())?;
                        },

                        NodeTypeId::CharacterData(CharacterDataTypeId::Text) => {
                            let cdata = n.downcast::<CharacterData>().unwrap();
                            serializer.write_text(&cdata.data())?;
                        },

                        NodeTypeId::CharacterData(CharacterDataTypeId::Comment) => {
                            let cdata = n.downcast::<CharacterData>().unwrap();
                            serializer.write_comment(&cdata.data())?;
                        },

                        NodeTypeId::CharacterData(CharacterDataTypeId::ProcessingInstruction) => {
                            let pi = n.downcast::<ProcessingInstruction>().unwrap();
                            let data = pi.upcast::<CharacterData>().data();
                            serializer.write_processing_instruction(&pi.target(), &data)?;
                        },

                        NodeTypeId::DocumentFragment => {}

                        NodeTypeId::Document(_) => panic!("Can't serialize Document node itself"),
                        NodeTypeId::Element(_) => panic!("Element shouldn't appear here"),
                    }
                }
            }
        }

        Ok(())
    }
}
