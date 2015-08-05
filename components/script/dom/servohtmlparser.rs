/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

//! The bulk of the HTML parser integration is in `script::parse::html`.
//! This module is mostly about its interaction with DOM memory management.

use document_loader::LoadType;
use dom::bindings::cell::DOMRefCell;
use dom::bindings::codegen::Bindings::ServoHTMLParserBinding;
use dom::bindings::global::GlobalRef;
use dom::bindings::trace::JSTraceable;
use dom::bindings::js::{JS, Root};
use dom::bindings::refcounted::Trusted;
use dom::bindings::utils::{Reflector, reflect_dom_object};
use dom::document::{Document, DocumentHelpers};
use dom::node::{window_from_node, Node};
use dom::window::Window;
use network_listener::PreInvoke;
use parse::Parser;
use script_task::{ScriptTask, ScriptChan};

use msg::constellation_msg::{PipelineId, SubpageId};
use net_traits::{Metadata, AsyncResponseListener};

use encoding::all::UTF_8;
use encoding::types::{Encoding, DecoderTrap};
use std::cell::{Cell, RefCell};
use std::default::Default;
use url::Url;
use js::jsapi::JSTracer;
use html5ever::tokenizer;
use html5ever::tree_builder;
use html5ever::tree_builder::{TreeBuilder, TreeBuilderOpts};
use hyper::header::ContentType;
use hyper::mime::{Mime, TopLevel, SubLevel};

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

/// The context required for asynchronously fetching a document and parsing it progressively.
pub struct ParserContext {
    /// The parser that initiated the request.
    parser: RefCell<Option<Trusted<ServoHTMLParser>>>,
    /// Is this document a synthesized document for a single image?
    is_image_document: Cell<bool>,
    /// The pipeline associated with this document.
    id: PipelineId,
    /// The subpage associated with this document.
    subpage: Option<SubpageId>,
    /// The target event loop for the response notifications.
    script_chan: Box<ScriptChan+Send>,
    /// The URL for this document.
    url: Url,
}

impl ParserContext {
    pub fn new(id: PipelineId, subpage: Option<SubpageId>, script_chan: Box<ScriptChan+Send>,
               url: Url) -> ParserContext {
        ParserContext {
            parser: RefCell::new(None),
            is_image_document: Cell::new(false),
            id: id,
            subpage: subpage,
            script_chan: script_chan,
            url: url,
        }
    }
}

impl AsyncResponseListener for ParserContext {
    fn headers_available(&self, metadata: Metadata) {
        let content_type = metadata.content_type.clone();

        let parser = ScriptTask::page_fetch_complete(self.id.clone(), self.subpage.clone(),
                                                     metadata);
        let parser = match parser {
            Some(parser) => parser,
            None => return,
        };

        let parser = parser.r();
        let win = parser.window();
        *self.parser.borrow_mut() = Some(Trusted::new(win.r().get_cx(), parser,
                                                      self.script_chan.clone()));

        match content_type {
            Some(ContentType(Mime(TopLevel::Image, _, _))) => {
                self.is_image_document.set(true);
                let page = format!("<html><body><img src='{}' /></body></html>",
                                   self.url.serialize());
                parser.pending_input.borrow_mut().push(page);
                parser.parse_sync();
            }
            Some(ContentType(Mime(TopLevel::Text, SubLevel::Plain, _))) => {
                // FIXME: When servo/html5ever#109 is fixed remove <plaintext> usage and
                // replace with fix from that issue.

                // text/plain documents require setting the tokenizer into PLAINTEXT mode.
                // This is done by using a <plaintext> element as the html5ever tokenizer
                // provides no other way to change to that state.
                // Spec for text/plain handling is:
                // https://html.spec.whatwg.org/multipage/#read-text
                let page = format!("<pre>\u{000A}<plaintext>");
                parser.pending_input.borrow_mut().push(page);
                parser.parse_sync();
            },
            _ => {}
        }
    }

    fn data_available(&self, payload: Vec<u8>) {
        if !self.is_image_document.get() {
            // FIXME: use Vec<u8> (html5ever #34)
            let data = UTF_8.decode(&payload, DecoderTrap::Replace).unwrap();
            let parser = match self.parser.borrow().as_ref() {
                Some(parser) => parser.root(),
                None => return,
            };
            parser.r().parse_chunk(data);
        }
    }

    fn response_complete(&self, status: Result<(), String>) {
        let parser = match self.parser.borrow().as_ref() {
            Some(parser) => parser.root(),
            None => return,
        };
        let doc = parser.r().document.root();
        doc.r().finish_load(LoadType::PageSource(self.url.clone()));

        if let Err(err) = status {
            debug!("Failed to load page URL {}, error: {}", self.url.serialize(), err);
            // TODO(Savago): we should send a notification to callers #5463.
        }

        parser.r().last_chunk_received.set(true);
        parser.r().parse_sync();
    }
}

impl PreInvoke for ParserContext {
}

#[dom_struct]
#[derive(HeapSizeOf)]
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
        self.document.root().r().set_current_parser(Some(self));
        self.pending_input.borrow_mut().push(input);
        self.parse_sync();
    }

    fn finish(self) {
        assert!(!self.suspended.get());
        assert!(self.pending_input.borrow().is_empty());

        self.tokenizer().borrow_mut().end();
        debug!("finished parsing");

        let document = self.document.root();
        document.r().set_current_parser(None);

        if let Some(pipeline) = self.pipeline {
            ScriptTask::parsing_complete(pipeline);
        }
    }
}

impl ServoHTMLParser {
    #[allow(unrooted_must_root)]
    pub fn new(base_url: Option<Url>, document: &Document, pipeline: Option<PipelineId>)
               -> Root<ServoHTMLParser> {
        let window = document.window();
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

        reflect_dom_object(box parser, GlobalRef::Window(window.r()),
                           ServoHTMLParserBinding::Wrap)
    }

    #[allow(unrooted_must_root)]
    pub fn new_for_fragment(base_url: Option<Url>, document: &Document,
                            fragment_context: FragmentContext) -> Root<ServoHTMLParser> {
        let window = document.window();
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

        reflect_dom_object(box parser, GlobalRef::Window(window.r()),
                           ServoHTMLParserBinding::Wrap)
    }

    #[inline]
    pub fn tokenizer<'a>(&'a self) -> &'a DOMRefCell<Tokenizer> {
        &self.tokenizer
    }
}

trait PrivateServoHTMLParserHelpers {
    /// Synchronously run the tokenizer parse loop until explicitly suspended or
    /// the tokenizer runs out of input.
    fn parse_sync(self);
    /// Retrieve the window object associated with this parser.
    fn window(self) -> Root<Window>;
}

impl<'a> PrivateServoHTMLParserHelpers for &'a ServoHTMLParser {
    fn parse_sync(self) {
        let mut first = true;

        // This parser will continue to parse while there is either pending input or
        // the parser remains unsuspended.
        loop {
            if self.suspended.get() {
                return;
            }

            if self.pending_input.borrow().is_empty() && !first {
                break;
            }

            let document = self.document.root();
            document.r().reflow_if_reflow_timer_expired();

            let mut pending_input = self.pending_input.borrow_mut();
            if !pending_input.is_empty() {
                let chunk = pending_input.remove(0);
                self.tokenizer.borrow_mut().feed(chunk.into());
            } else {
                self.tokenizer.borrow_mut().run();
            }

            first = false;
        }

        if self.last_chunk_received.get() {
            self.finish();
        }
    }

    fn window(self) -> Root<Window> {
        let doc = self.document.root();
        window_from_node(doc.r())
    }
}

pub trait ServoHTMLParserHelpers {
    /// Cause the parser to interrupt next time the tokenizer reaches a quiescent state.
    /// No further parsing will occur after that point until the `resume` method is called.
    /// Panics if the parser is already suspended.
    fn suspend(self);
    /// Immediately resume a suspended parser. Panics if the parser is not suspended.
    fn resume(self);
}

impl<'a> ServoHTMLParserHelpers for &'a ServoHTMLParser {
    fn suspend(self) {
        assert!(!self.suspended.get());
        self.suspended.set(true);
    }

    fn resume(self) {
        assert!(self.suspended.get());
        self.suspended.set(false);
        self.parse_sync();
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
