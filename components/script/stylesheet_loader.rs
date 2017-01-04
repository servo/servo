/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use document_loader::LoadType;
use dom::bindings::inheritance::Castable;
use dom::bindings::refcounted::Trusted;
use dom::bindings::reflector::DomObject;
use dom::document::Document;
use dom::element::Element;
use dom::eventtarget::EventTarget;
use dom::htmlelement::HTMLElement;
use dom::htmllinkelement::HTMLLinkElement;
use dom::node::{document_from_node, window_from_node};
use encoding::EncodingRef;
use encoding::all::UTF_8;
use hyper::header::ContentType;
use hyper::mime::{Mime, TopLevel, SubLevel};
use hyper_serde::Serde;
use ipc_channel::ipc;
use ipc_channel::router::ROUTER;
use net_traits::{FetchResponseListener, FetchMetadata, Metadata, NetworkError, ReferrerPolicy};
use net_traits::request::{CredentialsMode, Destination, RequestInit, Type as RequestType};
use network_listener::{NetworkListener, PreInvoke};
use parking_lot::RwLock;
use script_layout_interface::message::Msg;
use servo_url::ServoUrl;
use std::mem;
use std::sync::{Arc, Mutex};
use style::media_queries::MediaList;
use style::parser::ParserContextExtraData;
use style::stylesheets::{ImportRule, Stylesheet, Origin};
use style::stylesheets::StylesheetLoader as StyleStylesheetLoader;

pub trait StylesheetOwner {
    /// Returns whether this element was inserted by the parser (i.e., it should
    /// trigger a document-load-blocking load).
    fn parser_inserted(&self) -> bool;

    /// Which referrer policy should loads triggered by this owner follow, or
    /// `None` for the default.
    fn referrer_policy(&self) -> Option<ReferrerPolicy>;

    /// Notes that a new load is pending to finish.
    fn increment_pending_loads_count(&self);

    /// Returns None if there are still pending loads, or whether any load has
    /// failed since the loads started.
    fn load_finished(&self, successful: bool) -> Option<bool>;
}

pub enum StylesheetContextSource {
    // NB: `media` is just an option so we avoid cloning it.
    LinkElement { media: Option<MediaList>, url: ServoUrl },
    Import(Arc<RwLock<ImportRule>>),
}

impl StylesheetContextSource {
    fn url(&self) -> ServoUrl {
        match *self {
            StylesheetContextSource::LinkElement { ref url, .. } => url.clone(),
            StylesheetContextSource::Import(ref import) => {
                let import = import.read();
                // Look at the parser in style::stylesheets, where we don't
                // trigger a load if the url is invalid.
                import.url.url()
                    .expect("Invalid urls shouldn't enter the loader")
                    .clone()
            }
        }
    }
}

/// The context required for asynchronously loading an external stylesheet.
pub struct StylesheetContext {
    /// The element that initiated the request.
    elem: Trusted<HTMLElement>,
    source: StylesheetContextSource,
    metadata: Option<Metadata>,
    /// The response body received to date.
    data: Vec<u8>,
    /// The node document for elem when the load was initiated.
    document: Trusted<Document>,
}

impl PreInvoke for StylesheetContext {}

impl FetchResponseListener for StylesheetContext {
    fn process_request_body(&mut self) {}

    fn process_request_eof(&mut self) {}

    fn process_response(&mut self,
                        metadata: Result<FetchMetadata, NetworkError>) {
        self.metadata = metadata.ok().map(|m| {
            match m {
                FetchMetadata::Unfiltered(m) => m,
                FetchMetadata::Filtered { unsafe_, .. } => unsafe_
            }
        });
    }

    fn process_response_chunk(&mut self, mut payload: Vec<u8>) {
        self.data.append(&mut payload);
    }

    fn process_response_eof(&mut self, status: Result<(), NetworkError>) {
        let elem = self.elem.root();
        let document = self.document.root();
        let mut successful = false;

        if status.is_ok() {
            let metadata = match self.metadata.take() {
                Some(meta) => meta,
                None => return,
            };
            let is_css = metadata.content_type.map_or(false, |Serde(ContentType(Mime(top, sub, _)))|
                top == TopLevel::Text && sub == SubLevel::Css);

            let data = if is_css { mem::replace(&mut self.data, vec![]) } else { vec![] };

            // TODO: Get the actual value. http://dev.w3.org/csswg/css-syntax/#environment-encoding
            let environment_encoding = UTF_8 as EncodingRef;
            let protocol_encoding_label = metadata.charset.as_ref().map(|s| &**s);
            let final_url = metadata.final_url;

            let win = window_from_node(&*elem);

            let loader = StylesheetLoader::for_element(&elem);
            match self.source {
                StylesheetContextSource::LinkElement { ref mut media, .. } => {
                    let sheet =
                        Arc::new(Stylesheet::from_bytes(&data, final_url,
                                                        protocol_encoding_label,
                                                        Some(environment_encoding),
                                                        Origin::Author,
                                                        media.take().unwrap(),
                                                        Some(&loader),
                                                        win.css_error_reporter(),
                                                        ParserContextExtraData::default()));
                    elem.downcast::<HTMLLinkElement>()
                        .unwrap()
                        .set_stylesheet(sheet.clone());

                    let win = window_from_node(&*elem);
                    win.layout_chan().send(Msg::AddStylesheet(sheet)).unwrap();
                }
                StylesheetContextSource::Import(ref import) => {
                    let import = import.read();
                    Stylesheet::update_from_bytes(&import.stylesheet,
                                                  &data,
                                                  protocol_encoding_label,
                                                  Some(environment_encoding),
                                                  Some(&loader),
                                                  win.css_error_reporter(),
                                                  ParserContextExtraData::default());
                }
            }

            document.invalidate_stylesheets();

            // FIXME: Revisit once consensus is reached at:
            // https://github.com/whatwg/html/issues/1142
            successful = metadata.status.map_or(false, |(code, _)| code == 200);
        }

        let owner = elem.upcast::<Element>().as_stylesheet_owner()
            .expect("Stylesheet not loaded by <style> or <link> element!");
        if owner.parser_inserted() {
            document.decrement_script_blocking_stylesheet_count();
        }

        let url = self.source.url();
        document.finish_load(LoadType::Stylesheet(url));

        if let Some(any_failed) = owner.load_finished(successful) {
            let event = if any_failed { atom!("error") } else { atom!("load") };
            elem.upcast::<EventTarget>().fire_event(event);
        }
    }
}

pub struct StylesheetLoader<'a> {
    elem: &'a HTMLElement,
}

impl<'a> StylesheetLoader<'a> {
    pub fn for_element(element: &'a HTMLElement) -> Self {
        StylesheetLoader {
            elem: element,
        }
    }
}

impl<'a> StylesheetLoader<'a> {
    pub fn load(&self, source: StylesheetContextSource) {
        let url = source.url();
        let document = document_from_node(self.elem);
        let context = Arc::new(Mutex::new(StylesheetContext {
            elem: Trusted::new(&*self.elem),
            source: source,
            metadata: None,
            data: vec![],
            document: Trusted::new(&*document),
        }));

        let (action_sender, action_receiver) = ipc::channel().unwrap();
        let listener = NetworkListener {
            context: context,
            task_source: document.window().networking_task_source(),
            wrapper: Some(document.window().get_runnable_wrapper())
        };
        ROUTER.add_route(action_receiver.to_opaque(), box move |message| {
            listener.notify_fetch(message.to().unwrap());
        });


        let owner = self.elem.upcast::<Element>().as_stylesheet_owner()
            .expect("Stylesheet not loaded by <style> or <link> element!");
        let referrer_policy = owner.referrer_policy()
            .or_else(|| document.get_referrer_policy());
        owner.increment_pending_loads_count();
        if owner.parser_inserted() {
            document.increment_script_blocking_stylesheet_count();
        }

        let request = RequestInit {
            url: url.clone(),
            type_: RequestType::Style,
            destination: Destination::Style,
            credentials_mode: CredentialsMode::Include,
            use_url_credentials: true,
            origin: document.url(),
            pipeline_id: Some(self.elem.global().pipeline_id()),
            referrer_url: Some(document.url()),
            referrer_policy: referrer_policy,
            .. RequestInit::default()
        };

        document.fetch_async(LoadType::Stylesheet(url), request, action_sender);
    }
}

impl<'a> StyleStylesheetLoader for StylesheetLoader<'a> {
    fn request_stylesheet(&self, import: &Arc<RwLock<ImportRule>>) {
        self.load(StylesheetContextSource::Import(import.clone()))
    }
}
