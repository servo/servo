/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use http::method::Method;
use std::ascii::StrAsciiExt;
use time;
use time::{now, Timespec};
use url::Url;

#[deriving(Clone)]
pub struct BasicCORSCache(Vec<CORSCacheEntry>);

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

 /// An entry in the CORS cache
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

/// Properties of Request required to cache match.
pub struct CacheRequestDetails {
    origin: Url,
    destination: Url,
}

trait CORSCache {
    /// [Clear the cache](http://fetch.spec.whatwg.org/#concept-cache-clear)
    fn clear (&mut self, request: &CacheRequestDetails);

    /// Remove old entries
    fn cleanup(&mut self);

    /// [Finds an entry with a matching header](http://fetch.spec.whatwg.org/#concept-cache-match-header)
    fn find_entry_by_header<'a>(&'a mut self, request: &CacheRequestDetails, header_name: &str) -> Option<&'a mut CORSCacheEntry>;

    /// Returns true if an entry with a matching header is found
    fn match_header(&mut self, request: &CacheRequestDetails, header_name: &str) -> bool {
        self.find_entry_by_header(request, header_name).is_some()
    }

    /// Updates max age if an entry for the same header is found.
    fn match_header_and_update(&mut self, request: &CacheRequestDetails, header_name: &str, new_max_age: uint) -> bool {
        self.find_entry_by_header(request, header_name).map(|e| e.max_age = new_max_age).is_some()
    }

    /// [Finds an entry with a matching method](http://fetch.spec.whatwg.org/#concept-cache-match-method)
    fn find_entry_by_method<'a>(&'a mut self, request: &CacheRequestDetails, method: &Method) -> Option<&'a mut CORSCacheEntry>;

    /// Returns true if an entry with a matching method is found
    fn match_method(&mut self, request: &CacheRequestDetails, method: &Method) -> bool {
        self.find_entry_by_method(request, method).is_some()
    }

    /// Updates max age if an entry for the same method is found.
    fn match_method_and_update(&mut self, request: &CacheRequestDetails, method: &Method, new_max_age: uint) -> bool {
        self.find_entry_by_method(request, method).map(|e| e.max_age = new_max_age).is_some()
    }

    /// Insert an entry
    fn insert(&mut self, entry: CORSCacheEntry);
}

impl CORSCache for BasicCORSCache {
    /// http://fetch.spec.whatwg.org/#concept-cache-clear
    #[allow(dead_code)]
    fn clear (&mut self, request: &CacheRequestDetails) {
        let BasicCORSCache(buf) = self.clone();
        let new_buf: Vec<CORSCacheEntry> = buf.move_iter().filter(|e| e.origin == request.origin && request.destination == e.url).collect();
        *self = BasicCORSCache(new_buf);
    }

    // Remove old entries
    fn cleanup(&mut self) {
        let BasicCORSCache(buf) = self.clone();
        let now = time::now().to_timespec();
        let new_buf: Vec<CORSCacheEntry> = buf.move_iter().filter(|e| now.sec > e.created.sec + e.max_age as i64).collect();
        *self = BasicCORSCache(new_buf);
    }

    /// http://fetch.spec.whatwg.org/#concept-cache-match-header
    fn find_entry_by_header<'a>(&'a mut self, request: &CacheRequestDetails, header_name: &str) -> Option<&'a mut CORSCacheEntry> {
        self.cleanup();
        let BasicCORSCache(ref mut buf) = *self;
        // Credentials are not yet implemented here
        let entry = buf.mut_iter().find(|e| e.origin.scheme == request.origin.scheme &&
                            e.origin.host() == request.origin.host() &&
                            e.origin.port() == request.origin.port() &&
                            e.url == request.destination &&
                            e.header_or_method.match_header(header_name));
        entry
    }

    fn find_entry_by_method<'a>(&'a mut self, request: &CacheRequestDetails, method: &Method) -> Option<&'a mut CORSCacheEntry> {
        // we can take the method from CORSRequest itself
        self.cleanup();
        let BasicCORSCache(ref mut buf) = *self;
        // Credentials are not yet implemented here
        let entry = buf.mut_iter().find(|e| e.origin.scheme == request.origin.scheme &&
                            e.origin.host() == request.origin.host() &&
                            e.origin.port() == request.origin.port() &&
                            e.url == request.destination &&
                            e.header_or_method.match_method(method));
        entry
    }

    fn insert(&mut self, entry: CORSCacheEntry) {
        self.cleanup();
        let BasicCORSCache(ref mut buf) = *self;
        buf.push(entry);
    }
}
