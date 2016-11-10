/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use ReferrerPolicy;
use hyper::header::Headers;
use hyper::method::Method;
use msg::constellation_msg::PipelineId;
use std::cell::{Cell, RefCell};
use std::default::Default;
use std::mem::swap;
use url::{Origin as UrlOrigin, Url};

/// An [initiator](https://fetch.spec.whatwg.org/#concept-request-initiator)
#[derive(Copy, Clone, PartialEq, HeapSizeOf)]
pub enum Initiator {
    None,
    Download,
    ImageSet,
    Manifest,
    XSLT
}

/// A request [type](https://fetch.spec.whatwg.org/#concept-request-type)
#[derive(Copy, Clone, PartialEq, Serialize, Deserialize, HeapSizeOf)]
pub enum Type {
    None, Audio, Font, Image,
    Script, Style, Track, Video
}

/// A request [destination](https://fetch.spec.whatwg.org/#concept-request-destination)
#[derive(Copy, Clone, PartialEq, Serialize, Deserialize, HeapSizeOf)]
pub enum Destination {
    None, Document, Embed, Font, Image, Manifest,
    Media, Object, Report, Script, ServiceWorker,
    SharedWorker, Style, Worker, XSLT
}

/// A request [origin](https://fetch.spec.whatwg.org/#concept-request-origin)
#[derive(Clone, PartialEq, Debug, HeapSizeOf)]
pub enum Origin {
    Client,
    Origin(UrlOrigin)
}

/// A [referer](https://fetch.spec.whatwg.org/#concept-request-referrer)
#[derive(Clone, PartialEq, HeapSizeOf)]
pub enum Referrer {
    NoReferrer,
    /// Default referrer if nothing is specified
    Client,
    ReferrerUrl(Url)
}

/// A [request mode](https://fetch.spec.whatwg.org/#concept-request-mode)
#[derive(Copy, Clone, PartialEq, Serialize, Deserialize, HeapSizeOf)]
pub enum RequestMode {
    Navigate,
    SameOrigin,
    NoCors,
    CorsMode
}

/// Request [credentials mode](https://fetch.spec.whatwg.org/#concept-request-credentials-mode)
#[derive(Copy, Clone, PartialEq, Serialize, Deserialize, HeapSizeOf)]
pub enum CredentialsMode {
    Omit,
    CredentialsSameOrigin,
    Include
}

/// [Cache mode](https://fetch.spec.whatwg.org/#concept-request-cache-mode)
#[derive(Copy, Clone, PartialEq, HeapSizeOf)]
pub enum CacheMode {
    Default,
    NoStore,
    Reload,
    NoCache,
    ForceCache,
    OnlyIfCached
}

/// [Redirect mode](https://fetch.spec.whatwg.org/#concept-request-redirect-mode)
#[derive(Copy, Clone, PartialEq, Serialize, Deserialize, HeapSizeOf)]
pub enum RedirectMode {
    Follow,
    Error,
    Manual
}

/// [Response tainting](https://fetch.spec.whatwg.org/#concept-request-response-tainting)
#[derive(Copy, Clone, PartialEq, HeapSizeOf)]
pub enum ResponseTainting {
    Basic,
    CorsTainting,
    Opaque
}

/// [Window](https://fetch.spec.whatwg.org/#concept-request-window)
#[derive(Copy, Clone, PartialEq, HeapSizeOf)]
pub enum Window {
    NoWindow,
    Client,
    // TODO: Environmental settings object
}

/// [CORS settings attribute](https://html.spec.whatwg.org/multipage/#attr-crossorigin-anonymous)
#[derive(Copy, Clone, PartialEq, Serialize, Deserialize)]
pub enum CorsSettings {
    Anonymous,
    UseCredentials
}

#[derive(Serialize, Deserialize, Clone)]
pub struct RequestInit {
    #[serde(deserialize_with = "::hyper_serde::deserialize",
            serialize_with = "::hyper_serde::serialize")]
    pub method: Method,
    pub url: Url,
    #[serde(deserialize_with = "::hyper_serde::deserialize",
            serialize_with = "::hyper_serde::serialize")]
    pub headers: Headers,
    pub unsafe_request: bool,
    pub body: Option<Vec<u8>>,
    // TODO: client object
    pub type_: Type,
    pub destination: Destination,
    pub synchronous: bool,
    pub mode: RequestMode,
    pub use_cors_preflight: bool,
    pub credentials_mode: CredentialsMode,
    pub use_url_credentials: bool,
    // this should actually be set by fetch, but fetch
    // doesn't have info about the client right now
    pub origin: Url,
    // XXXManishearth these should be part of the client object
    pub referrer_url: Option<Url>,
    pub referrer_policy: Option<ReferrerPolicy>,
    pub pipeline_id: Option<PipelineId>,
    pub redirect_mode: RedirectMode,
}

impl Default for RequestInit {
    fn default() -> RequestInit {
        RequestInit {
            method: Method::Get,
            url: Url::parse("about:blank").unwrap(),
            headers: Headers::new(),
            unsafe_request: false,
            body: None,
            type_: Type::None,
            destination: Destination::None,
            synchronous: false,
            mode: RequestMode::NoCors,
            use_cors_preflight: false,
            credentials_mode: CredentialsMode::Omit,
            use_url_credentials: false,
            origin: Url::parse("about:blank").unwrap(),
            referrer_url: None,
            referrer_policy: None,
            pipeline_id: None,
            redirect_mode: RedirectMode::Follow,
        }
    }
}

/// A [Request](https://fetch.spec.whatwg.org/#requests) as defined by the Fetch spec
#[derive(Clone, HeapSizeOf)]
pub struct Request {
    #[ignore_heap_size_of = "Defined in hyper"]
    pub method: RefCell<Method>,
    pub local_urls_only: bool,
    pub sandboxed_storage_area_urls: bool,
    #[ignore_heap_size_of = "Defined in hyper"]
    pub headers: RefCell<Headers>,
    pub unsafe_request: bool,
    pub body: RefCell<Option<Vec<u8>>>,
    // TODO: client object
    pub is_service_worker_global_scope: bool,
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
    /// https://fetch.spec.whatwg.org/#concept-request-referrer
    pub referrer: RefCell<Referrer>,
    pub referrer_policy: Cell<Option<ReferrerPolicy>>,
    pub pipeline_id: Cell<Option<PipelineId>>,
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
    pub done: Cell<bool>,
}

impl Request {
    pub fn new(url: Url,
               origin: Option<Origin>,
               is_service_worker_global_scope: bool,
               pipeline_id: Option<PipelineId>) -> Request {
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
            referrer: RefCell::new(Referrer::Client),
            referrer_policy: Cell::new(None),
            pipeline_id: Cell::new(pipeline_id),
            synchronous: false,
            mode: RequestMode::NoCors,
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

    pub fn from_init(init: RequestInit) -> Request {
        let mut req = Request::new(init.url,
                                   Some(Origin::Origin(init.origin.origin())),
                                   false, init.pipeline_id);
        *req.method.borrow_mut() = init.method;
        *req.headers.borrow_mut() = init.headers;
        req.unsafe_request = init.unsafe_request;
        *req.body.borrow_mut() = init.body;
        req.type_ = init.type_;
        req.destination = init.destination;
        req.synchronous = init.synchronous;
        req.mode = init.mode;
        req.use_cors_preflight = init.use_cors_preflight;
        req.credentials_mode = init.credentials_mode;
        req.use_url_credentials = init.use_url_credentials;
        *req.referrer.borrow_mut() = if let Some(url) = init.referrer_url {
            Referrer::ReferrerUrl(url)
        } else {
            Referrer::NoReferrer
        };
        req.referrer_policy.set(init.referrer_policy);
        req.pipeline_id.set(init.pipeline_id);
        req.redirect_mode.set(init.redirect_mode);
        req
    }

    pub fn url(&self) -> Url {
        self.url_list.borrow().first().unwrap().clone()
    }

    pub fn current_url(&self) -> Url {
        self.url_list.borrow().last().unwrap().clone()
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

impl Referrer {
    pub fn to_url(&self) -> Option<&Url> {
        match *self {
            Referrer::NoReferrer | Referrer::Client => None,
            Referrer::ReferrerUrl(ref url) => Some(url)
        }
    }
    pub fn from_url(url: Option<Url>) -> Self {
        if let Some(url) = url {
            Referrer::ReferrerUrl(url)
        } else {
            Referrer::NoReferrer
        }
    }
    pub fn take(&mut self) -> Option<Url> {
        let mut new = Referrer::Client;
        swap(self, &mut new);
        match new {
            Referrer::NoReferrer | Referrer::Client => None,
            Referrer::ReferrerUrl(url) => Some(url)
        }
    }
}
