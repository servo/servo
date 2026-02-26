/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use std::borrow::Cow;
use std::cell::{Cell, RefCell};
use std::mem;
use std::rc::Rc;

use base::cross_process_instant::CrossProcessInstant;
use base::id::{PipelineId, WebViewId};
use base64::Engine as _;
use base64::engine::general_purpose;
use content_security_policy::sandboxing_directive::SandboxingFlagSet;
use devtools_traits::ScriptToDevtoolsControlMsg;
use dom_struct::dom_struct;
use embedder_traits::resources::{self, Resource};
use encoding_rs::{Encoding, UTF_8};
use html5ever::buffer_queue::BufferQueue;
use html5ever::tendril::StrTendril;
use html5ever::tree_builder::{ElementFlags, NodeOrText, QuirksMode, TreeSink};
use html5ever::{Attribute, ExpandedName, LocalName, QualName, local_name, ns};
use hyper_serde::Serde;
use markup5ever::TokenizerResult;
use mime::{self, Mime};
use net_traits::mime_classifier::{ApacheBugFlag, MediaType, MimeClassifier, NoSniffFlag};
use net_traits::policy_container::PolicyContainer;
use net_traits::request::RequestId;
use net_traits::{
    FetchMetadata, LoadContext, Metadata, NetworkError, ReferrerPolicy, ResourceFetchTiming,
};
use profile_traits::time::{
    ProfilerCategory, ProfilerChan, TimerMetadata, TimerMetadataFrameType, TimerMetadataReflowType,
};
use profile_traits::time_profile;
use script_bindings::script_runtime::temp_cx;
use script_traits::DocumentActivity;
use servo_config::pref;
use servo_url::ServoUrl;
use style::context::QuirksMode as ServoQuirksMode;
use tendril::stream::LossyDecoder;
use tendril::{ByteTendril, TendrilSink};

use crate::document_loader::{DocumentLoader, LoadType};
use crate::dom::bindings::cell::DomRefCell;
use crate::dom::bindings::codegen::Bindings::DocumentBinding::{
    DocumentMethods, DocumentReadyState,
};
use crate::dom::bindings::codegen::Bindings::HTMLImageElementBinding::HTMLImageElementMethods;
use crate::dom::bindings::codegen::Bindings::HTMLMediaElementBinding::HTMLMediaElementMethods;
use crate::dom::bindings::codegen::Bindings::HTMLTemplateElementBinding::HTMLTemplateElementMethods;
use crate::dom::bindings::codegen::Bindings::NodeBinding::NodeMethods;
use crate::dom::bindings::codegen::Bindings::ShadowRootBinding::{
    ShadowRootMode, SlotAssignmentMode,
};
use crate::dom::bindings::inheritance::Castable;
use crate::dom::bindings::refcounted::Trusted;
use crate::dom::bindings::reflector::{DomGlobal, Reflector, reflect_dom_object};
use crate::dom::bindings::root::{Dom, DomRoot, MutNullableDom};
use crate::dom::bindings::settings_stack::is_execution_stack_empty;
use crate::dom::bindings::str::{DOMString, USVString};
use crate::dom::characterdata::CharacterData;
use crate::dom::comment::Comment;
use crate::dom::csp::{GlobalCspReporting, Violation, parse_csp_list_from_metadata};
use crate::dom::customelementregistry::CustomElementReactionStack;
use crate::dom::document::{Document, DocumentSource, HasBrowsingContext, IsHTMLDocument};
use crate::dom::documentfragment::DocumentFragment;
use crate::dom::documenttype::DocumentType;
use crate::dom::element::{CustomElementCreationMode, Element, ElementCreator};
use crate::dom::globalscope::GlobalScope;
use crate::dom::html::htmlformelement::{FormControlElementHelpers, HTMLFormElement};
use crate::dom::html::htmlimageelement::HTMLImageElement;
use crate::dom::html::htmlscriptelement::{HTMLScriptElement, ScriptResult};
use crate::dom::html::htmltemplateelement::HTMLTemplateElement;
use crate::dom::node::{Node, ShadowIncluding};
use crate::dom::performance::performanceentry::PerformanceEntry;
use crate::dom::performance::performancenavigationtiming::PerformanceNavigationTiming;
use crate::dom::processinginstruction::ProcessingInstruction;
use crate::dom::processingoptions::{
    LinkHeader, LinkProcessingPhase, extract_links_from_headers, process_link_headers,
};
use crate::dom::reportingendpoint::ReportingEndpoint;
use crate::dom::shadowroot::IsUserAgentWidget;
use crate::dom::text::Text;
use crate::dom::types::{HTMLElement, HTMLMediaElement, HTMLOptionElement};
use crate::dom::virtualmethods::vtable_for;
use crate::network_listener::FetchResponseListener;
use crate::realms::{enter_auto_realm, enter_realm};
use crate::script_runtime::{CanGc, IntroductionType};
use crate::script_thread::ScriptThread;

mod async_html;
pub(crate) mod encoding;
pub(crate) mod html;
mod prefetch;
mod xml;

use encoding::{NetworkDecoderState, NetworkSink};
pub(crate) use html::serialize_html_fragment;

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
pub(crate) struct ServoParser {
    reflector: Reflector,
    /// The document associated with this parser.
    document: Dom<Document>,
    /// The decoder used for the network input.
    network_decoder: DomRefCell<NetworkDecoderState>,
    /// Input received from network.
    #[ignore_malloc_size_of = "Defined in html5ever"]
    #[no_trace]
    network_input: BufferQueue,
    /// Input received from script. Used only to support document.write().
    #[ignore_malloc_size_of = "Defined in html5ever"]
    #[no_trace]
    script_input: BufferQueue,
    /// The tokenizer of this parser.
    tokenizer: Tokenizer,
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
    /// A decoder exclusively for input to the prefetch tokenizer.
    ///
    /// Unlike the actual decoder, this one takes a best guess at the encoding and starts
    /// decoding immediately.
    #[no_trace]
    prefetch_decoder: RefCell<LossyDecoder<NetworkSink>>,
    /// We do a quick-and-dirty parse of the input looking for resources to prefetch.
    // TODO: if we had speculative parsing, we could do this when speculatively
    // building the DOM. https://github.com/servo/servo/pull/19203
    prefetch_tokenizer: prefetch::Tokenizer,
    #[ignore_malloc_size_of = "Defined in html5ever"]
    #[no_trace]
    prefetch_input: BufferQueue,
    // The whole input as a string, if needed for the devtools Sources panel.
    // TODO: use a faster type for concatenating strings?
    content_for_devtools: Option<DomRefCell<String>>,
}

pub(crate) struct ElementAttribute {
    name: QualName,
    value: DOMString,
}

#[derive(Clone, Copy, JSTraceable, MallocSizeOf, PartialEq)]
pub(crate) enum ParsingAlgorithm {
    Normal,
    Fragment,
}

impl ElementAttribute {
    pub(crate) fn new(name: QualName, value: DOMString) -> ElementAttribute {
        ElementAttribute { name, value }
    }
}

impl ServoParser {
    pub(crate) fn parser_is_not_active(&self) -> bool {
        self.can_write()
    }

    /// <https://html.spec.whatwg.org/multipage/#parse-html-from-a-string>
    pub(crate) fn parse_html_document(
        document: &Document,
        input: Option<DOMString>,
        url: ServoUrl,
        encoding_hint_from_content_type: Option<&'static Encoding>,
        encoding_of_container_document: Option<&'static Encoding>,
        cx: &mut js::context::JSContext,
    ) {
        // Step 1. Set document's type to "html".
        //
        // Set by callers of this function and asserted here
        assert!(document.is_html_document());

        // Step 2. Create an HTML parser parser, associated with document.
        let parser = ServoParser::new(
            document,
            if pref!(dom_servoparser_async_html_tokenizer_enabled) {
                Tokenizer::AsyncHtml(self::async_html::Tokenizer::new(document, url, None))
            } else {
                Tokenizer::Html(self::html::Tokenizer::new(
                    document,
                    url,
                    None,
                    ParsingAlgorithm::Normal,
                ))
            },
            ParserKind::Normal,
            encoding_hint_from_content_type,
            encoding_of_container_document,
            CanGc::from_cx(cx),
        );

        // Step 3. Place html into the input stream for parser. The encoding confidence is irrelevant.
        // Step 4. Start parser and let it run until it has consumed all the
        // characters just inserted into the input stream.
        //
        // Set as the document's current parser and initialize with `input`, if given.
        if let Some(input) = input {
            parser.parse_complete_string_chunk(String::from(input), cx);
        } else {
            parser.document.set_current_parser(Some(&parser));
        }
    }

    /// <https://html.spec.whatwg.org/multipage/#parsing-html-fragments>
    pub(crate) fn parse_html_fragment<'el>(
        context: &'el Element,
        input: DOMString,
        allow_declarative_shadow_roots: bool,
        cx: &mut js::context::JSContext,
    ) -> impl Iterator<Item = DomRoot<Node>> + use<'el> {
        let context_node = context.upcast::<Node>();
        let context_document = context_node.owner_doc();
        let window = context_document.window();
        let url = context_document.url();

        // Step 1. Let document be a Document node whose type is "html".
        let loader = DocumentLoader::new_with_threads(
            context_document.loader().resource_threads().clone(),
            Some(url.clone()),
        );
        let document = Document::new(
            window,
            HasBrowsingContext::No,
            Some(url.clone()),
            context_document.about_base_url(),
            context_document.origin().clone(),
            IsHTMLDocument::HTMLDocument,
            None,
            None,
            DocumentActivity::Inactive,
            DocumentSource::FromParser,
            loader,
            None,
            None,
            Default::default(),
            false,
            allow_declarative_shadow_roots,
            Some(context_document.insecure_requests_policy()),
            context_document.has_trustworthy_ancestor_or_current_origin(),
            context_document.custom_element_reaction_stack(),
            context_document.creation_sandboxing_flag_set(),
            CanGc::from_cx(cx),
        );

        // Step 2. If context's node document is in quirks mode, then set document's mode to "quirks".
        // Step 3. Otherwise, if context's node document is in limited-quirks mode, then set document's
        // mode to "limited-quirks".
        document.set_quirks_mode(context_document.quirks_mode());

        // NOTE: The following steps happened as part of Step 1.
        // Step 4. If allowDeclarativeShadowRoots is true, then set document's
        // allow declarative shadow roots to true.
        // Step 5. Create a new HTML parser, and associate it with document.

        // Step 11.
        let form = context_node
            .inclusive_ancestors(ShadowIncluding::No)
            .find(|element| element.is::<HTMLFormElement>());

        let fragment_context = FragmentContext {
            context_elem: context_node,
            form_elem: form.as_deref(),
            context_element_allows_scripting: context_document.scripting_enabled(),
        };

        let parser = ServoParser::new(
            &document,
            Tokenizer::Html(self::html::Tokenizer::new(
                &document,
                url,
                Some(fragment_context),
                ParsingAlgorithm::Fragment,
            )),
            ParserKind::Normal,
            None,
            None,
            CanGc::from_cx(cx),
        );
        parser.parse_complete_string_chunk(String::from(input), cx);

        // Step 14.
        let root_element = document.GetDocumentElement().expect("no document element");
        FragmentParsingResult {
            inner: root_element.upcast::<Node>().children(),
        }
    }

    pub(crate) fn parse_html_script_input(document: &Document, url: ServoUrl) {
        let parser = ServoParser::new(
            document,
            if pref!(dom_servoparser_async_html_tokenizer_enabled) {
                Tokenizer::AsyncHtml(self::async_html::Tokenizer::new(document, url, None))
            } else {
                Tokenizer::Html(self::html::Tokenizer::new(
                    document,
                    url,
                    None,
                    ParsingAlgorithm::Normal,
                ))
            },
            ParserKind::ScriptCreated,
            None,
            None,
            CanGc::note(),
        );
        document.set_current_parser(Some(&parser));
    }

    pub(crate) fn parse_xml_document(
        document: &Document,
        input: Option<DOMString>,
        url: ServoUrl,
        encoding_hint_from_content_type: Option<&'static Encoding>,
        cx: &mut js::context::JSContext,
    ) {
        let parser = ServoParser::new(
            document,
            Tokenizer::Xml(self::xml::Tokenizer::new(document, url)),
            ParserKind::Normal,
            encoding_hint_from_content_type,
            None,
            CanGc::from_cx(cx),
        );

        // Set as the document's current parser and initialize with `input`, if given.
        if let Some(input) = input {
            parser.parse_complete_string_chunk(String::from(input), cx);
        } else {
            parser.document.set_current_parser(Some(&parser));
        }
    }

    pub(crate) fn script_nesting_level(&self) -> usize {
        self.script_nesting_level.get()
    }

    pub(crate) fn is_script_created(&self) -> bool {
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
    pub(crate) fn resume_with_pending_parsing_blocking_script(
        &self,
        script: &HTMLScriptElement,
        result: ScriptResult,
        cx: &mut js::context::JSContext,
    ) {
        assert!(self.suspended.get());
        self.suspended.set(false);

        self.script_input.swap_with(&self.network_input);
        while let Some(chunk) = self.script_input.pop_front() {
            self.network_input.push_back(chunk);
        }

        let script_nesting_level = self.script_nesting_level.get();
        assert_eq!(script_nesting_level, 0);

        self.script_nesting_level.set(script_nesting_level + 1);
        script.execute(result, CanGc::from_cx(cx));
        self.script_nesting_level.set(script_nesting_level);

        if !self.suspended.get() && !self.aborted.get() {
            self.parse_sync(cx);
        }
    }

    pub(crate) fn can_write(&self) -> bool {
        self.script_created_parser || self.script_nesting_level.get() > 0
    }

    /// Steps 6-8 of <https://html.spec.whatwg.org/multipage/#document.write()>
    pub(crate) fn write(&self, text: DOMString, cx: &mut js::context::JSContext) {
        assert!(self.can_write());

        if self.document.has_pending_parsing_blocking_script() {
            // There is already a pending parsing blocking script so the
            // parser is suspended, we just append everything to the
            // script input and abort these steps.
            self.script_input.push_back(String::from(text).into());
            return;
        }

        // There is no pending parsing blocking script, so all previous calls
        // to document.write() should have seen their entire input tokenized
        // and process, with nothing pushed to the parser script input.
        assert!(self.script_input.is_empty());

        let input = BufferQueue::default();
        input.push_back(String::from(text).into());

        let profiler_chan = self
            .document
            .window()
            .as_global_scope()
            .time_profiler_chan()
            .clone();
        let profiler_metadata = TimerMetadata {
            url: self.document.url().as_str().into(),
            iframe: TimerMetadataFrameType::RootWindow,
            incremental: TimerMetadataReflowType::FirstReflow,
        };
        self.tokenize(
            |cx, tokenizer| {
                tokenizer.feed(&input, cx, profiler_chan.clone(), profiler_metadata.clone())
            },
            cx,
        );

        if self.suspended.get() {
            // Parser got suspended, insert remaining input at end of
            // script input, following anything written by scripts executed
            // reentrantly during this call.
            while let Some(chunk) = input.pop_front() {
                self.script_input.push_back(chunk);
            }
            return;
        }

        assert!(input.is_empty());
    }

    // Steps 4-6 of https://html.spec.whatwg.org/multipage/#dom-document-close
    pub(crate) fn close(&self, cx: &mut js::context::JSContext) {
        assert!(self.script_created_parser);

        // Step 4.
        self.last_chunk_received.set(true);

        if self.suspended.get() {
            // Step 5.
            return;
        }

        // Step 6.
        self.parse_sync(cx);
    }

    // https://html.spec.whatwg.org/multipage/#abort-a-parser
    pub(crate) fn abort(&self, cx: &mut js::context::JSContext) {
        assert!(!self.aborted.get());
        self.aborted.set(true);

        // Step 1.
        self.script_input.replace_with(BufferQueue::default());
        self.network_input.replace_with(BufferQueue::default());

        // Step 2.
        self.document
            .set_ready_state(DocumentReadyState::Interactive, CanGc::from_cx(cx));

        // Step 3.
        self.tokenizer.end(cx);
        self.document.set_current_parser(None);

        // Step 4.
        self.document
            .set_ready_state(DocumentReadyState::Complete, CanGc::from_cx(cx));
    }

    // https://html.spec.whatwg.org/multipage/#active-parser
    pub(crate) fn is_active(&self) -> bool {
        self.script_nesting_level() > 0 && !self.aborted.get()
    }

    pub(crate) fn get_current_line(&self) -> u32 {
        self.tokenizer.get_current_line()
    }

    #[cfg_attr(crown, expect(crown::unrooted_must_root))]
    fn new_inherited(
        document: &Document,
        tokenizer: Tokenizer,
        kind: ParserKind,
        encoding_hint_from_content_type: Option<&'static Encoding>,
        encoding_of_container_document: Option<&'static Encoding>,
    ) -> Self {
        // Store the whole input for the devtools Sources panel, if the devtools server is running
        // and we are parsing for a document load (not just things like innerHTML).
        // TODO: check if a devtools client is actually connected and/or wants the sources?
        let content_for_devtools = (document.global().devtools_chan().is_some() &&
            document.has_browsing_context())
        .then_some(DomRefCell::new(String::new()));

        ServoParser {
            reflector: Reflector::new(),
            document: Dom::from_ref(document),
            network_decoder: DomRefCell::new(NetworkDecoderState::new(
                encoding_hint_from_content_type,
                encoding_of_container_document,
            )),
            network_input: BufferQueue::default(),
            script_input: BufferQueue::default(),
            tokenizer,
            last_chunk_received: Cell::new(false),
            suspended: Default::default(),
            script_nesting_level: Default::default(),
            aborted: Default::default(),
            script_created_parser: kind == ParserKind::ScriptCreated,
            prefetch_decoder: RefCell::new(LossyDecoder::new_encoding_rs(
                encoding_hint_from_content_type.unwrap_or(UTF_8),
                Default::default(),
            )),
            prefetch_tokenizer: prefetch::Tokenizer::new(document),
            prefetch_input: BufferQueue::default(),
            content_for_devtools,
        }
    }

    #[cfg_attr(crown, expect(crown::unrooted_must_root))]
    fn new(
        document: &Document,
        tokenizer: Tokenizer,
        kind: ParserKind,
        encoding_hint_from_content_type: Option<&'static Encoding>,
        encoding_of_container_document: Option<&'static Encoding>,
        can_gc: CanGc,
    ) -> DomRoot<Self> {
        reflect_dom_object(
            Box::new(ServoParser::new_inherited(
                document,
                tokenizer,
                kind,
                encoding_hint_from_content_type,
                encoding_of_container_document,
            )),
            document.window(),
            can_gc,
        )
    }

    fn push_tendril_input_chunk(&self, chunk: StrTendril) {
        if let Some(mut content_for_devtools) = self
            .content_for_devtools
            .as_ref()
            .map(|content| content.borrow_mut())
        {
            // TODO: append these chunks more efficiently
            content_for_devtools.push_str(chunk.as_ref());
        }

        if chunk.is_empty() {
            return;
        }

        // Push the chunk into the network input stream,
        // which is tokenized lazily.
        self.network_input.push_back(chunk);
    }

    fn push_bytes_input_chunk(&self, chunk: Vec<u8>) {
        // For byte input, we convert it to text using the network decoder.
        if let Some(decoded_chunk) = self
            .network_decoder
            .borrow_mut()
            .push(&chunk, &self.document)
        {
            self.push_tendril_input_chunk(decoded_chunk);
        }

        if self.should_prefetch() {
            // Push the chunk into the prefetch input stream,
            // which is tokenized eagerly, to scan for resources
            // to prefetch. If the user script uses `document.write()`
            // to overwrite the network input, this prefetching may
            // have been wasted, but in most cases it won't.
            let mut prefetch_decoder = self.prefetch_decoder.borrow_mut();
            prefetch_decoder.process(ByteTendril::from(&*chunk));

            self.prefetch_input
                .push_back(mem::take(&mut prefetch_decoder.inner_sink_mut().output));
            self.prefetch_tokenizer.feed(&self.prefetch_input);
        }
    }

    fn should_prefetch(&self) -> bool {
        // Per https://github.com/whatwg/html/issues/1495
        // stylesheets should not be loaded for documents
        // without browsing contexts.
        // https://github.com/whatwg/html/issues/1495#issuecomment-230334047
        // suggests that no content should be preloaded in such a case.
        // We're conservative, and only prefetch for documents
        // with browsing contexts.
        self.document.browsing_context().is_some()
    }

    fn push_string_input_chunk(&self, chunk: String) {
        // The input has already been decoded as a string, so doesn't need
        // to be decoded by the network decoder again.
        let chunk = StrTendril::from(chunk);
        self.push_tendril_input_chunk(chunk);
    }

    fn parse_sync(&self, cx: &mut js::context::JSContext) {
        assert!(self.script_input.is_empty());

        // This parser will continue to parse while there is either pending input or
        // the parser remains unsuspended.

        if self.last_chunk_received.get() {
            let chunk = self.network_decoder.borrow_mut().finish(&self.document);
            if !chunk.is_empty() {
                self.push_tendril_input_chunk(chunk);
            }
        }

        if self.aborted.get() {
            return;
        }

        let profiler_chan = self
            .document
            .window()
            .as_global_scope()
            .time_profiler_chan()
            .clone();
        let profiler_metadata = TimerMetadata {
            url: self.document.url().as_str().into(),
            iframe: TimerMetadataFrameType::RootWindow,
            incremental: TimerMetadataReflowType::FirstReflow,
        };
        self.tokenize(
            |cx, tokenizer| {
                tokenizer.feed(
                    &self.network_input,
                    cx,
                    profiler_chan.clone(),
                    profiler_metadata.clone(),
                )
            },
            cx,
        );

        if self.suspended.get() {
            return;
        }

        assert!(self.network_input.is_empty());

        if self.last_chunk_received.get() {
            self.finish(cx);
        }
    }

    fn parse_complete_string_chunk(&self, input: String, cx: &mut js::context::JSContext) {
        self.document.set_current_parser(Some(self));
        self.push_string_input_chunk(input);
        self.last_chunk_received.set(true);
        if !self.suspended.get() {
            self.parse_sync(cx);
        }
    }

    fn parse_bytes_chunk(&self, input: Vec<u8>, cx: &mut js::context::JSContext) {
        let _realm = enter_realm(&*self.document);
        self.document.set_current_parser(Some(self));
        self.push_bytes_input_chunk(input);
        if !self.suspended.get() {
            self.parse_sync(cx);
        }
    }

    fn tokenize<F>(&self, feed: F, cx: &mut js::context::JSContext)
    where
        F: Fn(
            &mut js::context::JSContext,
            &Tokenizer,
        ) -> TokenizerResult<DomRoot<HTMLScriptElement>>,
    {
        loop {
            assert!(!self.suspended.get());
            assert!(!self.aborted.get());

            self.document.window().reflow_if_reflow_timer_expired();
            let script = match feed(cx, &self.tokenizer) {
                TokenizerResult::Done => return,
                TokenizerResult::EncodingIndicator(_) => continue,
                TokenizerResult::Script(script) => script,
            };

            // https://html.spec.whatwg.org/multipage/#parsing-main-incdata
            // branch "An end tag whose tag name is "script"
            // The spec says to perform the microtask checkpoint before
            // setting the insertion mode back from Text, but this is not
            // possible with the way servo and html5ever currently
            // relate to each other, and hopefully it is not observable.
            if is_execution_stack_empty() {
                self.document.window().perform_a_microtask_checkpoint(cx);
            }

            let script_nesting_level = self.script_nesting_level.get();

            self.script_nesting_level.set(script_nesting_level + 1);
            script.set_initial_script_text();
            let introduction_type_override =
                (script_nesting_level > 0).then_some(IntroductionType::INJECTED_SCRIPT);
            script.prepare(introduction_type_override, CanGc::from_cx(cx));
            self.script_nesting_level.set(script_nesting_level);

            if self.document.has_pending_parsing_blocking_script() {
                self.suspended.set(true);
                return;
            }
            if self.aborted.get() {
                return;
            }
        }
    }

    /// <https://html.spec.whatwg.org/multipage/#the-end>
    fn finish(&self, cx: &mut js::context::JSContext) {
        assert!(!self.suspended.get());
        assert!(self.last_chunk_received.get());
        assert!(self.script_input.is_empty());
        assert!(self.network_input.is_empty());
        assert!(self.network_decoder.borrow().is_finished());

        // Step 1.
        self.document
            .set_ready_state(DocumentReadyState::Interactive, CanGc::from_cx(cx));

        // Step 2.
        self.tokenizer.end(cx);
        self.document.set_current_parser(None);

        // Steps 3-12 are in another castle, namely finish_load.
        let url = self.tokenizer.url().clone();
        self.document.finish_load(LoadType::PageSource(url), cx);

        // Send the source contents to devtools, if needed.
        if let Some(content_for_devtools) = self
            .content_for_devtools
            .as_ref()
            .map(|content| content.take())
        {
            let global = self.document.global();
            let chan = global.devtools_chan().expect("Guaranteed by new");
            let pipeline_id = self.document.global().pipeline_id();
            let _ = chan.send(ScriptToDevtoolsControlMsg::UpdateSourceContent(
                pipeline_id,
                content_for_devtools,
            ));
        }
    }
}

struct FragmentParsingResult<I>
where
    I: Iterator<Item = DomRoot<Node>>,
{
    inner: I,
}

impl<I> Iterator for FragmentParsingResult<I>
where
    I: Iterator<Item = DomRoot<Node>>,
{
    type Item = DomRoot<Node>;

    fn next(&mut self) -> Option<DomRoot<Node>> {
        let next = self.inner.next()?;
        next.remove_self(CanGc::note());
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
#[cfg_attr(crown, crown::unrooted_must_root_lint::must_root)]
enum Tokenizer {
    Html(self::html::Tokenizer),
    AsyncHtml(self::async_html::Tokenizer),
    Xml(self::xml::Tokenizer),
}

impl Tokenizer {
    fn feed(
        &self,
        input: &BufferQueue,
        cx: &mut js::context::JSContext,
        profiler_chan: ProfilerChan,
        profiler_metadata: TimerMetadata,
    ) -> TokenizerResult<DomRoot<HTMLScriptElement>> {
        match *self {
            Tokenizer::Html(ref tokenizer) => time_profile!(
                ProfilerCategory::ScriptParseHTML,
                Some(profiler_metadata),
                profiler_chan,
                || tokenizer.feed(input),
            ),
            Tokenizer::AsyncHtml(ref tokenizer) => time_profile!(
                ProfilerCategory::ScriptParseHTML,
                Some(profiler_metadata),
                profiler_chan,
                || tokenizer.feed(input, cx),
            ),
            Tokenizer::Xml(ref tokenizer) => time_profile!(
                ProfilerCategory::ScriptParseXML,
                Some(profiler_metadata),
                profiler_chan,
                || tokenizer.feed(input),
            ),
        }
    }

    fn end(&self, cx: &mut js::context::JSContext) {
        match *self {
            Tokenizer::Html(ref tokenizer) => tokenizer.end(),
            Tokenizer::AsyncHtml(ref tokenizer) => tokenizer.end(cx),
            Tokenizer::Xml(ref tokenizer) => tokenizer.end(),
        }
    }

    fn url(&self) -> &ServoUrl {
        match *self {
            Tokenizer::Html(ref tokenizer) => tokenizer.url(),
            Tokenizer::AsyncHtml(ref tokenizer) => tokenizer.url(),
            Tokenizer::Xml(ref tokenizer) => tokenizer.url(),
        }
    }

    fn set_plaintext_state(&self) {
        match *self {
            Tokenizer::Html(ref tokenizer) => tokenizer.set_plaintext_state(),
            Tokenizer::AsyncHtml(ref tokenizer) => tokenizer.set_plaintext_state(),
            Tokenizer::Xml(_) => unimplemented!(),
        }
    }

    fn get_current_line(&self) -> u32 {
        match *self {
            Tokenizer::Html(ref tokenizer) => tokenizer.get_current_line(),
            Tokenizer::AsyncHtml(ref tokenizer) => tokenizer.get_current_line(),
            Tokenizer::Xml(ref tokenizer) => tokenizer.get_current_line(),
        }
    }
}

/// <https://html.spec.whatwg.org/multipage/#navigation-params>
/// This does not have the relevant fields, but mimics the intent
/// of the struct when used in loading document spec algorithms.
struct NavigationParams {
    /// <https://html.spec.whatwg.org/multipage/#navigation-params-policy-container>
    policy_container: PolicyContainer,
    /// content-type of this document, if known. Otherwise need to sniff it
    content_type: Option<Mime>,
    /// link headers from the response
    link_headers: Vec<LinkHeader>,
    /// <https://html.spec.whatwg.org/multipage/#navigation-params-sandboxing>
    final_sandboxing_flag_set: SandboxingFlagSet,
    /// <https://mimesniff.spec.whatwg.org/#resource-header>
    resource_header: Vec<u8>,
    /// <https://html.spec.whatwg.org/multipage/#navigation-params-about-base-url>
    about_base_url: Option<ServoUrl>,
}

/// The context required for asynchronously fetching a document
/// and parsing it progressively.
pub(crate) struct ParserContext {
    /// The parser that initiated the request.
    parser: Option<Trusted<ServoParser>>,
    /// Is this a synthesized document
    is_synthesized_document: bool,
    /// Has a document already been loaded (relevant for checking the resource header)
    has_loaded_document: bool,
    /// The [`WebViewId`] of the `WebView` associated with this document.
    webview_id: WebViewId,
    /// The [`PipelineId`] of the `Pipeline` associated with this document.
    pipeline_id: PipelineId,
    /// The URL for this document.
    url: ServoUrl,
    /// pushed entry index
    pushed_entry_index: Option<usize>,
    /// params required in document load algorithms
    navigation_params: NavigationParams,
}

impl ParserContext {
    pub(crate) fn new(
        webview_id: WebViewId,
        pipeline_id: PipelineId,
        url: ServoUrl,
        creation_sandboxing_flag_set: SandboxingFlagSet,
    ) -> ParserContext {
        ParserContext {
            parser: None,
            is_synthesized_document: false,
            has_loaded_document: false,
            webview_id,
            pipeline_id,
            url,
            pushed_entry_index: None,
            navigation_params: NavigationParams {
                policy_container: Default::default(),
                content_type: None,
                link_headers: vec![],
                final_sandboxing_flag_set: creation_sandboxing_flag_set,
                resource_header: vec![],
                about_base_url: Default::default(),
            },
        }
    }

    pub(crate) fn set_policy_container(&mut self, policy_container: Option<&PolicyContainer>) {
        let Some(policy_container) = policy_container else {
            return;
        };
        self.navigation_params.policy_container = policy_container.clone();
    }

    pub(crate) fn set_about_base_url(&mut self, about_base_url: Option<ServoUrl>) {
        self.navigation_params.about_base_url = about_base_url;
    }

    pub(crate) fn get_document(&self) -> Option<DomRoot<Document>> {
        self.parser
            .as_ref()
            .map(|parser| parser.root().document.as_rooted())
    }

    /// <https://html.spec.whatwg.org/multipage/#creating-a-policy-container-from-a-fetch-response>
    fn create_policy_container_from_fetch_response(metadata: &Metadata) -> PolicyContainer {
        // Step 1. If response's URL's scheme is "blob", then return a clone of response's URL's blob URL entry's environment's policy container.
        // TODO
        // Step 2. Let result be a new policy container.
        // Step 7. Return result.
        PolicyContainer {
            // Step 3. Set result's CSP list to the result of parsing a response's Content Security Policies given response.
            csp_list: parse_csp_list_from_metadata(&metadata.headers),
            // Step 5. Set result's referrer policy to the result of parsing the `Referrer-Policy` header given response. [REFERRERPOLICY]
            referrer_policy: ReferrerPolicy::parse_header_for_response(&metadata.headers),
        }
    }

    /// <https://html.spec.whatwg.org/multipage/#initialise-the-document-object>
    fn initialize_document_object(&self, document: &Document) {
        // Step 9. Let document be a new Document, with
        document.set_policy_container(self.navigation_params.policy_container.clone());
        document.set_active_sandboxing_flag_set(self.navigation_params.final_sandboxing_flag_set);
        document.set_about_base_url(self.navigation_params.about_base_url.clone());
        // Step 17. Process link headers given document, navigationParams's response, and "pre-media".
        process_link_headers(
            &self.navigation_params.link_headers,
            document,
            LinkProcessingPhase::PreMedia,
        );
    }

    /// Part of various load document methods
    fn process_link_headers_in_media_phase_with_task(&mut self, document: &Document) {
        // The first task that the networking task source places on the task queue
        // while fetching runs must process link headers given document,
        // navigationParams's response, and "media", after the task has been processed by the HTML parser.
        let link_headers = std::mem::take(&mut self.navigation_params.link_headers);
        if !link_headers.is_empty() {
            let window = document.window();
            let document = Trusted::new(document);
            window
                .upcast::<GlobalScope>()
                .task_manager()
                .networking_task_source()
                .queue(task!(process_link_headers_task: move || {
                    process_link_headers(&link_headers, &document.root(), LinkProcessingPhase::Media);
                }));
        }
    }

    /// <https://html.spec.whatwg.org/multipage/#loading-a-document>
    fn load_document(&mut self, cx: &mut js::context::JSContext) {
        assert!(!self.has_loaded_document);
        self.has_loaded_document = true;
        let Some(ref parser) = self.parser.as_ref().map(|p| p.root()) else {
            return;
        };
        // Step 1. Let type be the computed type of navigationParams's response.
        let content_type = &self.navigation_params.content_type;
        let mime_type = MimeClassifier::default().classify(
            LoadContext::Browsing,
            NoSniffFlag::Off,
            ApacheBugFlag::from_content_type(content_type.as_ref()),
            content_type,
            &self.navigation_params.resource_header,
        );
        // Step 2. If the user agent has been configured to process resources of the given type using
        // some mechanism other than rendering the content in a navigable, then skip this step.
        // Otherwise, if the type is one of the following types:
        let Some(media_type) = MimeClassifier::get_media_type(&mime_type) else {
            let page = format!(
                "<html><body><p>Unknown content type ({}).</p></body></html>",
                &mime_type,
            );
            self.load_inline_unknown_content(parser, page, cx);
            return;
        };
        match media_type {
            // Return the result of loading an HTML document, given navigationParams.
            MediaType::Html => self.load_html_document(parser),
            // Return the result of loading an XML document given navigationParams and type.
            MediaType::Xml => self.load_xml_document(parser),
            // Return the result of loading a text document given navigationParams and type.
            MediaType::JavaScript | MediaType::Json | MediaType::Text | MediaType::Css => {
                self.load_text_document(parser, cx)
            },
            // Return the result of loading a media document given navigationParams and type.
            MediaType::Image | MediaType::AudioVideo => {
                self.load_media_document(parser, media_type, &mime_type, cx);
                return;
            },
            MediaType::Font => {
                let page = format!(
                    "<html><body><p>Unable to load font with content type ({}).</p></body></html>",
                    &mime_type,
                );
                self.load_inline_unknown_content(parser, page, cx);
                return;
            },
        };

        parser.parse_bytes_chunk(
            std::mem::take(&mut self.navigation_params.resource_header),
            cx,
        );
    }

    /// <https://html.spec.whatwg.org/multipage/#navigate-html>
    fn load_html_document(&mut self, parser: &ServoParser) {
        // Step 1. Let document be the result of creating and initializing a
        // Document object given "html", "text/html", and navigationParams.
        self.initialize_document_object(&parser.document);
        // The first task that the networking task source places on the task queue while fetching
        // runs must process link headers given document, navigationParams's response, and "media",
        // after the task has been processed by the HTML parser.
        self.process_link_headers_in_media_phase_with_task(&parser.document);
    }

    /// <https://html.spec.whatwg.org/multipage/#read-xml>
    fn load_xml_document(&mut self, parser: &ServoParser) {
        // When faced with displaying an XML file inline, provided navigation params navigationParams
        // and a string type, user agents must follow the requirements defined in XML and Namespaces in XML,
        // XML Media Types, DOM, and other relevant specifications to create and initialize a
        // Document object document, given "xml", type, and navigationParams, and return that Document.
        // They must also create a corresponding XML parser. [XML] [XMLNS] [RFC7303] [DOM]
        self.initialize_document_object(&parser.document);
        // The first task that the networking task source places on the task queue while fetching
        // runs must process link headers given document, navigationParams's response, and "media",
        // after the task has been processed by the XML parser.
        self.process_link_headers_in_media_phase_with_task(&parser.document);
    }

    /// <https://html.spec.whatwg.org/multipage/#navigate-text>
    fn load_text_document(&mut self, parser: &ServoParser, cx: &mut js::context::JSContext) {
        // Step 1. Let document be the result of creating and initializing a Document
        // object given "html", type, and navigationParams.
        self.initialize_document_object(&parser.document);
        // Step 4. Create an HTML parser and associate it with the document.
        // Act as if the tokenizer had emitted a start tag token with the tag name "pre" followed by
        // a single U+000A LINE FEED (LF) character, and switch the HTML parser's tokenizer to the PLAINTEXT state.
        // Each task that the networking task source places on the task queue while fetching runs must then
        // fill the parser's input byte stream with the fetched bytes and cause the HTML parser to perform
        // the appropriate processing of the input stream.
        let page = "<pre>\n".into();
        parser.push_string_input_chunk(page);
        parser.parse_sync(cx);
        parser.tokenizer.set_plaintext_state();
        // The first task that the networking task source places on the task queue while fetching
        // runs must process link headers given document, navigationParams's response, and "media",
        // after the task has been processed by the HTML parser.
        self.process_link_headers_in_media_phase_with_task(&parser.document);
    }

    /// <https://html.spec.whatwg.org/multipage/#navigate-media>
    fn load_media_document(
        &mut self,
        parser: &ServoParser,
        media_type: MediaType,
        mime_type: &Mime,
        cx: &mut js::context::JSContext,
    ) {
        // Step 1. Let document be the result of creating and initializing a Document
        // object given "html", type, and navigationParams.
        self.initialize_document_object(&parser.document);
        // Step 8. Act as if the user agent had stopped parsing document.
        self.is_synthesized_document = true;
        // Step 3. Populate with html/head/body given document.
        let page = "<html><body></body></html>".into();
        parser.push_string_input_chunk(page);
        parser.parse_sync(cx);

        let doc = &parser.document;
        // Step 5. Set the appropriate attribute of the element host element, as described below,
        // to the address of the image, video, or audio resource.
        let node = if media_type == MediaType::Image {
            let img = Element::create(
                QualName::new(None, ns!(html), local_name!("img")),
                None,
                doc,
                ElementCreator::ParserCreated(1),
                CustomElementCreationMode::Asynchronous,
                None,
                CanGc::from_cx(cx),
            );
            let img = DomRoot::downcast::<HTMLImageElement>(img).unwrap();
            img.SetSrc(USVString(self.url.to_string()));
            DomRoot::upcast::<Node>(img)
        } else if mime_type.type_() == mime::AUDIO {
            let audio = Element::create(
                QualName::new(None, ns!(html), local_name!("audio")),
                None,
                doc,
                ElementCreator::ParserCreated(1),
                CustomElementCreationMode::Asynchronous,
                None,
                CanGc::from_cx(cx),
            );
            let audio = DomRoot::downcast::<HTMLMediaElement>(audio).unwrap();
            audio.SetControls(true);
            audio.SetSrc(USVString(self.url.to_string()));
            DomRoot::upcast::<Node>(audio)
        } else {
            let video = Element::create(
                QualName::new(None, ns!(html), local_name!("video")),
                None,
                doc,
                ElementCreator::ParserCreated(1),
                CustomElementCreationMode::Asynchronous,
                None,
                CanGc::from_cx(cx),
            );
            let video = DomRoot::downcast::<HTMLMediaElement>(video).unwrap();
            video.SetControls(true);
            video.SetSrc(USVString(self.url.to_string()));
            DomRoot::upcast::<Node>(video)
        };
        // Step 4. Append an element host element for the media, as described below, to the body element.
        let doc_body = DomRoot::upcast::<Node>(doc.GetBody().unwrap());
        doc_body
            .AppendChild(&node, CanGc::from_cx(cx))
            .expect("Appending failed");
        // Step 7. Process link headers given document, navigationParams's response, and "media".
        let link_headers = std::mem::take(&mut self.navigation_params.link_headers);
        process_link_headers(&link_headers, doc, LinkProcessingPhase::Media);
    }

    /// <https://html.spec.whatwg.org/multipage/#read-ua-inline>
    fn load_inline_unknown_content(
        &mut self,
        parser: &ServoParser,
        page: String,
        cx: &mut js::context::JSContext,
    ) {
        self.is_synthesized_document = true;
        parser.push_string_input_chunk(page);
        parser.parse_sync(cx);
    }

    /// Store a PerformanceNavigationTiming entry in the globalscope's Performance buffer
    fn submit_resource_timing(&mut self) {
        let Some(parser) = self.parser.as_ref() else {
            return;
        };
        let parser = parser.root();
        if parser.aborted.get() {
            return;
        }

        let document = &parser.document;

        // TODO: Pass a proper fetch start time here.
        let performance_entry = PerformanceNavigationTiming::new(
            &document.global(),
            CrossProcessInstant::now(),
            document,
            CanGc::note(),
        );
        self.pushed_entry_index = document
            .global()
            .performance()
            .queue_entry(performance_entry.upcast::<PerformanceEntry>());
    }
}

impl FetchResponseListener for ParserContext {
    fn process_request_body(&mut self, _: RequestId) {}

    fn process_request_eof(&mut self, _: RequestId) {}

    #[expect(unsafe_code)]
    fn process_response(&mut self, _: RequestId, meta_result: Result<FetchMetadata, NetworkError>) {
        // TODO: https://github.com/servo/servo/issues/42840
        let mut cx = unsafe { temp_cx() };
        let cx = &mut cx;
        let (metadata, error) = match meta_result {
            Ok(meta) => (
                Some(match meta {
                    FetchMetadata::Unfiltered(m) => m,
                    FetchMetadata::Filtered { unsafe_, .. } => unsafe_,
                }),
                None,
            ),
            Err(error) => (
                // Check variant without moving
                match &error {
                    NetworkError::LoadCancelled => {
                        return;
                    },
                    _ => {
                        let mut meta = Metadata::default(self.url.clone());
                        let mime: Option<Mime> = "text/html".parse().ok();
                        meta.set_content_type(mime.as_ref());
                        Some(meta)
                    },
                },
                Some(error),
            ),
        };
        let content_type: Option<Mime> = metadata
            .clone()
            .and_then(|meta| meta.content_type)
            .map(Serde::into_inner)
            .map(Into::into);

        let (policy_container, endpoints_list, link_headers) = match metadata.as_ref() {
            None => (PolicyContainer::default(), None, vec![]),
            Some(metadata) => (
                Self::create_policy_container_from_fetch_response(metadata),
                ReportingEndpoint::parse_reporting_endpoints_header(
                    &self.url.clone(),
                    &metadata.headers,
                ),
                extract_links_from_headers(&metadata.headers),
            ),
        };

        let parser = match ScriptThread::page_headers_available(
            self.webview_id,
            self.pipeline_id,
            metadata,
            cx,
        ) {
            Some(parser) => parser,
            None => return,
        };
        if parser.aborted.get() {
            return;
        }

        let mut realm = enter_auto_realm(cx, &*parser.document);
        let cx = &mut realm;
        let window = parser.document.window();

        // From Step 23.8.3 of https://html.spec.whatwg.org/multipage/#navigate
        // Let finalSandboxFlags be the union of targetSnapshotParams's sandboxing flags and
        // policyContainer's CSP list's CSP-derived sandboxing flags.
        //
        // TODO: This deviates a bit from the specification, because there isn't a `targetSnapshotParam`
        // concept yet.
        let final_sandboxing_flag_set = policy_container
            .csp_list
            .as_ref()
            .and_then(|csp| csp.get_sandboxing_flag_set_for_document())
            .unwrap_or(SandboxingFlagSet::empty())
            .union(parser.document.creation_sandboxing_flag_set());

        if let Some(endpoints) = endpoints_list {
            window.set_endpoints_list(endpoints);
        }
        self.parser = Some(Trusted::new(&*parser));
        self.navigation_params = NavigationParams {
            policy_container,
            content_type,
            final_sandboxing_flag_set,
            link_headers,
            about_base_url: parser.document.about_base_url(),
            resource_header: vec![],
        };
        self.submit_resource_timing();

        // Part of https://html.spec.whatwg.org/multipage/#loading-a-document
        //
        // Step 3. If, given type, the new resource is to be handled by displaying some sort of inline content,
        // e.g., a native rendering of the content or an error message because the specified type is not supported,
        // then return the result of creating a document for inline content that doesn't have a DOM given
        // navigationParams's navigable, navigationParams's id, navigationParams's navigation timing type,
        // and navigationParams's user involvement.
        if let Some(error) = error {
            let page = match error {
                NetworkError::SslValidation(reason, bytes) => {
                    let page = resources::read_string(Resource::BadCertHTML);
                    let page = page.replace("${reason}", &reason);
                    let encoded_bytes = general_purpose::STANDARD_NO_PAD.encode(bytes);
                    let page = page.replace("${bytes}", encoded_bytes.as_str());
                    page.replace("${secret}", &net_traits::PRIVILEGED_SECRET.to_string())
                },
                NetworkError::BlobURLStoreError(reason) |
                NetworkError::WebsocketConnectionFailure(reason) |
                NetworkError::HttpError(reason) |
                NetworkError::ResourceLoadError(reason) |
                NetworkError::MimeType(reason) => {
                    let page = resources::read_string(Resource::NetErrorHTML);
                    page.replace("${reason}", &reason)
                },
                NetworkError::Crash(details) => {
                    let page = resources::read_string(Resource::CrashHTML);
                    page.replace("${details}", &details)
                },
                NetworkError::UnsupportedScheme |
                NetworkError::CorsGeneral |
                NetworkError::CrossOriginResponse |
                NetworkError::CorsCredentials |
                NetworkError::CorsAllowMethods |
                NetworkError::CorsAllowHeaders |
                NetworkError::CorsMethod |
                NetworkError::CorsAuthorization |
                NetworkError::CorsHeaders |
                NetworkError::ConnectionFailure |
                NetworkError::RedirectError |
                NetworkError::TooManyRedirects |
                NetworkError::TooManyInFlightKeepAliveRequests |
                NetworkError::InvalidMethod |
                NetworkError::ContentSecurityPolicy |
                NetworkError::Nosniff |
                NetworkError::SubresourceIntegrity |
                NetworkError::MixedContent |
                NetworkError::CacheError |
                NetworkError::InvalidPort |
                NetworkError::LocalDirectoryError |
                NetworkError::PartialResponseToNonRangeRequestError |
                NetworkError::ProtocolHandlerSubstitutionError |
                NetworkError::DecompressionError => {
                    let page = resources::read_string(Resource::NetErrorHTML);
                    page.replace("${reason}", &format!("{:?}", error))
                },
                NetworkError::LoadCancelled => {
                    // The next load will show a page
                    return;
                },
            };
            self.load_inline_unknown_content(&parser, page, cx);
        }
    }

    #[expect(unsafe_code)]
    fn process_response_chunk(&mut self, _: RequestId, payload: Vec<u8>) {
        // TODO: https://github.com/servo/servo/issues/42841
        let mut cx = unsafe { temp_cx() };
        let cx = &mut cx;
        if self.is_synthesized_document {
            return;
        }
        let Some(parser) = self.parser.as_ref().map(|p| p.root()) else {
            return;
        };
        if parser.aborted.get() {
            return;
        }
        if !self.has_loaded_document {
            // https://mimesniff.spec.whatwg.org/#read-the-resource-header
            self.navigation_params
                .resource_header
                .extend_from_slice(&payload);
            // the number of bytes in buffer is greater than or equal to 1445.
            if self.navigation_params.resource_header.len() >= 1445 {
                self.load_document(cx);
            }
        } else {
            parser.parse_bytes_chunk(payload, cx);
        }
    }

    // This method is called via script_thread::handle_fetch_eof, so we must call
    // submit_resource_timing in this function
    // Resource listeners are called via net_traits::Action::process, which handles submission for them
    fn process_response_eof(
        mut self,
        cx: &mut js::context::JSContext,
        _: RequestId,
        status: Result<(), NetworkError>,
        timing: ResourceFetchTiming,
    ) {
        let parser = match self.parser.as_ref() {
            Some(parser) => parser.root(),
            None => return,
        };
        if parser.aborted.get() {
            return;
        }

        if let Err(error) = &status {
            // TODO(Savago): we should send a notification to callers #5463.
            debug!("Failed to load page URL {}, error: {error:?}", self.url);
        }

        // https://mimesniff.spec.whatwg.org/#read-the-resource-header
        //
        // the end of the resource is reached.
        if !self.has_loaded_document {
            self.load_document(cx);
        }

        let mut realm = enter_auto_realm(cx, &*parser);
        let cx = &mut realm;

        if status.is_ok() {
            parser.document.set_redirect_count(timing.redirect_count);
        }

        parser.last_chunk_received.set(true);
        if !parser.suspended.get() {
            parser.parse_sync(cx);
        }

        // TODO: Only update if this is the current document resource.
        // TODO(mrobinson): Pass a proper fetch_start parameter here instead of `CrossProcessInstant::now()`.
        if let Some(pushed_index) = self.pushed_entry_index {
            let document = &parser.document;
            let performance_entry = PerformanceNavigationTiming::new(
                &document.global(),
                CrossProcessInstant::now(),
                document,
                CanGc::from_cx(cx),
            );
            document
                .global()
                .performance()
                .update_entry(pushed_index, performance_entry.upcast::<PerformanceEntry>());
        }
    }

    fn process_csp_violations(&mut self, _request_id: RequestId, violations: Vec<Violation>) {
        let parser = match self.parser.as_ref() {
            Some(parser) => parser.root(),
            None => return,
        };
        let document = &parser.document;
        let global = &document.global();
        // TODO(https://github.com/w3c/webappsec-csp/issues/687): Update after spec is resolved
        global.report_csp_violations(violations, None, None);
    }
}

pub(crate) struct FragmentContext<'a> {
    pub(crate) context_elem: &'a Node,
    pub(crate) form_elem: Option<&'a Node>,
    pub(crate) context_element_allows_scripting: bool,
}

#[cfg_attr(crown, expect(crown::unrooted_must_root))]
fn insert(
    parent: &Node,
    reference_child: Option<&Node>,
    child: NodeOrText<Dom<Node>>,
    parsing_algorithm: ParsingAlgorithm,
    custom_element_reaction_stack: &CustomElementReactionStack,
    can_gc: CanGc,
) {
    match child {
        NodeOrText::AppendNode(n) => {
            // https://html.spec.whatwg.org/multipage/#insert-a-foreign-element
            // applies if this is an element; if not, it may be
            // https://html.spec.whatwg.org/multipage/#insert-a-comment
            let element_in_non_fragment =
                parsing_algorithm != ParsingAlgorithm::Fragment && n.is::<Element>();
            if element_in_non_fragment {
                custom_element_reaction_stack.push_new_element_queue();
            }
            parent.InsertBefore(&n, reference_child, can_gc).unwrap();
            if element_in_non_fragment {
                custom_element_reaction_stack.pop_current_element_queue(can_gc);
            }
        },
        NodeOrText::AppendText(t) => {
            // https://html.spec.whatwg.org/multipage/#insert-a-character
            let text = reference_child
                .and_then(Node::GetPreviousSibling)
                .or_else(|| parent.GetLastChild())
                .and_then(DomRoot::downcast::<Text>);

            if let Some(text) = text {
                text.upcast::<CharacterData>().append_data(&t);
            } else {
                let text = Text::new(String::from(t).into(), &parent.owner_doc(), can_gc);
                parent
                    .InsertBefore(text.upcast(), reference_child, can_gc)
                    .unwrap();
            }
        },
    }
}

#[derive(JSTraceable, MallocSizeOf)]
#[cfg_attr(crown, crown::unrooted_must_root_lint::must_root)]
pub(crate) struct Sink {
    #[no_trace]
    base_url: ServoUrl,
    document: Dom<Document>,
    current_line: Cell<u64>,
    script: MutNullableDom<HTMLScriptElement>,
    parsing_algorithm: ParsingAlgorithm,
    #[conditional_malloc_size_of]
    custom_element_reaction_stack: Rc<CustomElementReactionStack>,
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

impl TreeSink for Sink {
    type Output = Self;
    #[cfg_attr(crown, expect(crown::unrooted_must_root))]
    fn finish(self) -> Self {
        self
    }

    type Handle = Dom<Node>;
    type ElemName<'a>
        = ExpandedName<'a>
    where
        Self: 'a;

    #[cfg_attr(crown, expect(crown::unrooted_must_root))]
    fn get_document(&self) -> Dom<Node> {
        Dom::from_ref(self.document.upcast())
    }

    #[cfg_attr(crown, expect(crown::unrooted_must_root))]
    fn get_template_contents(&self, target: &Dom<Node>) -> Dom<Node> {
        let template = target
            .downcast::<HTMLTemplateElement>()
            .expect("tried to get template contents of non-HTMLTemplateElement in HTML parsing");
        Dom::from_ref(template.Content(CanGc::note()).upcast())
    }

    fn same_node(&self, x: &Dom<Node>, y: &Dom<Node>) -> bool {
        x == y
    }

    fn elem_name<'a>(&self, target: &'a Dom<Node>) -> ExpandedName<'a> {
        let elem = target
            .downcast::<Element>()
            .expect("tried to get name of non-Element in HTML parsing");
        ExpandedName {
            ns: elem.namespace(),
            local: elem.local_name(),
        }
    }

    #[expect(unsafe_code)]
    #[cfg_attr(crown, expect(crown::unrooted_must_root))]
    fn create_element(
        &self,
        name: QualName,
        attrs: Vec<Attribute>,
        flags: ElementFlags,
    ) -> Dom<Node> {
        // TODO: https://github.com/servo/servo/issues/42839
        let mut cx = unsafe { temp_cx() };
        let cx = &mut cx;
        let attrs = attrs
            .into_iter()
            .map(|attr| ElementAttribute::new(attr.name, DOMString::from(String::from(attr.value))))
            .collect();
        let parsing_algorithm = if flags.template {
            ParsingAlgorithm::Fragment
        } else {
            self.parsing_algorithm
        };
        let element = create_element_for_token(
            name,
            attrs,
            &self.document,
            ElementCreator::ParserCreated(self.current_line.get()),
            parsing_algorithm,
            &self.custom_element_reaction_stack,
            cx,
        );
        Dom::from_ref(element.upcast())
    }

    #[cfg_attr(crown, expect(crown::unrooted_must_root))]
    fn create_comment(&self, text: StrTendril) -> Dom<Node> {
        let comment = Comment::new(
            DOMString::from(String::from(text)),
            &self.document,
            None,
            CanGc::note(),
        );
        Dom::from_ref(comment.upcast())
    }

    #[cfg_attr(crown, expect(crown::unrooted_must_root))]
    fn create_pi(&self, target: StrTendril, data: StrTendril) -> Dom<Node> {
        let doc = &*self.document;
        let pi = ProcessingInstruction::new(
            DOMString::from(String::from(target)),
            DOMString::from(String::from(data)),
            doc,
            CanGc::note(),
        );
        Dom::from_ref(pi.upcast())
    }

    fn associate_with_form(
        &self,
        target: &Dom<Node>,
        form: &Dom<Node>,
        nodes: (&Dom<Node>, Option<&Dom<Node>>),
    ) {
        let (element, prev_element) = nodes;
        let tree_node = prev_element.map_or(element, |prev| {
            if self.has_parent_node(element) {
                element
            } else {
                prev
            }
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
            control.set_form_owner_from_parser(&form, CanGc::note());
        }
    }

    #[cfg_attr(crown, expect(crown::unrooted_must_root))]
    fn append_before_sibling(&self, sibling: &Dom<Node>, new_node: NodeOrText<Dom<Node>>) {
        let parent = sibling
            .GetParentNode()
            .expect("append_before_sibling called on node without parent");

        insert(
            &parent,
            Some(sibling),
            new_node,
            self.parsing_algorithm,
            &self.custom_element_reaction_stack,
            CanGc::note(),
        );
    }

    fn parse_error(&self, msg: Cow<'static, str>) {
        debug!("Parse error: {}", msg);
    }

    fn set_quirks_mode(&self, mode: QuirksMode) {
        let mode = match mode {
            QuirksMode::Quirks => ServoQuirksMode::Quirks,
            QuirksMode::LimitedQuirks => ServoQuirksMode::LimitedQuirks,
            QuirksMode::NoQuirks => ServoQuirksMode::NoQuirks,
        };
        self.document.set_quirks_mode(mode);
    }

    #[cfg_attr(crown, expect(crown::unrooted_must_root))]
    fn append(&self, parent: &Dom<Node>, child: NodeOrText<Dom<Node>>) {
        insert(
            parent,
            None,
            child,
            self.parsing_algorithm,
            &self.custom_element_reaction_stack,
            CanGc::note(),
        );
    }

    #[cfg_attr(crown, expect(crown::unrooted_must_root))]
    fn append_based_on_parent_node(
        &self,
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

    fn append_doctype_to_document(
        &self,
        name: StrTendril,
        public_id: StrTendril,
        system_id: StrTendril,
    ) {
        let doc = &*self.document;
        let doctype = DocumentType::new(
            DOMString::from(String::from(name)),
            Some(DOMString::from(String::from(public_id))),
            Some(DOMString::from(String::from(system_id))),
            doc,
            CanGc::note(),
        );
        doc.upcast::<Node>()
            .AppendChild(doctype.upcast(), CanGc::note())
            .expect("Appending failed");
    }

    fn add_attrs_if_missing(&self, target: &Dom<Node>, attrs: Vec<Attribute>) {
        let elem = target
            .downcast::<Element>()
            .expect("tried to set attrs on non-Element in HTML parsing");
        for attr in attrs {
            elem.set_attribute_from_parser(
                attr.name,
                DOMString::from(String::from(attr.value)),
                None,
                CanGc::note(),
            );
        }
    }

    fn remove_from_parent(&self, target: &Dom<Node>) {
        if let Some(ref parent) = target.GetParentNode() {
            parent.RemoveChild(target, CanGc::note()).unwrap();
        }
    }

    fn mark_script_already_started(&self, node: &Dom<Node>) {
        let script = node.downcast::<HTMLScriptElement>();
        if let Some(script) = script {
            script.set_already_started(true)
        }
    }

    fn reparent_children(&self, node: &Dom<Node>, new_parent: &Dom<Node>) {
        while let Some(ref child) = node.GetFirstChild() {
            new_parent.AppendChild(child, CanGc::note()).unwrap();
        }
    }

    /// <https://html.spec.whatwg.org/multipage/#html-integration-point>
    /// Specifically, the `<annotation-xml>` cases.
    fn is_mathml_annotation_xml_integration_point(&self, handle: &Dom<Node>) -> bool {
        let elem = handle.downcast::<Element>().unwrap();
        elem.get_attribute(&ns!(), &local_name!("encoding"))
            .is_some_and(|attr| {
                attr.value().eq_ignore_ascii_case("text/html") ||
                    attr.value().eq_ignore_ascii_case("application/xhtml+xml")
            })
    }

    fn set_current_line(&self, line_number: u64) {
        self.current_line.set(line_number);
    }

    fn pop(&self, node: &Dom<Node>) {
        let node = DomRoot::from_ref(&**node);
        vtable_for(&node).pop();
    }

    fn allow_declarative_shadow_roots(&self, intended_parent: &Dom<Node>) -> bool {
        intended_parent.owner_doc().allow_declarative_shadow_roots()
    }

    /// <https://html.spec.whatwg.org/multipage/#parsing-main-inhead>
    /// A start tag whose tag name is "template"
    /// Attach shadow path
    fn attach_declarative_shadow(
        &self,
        host: &Dom<Node>,
        template: &Dom<Node>,
        attributes: &[Attribute],
    ) -> bool {
        attach_declarative_shadow_inner(host, template, attributes)
    }

    fn maybe_clone_an_option_into_selectedcontent(&self, option: &Self::Handle) {
        let Some(option) = option.downcast::<HTMLOptionElement>() else {
            if cfg!(debug_assertions) {
                unreachable!();
            }
            log::error!(
                "Received non-option element in maybe_clone_an_option_into_selectedcontent"
            );
            return;
        };

        option.maybe_clone_an_option_into_selectedcontent(CanGc::note())
    }
}

/// <https://html.spec.whatwg.org/multipage/#create-an-element-for-the-token>
fn create_element_for_token(
    name: QualName,
    attrs: Vec<ElementAttribute>,
    document: &Document,
    creator: ElementCreator,
    parsing_algorithm: ParsingAlgorithm,
    custom_element_reaction_stack: &CustomElementReactionStack,
    cx: &mut js::context::JSContext,
) -> DomRoot<Element> {
    // Step 1. If the active speculative HTML parser is not null, then return the result
    // of creating a speculative mock element given namespace, token's tag name, and
    // token's attributes.
    // TODO: Implement

    // Step 2: Otherwise, optionally create a speculative mock element given namespace,
    // token's tag name, and token's attributes
    // TODO: Implement.

    // Step 3. Let document be intendedParent's node document.
    // Passed as argument.

    // Step 4. Let localName be token's tag name.
    // Passed as argument

    // Step 5. Let is be the value of the "is" attribute in token, if such an attribute
    // exists; otherwise null.
    let is = attrs
        .iter()
        .find(|attr| attr.name.local.eq_str_ignore_ascii_case("is"))
        .map(|attr| LocalName::from(&attr.value));

    // Step 6. Let registry be the result of looking up a custom element registry given intendedParent.
    // TODO: Implement registries other than `Document`.

    // Step 7. Let definition be the result of looking up a custom element definition
    // given registry, namespace, localName, and is.
    let definition = document.lookup_custom_element_definition(&name.ns, &name.local, is.as_ref());

    // Step 8. Let willExecuteScript be true if definition is non-null and the parser was
    // not created as part of the HTML fragment parsing algorithm; otherwise false.
    let will_execute_script =
        definition.is_some() && parsing_algorithm != ParsingAlgorithm::Fragment;

    // Step 9. If willExecuteScript is true:
    if will_execute_script {
        // Step 9.1. Increment document's throw-on-dynamic-markup-insertion counter.
        document.increment_throw_on_dynamic_markup_insertion_counter();
        // Step 6.2. If the JavaScript execution context stack is empty, then perform a
        // microtask checkpoint.
        if is_execution_stack_empty() {
            document.window().perform_a_microtask_checkpoint(cx);
        }
        // Step 9.3. Push a new element queue onto document's relevant agent's custom
        // element reactions stack.
        custom_element_reaction_stack.push_new_element_queue()
    }

    // Step 10. Let element be the result of creating an element given document,
    // localName, namespace, null, is, willExecuteScript, and registry.
    let creation_mode = if will_execute_script {
        CustomElementCreationMode::Synchronous
    } else {
        CustomElementCreationMode::Asynchronous
    };
    let element = Element::create(
        name,
        is,
        document,
        creator,
        creation_mode,
        None,
        CanGc::from_cx(cx),
    );

    // Step 11. Append each attribute in the given token to element.
    for attr in attrs {
        element.set_attribute_from_parser(attr.name, attr.value, None, CanGc::from_cx(cx));
    }

    // Step 12. If willExecuteScript is true:
    if will_execute_script {
        // Step 12.1. Let queue be the result of popping from document's relevant agent's
        // custom element reactions stack. (This will be the same element queue as was
        // pushed above.)
        // Step 12.2 Invoke custom element reactions in queue.
        custom_element_reaction_stack.pop_current_element_queue(CanGc::from_cx(cx));
        // Step 12.3. Decrement document's throw-on-dynamic-markup-insertion counter.
        document.decrement_throw_on_dynamic_markup_insertion_counter();
    }

    // Step 13. If element has an xmlns attribute in the XMLNS namespace whose value is
    // not exactly the same as the element's namespace, that is a parse error. Similarly,
    // if element has an xmlns:xlink attribute in the XMLNS namespace whose value is not
    // the XLink Namespace, that is a parse error.
    // TODO: Implement.

    // Step 14. If element is a resettable element and not a form-associated custom
    // element, then invoke its reset algorithm. (This initializes the element's value and
    // checkedness based on the element's attributes.)
    if let Some(html_element) = element.downcast::<HTMLElement>() {
        if element.is_resettable() && !html_element.is_form_associated_custom_element() {
            element.reset(CanGc::from_cx(cx));
        }
    }

    // Step 15. If element is a form-associated element and not a form-associated custom
    // element, the form element pointer is not null, there is no template element on the
    // stack of open elements, element is either not listed or doesn't have a form attribute,
    // and the intendedParent is in the same tree as the element pointed to by the form
    // element pointer, then associate element with the form element pointed to by the form
    // element pointer and set element's parser inserted flag.
    // TODO: Implement

    // Step 16. Return element.
    element
}

fn attach_declarative_shadow_inner(host: &Node, template: &Node, attributes: &[Attribute]) -> bool {
    let host_element = host.downcast::<Element>().unwrap();

    if host_element.shadow_root().is_some() {
        return false;
    }

    let template_element = template.downcast::<HTMLTemplateElement>().unwrap();

    // Step 3. Let mode be template start tag's shadowrootmode attribute's value.
    // Step 4. Let clonable be true if template start tag has a shadowrootclonable attribute; otherwise false.
    // Step 5. Let delegatesfocus be true if template start tag
    // has a shadowrootdelegatesfocus attribute; otherwise false.
    // Step 6. Let serializable be true if template start tag
    // has a shadowrootserializable attribute; otherwise false.
    let mut shadow_root_mode = ShadowRootMode::Open;
    let mut clonable = false;
    let mut delegatesfocus = false;
    let mut serializable = false;

    let attributes: Vec<ElementAttribute> = attributes
        .iter()
        .map(|attr| {
            ElementAttribute::new(
                attr.name.clone(),
                DOMString::from(String::from(attr.value.clone())),
            )
        })
        .collect();

    attributes
        .iter()
        .for_each(|attr: &ElementAttribute| match attr.name.local {
            local_name!("shadowrootmode") => {
                if attr.value.str().eq_ignore_ascii_case("open") {
                    shadow_root_mode = ShadowRootMode::Open;
                } else if attr.value.str().eq_ignore_ascii_case("closed") {
                    shadow_root_mode = ShadowRootMode::Closed;
                } else {
                    unreachable!("shadowrootmode value is not open nor closed");
                }
            },
            local_name!("shadowrootclonable") => {
                clonable = true;
            },
            local_name!("shadowrootdelegatesfocus") => {
                delegatesfocus = true;
            },
            local_name!("shadowrootserializable") => {
                serializable = true;
            },
            _ => {},
        });

    // Step 8.1. Attach a shadow root with declarative shadow host element,
    // mode, clonable, serializable, delegatesFocus, and "named".
    match host_element.attach_shadow(
        IsUserAgentWidget::No,
        shadow_root_mode,
        clonable,
        serializable,
        delegatesfocus,
        SlotAssignmentMode::Named,
        CanGc::note(),
    ) {
        Ok(shadow_root) => {
            // Step 8.3. Set shadow's declarative to true.
            shadow_root.set_declarative(true);

            // Set 8.4. Set template's template contents property to shadow.
            let shadow = shadow_root.upcast::<DocumentFragment>();
            template_element.set_contents(Some(shadow));

            // Step 8.5. Set shadow’s available to element internals to true.
            shadow_root.set_available_to_element_internals(true);

            true
        },
        Err(_) => false,
    }
}
