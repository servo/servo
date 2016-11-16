/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use document_loader::{DocumentLoader, LoadType};
use dom::bindings::cell::DOMRefCell;
use dom::bindings::codegen::Bindings::DocumentBinding::DocumentMethods;
use dom::bindings::codegen::Bindings::HTMLImageElementBinding::HTMLImageElementMethods;
use dom::bindings::codegen::Bindings::NodeBinding::NodeMethods;
use dom::bindings::codegen::Bindings::ServoParserBinding;
use dom::bindings::inheritance::Castable;
use dom::bindings::js::{JS, Root, RootedReference};
use dom::bindings::refcounted::Trusted;
use dom::bindings::reflector::{Reflector, reflect_dom_object};
use dom::bindings::str::DOMString;
use dom::document::{Document, DocumentSource, IsHTMLDocument};
use dom::globalscope::GlobalScope;
use dom::htmlformelement::HTMLFormElement;
use dom::htmlimageelement::HTMLImageElement;
use dom::htmlscriptelement::HTMLScriptElement;
use dom::node::{Node, document_from_node, window_from_node};
use encoding::all::UTF_8;
use encoding::types::{DecoderTrap, Encoding};
use html5ever::tokenizer::buffer_queue::BufferQueue;
use hyper::header::ContentType;
use hyper::mime::{Mime, SubLevel, TopLevel};
use hyper_serde::Serde;
use msg::constellation_msg::PipelineId;
use net_traits::{FetchMetadata, FetchResponseListener, Metadata, NetworkError};
use network_listener::PreInvoke;
use profile_traits::time::{TimerMetadata, TimerMetadataFrameType};
use profile_traits::time::{TimerMetadataReflowType, ProfilerCategory, profile};
use script_thread::ScriptThread;
use servo_url::ServoUrl;
use std::cell::Cell;
use util::resource_files::read_resource_file;

mod html;
mod xml;

#[dom_struct]
pub struct ServoParser {
    reflector: Reflector,
    /// The document associated with this parser.
    document: JS<Document>,
    /// The pipeline associated with this parse, unavailable if this parse
    /// does not correspond to a page load.
    pipeline: Option<PipelineId>,
    /// Input chunks received but not yet passed to the parser.
    #[ignore_heap_size_of = "Defined in html5ever"]
    pending_input: DOMRefCell<BufferQueue>,
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
    pub fn parse_html_document(
            document: &Document,
            input: DOMString,
            url: ServoUrl,
            owner: Option<PipelineId>) {
        let parser = ServoParser::new(
            document,
            owner,
            Tokenizer::Html(self::html::Tokenizer::new(document, url, None)),
            LastChunkState::NotReceived);
        parser.parse_chunk(String::from(input));
    }

    // https://html.spec.whatwg.org/multipage/#parsing-html-fragments
    pub fn parse_html_fragment(
            context_node: &Node,
            input: DOMString,
            output: &Node) {
        let window = window_from_node(context_node);
        let context_document = document_from_node(context_node);
        let url = context_document.url();

        // Step 1.
        let loader = DocumentLoader::new(&*context_document.loader());
        let document = Document::new(&window, None, Some(url.clone()),
                                     IsHTMLDocument::HTMLDocument,
                                     None, None,
                                     DocumentSource::FromParser,
                                     loader,
                                     None, None);

        // Step 2.
        document.set_quirks_mode(context_document.quirks_mode());

        // Step 11.
        let form = context_node.inclusive_ancestors()
                               .find(|element| element.is::<HTMLFormElement>());
        let fragment_context = FragmentContext {
            context_elem: context_node,
            form_elem: form.r(),
        };

        let parser = ServoParser::new(
            &document,
            None,
            Tokenizer::Html(
                self::html::Tokenizer::new(&document, url.clone(), Some(fragment_context))),
            LastChunkState::Received);
        parser.parse_chunk(String::from(input));

        // Step 14.
        let root_element = document.GetDocumentElement().expect("no document element");
        for child in root_element.upcast::<Node>().children() {
            output.AppendChild(&child).unwrap();
        }
    }

    pub fn parse_xml_document(
            document: &Document,
            input: DOMString,
            url: ServoUrl,
            owner: Option<PipelineId>) {
        let parser = ServoParser::new(
            document,
            owner,
            Tokenizer::Xml(self::xml::Tokenizer::new(document, url)),
            LastChunkState::NotReceived);
        parser.parse_chunk(String::from(input));
    }

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
            pending_input: DOMRefCell::new(BufferQueue::new()),
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
        self.pending_input.borrow_mut().push_back(chunk.into());
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
            if let Err(script) = self.tokenizer.borrow_mut().feed(&mut *self.pending_input.borrow_mut()) {
                if script.prepare() {
                    continue;
                }
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

#[derive(HeapSizeOf, JSTraceable)]
#[must_root]
enum Tokenizer {
    Html(self::html::Tokenizer),
    Xml(self::xml::Tokenizer),
}

impl Tokenizer {
    fn feed(&mut self, input: &mut BufferQueue) -> Result<(), Root<HTMLScriptElement>> {
        match *self {
            Tokenizer::Html(ref mut tokenizer) => tokenizer.feed(input),
            Tokenizer::Xml(ref mut tokenizer) => tokenizer.feed(input),
        }
    }

    fn end(&mut self) {
        match *self {
            Tokenizer::Html(ref mut tokenizer) => tokenizer.end(),
            Tokenizer::Xml(ref mut tokenizer) => tokenizer.end(),
        }
    }

    fn set_plaintext_state(&mut self) {
        match *self {
            Tokenizer::Html(ref mut tokenizer) => tokenizer.set_plaintext_state(),
            Tokenizer::Xml(_) => unimplemented!(),
        }
    }

    fn profiler_category(&self) -> ProfilerCategory {
        match *self {
            Tokenizer::Html(_) => ProfilerCategory::ScriptParseHTML,
            Tokenizer::Xml(_) => ProfilerCategory::ScriptParseXML,
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
    url: ServoUrl,
}

impl ParserContext {
    pub fn new(id: PipelineId, url: ServoUrl) -> ParserContext {
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

pub struct FragmentContext<'a> {
    pub context_elem: &'a Node,
    pub form_elem: Option<&'a Node>,
}
