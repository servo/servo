/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

#![allow(unrooted_must_root)]

use dom::bindings::codegen::Bindings::HTMLTemplateElementBinding::HTMLTemplateElementMethods;
use dom::bindings::inheritance::{Castable, CharacterDataTypeId, NodeTypeId};
use dom::bindings::js::{JS, Root};
use dom::bindings::trace::JSTraceable;
use dom::characterdata::CharacterData;
use dom::document::Document;
use dom::documenttype::DocumentType;
use dom::element::Element;
use dom::htmlscriptelement::HTMLScriptElement;
use dom::htmltemplateelement::HTMLTemplateElement;
use dom::node::{Node, TreeIterator};
use dom::processinginstruction::ProcessingInstruction;
use dom::servoparser::Sink;
use html5ever::QualName;
use html5ever::buffer_queue::BufferQueue;
use html5ever::serialize::{AttrRef, Serialize, Serializer};
use html5ever::serialize::TraversalScope;
use html5ever::serialize::TraversalScope::{ChildrenOnly, IncludeNode};
use html5ever::tokenizer::{Tokenizer as HtmlTokenizer, TokenizerOpts, TokenizerResult};
use html5ever::tree_builder::{Tracer as HtmlTracer, TreeBuilder, TreeBuilderOpts};
use js::jsapi::JSTracer;
use servo_url::ServoUrl;
use std::io;

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
            script: Default::default(),
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
        &self.inner.sink.sink.base_url
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

        let tree_builder = &self.sink;
        tree_builder.trace_handles(&tracer);
        tree_builder.sink.trace(trc);
    }
}

fn start_element<S: Serializer>(node: &Node, serializer: &mut S) -> io::Result<()> {
    let elem = node.downcast::<Element>().expect("Node should be an Element");
    let name = QualName::new(None, elem.namespace().clone(),
                             elem.local_name().clone());
    let attrs = elem.attrs().iter().map(|attr| {
        let qname = QualName::new(None, attr.namespace().clone(),
                                  attr.local_name().clone());
        let value = attr.value().clone();
        (qname, value)
    }).collect::<Vec<_>>();
    let attr_refs = attrs.iter().map(|&(ref qname, ref value)| {
        let ar: AttrRef = (&qname, &**value);
        ar
    });
    serializer.start_elem(name.clone(), attr_refs)?;
    Ok(())
}

fn end_element<S: Serializer>(node: &Node, serializer: &mut S) -> io::Result<()> {
    let elem = node.downcast::<Element>().expect("node should be an Element");
    let name = QualName::new(None, elem.namespace().clone(),
                             elem.local_name().clone());
    serializer.end_elem(name.clone())
}

// This is the same as TreeIterator from traverse_preorder, but knows how to go into HTMLTemplateElement's contents.
struct SerializationIterator {
    stack: Vec<TreeIterator>,
}

impl SerializationIterator {
    fn new(node: &Node) -> SerializationIterator {
        SerializationIterator {
            stack: vec![node.traverse_preorder()],
        }
    }
}

impl Iterator for SerializationIterator {
    type Item = Root<Node>;

    fn next(&mut self) -> Option<Root<Node>> {
        if self.stack.len() == 0 { return None; }

        if let Some(n) = self.stack.last_mut().unwrap().next() {
            // https://github.com/w3c/DOM-Parsing/issues/1
            if let Some(tpl) = n.downcast::<HTMLTemplateElement>() {
                // We need to visit the contents of the template from left to right.
                // Since we always work from the top of the stack down, push the iterators in reverse order.
                for c in tpl.Content().upcast::<Node>().rev_children() {
                    self.stack.push(c.traverse_preorder());
                }
            }
            return Some(n);
        } else {
            self.stack.pop();
        }

        None
    }
}

impl<'a> Serialize for &'a Node {
    fn serialize<S: Serializer>(&self, serializer: &mut S,
                                traversal_scope: TraversalScope) -> io::Result<()> {
        let node = *self;

        // As we encounter elements, we push them to this stack with a count of their children.
        // Every time through the following loop, we decrement the child count of the topmost element.
        // When it gets to 0, it's time to close the node.
        let mut open_stack: Vec<(Root<Node>, u32)> = vec![];
        let skip_count = if traversal_scope == ChildrenOnly {
            1
        } else {
            0
        };
        let iter = SerializationIterator::new(node).skip(skip_count);

        for n in iter {
            match n.type_id() {
                NodeTypeId::Element(..) => {
                    start_element(&n, serializer)?;
                    let children_count = if let Some(tpl) = n.downcast::<HTMLTemplateElement>() {
                        tpl.Content().upcast::<Node>().children_count()
                    } else {
                        n.children_count()
                    };

                    open_stack.push((n, children_count));
                },

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
            }

            if let Some(top) = open_stack.pop() {
                if top.1 == 0 {
                    end_element(&top.0, serializer)?;
                } else {
                    open_stack.push((top.0, top.1 - 1));
                }
            }
        }

        Ok(())
    }
}
