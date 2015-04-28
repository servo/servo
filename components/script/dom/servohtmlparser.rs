/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

//! The bulk of the HTML parser integration is in `script::parse::html`.
//! This module is mostly about its interaction with DOM memory management.

use dom::bindings::cell::DOMRefCell;
use dom::bindings::codegen::Bindings::ServoHTMLParserBinding;
use dom::bindings::global::GlobalRef;
use dom::bindings::trace::JSTraceable;
use dom::bindings::js::{JS, JSRef, Rootable, Temporary};
use dom::bindings::utils::{Reflectable, Reflector, reflect_dom_object};
use dom::document::{Document, DocumentHelpers};
use dom::node::Node;
use parse::Parser;

use util::task_state;

use std::default::Default;
use url::Url;
use js::jsapi::JSTracer;
use html5ever::tokenizer;
use html5ever::tree_builder;
use html5ever::tree_builder::{TreeBuilder, TreeBuilderOpts};

#[must_root]
#[jstraceable]
pub struct Sink {
    pub base_url: Option<Url>,
    pub document: JS<Document>,
}

/// FragmentContext is used only to pass this group of related values
/// into functions.
#[derive(Copy, Clone)]
pub struct FragmentContext<'a> {
    pub context_elem: JSRef<'a, Node>,
    pub form_elem: Option<JSRef<'a, Node>>,
}

pub type Tokenizer = tokenizer::Tokenizer<TreeBuilder<JS<Node>, Sink>>;

// NB: JSTraceable is *not* auto-derived.
// You must edit the impl below if you add fields!
#[must_root]
#[privatize]
pub struct ServoHTMLParser {
    reflector_: Reflector,
    tokenizer: DOMRefCell<Tokenizer>,
}

impl Parser for ServoHTMLParser{
    fn parse_chunk(&self, input: String) {
        self.tokenizer().borrow_mut().feed(input);
    }
    fn finish(&self){
        self.tokenizer().borrow_mut().end();
    }
}

impl ServoHTMLParser {
    #[allow(unrooted_must_root)]
    pub fn new(base_url: Option<Url>, document: JSRef<Document>) -> Temporary<ServoHTMLParser> {
        let window = document.window().root();
        let sink = Sink {
            base_url: base_url,
            document: JS::from_rooted(document),
        };

        let tb = TreeBuilder::new(sink, TreeBuilderOpts {
            ignore_missing_rules: true,
            .. Default::default()
        });

        let tok = tokenizer::Tokenizer::new(tb, Default::default());

        let parser = ServoHTMLParser {
            reflector_: Reflector::new(),
            tokenizer: DOMRefCell::new(tok),
        };

        reflect_dom_object(box parser, GlobalRef::Window(window.r()),
                           ServoHTMLParserBinding::Wrap)
    }

    #[allow(unrooted_must_root)]
    pub fn new_for_fragment(base_url: Option<Url>, document: JSRef<Document>,
                            fragment_context: FragmentContext) -> Temporary<ServoHTMLParser> {
        let window = document.window().root();
        let sink = Sink {
            base_url: base_url,
            document: JS::from_rooted(document),
        };

        let tb_opts = TreeBuilderOpts {
            ignore_missing_rules: true,
            .. Default::default()
        };
        let tb = TreeBuilder::new_for_fragment(sink,
                                               JS::from_rooted(fragment_context.context_elem),
                                               fragment_context.form_elem.map(|n| JS::from_rooted(n)),
                                               tb_opts);

        let tok_opts = tokenizer::TokenizerOpts {
            initial_state: Some(tb.tokenizer_state_for_context_elem()),
            .. Default::default()
        };
        let tok = tokenizer::Tokenizer::new(tb, tok_opts);

        let parser = ServoHTMLParser {
            reflector_: Reflector::new(),
            tokenizer: DOMRefCell::new(tok),
        };

        reflect_dom_object(box parser, GlobalRef::Window(window.r()),
                           ServoHTMLParserBinding::Wrap)
    }

    #[inline]
    pub fn tokenizer<'a>(&'a self) -> &'a DOMRefCell<Tokenizer> {
        &self.tokenizer
    }
}

impl Reflectable for ServoHTMLParser {
    fn reflector<'a>(&'a self) -> &'a Reflector {
        &self.reflector_
    }
}

struct Tracer {
    trc: *mut JSTracer,
}

impl tree_builder::Tracer for Tracer {
    type Handle = JS<Node>;
    #[allow(unrooted_must_root)]
    fn trace_handle(&self, node: JS<Node>) {
        node.trace(self.trc);
    }
}

impl JSTraceable for ServoHTMLParser {
    #[allow(unsafe_code)]
    fn trace(&self, trc: *mut JSTracer) {
        self.reflector_.trace(trc);

        let tracer = Tracer {
            trc: trc,
        };
        let tracer = &tracer as &tree_builder::Tracer<Handle=JS<Node>>;

        unsafe {
            // Assertion: If the parser is mutably borrowed, we're in the
            // parsing code paths.
            debug_assert!(task_state::get().contains(task_state::IN_HTML_PARSER)
                || !self.tokenizer.is_mutably_borrowed());

            let tokenizer = self.tokenizer.borrow_for_gc_trace();
            let tree_builder = tokenizer.sink();
            tree_builder.trace_handles(tracer);
            tree_builder.sink().trace(trc);
        }
    }
}
