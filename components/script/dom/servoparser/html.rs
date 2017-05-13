/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

#![allow(unrooted_must_root)]

use dom::bindings::codegen::Bindings::HTMLTemplateElementBinding::HTMLTemplateElementMethods;
use dom::bindings::inheritance::{Castable, CharacterDataTypeId, NodeTypeId};
use dom::bindings::js::{JS, Root, RootedReference};
use dom::bindings::trace::JSTraceable;
use dom::characterdata::CharacterData;
use dom::document::Document;
use dom::documenttype::DocumentType;
use dom::element::Element;
use dom::htmlscriptelement::HTMLScriptElement;
use dom::htmltemplateelement::HTMLTemplateElement;
use dom::node::Node;
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

impl<'a> Serialize for &'a Node {
    fn serialize<S: Serializer>(&self, serializer: &mut S,
                                traversal_scope: TraversalScope) -> io::Result<()> {
        let mut roots = self.traverse_preorder();
        let mut stack = Vec::new();
        if traversal_scope == ChildrenOnly {
            roots.next().unwrap();
        }
        for root in roots {
            let node = root.r();
            match node.type_id() {
                NodeTypeId::Element(..) => {
                    let elem = node.downcast::<Element>().unwrap();
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
                    try!(serializer.start_elem(name.clone(), attr_refs));
                    stack.push((name.clone(), node.children_count()));
                },
                NodeTypeId::DocumentType => {
                    let doctype = node.downcast::<DocumentType>().unwrap();
                    try!(serializer.write_doctype(&doctype.name()));
                    if let Some(head) = stack.last_mut() {
                        head.1 -= 1;
                    }
                },
                NodeTypeId::CharacterData(CharacterDataTypeId::Text) => {
                    let cdata = node.downcast::<CharacterData>().unwrap();
                    try!(serializer.write_text(&cdata.data()));
                    if let Some(head) = stack.last_mut() {
                        head.1 -= 1;
                    }
                },
                NodeTypeId::CharacterData(CharacterDataTypeId::Comment) => {
                    let cdata = node.downcast::<CharacterData>().unwrap();
                    try!(serializer.write_comment(&cdata.data()));
                    if let Some(head) = stack.last_mut() {
                        head.1 -= 1;
                    }
                },
                NodeTypeId::CharacterData(CharacterDataTypeId::ProcessingInstruction) => {
                    let pi = node.downcast::<ProcessingInstruction>().unwrap();
                    let data = pi.upcast::<CharacterData>().data();
                    try!(serializer.write_processing_instruction(&pi.target(), &data));
                    if let Some(head) = stack.last_mut() {
                        head.1 -= 1;
                    }
                },
                NodeTypeId::DocumentFragment => {},
                NodeTypeId::Document(_) => panic!("Can't serialize Document node itself"),
            };
            while {
                let mut n_children = 1;
                if let Some(head) = stack.last() {
                    n_children = head.1;
                }
                n_children == 0
            } {
                try!(serializer.end_elem(stack.pop().unwrap().0));
                if let Some(mut head) = stack.last_mut() {
                    head.1 -= 1;
                }
            }
        }
        if stack.is_empty() {
            Ok(())
        } else {
            panic!("Stack is not empty!!!!")
        }
    }
}
