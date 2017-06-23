/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use ReferrerPolicy;
use hyper::header::Headers;
use hyper::method::Method;
use msg::constellation_msg::PipelineId;
use servo_url::{ImmutableOrigin, ServoUrl};
use std::default::Default;

/// An [initiator](https://fetch.spec.whatwg.org/#concept-request-initiator)
#[derive(Copy, Clone, PartialEq, HeapSizeOf)]
pub enum Initiator {
    None,
    Download,
    ImageSet,
    Manifest,
    XSLT,
}

/// A request [type](https://fetch.spec.whatwg.org/#concept-request-type)
#[derive(Copy, Clone, PartialEq, Serialize, Deserialize, HeapSizeOf)]
pub enum Type {
    None,
    Audio,
    Font,
    Image,
    Script,
    Style,
    Track,
    Video,
}

/// A request [destination](https://fetch.spec.whatwg.org/#concept-request-destination)
#[derive(Copy, Clone, PartialEq, Serialize, Deserialize, HeapSizeOf)]
pub enum Destination {
    None,
    Document,
    Embed,
    Font,
    Image,
    Manifest,
    Media,
    Object,
    Report,
    Script,
    ServiceWorker,
    SharedWorker,
    Style,
    Worker,
    XSLT,
}

/// A request [origin](https://fetch.spec.whatwg.org/#concept-request-origin)
#[derive(Clone, PartialEq, Debug, Serialize, Deserialize, HeapSizeOf)]
pub enum Origin {
    Client,
    Origin(ImmutableOrigin),
}

/// A [referer](https://fetch.spec.whatwg.org/#concept-request-referrer)
#[derive(Clone, PartialEq, Serialize, Deserialize, HeapSizeOf)]
pub enum Referrer {
    NoReferrer,
    /// Default referrer if nothing is specified
    Client,
    ReferrerUrl(ServoUrl),
}

/// A [request mode](https://fetch.spec.whatwg.org/#concept-request-mode)
#[derive(Copy, Clone, PartialEq, Serialize, Deserialize, HeapSizeOf)]
pub enum RequestMode {
    Navigate,
    SameOrigin,
    NoCors,
    CorsMode,
    WebSocket
}

/// Request [credentials mode](https://fetch.spec.whatwg.org/#concept-request-credentials-mode)
#[derive(Copy, Clone, PartialEq, Serialize, Deserialize, HeapSizeOf)]
pub enum CredentialsMode {
    Omit,
    CredentialsSameOrigin,
    Include,
}

/// [Cache mode](https://fetch.spec.whatwg.org/#concept-request-cache-mode)
#[derive(Copy, Clone, PartialEq, Serialize, Deserialize, HeapSizeOf)]
pub enum CacheMode {
    Default,
    NoStore,
    Reload,
    NoCache,
    ForceCache,
    OnlyIfCached,
}

/// [Service-workers mode](https://fetch.spec.whatwg.org/#request-service-workers-mode)
#[derive(Copy, Clone, PartialEq, Serialize, Deserialize, HeapSizeOf)]
pub enum ServiceWorkersMode {
    All,
    Foreign,
    None,
}

/// [Redirect mode](https://fetch.spec.whatwg.org/#concept-request-redirect-mode)
#[derive(Copy, Clone, PartialEq, Serialize, Deserialize, HeapSizeOf)]
pub enum RedirectMode {
    Follow,
    Error,
    Manual,
}

/// [Response tainting](https://fetch.spec.whatwg.org/#concept-request-response-tainting)
#[derive(Copy, Clone, PartialEq, HeapSizeOf)]
pub enum ResponseTainting {
    Basic,
    CorsTainting,
    Opaque,
}

/// [Window](https://fetch.spec.whatwg.org/#concept-request-window)
#[derive(Copy, Clone, PartialEq, HeapSizeOf)]
pub enum Window {
    NoWindow,
    Client, // TODO: Environmental settings object
}

/// [CORS settings attribute](https://html.spec.whatwg.org/multipage/#attr-crossorigin-anonymous)
#[derive(Copy, Clone, PartialEq, Serialize, Deserialize)]
pub enum CorsSettings {
    Anonymous,
    UseCredentials,
}

#[derive(Serialize, Deserialize, Clone, HeapSizeOf)]
pub struct RequestInit {
    #[serde(deserialize_with = "::hyper_serde::deserialize",
            serialize_with = "::hyper_serde::serialize")]
    #[ignore_heap_size_of = "Defined in hyper"]
    pub method: Method,
    pub url: ServoUrl,
    #[serde(deserialize_with = "::hyper_serde::deserialize",
            serialize_with = "::hyper_serde::serialize")]
    #[ignore_heap_size_of = "Defined in hyper"]
    pub headers: Headers,
    pub unsafe_request: bool,
    pub body: Option<Vec<u8>>,
    pub service_workers_mode: ServiceWorkersMode,
    // TODO: client object
    pub type_: Type,
    pub destination: Destination,
    pub synchronous: bool,
    pub mode: RequestMode,
    pub cache_mode: CacheMode,
    pub use_cors_preflight: bool,
    pub credentials_mode: CredentialsMode,
    pub use_url_credentials: bool,
    // this should actually be set by fetch, but fetch
    // doesn't have info about the client right now
    pub origin: ServoUrl,
    // XXXManishearth these should be part of the client object
    pub referrer_url: Option<ServoUrl>,
    pub referrer_policy: Option<ReferrerPolicy>,
    pub pipeline_id: Option<PipelineId>,
    pub redirect_mode: RedirectMode,
    pub integrity_metadata: String,
    // to keep track of redirects
    pub url_list: Vec<ServoUrl>,
}

impl Default for RequestInit {
    fn default() -> RequestInit {
        RequestInit {
            method: Method::Get,
            url: ServoUrl::parse("about:blank").unwrap(),
            headers: Headers::new(),
            unsafe_request: false,
            body: None,
            service_workers_mode: ServiceWorkersMode::All,
            type_: Type::None,
            destination: Destination::None,
            synchronous: false,
            mode: RequestMode::NoCors,
            cache_mode: CacheMode::Default,
            use_cors_preflight: false,
            credentials_mode: CredentialsMode::Omit,
            use_url_credentials: false,
            origin: ServoUrl::parse("about:blank").unwrap(),
            referrer_url: None,
            referrer_policy: None,
            pipeline_id: None,
            redirect_mode: RedirectMode::Follow,
            integrity_metadata: "".to_owned(),
            url_list: vec![],
        }
    }
}

/// A [Request](https://fetch.spec.whatwg.org/#concept-request) as defined by
/// the Fetch spec.
#[derive(Clone, HeapSizeOf)]
pub struct Request {
    /// https://fetch.spec.whatwg.org/#concept-request-method
    #[ignore_heap_size_of = "Defined in hyper"]
    pub method: Method,
    /// https://fetch.spec.whatwg.org/#local-urls-only-flag
    pub local_urls_only: bool,
    /// https://fetch.spec.whatwg.org/#sandboxed-storage-area-urls-flag
    pub sandboxed_storage_area_urls: bool,
    /// https://fetch.spec.whatwg.org/#concept-request-header-list
    #[ignore_heap_size_of = "Defined in hyper"]
    pub headers: Headers,
    /// https://fetch.spec.whatwg.org/#unsafe-request-flag
    pub unsafe_request: bool,
    /// https://fetch.spec.whatwg.org/#concept-request-body
    pub body: Option<Vec<u8>>,
    // TODO: client object
    pub window: Window,
    // TODO: target browsing context
    /// https://fetch.spec.whatwg.org/#request-keepalive-flag
    pub keep_alive: bool,
    // https://fetch.spec.whatwg.org/#request-service-workers-mode
    pub service_workers_mode: ServiceWorkersMode,
    /// https://fetch.spec.whatwg.org/#concept-request-initiator
    pub initiator: Initiator,
    /// https://fetch.spec.whatwg.org/#concept-request-type
    pub type_: Type,
    /// https://fetch.spec.whatwg.org/#concept-request-destination
    pub destination: Destination,
    // TODO: priority object
    /// https://fetch.spec.whatwg.org/#concept-request-origin
    pub origin: Origin,
    /// https://fetch.spec.whatwg.org/#concept-request-referrer
    pub referrer: Referrer,
    /// https://fetch.spec.whatwg.org/#concept-request-referrer-policy
    pub referrer_policy: Option<ReferrerPolicy>,
    pub pipeline_id: Option<PipelineId>,
    /// https://fetch.spec.whatwg.org/#synchronous-flag
    pub synchronous: bool,
    /// https://fetch.spec.whatwg.org/#concept-request-mode
    pub mode: RequestMode,
    /// https://fetch.spec.whatwg.org/#use-cors-preflight-flag
    pub use_cors_preflight: bool,
    /// https://fetch.spec.whatwg.org/#concept-request-credentials-mode
    pub credentials_mode: CredentialsMode,
    /// https://fetch.spec.whatwg.org/#concept-request-use-url-credentials-flag
    pub use_url_credentials: bool,
    /// https://fetch.spec.whatwg.org/#concept-request-cache-mode
    pub cache_mode: CacheMode,
    /// https://fetch.spec.whatwg.org/#concept-request-redirect-mode
    pub redirect_mode: RedirectMode,
    /// https://fetch.spec.whatwg.org/#concept-request-integrity-metadata
    pub integrity_metadata: String,
    // Use the last method on url_list to act as spec current url field, and
    // first method to act as spec url field
    /// https://fetch.spec.whatwg.org/#concept-request-url-list
    pub url_list: Vec<ServoUrl>,
    /// https://fetch.spec.whatwg.org/#concept-request-redirect-count
    pub redirect_count: u32,
    /// https://fetch.spec.whatwg.org/#concept-request-response-tainting
    pub response_tainting: ResponseTainting,
}

impl Request {
    pub fn new(url: ServoUrl,
               origin: Option<Origin>,
               pipeline_id: Option<PipelineId>)
               -> Request {
        Request {
            method: Method::Get,
            local_urls_only: false,
            sandboxed_storage_area_urls: false,
            headers: Headers::new(),
            unsafe_request: false,
            body: None,
            window: Window::Client,
            keep_alive: false,
            service_workers_mode: ServiceWorkersMode::All,
            initiator: Initiator::None,
            type_: Type::None,
            destination: Destination::None,
            origin: origin.unwrap_or(Origin::Client),
            referrer: Referrer::Client,
            referrer_policy: None,
            pipeline_id: pipeline_id,
            synchronous: false,
            mode: RequestMode::NoCors,
            use_cors_preflight: false,
            credentials_mode: CredentialsMode::Omit,
            use_url_credentials: false,
            cache_mode: CacheMode::Default,
            redirect_mode: RedirectMode::Follow,
            integrity_metadata: String::new(),
            url_list: vec![url],
            redirect_count: 0,
            response_tainting: ResponseTainting::Basic,
        }
    }

    pub fn from_init(init: RequestInit) -> Request {
        let mut req = Request::new(init.url.clone(),
                                   Some(Origin::Origin(init.origin.origin())),
                                   init.pipeline_id);
        req.method = init.method;
        req.headers = init.headers;
        req.unsafe_request = init.unsafe_request;
        req.body = init.body;
        req.service_workers_mode = init.service_workers_mode;
        req.type_ = init.type_;
        req.destination = init.destination;
        req.synchronous = init.synchronous;
        req.mode = init.mode;
        req.use_cors_preflight = init.use_cors_preflight;
        req.credentials_mode = init.credentials_mode;
        req.use_url_credentials = init.use_url_credentials;
        req.cache_mode = init.cache_mode;
        req.referrer = if let Some(url) = init.referrer_url {
            Referrer::ReferrerUrl(url)
        } else {
            Referrer::NoReferrer
        };
        req.referrer_policy = init.referrer_policy;
        req.pipeline_id = init.pipeline_id;
        req.redirect_mode = init.redirect_mode;
        let mut url_list = init.url_list;
        if url_list.is_empty() {
            url_list.push(init.url);
        }
        req.redirect_count = url_list.len() as u32 - 1;
        req.url_list = url_list;
        req.integrity_metadata = init.integrity_metadata;
        req
    }

    /// https://fetch.spec.whatwg.org/#concept-request-url
    pub fn url(&self) -> ServoUrl {
        self.url_list.first().unwrap().clone()
    }

    /// https://fetch.spec.whatwg.org/#concept-request-current-url
    pub fn current_url(&self) -> ServoUrl {
        self.url_list.last().unwrap().clone()
    }

    /// https://fetch.spec.whatwg.org/#concept-request-current-url
    pub fn current_url_mut(&mut self) -> &mut ServoUrl {
        self.url_list.last_mut().unwrap()
    }

    /// https://fetch.spec.whatwg.org/#navigation-request
    pub fn is_navigation_request(&self) -> bool {
        self.destination == Destination::Document
    }

    /// https://fetch.spec.whatwg.org/#subresource-request
    pub fn is_subresource_request(&self) -> bool {
        match self.destination {
            Destination::Font | Destination::Image | Destination::Manifest | Destination::Media |
            Destination::Script | Destination::Style | Destination::XSLT | Destination::None => true,
            _ => false,
        }
    }
}

impl Referrer {
    pub fn to_url(&self) -> Option<&ServoUrl> {
        match *self {
            Referrer::NoReferrer | Referrer::Client => None,
            Referrer::ReferrerUrl(ref url) => Some(url),
        }
    }
}
