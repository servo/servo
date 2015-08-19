/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

//! A partial implementation of CORS
//! For now this library is XHR-specific.
//! For stuff involving `<img>`, `<iframe>`, `<form>`, etc please check what
//! the request mode should be and compare with the fetch spec
//! This library will eventually become the core of the Fetch crate
//! with CORSRequest being expanded into FetchRequest (etc)

use net_traits::{AsyncResponseListener, ResponseAction, Metadata};
use network_listener::{NetworkListener, PreInvoke};
use script_task::ScriptChan;

use std::ascii::AsciiExt;
use std::borrow::ToOwned;
use std::cell::RefCell;
use std::sync::{Arc, Mutex};
use time;
use time::{now, Timespec};

use hyper::client::Request;
use hyper::header::{AccessControlMaxAge, AccessControlAllowOrigin};
use hyper::header::{AccessControlRequestHeaders, AccessControlAllowHeaders};
use hyper::header::{AccessControlRequestMethod, AccessControlAllowMethods};
use hyper::header::{ContentType, Host};
use hyper::header::{Headers, HeaderView};
use hyper::method::Method;
use hyper::mime::{Mime, TopLevel, SubLevel};
use hyper::status::StatusClass::Success;

use unicase::UniCase;
use url::{SchemeData, Url};
use util::mem::HeapSizeOf;
use util::task::spawn_named;

/// Interface for network listeners concerned with CORS checks. Proper network requests
/// should be initiated from this method, based on the response provided.
pub trait AsyncCORSResponseListener {
    fn response_available(&self, response: CORSResponse);
}

#[derive(Clone, HeapSizeOf)]
pub struct CORSRequest {
    pub origin: Url,
    pub destination: Url,
    pub mode: RequestMode,
    pub method: Method,
    #[ignore_heap_size_of = "Defined in hyper"]
    pub headers: Headers,
    /// CORS preflight flag (https://fetch.spec.whatwg.org/#concept-http-fetch)
    /// Indicates that a CORS preflight request and/or cache check is to be performed
    pub preflight_flag: bool
}

/// https://fetch.spec.whatwg.org/#concept-request-mode
/// This only covers some of the request modes. The
/// `same-origin` and `no CORS` modes are unnecessary for XHR.
#[derive(PartialEq, Copy, Clone, HeapSizeOf)]
pub enum RequestMode {
    CORS, // CORS
    ForcedPreflight // CORS-with-forced-preflight
}

impl CORSRequest {
    /// Creates a CORS request if necessary. Will return an error when fetching is forbidden
    pub fn maybe_new(referer: Url, destination: Url, mode: RequestMode,
                     method: Method, headers: Headers) -> Result<Option<CORSRequest>, ()> {
        if referer.scheme == destination.scheme &&
           referer.host() == destination.host() &&
           referer.port() == destination.port() {
            return Ok(None); // Not cross-origin, proceed with a normal fetch
        }
        match &*destination.scheme {
            // TODO: If the request's same origin data url flag is set (which isn't the case for XHR)
            // we can fetch a data URL normally. about:blank can also be fetched by XHR
            "http" | "https" => {
                let mut req = CORSRequest::new(referer, destination, mode, method, headers);
                req.preflight_flag = !is_simple_method(&req.method) || mode == RequestMode::ForcedPreflight;
                if req.headers.iter().all(|h| is_simple_header(&h)) {
                    req.preflight_flag = true;
                }
                Ok(Some(req))
            },
            _ => Err(()),
        }
    }

    fn new(mut referer: Url, destination: Url, mode: RequestMode, method: Method,
           headers: Headers) -> CORSRequest {
        match referer.scheme_data {
            SchemeData::Relative(ref mut data) => data.path = vec!(),
            _ => {}
        };
        referer.fragment = None;
        referer.query = None;
        CORSRequest {
            origin: referer,
            destination: destination,
            mode: mode,
            method: method,
            headers: headers,
            preflight_flag: false
        }
    }

    pub fn http_fetch_async(&self,
                            listener: Box<AsyncCORSResponseListener + Send>,
                            script_chan: Box<ScriptChan + Send>) {
        struct CORSContext {
            listener: Box<AsyncCORSResponseListener + Send>,
            response: RefCell<Option<CORSResponse>>,
        }

        // This is shoe-horning the CORSReponse stuff into the rest of the async network
        // framework right now. It would be worth redesigning http_fetch to do this properly.
        impl AsyncResponseListener for CORSContext {
            fn headers_available(&self, _metadata: Metadata) {
            }

            fn data_available(&self, _payload: Vec<u8>) {
            }

            fn response_complete(&self, _status: Result<(), String>) {
                let response = self.response.borrow_mut().take().unwrap();
                self.listener.response_available(response);
            }
        }
        impl PreInvoke for CORSContext {}

        let context = CORSContext {
            listener: listener,
            response: RefCell::new(None),
        };
        let listener = NetworkListener {
            context: Arc::new(Mutex::new(context)),
            script_chan: script_chan,
        };

        // TODO: this exists only to make preflight check non-blocking
        // perhaps should be handled by the resource task?
        let req = self.clone();
        spawn_named("cors".to_owned(), move || {
            let response = req.http_fetch();
            let mut context = listener.context.lock();
            let context = context.as_mut().unwrap();
            *context.response.borrow_mut() = Some(response);
            listener.notify(ResponseAction::ResponseComplete(Ok(())));
        });
    }

    /// http://fetch.spec.whatwg.org/#concept-http-fetch
    /// This method assumes that the CORS flag is set
    /// This does not perform the full HTTP fetch, rather it handles part of the CORS filtering
    /// if self.mode is ForcedPreflight, then the CORS-with-forced-preflight
    /// fetch flag is set as well
    pub fn http_fetch(&self) -> CORSResponse {
        let response = CORSResponse::new();
        // Step 2: Handle service workers (unimplemented)
        // Step 3
        // Substep 1: Service workers (unimplemented )
        // Substep 2
        let cache = &mut CORSCache(vec!()); // XXXManishearth Should come from user agent
        if self.preflight_flag &&
           !cache.match_method(self, &self.method) &&
           !self.headers.iter().all(|h| is_simple_header(&h) && cache.match_header(self, h.name())) {
            if !is_simple_method(&self.method) || self.mode == RequestMode::ForcedPreflight {
                return self.preflight_fetch();
                // Everything after this is part of XHR::fetch()
                // Expect the organization of code to improve once we have a fetch crate
            }
        }
        response
    }

    /// https://fetch.spec.whatwg.org/#cors-preflight-fetch
    fn preflight_fetch(&self) -> CORSResponse {
        let error = CORSResponse::new_error();
        let mut cors_response = CORSResponse::new();

        let mut preflight = self.clone(); // Step 1
        preflight.method = Method::Options; // Step 2
        preflight.headers = Headers::new(); // Step 3
        // Step 4
        preflight.headers.set(AccessControlRequestMethod(self.method.clone()));

        // Step 5 - 7
        let mut header_names = vec!();
        for header in self.headers.iter() {
            header_names.push(header.name().to_owned());
        }
        header_names.sort();
        preflight.headers.set(AccessControlRequestHeaders(header_names.into_iter().map(UniCase).collect()));

        // Step 8 unnecessary, we don't use the request body
        // Step 9, 10 unnecessary, we're writing our own fetch code

        // Step 11
        let preflight_request = Request::new(preflight.method, preflight.destination);
        let mut req = match preflight_request {
            Ok(req) => req,
            Err(_) => return error
        };

        let host = req.headers().get::<Host>().unwrap().clone();
        *req.headers_mut() = preflight.headers.clone();
        req.headers_mut().set(host);
        let stream = match req.start() {
            Ok(s) => s,
            Err(_) => return error
        };
        let response = match stream.send() {
            Ok(r) => r,
            Err(_) => return error
        };

        // Step 12
        match response.status.class() {
            Success => {}
            _ => return error
        }
        cors_response.headers = response.headers.clone();
        // Substeps 1-3 (parsing rules: https://fetch.spec.whatwg.org/#http-new-header-syntax)
        let methods_substep4 = [self.method.clone()];
        let mut methods = match response.headers.get() {
            Some(&AccessControlAllowMethods(ref v)) => &**v,
            _ => return error
        };
        let headers = match response.headers.get() {
            Some(&AccessControlAllowHeaders(ref h)) => h,
            _ => return error
        };
        // Substep 4
        if methods.is_empty() || preflight.mode == RequestMode::ForcedPreflight {
            methods = &methods_substep4;
        }
        // Substep 5
        if !is_simple_method(&self.method) &&
           !methods.iter().any(|m| m == &self.method) {
           return error;
        }
        // Substep 6
        for h in self.headers.iter() {
            if is_simple_header(&h) {
                continue;
            }
            if !headers.iter().any(|ref h2| h.name().eq_ignore_ascii_case(h2)) {
                return error;
            }
        }
        // Substep 7, 8
        let max_age = match response.headers.get() {
            Some(&AccessControlMaxAge(num)) => num,
            None => 0
        };
        // Substep 9: Impose restrictions on max-age, if any (unimplemented)
        // Substeps 10-12: Add a cache (partially implemented, XXXManishearth)
        // This cache should come from the user agent, creating a new one here to check
        // for compile time errors
        let cache = &mut CORSCache(vec!());
        for m in methods {
            let cache_match = cache.match_method_and_update(self, m, max_age);
            if !cache_match {
                cache.insert(CORSCacheEntry::new(self.origin.clone(), self.destination.clone(),
                                                 max_age, false, HeaderOrMethod::MethodData(m.clone())));
            }
        }
        for h in response.headers.iter() {
            let cache_match = cache.match_header_and_update(self, h.name(), max_age);
            if !cache_match {
                cache.insert(CORSCacheEntry::new(self.origin.clone(), self.destination.clone(),
                                                 max_age, false, HeaderOrMethod::HeaderData(h.to_string())));
            }
        }
        cors_response
    }
}


pub struct CORSResponse {
    pub network_error: bool,
    pub headers: Headers
}

impl CORSResponse {
    fn new() -> CORSResponse {
        CORSResponse {
            network_error: false,
            headers: Headers::new()
        }
    }

    fn new_error() -> CORSResponse {
         CORSResponse {
            network_error: true,
            headers: Headers::new()
        }
    }
}

// CORS Cache stuff

/// A CORS cache object. Anchor it somewhere to the user agent.
#[derive(Clone)]
pub struct CORSCache(Vec<CORSCacheEntry>);

/// Union type for CORS cache entries
/// Each entry might pertain to a header or method
#[derive(Clone)]
pub enum HeaderOrMethod {
    HeaderData(String),
    MethodData(Method)
}

impl HeaderOrMethod {
    fn match_header(&self, header_name: &str) -> bool {
        match *self {
            HeaderOrMethod::HeaderData(ref s) => s.eq_ignore_ascii_case(header_name),
            _ => false
        }
    }

    fn match_method(&self, method: &Method) -> bool {
        match *self {
            HeaderOrMethod::MethodData(ref m) => m == method,
            _ => false
        }
    }
}

// An entry in the CORS cache
#[derive(Clone)]
pub struct CORSCacheEntry {
    pub origin: Url,
    pub url: Url,
    pub max_age: u32,
    pub credentials: bool,
    pub header_or_method: HeaderOrMethod,
    created: Timespec
}

impl CORSCacheEntry {
    fn new(origin: Url,
           url: Url,
           max_age: u32,
           credentials: bool,
           header_or_method: HeaderOrMethod) -> CORSCacheEntry {
        CORSCacheEntry {
            origin: origin,
            url: url,
            max_age: max_age,
            credentials: credentials,
            header_or_method: header_or_method,
            created: time::now().to_timespec()
        }
    }
}

impl CORSCache {
    /// https://fetch.spec.whatwg.org/#concept-cache-clear
    #[allow(dead_code)]
    fn clear(&mut self, request: &CORSRequest) {
        let CORSCache(buf) = self.clone();
        let new_buf: Vec<CORSCacheEntry> =
            buf.into_iter()
               .filter(|e| e.origin == request.origin && request.destination == e.url)
               .collect();
        *self = CORSCache(new_buf);
    }

    // Remove old entries
    fn cleanup(&mut self) {
        let CORSCache(buf) = self.clone();
        let now = time::now().to_timespec();
        let new_buf: Vec<CORSCacheEntry> = buf.into_iter()
                                              .filter(|e| now.sec > e.created.sec + e.max_age as i64)
                                              .collect();
        *self = CORSCache(new_buf);
    }

    /// https://fetch.spec.whatwg.org/#concept-cache-match-header
    fn find_entry_by_header<'a>(&'a mut self,
                                request: &CORSRequest,
                                header_name: &str) -> Option<&'a mut CORSCacheEntry> {
        self.cleanup();
        let CORSCache(ref mut buf) = *self;
        // Credentials are not yet implemented here
        let entry = buf.iter_mut().find(|e| e.origin.scheme == request.origin.scheme &&
                            e.origin.host() == request.origin.host() &&
                            e.origin.port() == request.origin.port() &&
                            e.url == request.destination &&
                            e.header_or_method.match_header(header_name));
        entry
    }

    fn match_header(&mut self, request: &CORSRequest, header_name: &str) -> bool {
        self.find_entry_by_header(request, header_name).is_some()
    }

    fn match_header_and_update(&mut self, request: &CORSRequest, header_name: &str, new_max_age: u32) -> bool {
        self.find_entry_by_header(request, header_name).map(|e| e.max_age = new_max_age).is_some()
    }

    fn find_entry_by_method<'a>(&'a mut self,
                                request: &CORSRequest,
                                method: &Method) -> Option<&'a mut CORSCacheEntry> {
        // we can take the method from CORSRequest itself
        self.cleanup();
        let CORSCache(ref mut buf) = *self;
        // Credentials are not yet implemented here
        let entry = buf.iter_mut().find(|e| e.origin.scheme == request.origin.scheme &&
                            e.origin.host() == request.origin.host() &&
                            e.origin.port() == request.origin.port() &&
                            e.url == request.destination &&
                            e.header_or_method.match_method(method));
        entry
    }

    /// https://fetch.spec.whatwg.org/#concept-cache-match-method
    fn match_method(&mut self, request: &CORSRequest, method: &Method) -> bool {
        self.find_entry_by_method(request, method).is_some()
    }

    fn match_method_and_update(&mut self, request: &CORSRequest, method: &Method, new_max_age: u32) -> bool {
        self.find_entry_by_method(request, method).map(|e| e.max_age = new_max_age).is_some()
    }

    fn insert(&mut self, entry: CORSCacheEntry) {
        self.cleanup();
        let CORSCache(ref mut buf) = *self;
        buf.push(entry);
    }
}

fn is_simple_header(h: &HeaderView) -> bool {
    //FIXME: use h.is::<HeaderType>() when AcceptLanguage and
    //ContentLanguage headers exist
    match &*h.name().to_ascii_lowercase() {
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

/// Perform a CORS check on a header list and CORS request
/// https://fetch.spec.whatwg.org/#cors-check
pub fn allow_cross_origin_request(req: &CORSRequest, headers: &Headers) -> bool {
    match headers.get::<AccessControlAllowOrigin>() {
        Some(&AccessControlAllowOrigin::Any) => true, // Not always true, depends on credentials mode
        Some(&AccessControlAllowOrigin::Value(ref url)) => req.origin.serialize() == *url,
        Some(&AccessControlAllowOrigin::Null) |
        None => false
    }
}
