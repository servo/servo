/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use hyper::header::Headers;
use hyper::method::Method;
use std::cell::{Cell, RefCell};
use url::{Origin, Url};

/// A [request context](https://fetch.spec.whatwg.org/#concept-request-context)
#[derive(Copy, Clone, PartialEq)]
pub enum Context {
    Audio, Beacon, CSPreport, Download, Embed, Eventsource,
    Favicon, Fetch, Font, Form, Frame, Hyperlink, IFrame, Image,
    ImageSet, Import, Internal, Location, Manifest, MetaRefresh, Object,
    Ping, Plugin, Prefetch, PreRender, Script, ServiceWorker, SharedWorker,
    Subresource, Style, Track, Video, Worker, XMLHttpRequest, XSLT
}

/// A [request context frame type](https://fetch.spec.whatwg.org/#concept-request-context-frame-type)
#[derive(Copy, Clone, PartialEq)]
pub enum ContextFrameType {
    Auxiliary,
    TopLevel,
    Nested,
    ContextNone
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

/// [CORS settings attribute](https://html.spec.whatwg.org/multipage/#attr-crossorigin-anonymous)
pub enum CORSSettings {
    Anonymous,
    UseCredentials
}

/// A [Request](https://fetch.spec.whatwg.org/#requests) as defined by the Fetch spec
#[derive(Clone)]
pub struct Request {
    pub method: RefCell<Method>,
    pub local_urls_only: bool,
    // Use the last method on url_list to act as spec url field
    pub url_list: RefCell<Vec<Url>>,
    pub headers: RefCell<Headers>,
    pub unsafe_request: bool,
    pub body: Option<Vec<u8>>,
    pub preserve_content_codings: bool,
    // pub client: GlobalRef, // XXXManishearth copy over only the relevant fields of the global scope,
                              // not the entire scope to avoid the libscript dependency
    pub is_service_worker_global_scope: bool,
    pub skip_service_worker: Cell<bool>,
    pub context: Context,
    pub context_frame_type: ContextFrameType,
    pub origin: Origin,
    pub force_origin_header: bool,
    pub omit_origin_header: bool,
    pub same_origin_data: Cell<bool>,
    pub referer: Referer,
    pub authentication: bool,
    pub synchronous: bool,
    pub use_cors_preflight: bool,
    pub mode: RequestMode,
    pub credentials_mode: CredentialsMode,
    pub use_url_credentials: bool,
    pub cache_mode: Cell<CacheMode>,
    pub redirect_mode: Cell<RedirectMode>,
    pub redirect_count: Cell<u32>,
    pub response_tainting: Cell<ResponseTainting>
}

impl Request {
    pub fn new(url: Url,
               context: Context,
               origin: Origin,
               is_service_worker_global_scope: bool) -> Request {
         Request {
            method: RefCell::new(Method::Get),
            local_urls_only: false,
            url_list: RefCell::new(vec![url]),
            headers: RefCell::new(Headers::new()),
            unsafe_request: false,
            body: None,
            preserve_content_codings: false,
            is_service_worker_global_scope: is_service_worker_global_scope,
            skip_service_worker: Cell::new(false),
            context: context,
            context_frame_type: ContextFrameType::ContextNone,
            origin: origin,
            force_origin_header: false,
            omit_origin_header: false,
            same_origin_data: Cell::new(false),
            referer: Referer::Client,
            authentication: false,
            synchronous: false,
            use_cors_preflight: false,
            mode: RequestMode::NoCORS,
            credentials_mode: CredentialsMode::Omit,
            use_url_credentials: false,
            cache_mode: Cell::new(CacheMode::Default),
            redirect_mode: Cell::new(RedirectMode::Follow),
            redirect_count: Cell::new(0),
            response_tainting: Cell::new(ResponseTainting::Basic)
        }
    }

    /// https://html.spec.whatwg.org/multipage/#create-a-potential-cors-request
    pub fn potential_cors_request(url: Url,
                               cors_attribute_state: Option<CORSSettings>,
                               context: Context,
                               is_service_worker_global_scope: bool,
                               same_origin_fallback: bool) -> Request {
        let origin = url.origin();
        Request {
            method: RefCell::new(Method::Get),
            local_urls_only: false,
            url_list: RefCell::new(vec![url]),
            headers: RefCell::new(Headers::new()),
            unsafe_request: false,
            body: None,
            preserve_content_codings: false,
            is_service_worker_global_scope: is_service_worker_global_scope,
            skip_service_worker: Cell::new(false),
            context: context,
            context_frame_type: ContextFrameType::ContextNone,
            origin: origin,
            force_origin_header: false,
            same_origin_data: Cell::new(false),
            omit_origin_header: false,
            referer: Referer::Client,
            authentication: false,
            synchronous: false,
            use_cors_preflight: false,
            // Step 1-2
            mode: match cors_attribute_state {
                Some(_) => RequestMode::CORSMode,
                None if same_origin_fallback => RequestMode::SameOrigin,
                None => RequestMode::NoCORS
            },
            // Step 3-4
            credentials_mode: match cors_attribute_state {
                Some(CORSSettings::Anonymous) => CredentialsMode::CredentialsSameOrigin,
                _ => CredentialsMode::Include,
            },
            use_url_credentials: true,
            cache_mode: Cell::new(CacheMode::Default),
            redirect_mode: Cell::new(RedirectMode::Follow),
            redirect_count: Cell::new(0),
            response_tainting: Cell::new(ResponseTainting::Basic)
        }
    }

    pub fn get_last_url_string(&self) -> String {
        self.url_list.borrow().last().unwrap().serialize()
    }

    pub fn current_url(&self) -> Url {
        self.url_list.borrow().last().unwrap().clone()
    }
}
