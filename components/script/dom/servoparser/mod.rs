/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use document_loader::{DocumentLoader, LoadType};
use dom::bindings::cell::DOMRefCell;
use dom::bindings::codegen::Bindings::DocumentBinding::{DocumentMethods, DocumentReadyState};
use dom::bindings::codegen::Bindings::HTMLImageElementBinding::HTMLImageElementMethods;
use dom::bindings::codegen::Bindings::NodeBinding::NodeMethods;
use dom::bindings::codegen::Bindings::ServoParserBinding;
use dom::bindings::inheritance::Castable;
use dom::bindings::js::{JS, Root, RootedReference};
use dom::bindings::refcounted::Trusted;
use dom::bindings::reflector::{Reflector, reflect_dom_object};
use dom::bindings::str::DOMString;
use dom::characterdata::CharacterData;
use dom::document::{Document, DocumentSource, HasBrowsingContext, IsHTMLDocument};
use dom::element::Element;
use dom::globalscope::GlobalScope;
use dom::htmlformelement::HTMLFormElement;
use dom::htmlimageelement::HTMLImageElement;
use dom::htmlscriptelement::{HTMLScriptElement, ScriptResult};
use dom::node::{Node, NodeSiblingIterator};
use dom::text::Text;
use dom_struct::dom_struct;
use encoding::all::UTF_8;
use encoding::types::{DecoderTrap, Encoding};
use html5ever::tokenizer::buffer_queue::BufferQueue;
use html5ever::tree_builder::NodeOrText;
use hyper::header::ContentType;
use hyper::mime::{Mime, SubLevel, TopLevel};
use hyper_serde::Serde;
use msg::constellation_msg::PipelineId;
use net_traits::{FetchMetadata, FetchResponseListener, Metadata, NetworkError};
use network_listener::PreInvoke;
use profile_traits::time::{TimerMetadata, TimerMetadataFrameType};
use profile_traits::time::{TimerMetadataReflowType, ProfilerCategory, profile};
use script_thread::ScriptThread;
use script_traits::DocumentActivity;
use servo_config::resource_files::read_resource_file;
use servo_url::ServoUrl;
use std::ascii::AsciiExt;
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
    /// https://html.spec.whatwg.org/multipage/#abort-a-parser
    aborted: Cell<bool>,
    /// https://html.spec.whatwg.org/multipage/#script-created-parser
    script_created_parser: bool,
}

#[derive(PartialEq)]
enum LastChunkState {
    Received,
    NotReceived,
}

impl ServoParser {
    pub fn parse_html_document(document: &Document, input: DOMString, url: ServoUrl) {
        let parser = ServoParser::new(document,
                                      Tokenizer::Html(self::html::Tokenizer::new(document, url, None)),
                                      LastChunkState::NotReceived,
                                      ParserKind::Normal);
        parser.parse_chunk(String::from(input));
    }

    // https://html.spec.whatwg.org/multipage/#parsing-html-fragments
    pub fn parse_html_fragment(context: &Element, input: DOMString) -> FragmentParsingResult {
        let context_node = context.upcast::<Node>();
        let context_document = context_node.owner_doc();
        let window = context_document.window();
        let url = context_document.url();

        // Step 1.
        let loader = DocumentLoader::new_with_threads(context_document.loader().resource_threads().clone(),
                                                      Some(url.clone()));
        let document = Document::new(window,
                                     HasBrowsingContext::No,
                                     Some(url.clone()),
                                     context_document.origin().clone(),
                                     IsHTMLDocument::HTMLDocument,
                                     None,
                                     None,
                                     DocumentActivity::Inactive,
                                     DocumentSource::FromParser,
                                     loader,
                                     None,
                                     None);

        // Step 2.
        document.set_quirks_mode(context_document.quirks_mode());

        // Step 11.
        let form = context_node.inclusive_ancestors()
            .find(|element| element.is::<HTMLFormElement>());
        let fragment_context = FragmentContext {
            context_elem: context_node,
            form_elem: form.r(),
        };

        let parser = ServoParser::new(&document,
                                      Tokenizer::Html(self::html::Tokenizer::new(&document,
                                                                                 url.clone(),
                                                                                 Some(fragment_context))),
                                      LastChunkState::Received,
                                      ParserKind::Normal);
        parser.parse_chunk(String::from(input));

        // Step 14.
        let root_element = document.GetDocumentElement().expect("no document element");
        FragmentParsingResult {
            inner: root_element.upcast::<Node>().children(),
        }
    }

    pub fn parse_html_script_input(document: &Document, url: ServoUrl, type_: &str) {
        let parser = ServoParser::new(document,
                                      Tokenizer::Html(self::html::Tokenizer::new(document, url, None)),
                                      LastChunkState::NotReceived,
                                      ParserKind::ScriptCreated);
        document.set_current_parser(Some(&parser));
        if !type_.eq_ignore_ascii_case("text/html") {
            parser.parse_chunk("<pre>\n".to_owned());
            parser.tokenizer.borrow_mut().set_plaintext_state();
        }
    }

    pub fn parse_xml_document(document: &Document, input: DOMString, url: ServoUrl) {
        let parser = ServoParser::new(document,
                                      Tokenizer::Xml(self::xml::Tokenizer::new(document, url)),
                                      LastChunkState::NotReceived,
                                      ParserKind::Normal);
        parser.parse_chunk(String::from(input));
    }

    pub fn script_nesting_level(&self) -> usize {
        self.script_nesting_level.get()
    }

    pub fn is_script_created(&self) -> bool {
        self.script_created_parser
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
    pub fn resume_with_pending_parsing_blocking_script(&self, script: &HTMLScriptElement, result: ScriptResult) {
        assert!(self.suspended.get());
        self.suspended.set(false);

        mem::swap(&mut *self.script_input.borrow_mut(),
                  &mut *self.network_input.borrow_mut());
        while let Some(chunk) = self.script_input.borrow_mut().pop_front() {
            self.network_input.borrow_mut().push_back(chunk);
        }

        let script_nesting_level = self.script_nesting_level.get();
        assert_eq!(script_nesting_level, 0);

        self.script_nesting_level.set(script_nesting_level + 1);
        script.execute(result);
        self.script_nesting_level.set(script_nesting_level);

        if !self.suspended.get() {
            self.parse_sync();
        }
    }

    pub fn can_write(&self) -> bool {
        self.script_created_parser || self.script_nesting_level.get() > 0
    }

    /// Steps 6-8 of https://html.spec.whatwg.org/multipage/#document.write()
    pub fn write(&self, text: Vec<DOMString>) {
        assert!(self.can_write());

        if self.document.has_pending_parsing_blocking_script() {
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

    // Steps 4-6 of https://html.spec.whatwg.org/multipage/#dom-document-close
    pub fn close(&self) {
        assert!(self.script_created_parser);

        // Step 4.
        self.last_chunk_received.set(true);

        if self.suspended.get() {
            // Step 5.
            return;
        }

        // Step 6.
        self.parse_sync();
    }

    // https://html.spec.whatwg.org/multipage/#abort-a-parser
    pub fn abort(&self) {
        assert!(!self.aborted.get());
        self.aborted.set(true);

        // Step 1.
        *self.script_input.borrow_mut() = BufferQueue::new();
        *self.network_input.borrow_mut() = BufferQueue::new();

        // Step 2.
        self.document.set_ready_state(DocumentReadyState::Interactive);

        // Step 3.
        self.tokenizer.borrow_mut().end();
        self.document.set_current_parser(None);

        // Step 4.
        self.document.set_ready_state(DocumentReadyState::Interactive);
    }

    #[allow(unrooted_must_root)]
    fn new_inherited(document: &Document,
                     tokenizer: Tokenizer,
                     last_chunk_state: LastChunkState,
                     kind: ParserKind)
                     -> Self {
        ServoParser {
            reflector: Reflector::new(),
            document: JS::from_ref(document),
            network_input: DOMRefCell::new(BufferQueue::new()),
            script_input: DOMRefCell::new(BufferQueue::new()),
            tokenizer: DOMRefCell::new(tokenizer),
            last_chunk_received: Cell::new(last_chunk_state == LastChunkState::Received),
            suspended: Default::default(),
            script_nesting_level: Default::default(),
            aborted: Default::default(),
            script_created_parser: kind == ParserKind::ScriptCreated,
        }
    }

    #[allow(unrooted_must_root)]
    fn new(document: &Document,
           tokenizer: Tokenizer,
           last_chunk_state: LastChunkState,
           kind: ParserKind)
           -> Root<Self> {
        reflect_dom_object(box ServoParser::new_inherited(document, tokenizer, last_chunk_state, kind),
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
        where F: FnMut(&mut Tokenizer) -> Result<(), Root<HTMLScriptElement>>,
    {
        loop {
            assert!(!self.suspended.get());
            assert!(!self.aborted.get());

            self.document.reflow_if_reflow_timer_expired();
            let script = match feed(&mut *self.tokenizer.borrow_mut()) {
                Ok(()) => return,
                Err(script) => script,
            };

            let script_nesting_level = self.script_nesting_level.get();

            self.script_nesting_level.set(script_nesting_level + 1);
            script.prepare();
            self.script_nesting_level.set(script_nesting_level);

            if self.document.has_pending_parsing_blocking_script() {
                self.suspended.set(true);
                return;
            }
        }
    }

    // https://html.spec.whatwg.org/multipage/#the-end
    fn finish(&self) {
        assert!(!self.suspended.get());
        assert!(self.last_chunk_received.get());
        assert!(self.script_input.borrow().is_empty());
        assert!(self.network_input.borrow().is_empty());

        // Step 1.
        self.document.set_ready_state(DocumentReadyState::Interactive);

        // Step 2.
        self.tokenizer.borrow_mut().end();
        self.document.set_current_parser(None);

        // Steps 3-12 are in another castle, namely finish_load.
        let url = self.tokenizer.borrow().url().clone();
        self.document.finish_load(LoadType::PageSource(url));
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

#[derive(HeapSizeOf, JSTraceable, PartialEq)]
enum ParserKind {
    Normal,
    ScriptCreated,
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

    fn url(&self) -> &ServoUrl {
        match *self {
            Tokenizer::Html(ref tokenizer) => tokenizer.url(),
            Tokenizer::Xml(ref tokenizer) => tokenizer.url(),
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
#[derive(JSTraceable)]
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

    fn process_response(&mut self, meta_result: Result<FetchMetadata, NetworkError>) {
        let mut ssl_error = None;
        let mut network_error = None;
        let metadata = match meta_result {
            Ok(meta) => {
                Some(match meta {
                    FetchMetadata::Unfiltered(m) => m,
                    FetchMetadata::Filtered { unsafe_, .. } => unsafe_,
                })
            },
            Err(NetworkError::SslValidation(url, reason)) => {
                ssl_error = Some(reason);
                let mut meta = Metadata::default(url);
                let mime: Option<Mime> = "text/html".parse().ok();
                meta.set_content_type(mime.as_ref());
                Some(meta)
            },
            Err(NetworkError::Internal(reason)) => {
                network_error = Some(reason);
                let mut meta = Metadata::default(self.url.clone());
                let mime: Option<Mime> = "text/html".parse().ok();
                meta.set_content_type(mime.as_ref());
                Some(meta)
            },
            Err(_) => None,
        };
        let content_type = metadata.clone().and_then(|meta| meta.content_type).map(Serde::into_inner);
        let parser = match ScriptThread::page_headers_available(&self.id, metadata) {
            Some(parser) => parser,
            None => return,
        };
        if parser.aborted.get() {
            return;
        }

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
            Some(ContentType(Mime(TopLevel::Text, SubLevel::Html, _))) => {
                // Handle text/html
                if let Some(reason) = ssl_error {
                    self.is_synthesized_document = true;
                    let page_bytes = read_resource_file("badcert.html").unwrap();
                    let page = String::from_utf8(page_bytes).unwrap();
                    let page = page.replace("${reason}", &reason);
                    parser.push_input_chunk(page);
                    parser.parse_sync();
                }
                if let Some(reason) = network_error {
                    self.is_synthesized_document = true;
                    let page_bytes = read_resource_file("neterror.html").unwrap();
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
                                   toplevel.as_str(),
                                   sublevel.as_str());
                self.is_synthesized_document = true;
                parser.push_input_chunk(page);
                parser.parse_sync();
            },
            None => {
                // No content-type header.
                // Merge with #4212 when fixed.
            },
        }
    }

    fn process_response_chunk(&mut self, payload: Vec<u8>) {
        if self.is_synthesized_document {
            return;
        }
        // FIXME: use Vec<u8> (html5ever #34)
        let data = UTF_8.decode(&payload, DecoderTrap::Replace).unwrap();
        let parser = match self.parser.as_ref() {
            Some(parser) => parser.root(),
            None => return,
        };
        if parser.aborted.get() {
            return;
        }
        parser.parse_chunk(data);
    }

    fn process_response_eof(&mut self, status: Result<(), NetworkError>) {
        let parser = match self.parser.as_ref() {
            Some(parser) => parser.root(),
            None => return,
        };
        if parser.aborted.get() {
            return;
        }

        if let Err(err) = status {
            // TODO(Savago): we should send a notification to callers #5463.
            debug!("Failed to load page URL {}, error: {:?}", self.url, err);
        }

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

#[allow(unrooted_must_root)]
fn insert(parent: &Node, reference_child: Option<&Node>, child: NodeOrText<JS<Node>>) {
    match child {
        NodeOrText::AppendNode(n) => {
            parent.InsertBefore(&n, reference_child).unwrap();
        },
        NodeOrText::AppendText(t) => {
            let text = reference_child
                .and_then(Node::GetPreviousSibling)
                .or_else(|| parent.GetLastChild())
                .and_then(Root::downcast::<Text>);

            if let Some(text) = text {
                text.upcast::<CharacterData>().append_data(&t);
            } else {
                let text = Text::new(String::from(t).into(), &parent.owner_doc());
                parent.InsertBefore(text.upcast(), reference_child).unwrap();
            }
        },
    }
}
