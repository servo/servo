/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

//! A partial implementation of CORS
//! For now this library is XHR-specific.
//! For stuff involving `<img>`, `<iframe>`, `<form>`, etc please check what
//! the request mode should be and compare with the fetch spec
//! This library will eventually become the core of the Fetch crate
//! with CORSRequest being expanded into FetchRequest (etc)

use std::ascii::{StrAsciiExt, OwnedStrAsciiExt};
use std::from_str::FromStr;
use std::io::BufReader;
use std::str::StrSlice;
use time;
use time::{now, Timespec};

use http::headers::response::HeaderCollection as ResponseHeaderCollection;
use http::headers::request::HeaderCollection as RequestHeaderCollection;
use http::headers::request::Header as RequestHeader;

use http::client::{RequestWriter, NetworkStream};
use http::headers::{HeaderConvertible, HeaderEnum, HeaderValueByteIterator};
use http::headers::content_type::MediaType;
use http::headers::request::{Accept, AcceptLanguage, ContentLanguage, ContentType};
use http::method::{Method, Get, Head, Post, Options};

use url::{RelativeSchemeData, Url, UrlParser};

#[deriving(Clone)]
pub struct CORSRequest {
    pub origin: Url,
    pub destination: Url,
    pub mode: RequestMode,
    pub method: Method,
    pub headers: RequestHeaderCollection,
    /// CORS preflight flag (http://fetch.spec.whatwg.org/#concept-http-fetch)
    /// Indicates that a CORS preflight request and/or cache check is to be performed
    pub preflight_flag: bool
}

/// http://fetch.spec.whatwg.org/#concept-request-mode
/// This only covers some of the request modes. The
/// `same-origin` and `no CORS` modes are unnecessary for XHR.
#[deriving(PartialEq, Clone)]
pub enum RequestMode {
    CORSMode, // CORS
    ForcedPreflightMode // CORS-with-forced-preflight
}

impl CORSRequest {
    /// Creates a CORS request if necessary. Will return an error when fetching is forbidden
    pub fn maybe_new(referer: Url, destination: Url, mode: RequestMode,
                     method: Method, headers: RequestHeaderCollection) -> Result<Option<CORSRequest>, ()> {
        if referer.scheme == destination.scheme &&
           referer.host() == destination.host() &&
           referer.port() == destination.port() {
            return Ok(None); // Not cross-origin, proceed with a normal fetch
        }
        match destination.scheme.as_slice() {
            // Todo: If the request's same origin data url flag is set (which isn't the case for XHR)
            // we can fetch a data URL normally. about:blank can also be fetched by XHR
            "http" | "https" => {
                let mut req = CORSRequest::new(referer, destination, mode, method, headers);
                req.preflight_flag = !is_simple_method(&req.method) || mode == ForcedPreflightMode;
                if req.headers.iter().all(|h| is_simple_header(&h)) {
                    req.preflight_flag = true;
                }
                Ok(Some(req))
            },
            _ => Err(()),
        }
    }

    fn new(mut referer: Url, destination: Url, mode: RequestMode, method: Method,
           headers: RequestHeaderCollection) -> CORSRequest {
        match referer.scheme_data {
            RelativeSchemeData(ref mut data) => data.path = vec!(),
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

    /// http://fetch.spec.whatwg.org/#concept-http-fetch
    /// This method assumes that the CORS flag is set
    /// This does not perform the full HTTP fetch, rather it handles part of the CORS filtering
    /// if self.mode is ForcedPreflightMode, then the CORS-with-forced-preflight
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
           !self.headers.iter().all(|h| is_simple_header(&h) && cache.match_header(self, h.header_name().as_slice())) {
            if !is_simple_method(&self.method) || self.mode == ForcedPreflightMode {
                return self.preflight_fetch();
                // Everything after this is part of XHR::fetch()
                // Expect the organization of code to improve once we have a fetch crate
            }
        }
        response
    }

    /// http://fetch.spec.whatwg.org/#cors-preflight-fetch
    fn preflight_fetch(&self) -> CORSResponse {
        let error = CORSResponse::new_error();
        let mut cors_response = CORSResponse::new();

        let mut preflight = self.clone(); // Step 1
        preflight.method = Options; // Step 2
        preflight.headers = RequestHeaderCollection::new(); // Step 3
        // Step 4
        preflight.insert_string_header("Access-Control-Request-Method".to_string(), self.method.http_value());

        // Step 5 - 7
        let mut header_names = vec!();
        for header in self.headers.iter() {
            header_names.push(header.header_name().into_ascii_lower());
        }
        header_names.sort();
        let header_list = header_names.connect(", "); // 0x2C 0x20
        preflight.insert_string_header("Access-Control-Request-Headers".to_string(), header_list);

        // Step 8 unnecessary, we don't use the request body
        // Step 9, 10 unnecessary, we're writing our own fetch code

        // Step 11
        let preflight_request = RequestWriter::<NetworkStream>::new(preflight.method, preflight.destination);
        let mut writer = match preflight_request {
            Ok(w) => box w,
            Err(_) => return error
        };

        let host = writer.headers.host.clone();
        writer.headers = preflight.headers.clone();
        writer.headers.host = host;
        let response = match writer.read_response() {
            Ok(r) => r,
            Err(_) => return error
        };

        // Step 12
        match response.status.code() {
         200 .. 299 => {}
         _ => return error
        }
        cors_response.headers = response.headers.clone();
        // Substeps 1-3 (parsing rules: http://fetch.spec.whatwg.org/#http-new-header-syntax)
        fn find_header(headers: &ResponseHeaderCollection, name: &str) -> Option<String> {
            headers.iter().find(|h| h.header_name().as_slice()
                                                   .eq_ignore_ascii_case(name))
                                    .map(|h| h.header_value())
        }
        let methods_string = match find_header(&response.headers, "Access-Control-Allow-Methods") {
            Some(s) => s,
            _ => return error
        };
        let methods = methods_string.as_slice().split(',');
        let headers_string = match find_header(&response.headers, "Access-Control-Allow-Headers") {
            Some(s) => s,
            _ => return error
        };
        let headers = headers_string.as_slice().split(0x2Cu8 as char);
        // The ABNF # rule will consider consecutive delimeters as a single delimeter
        let mut methods: Vec<String> = methods.filter(|s| s.len() > 0).map(|s| s.to_string()).collect();
        let headers: Vec<String> = headers.filter(|s| s.len() > 0).map(|s| s.to_string()).collect();
        // Substep 4
        if methods.len() == 0 || preflight.mode == ForcedPreflightMode {
            methods = vec!(self.method.http_value());
        }
        // Substep 5
        if !is_simple_method(&self.method) &&
           !methods.iter().any(|ref m| self.method.http_value().as_slice().eq_ignore_ascii_case(m.as_slice())) {
           return error;
        }
        // Substep 6
        for h in self.headers.iter() {
            if is_simple_header(&h) {
                continue;
            }
            if !headers.iter().any(|ref h2| h.header_name().as_slice().eq_ignore_ascii_case(h2.as_slice())) {
                return error;
            }
        }
        // Substep 7, 8
        let max_age: uint = find_header(&response.headers, "Access-Control-Max-Age")
                                .and_then(|h| FromStr::from_str(h.as_slice())).unwrap_or(0);
        // Substep 9: Impose restrictions on max-age, if any (unimplemented)
        // Substeps 10-12: Add a cache (partially implemented, XXXManishearth)
        // This cache should come from the user agent, creating a new one here to check
        // for compile time errors
        let cache = &mut CORSCache(vec!());
        for m in methods.iter() {
            let maybe_method: Option<Method> = FromStr::from_str(m.as_slice());
            maybe_method.map(|ref m| {
                let cache_match = cache.match_method_and_update(self, m, max_age);
                if !cache_match {
                    cache.insert(CORSCacheEntry::new(self.origin.clone(), self.destination.clone(),
                                                     max_age, false, MethodData(m.clone())));
                }
            });
        }
        for h in headers.iter() {
            let cache_match = cache.match_header_and_update(self, h.as_slice(), max_age);
            if !cache_match {
                cache.insert(CORSCacheEntry::new(self.origin.clone(), self.destination.clone(),
                                                 max_age, false, HeaderData(h.to_string())));
            }
        }
        cors_response
    }

    fn insert_string_header(&mut self, name: String, value: String) {
        let value_bytes = value.into_bytes();
        let mut reader = BufReader::new(value_bytes.as_slice());
        let maybe_header: Option<RequestHeader> = HeaderEnum::value_from_stream(
                                                                String::from_str(name.as_slice()),
                                                                &mut HeaderValueByteIterator::new(&mut reader));
        self.headers.insert(maybe_header.unwrap());
    }
}


pub struct CORSResponse {
    pub network_error: bool,
    pub headers: ResponseHeaderCollection
}

impl CORSResponse {
    fn new() -> CORSResponse {
        CORSResponse {
            network_error: false,
            headers: ResponseHeaderCollection::new()
        }
    }

    fn new_error() -> CORSResponse {
         CORSResponse {
            network_error: true,
            headers: ResponseHeaderCollection::new()
        }
    }
}

// CORS Cache stuff

/// A CORS cache object. Anchor it somewhere to the user agent.
#[deriving(Clone)]
pub struct CORSCache(Vec<CORSCacheEntry>);

/// Union type for CORS cache entries
/// Each entry might pertain to a header or method
#[deriving(Clone)]
pub enum HeaderOrMethod {
    HeaderData(String),
    MethodData(Method)
}

impl HeaderOrMethod {
    fn match_header(&self, header_name: &str) -> bool {
        match *self {
            HeaderData(ref s) => s.as_slice().eq_ignore_ascii_case(header_name),
            _ => false
        }
    }

    fn match_method(&self, method: &Method) -> bool {
        match *self {
            MethodData(ref m) => m == method,
            _ => false
        }
    }
}

// An entry in the CORS cache
#[deriving(Clone)]
pub struct CORSCacheEntry {
    pub origin: Url,
    pub url: Url,
    pub max_age: uint,
    pub credentials: bool,
    pub header_or_method: HeaderOrMethod,
    created: Timespec
}

impl CORSCacheEntry {
    fn new (origin:Url, url: Url, max_age: uint, credentials: bool, header_or_method: HeaderOrMethod) -> CORSCacheEntry {
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
    /// http://fetch.spec.whatwg.org/#concept-cache-clear
    #[allow(dead_code)]
    fn clear (&mut self, request: &CORSRequest) {
        let CORSCache(buf) = self.clone();
        let new_buf: Vec<CORSCacheEntry> = buf.into_iter().filter(|e| e.origin == request.origin && request.destination == e.url).collect();
        *self = CORSCache(new_buf);
    }

    // Remove old entries
    fn cleanup(&mut self) {
        let CORSCache(buf) = self.clone();
        let now = time::now().to_timespec();
        let new_buf: Vec<CORSCacheEntry> = buf.into_iter().filter(|e| now.sec > e.created.sec + e.max_age as i64).collect();
        *self = CORSCache(new_buf);
    }

    /// http://fetch.spec.whatwg.org/#concept-cache-match-header
    fn find_entry_by_header<'a>(&'a mut self, request: &CORSRequest, header_name: &str) -> Option<&'a mut CORSCacheEntry> {
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

    fn match_header_and_update(&mut self, request: &CORSRequest, header_name: &str, new_max_age: uint) -> bool {
        self.find_entry_by_header(request, header_name).map(|e| e.max_age = new_max_age).is_some()
    }

    fn find_entry_by_method<'a>(&'a mut self, request: &CORSRequest, method: &Method) -> Option<&'a mut CORSCacheEntry> {
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

    /// http://fetch.spec.whatwg.org/#concept-cache-match-method
    fn match_method(&mut self, request: &CORSRequest, method: &Method) -> bool {
        self.find_entry_by_method(request, method).is_some()
    }

    fn match_method_and_update(&mut self, request: &CORSRequest, method: &Method, new_max_age: uint) -> bool {
        self.find_entry_by_method(request, method).map(|e| e.max_age = new_max_age).is_some()
    }

    fn insert(&mut self, entry: CORSCacheEntry) {
        self.cleanup();
        let CORSCache(ref mut buf) = *self;
        buf.push(entry);
    }
}

fn is_simple_header(h: &RequestHeader) -> bool {
    match *h {
        Accept(_) | AcceptLanguage(_) | ContentLanguage(_) => true,
        ContentType(MediaType {type_: ref t, subtype: ref s, ..}) => match (t.as_slice(), s.as_slice()) {
            ("text", "plain") | ("application", "x-www-form-urlencoded") | ("multipart", "form-data") => true,
            _ => false
        },
        _ => false
    }
}

fn is_simple_method(m: &Method) -> bool {
    match *m {
        Get | Head | Post => true,
        _ => false
    }
}

/// Perform a CORS check on a header list and CORS request
/// http://fetch.spec.whatwg.org/#cors-check
pub fn allow_cross_origin_request(req: &CORSRequest, headers: &ResponseHeaderCollection) -> bool {
    let allow_cross_origin_request =  headers.iter().find(|h| h.header_name()
                                                               .as_slice()
                                                               .eq_ignore_ascii_case("Access-Control-Allow-Origin"));
    match allow_cross_origin_request {
        Some(h) => {
            let origin_str = h.header_value();
            if origin_str == "*".to_string() {
                return true; // Not always true, depends on credentials mode
            }
            match UrlParser::new().parse(origin_str.as_slice()) {
                Ok(parsed) => parsed.scheme == req.origin.scheme &&
                              parsed.host() == req.origin.host() &&
                              parsed.port() == req.origin.port(),
                _ => false
            }
        },
        None => false
    }
}
