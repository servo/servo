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
use net_traits::request::Origin;
use std::ascii::AsciiExt;
use std::sync::mpsc::{Sender, Receiver, channel};
use time;
use time::{now, Timespec};
use url::Url;

/// Union type for CORS cache entries
///
/// Each entry might pertain to a header or method
#[derive(Clone)]
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
#[derive(Clone)]
pub struct CORSCacheEntry {
    pub origin: Origin,
    pub url: Url,
    pub max_age: u32,
    pub credentials: bool,
    pub header_or_method: HeaderOrMethod,
    created: Timespec
}

impl CORSCacheEntry {
    fn new(origin: Origin, url: Url, max_age: u32, credentials: bool,
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

/// Properties of Request required to cache match.
pub struct CacheRequestDetails {
    pub origin: Origin,
    pub destination: Url,
    pub credentials: bool
}

/// Trait for a generic CORS Cache
pub trait CORSCache {
    /// [Clear the cache](https://fetch.spec.whatwg.org/#concept-cache-clear)
    fn clear (&mut self, request: CacheRequestDetails);

    /// Remove old entries
    fn cleanup(&mut self);

    /// Returns true if an entry with a
    /// [matching header](https://fetch.spec.whatwg.org/#concept-cache-match-header) is found
    fn match_header(&mut self, request: CacheRequestDetails, header_name: &str) -> bool;

    /// Updates max age if an entry for a
    /// [matching header](https://fetch.spec.whatwg.org/#concept-cache-match-header) is found.
    ///
    /// If not, it will insert an equivalent entry
    fn match_header_and_update(&mut self, request: CacheRequestDetails, header_name: &str, new_max_age: u32) -> bool;

    /// Returns true if an entry with a
    /// [matching method](https://fetch.spec.whatwg.org/#concept-cache-match-method) is found
    fn match_method(&mut self, request: CacheRequestDetails, method: Method) -> bool;

    /// Updates max age if an entry for
    /// [a matching method](https://fetch.spec.whatwg.org/#concept-cache-match-method) is found.
    ///
    /// If not, it will insert an equivalent entry
    fn match_method_and_update(&mut self, request: CacheRequestDetails, method: Method, new_max_age: u32) -> bool;
    /// Insert an entry
    fn insert(&mut self, entry: CORSCacheEntry);
}

/// A simple, vector-based CORS Cache
#[derive(Clone)]
pub struct BasicCORSCache(Vec<CORSCacheEntry>);

fn match_headers(cors_cache: &CORSCacheEntry, cors_req: &CacheRequestDetails) -> bool {
    cors_cache.origin == cors_req.origin &&
        cors_cache.url == cors_req.destination &&
        cors_cache.credentials == cors_req.credentials
}

impl BasicCORSCache {

    pub fn new() -> BasicCORSCache {
        BasicCORSCache(vec![])
    }

    fn find_entry_by_header<'a>(&'a mut self, request: &CacheRequestDetails,
                                header_name: &str) -> Option<&'a mut CORSCacheEntry> {
        self.cleanup();
        self.0.iter_mut().find(|e| match_headers(e, request) && e.header_or_method.match_header(header_name))
    }

    fn find_entry_by_method<'a>(&'a mut self, request: &CacheRequestDetails,
                                method: Method) -> Option<&'a mut CORSCacheEntry> {
        // we can take the method from CORSRequest itself
        self.cleanup();
        self.0.iter_mut().find(|e| match_headers(e, request) && e.header_or_method.match_method(&method))
    }
}

impl CORSCache for BasicCORSCache {
    /// https://fetch.spec.whatwg.org/#concept-cache-clear
    #[allow(dead_code)]
    fn clear (&mut self, request: CacheRequestDetails) {
        let BasicCORSCache(buf) = self.clone();
        let new_buf: Vec<CORSCacheEntry> =
            buf.into_iter().filter(|e| e.origin == request.origin && request.destination == e.url).collect();
        *self = BasicCORSCache(new_buf);
    }

    // Remove old entries
    fn cleanup(&mut self) {
        let BasicCORSCache(buf) = self.clone();
        let now = time::now().to_timespec();
        let new_buf: Vec<CORSCacheEntry> = buf.into_iter()
                                              .filter(|e| now.sec > e.created.sec + e.max_age as i64)
                                              .collect();
        *self = BasicCORSCache(new_buf);
    }

    fn match_header(&mut self, request: CacheRequestDetails, header_name: &str) -> bool {
        self.find_entry_by_header(&request, header_name).is_some()
    }

    fn match_header_and_update(&mut self, request: CacheRequestDetails, header_name: &str, new_max_age: u32) -> bool {
        match self.find_entry_by_header(&request, header_name).map(|e| e.max_age = new_max_age) {
            Some(_) => true,
            None => {
                self.insert(CORSCacheEntry::new(request.origin, request.destination, new_max_age,
                                                request.credentials,
                                                HeaderOrMethod::HeaderData(header_name.to_owned())));
                false
            }
        }
    }

    fn match_method(&mut self, request: CacheRequestDetails, method: Method) -> bool {
        self.find_entry_by_method(&request, method).is_some()
    }

    fn match_method_and_update(&mut self, request: CacheRequestDetails, method: Method, new_max_age: u32) -> bool {
        match self.find_entry_by_method(&request, method.clone()).map(|e| e.max_age = new_max_age) {
            Some(_) => true,
            None => {
                self.insert(CORSCacheEntry::new(request.origin, request.destination, new_max_age,
                                                request.credentials, HeaderOrMethod::MethodData(method)));
                false
            }
        }
    }

    fn insert(&mut self, entry: CORSCacheEntry) {
        self.cleanup();
        self.0.push(entry);
    }
}

/// Various messages that can be sent to a CORSCacheThread
pub enum CORSCacheThreadMsg {
    Clear(CacheRequestDetails, Sender<()>),
    Cleanup(Sender<()>),
    MatchHeader(CacheRequestDetails, String, Sender<bool>),
    MatchHeaderUpdate(CacheRequestDetails, String, u32, Sender<bool>),
    MatchMethod(CacheRequestDetails, Method, Sender<bool>),
    MatchMethodUpdate(CacheRequestDetails, Method, u32, Sender<bool>),
    Insert(CORSCacheEntry, Sender<()>),
    ExitMsg
}

/// A Sender to a CORSCacheThread
///
/// This can be used as a CORS Cache.
/// The methods on this type block until they can run, and it behaves similar to a mutex
pub type CORSCacheSender = Sender<CORSCacheThreadMsg>;

impl CORSCache for CORSCacheSender {
    fn clear (&mut self, request: CacheRequestDetails) {
        let (tx, rx) = channel();
        let _ = self.send(CORSCacheThreadMsg::Clear(request, tx));
        let _ = rx.recv();
    }

    fn cleanup(&mut self) {
        let (tx, rx) = channel();
        let _ = self.send(CORSCacheThreadMsg::Cleanup(tx));
        let _ = rx.recv();
    }

    fn match_header(&mut self, request: CacheRequestDetails, header_name: &str) -> bool {
        let (tx, rx) = channel();
        let _ = self.send(CORSCacheThreadMsg::MatchHeader(request, header_name.to_owned(), tx));
        rx.recv().unwrap_or(false)
    }

    fn match_header_and_update(&mut self, request: CacheRequestDetails, header_name: &str, new_max_age: u32) -> bool {
        let (tx, rx) = channel();
        let _ = self.send(CORSCacheThreadMsg::MatchHeaderUpdate(request, header_name.to_owned(), new_max_age, tx));
        rx.recv().unwrap_or(false)
    }

    fn match_method(&mut self, request: CacheRequestDetails, method: Method) -> bool {
        let (tx, rx) = channel();
        let _ = self.send(CORSCacheThreadMsg::MatchMethod(request, method, tx));
        rx.recv().unwrap_or(false)
    }

    fn match_method_and_update(&mut self, request: CacheRequestDetails, method: Method, new_max_age: u32) -> bool {
        let (tx, rx) = channel();
        let _ = self.send(CORSCacheThreadMsg::MatchMethodUpdate(request, method, new_max_age, tx));
        rx.recv().unwrap_or(false)
    }

    fn insert(&mut self, entry: CORSCacheEntry) {
        let (tx, rx) = channel();
        let _ = self.send(CORSCacheThreadMsg::Insert(entry, tx));
        let _ = rx.recv();
    }
}

/// A simple thread-based CORS Cache that can be sent messages
///
/// #Example
/// ```ignore
/// let thread = CORSCacheThread::new();
/// let builder = ThreadBuilder::new().named("XHRThread");
/// let mut sender = thread.sender();
/// builder.spawn(move || { thread.run() });
/// sender.insert(CORSCacheEntry::new(/* parameters here */));
/// ```
pub struct CORSCacheThread {
    receiver: Receiver<CORSCacheThreadMsg>,
    cache: BasicCORSCache,
    sender: CORSCacheSender
}

impl CORSCacheThread {
    pub fn new() -> CORSCacheThread {
        let (tx, rx) = channel();
        CORSCacheThread {
            receiver: rx,
            cache: BasicCORSCache(vec![]),
            sender: tx
        }
    }

    /// Provides a sender to the cache thread
    pub fn sender(&self) -> CORSCacheSender {
        self.sender.clone()
    }

    /// Runs the cache thread
    /// This blocks the current thread, so it is advised
    /// to spawn a new thread for this
    /// Send ExitMsg to the associated Sender to exit
    pub fn run(&mut self) {
        loop {
            match self.receiver.recv().unwrap() {
                CORSCacheThreadMsg::Clear(request, tx) => {
                    self.cache.clear(request);
                    let _ = tx.send(());
                },
                CORSCacheThreadMsg::Cleanup(tx) => {
                    self.cache.cleanup();
                    let _ = tx.send(());
                },
                CORSCacheThreadMsg::MatchHeader(request, header, tx) => {
                    let _ = tx.send(self.cache.match_header(request, &header));
                },
                CORSCacheThreadMsg::MatchHeaderUpdate(request, header, new_max_age, tx) => {
                    let _ = tx.send(self.cache.match_header_and_update(request, &header, new_max_age));
                },
                CORSCacheThreadMsg::MatchMethod(request, method, tx) => {
                    let _ = tx.send(self.cache.match_method(request, method));
                },
                CORSCacheThreadMsg::MatchMethodUpdate(request, method, new_max_age, tx) => {
                    let _ = tx.send(self.cache.match_method_and_update(request, method, new_max_age));
                },
                CORSCacheThreadMsg::Insert(entry, tx) => {
                    self.cache.insert(entry);
                    let _ = tx.send(());
                },
                CORSCacheThreadMsg::ExitMsg => break
            }
        }
    }
}
