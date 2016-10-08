/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use document_loader::LoadType;
use dom::bindings::codegen::Bindings::DocumentBinding::DocumentMethods;
use dom::bindings::codegen::Bindings::HTMLImageElementBinding::HTMLImageElementMethods;
use dom::bindings::codegen::Bindings::NodeBinding::NodeMethods;
use dom::bindings::inheritance::Castable;
use dom::bindings::js::{JS, Root};
use dom::bindings::refcounted::Trusted;
use dom::bindings::str::DOMString;
use dom::htmlimageelement::HTMLImageElement;
use dom::node::Node;
use dom::servohtmlparser::ServoHTMLParser;
use dom::servoparser::ServoParser;
use dom::servoxmlparser::ServoXMLParser;
use dom::window::Window;
use encoding::all::UTF_8;
use encoding::types::{DecoderTrap, Encoding};
use hyper::header::ContentType;
use hyper::mime::{Mime, SubLevel, TopLevel};
use hyper_serde::Serde;
use msg::constellation_msg::PipelineId;
use net_traits::{AsyncResponseListener, Metadata, NetworkError};
use network_listener::PreInvoke;
use script_thread::ScriptThread;
use std::cell::UnsafeCell;
use std::ptr;
use url::Url;
use util::resource_files::read_resource_file;

pub mod html;
pub mod xml;

pub trait Parser {
    fn parse_chunk(self, input: String);
    fn finish(self);
}

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

pub enum TrustedParser {
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
    pub fn as_servo_parser(&self) -> &ServoParser {
        match *self {
            ParserRef::HTML(parser) => parser.upcast(),
            ParserRef::XML(parser) => parser.upcast(),
        }
    }

    pub fn parse_chunk(&self, input: String) {
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
}

/// The context required for asynchronously fetching a document
/// and parsing it progressively.
pub struct ParserContext {
    /// The parser that initiated the request.
    parser: Option<TrustedParser>,
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

impl AsyncResponseListener for ParserContext {
    fn headers_available(&mut self, meta_result: Result<Metadata, NetworkError>) {
        let mut ssl_error = None;
        let metadata = match meta_result {
            Ok(meta) => Some(meta),
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

        let parser = parser.r();
        let servo_parser = parser.as_servo_parser();
        self.parser = Some(match parser {
            ParserRef::HTML(parser) => TrustedParser::HTML(
                                        Trusted::new(parser)),
            ParserRef::XML(parser) => TrustedParser::XML(
                                        Trusted::new(parser)),
        });

        match content_type {
            Some(ContentType(Mime(TopLevel::Image, _, _))) => {
                self.is_synthesized_document = true;
                let page = "<html><body></body></html>".into();
                servo_parser.push_input_chunk(page);
                parser.parse_sync();

                let doc = servo_parser.document();
                let doc_body = Root::upcast::<Node>(doc.GetBody().unwrap());
                let img = HTMLImageElement::new(atom!("img"), None, doc);
                img.SetSrc(DOMString::from(self.url.to_string()));
                doc_body.AppendChild(&Root::upcast::<Node>(img)).expect("Appending failed");

            },
            Some(ContentType(Mime(TopLevel::Text, SubLevel::Plain, _))) => {
                // https://html.spec.whatwg.org/multipage/#read-text
                let page = "<pre>\n".into();
                servo_parser.push_input_chunk(page);
                parser.parse_sync();
                parser.set_plaintext_state();
            },
            Some(ContentType(Mime(TopLevel::Text, SubLevel::Html, _))) => { // Handle text/html
                if let Some(reason) = ssl_error {
                    self.is_synthesized_document = true;
                    let page_bytes = read_resource_file("badcert.html").unwrap();
                    let page = String::from_utf8(page_bytes).unwrap();
                    let page = page.replace("${reason}", &reason);
                    servo_parser.push_input_chunk(page);
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
                servo_parser.push_input_chunk(page);
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

    fn response_complete(&mut self, status: Result<(), NetworkError>) {
        let parser = match self.parser.as_ref() {
            Some(parser) => parser.root(),
            None => return,
        };

        if let Err(NetworkError::Internal(ref reason)) = status {
            // Show an error page for network errors,
            // certificate errors are handled earlier.
            self.is_synthesized_document = true;
            let parser = parser.r();
            let page_bytes = read_resource_file("neterror.html").unwrap();
            let page = String::from_utf8(page_bytes).unwrap();
            let page = page.replace("${reason}", reason);
            parser.as_servo_parser().push_input_chunk(page);
            parser.parse_sync();
        } else if let Err(err) = status {
            // TODO(Savago): we should send a notification to callers #5463.
            debug!("Failed to load page URL {}, error: {:?}", self.url, err);
        }

        let parser = parser.r();
        let servo_parser = parser.as_servo_parser();

        servo_parser.document()
            .finish_load(LoadType::PageSource(self.url.clone()));

        servo_parser.mark_last_chunk_received();
        if !parser.is_suspended() {
            parser.parse_sync();
        }
    }
}

impl PreInvoke for ParserContext {}
