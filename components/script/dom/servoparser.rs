/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use dom::bindings::cell::DOMRefCell;
use dom::bindings::codegen::Bindings::ServoParserBinding;
use dom::bindings::inheritance::Castable;
use dom::bindings::js::{JS, Root};
use dom::bindings::reflector::{Reflector, reflect_dom_object};
use dom::bindings::trace::JSTraceable;
use dom::document::Document;
use dom::globalscope::GlobalScope;
use dom::node::Node;
use dom::window::Window;
use html5ever::tokenizer::Tokenizer as HtmlTokenizer;
use html5ever::tree_builder::Tracer as HtmlTracer;
use html5ever::tree_builder::TreeBuilder as HtmlTreeBuilder;
use js::jsapi::JSTracer;
use msg::constellation_msg::PipelineId;
use parse::Sink;
use profile_traits::time::{TimerMetadata, TimerMetadataFrameType};
use profile_traits::time::{TimerMetadataReflowType, ProfilerCategory, profile};
use script_thread::ScriptThread;
use std::cell::Cell;
use xml5ever::tokenizer::XmlTokenizer;
use xml5ever::tree_builder::{Tracer as XmlTracer, XmlTreeBuilder};

#[dom_struct]
pub struct ServoParser {
    reflector: Reflector,
    /// The document associated with this parser.
    document: JS<Document>,
    /// The pipeline associated with this parse, unavailable if this parse
    /// does not correspond to a page load.
    pipeline: Option<PipelineId>,
    /// Input chunks received but not yet passed to the parser.
    pending_input: DOMRefCell<Vec<String>>,
    /// The tokenizer of this parser.
    tokenizer: DOMRefCell<Tokenizer>,
    /// Whether to expect any further input from the associated network request.
    last_chunk_received: Cell<bool>,
    /// Whether this parser should avoid passing any further data to the tokenizer.
    suspended: Cell<bool>,
}

impl ServoParser {
    #[allow(unrooted_must_root)]
    fn new_inherited(
            document: &Document,
            pipeline: Option<PipelineId>,
            tokenizer: Tokenizer,
            last_chunk_received: bool)
            -> Self {
        ServoParser {
            reflector: Reflector::new(),
            document: JS::from_ref(document),
            pipeline: pipeline,
            pending_input: DOMRefCell::new(vec![]),
            tokenizer: DOMRefCell::new(tokenizer),
            last_chunk_received: Cell::new(last_chunk_received),
            suspended: Default::default(),
        }
    }

    #[allow(unrooted_must_root)]
    pub fn new(
            document: &Document,
            pipeline: Option<PipelineId>,
            tokenizer: Tokenizer,
            last_chunk_received: bool)
            -> Root<Self> {
        reflect_dom_object(
            box ServoParser::new_inherited(document, pipeline, tokenizer, last_chunk_received),
            document.window(),
            ServoParserBinding::Wrap)
    }

    pub fn document(&self) -> &Document {
        &self.document
    }

    pub fn pipeline(&self) -> Option<PipelineId> {
        self.pipeline
    }

    pub fn has_pending_input(&self) -> bool {
        !self.pending_input.borrow().is_empty()
    }

    pub fn push_input_chunk(&self, chunk: String) {
        self.pending_input.borrow_mut().push(chunk);
    }

    pub fn take_next_input_chunk(&self) -> Option<String> {
        let mut pending_input = self.pending_input.borrow_mut();
        if pending_input.is_empty() {
            None
        } else {
            Some(pending_input.remove(0))
        }
    }

    pub fn last_chunk_received(&self) -> bool {
        self.last_chunk_received.get()
    }

    pub fn mark_last_chunk_received(&self) {
        self.last_chunk_received.set(true)
    }

    pub fn set_plaintext_state(&self) {
        self.tokenizer.borrow_mut().set_plaintext_state()
    }

    pub fn end_tokenizer(&self) {
        self.tokenizer.borrow_mut().end()
    }

    pub fn window(&self) -> &Window {
        self.document().window()
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

    pub fn parse_sync(&self) {
        let metadata = TimerMetadata {
            url: self.document().url().as_str().into(),
            iframe: TimerMetadataFrameType::RootWindow,
            incremental: TimerMetadataReflowType::FirstReflow,
        };
        let profiler_category = self.tokenizer.borrow().profiler_category();
        profile(profiler_category,
                Some(metadata),
                self.document().window().upcast::<GlobalScope>().time_profiler_chan().clone(),
                || self.do_parse_sync())
    }

    fn do_parse_sync(&self) {
        // This parser will continue to parse while there is either pending input or
        // the parser remains unsuspended.
        loop {
            self.document().reflow_if_reflow_timer_expired();
            if let Some(chunk) = self.take_next_input_chunk() {
                self.tokenizer.borrow_mut().feed(chunk);
            } else {
                self.tokenizer.borrow_mut().run();
            }

            // Document parsing is blocked on an external resource.
            if self.suspended.get() {
                return;
            }

            if !self.has_pending_input() {
                break;
            }
        }

        if self.last_chunk_received() {
            self.finish();
        }
    }

    pub fn parse_chunk(&self, input: String) {
        self.document().set_current_parser(Some(self));
        self.push_input_chunk(input);
        if !self.is_suspended() {
            self.parse_sync();
        }
    }

    pub fn finish(&self) {
        assert!(!self.suspended.get());
        assert!(!self.has_pending_input());

        self.tokenizer.borrow_mut().end();
        debug!("finished parsing");

        self.document().set_current_parser(None);

        if let Some(pipeline) = self.pipeline() {
            ScriptThread::parsing_complete(pipeline);
        }
    }
}

#[derive(HeapSizeOf)]
#[must_root]
pub enum Tokenizer {
    HTML(
        #[ignore_heap_size_of = "Defined in html5ever"]
        HtmlTokenizer<HtmlTreeBuilder<JS<Node>, Sink>>
    ),
    XML(
        #[ignore_heap_size_of = "Defined in xml5ever"]
        XmlTokenizer<XmlTreeBuilder<JS<Node>, Sink>>
    ),
}

impl Tokenizer {
    pub fn feed(&mut self, input: String) {
        match *self {
            Tokenizer::HTML(ref mut tokenizer) => tokenizer.feed(input.into()),
            Tokenizer::XML(ref mut tokenizer) => tokenizer.feed(input.into()),
        }
    }

    pub fn run(&mut self) {
        match *self {
            Tokenizer::HTML(ref mut tokenizer) => tokenizer.run(),
            Tokenizer::XML(ref mut tokenizer) => tokenizer.run(),
        }
    }

    pub fn end(&mut self) {
        match *self {
            Tokenizer::HTML(ref mut tokenizer) => tokenizer.end(),
            Tokenizer::XML(ref mut tokenizer) => tokenizer.end(),
        }
    }

    pub fn set_plaintext_state(&mut self) {
        match *self {
            Tokenizer::HTML(ref mut tokenizer) => tokenizer.set_plaintext_state(),
            Tokenizer::XML(_) => { /* todo */ },
        }
    }

    pub fn profiler_category(&self) -> ProfilerCategory {
        match *self {
            Tokenizer::HTML(_) => ProfilerCategory::ScriptParseHTML,
            Tokenizer::XML(_) => ProfilerCategory::ScriptParseXML,
        }
    }
}

impl JSTraceable for Tokenizer {
    fn trace(&self, trc: *mut JSTracer) {
        struct Tracer(*mut JSTracer);
        let tracer = Tracer(trc);

        match *self {
            Tokenizer::HTML(ref tokenizer) => {
                impl HtmlTracer for Tracer {
                    type Handle = JS<Node>;
                    #[allow(unrooted_must_root)]
                    fn trace_handle(&self, node: &JS<Node>) {
                        node.trace(self.0);
                    }
                }
                let tree_builder = tokenizer.sink();
                tree_builder.trace_handles(&tracer);
                tree_builder.sink().trace(trc);
            },
            Tokenizer::XML(ref tokenizer) => {
                impl XmlTracer for Tracer {
                    type Handle = JS<Node>;
                    #[allow(unrooted_must_root)]
                    fn trace_handle(&self, node: JS<Node>) {
                        node.trace(self.0);
                    }
                }
                let tree_builder = tokenizer.sink();
                tree_builder.trace_handles(&tracer);
                tree_builder.sink().trace(trc);
            }
        }
    }
}
