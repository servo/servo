/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use document_loader::LoadType;
use dom::bindings::codegen::Bindings::DocumentBinding::DocumentMethods;
use dom::bindings::codegen::Bindings::HTMLImageElementBinding::HTMLImageElementMethods;
use dom::bindings::codegen::Bindings::NodeBinding::NodeMethods;
use dom::bindings::js::{JS, Root};
use dom::bindings::refcounted::Trusted;
use dom::bindings::str::DOMString;
use dom::document::Document;
use dom::htmlimageelement::HTMLImageElement;
use dom::node::Node;
use dom::servoparser::ServoParser;
use encoding::all::UTF_8;
use encoding::types::{DecoderTrap, Encoding};
use hyper::header::ContentType;
use hyper::mime::{Mime, SubLevel, TopLevel};
use hyper_serde::Serde;
use msg::constellation_msg::PipelineId;
use net_traits::{AsyncResponseListener, Metadata, NetworkError};
use network_listener::PreInvoke;
use script_thread::ScriptThread;
use url::Url;
use util::resource_files::read_resource_file;

pub mod html;
pub mod xml;

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

        self.parser = Some(Trusted::new(&*parser));

        match content_type {
            Some(ContentType(Mime(TopLevel::Image, _, _))) => {
                self.is_synthesized_document = true;
                let page = "<html><body></body></html>".into();
                parser.push_input_chunk(page);
                parser.parse_sync();

                let doc = parser.document();
                let doc_body = Root::upcast::<Node>(doc.GetBody().unwrap());
                let img = HTMLImageElement::new(atom!("img"), None, doc);
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

    fn data_available(&mut self, payload: Vec<u8>) {
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

    fn response_complete(&mut self, status: Result<(), NetworkError>) {
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

#[derive(JSTraceable, HeapSizeOf)]
#[must_root]
pub struct Sink {
    pub base_url: Url,
    pub document: JS<Document>,
}
