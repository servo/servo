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
use dom::element::Element;
use dom::globalscope::GlobalScope;
use dom::htmlformelement::HTMLFormElement;
use dom::htmlimageelement::HTMLImageElement;
use dom::htmlscriptelement::HTMLScriptElement;
use dom::node::{Node, NodeSiblingIterator};
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
use servo_config::resource_files::read_resource_file;
use servo_url::ServoUrl;
use std::cell::Cell;
use std::mem;

mod html;
mod xml;

#[dom_struct]
/// The parser maintains two input streams: one for input from script through
/// document.write(), and one for input from network.
///
/// There is no concrete representation of the insertion point, instead it
/// always points to just before the next character from the network input,
/// with all of the script input before itself.
///
/// ```text
///     ... script input ... | ... network input ...
///                          ^
///                 insertion point
/// ```
pub struct ServoParser {
    reflector: Reflector,
    /// The document associated with this parser.
    document: JS<Document>,
    /// The pipeline associated with this parse, unavailable if this parse
    /// does not correspond to a page load.
    pipeline: Option<PipelineId>,
    /// Input received from network.
    #[ignore_heap_size_of = "Defined in html5ever"]
    network_input: DOMRefCell<BufferQueue>,
    /// Input received from script. Used only to support document.write().
    #[ignore_heap_size_of = "Defined in html5ever"]
    script_input: DOMRefCell<BufferQueue>,
    /// The tokenizer of this parser.
    tokenizer: DOMRefCell<Tokenizer>,
    /// Whether to expect any further input from the associated network request.
    last_chunk_received: Cell<bool>,
    /// Whether this parser should avoid passing any further data to the tokenizer.
    suspended: Cell<bool>,
    /// https://html.spec.whatwg.org/multipage/#script-nesting-level
    script_nesting_level: Cell<usize>,
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
    pub fn parse_html_fragment(context: &Element, input: DOMString) -> FragmentParsingResult {
        let context_node = context.upcast::<Node>();
        let context_document = context_node.owner_doc();
        let window = context_document.window();
        let url = context_document.url();

        // Step 1.
        let loader = DocumentLoader::new(&*context_document.loader());
        let document = Document::new(window, None, Some(url.clone()),
                                     context_document.origin().alias(),
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
        FragmentParsingResult { inner: root_element.upcast::<Node>().children() }
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

    pub fn script_nesting_level(&self) -> usize {
        self.script_nesting_level.get()
    }

    /// Corresponds to the latter part of the "Otherwise" branch of the 'An end
    /// tag whose tag name is "script"' of
    /// https://html.spec.whatwg.org/multipage/#parsing-main-incdata
    ///
    /// This first moves everything from the script input to the beginning of
    /// the network input, effectively resetting the insertion point to just
    /// before the next character to be consumed.
    ///
    ///
    /// ```text
    ///     | ... script input ... network input ...
    ///     ^
    ///     insertion point
    /// ```
    pub fn resume_with_pending_parsing_blocking_script(&self, script: &HTMLScriptElement) {
        assert!(self.suspended.get());
        self.suspended.set(false);

        mem::swap(&mut *self.script_input.borrow_mut(), &mut *self.network_input.borrow_mut());
        while let Some(chunk) = self.script_input.borrow_mut().pop_front() {
            self.network_input.borrow_mut().push_back(chunk);
        }

        let script_nesting_level = self.script_nesting_level.get();
        assert_eq!(script_nesting_level, 0);

        self.script_nesting_level.set(script_nesting_level + 1);
        script.execute();
        self.script_nesting_level.set(script_nesting_level);

        if !self.suspended.get() {
            self.parse_sync();
        }
    }

    /// Steps 6-8 of https://html.spec.whatwg.org/multipage/#document.write()
    pub fn write(&self, text: Vec<DOMString>) {
        assert!(self.script_nesting_level.get() > 0);

        if self.document.get_pending_parsing_blocking_script().is_some() {
            // There is already a pending parsing blocking script so the
            // parser is suspended, we just append everything to the
            // script input and abort these steps.
            for chunk in text {
                self.script_input.borrow_mut().push_back(String::from(chunk).into());
            }
            return;
        }

        // There is no pending parsing blocking script, so all previous calls
        // to document.write() should have seen their entire input tokenized
        // and process, with nothing pushed to the parser script input.
        assert!(self.script_input.borrow().is_empty());

        let mut input = BufferQueue::new();
        for chunk in text {
            input.push_back(String::from(chunk).into());
        }

        self.tokenize(|tokenizer| tokenizer.feed(&mut input));

        if self.suspended.get() {
            // Parser got suspended, insert remaining input at end of
            // script input, following anything written by scripts executed
            // reentrantly during this call.
            while let Some(chunk) = input.pop_front() {
                self.script_input.borrow_mut().push_back(chunk);
            }
            return;
        }

        assert!(input.is_empty());
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
            network_input: DOMRefCell::new(BufferQueue::new()),
            script_input: DOMRefCell::new(BufferQueue::new()),
            tokenizer: DOMRefCell::new(tokenizer),
            last_chunk_received: Cell::new(last_chunk_state == LastChunkState::Received),
            suspended: Default::default(),
            script_nesting_level: Default::default(),
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

    fn push_input_chunk(&self, chunk: String) {
        self.network_input.borrow_mut().push_back(chunk.into());
    }

    fn parse_sync(&self) {
        let metadata = TimerMetadata {
            url: self.document.url().as_str().into(),
            iframe: TimerMetadataFrameType::RootWindow,
            incremental: TimerMetadataReflowType::FirstReflow,
        };
        let profiler_category = self.tokenizer.borrow().profiler_category();
        profile(profiler_category,
                Some(metadata),
                self.document.window().upcast::<GlobalScope>().time_profiler_chan().clone(),
                || self.do_parse_sync())
    }

    fn do_parse_sync(&self) {
        assert!(self.script_input.borrow().is_empty());

        // This parser will continue to parse while there is either pending input or
        // the parser remains unsuspended.

        self.tokenize(|tokenizer| tokenizer.feed(&mut *self.network_input.borrow_mut()));

        if self.suspended.get() {
            return;
        }

        assert!(self.network_input.borrow().is_empty());

        if self.last_chunk_received.get() {
            self.finish();
        }
    }

    fn parse_chunk(&self, input: String) {
        self.document.set_current_parser(Some(self));
        self.push_input_chunk(input);
        if !self.suspended.get() {
            self.parse_sync();
        }
    }

    fn tokenize<F>(&self, mut feed: F)
        where F: FnMut(&mut Tokenizer) -> Result<(), Root<HTMLScriptElement>>
    {
        loop {
            assert!(!self.suspended.get());

            self.document.reflow_if_reflow_timer_expired();
            let script = match feed(&mut *self.tokenizer.borrow_mut()) {
                Ok(()) => return,
                Err(script) => script,
            };

            let script_nesting_level = self.script_nesting_level.get();

            self.script_nesting_level.set(script_nesting_level + 1);
            script.prepare();
            self.script_nesting_level.set(script_nesting_level);

            if self.document.get_pending_parsing_blocking_script().is_some() {
                self.suspended.set(true);
                return;
            }
        }
    }

    fn finish(&self) {
        assert!(!self.suspended.get());
        assert!(self.last_chunk_received.get());
        assert!(self.script_input.borrow().is_empty());
        assert!(self.network_input.borrow().is_empty());

        self.tokenizer.borrow_mut().end();
        debug!("finished parsing");

        self.document.set_current_parser(None);

        if let Some(pipeline) = self.pipeline {
            ScriptThread::parsing_complete(pipeline);
        }
    }
}

pub struct FragmentParsingResult {
    inner: NodeSiblingIterator,
}

impl Iterator for FragmentParsingResult {
    type Item = Root<Node>;

    fn next(&mut self) -> Option<Root<Node>> {
        let next = match self.inner.next() {
            Some(next) => next,
            None => return None,
        };
        next.remove_self();
        Some(next)
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

                let doc = &parser.document;
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
                parser.tokenizer.borrow_mut().set_plaintext_state();
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

        parser.document
            .finish_load(LoadType::PageSource(self.url.clone()));

        parser.last_chunk_received.set(true);
        if !parser.suspended.get() {
            parser.parse_sync();
        }
    }
}

impl PreInvoke for ParserContext {}

pub struct FragmentContext<'a> {
    pub context_elem: &'a Node,
    pub form_elem: Option<&'a Node>,
}
