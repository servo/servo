/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use cssparser::SourceLocation;
use document_loader::LoadType;
use dom::bindings::inheritance::Castable;
use dom::bindings::refcounted::Trusted;
use dom::bindings::reflector::DomObject;
use dom::document::Document;
use dom::element::Element;
use dom::eventtarget::EventTarget;
use dom::htmlelement::HTMLElement;
use dom::htmllinkelement::{RequestGenerationId, HTMLLinkElement};
use dom::node::{document_from_node, window_from_node};
use encoding::EncodingRef;
use encoding::all::UTF_8;
use hyper::header::ContentType;
use hyper::mime::{Mime, TopLevel, SubLevel};
use hyper_serde::Serde;
use ipc_channel::ipc;
use ipc_channel::router::ROUTER;
use net_traits::{FetchResponseListener, FetchMetadata, FilteredMetadata, Metadata, NetworkError, ReferrerPolicy};
use net_traits::request::{CorsSettings, CredentialsMode, Destination, RequestInit, RequestMode, Type as RequestType};
use network_listener::{NetworkListener, PreInvoke};
use parking_lot::RwLock;
use servo_arc::Arc;
use servo_url::ServoUrl;
use std::mem;
use std::sync::Mutex;
use std::sync::atomic::AtomicBool;
use style::media_queries::MediaList;
use style::parser::ParserContext;
use style::shared_lock::{Locked, SharedRwLock};
use style::stylesheets::{CssRules, ImportRule, Namespaces, Stylesheet, StylesheetContents, Origin};
use style::stylesheets::StylesheetLoader as StyleStylesheetLoader;
use style::stylesheets::import_rule::ImportSheet;
use style::values::specified::url::SpecifiedUrl;

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

    /// Sets origin_clean flag.
    fn set_origin_clean(&self, origin_clean: bool);
}

pub enum StylesheetContextSource {
    // NB: `media` is just an option so we avoid cloning it.
    LinkElement { media: Option<MediaList>, },
    Import(Arc<Stylesheet>),
}

/// The context required for asynchronously loading an external stylesheet.
pub struct StylesheetContext {
    /// The element that initiated the request.
    elem: Trusted<HTMLElement>,
    source: StylesheetContextSource,
    url: ServoUrl,
    metadata: Option<Metadata>,
    /// The response body received to date.
    data: Vec<u8>,
    /// The node document for elem when the load was initiated.
    document: Trusted<Document>,
    origin_clean: bool,
    /// A token which must match the generation id of the `HTMLLinkElement` for it to load the stylesheet.
    /// This is ignored for `HTMLStyleElement` and imports.
    request_generation_id: Option<RequestGenerationId>,
}

impl PreInvoke for StylesheetContext {}

impl FetchResponseListener for StylesheetContext {
    fn process_request_body(&mut self) {}

    fn process_request_eof(&mut self) {}

    fn process_response(&mut self,
                        metadata: Result<FetchMetadata, NetworkError>) {
        if let Ok(FetchMetadata::Filtered { ref filtered, .. }) = metadata {
            match *filtered {
                FilteredMetadata::Opaque |
                FilteredMetadata::OpaqueRedirect => {
                    self.origin_clean = false;
                },
                _ => {},
            }
        }

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
                StylesheetContextSource::LinkElement { ref mut media } => {
                    let link = elem.downcast::<HTMLLinkElement>().unwrap();
                    // We must first check whether the generations of the context and the element match up,
                    // else we risk applying the wrong stylesheet when responses come out-of-order.
                    let is_stylesheet_load_applicable =
                        self.request_generation_id.map_or(true, |gen| gen == link.get_request_generation_id());
                    if is_stylesheet_load_applicable {
                        let shared_lock = document.style_shared_lock().clone();
                        let sheet =
                            Arc::new(Stylesheet::from_bytes(&data, final_url,
                                                            protocol_encoding_label,
                                                            Some(environment_encoding),
                                                            Origin::Author,
                                                            media.take().unwrap(),
                                                            shared_lock,
                                                            Some(&loader),
                                                            win.css_error_reporter(),
                                                            document.quirks_mode()));

                        if link.is_alternate() {
                            sheet.set_disabled(true);
                        }

                        link.set_stylesheet(sheet);
                    }
                }
                StylesheetContextSource::Import(ref stylesheet) => {
                    Stylesheet::update_from_bytes(&stylesheet,
                                                  &data,
                                                  protocol_encoding_label,
                                                  Some(environment_encoding),
                                                  final_url,
                                                  Some(&loader),
                                                  win.css_error_reporter());
                }
            }

            document.invalidate_stylesheets();

            // FIXME: Revisit once consensus is reached at:
            // https://github.com/whatwg/html/issues/1142
            successful = metadata.status.map_or(false, |(code, _)| code == 200);
        }

        let owner = elem.upcast::<Element>().as_stylesheet_owner()
            .expect("Stylesheet not loaded by <style> or <link> element!");
        owner.set_origin_clean(self.origin_clean);
        if owner.parser_inserted() {
            document.decrement_script_blocking_stylesheet_count();
        }

        document.finish_load(LoadType::Stylesheet(self.url.clone()));

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
    pub fn load(&self, source: StylesheetContextSource, url: ServoUrl,
                cors_setting: Option<CorsSettings>,
                integrity_metadata: String) {
        let document = document_from_node(self.elem);
        let gen = self.elem.downcast::<HTMLLinkElement>()
                           .map(HTMLLinkElement::get_request_generation_id);
        let context = ::std::sync::Arc::new(Mutex::new(StylesheetContext {
            elem: Trusted::new(&*self.elem),
            source: source,
            url: url.clone(),
            metadata: None,
            data: vec![],
            document: Trusted::new(&*document),
            origin_clean: true,
            request_generation_id: gen,
        }));

        let (action_sender, action_receiver) = ipc::channel().unwrap();
        let listener = NetworkListener {
            context: context,
            task_source: document.window().networking_task_source(),
            canceller: Some(document.window().task_canceller())
        };
        ROUTER.add_route(action_receiver.to_opaque(), Box::new(move |message| {
            listener.notify_fetch(message.to().unwrap());
        }));


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
            // https://html.spec.whatwg.org/multipage/#create-a-potential-cors-request
            // Step 1
            mode: match cors_setting {
                Some(_) => RequestMode::CorsMode,
                None => RequestMode::NoCors,
            },
            // https://html.spec.whatwg.org/multipage/#create-a-potential-cors-request
            // Step 3-4
            credentials_mode: match cors_setting {
                Some(CorsSettings::Anonymous) => CredentialsMode::CredentialsSameOrigin,
                _ => CredentialsMode::Include,
            },
            origin: document.origin().immutable().clone(),
            pipeline_id: Some(self.elem.global().pipeline_id()),
            referrer_url: Some(document.url()),
            referrer_policy: referrer_policy,
            integrity_metadata: integrity_metadata,
            .. RequestInit::default()
        };

        document.fetch_async(LoadType::Stylesheet(url), request, action_sender);
    }
}

impl<'a> StyleStylesheetLoader for StylesheetLoader<'a> {
    /// Request a stylesheet after parsing a given `@import` rule, and return
    /// the constructed `@import` rule.
    fn request_stylesheet(
        &self,
        url: SpecifiedUrl,
        source_location: SourceLocation,
        context: &ParserContext,
        lock: &SharedRwLock,
        media: Arc<Locked<MediaList>>,
    ) -> Arc<Locked<ImportRule>> {
        let sheet = Arc::new(Stylesheet {
            contents: StylesheetContents {
                rules: CssRules::new(Vec::new(), lock),
                origin: context.stylesheet_origin,
                url_data: RwLock::new(context.url_data.clone()),
                quirks_mode: context.quirks_mode,
                namespaces: RwLock::new(Namespaces::default()),
                source_map_url: RwLock::new(None),
                source_url: RwLock::new(None),
            },
            media: media,
            shared_lock: lock.clone(),
            disabled: AtomicBool::new(false),
        });

        let stylesheet = ImportSheet(sheet.clone());
        let import = ImportRule { url, source_location, stylesheet };

        let url = match import.url.url().cloned() {
            Some(url) => url,
            None => return Arc::new(lock.wrap(import)),
        };

        // TODO (mrnayak) : Whether we should use the original loader's CORS
        // setting? Fix this when spec has more details.
        let source = StylesheetContextSource::Import(sheet.clone());
        self.load(source, url, None, "".to_owned());

        Arc::new(lock.wrap(import))
    }
}
