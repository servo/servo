/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

//! An implementation of the [CORS preflight cache](https://fetch.spec.whatwg.org/#cors-preflight-cache)
//! For now this library is XHR-specific.
//! For stuff involving `<img>`, `<iframe>`, `<form>`, etc please check what
//! the request mode should be and compare with the fetch spec
//! This library will eventually become the core of the Fetch crate
//! with CORSRequest being expanded into FetchRequest (etc)

use hyper::method::Method;
use net_traits::request::{CredentialsMode, Origin, Request};
use servo_url::ServoUrl;
use std::ascii::AsciiExt;
use time::{self, Timespec};

/// Union type for CORS cache entries
///
/// Each entry might pertain to a header or method
#[derive(Clone, Debug)]
pub enum HeaderOrMethod {
    HeaderData(String),
    MethodData(Method)
}

impl HeaderOrMethod {
    fn match_header(&self, header_name: &str) -> bool {
        match *self {
            HeaderOrMethod::HeaderData(ref s) => (&**s).eq_ignore_ascii_case(header_name),
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

/// An entry in the CORS cache
#[derive(Clone, Debug)]
pub struct CorsCacheEntry {
    pub origin: Origin,
    pub url: ServoUrl,
    pub max_age: u32,
    pub credentials: bool,
    pub header_or_method: HeaderOrMethod,
    created: Timespec
}

impl CorsCacheEntry {
    fn new(origin: Origin, url: ServoUrl, max_age: u32, credentials: bool,
            header_or_method: HeaderOrMethod) -> CorsCacheEntry {
        CorsCacheEntry {
            origin: origin,
            url: url,
            max_age: max_age,
            credentials: credentials,
            header_or_method: header_or_method,
            created: time::now().to_timespec()
        }
    }
}

fn match_headers(cors_cache: &CorsCacheEntry, cors_req: &Request) -> bool {
    cors_cache.origin == *cors_req.origin.borrow() &&
        cors_cache.url == cors_req.current_url() &&
        (cors_cache.credentials || cors_req.credentials_mode != CredentialsMode::Include)
}

/// A simple, vector-based CORS Cache
#[derive(Clone)]
pub struct CorsCache(Vec<CorsCacheEntry>);

impl CorsCache {
    pub fn new() -> CorsCache {
        CorsCache(vec![])
    }

    fn find_entry_by_header<'a>(&'a mut self, request: &Request,
                                header_name: &str) -> Option<&'a mut CorsCacheEntry> {
        self.cleanup();
        self.0.iter_mut().find(|e| match_headers(e, request) && e.header_or_method.match_header(header_name))
    }

    fn find_entry_by_method<'a>(&'a mut self, request: &Request,
                                method: Method) -> Option<&'a mut CorsCacheEntry> {
        // we can take the method from CorSRequest itself
        self.cleanup();
        self.0.iter_mut().find(|e| match_headers(e, request) && e.header_or_method.match_method(&method))
    }

    /// [Clear the cache](https://fetch.spec.whatwg.org/#concept-cache-clear)
    pub fn clear(&mut self, request: &Request) {
        let CorsCache(buf) = self.clone();
        let new_buf: Vec<CorsCacheEntry> =
            buf.into_iter().filter(|e| e.origin == *request.origin.borrow() &&
                                       request.current_url() == e.url).collect();
        *self = CorsCache(new_buf);
    }

    /// Remove old entries
    pub fn cleanup(&mut self) {
        let CorsCache(buf) = self.clone();
        let now = time::now().to_timespec();
        let new_buf: Vec<CorsCacheEntry> = buf.into_iter()
                                              .filter(|e| now.sec < e.created.sec + e.max_age as i64)
                                              .collect();
        *self = CorsCache(new_buf);
    }

    /// Returns true if an entry with a
    /// [matching header](https://fetch.spec.whatwg.org/#concept-cache-match-header) is found
    pub fn match_header(&mut self, request: &Request, header_name: &str) -> bool {
        self.find_entry_by_header(&request, header_name).is_some()
    }

    /// Updates max age if an entry for a
    /// [matching header](https://fetch.spec.whatwg.org/#concept-cache-match-header) is found.
    ///
    /// If not, it will insert an equivalent entry
    pub fn match_header_and_update(&mut self, request: &Request,
                                   header_name: &str, new_max_age: u32) -> bool {
        match self.find_entry_by_header(&request, header_name).map(|e| e.max_age = new_max_age) {
            Some(_) => true,
            None => {
                self.insert(CorsCacheEntry::new(request.origin.borrow().clone(), request.current_url(), new_max_age,
                                                request.credentials_mode == CredentialsMode::Include,
                                                HeaderOrMethod::HeaderData(header_name.to_owned())));
                false
            }
        }
    }

    /// Returns true if an entry with a
    /// [matching method](https://fetch.spec.whatwg.org/#concept-cache-match-method) is found
    pub fn match_method(&mut self, request: &Request, method: Method) -> bool {
        self.find_entry_by_method(&request, method).is_some()
    }

    /// Updates max age if an entry for
    /// [a matching method](https://fetch.spec.whatwg.org/#concept-cache-match-method) is found.
    ///
    /// If not, it will insert an equivalent entry
    pub fn match_method_and_update(&mut self, request: &Request, method: Method, new_max_age: u32) -> bool {
        match self.find_entry_by_method(&request, method.clone()).map(|e| e.max_age = new_max_age) {
            Some(_) => true,
            None => {
                self.insert(CorsCacheEntry::new(request.origin.borrow().clone(), request.current_url(), new_max_age,
                                                request.credentials_mode == CredentialsMode::Include,
                                                HeaderOrMethod::MethodData(method)));
                false
            }
        }
    }

    /// Insert an entry
    pub fn insert(&mut self, entry: CorsCacheEntry) {
        self.cleanup();
        self.0.push(entry);
    }
}
