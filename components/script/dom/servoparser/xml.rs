/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

#![cfg_attr(crown, allow(crown::unrooted_must_root))]

use std::cell::Cell;

use markup5ever::TokenizerResult;
use script_bindings::trace::CustomTraceable;
use servo_url::ServoUrl;
use xml5ever::buffer_queue::BufferQueue;
use xml5ever::tokenizer::XmlTokenizer;
use xml5ever::tree_builder::XmlTreeBuilder;

use crate::dom::bindings::inheritance::Castable;
use crate::dom::bindings::root::{Dom, DomRoot};
use crate::dom::document::Document;
use crate::dom::htmlscriptelement::HTMLScriptElement;
use crate::dom::node::Node;
use crate::dom::servoparser::{ParsingAlgorithm, Sink};

#[derive(JSTraceable, MallocSizeOf)]
#[cfg_attr(crown, crown::unrooted_must_root_lint::must_root)]
pub(crate) struct Tokenizer {
    #[ignore_malloc_size_of = "Defined in xml5ever"]
    inner: XmlTokenizer<XmlTreeBuilder<Dom<Node>, Sink>>,
}

impl Tokenizer {
    pub(crate) fn new(document: &Document, url: ServoUrl) -> Self {
        let sink = Sink {
            base_url: url,
            document: Dom::from_ref(document),
            current_line: Cell::new(1),
            script: Default::default(),
            parsing_algorithm: ParsingAlgorithm::Normal,
        };

        let tb = XmlTreeBuilder::new(sink, Default::default());
        let tok = XmlTokenizer::new(tb, Default::default());

        Tokenizer { inner: tok }
    }

    pub(crate) fn feed(&self, input: &BufferQueue) -> TokenizerResult<DomRoot<HTMLScriptElement>> {
        loop {
            match self.inner.run(input) {
                TokenizerResult::Done => return TokenizerResult::Done,
                TokenizerResult::Script(handle) => {
                    // Apparently the parser can sometimes create <script> elements without a namespace, resulting
                    // in them not being HTMLScriptElements.
                    if let Some(script) = handle.downcast::<HTMLScriptElement>() {
                        return TokenizerResult::Script(DomRoot::from_ref(script));
                    }
                },
            }
        }
    }

    pub(crate) fn end(&self) {
        self.inner.end()
    }

    pub(crate) fn url(&self) -> &ServoUrl {
        &self.inner.sink.sink.base_url
    }
}
