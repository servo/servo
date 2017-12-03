/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use document_loader::{DocumentLoader, LoadType};
use dom::bindings::cell::DomRefCell;
use dom::bindings::codegen::Bindings::DocumentBinding::{DocumentMethods, DocumentReadyState};
use dom::bindings::codegen::Bindings::HTMLImageElementBinding::HTMLImageElementMethods;
use dom::bindings::codegen::Bindings::HTMLTemplateElementBinding::HTMLTemplateElementMethods;
use dom::bindings::codegen::Bindings::NodeBinding::NodeMethods;
use dom::bindings::codegen::Bindings::ServoParserBinding;
use dom::bindings::inheritance::Castable;
use dom::bindings::refcounted::Trusted;
use dom::bindings::reflector::{Reflector, reflect_dom_object};
use dom::bindings::root::{Dom, DomRoot, MutNullableDom, RootedReference};
use dom::bindings::settings_stack::is_execution_stack_empty;
use dom::bindings::str::DOMString;
use dom::characterdata::CharacterData;
use dom::comment::Comment;
use dom::document::{Document, DocumentSource, HasBrowsingContext, IsHTMLDocument};
use dom::documenttype::DocumentType;
use dom::element::{Element, ElementCreator, CustomElementCreationMode};
use dom::globalscope::GlobalScope;
use dom::htmlformelement::{FormControlElementHelpers, HTMLFormElement};
use dom::htmlimageelement::HTMLImageElement;
use dom::htmlscriptelement::{HTMLScriptElement, ScriptResult};
use dom::htmltemplateelement::HTMLTemplateElement;
use dom::node::Node;
use dom::processinginstruction::ProcessingInstruction;
use dom::text::Text;
use dom::virtualmethods::vtable_for;
use dom_struct::dom_struct;
use html5ever::{Attribute, ExpandedName, LocalName, QualName};
use html5ever::buffer_queue::BufferQueue;
use html5ever::tendril::{StrTendril, ByteTendril, IncompleteUtf8};
use html5ever::tree_builder::{NodeOrText, TreeSink, NextParserState, QuirksMode, ElementFlags};
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
use servo_config::prefs::PREFS;
use servo_config::resource_files::read_resource_file;
use servo_url::ServoUrl;
use std::borrow::Cow;
use std::cell::Cell;
use std::mem;
use style::context::QuirksMode as ServoQuirksMode;

mod async_html;
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
    document: Dom<Document>,
    /// Input received from network.
    #[ignore_malloc_size_of = "Defined in html5ever"]
    network_input: DomRefCell<BufferQueue>,
    /// Part of an UTF-8 code point spanning input chunks
    #[ignore_malloc_size_of = "Defined in html5ever"]
    incomplete_utf8: DomRefCell<Option<IncompleteUtf8>>,
    /// Input received from script. Used only to support document.write().
    #[ignore_malloc_size_of = "Defined in html5ever"]
    script_input: DomRefCell<BufferQueue>,
    /// The tokenizer of this parser.
    tokenizer: DomRefCell<Tokenizer>,
    /// Whether to expect any further input from the associated network request.
    last_chunk_received: Cell<bool>,
    /// Whether this parser should avoid passing any further data to the tokenizer.
    suspended: Cell<bool>,
    /// <https://html.spec.whatwg.org/multipage/#script-nesting-level>
    script_nesting_level: Cell<usize>,
    /// <https://html.spec.whatwg.org/multipage/#abort-a-parser>
    aborted: Cell<bool>,
    /// <https://html.spec.whatwg.org/multipage/#script-created-parser>
    script_created_parser: bool,
}

#[derive(PartialEq)]
enum LastChunkState {
    Received,
    NotReceived,
}

pub struct ElementAttribute {
    name: QualName,
    value: DOMString
}

#[derive(Clone, Copy, JSTraceable, MallocSizeOf, PartialEq)]
pub enum ParsingAlgorithm {
    Normal,
    Fragment,
}

impl ElementAttribute {
    pub fn new(name: QualName, value: DOMString) -> ElementAttribute {
        ElementAttribute {
            name: name,
            value: value
        }
    }
}

impl ServoParser {
    pub fn parser_is_not_active(&self) -> bool {
        self.can_write() || self.tokenizer.try_borrow_mut().is_ok()
    }

    pub fn parse_html_document(document: &Document, input: DOMString, url: ServoUrl) {
        let parser = if PREFS.get("dom.servoparser.async_html_tokenizer.enabled").as_boolean().unwrap() {
            ServoParser::new(document,
                             Tokenizer::AsyncHtml(self::async_html::Tokenizer::new(document, url, None)),
                             LastChunkState::NotReceived,
                             ParserKind::Normal)
        } else {
            ServoParser::new(document,
                             Tokenizer::Html(self::html::Tokenizer::new(document, url, None, ParsingAlgorithm::Normal)),
                             LastChunkState::NotReceived,
                             ParserKind::Normal)
        };
        parser.parse_string_chunk(String::from(input));
    }

    // https://html.spec.whatwg.org/multipage/#parsing-html-fragments
    pub fn parse_html_fragment(context: &Element, input: DOMString) -> impl Iterator<Item=DomRoot<Node>> {
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
                                     None,
                                     Default::default());

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
                                                                                 url,
                                                                                 Some(fragment_context),
                                                                                 ParsingAlgorithm::Fragment)),
                                      LastChunkState::Received,
                                      ParserKind::Normal);
        parser.parse_string_chunk(String::from(input));

        // Step 14.
        let root_element = document.GetDocumentElement().expect("no document element");
        FragmentParsingResult {
            inner: root_element.upcast::<Node>().children(),
        }
    }

    pub fn parse_html_script_input(document: &Document, url: ServoUrl, type_: &str) {
        let parser = ServoParser::new(
            document,
            Tokenizer::Html(self::html::Tokenizer::new(
                document,
                url,
                None,
                ParsingAlgorithm::Normal,
            )),
            LastChunkState::NotReceived,
            ParserKind::ScriptCreated,
        );
        document.set_current_parser(Some(&parser));
        if !type_.eq_ignore_ascii_case("text/html") {
            parser.parse_string_chunk("<pre>\n".to_owned());
            parser.tokenizer.borrow_mut().set_plaintext_state();
        }
    }

    pub fn parse_xml_document(document: &Document, input: DOMString, url: ServoUrl) {
        let parser = ServoParser::new(document,
                                      Tokenizer::Xml(self::xml::Tokenizer::new(document, url)),
                                      LastChunkState::NotReceived,
                                      ParserKind::Normal);
        parser.parse_string_chunk(String::from(input));
    }

    pub fn script_nesting_level(&self) -> usize {
        self.script_nesting_level.get()
    }

    pub fn is_script_created(&self) -> bool {
        self.script_created_parser
    }

    /// Corresponds to the latter part of the "Otherwise" branch of the 'An end
    /// tag whose tag name is "script"' of
    /// <https://html.spec.whatwg.org/multipage/#parsing-main-incdata>
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
            document: Dom::from_ref(document),
            incomplete_utf8: DomRefCell::new(None),
            network_input: DomRefCell::new(BufferQueue::new()),
            script_input: DomRefCell::new(BufferQueue::new()),
            tokenizer: DomRefCell::new(tokenizer),
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
           -> DomRoot<Self> {
        reflect_dom_object(Box::new(ServoParser::new_inherited(document, tokenizer, last_chunk_state, kind)),
                           document.window(),
                           ServoParserBinding::Wrap)
    }

    fn push_bytes_input_chunk(&self, chunk: Vec<u8>) {
        let mut chunk = ByteTendril::from(&*chunk);
        let mut network_input = self.network_input.borrow_mut();
        let mut incomplete_utf8 = self.incomplete_utf8.borrow_mut();

        if let Some(mut incomplete) = incomplete_utf8.take() {
            let result = incomplete.try_complete(chunk, |s| network_input.push_back(s));
            match result {
                Err(()) => {
                    *incomplete_utf8 = Some(incomplete);
                    return
                }
                Ok(remaining) => {
                    chunk = remaining
                }
            }
        }

        *incomplete_utf8 = chunk.decode_utf8_lossy(|s| network_input.push_back(s));
    }

    fn push_string_input_chunk(&self, chunk: String) {
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

        if self.last_chunk_received.get() {
            if let Some(_) = self.incomplete_utf8.borrow_mut().take() {
                self.network_input.borrow_mut().push_back(StrTendril::from("\u{FFFD}"))
            }
        }
        self.tokenize(|tokenizer| tokenizer.feed(&mut *self.network_input.borrow_mut()));

        if self.suspended.get() {
            return;
        }

        assert!(self.network_input.borrow().is_empty());

        if self.last_chunk_received.get() {
            self.finish();
        }
    }

    fn parse_string_chunk(&self, input: String) {
        self.document.set_current_parser(Some(self));
        self.push_string_input_chunk(input);
        if !self.suspended.get() {
            self.parse_sync();
        }
    }

    fn parse_bytes_chunk(&self, input: Vec<u8>) {
        self.document.set_current_parser(Some(self));
        self.push_bytes_input_chunk(input);
        if !self.suspended.get() {
            self.parse_sync();
        }
    }

    fn tokenize<F>(&self, mut feed: F)
        where F: FnMut(&mut Tokenizer) -> Result<(), DomRoot<HTMLScriptElement>>,
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
        assert!(self.incomplete_utf8.borrow().is_none());

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

struct FragmentParsingResult<I>
    where I: Iterator<Item=DomRoot<Node>>
{
    inner: I,
}

impl<I> Iterator for FragmentParsingResult<I>
    where I: Iterator<Item=DomRoot<Node>>
{
    type Item = DomRoot<Node>;

    fn next(&mut self) -> Option<DomRoot<Node>> {
        let next = self.inner.next()?;
        next.remove_self();
        Some(next)
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        self.inner.size_hint()
    }
}

#[derive(JSTraceable, MallocSizeOf, PartialEq)]
enum ParserKind {
    Normal,
    ScriptCreated,
}

#[derive(JSTraceable, MallocSizeOf)]
#[must_root]
enum Tokenizer {
    Html(self::html::Tokenizer),
    AsyncHtml(self::async_html::Tokenizer),
    Xml(self::xml::Tokenizer),
}

impl Tokenizer {
    fn feed(&mut self, input: &mut BufferQueue) -> Result<(), DomRoot<HTMLScriptElement>> {
        match *self {
            Tokenizer::Html(ref mut tokenizer) => tokenizer.feed(input),
            Tokenizer::AsyncHtml(ref mut tokenizer) => tokenizer.feed(input),
            Tokenizer::Xml(ref mut tokenizer) => tokenizer.feed(input),
        }
    }

    fn end(&mut self) {
        match *self {
            Tokenizer::Html(ref mut tokenizer) => tokenizer.end(),
            Tokenizer::AsyncHtml(ref mut tokenizer) => tokenizer.end(),
            Tokenizer::Xml(ref mut tokenizer) => tokenizer.end(),
        }
    }

    fn url(&self) -> &ServoUrl {
        match *self {
            Tokenizer::Html(ref tokenizer) => tokenizer.url(),
            Tokenizer::AsyncHtml(ref tokenizer) => tokenizer.url(),
            Tokenizer::Xml(ref tokenizer) => tokenizer.url(),
        }
    }

    fn set_plaintext_state(&mut self) {
        match *self {
            Tokenizer::Html(ref mut tokenizer) => tokenizer.set_plaintext_state(),
            Tokenizer::AsyncHtml(ref mut tokenizer) => tokenizer.set_plaintext_state(),
            Tokenizer::Xml(_) => unimplemented!(),
        }
    }

    fn profiler_category(&self) -> ProfilerCategory {
        match *self {
            Tokenizer::Html(_) => ProfilerCategory::ScriptParseHTML,
            Tokenizer::AsyncHtml(_) => ProfilerCategory::ScriptParseHTML,
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
                parser.push_string_input_chunk(page);
                parser.parse_sync();

                let doc = &parser.document;
                let doc_body = DomRoot::upcast::<Node>(doc.GetBody().unwrap());
                let img = HTMLImageElement::new(local_name!("img"), None, doc);
                img.SetSrc(DOMString::from(self.url.to_string()));
                doc_body.AppendChild(&DomRoot::upcast::<Node>(img)).expect("Appending failed");

            },
            Some(ContentType(Mime(TopLevel::Text, SubLevel::Plain, _))) => {
                // https://html.spec.whatwg.org/multipage/#read-text
                let page = "<pre>\n".into();
                parser.push_string_input_chunk(page);
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
                    parser.push_string_input_chunk(page);
                    parser.parse_sync();
                }
                if let Some(reason) = network_error {
                    self.is_synthesized_document = true;
                    let page_bytes = read_resource_file("neterror.html").unwrap();
                    let page = String::from_utf8(page_bytes).unwrap();
                    let page = page.replace("${reason}", &reason);
                    parser.push_string_input_chunk(page);
                    parser.parse_sync();
                }
            },
            Some(ContentType(Mime(TopLevel::Text, SubLevel::Xml, _))) | // Handle text/xml, application/xml
            Some(ContentType(Mime(TopLevel::Application, SubLevel::Xml, _))) => {},
            Some(ContentType(Mime(TopLevel::Application, SubLevel::Ext(ref sub), _)))
                if sub.as_str() == "xhtml+xml".to_owned() => {}, // Handle xhtml (application/xhtml+xml)
            Some(ContentType(Mime(toplevel, sublevel, _))) => {
                // Show warning page for unknown mime types.
                let page = format!("<html><body><p>Unknown content type ({}/{}).</p></body></html>",
                                   toplevel.as_str(),
                                   sublevel.as_str());
                self.is_synthesized_document = true;
                parser.push_string_input_chunk(page);
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
        let parser = match self.parser.as_ref() {
            Some(parser) => parser.root(),
            None => return,
        };
        if parser.aborted.get() {
            return;
        }
        parser.parse_bytes_chunk(payload);
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
fn insert(parent: &Node, reference_child: Option<&Node>, child: NodeOrText<Dom<Node>>) {
    match child {
        NodeOrText::AppendNode(n) => {
            parent.InsertBefore(&n, reference_child).unwrap();
        },
        NodeOrText::AppendText(t) => {
            let text = reference_child
                .and_then(Node::GetPreviousSibling)
                .or_else(|| parent.GetLastChild())
                .and_then(DomRoot::downcast::<Text>);

            if let Some(text) = text {
                text.upcast::<CharacterData>().append_data(&t);
            } else {
                let text = Text::new(String::from(t).into(), &parent.owner_doc());
                parent.InsertBefore(text.upcast(), reference_child).unwrap();
            }
        },
    }
}

#[derive(JSTraceable, MallocSizeOf)]
#[must_root]
pub struct Sink {
    base_url: ServoUrl,
    document: Dom<Document>,
    current_line: u64,
    script: MutNullableDom<HTMLScriptElement>,
    parsing_algorithm: ParsingAlgorithm,
}

impl Sink {
    fn same_tree(&self, x: &Dom<Node>, y: &Dom<Node>) -> bool {
        let x = x.downcast::<Element>().expect("Element node expected");
        let y = y.downcast::<Element>().expect("Element node expected");

        x.is_in_same_home_subtree(y)
    }

    fn has_parent_node(&self, node: &Dom<Node>) -> bool {
         node.GetParentNode().is_some()
    }
}

#[allow(unrooted_must_root)]  // FIXME: really?
impl TreeSink for Sink {
    type Output = Self;
    fn finish(self) -> Self { self }

    type Handle = Dom<Node>;

    fn get_document(&mut self) -> Dom<Node> {
        Dom::from_ref(self.document.upcast())
    }

    fn get_template_contents(&mut self, target: &Dom<Node>) -> Dom<Node> {
        let template = target.downcast::<HTMLTemplateElement>()
            .expect("tried to get template contents of non-HTMLTemplateElement in HTML parsing");
        Dom::from_ref(template.Content().upcast())
    }

    fn same_node(&self, x: &Dom<Node>, y: &Dom<Node>) -> bool {
        x == y
    }

    fn elem_name<'a>(&self, target: &'a Dom<Node>) -> ExpandedName<'a> {
        let elem = target.downcast::<Element>()
            .expect("tried to get name of non-Element in HTML parsing");
        ExpandedName {
            ns: elem.namespace(),
            local: elem.local_name(),
        }
    }

    fn create_element(&mut self, name: QualName, attrs: Vec<Attribute>, _flags: ElementFlags)
            -> Dom<Node> {
        let attrs = attrs
            .into_iter()
            .map(|attr| ElementAttribute::new(attr.name, DOMString::from(String::from(attr.value))))
            .collect();
        let element = create_element_for_token(
            name,
            attrs,
            &*self.document,
            ElementCreator::ParserCreated(self.current_line),
            self.parsing_algorithm,
        );
        Dom::from_ref(element.upcast())
    }

    fn create_comment(&mut self, text: StrTendril) -> Dom<Node> {
        let comment = Comment::new(DOMString::from(String::from(text)), &*self.document);
        Dom::from_ref(comment.upcast())
    }

    fn create_pi(&mut self, target: StrTendril, data: StrTendril) -> Dom<Node> {
        let doc = &*self.document;
        let pi = ProcessingInstruction::new(
            DOMString::from(String::from(target)), DOMString::from(String::from(data)),
            doc);
        Dom::from_ref(pi.upcast())
    }

    fn associate_with_form(&mut self, target: &Dom<Node>, form: &Dom<Node>, nodes: (&Dom<Node>, Option<&Dom<Node>>)) {
        let (element, prev_element) = nodes;
        let tree_node = prev_element.map_or(element, |prev| {
            if self.has_parent_node(element) { element } else { prev }
        });
        if !self.same_tree(tree_node, form) {
            return;
        }

        let node = target;
        let form = DomRoot::downcast::<HTMLFormElement>(DomRoot::from_ref(&**form))
            .expect("Owner must be a form element");

        let elem = node.downcast::<Element>();
        let control = elem.and_then(|e| e.as_maybe_form_control());

        if let Some(control) = control {
            control.set_form_owner_from_parser(&form);
        } else {
            // TODO remove this code when keygen is implemented.
            assert!(node.NodeName() == "KEYGEN", "Unknown form-associatable element");
        }
    }

    fn append_before_sibling(&mut self,
            sibling: &Dom<Node>,
            new_node: NodeOrText<Dom<Node>>) {
        let parent = sibling.GetParentNode()
            .expect("append_before_sibling called on node without parent");

        insert(&parent, Some(&*sibling), new_node);
    }

    fn parse_error(&mut self, msg: Cow<'static, str>) {
        debug!("Parse error: {}", msg);
    }

    fn set_quirks_mode(&mut self, mode: QuirksMode) {
        let mode = match mode {
            QuirksMode::Quirks => ServoQuirksMode::Quirks,
            QuirksMode::LimitedQuirks => ServoQuirksMode::LimitedQuirks,
            QuirksMode::NoQuirks => ServoQuirksMode::NoQuirks,
        };
        self.document.set_quirks_mode(mode);
    }

    fn append(&mut self, parent: &Dom<Node>, child: NodeOrText<Dom<Node>>) {
        insert(&parent, None, child);
    }

    fn append_based_on_parent_node(
        &mut self,
        elem: &Dom<Node>,
        prev_elem: &Dom<Node>,
        child: NodeOrText<Dom<Node>>,
    ) {
        if self.has_parent_node(elem) {
            self.append_before_sibling(elem, child);
        } else {
            self.append(prev_elem, child);
        }
    }

    fn append_doctype_to_document(&mut self, name: StrTendril, public_id: StrTendril,
                                  system_id: StrTendril) {
        let doc = &*self.document;
        let doctype = DocumentType::new(
            DOMString::from(String::from(name)), Some(DOMString::from(String::from(public_id))),
            Some(DOMString::from(String::from(system_id))), doc);
        doc.upcast::<Node>().AppendChild(doctype.upcast()).expect("Appending failed");
    }

    fn add_attrs_if_missing(&mut self, target: &Dom<Node>, attrs: Vec<Attribute>) {
        let elem = target.downcast::<Element>()
            .expect("tried to set attrs on non-Element in HTML parsing");
        for attr in attrs {
            elem.set_attribute_from_parser(attr.name, DOMString::from(String::from(attr.value)), None);
        }
    }

    fn remove_from_parent(&mut self, target: &Dom<Node>) {
        if let Some(ref parent) = target.GetParentNode() {
            parent.RemoveChild(&*target).unwrap();
        }
    }

    fn mark_script_already_started(&mut self, node: &Dom<Node>) {
        let script = node.downcast::<HTMLScriptElement>();
        script.map(|script| script.set_already_started(true));
    }

    fn complete_script(&mut self, node: &Dom<Node>) -> NextParserState {
        if let Some(script) = node.downcast() {
            self.script.set(Some(script));
            NextParserState::Suspend
        } else {
            NextParserState::Continue
        }
    }

    fn reparent_children(&mut self, node: &Dom<Node>, new_parent: &Dom<Node>) {
        while let Some(ref child) = node.GetFirstChild() {
            new_parent.AppendChild(&child).unwrap();
        }
    }

    /// <https://html.spec.whatwg.org/multipage/#html-integration-point>
    /// Specifically, the <annotation-xml> cases.
    fn is_mathml_annotation_xml_integration_point(&self, handle: &Dom<Node>) -> bool {
        let elem = handle.downcast::<Element>().unwrap();
        elem.get_attribute(&ns!(), &local_name!("encoding")).map_or(false, |attr| {
            attr.value().eq_ignore_ascii_case("text/html")
                || attr.value().eq_ignore_ascii_case("application/xhtml+xml")
        })
    }

    fn set_current_line(&mut self, line_number: u64) {
        self.current_line = line_number;
    }

    fn pop(&mut self, node: &Dom<Node>) {
        let node = DomRoot::from_ref(&**node);
        vtable_for(&node).pop();
    }
}

/// https://html.spec.whatwg.org/multipage/#create-an-element-for-the-token
fn create_element_for_token(
    name: QualName,
    attrs: Vec<ElementAttribute>,
    document: &Document,
    creator: ElementCreator,
    parsing_algorithm: ParsingAlgorithm,
) -> DomRoot<Element> {
    // Step 3.
    let is = attrs.iter()
        .find(|attr| attr.name.local.eq_str_ignore_ascii_case("is"))
        .map(|attr| LocalName::from(&*attr.value));

    // Step 4.
    let definition = document.lookup_custom_element_definition(&name.ns, &name.local, is.as_ref());

    // Step 5.
    let will_execute_script = definition.is_some() && parsing_algorithm != ParsingAlgorithm::Fragment;

    // Step 6.
    if will_execute_script {
        // Step 6.1.
        document.increment_throw_on_dynamic_markup_insertion_counter();
        // Step 6.2
        if is_execution_stack_empty() {
            document.window().upcast::<GlobalScope>().perform_a_microtask_checkpoint();
        }
        // Step 6.3
        ScriptThread::push_new_element_queue()
    }

    // Step 7.
    let creation_mode = if will_execute_script {
        CustomElementCreationMode::Synchronous
    } else {
        CustomElementCreationMode::Asynchronous
    };
    let element = Element::create(name, is, document, creator, creation_mode);

    // Step 8.
    for attr in attrs {
        element.set_attribute_from_parser(attr.name, attr.value, None);
    }

    // Step 9.
    if will_execute_script {
        // Steps 9.1 - 9.2.
        ScriptThread::pop_current_element_queue();
        // Step 9.3.
        document.decrement_throw_on_dynamic_markup_insertion_counter();
    }

    // TODO: Step 10.
    // TODO: Step 11.

    // Step 12 is handled in `associate_with_form`.

    // Step 13.
    element
}
