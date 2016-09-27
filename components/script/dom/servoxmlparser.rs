/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use dom::bindings::cell::DOMRefCell;
use dom::bindings::codegen::Bindings::ServoXMLParserBinding;
use dom::bindings::js::{JS, Root};
use dom::bindings::reflector::{Reflector, reflect_dom_object};
use dom::bindings::trace::JSTraceable;
use dom::document::Document;
use dom::node::Node;
use dom::window::Window;
use js::jsapi::JSTracer;
use msg::constellation_msg::PipelineId;
use parse::{Parser, ParserRef};
use script_thread::ScriptThread;
use std::cell::Cell;
use url::Url;
use xml5ever::tokenizer;
use xml5ever::tree_builder::{self, XmlTreeBuilder};

pub type Tokenizer = tokenizer::XmlTokenizer<XmlTreeBuilder<JS<Node>, Sink>>;

#[must_root]
#[derive(JSTraceable, HeapSizeOf)]
pub struct Sink {
    pub base_url: Option<Url>,
    pub document: JS<Document>,
}

#[must_root]
#[dom_struct]
pub struct ServoXMLParser {
    reflector_: Reflector,
    #[ignore_heap_size_of = "Defined in xml5ever"]
    tokenizer: DOMRefCell<Tokenizer>,
    /// Input chunks received but not yet passed to the parser.
    pending_input: DOMRefCell<Vec<String>>,
    /// The document associated with this parser.
    document: JS<Document>,
    /// True if this parser should avoid passing any further data to the tokenizer.
    suspended: Cell<bool>,
    /// Whether to expect any further input from the associated network request.
    last_chunk_received: Cell<bool>,
    /// The pipeline associated with this parse, unavailable if this parse does not
    /// correspond to a page load.
    pipeline: Option<PipelineId>,
}

impl<'a> Parser for &'a ServoXMLParser {
    fn parse_chunk(self, input: String) {
        self.document.set_current_parser(Some(ParserRef::XML(self)));
        self.pending_input.borrow_mut().push(input);
        if !self.is_suspended() {
            self.parse_sync();
        }
    }

    fn finish(self) {
        assert!(!self.suspended.get());
        assert!(self.pending_input.borrow().is_empty());

        self.tokenizer.borrow_mut().end();
        debug!("finished parsing");

        self.document.set_current_parser(None);

        if let Some(pipeline) = self.pipeline {
            ScriptThread::parsing_complete(pipeline);
        }
    }
}

impl ServoXMLParser {
    #[allow(unrooted_must_root)]
    pub fn new(base_url: Option<Url>, document: &Document, pipeline: Option<PipelineId>)
               -> Root<ServoXMLParser> {
        let sink = Sink {
            base_url: base_url,
            document: JS::from_ref(document),
        };

        let tb = XmlTreeBuilder::new(sink);

        let tok = tokenizer::XmlTokenizer::new(tb, Default::default());

        let parser = ServoXMLParser {
            reflector_: Reflector::new(),
            tokenizer: DOMRefCell::new(tok),
            pending_input: DOMRefCell::new(vec!()),
            document: JS::from_ref(document),
            suspended: Cell::new(false),
            last_chunk_received: Cell::new(false),
            pipeline: pipeline,
        };

        reflect_dom_object(box parser, document.window(), ServoXMLParserBinding::Wrap)
    }

    pub fn window(&self) -> &Window {
        self.document.window()
    }

    pub fn resume(&self) {
        assert!(self.suspended.get());
        self.suspended.set(false);
        self.parse_sync();
    }

    pub fn suspend(&self) {
        assert!(!self.suspended.get());
        self.suspended.set(true);
    }

    pub fn is_suspended(&self) -> bool {
        self.suspended.get()
    }

    pub fn parse_sync(&self) {
        // This parser will continue to parse while there is either pending input or
        // the parser remains unsuspended.
        loop {
           self.document.reflow_if_reflow_timer_expired();
            let mut pending_input = self.pending_input.borrow_mut();
            if !pending_input.is_empty() {
                let chunk = pending_input.remove(0);
                self.tokenizer.borrow_mut().feed(chunk.into());
            } else {
                self.tokenizer.borrow_mut().run();
            }

            // Document parsing is blocked on an external resource.
            if self.suspended.get() {
                return;
            }

            if pending_input.is_empty() {
                break;
            }
        }

        if self.last_chunk_received.get() {
            self.finish();
        }
    }

    pub fn pending_input(&self) -> &DOMRefCell<Vec<String>> {
        &self.pending_input
    }

    pub fn set_plaintext_state(&self) {
        //self.tokenizer.borrow_mut().set_plaintext_state()
    }

    pub fn end_tokenizer(&self) {
        self.tokenizer.borrow_mut().end()
    }

    pub fn document(&self) -> &Document {
        &self.document
    }

    pub fn last_chunk_received(&self) -> &Cell<bool> {
        &self.last_chunk_received
    }

    pub fn tokenizer(&self) -> &DOMRefCell<Tokenizer> {
        &self.tokenizer
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
