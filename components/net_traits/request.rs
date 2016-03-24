/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use hyper::header::Headers;
use hyper::method::Method;
use std::cell::{Cell, RefCell};
use url::{Origin as UrlOrigin, Url};

/// An [initiator](https://fetch.spec.whatwg.org/#concept-request-initiator)
#[derive(Copy, Clone, PartialEq)]
pub enum Initiator {
    None,
    Download,
    ImageSet,
    Manifest,
    XSLT
}

/// A request [type](https://fetch.spec.whatwg.org/#concept-request-type)
#[derive(Copy, Clone, PartialEq)]
pub enum Type {
    None, Audio, Font, Image,
    Script, Style, Track, Video
}

/// A request [destination](https://fetch.spec.whatwg.org/#concept-request-destination)
#[derive(Copy, Clone, PartialEq)]
pub enum Destination {
    None, Document, Embed, Font, Image, Manifest,
    Media, Object, Report, Script, ServiceWorker,
    SharedWorker, Style, Worker, XSLT
}

/// A request [origin](https://fetch.spec.whatwg.org/#concept-request-origin)
#[derive(Clone, PartialEq)]
pub enum Origin {
    Client,
    Origin(UrlOrigin)
}

/// A [referer](https://fetch.spec.whatwg.org/#concept-request-referrer)
#[derive(Clone, PartialEq)]
pub enum Referer {
    NoReferer,
    Client,
    RefererUrl(Url)
}

/// A [request mode](https://fetch.spec.whatwg.org/#concept-request-mode)
#[derive(Copy, Clone, PartialEq)]
pub enum RequestMode {
    Navigate,
    SameOrigin,
    NoCORS,
    CORSMode
}

/// Request [credentials mode](https://fetch.spec.whatwg.org/#concept-request-credentials-mode)
#[derive(Copy, Clone, PartialEq)]
pub enum CredentialsMode {
    Omit,
    CredentialsSameOrigin,
    Include
}

/// [Cache mode](https://fetch.spec.whatwg.org/#concept-request-cache-mode)
#[derive(Copy, Clone, PartialEq)]
pub enum CacheMode {
    Default,
    NoStore,
    Reload,
    NoCache,
    ForceCache,
    OnlyIfCached
}

/// [Redirect mode](https://fetch.spec.whatwg.org/#concept-request-redirect-mode)
#[derive(Copy, Clone, PartialEq)]
pub enum RedirectMode {
    Follow,
    Error,
    Manual
}

/// [Response tainting](https://fetch.spec.whatwg.org/#concept-request-response-tainting)
#[derive(Copy, Clone, PartialEq)]
pub enum ResponseTainting {
    Basic,
    CORSTainting,
    Opaque
}

/// [Window](https://fetch.spec.whatwg.org/#concept-request-window)
#[derive(Copy, Clone, PartialEq)]
pub enum Window {
    NoWindow,
    Client,
    // TODO: Environmental settings object
}

/// [CORS settings attribute](https://html.spec.whatwg.org/multipage/#attr-crossorigin-anonymous)
#[derive(Copy, Clone, PartialEq)]
pub enum CORSSettings {
    Anonymous,
    UseCredentials
}

/// A [Request](https://fetch.spec.whatwg.org/#requests) as defined by the Fetch spec
#[derive(Clone)]
pub struct Request {
    pub method: RefCell<Method>,
    pub local_urls_only: bool,
    pub sandboxed_storage_area_urls: bool,
    pub headers: RefCell<Headers>,
    pub unsafe_request: bool,
    pub body: RefCell<Option<Vec<u8>>>,
    // TODO: client object
    pub is_service_worker_global_scope: bool,
    // pub client: GlobalRef, // XXXManishearth copy over only the relevant fields of the global scope,
                              // not the entire scope to avoid the libscript dependency
    pub window: Cell<Window>,
    // TODO: target browsing context
    pub keep_alive: Cell<bool>,
    pub skip_service_worker: Cell<bool>,
    pub initiator: Initiator,
    pub type_: Type,
    pub destination: Destination,
    // TODO: priority object
    pub origin: RefCell<Origin>,
    pub omit_origin_header: Cell<bool>,
    pub same_origin_data: Cell<bool>,
    pub referer: Referer,
    // TODO: referrer policy
    pub synchronous: bool,
    pub mode: RequestMode,
    pub use_cors_preflight: bool,
    pub credentials_mode: CredentialsMode,
    pub use_url_credentials: bool,
    pub cache_mode: Cell<CacheMode>,
    pub redirect_mode: Cell<RedirectMode>,
    pub integrity_metadata: RefCell<String>,
    // Use the last method on url_list to act as spec current url field, and
    // first method to act as spec url field
    pub url_list: RefCell<Vec<Url>>,
    pub redirect_count: Cell<u32>,
    pub response_tainting: Cell<ResponseTainting>,
    pub done: Cell<bool>
}

impl Request {
    pub fn new(url: Url,
               origin: Option<Origin>,
               is_service_worker_global_scope: bool) -> Request {
        Request {
            method: RefCell::new(Method::Get),
            local_urls_only: false,
            sandboxed_storage_area_urls: false,
            headers: RefCell::new(Headers::new()),
            unsafe_request: false,
            body: RefCell::new(None),
            is_service_worker_global_scope: is_service_worker_global_scope,
            window: Cell::new(Window::Client),
            keep_alive: Cell::new(false),
            skip_service_worker: Cell::new(false),
            initiator: Initiator::None,
            type_: Type::None,
            destination: Destination::None,
            origin: RefCell::new(origin.unwrap_or(Origin::Client)),
            omit_origin_header: Cell::new(false),
            same_origin_data: Cell::new(false),
            referer: Referer::Client,
            synchronous: false,
            mode: RequestMode::NoCORS,
            use_cors_preflight: false,
            credentials_mode: CredentialsMode::Omit,
            use_url_credentials: false,
            cache_mode: Cell::new(CacheMode::Default),
            redirect_mode: Cell::new(RedirectMode::Follow),
            integrity_metadata: RefCell::new(String::new()),
            url_list: RefCell::new(vec![url]),
            redirect_count: Cell::new(0),
            response_tainting: Cell::new(ResponseTainting::Basic),
            done: Cell::new(false)
        }
    }

    /// https://html.spec.whatwg.org/multipage/#create-a-potential-cors-request
    pub fn potential_cors_request(url: Url,
                                  cors_attribute_state: Option<CORSSettings>,
                                  is_service_worker_global_scope: bool,
                                  same_origin_fallback: bool) -> Request {
        Request {
            method: RefCell::new(Method::Get),
            local_urls_only: false,
            sandboxed_storage_area_urls: false,
            headers: RefCell::new(Headers::new()),
            unsafe_request: false,
            body: RefCell::new(None),
            is_service_worker_global_scope: is_service_worker_global_scope,
            window: Cell::new(Window::Client),
            keep_alive: Cell::new(false),
            skip_service_worker: Cell::new(false),
            initiator: Initiator::None,
            type_: Type::None,
            destination: Destination::None,
            origin: RefCell::new(Origin::Client),
            omit_origin_header: Cell::new(false),
            same_origin_data: Cell::new(false),
            referer: Referer::Client,
            synchronous: false,
            // Step 1-2
            mode: match cors_attribute_state {
                Some(_) => RequestMode::CORSMode,
                None if same_origin_fallback => RequestMode::SameOrigin,
                None => RequestMode::NoCORS
            },
            use_cors_preflight: false,
            // Step 3-4
            credentials_mode: match cors_attribute_state {
                Some(CORSSettings::Anonymous) => CredentialsMode::CredentialsSameOrigin,
                _ => CredentialsMode::Include,
            },
            use_url_credentials: true,
            cache_mode: Cell::new(CacheMode::Default),
            redirect_mode: Cell::new(RedirectMode::Follow),
            integrity_metadata: RefCell::new(String::new()),
            url_list: RefCell::new(vec![url]),
            redirect_count: Cell::new(0),
            response_tainting: Cell::new(ResponseTainting::Basic),
            done: Cell::new(false)
        }
    }

    pub fn url(&self) -> Url {
        self.url_list.borrow().first().unwrap().clone()
    }

    pub fn current_url(&self) -> Url {
        self.url_list.borrow().last().unwrap().clone()
    }

    pub fn current_url_string(&self) -> String {
        self.url_list.borrow().last().unwrap().serialize()
    }

    pub fn is_navigation_request(&self) -> bool {
        self.destination == Destination::Document
    }

    pub fn is_subresource_request(&self) -> bool {
        match self.destination {
            Destination::Font | Destination::Image | Destination::Manifest
                | Destination::Media | Destination::Script
                | Destination::Style | Destination::XSLT
                | Destination::None => true,
            _ => false
        }
    }
}
