/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use crate::ReferrerPolicy;
use crate::ResourceTimingType;
use http::HeaderMap;
use hyper::Method;
use msg::constellation_msg::PipelineId;
use servo_url::{ImmutableOrigin, ServoUrl};

/// An [initiator](https://fetch.spec.whatwg.org/#concept-request-initiator)
#[derive(Clone, Copy, MallocSizeOf, PartialEq)]
pub enum Initiator {
    None,
    Download,
    ImageSet,
    Manifest,
    XSLT,
}

/// A request [destination](https://fetch.spec.whatwg.org/#concept-request-destination)
#[derive(Clone, Copy, Debug, Deserialize, MallocSizeOf, PartialEq, Serialize)]
pub enum Destination {
    None,
    Audio,
    Document,
    Embed,
    Font,
    Image,
    Manifest,
    Object,
    Report,
    Script,
    ServiceWorker,
    SharedWorker,
    Style,
    Track,
    Video,
    Worker,
    Xslt,
}

impl Destination {
    /// https://fetch.spec.whatwg.org/#request-destination-script-like
    #[inline]
    pub fn is_script_like(&self) -> bool {
        *self == Destination::Script ||
            *self == Destination::ServiceWorker ||
            *self == Destination::SharedWorker ||
            *self == Destination::Worker
    }
}

/// A request [origin](https://fetch.spec.whatwg.org/#concept-request-origin)
#[derive(Clone, Debug, Deserialize, MallocSizeOf, PartialEq, Serialize)]
pub enum Origin {
    Client,
    Origin(ImmutableOrigin),
}

/// A [referer](https://fetch.spec.whatwg.org/#concept-request-referrer)
#[derive(Clone, Debug, Deserialize, MallocSizeOf, PartialEq, Serialize)]
pub enum Referrer {
    NoReferrer,
    /// Default referrer if nothing is specified
    Client,
    ReferrerUrl(ServoUrl),
}

/// A [request mode](https://fetch.spec.whatwg.org/#concept-request-mode)
#[derive(Clone, Debug, Deserialize, MallocSizeOf, PartialEq, Serialize)]
pub enum RequestMode {
    Navigate,
    SameOrigin,
    NoCors,
    CorsMode,
    WebSocket { protocols: Vec<String> },
}

/// Request [credentials mode](https://fetch.spec.whatwg.org/#concept-request-credentials-mode)
#[derive(Clone, Copy, Debug, Deserialize, MallocSizeOf, PartialEq, Serialize)]
pub enum CredentialsMode {
    Omit,
    CredentialsSameOrigin,
    Include,
}

/// [Cache mode](https://fetch.spec.whatwg.org/#concept-request-cache-mode)
#[derive(Clone, Copy, Debug, Deserialize, MallocSizeOf, PartialEq, Serialize)]
pub enum CacheMode {
    Default,
    NoStore,
    Reload,
    NoCache,
    ForceCache,
    OnlyIfCached,
}

/// [Service-workers mode](https://fetch.spec.whatwg.org/#request-service-workers-mode)
#[derive(Clone, Copy, Debug, Deserialize, MallocSizeOf, PartialEq, Serialize)]
pub enum ServiceWorkersMode {
    All,
    None,
}

/// [Redirect mode](https://fetch.spec.whatwg.org/#concept-request-redirect-mode)
#[derive(Clone, Copy, Debug, Deserialize, MallocSizeOf, PartialEq, Serialize)]
pub enum RedirectMode {
    Follow,
    Error,
    Manual,
}

/// [Response tainting](https://fetch.spec.whatwg.org/#concept-request-response-tainting)
#[derive(Clone, Copy, MallocSizeOf, PartialEq)]
pub enum ResponseTainting {
    Basic,
    CorsTainting,
    Opaque,
}

/// [Window](https://fetch.spec.whatwg.org/#concept-request-window)
#[derive(Clone, Copy, MallocSizeOf, PartialEq)]
pub enum Window {
    NoWindow,
    Client, // TODO: Environmental settings object
}

/// [CORS settings attribute](https://html.spec.whatwg.org/multipage/#attr-crossorigin-anonymous)
#[derive(Clone, Copy, Debug, Deserialize, PartialEq, Serialize)]
pub enum CorsSettings {
    Anonymous,
    UseCredentials,
}

#[derive(Clone, Debug, Deserialize, MallocSizeOf, Serialize)]
pub struct RequestBuilder {
    #[serde(
        deserialize_with = "::hyper_serde::deserialize",
        serialize_with = "::hyper_serde::serialize"
    )]
    #[ignore_malloc_size_of = "Defined in hyper"]
    pub method: Method,
    pub url: ServoUrl,
    #[serde(
        deserialize_with = "::hyper_serde::deserialize",
        serialize_with = "::hyper_serde::serialize"
    )]
    #[ignore_malloc_size_of = "Defined in hyper"]
    pub headers: HeaderMap,
    pub unsafe_request: bool,
    pub body: Option<Vec<u8>>,
    pub service_workers_mode: ServiceWorkersMode,
    // TODO: client object
    pub destination: Destination,
    pub synchronous: bool,
    pub mode: RequestMode,
    pub cache_mode: CacheMode,
    pub use_cors_preflight: bool,
    pub credentials_mode: CredentialsMode,
    pub use_url_credentials: bool,
    pub origin: ImmutableOrigin,
    // XXXManishearth these should be part of the client object
    pub referrer_url: Option<ServoUrl>,
    pub referrer_policy: Option<ReferrerPolicy>,
    pub pipeline_id: Option<PipelineId>,
    pub redirect_mode: RedirectMode,
    pub integrity_metadata: String,
    // to keep track of redirects
    pub url_list: Vec<ServoUrl>,
}

impl RequestBuilder {
    pub fn new(url: ServoUrl) -> RequestBuilder {
        RequestBuilder {
            method: Method::GET,
            url: url,
            headers: HeaderMap::new(),
            unsafe_request: false,
            body: None,
            service_workers_mode: ServiceWorkersMode::All,
            destination: Destination::None,
            synchronous: false,
            mode: RequestMode::NoCors,
            cache_mode: CacheMode::Default,
            use_cors_preflight: false,
            credentials_mode: CredentialsMode::Omit,
            use_url_credentials: false,
            origin: ImmutableOrigin::new_opaque(),
            referrer_url: None,
            referrer_policy: None,
            pipeline_id: None,
            redirect_mode: RedirectMode::Follow,
            integrity_metadata: "".to_owned(),
            url_list: vec![],
        }
    }

    pub fn method(mut self, method: Method) -> RequestBuilder {
        self.method = method;
        self
    }

    pub fn headers(mut self, headers: HeaderMap) -> RequestBuilder {
        self.headers = headers;
        self
    }

    pub fn unsafe_request(mut self, unsafe_request: bool) -> RequestBuilder {
        self.unsafe_request = unsafe_request;
        self
    }

    pub fn body(mut self, body: Option<Vec<u8>>) -> RequestBuilder {
        self.body = body;
        self
    }

    pub fn service_workers_mode(
        mut self,
        service_workers_mode: ServiceWorkersMode,
    ) -> RequestBuilder {
        self.service_workers_mode = service_workers_mode;
        self
    }

    pub fn destination(mut self, destination: Destination) -> RequestBuilder {
        self.destination = destination;
        self
    }

    pub fn synchronous(mut self, synchronous: bool) -> RequestBuilder {
        self.synchronous = synchronous;
        self
    }

    pub fn mode(mut self, mode: RequestMode) -> RequestBuilder {
        self.mode = mode;
        self
    }

    pub fn cache_mode(mut self, cache_mode: CacheMode) -> RequestBuilder {
        self.cache_mode = cache_mode;
        self
    }

    pub fn use_cors_preflight(mut self, use_cors_preflight: bool) -> RequestBuilder {
        self.use_cors_preflight = use_cors_preflight;
        self
    }

    pub fn credentials_mode(mut self, credentials_mode: CredentialsMode) -> RequestBuilder {
        self.credentials_mode = credentials_mode;
        self
    }

    pub fn use_url_credentials(mut self, use_url_credentials: bool) -> RequestBuilder {
        self.use_url_credentials = use_url_credentials;
        self
    }

    pub fn origin(mut self, origin: ImmutableOrigin) -> RequestBuilder {
        self.origin = origin;
        self
    }

    pub fn referrer_url(mut self, referrer_url: Option<ServoUrl>) -> RequestBuilder {
        self.referrer_url = referrer_url;
        self
    }

    pub fn referrer_policy(mut self, referrer_policy: Option<ReferrerPolicy>) -> RequestBuilder {
        self.referrer_policy = referrer_policy;
        self
    }

    pub fn pipeline_id(mut self, pipeline_id: Option<PipelineId>) -> RequestBuilder {
        self.pipeline_id = pipeline_id;
        self
    }

    pub fn redirect_mode(mut self, redirect_mode: RedirectMode) -> RequestBuilder {
        self.redirect_mode = redirect_mode;
        self
    }

    pub fn integrity_metadata(mut self, integrity_metadata: String) -> RequestBuilder {
        self.integrity_metadata = integrity_metadata;
        self
    }

    pub fn url_list(mut self, url_list: Vec<ServoUrl>) -> RequestBuilder {
        self.url_list = url_list;
        self
    }

    pub fn build(self) -> Request {
        let mut request = Request::new(
            self.url.clone(),
            Some(Origin::Origin(self.origin)),
            self.pipeline_id,
        );
        request.method = self.method;
        request.headers = self.headers;
        request.unsafe_request = self.unsafe_request;
        request.body = self.body;
        request.service_workers_mode = self.service_workers_mode;
        request.destination = self.destination;
        request.synchronous = self.synchronous;
        request.mode = self.mode;
        request.use_cors_preflight = self.use_cors_preflight;
        request.credentials_mode = self.credentials_mode;
        request.use_url_credentials = self.use_url_credentials;
        request.cache_mode = self.cache_mode;
        request.referrer = if let Some(url) = self.referrer_url {
            Referrer::ReferrerUrl(url)
        } else {
            Referrer::NoReferrer
        };
        request.referrer_policy = self.referrer_policy;
        request.redirect_mode = self.redirect_mode;
        let mut url_list = self.url_list;
        if url_list.is_empty() {
            url_list.push(self.url);
        }
        request.redirect_count = url_list.len() as u32 - 1;
        request.url_list = url_list;
        request.integrity_metadata = self.integrity_metadata;
        request
    }
}

/// A [Request](https://fetch.spec.whatwg.org/#concept-request) as defined by
/// the Fetch spec.
#[derive(Clone, MallocSizeOf)]
pub struct Request {
    /// <https://fetch.spec.whatwg.org/#concept-request-method>
    #[ignore_malloc_size_of = "Defined in hyper"]
    pub method: Method,
    /// <https://fetch.spec.whatwg.org/#local-urls-only-flag>
    pub local_urls_only: bool,
    /// <https://fetch.spec.whatwg.org/#sandboxed-storage-area-urls-flag>
    pub sandboxed_storage_area_urls: bool,
    /// <https://fetch.spec.whatwg.org/#concept-request-header-list>
    #[ignore_malloc_size_of = "Defined in hyper"]
    pub headers: HeaderMap,
    /// <https://fetch.spec.whatwg.org/#unsafe-request-flag>
    pub unsafe_request: bool,
    /// <https://fetch.spec.whatwg.org/#concept-request-body>
    pub body: Option<Vec<u8>>,
    // TODO: client object
    pub window: Window,
    // TODO: target browsing context
    /// <https://fetch.spec.whatwg.org/#request-keepalive-flag>
    pub keep_alive: bool,
    /// <https://fetch.spec.whatwg.org/#request-service-workers-mode>
    pub service_workers_mode: ServiceWorkersMode,
    /// <https://fetch.spec.whatwg.org/#concept-request-initiator>
    pub initiator: Initiator,
    /// <https://fetch.spec.whatwg.org/#concept-request-destination>
    pub destination: Destination,
    // TODO: priority object
    /// <https://fetch.spec.whatwg.org/#concept-request-origin>
    pub origin: Origin,
    /// <https://fetch.spec.whatwg.org/#concept-request-referrer>
    pub referrer: Referrer,
    /// <https://fetch.spec.whatwg.org/#concept-request-referrer-policy>
    pub referrer_policy: Option<ReferrerPolicy>,
    pub pipeline_id: Option<PipelineId>,
    /// <https://fetch.spec.whatwg.org/#synchronous-flag>
    pub synchronous: bool,
    /// <https://fetch.spec.whatwg.org/#concept-request-mode>
    pub mode: RequestMode,
    /// <https://fetch.spec.whatwg.org/#use-cors-preflight-flag>
    pub use_cors_preflight: bool,
    /// <https://fetch.spec.whatwg.org/#concept-request-credentials-mode>
    pub credentials_mode: CredentialsMode,
    /// <https://fetch.spec.whatwg.org/#concept-request-use-url-credentials-flag>
    pub use_url_credentials: bool,
    /// <https://fetch.spec.whatwg.org/#concept-request-cache-mode>
    pub cache_mode: CacheMode,
    /// <https://fetch.spec.whatwg.org/#concept-request-redirect-mode>
    pub redirect_mode: RedirectMode,
    /// <https://fetch.spec.whatwg.org/#concept-request-integrity-metadata>
    pub integrity_metadata: String,
    // Use the last method on url_list to act as spec current url field, and
    // first method to act as spec url field
    /// <https://fetch.spec.whatwg.org/#concept-request-url-list>
    pub url_list: Vec<ServoUrl>,
    /// <https://fetch.spec.whatwg.org/#concept-request-redirect-count>
    pub redirect_count: u32,
    /// <https://fetch.spec.whatwg.org/#concept-request-response-tainting>
    pub response_tainting: ResponseTainting,
}

impl Request {
    pub fn new(url: ServoUrl, origin: Option<Origin>, pipeline_id: Option<PipelineId>) -> Request {
        Request {
            method: Method::GET,
            local_urls_only: false,
            sandboxed_storage_area_urls: false,
            headers: HeaderMap::new(),
            unsafe_request: false,
            body: None,
            window: Window::Client,
            keep_alive: false,
            service_workers_mode: ServiceWorkersMode::All,
            initiator: Initiator::None,
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

    /// <https://fetch.spec.whatwg.org/#concept-request-url>
    pub fn url(&self) -> ServoUrl {
        self.url_list.first().unwrap().clone()
    }

    /// <https://fetch.spec.whatwg.org/#concept-request-current-url>
    pub fn current_url(&self) -> ServoUrl {
        self.url_list.last().unwrap().clone()
    }

    /// <https://fetch.spec.whatwg.org/#concept-request-current-url>
    pub fn current_url_mut(&mut self) -> &mut ServoUrl {
        self.url_list.last_mut().unwrap()
    }

    /// <https://fetch.spec.whatwg.org/#navigation-request>
    pub fn is_navigation_request(&self) -> bool {
        self.destination == Destination::Document
    }

    /// <https://fetch.spec.whatwg.org/#subresource-request>
    pub fn is_subresource_request(&self) -> bool {
        match self.destination {
            Destination::Audio |
            Destination::Font |
            Destination::Image |
            Destination::Manifest |
            Destination::Script |
            Destination::Style |
            Destination::Track |
            Destination::Video |
            Destination::Xslt |
            Destination::None => true,
            _ => false,
        }
    }

    pub fn timing_type(&self) -> ResourceTimingType {
        if self.is_navigation_request() {
            ResourceTimingType::Navigation
        } else {
            ResourceTimingType::Resource
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
