/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

#![allow(unrooted_must_root)]

use js::jsapi::JSTracer;
use script;
use script::dom::bindings::root::{Dom, DomRoot};
use script::dom::bindings::trace::JSTraceable;
use script::dom::document::Document;
use script::dom::htmlscriptelement::HTMLScriptElement;
use script::dom::node::Node;
use script::dom::servoparser::{ParsingAlgorithm, Sink, TokenizerTrait};
use servo_url::ServoUrl;
use std::marker::PhantomData;
use xml5ever::buffer_queue::BufferQueue;
use xml5ever::tokenizer::XmlTokenizer;
use xml5ever::tree_builder::{Tracer as XmlTracer, XmlTreeBuilder};

#[derive(JSTraceable, MallocSizeOf)]
#[base = "script"]
#[must_root]
pub struct Tokenizer {
    #[ignore_malloc_size_of = "Defined in xml5ever"]
    inner: XmlTokenizer<XmlTreeBuilder<Dom<Node<super::TypeHolder>>, Sink<super::TypeHolder>>>,
}

impl TokenizerTrait<super::TypeHolder> for Tokenizer {
    fn new(
            document: &Document<super::TypeHolder>,
            url: ServoUrl,
            _fragment_context: Option<super::FragmentContext<super::TypeHolder>>,
            _parsing_algorithm: ParsingAlgorithm) -> Self {
        let sink = Sink {
            base_url: url,
            document: Dom::from_ref(document),
            current_line: 1,
            script: Default::default(),
            parsing_algorithm: ParsingAlgorithm::Normal,
        };

        let tb = XmlTreeBuilder::new(sink, Default::default());
        let tok = XmlTokenizer::new(tb, Default::default());

        Tokenizer {
            inner: tok,
        }
    }

    fn feed(&mut self, input: &mut BufferQueue) -> Result<(), DomRoot<HTMLScriptElement<super::TypeHolder>>> {
        if !input.is_empty() {
            while let Some(chunk) = input.pop_front() {
                self.inner.feed(chunk);
                if let Some(script) = self.inner.sink.sink.script.take() {
                    return Err(script);
                }
            }
        } else {
            self.inner.run();
            if let Some(script) = self.inner.sink.sink.script.take() {
                return Err(script);
            }
        }
        Ok(())
    }

    fn end(&mut self) {
        self.inner.end()
    }

    fn url(&self) -> &ServoUrl {
        &self.inner.sink.sink.base_url
    }

    fn set_plaintext_state(&mut self) { }
}
