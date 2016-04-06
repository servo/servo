/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

//! The bulk of the HTML parser integration is in `script::parse::html`.
//! This module is mostly about its interaction with DOM memory management.

use document_loader::LoadType;
use dom::bindings::cell::DOMRefCell;
use dom::bindings::codegen::Bindings::ServoHTMLParserBinding;
use dom::bindings::global::GlobalRef;
use dom::bindings::js::{JS, Root};
use dom::bindings::refcounted::Trusted;
use dom::bindings::reflector::{Reflector, reflect_dom_object};
use dom::bindings::trace::JSTraceable;
use dom::document::Document;
use dom::node::Node;
use dom::servoxmlparser::ServoXMLParser;
use dom::window::Window;
use encoding::all::UTF_8;
use encoding::types::{DecoderTrap, Encoding};
use html5ever::tokenizer;
use html5ever::tree_builder;
use html5ever::tree_builder::{TreeBuilder, TreeBuilderOpts};
use hyper::header::ContentType;
use hyper::mime::{Mime, SubLevel, TopLevel};
use js::jsapi::JSTracer;
use msg::constellation_msg::{PipelineId, SubpageId};
use net_traits::{AsyncResponseListener, Metadata};
use network_listener::PreInvoke;
use parse::Parser;
use script_runtime::ScriptChan;
use script_thread::ScriptThread;
use std::cell::Cell;
use std::cell::UnsafeCell;
use std::default::Default;
use std::ptr;
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

#[must_root]
#[derive(JSTraceable, HeapSizeOf)]
pub enum ParserField {
    HTML(JS<ServoHTMLParser>),
    XML(JS<ServoXMLParser>),
}

#[must_root]
#[derive(JSTraceable, HeapSizeOf)]
pub struct MutNullableParserField {
    #[ignore_heap_size_of = "XXXjdm"]
    ptr: UnsafeCell<Option<ParserField>>,
}

impl Default for MutNullableParserField {
    #[allow(unrooted_must_root)]
    fn default() -> MutNullableParserField {
        MutNullableParserField {
            ptr: UnsafeCell::new(None),
        }
    }
}

impl MutNullableParserField {
    #[allow(unsafe_code)]
    pub fn set(&self, val: Option<ParserRef>) {
        unsafe {
            *self.ptr.get() = val.map(|val| {
                match val {
                    ParserRef::HTML(parser) => ParserField::HTML(JS::from_ref(parser)),
                    ParserRef::XML(parser) => ParserField::XML(JS::from_ref(parser)),
                }
            });
        }
    }

    #[allow(unsafe_code, unrooted_must_root)]
    pub fn get(&self) -> Option<ParserRoot> {
        unsafe {
            ptr::read(self.ptr.get()).map(|o| {
                match o {
                    ParserField::HTML(parser) => ParserRoot::HTML(Root::from_ref(&*parser)),
                    ParserField::XML(parser) => ParserRoot::XML(Root::from_ref(&*parser)),
                }
            })
        }
    }
}

pub enum ParserRoot {
    HTML(Root<ServoHTMLParser>),
    XML(Root<ServoXMLParser>),
}

impl ParserRoot {
    pub fn r(&self) -> ParserRef {
        match *self {
            ParserRoot::HTML(ref parser) => ParserRef::HTML(parser.r()),
            ParserRoot::XML(ref parser) => ParserRef::XML(parser.r()),
        }
    }
}

enum TrustedParser {
    HTML(Trusted<ServoHTMLParser>),
    XML(Trusted<ServoXMLParser>),
}

impl TrustedParser {
    pub fn root(&self) -> ParserRoot {
        match *self {
            TrustedParser::HTML(ref parser) => ParserRoot::HTML(parser.root()),
            TrustedParser::XML(ref parser) => ParserRoot::XML(parser.root()),
        }
    }
}

pub enum ParserRef<'a> {
    HTML(&'a ServoHTMLParser),
    XML(&'a ServoXMLParser),
}

impl<'a> ParserRef<'a> {
    fn parse_chunk(&self, input: String) {
        match *self {
            ParserRef::HTML(parser) => parser.parse_chunk(input),
            ParserRef::XML(parser) => parser.parse_chunk(input),
        }
    }

    pub fn window(&self) -> &Window {
        match *self {
            ParserRef::HTML(parser) => parser.window(),
            ParserRef::XML(parser) => parser.window(),
        }
    }

    pub fn resume(&self) {
        match *self {
            ParserRef::HTML(parser) => parser.resume(),
            ParserRef::XML(parser) => parser.resume(),
        }
    }

    pub fn suspend(&self) {
        match *self {
            ParserRef::HTML(parser) => parser.suspend(),
            ParserRef::XML(parser) => parser.suspend(),
        }
    }

    pub fn is_suspended(&self) -> bool {
        match *self {
            ParserRef::HTML(parser) => parser.is_suspended(),
            ParserRef::XML(parser) => parser.is_suspended(),
        }
    }

    pub fn pending_input(&self) -> &DOMRefCell<Vec<String>> {
        match *self {
            ParserRef::HTML(parser) => parser.pending_input(),
            ParserRef::XML(parser) => parser.pending_input(),
        }
    }

    pub fn set_plaintext_state(&self) {
        match *self {
            ParserRef::HTML(parser) => parser.set_plaintext_state(),
            ParserRef::XML(parser) => parser.set_plaintext_state(),
        }
    }

    pub fn parse_sync(&self) {
        match *self {
            ParserRef::HTML(parser) => parser.parse_sync(),
            ParserRef::XML(parser) => parser.parse_sync(),
        }
    }

    pub fn document(&self) -> &Document {
        match *self {
            ParserRef::HTML(parser) => parser.document(),
            ParserRef::XML(parser) => parser.document(),
        }
    }

    pub fn last_chunk_received(&self) -> &Cell<bool> {
        match *self {
            ParserRef::HTML(parser) => parser.last_chunk_received(),
            ParserRef::XML(parser) => parser.last_chunk_received(),
        }
    }
}

/// The context required for asynchronously fetching a document and parsing it progressively.
pub struct ParserContext {
    /// The parser that initiated the request.
    parser: Option<TrustedParser>,
    /// Is this a synthesized document
    is_synthesized_document: bool,
    /// The pipeline associated with this document.
    id: PipelineId,
    /// The subpage associated with this document.
    subpage: Option<SubpageId>,
    /// The target event loop for the response notifications.
    script_chan: Box<ScriptChan + Send>,
    /// The URL for this document.
    url: Url,
}

impl ParserContext {
    pub fn new(id: PipelineId, subpage: Option<SubpageId>, script_chan: Box<ScriptChan + Send>,
               url: Url) -> ParserContext {
        ParserContext {
            parser: None,
            is_synthesized_document: false,
            id: id,
            subpage: subpage,
            script_chan: script_chan,
            url: url,
        }
    }
}

impl AsyncResponseListener for ParserContext {
    fn headers_available(&mut self, metadata: Metadata) {
        let content_type = metadata.content_type.clone();

        let parser = ScriptThread::page_fetch_complete(self.id.clone(), self.subpage.clone(),
                                                     metadata);
        let parser = match parser {
            Some(parser) => parser,
            None => return,
        };

        let parser = parser.r();
        self.parser = Some(match parser {
            ParserRef::HTML(parser) => TrustedParser::HTML(
                                        Trusted::new(parser,
                                                     self.script_chan.clone())),
            ParserRef::XML(parser) => TrustedParser::XML(
                                        Trusted::new(parser,
                                                     self.script_chan.clone())),
        });

        match content_type {
            Some(ContentType(Mime(TopLevel::Image, _, _))) => {
                self.is_synthesized_document = true;
                let page = format!("<html><body><img src='{}' /></body></html>",
                                   self.url.serialize());
                parser.pending_input().borrow_mut().push(page);
                parser.parse_sync();
            },
            Some(ContentType(Mime(TopLevel::Text, SubLevel::Plain, _))) => {
                // https://html.spec.whatwg.org/multipage/#read-text
                let page = format!("<pre>\n");
                parser.pending_input().borrow_mut().push(page);
                parser.parse_sync();
                parser.set_plaintext_state();
            },
            Some(ContentType(Mime(TopLevel::Text, SubLevel::Html, _))) => {}, // Handle text/html
            Some(ContentType(Mime(TopLevel::Text, SubLevel::Xml, _))) => {}, // Handle text/xml
            Some(ContentType(Mime(toplevel, sublevel, _))) => {
                if toplevel.as_str() == "application" && sublevel.as_str() == "xhtml+xml" {
                    // Handle xhtml (application/xhtml+xml).
                    return;
                }

                // Show warning page for unknown mime types.
                let page = format!("<html><body><p>Unknown content type ({}/{}).</p></body></html>",
                    toplevel.as_str(), sublevel.as_str());
                self.is_synthesized_document = true;
                parser.pending_input().borrow_mut().push(page);
                parser.parse_sync();
            },
            None => {
                // No content-type header.
                // Merge with #4212 when fixed.
            }
        }
    }

    fn data_available(&mut self, payload: Vec<u8>) {
        if !self.is_synthesized_document {
            // FIXME: use Vec<u8> (html5ever #34)
            let data = UTF_8.decode(&payload, DecoderTrap::Replace).unwrap();
            let parser = match self.parser.as_ref() {
                Some(parser) => parser.root(),
                None => return,
            };
            parser.r().parse_chunk(data);
        }
    }

    fn response_complete(&mut self, status: Result<(), String>) {
        let parser = match self.parser.as_ref() {
            Some(parser) => parser.root(),
            None => return,
        };
        parser.r().document().finish_load(LoadType::PageSource(self.url.clone()));

        if let Err(err) = status {
            debug!("Failed to load page URL {}, error: {}", self.url.serialize(), err);
            // TODO(Savago): we should send a notification to callers #5463.
        }

        parser.r().last_chunk_received().set(true);
        if !parser.r().is_suspended() {
            parser.r().parse_sync();
        }
    }
}

impl PreInvoke for ParserContext {
}

#[dom_struct]
pub struct ServoHTMLParser {
    reflector_: Reflector,
    #[ignore_heap_size_of = "Defined in html5ever"]
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

impl<'a> Parser for &'a ServoHTMLParser {
    fn parse_chunk(self, input: String) {
        self.document.set_current_parser(Some(ParserRef::HTML(self)));
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
            reflector_: Reflector::new(),
            tokenizer: DOMRefCell::new(tok),
            pending_input: DOMRefCell::new(vec!()),
            document: JS::from_ref(document),
            suspended: Cell::new(false),
            last_chunk_received: Cell::new(false),
            pipeline: pipeline,
        };

        reflect_dom_object(box parser, GlobalRef::Window(document.window()),
                           ServoHTMLParserBinding::Wrap)
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
            reflector_: Reflector::new(),
            tokenizer: DOMRefCell::new(tok),
            pending_input: DOMRefCell::new(vec!()),
            document: JS::from_ref(document),
            suspended: Cell::new(false),
            last_chunk_received: Cell::new(true),
            pipeline: None,
        };

        reflect_dom_object(box parser, GlobalRef::Window(document.window()),
                           ServoHTMLParserBinding::Wrap)
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

    pub fn pending_input(&self) -> &DOMRefCell<Vec<String>> {
        &self.pending_input
    }

}


impl ServoHTMLParser {
    fn parse_sync(&self) {
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

    fn window(&self) -> &Window {
        self.document.window()
    }

    fn suspend(&self) {
        assert!(!self.suspended.get());
        self.suspended.set(true);
    }

    fn resume(&self) {
        assert!(self.suspended.get());
        self.suspended.set(false);
        self.parse_sync();
    }

    fn is_suspended(&self) -> bool {
        self.suspended.get()
    }

    fn document(&self) -> &Document {
        &self.document
    }

    fn last_chunk_received(&self) -> &Cell<bool> {
        &self.last_chunk_received
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
