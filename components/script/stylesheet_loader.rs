/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use document_loader::LoadType;
use dom::bindings::codegen::Bindings::DOMTokenListBinding::DOMTokenListBinding::DOMTokenListMethods;
use dom::bindings::codegen::Bindings::HTMLLinkElementBinding::HTMLLinkElementBinding::HTMLLinkElementMethods;
use dom::bindings::inheritance::Castable;
use dom::bindings::refcounted::Trusted;
use dom::bindings::reflector::DomObject;
use dom::eventtarget::EventTarget;
use dom::htmlelement::HTMLElement;
use dom::htmllinkelement::HTMLLinkElement;
use dom::htmlstyleelement::HTMLStyleElement;
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

pub enum StylesheetContextSource {
    LinkElement { media: MediaList, url: ServoUrl },
    Import(Arc<RwLock<ImportRule>>),
}

/// The context required for asynchronously loading an external stylesheet.
pub struct StylesheetContext {
    /// The element that initiated the request.
    elem: Trusted<HTMLElement>,
    source: StylesheetContextSource,
    metadata: Option<Metadata>,
    /// The response body received to date.
    data: Vec<u8>,
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
        if let Some(ref meta) = self.metadata {
            if let Some(Serde(ContentType(Mime(TopLevel::Text, SubLevel::Css, _)))) = meta.content_type {
            } else if let StylesheetContextSource::LinkElement { .. } = self.source {
                self.elem.root().upcast::<EventTarget>().fire_event(atom!("error"));
            }
        }
    }

    fn process_response_chunk(&mut self, mut payload: Vec<u8>) {
        self.data.append(&mut payload);
    }

    fn process_response_eof(&mut self, status: Result<(), NetworkError>) {
        let elem = self.elem.root();
        let document = document_from_node(&*elem);
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
                StylesheetContextSource::LinkElement { ref media, .. } => {
                    let sheet =
                        Arc::new(Stylesheet::from_bytes(&data, final_url,
                                                        protocol_encoding_label,
                                                        Some(environment_encoding),
                                                        Origin::Author,
                                                        media.clone(),
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

            // FIXME: Revisit once consensus is reached at: https://github.com/whatwg/html/issues/1142
            successful = metadata.status.map_or(false, |(code, _)| code == 200);
        }

        if let Some(ref link) = elem.downcast::<HTMLLinkElement>() {
            if link.parser_inserted() {
                document.decrement_script_blocking_stylesheet_count();
            }
        } else if let Some(ref style) = elem.downcast::<HTMLStyleElement>() {
            if style.parser_inserted() {
                document.decrement_script_blocking_stylesheet_count();
            }
        }

        let url = match self.source {
            StylesheetContextSource::LinkElement { ref url, .. } => url.clone(),
            StylesheetContextSource::Import(ref import) => {
                let import = import.read();
                import.url.url().expect("Tried to load an invalid url").clone()
            }
        };

        document.finish_load(LoadType::Stylesheet(url));

        if let Some(ref link) = elem.downcast::<HTMLLinkElement>() {
            if link.decrement_pending_loads_count() {
                let event = if successful { atom!("load") } else { atom!("error") };
                link.upcast::<EventTarget>().fire_event(event);
            }
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
        let url = match source {
            StylesheetContextSource::Import(ref import) => {
                let import = import.read();
                import.url.url().expect("Requested import with invalid url!").clone()
            }
            StylesheetContextSource::LinkElement { ref url, .. } => {
                url.clone()
            }
        };

        let context = Arc::new(Mutex::new(StylesheetContext {
            elem: Trusted::new(&*self.elem),
            source: source,
            metadata: None,
            data: vec![],
        }));

        let document = document_from_node(self.elem);

        let (action_sender, action_receiver) = ipc::channel().unwrap();
        let listener = NetworkListener {
            context: context,
            task_source: document.window().networking_task_source(),
            wrapper: Some(document.window().get_runnable_wrapper())
        };
        ROUTER.add_route(action_receiver.to_opaque(), box move |message| {
            listener.notify_fetch(message.to().unwrap());
        });


        let mut referrer_policy = document.get_referrer_policy();
        if let Some(ref link) = self.elem.downcast::<HTMLLinkElement>() {
            link.increment_pending_loads_count();
            if link.parser_inserted() {
                document.increment_script_blocking_stylesheet_count();
            }
            if link.RelList().Contains("noreferrer".into()) {
                referrer_policy = Some(ReferrerPolicy::NoReferrer);
            }
        } else if let Some(ref style) = self.elem.downcast::<HTMLStyleElement>() {
            if style.parser_inserted() {
                document.increment_script_blocking_stylesheet_count();
            }
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
