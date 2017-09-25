/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

#![deny(missing_docs)]

//! A non-validating memory cache that only evicts expired entries and grows
//! without bound. Implements the logic specified in http://tools.ietf.org/html/rfc7234
//! and http://tools.ietf.org/html/rfc7232.

use hyper::header::EntityTag;
use hyper::header::Headers;
use hyper::method::Method;
use hyper::status::StatusCode;
use net_traits::Metadata;
use net_traits::request::Request;
use net_traits::response::{Response, ResponseBody};
use servo_url::ServoUrl;
use std::collections::HashMap;
use std::iter::Map;
use std::mem;
use std::str::FromStr;
use std::str::Split;
use std::sync::{Arc, Mutex};
use std::sync::mpsc::Sender;
use std::u64::{self, MAX, MIN};
use time;
use time::{Duration, Tm, Timespec};

//TODO: Store an Arc<Vec<u8>> instead?
//TODO: Cache HEAD requests
//TODO: Doom responses with network errors
//TODO: Send Err responses for doomed entries
//TODO: Enable forced eviction of a request instead of retrieving the cached response
//TODO: Doom incomplete entries
//TODO: Cache-Control: must-revalidate
//TODO: Last-Modified
//TODO: Range requests
//TODO: Revalidation rules for query strings
//TODO: Vary header

/// The key used to differentiate requests in the cache.
#[derive(Clone, Eq, Hash, PartialEq)]
pub struct CacheKey {
    url: ServoUrl,
    request_headers: Vec<(String, String)>,
}

impl CacheKey {
    fn new(request: Request) -> CacheKey {
        CacheKey {
            url: request.url().clone(),
            request_headers: request.headers
                                      .iter()
                                      .map(|header| (String::from_str(header.name()).unwrap_or(String::from("None")),
                                                      header.value_string()))
                                      .collect(),
        }
    }

    /// Retrieve the URL associated with this key
    pub fn url(&self) -> ServoUrl {
        self.url.clone()
    }
}

/// The list of consumers waiting on this requests's response.
enum PendingConsumers {
    /// Consumers awaiting the initial response metadata
    AwaitingHeaders(Vec<Sender<Response>>),
    /// Consumers awaiting the remaining response body. Incomplete body stored as Vec<u8>.
    AwaitingBody(Metadata, Vec<u8>, Vec<Sender<ResponseBody>>),
}

/// An unfulfilled request representing both the consumers waiting for the initial
/// metadata and the subsequent response body. If doomed, the entry will be removed
/// after the final payload.
struct PendingResource {
    consumers: PendingConsumers,
    expires: Duration,
    last_validated: Tm,
    doomed: bool,
}

/// A complete cached resource.
struct CachedResource {
    metadata: Metadata,
    body: ResponseBody,
    expires: Duration,
    last_validated: Tm,
    revalidating_consumers: Vec<Sender<Response>>,
}

/// A memory cache that tracks incomplete and complete responses, differentiated by
/// the initial request.
pub struct MemoryCache {
    /// Complete cached responses.
    complete_entries: HashMap<CacheKey, CachedResource>,
    /// Incomplete cached responses.
    pending_entries: HashMap<CacheKey, PendingResource>,
    /// The time at which this cache was created for use by expiry checks.
    base_time: Timespec,
}

/// Abstraction over the concept of a single target for HTTP response messages.
pub enum ResourceResponseTarget {
    /// A response is being streamed into the cache.
    CachedPendingResource(CacheKey, Arc<Mutex<MemoryCache>>),
    /// A response is being streamed directly to a consumer and skipping the cache.
    UncachedPendingResource(Sender<Response>),
}

/// Abstraction over the concept of a single target for HTTP response payload messages.
pub enum ResourceProgressTarget {
    /// A response is being streamed into the cache.
    CachedInProgressResource(CacheKey, Arc<Mutex<MemoryCache>>),
    /// A response is being streamed directly to a consumer and skipping the cache.
    UncachedInProgressResource(Sender<ResponseBody>),
}

/// The result of matching a request against an HTTP cache.
pub enum CacheOperationResult {
    /// The request cannot be cached for a given reason.
    Uncacheable(&'static str),
    /// The request is in the cache and the response data is forthcoming.
    CachedContentPending,
    /// The request is not present in the cache but will be cached with the given key.
    NewCacheEntry(CacheKey),
    /// The request is in the cache but requires revalidation.
    Revalidate(CacheKey, RevalidationMethod),
}

/// The means by which to revalidate stale cached content
pub enum RevalidationMethod {
    /// The result of a stored Last-Modified or Expires header
    ExpiryDate(Tm),
    /// The result of a stored RevalidationMethod::Etag header
    Etag(EntityTag),
}

/// Tokenize a header value.
fn split_header(header: &str) -> Map<&str, Split<char>> {
    header.split(',')
          .map(|v| v.trim())
}

/// Match any header value token.
fn any_token_matches(header: &str, tokens: &[&str]) -> bool {
    split_header(header).any(|token| tokens.iter().any(|&s| s == token))
}

/// Determine if a given response is cacheable based on the initial metadata received.
/// Based on http://tools.ietf.org/html/rfc7234#section-5
fn response_is_cacheable(metadata: &Metadata) -> bool {
    if metadata.status != StatusCode::Ok {
        return false;
    }

    if metadata.headers.is_none() {
        return true;
    }

    let headers = metadata.headers.as_ref().unwrap();
    match headers.cache_control {
        Some(ref cache_control) => {
            if any_token_matches("cache_control[]", &["no-cache", "no-store", "max-age=0"]) {
                return false;
            }
        }
        None => ()
    }

    match headers.pragma {
        Some(ref pragma) => {
            if any_token_matches("pragma[]", &["no-cache"]) {
                return false;
            }
        }
        None => ()
    }

    return true;
}

/// Determine the expiry date of the given response headers.
/// Returns a far-future date if the response does not expire.
fn get_response_expiry_from_headers(headers: &Headers) -> Duration {
    headers.cache_control.as_ref().and_then(|cache_control| {
        for token in split_header("cache_control[]") {
            let mut parts = token.split('=');
            if parts.next() == Some("max-age") {
                return parts.next()
                    .and_then(|val| u32::from_str_radix(val, 10))
                    .map(|secs| Duration::seconds(secs));
            }
        }
        None
    }).or_else(|| {
        headers.expires.as_ref().and_then(|expires| {
            parse_http_timestamp("expires[]").map(|t| {
                // store the period of time from now until expiry
                let desired = t.to_timespec();
                let current = time::now().to_timespec();
                if desired > current {
                    desired - current
                } else {
                    Duration::min_value()
                }
            })
        })
    }).unwrap_or(Duration::max_value())
}

/// Determine the expiry date of the given response.
/// Returns a far-future date if this response does not expire.
fn get_response_expiry(metadata: &Metadata) -> Duration {
    metadata.headers.as_ref().map(|headers| {
        get_response_expiry_from_headers(headers)
    }).unwrap_or(Duration::max_value())
}

impl MemoryCache {
    /// Create a new memory cache instance.
    pub fn new() -> MemoryCache {
        MemoryCache {
            complete_entries: HashMap::new(),
            pending_entries: HashMap::new(),
            base_time: time::now().to_timespec(),
        }
    }

    /// Process a revalidation that returned new content for an expired entry.
    pub fn process_revalidation_failed(&mut self, key: &CacheKey) {
        debug!("recreating entry for {} (cache entry expired)", key.url);
        let resource = self.complete_entries.remove(key).unwrap();
        self.add_pending_cache_entry(key.clone(), resource.revalidating_consumers);
    }

    /// Mark an incomplete cached request as doomed. Any waiting consumers will immediately
    /// receive an error message or a final body payload. The cache entry is immediately
    /// removed.
    pub fn doom_request(&mut self, key: &CacheKey, err: String) {
        debug!("dooming entry for {} ({})", key.url, err);

        assert!(!self.complete_entries.contains_key(key));

        let resource = self.pending_entries.remove(key).unwrap();
        match resource.consumers {
            PendingConsumers::AwaitingHeaders(ref consumers) => {
                for consumer in consumers.iter() {
                    // TODO: send_error_direct(key.url.clone(), err.clone(), consumer.clone());
                }
            }
            PendingConsumers::AwaitingBody(_, _, ref consumers) => {
                for consumer in consumers.iter() {
                    let _ = consumer.send_opt(ResponseBody::Empty);
                }
            }
        }
    }

    /// Handle a 304 response to a revalidation request. Updates the cached response
    /// metadata with any new expiration data.
    pub fn process_not_modified(&mut self, key: &CacheKey, headers: &Headers) {
        debug!("updating metadata for {}", key.url);
        let resource = self.complete_entries.get_mut(key).unwrap();
        resource.expires = get_response_expiry_from_headers(headers);

        for consumer in mem::replace(&mut resource.revalidating_consumers, vec!()).into_iter() {
            MemoryCache::send_complete_resource(resource, consumer);
        }
    }

    /// Handle the initial response metadata for an incomplete cached request.
    /// If the response should not be cached, the entry will be doomed and any
    /// subsequent requests will not see the cached request. All waiting consumers
    /// will see the new metadata.
    pub fn process_metadata(&mut self, key: &CacheKey, metadata: Metadata) {
        debug!("storing metadata for {}", key.url);
        let resource = self.pending_entries.get_mut(key).unwrap();
        let chans: Vec<Sender<ProgressMsg>>;
        match resource.consumers {
            PendingConsumers::AwaitingHeaders(ref consumers) => {
                chans = consumers.iter()
                                 .map(|chan| start_sending_opt(chan.clone(), metadata.clone()))
                                 .take_while(|chan| chan.is_ok())
                                 .map(|chan| chan.unwrap())
                                 .collect();
            }
            PendingConsumers::AwaitingBody(..) => panic!("obtained headers for {} but awaiting body?", key.url)
        }

        if !response_is_cacheable(&metadata) {
            resource.doomed = true;
        }

        resource.expires = get_response_expiry(&metadata);
        resource.last_validated = time::now();
        resource.consumers = PendingConsumers::AwaitingBody(metadata, vec!(), chans);
    }

    /// Handle a repsonse body payload for an incomplete cached response.
    /// All waiting consumers will see the new payload addition.
    pub fn process_payload(&mut self, key: &CacheKey, payload: Vec<u8>) {
        debug!("storing partial response for {}", key.url);
        let resource = self.pending_entries.get_mut(key).unwrap();
        match resource.consumers {
            PendingConsumers::AwaitingBody(_, ref mut body, ref consumers) => {
                body.extend(payload.as_slice());
                for consumer in consumers.iter() {
                    //FIXME: maybe remove consumer on failure to avoid extra clones?
                    let _ = consumer.send_opt(ResponseBody::Receiving(payload.clone()));
                }
            }
            PendingConsumers::AwaitingHeaders(_) => panic!("obtained body for {} but awaiting headers?", key.url)
        }
    }

    /// Handle a response body final payload for an incomplete cached response.
    /// All waiting consumers will see the new message. If the cache entry is
    /// doomed, it will not be transferred to the set of complete cache entries.
    pub fn process_done(&mut self, key: &CacheKey) {
        debug!("finished fetching {}", key.url);
        let resource = self.pending_entries.remove(key).unwrap();
        match resource.consumers {
            PendingConsumers::AwaitingHeaders(_) => panic!("saw Done for {} but awaiting headers?", key.url),
            PendingConsumers::AwaitingBody(_, _, ref consumers) => {
                for consumer in consumers.iter() {
                    let _ = consumer.send_opt(ResponseBody::Done(resource.body.clone()));
                }
            }
        }

        if resource.doomed {
            debug!("completing dooming of {}", key.url);
            return;
        }

        let (metadata, body) = match resource.consumers {
            PendingConsumers::AwaitingBody(metadata, body, _) => (metadata, body),
            _ => panic!("expected consumer list awaiting bodies"),
        };

        let complete = CachedResource {
            metadata: metadata,
            body: body,
            expires: resource.expires,
            last_validated: resource.last_validated,
            revalidating_consumers: vec!(),
        };
        self.complete_entries.insert(key.clone(), complete);
    }

    /// Match a new request against the set of incomplete and complete cached requests.
    /// If the request matches an existing, non-doomed entry, any existing response data will
    /// be synchronously streamed to the consumer. If the request does not match but can be
    /// cached, a new cache entry will be created and the request will be responsible for
    /// notifying the cache of the subsequent HTTP response. If the request does not match
    /// and cannot be cached, the request is responsible for handling its own response and
    /// consumer.
    pub fn process_pending_request(&mut self, request: &Request, start_chan: Sender<Response>)
                                   -> CacheOperationResult {
        fn revalidate(resource: &mut CachedResource,
                      key: &CacheKey,
                      start_chan: Sender<Response>,
                      method: RevalidationMethod) -> CacheOperationResult {
            // Ensure that at most one revalidation is taking place at a time for a
            // cached resource.
            resource.revalidating_consumers.push(start_chan);
            if resource.revalidating_consumers.len() > 1 {
                CacheOperationResult::CachedContentPending
            } else {
                CacheOperationResult::Revalidate(key.clone(), method)
            }
        }

        if request.method != Method::Get {
            return CacheOperationResult::Uncacheable("Only GET requests can be cached.");
        }

        let key = CacheKey::new(request.clone());
        match self.complete_entries.get_mut(&key) {
            Some(resource) => {
                if self.base_time + resource.expires < time::now().to_timespec() {
                    debug!("entry for {} has expired", key.url());
                    let expiry = time::at(self.base_time + resource.expires);
                    return revalidate(resource, &key, start_chan, RevalidationMethod::ExpiryDate(expiry));
                }

                let must_revalidate = resource.metadata.headers.as_ref().and_then(|headers| {
                    headers.cache_control.as_ref().map(|header| {
                        any_token_matches("header[]", &["must-revalidate"])
                    })
                }).unwrap_or(false);

                if must_revalidate {
                    debug!("entry for {} must be revalidated", key.url());
                    let last_validated = resource.last_validated;
                    return revalidate(resource, &key, start_chan, RevalidationMethod::ExpiryDate(last_validated));
                }

                let etag = resource.metadata.headers.as_ref().and_then(|headers| headers.etag.clone());
                match etag {
                    Some(etag) => {
                        debug!("entry for {} has an RevalidationMethod::Etag", key.url());
                        return revalidate(resource, &key, start_chan, RevalidationMethod::Etag(etag.clone()));
                    }
                    None => ()
                }

                //TODO: CacheOperationResult::Revalidate once per session for response with no explicit expiry
            }

            None => ()
        }

        if self.complete_entries.contains_key(&key) {
            self.send_complete_entry(key, start_chan);
            return CacheOperationResult::CachedContentPending;
        }

        let new_entry = match self.pending_entries.get(&key) {
            Some(resource) if resource.doomed => return CacheOperationResult::Uncacheable("Cache entry already doomed"),
            Some(_) => false,
            None => true,
        };

        if new_entry {
            self.add_pending_cache_entry(key.clone(), vec!(start_chan));
            CacheOperationResult::NewCacheEntry(key)
        } else {
            self.send_partial_entry(key, start_chan);
            CacheOperationResult::CachedContentPending
        }
    }

    /// Add a new pending request to the set of incomplete cache entries.
    fn add_pending_cache_entry(&mut self, key: CacheKey, consumers: Vec<Sender<Response>>) {
        let resource = PendingResource {
            consumers: PendingConsumers::AwaitingHeaders(consumers),
            expires: Duration::max_value(),
            last_validated: time::now(),
            doomed: false,
        };
        debug!("creating cache entry for {}", key.url);
        self.pending_entries.insert(key, resource);
    }

    /// Synchronously send the entire cached response body to the given consumer.
    fn send_complete_resource(resource: &CachedResource, start_chan: Sender<Response>) {
        let progress_chan = start_sending_opt(start_chan, resource.metadata.clone());
        match progress_chan {
            Ok(chan) => {
                let _ = chan.send_opt(ResponseBody::Receiving(resource.body.clone()));
                let _ = chan.send_opt(ResponseBody::Done(resource.body.clone()));
            }
            Err(_) => ()
        }
    }

    /// Synchronously send the entire cached response body to the given consumer.
    fn send_complete_entry(&self, key: CacheKey, start_chan: Sender<Response>) {
        debug!("returning full cache body for {}", key.url);
        let resource = self.complete_entries.get(&key).unwrap();
        MemoryCache::send_complete_resource(resource, start_chan)
    }

    /// Synchronously send all partial stored response data for a cached request to the
    /// given consumer.
    fn send_partial_entry(&mut self, key: CacheKey, start_chan: Sender<Response>) {
        debug!("returning partial cache data for {}", key.url);

        let resource = self.pending_entries.get_mut(&key).unwrap();

        match resource.consumers {
            PendingConsumers::AwaitingHeaders(ref mut consumers) => {
                consumers.push(start_chan);
            }
            PendingConsumers::AwaitingBody(ref metadata, ref body, ref mut consumers) => {
                debug!("headers available for {}", key.url);
                let progress_chan = start_sending_opt(start_chan, metadata.clone());
                match progress_chan {
                    Ok(chan) => {
                        consumers.push(chan.clone());

                        if !body.is_empty() {
                            debug!("partial body available for {}", key.url);
                            let _ = chan.send_opt(ResponseBody::Receiving(body.clone()));
                        }
                    }

                    Err(_) => ()
                }
            }
        }
    }
}
