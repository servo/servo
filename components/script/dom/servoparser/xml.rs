/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

#![allow(crown::unrooted_must_root)]

use html5ever::tokenizer::TokenizerResult;
use js::jsapi::JSTracer;
use servo_url::ServoUrl;
use xml5ever::buffer_queue::BufferQueue;
use xml5ever::tokenizer::XmlTokenizer;
use xml5ever::tree_builder::{Tracer as XmlTracer, XmlTreeBuilder};

use crate::dom::bindings::root::{Dom, DomRoot};
use crate::dom::bindings::trace::{CustomTraceable, JSTraceable};
use crate::dom::document::Document;
use crate::dom::htmlscriptelement::HTMLScriptElement;
use crate::dom::node::Node;
use crate::dom::servoparser::{ParsingAlgorithm, Sink};

#[derive(JSTraceable, MallocSizeOf)]
#[crown::unrooted_must_root_lint::must_root]
pub struct Tokenizer {
    #[ignore_malloc_size_of = "Defined in xml5ever"]
    inner: XmlTokenizer<XmlTreeBuilder<Dom<Node>, Sink>>,
}

impl Tokenizer {
    pub fn new(document: &Document, url: ServoUrl) -> Self {
        let sink = Sink {
            base_url: url,
            document: Dom::from_ref(document),
            current_line: 1,
            script: Default::default(),
            parsing_algorithm: ParsingAlgorithm::Normal,
        };

        let tb = XmlTreeBuilder::new(sink, Default::default());
        let tok = XmlTokenizer::new(tb, Default::default());

        Tokenizer { inner: tok }
    }

    pub fn feed(&mut self, input: &mut BufferQueue) -> TokenizerResult<DomRoot<HTMLScriptElement>> {
        self.inner.run(input);
        match self.inner.sink.sink.script.take() {
            Some(script) => TokenizerResult::Script(script),
            None => TokenizerResult::Done,
        }
    }

    pub fn end(&mut self) {
        self.inner.end()
    }

    pub fn url(&self) -> &ServoUrl {
        &self.inner.sink.sink.base_url
    }
}

#[allow(unsafe_code)]
unsafe impl CustomTraceable for XmlTokenizer<XmlTreeBuilder<Dom<Node>, Sink>> {
    unsafe fn trace(&self, trc: *mut JSTracer) {
        struct Tracer(*mut JSTracer);
        let tracer = Tracer(trc);

        impl XmlTracer for Tracer {
            type Handle = Dom<Node>;
            #[allow(crown::unrooted_must_root)]
            fn trace_handle(&self, node: &Dom<Node>) {
                unsafe {
                    node.trace(self.0);
                }
            }
        }

        let tree_builder = &self.sink;
        tree_builder.trace_handles(&tracer);
        tree_builder.sink.trace(trc);
    }
}
