/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use dom::bindings::cell::DOMRefCell;
use dom::bindings::codegen::Bindings::ServoXMLParserBinding;
use dom::bindings::global::GlobalRef;
use dom::bindings::js::{JS, Root};
use dom::bindings::reflector::{Reflector, reflect_dom_object};
use dom::bindings::trace::JSTraceable;
use dom::document::Document;
use dom::node::Node;
use dom::servohtmlparser::ParserRef;
use dom::text::Text;
use dom::window::Window;
use js::jsapi::JSTracer;
use msg::constellation_msg::PipelineId;
use parse::Parser;
use script_task::ScriptTask;
use std::cell::Cell;
use url::Url;
use util::str::DOMString;
use xml5ever::tokenizer;
use xml5ever::tree_builder::{self, NodeOrText, XmlTreeBuilder};

pub type Tokenizer = tokenizer::XmlTokenizer<XmlTreeBuilder<JS<Node>, Sink>>;

#[must_root]
#[derive(JSTraceable, HeapSizeOf)]
pub struct Sink {
    pub base_url: Option<Url>,
    pub document: JS<Document>,
}

impl Sink {
    #[allow(unrooted_must_root)] // method is only run at parse time
    pub fn get_or_create(&self, child: NodeOrText<JS<Node>>) -> Root<Node> {
        match child {
            NodeOrText::AppendNode(n) => Root::from_ref(&*n),
            NodeOrText::AppendText(t) => {
                let s: String = t.into();
                let text = Text::new(DOMString::from(s), &self.document);
                Root::upcast(text)
            }
        }
    }
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
            ScriptTask::parsing_complete(pipeline);
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

        reflect_dom_object(box parser, GlobalRef::Window(document.window()),
                           ServoXMLParserBinding::Wrap)
    }

    pub fn window(&self) -> &Window {
        self.document.window()
    }

    pub fn resume(&self) {
        panic!()
    }

    pub fn suspend(&self) {
        panic!()
    }

    pub fn is_suspended(&self) -> bool {
        panic!()
    }

    pub fn parse_sync(&self) {
        panic!()
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
