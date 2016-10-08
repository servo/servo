/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

//! The bulk of the HTML parser integration is in `script::parse::html`.
//! This module is mostly about its interaction with DOM memory management.

use dom::bindings::cell::DOMRefCell;
use dom::bindings::codegen::Bindings::ServoHTMLParserBinding;
use dom::bindings::inheritance::Castable;
use dom::bindings::js::{JS, Root};
use dom::bindings::reflector::reflect_dom_object;
use dom::bindings::trace::JSTraceable;
use dom::document::Document;
use dom::globalscope::GlobalScope;
use dom::node::Node;
use dom::servoparser::ServoParser;
use dom::window::Window;
use html5ever::tokenizer;
use html5ever::tree_builder;
use html5ever::tree_builder::{TreeBuilder, TreeBuilderOpts};
use js::jsapi::JSTracer;
use msg::constellation_msg::PipelineId;
use parse::{Parser, ParserRef};
use profile_traits::time::{TimerMetadata, TimerMetadataFrameType, TimerMetadataReflowType, profile};
use profile_traits::time::ProfilerCategory;
use script_thread::ScriptThread;
use std::cell::Cell;
use std::default::Default;
use url::Url;

#[must_root]
#[derive(JSTraceable, HeapSizeOf)]
pub struct Sink {
    pub base_url: Option<Url>,
    pub document: JS<Document>,
}

/// FragmentContext is used only to pass this group of related values
/// into functions.
#[derive(Copy, Clone)]
pub struct FragmentContext<'a> {
    pub context_elem: &'a Node,
    pub form_elem: Option<&'a Node>,
}

pub type Tokenizer = tokenizer::Tokenizer<TreeBuilder<JS<Node>, Sink>>;

#[dom_struct]
pub struct ServoHTMLParser {
    servoparser: ServoParser,
    #[ignore_heap_size_of = "Defined in html5ever"]
    tokenizer: DOMRefCell<Tokenizer>,
    /// True if this parser should avoid passing any further data to the tokenizer.
    suspended: Cell<bool>,
    /// The pipeline associated with this parse, unavailable if this parse does not
    /// correspond to a page load.
    pipeline: Option<PipelineId>,
}

impl<'a> Parser for &'a ServoHTMLParser {
    fn parse_chunk(self, input: String) {
        self.upcast().document().set_current_parser(Some(ParserRef::HTML(self)));
        self.upcast().push_input_chunk(input);
        if !self.is_suspended() {
            self.parse_sync();
        }
    }

    fn finish(self) {
        assert!(!self.suspended.get());
        assert!(!self.upcast().has_pending_input());

        self.tokenizer.borrow_mut().end();
        debug!("finished parsing");

        self.upcast().document().set_current_parser(None);

        if let Some(pipeline) = self.pipeline {
            ScriptThread::parsing_complete(pipeline);
        }
    }
}

impl ServoHTMLParser {
    #[allow(unrooted_must_root)]
    pub fn new(base_url: Option<Url>, document: &Document, pipeline: Option<PipelineId>)
               -> Root<ServoHTMLParser> {
        let sink = Sink {
            base_url: base_url,
            document: JS::from_ref(document),
        };

        let tb = TreeBuilder::new(sink, TreeBuilderOpts {
            ignore_missing_rules: true,
            .. Default::default()
        });

        let tok = tokenizer::Tokenizer::new(tb, Default::default());

        let parser = ServoHTMLParser {
            servoparser: ServoParser::new_inherited(document, false),
            tokenizer: DOMRefCell::new(tok),
            suspended: Cell::new(false),
            pipeline: pipeline,
        };

        reflect_dom_object(box parser, document.window(), ServoHTMLParserBinding::Wrap)
    }

    #[allow(unrooted_must_root)]
    pub fn new_for_fragment(base_url: Option<Url>, document: &Document,
                            fragment_context: FragmentContext) -> Root<ServoHTMLParser> {
        let sink = Sink {
            base_url: base_url,
            document: JS::from_ref(document),
        };

        let tb_opts = TreeBuilderOpts {
            ignore_missing_rules: true,
            .. Default::default()
        };
        let tb = TreeBuilder::new_for_fragment(sink,
                                               JS::from_ref(fragment_context.context_elem),
                                               fragment_context.form_elem.map(|n| JS::from_ref(n)),
                                               tb_opts);

        let tok_opts = tokenizer::TokenizerOpts {
            initial_state: Some(tb.tokenizer_state_for_context_elem()),
            .. Default::default()
        };
        let tok = tokenizer::Tokenizer::new(tb, tok_opts);

        let parser = ServoHTMLParser {
            servoparser: ServoParser::new_inherited(document, true),
            tokenizer: DOMRefCell::new(tok),
            suspended: Cell::new(false),
            pipeline: None,
        };

        reflect_dom_object(box parser, document.window(), ServoHTMLParserBinding::Wrap)
    }

    #[inline]
    pub fn tokenizer(&self) -> &DOMRefCell<Tokenizer> {
        &self.tokenizer
    }

    pub fn set_plaintext_state(&self) {
        self.tokenizer.borrow_mut().set_plaintext_state()
    }

    pub fn end_tokenizer(&self) {
        self.tokenizer.borrow_mut().end()
    }
}

impl ServoHTMLParser {
    pub fn parse_sync(&self) {
        let metadata = TimerMetadata {
            url: self.upcast().document().url().as_str().into(),
            iframe: TimerMetadataFrameType::RootWindow,
            incremental: TimerMetadataReflowType::FirstReflow,
        };
        profile(ProfilerCategory::ScriptParseHTML,
                Some(metadata),
                self.upcast().document().window().upcast::<GlobalScope>().time_profiler_chan().clone(),
                || self.do_parse_sync())
    }

    fn do_parse_sync(&self) {
        // This parser will continue to parse while there is either pending input or
        // the parser remains unsuspended.
        loop {
            self.upcast().document().reflow_if_reflow_timer_expired();
            if let Some(chunk) = self.upcast().take_next_input_chunk() {
                self.tokenizer.borrow_mut().feed(chunk.into());
            } else {
                self.tokenizer.borrow_mut().run();
            }

            // Document parsing is blocked on an external resource.
            if self.suspended.get() {
                return;
            }

            if !self.upcast().has_pending_input() {
                break;
            }
        }

        if self.upcast().last_chunk_received() {
            self.finish();
        }
    }

    pub fn window(&self) -> &Window {
        self.upcast().document().window()
    }

    pub fn suspend(&self) {
        assert!(!self.suspended.get());
        self.suspended.set(true);
    }

    pub fn resume(&self) {
        assert!(self.suspended.get());
        self.suspended.set(false);
        self.parse_sync();
    }

    pub fn is_suspended(&self) -> bool {
        self.suspended.get()
    }
}

struct Tracer {
    trc: *mut JSTracer,
}

impl tree_builder::Tracer for Tracer {
    type Handle = JS<Node>;
    #[allow(unrooted_must_root)]
    fn trace_handle(&self, node: &JS<Node>) {
        node.trace(self.trc);
    }
}

impl JSTraceable for Tokenizer {
    fn trace(&self, trc: *mut JSTracer) {
        let tracer = Tracer {
            trc: trc,
        };
        let tracer = &tracer as &tree_builder::Tracer<Handle=JS<Node>>;

        let tree_builder = self.sink();
        tree_builder.trace_handles(tracer);
        tree_builder.sink().trace(trc);
    }
}
