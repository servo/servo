/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use url::Url;
use hyper::method::Method;
use hyper::mime::{Mime, TopLevel, SubLevel, Attr, Value};
use hyper::header::{Header, Headers, ContentType, IfModifiedSince, IfNoneMatch};
use hyper::header::{Accept, IfUnmodifiedSince, IfMatch, IfRange, Location};
use hyper::header::{HeaderView, AcceptLanguage, ContentLanguage};
use hyper::header::{QualityItem, qitem, q};
use hyper::status::StatusCode;
use fetch::cors_cache::{CORSCache, CacheRequestDetails};
use fetch::response::{Response, ResponseType};
use std::ascii::AsciiExt;
use std::str::FromStr;

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
pub enum Referer {
    RefererNone,
    Client,
    RefererUrl(Url)
}

/// A [request mode](https://fetch.spec.whatwg.org/#concept-request-mode)
#[derive(Copy, Clone, PartialEq)]
pub enum RequestMode {
    SameOrigin,
    NoCORS,
    CORSMode,
    ForcedPreflightMode
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

/// A [Request](https://fetch.spec.whatwg.org/#requests) as defined by the Fetch spec
pub struct Request {
    pub method: Method,
    pub url: Url,
    pub headers: Headers,
    pub unsafe_request: bool,
    pub body: Option<Vec<u8>>,
    pub preserve_content_codings: bool,
    // pub client: GlobalRef, // XXXManishearth copy over only the relevant fields of the global scope,
                              // not the entire scope to avoid the libscript dependency
    pub is_service_worker_global_scope: bool,
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
    pub redirect_mode: RedirectMode,
    pub redirect_count: usize,
    pub response_tainting: ResponseTainting,
    pub cache: Option<Box<CORSCache + 'static>>
}

impl Request {
    pub fn new(url: Url, context: Context, is_service_worker_global_scope: bool) -> Request {
         Request {
            method: Method::Get,
            url: url,
            headers: Headers::new(),
            unsafe_request: false,
            body: None,
            preserve_content_codings: false,
            is_service_worker_global_scope: is_service_worker_global_scope,
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
            redirect_mode: RedirectMode::Follow,
            redirect_count: 0,
            response_tainting: ResponseTainting::Basic,
            cache: None
        }
    }

    /// [Fetch](https://fetch.spec.whatwg.org#concept-fetch)
    pub fn fetch(&mut self, cors_flag: bool) -> Response {
        // Step 1
        if self.context != Context::Fetch && !self.headers.has::<Accept>() {
            // Substep 1
            let value = match self.context {
                Context::Favicon | Context::Image | Context::ImageSet
                    => vec![qitem(Mime(TopLevel::Image, SubLevel::Png, vec![])),
                        // FIXME: This should properly generate a MimeType that has a
                        // SubLevel of svg+xml (https://github.com/hyperium/mime.rs/issues/22)
                        qitem(Mime(TopLevel::Image, SubLevel::Ext("svg+xml".to_string()), vec![])),
                        QualityItem::new(Mime(TopLevel::Image, SubLevel::Star, vec![]), q(0.8)),
                        QualityItem::new(Mime(TopLevel::Star, SubLevel::Star, vec![]), q(0.5))],
                Context::Form | Context::Frame | Context::Hyperlink |
                Context::IFrame | Context::Location | Context::MetaRefresh |
                Context::PreRender
                    => vec![qitem(Mime(TopLevel::Text, SubLevel::Html, vec![])),
                        // FIXME: This should properly generate a MimeType that has a
                        // SubLevel of xhtml+xml (https://github.com/hyperium/mime.rs/issues/22)
                        qitem(Mime(TopLevel::Application, SubLevel::Ext("xhtml+xml".to_string()), vec![])),
                        QualityItem::new(Mime(TopLevel::Application, SubLevel::Xml, vec![]), q(0.9)),
                        QualityItem::new(Mime(TopLevel::Star, SubLevel::Star, vec![]), q(0.8))],
                Context::Internal if self.context_frame_type != ContextFrameType::ContextNone
                    => vec![qitem(Mime(TopLevel::Text, SubLevel::Html, vec![])),
                        // FIXME: This should properly generate a MimeType that has a
                        // SubLevel of xhtml+xml (https://github.com/hyperium/mime.rs/issues/22)
                        qitem(Mime(TopLevel::Application, SubLevel::Ext("xhtml+xml".to_string()), vec![])),
                        QualityItem::new(Mime(TopLevel::Application, SubLevel::Xml, vec![]), q(0.9)),
                        QualityItem::new(Mime(TopLevel::Star, SubLevel::Star, vec![]), q(0.8))],
                Context::Style
                    => vec![qitem(Mime(TopLevel::Text, SubLevel::Css, vec![])),
                        QualityItem::new(Mime(TopLevel::Star, SubLevel::Star, vec![]), q(0.1))],
                _ => vec![qitem(Mime(TopLevel::Star, SubLevel::Star, vec![]))]
            };
            // Substep 2
            self.headers.set(Accept(value));
        }
        // Step 2
        if self.context != Context::Fetch && !self.headers.has::<AcceptLanguage>() {
            self.headers.set(AcceptLanguage(vec![qitem("en-US".parse().unwrap())]));
        }
        // TODO: Figure out what a Priority object is
        // Step 3
        // Step 4
        self.main_fetch(cors_flag)
    }

    /// [Main fetch](https://fetch.spec.whatwg.org/#concept-main-fetch)
    pub fn main_fetch(&mut self, _cors_flag: bool) -> Response {
        // TODO: Implement main fetch spec
        Response::network_error()
    }

    /// [Basic fetch](https://fetch.spec.whatwg.org#basic-fetch)
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

    /// [HTTP fetch](https://fetch.spec.whatwg.org#http-fetch)
    pub fn http_fetch(&mut self, cors_flag: bool, cors_preflight_flag: bool,
                      authentication_fetch_flag: bool) -> Response {
        // Step 1
        let mut response: Option<Response> = None;
        // Step 2
        if !self.skip_service_worker && !self.is_service_worker_global_scope {
            // TODO: Substep 1 (handle fetch unimplemented)
            // Substep 2
            if let Some(ref res) = response {
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
                let mut method_mismatch = false;
                let mut header_mismatch = false;
                if let Some(ref mut cache) = self.cache {
                    let origin = self.origin.clone().unwrap_or(Url::parse("").unwrap());
                    let url = self.url.clone();
                    let credentials = self.credentials_mode == CredentialsMode::Include;
                    let method_cache_match = cache.match_method(CacheRequestDetails {
                        origin: origin.clone(),
                        destination: url.clone(),
                        credentials: credentials
                    }, self.method.clone());
                    method_mismatch = !method_cache_match && (!is_simple_method(&self.method) ||
                        self.mode == RequestMode::ForcedPreflightMode);
                    header_mismatch = self.headers.iter().any(|view|
                        !cache.match_header(CacheRequestDetails {
                            origin: origin.clone(),
                            destination: url.clone(),
                            credentials: credentials
                        }, view.name()) && !is_simple_header(&view)
                        );
                }
                if method_mismatch || header_mismatch {
                    let preflight_result = self.preflight_fetch();
                    if preflight_result.response_type == ResponseType::Error {
                        return Response::network_error();
                    }
                    response = Some(preflight_result);
                }
            }
            // Substep 2
            self.skip_service_worker = true;
            // Substep 3
            let credentials = match self.credentials_mode {
                CredentialsMode::Include => true,
                CredentialsMode::CredentialsSameOrigin if !cors_flag => true,
                _ => false
            };
            // Substep 4
            if self.cache_mode == CacheMode::Default && is_no_store_cache(&self.headers) {
                self.cache_mode = CacheMode::NoStore;
            }
            // Substep 5
            let fetch_result = self.http_network_or_cache_fetch(credentials, authentication_fetch_flag);
            // Substep 6
            if cors_flag && self.cors_check(&fetch_result).is_err() {
                return Response::network_error();
            }
            response = Some(fetch_result);
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
                if self.redirect_mode == RedirectMode::Error {
                    return Response::network_error();
                }
                // Step 2-4
                if !response.headers.has::<Location>() {
                    return response;
                }
                let location = match response.headers.get::<Location>() {
                    None => return Response::network_error(),
                    Some(location) => location,
                };
                // Step 5
                let location_url = Url::parse(location);
                // Step 6
                let location_url = match location_url {
                    Ok(url) => url,
                    Err(_) => return Response::network_error()
                };
                // Step 7
                if self.redirect_count == 20 {
                    return Response::network_error();
                }
                // Step 8
                self.redirect_count += 1;
                // Step 9
                self.same_origin_data = false;
                // Step 10
                if self.redirect_mode == RedirectMode::Follow {
                    // FIXME: Origin method of the Url crate hasn't been implemented
                    // https://github.com/servo/rust-url/issues/54

                    // Substep 1
                    // if cors_flag && location_url.origin() != self.url.origin() { self.origin = None; }
                    // Substep 2
                    if cors_flag && (!location_url.username().unwrap_or("").is_empty() ||
                                      location_url.password().is_some()) {
                        return Response::network_error();
                    }
                    // Substep 3
                    if response.status.unwrap() == StatusCode::MovedPermanently ||
                       response.status.unwrap() == StatusCode::SeeOther ||
                       (response.status.unwrap() == StatusCode::Found && self.method == Method::Post) {
                        self.method = Method::Get;
                    }
                    // Substep 4
                    self.url = location_url;
                    // Substep 5
                    return self.fetch(cors_flag);
                }
            }
            // Code 401
            StatusCode::Unauthorized => {
                // Step 1
                if !self.authentication || cors_flag {
                    return response;
                }
                // Step 2
                // TODO: Spec says requires testing
                // Step 3
                if !self.use_url_credentials || authentication_fetch_flag {
                    // TODO: Prompt the user for username and password
                }
                return self.http_fetch(cors_flag, cors_preflight_flag, true);
            }
            // Code 407
            StatusCode::ProxyAuthenticationRequired => {
                // Step 1
                // TODO: Spec says requires testing
                // Step 2
                // TODO: Prompt the user for proxy authentication credentials
                // Step 3
                return self.http_fetch(cors_flag, cors_preflight_flag, authentication_fetch_flag);
            }
            _ => { }
        }
        // Step 5
        if authentication_fetch_flag {
            // TODO: Create authentication entry for this request
        }
        // Step 6
        response
    }

    /// [HTTP network or cache fetch](https://fetch.spec.whatwg.org#http-network-or-cache-fetch)
    pub fn http_network_or_cache_fetch(&mut self,
                                       _credentials_flag: bool,
                                       _authentication_fetch_flag: bool) -> Response {
        // TODO: Implement HTTP network or cache fetch spec
        Response::network_error()
    }

    /// [CORS preflight fetch](https://fetch.spec.whatwg.org#cors-preflight-fetch)
    pub fn preflight_fetch(&mut self) -> Response {
        // TODO: Implement preflight fetch spec
        Response::network_error()
    }

    /// [CORS check](https://fetch.spec.whatwg.org#concept-cors-check)
    pub fn cors_check(&mut self, response: &Response) -> Result<(), ()> {
        // TODO: Implement CORS check spec
        Err(())
    }
}

fn is_no_store_cache(headers: &Headers) -> bool {
    headers.has::<IfModifiedSince>() | headers.has::<IfNoneMatch>() |
    headers.has::<IfUnmodifiedSince>() | headers.has::<IfMatch>() |
    headers.has::<IfRange>()
}

fn is_simple_header(h: &HeaderView) -> bool {
    if h.is::<ContentType>() {
        match h.value() {
            Some(&ContentType(Mime(TopLevel::Text, SubLevel::Plain, _))) |
            Some(&ContentType(Mime(TopLevel::Application, SubLevel::WwwFormUrlEncoded, _))) |
            Some(&ContentType(Mime(TopLevel::Multipart, SubLevel::FormData, _))) => true,
            _ => false

        }
    } else {
        h.is::<Accept>() || h.is::<AcceptLanguage>() || h.is::<ContentLanguage>()
    }
}

fn is_simple_method(m: &Method) -> bool {
    match *m {
        Method::Get | Method::Head | Method::Post => true,
        _ => false
    }
}
