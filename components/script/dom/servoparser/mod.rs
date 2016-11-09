/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use document_loader::LoadType;
use dom::bindings::cell::DOMRefCell;
use dom::bindings::codegen::Bindings::DocumentBinding::DocumentMethods;
use dom::bindings::codegen::Bindings::HTMLImageElementBinding::HTMLImageElementMethods;
use dom::bindings::codegen::Bindings::NodeBinding::NodeMethods;
use dom::bindings::codegen::Bindings::ServoParserBinding;
use dom::bindings::inheritance::Castable;
use dom::bindings::js::{JS, Root};
use dom::bindings::refcounted::Trusted;
use dom::bindings::reflector::{Reflector, reflect_dom_object};
use dom::bindings::str::DOMString;
use dom::bindings::trace::JSTraceable;
use dom::document::Document;
use dom::globalscope::GlobalScope;
use dom::htmlimageelement::HTMLImageElement;
use dom::htmlscriptelement::HTMLScriptElement;
use dom::node::Node;
use encoding::all::UTF_8;
use encoding::types::{DecoderTrap, Encoding};
use html5ever::tokenizer::{Tokenizer as H5ETokenizer, TokenizerResult};
use html5ever::tokenizer::buffer_queue::BufferQueue;
use html5ever::tree_builder::Tracer as HtmlTracer;
use html5ever::tree_builder::TreeBuilder as HtmlTreeBuilder;
use hyper::header::ContentType;
use hyper::mime::{Mime, SubLevel, TopLevel};
use hyper_serde::Serde;
use js::jsapi::JSTracer;
use msg::constellation_msg::PipelineId;
use net_traits::{FetchMetadata, FetchResponseListener, Metadata, NetworkError};
use network_listener::PreInvoke;
use profile_traits::time::{TimerMetadata, TimerMetadataFrameType};
use profile_traits::time::{TimerMetadataReflowType, ProfilerCategory, profile};
use script_thread::ScriptThread;
use std::cell::Cell;
use std::collections::VecDeque;
use url::Url;
use util::resource_files::read_resource_file;
use xml5ever::tokenizer::XmlTokenizer;
use xml5ever::tree_builder::{Tracer as XmlTracer, XmlTreeBuilder};

pub mod html;
pub mod xml;

#[dom_struct]
pub struct ServoParser {
    reflector: Reflector,
    /// The document associated with this parser.
    document: JS<Document>,
    /// The pipeline associated with this parse, unavailable if this parse
    /// does not correspond to a page load.
    pipeline: Option<PipelineId>,
    /// Input chunks received but not yet passed to the parser.
    pending_input: DOMRefCell<VecDeque<String>>,
    /// The tokenizer of this parser.
    tokenizer: DOMRefCell<Tokenizer>,
    /// Whether to expect any further input from the associated network request.
    last_chunk_received: Cell<bool>,
    /// Whether this parser should avoid passing any further data to the tokenizer.
    suspended: Cell<bool>,
}

#[derive(PartialEq)]
enum LastChunkState {
    Received,
    NotReceived,
}

impl ServoParser {
    #[allow(unrooted_must_root)]
    fn new_inherited(
            document: &Document,
            pipeline: Option<PipelineId>,
            tokenizer: Tokenizer,
            last_chunk_state: LastChunkState)
            -> Self {
        ServoParser {
            reflector: Reflector::new(),
            document: JS::from_ref(document),
            pipeline: pipeline,
            pending_input: DOMRefCell::new(VecDeque::new()),
            tokenizer: DOMRefCell::new(tokenizer),
            last_chunk_received: Cell::new(last_chunk_state == LastChunkState::Received),
            suspended: Default::default(),
        }
    }

    #[allow(unrooted_must_root)]
    fn new(
            document: &Document,
            pipeline: Option<PipelineId>,
            tokenizer: Tokenizer,
            last_chunk_state: LastChunkState)
            -> Root<Self> {
        reflect_dom_object(
            box ServoParser::new_inherited(document, pipeline, tokenizer, last_chunk_state),
            document.window(),
            ServoParserBinding::Wrap)
    }

    pub fn document(&self) -> &Document {
        &self.document
    }

    pub fn pipeline(&self) -> Option<PipelineId> {
        self.pipeline
    }

    fn has_pending_input(&self) -> bool {
        !self.pending_input.borrow().is_empty()
    }

    fn push_input_chunk(&self, chunk: String) {
        self.pending_input.borrow_mut().push_back(chunk);
    }

    fn take_next_input_chunk(&self) -> Option<String> {
        let mut pending_input = self.pending_input.borrow_mut();
        if pending_input.is_empty() {
            None
        } else {
            pending_input.pop_front()
        }
    }

    fn last_chunk_received(&self) -> bool {
        self.last_chunk_received.get()
    }

    fn mark_last_chunk_received(&self) {
        self.last_chunk_received.set(true)
    }

    fn set_plaintext_state(&self) {
        self.tokenizer.borrow_mut().set_plaintext_state()
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

    fn parse_sync(&self) {
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

    fn parse_chunk(&self, input: String) {
        self.document().set_current_parser(Some(self));
        self.push_input_chunk(input);
        if !self.is_suspended() {
            self.parse_sync();
        }
    }

    fn finish(&self) {
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
enum Tokenizer {
    HTML(HtmlTokenizer),
    XML(
        #[ignore_heap_size_of = "Defined in xml5ever"]
        XmlTokenizer<XmlTreeBuilder<JS<Node>, Sink>>
    ),
}

#[derive(HeapSizeOf)]
#[must_root]
struct HtmlTokenizer {
    #[ignore_heap_size_of = "Defined in html5ever"]
    inner: H5ETokenizer<HtmlTreeBuilder<JS<Node>, Sink>>,
    #[ignore_heap_size_of = "Defined in html5ever"]
    input_buffer: BufferQueue,
}

impl HtmlTokenizer {
    #[allow(unrooted_must_root)]
    fn new(inner: H5ETokenizer<HtmlTreeBuilder<JS<Node>, Sink>>) -> Self {
        HtmlTokenizer {
            inner: inner,
            input_buffer: BufferQueue::new(),
        }
    }

    fn feed(&mut self, input: String) {
        self.input_buffer.push_back(input.into());
        self.run();
    }

    #[allow(unrooted_must_root)]
    fn run(&mut self) {
        while let TokenizerResult::Script(script) = self.inner.feed(&mut self.input_buffer) {
            let script = Root::from_ref(script.downcast::<HTMLScriptElement>().unwrap());
            if !script.prepare() {
                break;
            }
        }
    }

    fn end(&mut self) {
        assert!(self.input_buffer.is_empty());
        self.inner.end();
    }

    fn set_plaintext_state(&mut self) {
        self.inner.set_plaintext_state();
    }
}

#[derive(JSTraceable, HeapSizeOf)]
#[must_root]
struct Sink {
    pub base_url: Url,
    pub document: JS<Document>,
}

impl Tokenizer {
    fn feed(&mut self, input: String) {
        match *self {
            Tokenizer::HTML(ref mut tokenizer) => tokenizer.feed(input),
            Tokenizer::XML(ref mut tokenizer) => tokenizer.feed(input.into()),
        }
    }

    fn run(&mut self) {
        match *self {
            Tokenizer::HTML(ref mut tokenizer) => tokenizer.run(),
            Tokenizer::XML(ref mut tokenizer) => tokenizer.run(),
        }
    }

    fn end(&mut self) {
        match *self {
            Tokenizer::HTML(ref mut tokenizer) => tokenizer.end(),
            Tokenizer::XML(ref mut tokenizer) => tokenizer.end(),
        }
    }

    fn set_plaintext_state(&mut self) {
        match *self {
            Tokenizer::HTML(ref mut tokenizer) => tokenizer.set_plaintext_state(),
            Tokenizer::XML(_) => { /* todo */ },
        }
    }

    fn profiler_category(&self) -> ProfilerCategory {
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
                let tree_builder = tokenizer.inner.sink();
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

/// The context required for asynchronously fetching a document
/// and parsing it progressively.
pub struct ParserContext {
    /// The parser that initiated the request.
    parser: Option<Trusted<ServoParser>>,
    /// Is this a synthesized document
    is_synthesized_document: bool,
    /// The pipeline associated with this document.
    id: PipelineId,
    /// The URL for this document.
    url: Url,
}

impl ParserContext {
    pub fn new(id: PipelineId, url: Url) -> ParserContext {
        ParserContext {
            parser: None,
            is_synthesized_document: false,
            id: id,
            url: url,
        }
    }
}

impl FetchResponseListener for ParserContext {
    fn process_request_body(&mut self) {}

    fn process_request_eof(&mut self) {}

    fn process_response(&mut self,
                        meta_result: Result<FetchMetadata, NetworkError>) {
        let mut ssl_error = None;
        let metadata = match meta_result {
            Ok(meta) => {
                Some(match meta {
                    FetchMetadata::Unfiltered(m) => m,
                    FetchMetadata::Filtered { unsafe_, .. } => unsafe_
                })
            },
            Err(NetworkError::SslValidation(url, reason)) => {
                ssl_error = Some(reason);
                let mut meta = Metadata::default(url);
                let mime: Option<Mime> = "text/html".parse().ok();
                meta.set_content_type(mime.as_ref());
                Some(meta)
            },
            Err(_) => None,
        };
        let content_type =
            metadata.clone().and_then(|meta| meta.content_type).map(Serde::into_inner);
        let parser = match ScriptThread::page_headers_available(&self.id,
                                                                metadata) {
            Some(parser) => parser,
            None => return,
        };

        self.parser = Some(Trusted::new(&*parser));

        match content_type {
            Some(ContentType(Mime(TopLevel::Image, _, _))) => {
                self.is_synthesized_document = true;
                let page = "<html><body></body></html>".into();
                parser.push_input_chunk(page);
                parser.parse_sync();

                let doc = parser.document();
                let doc_body = Root::upcast::<Node>(doc.GetBody().unwrap());
                let img = HTMLImageElement::new(local_name!("img"), None, doc);
                img.SetSrc(DOMString::from(self.url.to_string()));
                doc_body.AppendChild(&Root::upcast::<Node>(img)).expect("Appending failed");

            },
            Some(ContentType(Mime(TopLevel::Text, SubLevel::Plain, _))) => {
                // https://html.spec.whatwg.org/multipage/#read-text
                let page = "<pre>\n".into();
                parser.push_input_chunk(page);
                parser.parse_sync();
                parser.set_plaintext_state();
            },
            Some(ContentType(Mime(TopLevel::Text, SubLevel::Html, _))) => { // Handle text/html
                if let Some(reason) = ssl_error {
                    self.is_synthesized_document = true;
                    let page_bytes = read_resource_file("badcert.html").unwrap();
                    let page = String::from_utf8(page_bytes).unwrap();
                    let page = page.replace("${reason}", &reason);
                    parser.push_input_chunk(page);
                    parser.parse_sync();
                }
            },
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
                parser.push_input_chunk(page);
                parser.parse_sync();
            },
            None => {
                // No content-type header.
                // Merge with #4212 when fixed.
            }
        }
    }

    fn process_response_chunk(&mut self, payload: Vec<u8>) {
        if !self.is_synthesized_document {
            // FIXME: use Vec<u8> (html5ever #34)
            let data = UTF_8.decode(&payload, DecoderTrap::Replace).unwrap();
            let parser = match self.parser.as_ref() {
                Some(parser) => parser.root(),
                None => return,
            };
            parser.parse_chunk(data);
        }
    }

    fn process_response_eof(&mut self, status: Result<(), NetworkError>) {
        let parser = match self.parser.as_ref() {
            Some(parser) => parser.root(),
            None => return,
        };

        if let Err(NetworkError::Internal(ref reason)) = status {
            // Show an error page for network errors,
            // certificate errors are handled earlier.
            self.is_synthesized_document = true;
            let page_bytes = read_resource_file("neterror.html").unwrap();
            let page = String::from_utf8(page_bytes).unwrap();
            let page = page.replace("${reason}", reason);
            parser.push_input_chunk(page);
            parser.parse_sync();
        } else if let Err(err) = status {
            // TODO(Savago): we should send a notification to callers #5463.
            debug!("Failed to load page URL {}, error: {:?}", self.url, err);
        }

        parser.document()
            .finish_load(LoadType::PageSource(self.url.clone()));

        parser.mark_last_chunk_received();
        if !parser.is_suspended() {
            parser.parse_sync();
        }
    }
}

impl PreInvoke for ParserContext {}
