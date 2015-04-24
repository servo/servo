/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use url::Url;
use hyper::method::Method;
use hyper::mime::{Mime, TopLevel, SubLevel, Attr, Value};
use hyper::header::{Headers, ContentType, IfModifiedSince, IfNoneMatch};
use hyper::header::{IfUnmodifiedSince, IfMatch, Location, HeaderView};
use hyper::status::StatusCode;
use fetch::cors_cache::{CORSCache, CacheRequestDetails};
use fetch::response::{Response, ResponseType};
use std::ascii::AsciiExt;

/// A [request context](http://fetch.spec.whatwg.org/#concept-request-context)
#[derive(Copy, PartialEq)]
pub enum Context {
    Audio, Beacon, CSPreport, Download, Embed, Eventsource,
    Favicon, Fetch, Font, Form, Frame, Hyperlink, IFrame, Image,
    ImageSet, Import, Internal, Location, Manifest, Object, Ping,
    Plugin, Prefetch, Script, ServiceWorker, SharedWorker, Subresource,
    Style, Track, Video, Worker, XMLHttpRequest, XSLT
}

/// A [request context frame type](http://fetch.spec.whatwg.org/#concept-request-context-frame-type)
#[derive(Copy, PartialEq)]
pub enum ContextFrameType {
    Auxiliary,
    TopLevel,
    Nested,
    ContextNone
}

/// A [referer](http://fetch.spec.whatwg.org/#concept-request-referrer)
pub enum Referer {
    RefererNone,
    Client,
    RefererUrl(Url)
}

/// A [request mode](http://fetch.spec.whatwg.org/#concept-request-mode)
#[derive(Copy, PartialEq)]
pub enum RequestMode {
    SameOrigin,
    NoCORS,
    CORSMode,
    ForcedPreflightMode
}

/// Request [credentials mode](http://fetch.spec.whatwg.org/#concept-request-credentials-mode)
#[derive(Copy, PartialEq)]
pub enum CredentialsMode {
    Omit,
    CredentialsSameOrigin,
    Include
}

/// [Cache mode](http://fetch.spec.whatwg.org/#concept-request-cache-mode)
#[derive(Copy, PartialEq)]
pub enum CacheMode {
    Default,
    NoStore,
    Reload,
    NoCache,
    ForceCache,
    OnlyIfCached
}

/// [Response tainting](http://fetch.spec.whatwg.org/#concept-request-response-tainting)
#[derive(Copy, PartialEq)]
pub enum ResponseTainting {
    Basic,
    CORSTainting,
    Opaque
}

/// A [Request](http://fetch.spec.whatwg.org/#requests) as defined by the Fetch spec
pub struct Request {
    pub method: Method,
    pub url: Url,
    pub headers: Headers,
    pub unsafe_request: bool,
    pub body: Option<Vec<u8>>,
    pub preserve_content_codings: bool,
    // pub client: GlobalRef, // XXXManishearth copy over only the relevant fields of the global scope,
                              // not the entire scope to avoid the libscript dependency
    pub is_service_worker_global_scope: Option<bool>,
    pub skip_service_worker: bool,
    pub context: Context,
    pub context_frame_type: ContextFrameType,
    pub origin: Option<Url>,
    pub force_origin_header: bool,
    pub same_origin_data: bool,
    pub referer: Referer,
    pub authentication: bool,
    pub sync: bool,
    pub mode: RequestMode,
    pub credentials_mode: CredentialsMode,
    pub use_url_credentials: bool,
    pub cache_mode: CacheMode,
    pub manual_redirect: bool,
    pub redirect_count: uint,
    pub response_tainting: ResponseTainting,
    pub cache: Option<Box<CORSCache+'static>>
}

impl Request {
    pub fn new(url: Url, context: Context, isServiceWorkerGlobalScope: Option<bool>) -> Request {
         Request {
            method: Method::Get,
            url: url,
            headers: Headers::new(),
            unsafe_request: false,
            body: None,
            preserve_content_codings: false,
            is_service_worker_global_scope: isServiceWorkerGlobalScope,
            skip_service_worker: false,
            context: context,
            context_frame_type: ContextFrameType::ContextNone,
            origin: None,
            force_origin_header: false,
            same_origin_data: false,
            referer: Referer::Client,
            authentication: false,
            sync: false,
            mode: RequestMode::NoCORS,
            credentials_mode: CredentialsMode::Omit,
            use_url_credentials: false,
            cache_mode: CacheMode::Default,
            manual_redirect: false,
            redirect_count: 0,
            response_tainting: ResponseTainting::Basic,
            cache: None
        }
    }

    // [Fetch](http://fetch.spec.whatwg.org#fetch)
    pub fn fetch(&mut self, _cors_flag: bool) -> Response {
        Response::network_error()
    }

    /// [Basic fetch](http://fetch.spec.whatwg.org#basic-fetch)
    pub fn basic_fetch(&mut self) -> Response {
        match &*self.url.scheme {
            "about" => match self.url.non_relative_scheme_data() {
                Some(s) if &*s == "blank" => {
                    let mut response = Response::new();
                    response.headers.set(ContentType(Mime(
                        TopLevel::Text, SubLevel::Html,
                        vec![(Attr::Charset, Value::Utf8)])));
                    response
                },
                _ => Response::network_error()
            },
            "http" | "https" => {
                self.http_fetch(false, false, false)
            },
            "blob" | "data" | "file" | "ftp" => {
                // XXXManishearth handle these
                panic!("Unimplemented scheme for Fetch")
            },

            _ => Response::network_error()
        }
    }

    // [HTTP fetch](http://fetch.spec.whatwg.org#http-fetch)
    pub fn http_fetch(&mut self, _cors_flag: bool, cors_preflight_flag: bool, _authentication_fetch_flag: bool) -> Response {
        // Step 1
        let mut response: Option<Response> = None;
        // Step 2
        if !self.skip_service_worker && !self.is_service_worker_global_scope.unwrap_or(false) {
            // TODO: Substep 1 (handle fetch unimplemented)
            // Substep 2
            if let Some(res) = response {
                if (res.response_type == ResponseType::Opaque && self.mode != RequestMode::NoCORS) ||
                   res.response_type == ResponseType::Error {
                    return Response::network_error();
                }
            }
        }
        // Step 3
        if response.is_none() {
            // Substep 1
            if cors_preflight_flag {
                let method_cache_match = self.cache.unwrap().match_method(CacheRequestDetails {
                    origin: self.origin.unwrap_or(Url::parse("").unwrap()),
                    destination: self.url,
                    credentials: self.credentials_mode == CredentialsMode::Include
                }, self.method);
                let condition1 = !method_cache_match && (!is_simple_method(&self.method) ||
                    self.mode == RequestMode::ForcedPreflightMode);
                let condition2 = self.headers.iter().any(|view|
                    !self.cache.unwrap().match_header(CacheRequestDetails {
                        origin: self.origin.unwrap_or(Url::parse("").unwrap()),
                        destination: self.url,
                        credentials: self.credentials_mode == CredentialsMode::Include
                    }, view.name()) && !is_simple_header(&view)
                    );
                if (condition1 || condition2) {
                    response = Some(self.preflight_fetch());
                    if response.unwrap().response_type == ResponseType::Error {
                        return Response::network_error();
                    }
                }
            }
            // Substep 2
            self.skip_service_worker = true;
            // Substep 3
            let credentials = match self.credentials_mode {
                CredentialsMode::Include => true,
                CredentialsMode::CredentialsSameOrigin if !_cors_flag => true,
                _ => false
            };
            // Substep 4
            if self.cache_mode == CacheMode::Default || is_no_store_cache(self.headers) {
                self.cache_mode = CacheMode::NoStore;
            }
            // Substep 5
            response = Some(self.http_network_or_cache_fetch(credentials, _authentication_fetch_flag));
            // Substep 6
            if _cors_flag && self.cors_check(response.unwrap()).is_err() {
                return Response::network_error();
            }
        }
        // Step 4
        let mut response = response.unwrap();
        match response.status.unwrap() {
            // Code 304
            StatusCode::NotModified => match self.cache_mode {
                CacheMode::Default | CacheMode::NoCache => {
                    // TODO: Check HTTP cache for request and response entry
                }
                _ => { }
            },
            // Code 301, 302, 303, 307, 308
            StatusCode::MovedPermanently | StatusCode::Found | StatusCode::SeeOther |
            StatusCode::TemporaryRedirect | StatusCode::PermanentRedirect => {
                // Step 1
                let location = response.headers.get::<Location>();
                // Step 2-3
                match location {
                    Some(val) => if val.as_slice() == "null" { return response; },
                    None => { return Response::network_error(); }
                }
                // Step 4
                let locationUrl = Url::parse(location.unwrap());
                // Step 5
                if locationUrl.is_err() {
                    return Response::network_error();
                }
                let locationUrl = locationUrl.unwrap();
                // Step 6
                if self.redirect_count == 20 {
                    return Response::network_error();
                }
                // Step 7
                self.redirect_count += 1;
                // Substep 8
                self.same_origin_data = false;
                // Step 9
                if cors_preflight_flag {
                    response.response_type = ResponseType::Error;
                    self.manual_redirect = true;
                }
                // Step 10
                if !self.manual_redirect {
                    // FIXME: Origin method of the Url crate hasn't been implemented
                    // Substep 1
                    // if _cors_flag && locationUrl.origin() != self.url.origin() { self.origin = None; }
                    // Substep 2
                    if _cors_flag && (!locationUrl.username().unwrap().is_empty() ||
                                      locationUrl.password().is_some()) {
                        return Response::network_error();
                    }
                    // Substep 3
                    self.url = locationUrl;
                    // Substep 4
                    return self.fetch(_cors_flag);
                }
            }
            // Code 401
            StatusCode::Unauthorized => {
                // Step 1
                if !self.authentication || _cors_flag {
                    return response;
                }
                // Step 2: Spec says requires testing
                // Step 3
                if !self.use_url_credentials || _authentication_fetch_flag {
                    // TODO: Prompt the user for username and password
                }
                return self.http_fetch(_cors_flag, cors_preflight_flag, true);
            }
            // Code 407
            StatusCode::ProxyAuthenticationRequired => {
                // Step 1: Spec says requires testing
                // Step 2
                // TODO: Prompt the user for proxy authentication credentials
                // Step 3
                return self.http_fetch(_cors_flag, cors_preflight_flag, _authentication_fetch_flag);
            }
            _ => { }
        }
        // Step 5
        if _authentication_fetch_flag {
            // TODO: Create authentication entry for this request
        }
        // Step 6
        if cors_preflight_flag && response.response_type == ResponseType::Error {
            self.cache.unwrap().clear(CacheRequestDetails {
                // FIXME: Opaque identifier for origin
                origin: self.origin.unwrap_or(Url::parse("").unwrap()),
                destination: self.url,
                credentials: false
            });
        }
        // Step 7
        response
    }

    // [HTTP network or cache fetch](http://fetch.spec.whatwg.org#http-network-or-cache-fetch)
    pub fn http_network_or_cache_fetch(&mut self, _credentials_flag: bool, _authentication_fetch_flag: bool) -> Response {
        Response::network_error()
    }

    // [CORS preflight fetch](http://fetch.spec.whatwg.org#cors-preflight-fetch)
    pub fn preflight_fetch(&mut self) -> Response {
        Response::network_error()
    }

    // [CORS check](http://fetch.spec.whatwg.org#concept-cors-check)
    pub fn cors_check(&mut self, response: Response) -> Result<(), ()> {
        Err(())
    }
}

fn is_no_store_cache(headers: Headers) -> bool {
    // TODO: Hyper is missing the header parsing for If-Range
    // Add an additional or clause for IfRange once it is implemented
    headers.has::<IfModifiedSince>() | headers.has::<IfNoneMatch>() |
    headers.has::<IfUnmodifiedSince>() | headers.has::<IfMatch>()
}

fn is_simple_header(h: &HeaderView) -> bool {
    //FIXME: use h.is::<HeaderType>() when AcceptLanguage and
    //ContentLanguage headers exist
    match h.name().to_ascii_lowercase().as_slice() {
        "accept" | "accept-language" | "content-language" => true,
        "content-type" => match h.value() {
            Some(&ContentType(Mime(TopLevel::Text, SubLevel::Plain, _))) |
            Some(&ContentType(Mime(TopLevel::Application, SubLevel::WwwFormUrlEncoded, _))) |
            Some(&ContentType(Mime(TopLevel::Multipart, SubLevel::FormData, _))) => true,

            _ => false

        },
        _ => false
    }
}

fn is_simple_method(m: &Method) -> bool {
    match *m {
        Method::Get | Method::Head | Method::Post => true,
        _ => false
    }
}
