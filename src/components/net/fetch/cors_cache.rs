/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use http::method::Method;
use std::ascii::StrAsciiExt;
use std::comm::{Sender, Receiver, channel};
use time;
use time::{now, Timespec};
use url::Url;

/// Union type for CORS cache entries
///
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
    pub origin: Url,
    pub destination: Url,
    pub credentials: bool
}

/// Trait for a generic CORS Cache
pub trait CORSCache {
    /// [Clear the cache](http://fetch.spec.whatwg.org/#concept-cache-clear)
    fn clear (&mut self, request: CacheRequestDetails);

    /// Remove old entries
    fn cleanup(&mut self);

    /// Returns true if an entry with a [matching header](http://fetch.spec.whatwg.org/#concept-cache-match-header) is found
    fn match_header(&mut self, request: CacheRequestDetails, header_name: &str) -> bool;

    /// Updates max age if an entry for a [matching header](http://fetch.spec.whatwg.org/#concept-cache-match-header) is found.
    ///
    /// If not, it will insert an equivalent entry
    fn match_header_and_update(&mut self, request: CacheRequestDetails, header_name: &str, new_max_age: uint) -> bool;

    /// Returns true if an entry with a [matching method](http://fetch.spec.whatwg.org/#concept-cache-match-method) is found
    fn match_method(&mut self, request: CacheRequestDetails, method: &Method) -> bool;

    /// Updates max age if an entry for [a matching method](http://fetch.spec.whatwg.org/#concept-cache-match-method) is found.
    ///
    /// If not, it will insert an equivalent entry
    fn match_method_and_update(&mut self, request: CacheRequestDetails, method: &Method, new_max_age: uint) -> bool;
    /// Insert an entry
    fn insert(&mut self, entry: CORSCacheEntry);
}

/// A simple, vector-based CORS Cache
#[deriving(Clone)]
pub struct BasicCORSCache(Vec<CORSCacheEntry>);


impl BasicCORSCache {
    fn find_entry_by_header<'a>(&'a mut self, request: &CacheRequestDetails, header_name: &str) -> Option<&'a mut CORSCacheEntry> {
        self.cleanup();
        let BasicCORSCache(ref mut buf) = *self;
        // Credentials are not yet implemented here
        let entry = buf.mut_iter().find(|e| e.origin.scheme == request.origin.scheme &&
                            e.origin.host() == request.origin.host() &&
                            e.origin.port() == request.origin.port() &&
                            e.url == request.destination &&
                            e.credentials == request.credentials &&
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
                            e.credentials == request.credentials &&
                            e.header_or_method.match_method(method));
        entry
    }
}

impl CORSCache for BasicCORSCache {
    /// http://fetch.spec.whatwg.org/#concept-cache-clear
    #[allow(dead_code)]
    fn clear (&mut self, request: CacheRequestDetails) {
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

    fn match_header(&mut self, request: CacheRequestDetails, header_name: &str) -> bool {
        self.find_entry_by_header(&request, header_name).is_some()
    }

    fn match_header_and_update(&mut self, request: CacheRequestDetails, header_name: &str, new_max_age: uint) -> bool {
        match self.find_entry_by_header(&request, header_name).map(|e| e.max_age = new_max_age) {
            Some(_) => true,
            None => {
                self.insert(CORSCacheEntry::new(request.origin, request.destination, new_max_age,
                                                request.credentials, HeaderData(header_name.to_string())));
                false
            }
        }
    }

    fn match_method(&mut self, request: CacheRequestDetails, method: &Method) -> bool {
        self.find_entry_by_method(&request, method).is_some()
    }

    fn match_method_and_update(&mut self, request: CacheRequestDetails, method: &Method, new_max_age: uint) -> bool {
        match self.find_entry_by_method(&request, method).map(|e| e.max_age = new_max_age) {
            Some(_) => true,
            None => {
                self.insert(CORSCacheEntry::new(request.origin, request.destination, new_max_age,
                                                request.credentials, MethodData(method.clone())));
                false
            }
        }
    }

    fn insert(&mut self, entry: CORSCacheEntry) {
        self.cleanup();
        let BasicCORSCache(ref mut buf) = *self;
        buf.push(entry);
    }
}

/// Various messages that can be sent to a CORSCacheTask
pub enum CORSCacheTaskMsg {
    Clear(CacheRequestDetails, Sender<()>),
    Cleanup(Sender<()>),
    MatchHeader(CacheRequestDetails, String, Sender<bool>),
    MatchHeaderUpdate(CacheRequestDetails, String, uint, Sender<bool>),
    MatchMethod(CacheRequestDetails, Method, Sender<bool>),
    MatchMethodUpdate(CacheRequestDetails, Method, uint, Sender<bool>),
    Insert(CORSCacheEntry, Sender<()>)
}

/// A Sender to a CORSCacheTask
///
/// This can be used as a CORS Cache.
/// The methods on this type block until they can run, and it behaves similar to a mutex
pub type CORSCacheSender = Sender<CORSCacheTaskMsg>;

impl CORSCache for CORSCacheSender {
    fn clear (&mut self, request: CacheRequestDetails) {
        let (tx, rx) = channel();
        self.send(Clear(request, tx));
        rx.recv_opt().ok();
    }

    fn cleanup(&mut self) {
        let (tx, rx) = channel();
        self.send(Cleanup(tx));
        rx.recv_opt().ok();
    }

    fn match_header(&mut self, request: CacheRequestDetails, header_name: &str) -> bool {
        let (tx, rx) = channel();
        self.send(MatchHeader(request, header_name.to_string(), tx));
        rx.recv_opt().ok().unwrap_or(false)
    }

    fn match_header_and_update(&mut self, request: CacheRequestDetails, header_name: &str, new_max_age: uint) -> bool {
        let (tx, rx) = channel();
        self.send(MatchHeaderUpdate(request, header_name.to_string(), new_max_age, tx));
        rx.recv_opt().ok().unwrap_or(false)
    }

    fn match_method(&mut self, request: CacheRequestDetails, method: &Method) -> bool {
        let (tx, rx) = channel();
        self.send(MatchMethod(request, method.clone(), tx));
        rx.recv_opt().ok().unwrap_or(false)
    }

    fn match_method_and_update(&mut self, request: CacheRequestDetails, method: &Method, new_max_age: uint) -> bool {
        let (tx, rx) = channel();
        self.send(MatchMethodUpdate(request, method.clone(), new_max_age, tx));
        rx.recv_opt().ok().unwrap_or(false)
    }

    fn insert(&mut self, entry: CORSCacheEntry) {
        let (tx, rx) = channel();
        self.send(Insert(entry, tx));
        rx.recv_opt().ok()  ;
    }        
}

/// A simple task-based CORS Cache that can be sent messages
///
/// #Example
/// ```
/// let task = CORSCacheTask::new();
/// let builder = TaskBuilder::new().named("XHRTask");
/// let mut sender = task.get_sender();
/// builder.spawn(proc() { task.run() });
/// sender.insert(CORSCacheEntry::new(/* parameters here */));
/// ```
pub struct CORSCacheTask {
    receiver: Receiver<CORSCacheTaskMsg>,
    pub cache: BasicCORSCache,
    sender: CORSCacheSender
}

impl CORSCacheTask {
    pub fn new() -> CORSCacheTask {
        let (tx, rx) = channel();
        CORSCacheTask {
            receiver: rx,
            cache: BasicCORSCache(vec![]),
            sender: tx
        }
    }

    /// Provides a sender to the cache task
    pub fn get_sender(&self) -> CORSCacheSender {
        self.sender.clone()
    }

    /// Runs the cache task
    /// This blocks the current task, so it is advised
    /// to spawn a new task for this
    pub fn run(&mut self) {
        loop {
            // The recv() here should never fail, we always
            // carry a copy of the Sender with us.
            match self.receiver.recv() {
                Clear(request, tx) => {
                    self.cache.clear(request);
                    tx.send(());
                },
                Cleanup(tx) => {
                    self.cache.cleanup();
                    tx.send(());
                },
                MatchHeader(request, header, tx) => {
                    tx.send(self.cache.match_header(request, header.as_slice()));
                },
                MatchHeaderUpdate(request, header, new_max_age, tx) => {
                    tx.send(self.cache.match_header_and_update(request, header.as_slice(), new_max_age));
                },
                MatchMethod(request, method, tx) => {
                    tx.send(self.cache.match_method(request, &method));
                },
                MatchMethodUpdate(request, method, new_max_age, tx) => {
                    tx.send(self.cache.match_method_and_update(request, &method, new_max_age));
                },
                Insert(entry, tx) => {
                    self.cache.insert(entry);
                    tx.send(());
                }
            }
        }
    }
}