/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

//! The bulk of the HTML parser integration is in `script::parse::html`.
//! This module is mostly about its interaction with DOM memory management.

use dom::bindings::cell::DOMRefCell;
use dom::bindings::codegen::Bindings::ServoHTMLParserBinding;
use dom::bindings::global::GlobalRef;
use dom::bindings::trace::JSTraceable;
use dom::bindings::js::{JS, JSRef, Temporary};
use dom::bindings::refcounted::Trusted;
use dom::bindings::utils::{Reflectable, Reflector, reflect_dom_object};
use dom::document::{Document, DocumentHelpers};
use dom::node::{window_from_node, Node};
use dom::window::Window;
use network_listener::PreInvoke;
use parse::Parser;
use script_task::{ScriptTask, ScriptChan};

use msg::constellation_msg::{PipelineId, SubpageId};
use net::resource_task::{Metadata, AsyncResponseListener};

use encoding::all::UTF_8;
use encoding::types::{Encoding, DecoderTrap};
use std::ascii::AsciiExt;
use std::borrow::ToOwned;
use std::cell::{Cell, RefCell};
use std::default::Default;
use url::Url;
use js::jsapi::JSTracer;
use html5ever::tokenizer;
use html5ever::tree_builder;
use html5ever::tree_builder::{TreeBuilder, TreeBuilderOpts};

#[must_root]
#[jstraceable]
pub struct Sink {
    pub base_url: Option<Url>,
    pub document: JS<Document>,
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
                                                     metadata).root();
        let win = parser.r().window().root();
        *self.parser.borrow_mut() = Some(Trusted::new(win.r().get_cx(), parser.r(),
                                                      self.script_chan.clone()));

        match content_type {
            Some((ref t, _)) if t.as_slice().eq_ignore_ascii_case("image") => {
                self.is_image_document.set(true);
                let page = format!("<html><body><img src='{}' /></body></html>",
                                   self.url.serialize());
                parser.r().pending_input.borrow_mut().push(page);
                parser.r().parse_sync();
            }
            _ => {}
        }
    }

    fn data_available(&self, payload: Vec<u8>) {
        // FIXME: use Vec<u8> (html5ever #34)
        let data = UTF_8.decode(payload.as_slice(), DecoderTrap::Replace).unwrap();
        let parser = self.parser.borrow().as_ref().unwrap().to_temporary().root();
        parser.r().parse_chunk(data);
    }

    fn response_complete(&self, status: Result<(), String>) {
        if let Err(err) = status {
            panic!("Failed to load page URL {}, error: {}", self.url.serialize(), err);
        }

        let parser = self.parser.borrow().as_ref().unwrap().to_temporary().root();
        parser.r().last_chunk_received.set(true);
        parser.r().parse_sync();
    }
}

impl PreInvoke for ParserContext {
    fn should_invoke(&self) -> bool {
        // We ignore the remaining image data and response status if we're synthesizing
        // a document specifically for an image.
        !self.is_image_document.get()
    }
}

// NB: JSTraceable is *not* auto-derived.
// You must edit the impl below if you add fields!
#[must_root]
#[privatize]
pub struct ServoHTMLParser {
    reflector_: Reflector,
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

impl<'a> Parser for JSRef<'a, ServoHTMLParser> {
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
    pub fn new(base_url: Option<Url>, document: JSRef<Document>, pipeline: Option<PipelineId>)
               -> Temporary<ServoHTMLParser> {
        let window = document.window().root();
        let sink = Sink {
            base_url: base_url,
            document: JS::from_rooted(document),
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
            document: JS::from_rooted(document),
            suspended: Cell::new(false),
            last_chunk_received: Cell::new(false),
            pipeline: pipeline,
        };

        reflect_dom_object(box parser, GlobalRef::Window(window.r()),
                           ServoHTMLParserBinding::Wrap)
    }

    #[inline]
    pub fn tokenizer<'a>(&'a self) -> &'a DOMRefCell<Tokenizer> {
        &self.tokenizer
    }
}

impl Reflectable for ServoHTMLParser {
    fn reflector<'a>(&'a self) -> &'a Reflector {
        &self.reflector_
    }
}

trait PrivateServoHTMLParserHelpers {
    /// Synchronously run the tokenizer parse loop until explicitly suspended or
    /// the tokenizer runs out of input.
    fn parse_sync(self);
    /// Retrieve the window object associated with this parser.
    fn window(self) -> Temporary<Window>;
}

impl<'a> PrivateServoHTMLParserHelpers for JSRef<'a, ServoHTMLParser> {
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

            let mut pending_input = self.pending_input.borrow_mut();
            let chunk = if !pending_input.is_empty() {
                pending_input.remove(0)
            } else {
                "".to_owned()
            };
            self.tokenizer.borrow_mut().feed(chunk);

            first = false;
        }

        if self.last_chunk_received.get() {
            self.finish();
        }
    }

    fn window(self) -> Temporary<Window> {
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

impl<'a> ServoHTMLParserHelpers for JSRef<'a, ServoHTMLParser> {
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

impl JSTraceable for ServoHTMLParser {
    #[allow(unsafe_blocks)]
    fn trace(&self, trc: *mut JSTracer) {
        self.reflector_.trace(trc);

        let tracer = Tracer {
            trc: trc,
        };
        let tracer = &tracer as &tree_builder::Tracer<Handle=JS<Node>>;

        unsafe {
            let tokenizer = self.tokenizer.borrow_for_gc_trace();
            let tree_builder = tokenizer.sink();
            tree_builder.trace_handles(tracer);
            tree_builder.sink().trace(trc);
        }
    }
}
