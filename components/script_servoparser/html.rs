/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

#![allow(unrooted_must_root)]

use script::dom::bindings::codegen::Bindings::HTMLTemplateElementBinding::HTMLTemplateElementMethods;
use script::dom::bindings::inheritance::{Castable, CharacterDataTypeId, NodeTypeId};
use script::dom::bindings::root::{Dom, DomRoot};
use script::dom::bindings::trace::JSTraceable;
use script::dom::characterdata::CharacterData;
use script::dom::document::Document;
use script::dom::documenttype::DocumentType;
use script::dom::element::Element;
use script::dom::htmlscriptelement::HTMLScriptElement;
use script::dom::htmltemplateelement::HTMLTemplateElement;
use script::dom::node::Node;
use script::dom::processinginstruction::ProcessingInstruction;
use script::dom::servoparser::{ParsingAlgorithm, Sink, TokenizerTrait};
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
use std::marker::PhantomData;
use script;

#[derive(JSTraceable, MallocSizeOf)]
#[base = "script"]
#[must_root]
pub struct Tokenizer{
    #[ignore_malloc_size_of = "Defined in html5ever"]
    inner: HtmlTokenizer<TreeBuilder<Dom<Node<super::TypeHolder>>, Sink<super::TypeHolder>>>,
}

impl TokenizerTrait<super::TypeHolder> for Tokenizer {
    fn new(
            document: &Document<super::TypeHolder>,
            url: ServoUrl,
            fragment_context: Option<super::FragmentContext<super::TypeHolder>>,
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

    fn feed(&mut self, input: &mut BufferQueue) -> Result<(), DomRoot<HTMLScriptElement<super::TypeHolder>>> {
        match self.inner.feed(input) {
            TokenizerResult::Done => Ok(()),
            TokenizerResult::Script(script) => Err(DomRoot::from_ref(script.downcast().unwrap())),
        }
    }

    fn end(&mut self) {
        self.inner.end();
    }

    fn url(&self) -> &ServoUrl {
        &self.inner.sink.sink.base_url
    }

    fn set_plaintext_state(&mut self) {
        self.inner.set_plaintext_state();
    }
}