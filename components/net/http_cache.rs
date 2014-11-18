/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

#![deny(missing_docs)]

//! A non-validating memory cache that only evicts expired entries and grows
//! without bound.

use http_loader::send_error_direct;
use resource_task::{Metadata, ProgressMsg, LoadResponse, LoadData, Payload, Done, start_sending_opt};

use servo_util::time::parse_http_timestamp;

use http::headers::etag::EntityTag;
use http::headers::HeaderEnum;
use http::headers::response::HeaderCollection as ResponseHeaderCollection;
use http::method::Get;
use http::status::Ok as StatusOk;

use std::collections::HashMap;
use std::comm::Sender;
use std::iter::Map;
use std::num::{Bounded, FromStrRadix};
use std::str::CharSplits;
use std::sync::{Arc, Mutex};
use std::time::duration::{MAX, Duration};
use time;
use time::{Tm, Timespec};
use url::Url;

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
//TODO: Vary

/// The key used to differentiate requests in the cache.
#[deriving(Clone, Hash, PartialEq, Eq)]
pub struct CacheKey {
    url: Url,
    request_headers: Vec<(String, String)>,
}

impl CacheKey {
    fn new(load_data: LoadData) -> CacheKey {
        CacheKey {
            url: load_data.url.clone(),
            request_headers: load_data.headers
                                      .iter()
                                      .map(|header| (header.header_name(), header.header_value()))
                                      .collect(),
        }
    }

    /// Retrieve the URL associated with this key
    pub fn url(&self) -> Url {
        self.url.clone()
    }
}

/// The list of consumers waiting on this requests's response.
enum PendingConsumers {
    /// Consumers awaiting the initial response metadata
    AwaitingHeaders(Vec<Sender<LoadResponse>>),
    /// Consumers awaiting the remaining response body
    AwaitingBody(Metadata, Vec<u8>, Vec<Sender<ProgressMsg>>),
}

/// An unfulfilled request representing both the consumers waiting for the initial
/// metadata and the subsequent response body. If doomed, the entry will be removed
/// after the final payload.
struct PendingResource {
    consumers: PendingConsumers,
    expires: Duration,
    doomed: bool,
    last_validated: Tm,
}

/// A complete cached resource.
struct CachedResource {
    metadata: Metadata,
    body: Vec<u8>,
    expires: Duration,
    last_validated: Tm,
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
    UncachedPendingResource(Sender<LoadResponse>),
}

/// Abstraction over the concept of a single target for HTTP response payload messages.
pub enum ResourceProgressTarget {
    /// A response is being streamed into the cache.
    CachedInProgressResource(CacheKey, Arc<Mutex<MemoryCache>>),
    /// A response is being streamed directly to a consumer and skipping the cache.
    UncachedInProgressResource(Sender<ProgressMsg>),
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
    /// The result of a stored Etag header
    Etag(EntityTag),
}

/// Tokenize a header value.
fn split_header(header: &str) -> Map<&str, &str, CharSplits<char>> {
    header.split(',')
          .map(|v| v.trim())
}

/// Match any header value token.
fn any_token_matches(header: &str, tokens: &[&str]) -> bool {
    split_header(header).any(|token| tokens.iter().any(|&s| s == token))
}

/// Determine if a given response is cacheable based on the initial metadata received.
fn response_is_cacheable(metadata: &Metadata) -> bool {
    if metadata.status != StatusOk {
        return false;
    }

    if metadata.headers.is_none() {
        return true;
    }

    let headers = metadata.headers.as_ref().unwrap();
    match headers.cache_control {
        Some(ref cache_control) => {
            if any_token_matches(cache_control[], &["no-cache", "no-store", "max-age=0"]) {
                return false;
            }
        }
        None => ()
    }

    match headers.pragma {
        Some(ref pragma) => {
            if any_token_matches(pragma[], &["no-cache"]) {
                return false;
            }
        }
        None => ()
    }

    return true;
}

/// Determine the expiry date of the given response headers.
fn get_response_expiry_from_headers(headers: &ResponseHeaderCollection) -> Duration {
    headers.cache_control.as_ref().and_then(|cache_control| {
        for token in split_header(cache_control[]) {
            let mut parts = token.split('=');
            if parts.next().unwrap() == "max-age" {
                return parts.next()
                    .and_then(|val| FromStrRadix::from_str_radix(val, 10))
                    .map(|secs| Duration::seconds(secs));
            }
        }
        None
    }).or_else(|| {
        headers.expires.as_ref().and_then(|expires| {
            parse_http_timestamp(expires[]).map(|t| {
                // store the period of time from now until expiry
                let desired = t.to_timespec();
                let current = time::now().to_timespec();
                if desired > current {
                    desired - current
                } else {
                    Bounded::min_value()
                }
            })
        })
    }).unwrap_or(Bounded::max_value())
}

/// Determine the expiry date of the given response.
fn get_response_expiry(metadata: &Metadata) -> Duration {
    metadata.headers.as_ref().map(|headers| {
        get_response_expiry_from_headers(headers)
    }).unwrap_or(Bounded::max_value())
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

    /// Mark a cached request as doomed. Any waiting consumers will immediately receive
    /// an error message or a final body payload. The cache entry is immediately removed.
    pub fn doom_request(&mut self, key: &CacheKey, err: String) {
        info!("dooming entry for {}", key.url);
        match self.complete_entries.remove(key) {
            Some(_) => return,
            None => (),
        }

        let resource = self.pending_entries.remove(key).unwrap();
        match resource.consumers {
            AwaitingHeaders(ref consumers) => {
                for consumer in consumers.iter() {
                    send_error_direct(key.url.clone(), err.clone(), consumer.clone());
                }
            }
            AwaitingBody(_, _, ref consumers) => {
                for consumer in consumers.iter() {
                    let _ = consumer.send_opt(Done(Ok(())));
                }
            }
        }
    }

    /// Handle a 304 response to a revalidation request. Updates the cached response
    /// metadata with any new expiration data.
    pub fn process_not_modified(&mut self, key: &CacheKey, headers: &ResponseHeaderCollection) {
        info!("updating metadata for {}", key.url);
        let resource = self.complete_entries.get_mut(key).unwrap();
        resource.expires = get_response_expiry_from_headers(headers);
    }

    /// Handle the initial response metadata for an incomplete cached request.
    /// If the response should not be cached, the entry will be doomed and any
    /// subsequent requests will not see the cached request. All waiting consumers
    /// will see the new metadata.
    pub fn process_metadata(&mut self, key: &CacheKey, metadata: Metadata) {
        info!("storing metadata for {}", key.url);
        let resource = self.pending_entries.get_mut(key).unwrap();
        let chans: Vec<Sender<ProgressMsg>>;
        match resource.consumers {
            AwaitingHeaders(ref consumers) => {
                chans = consumers.iter()
                                 .map(|chan| start_sending_opt(chan.clone(), metadata.clone()))
                                 .take_while(|chan| chan.is_ok())
                                 .map(|chan| chan.unwrap())
                                 .collect();
            }
            AwaitingBody(..) => panic!("obtained headers for {} but awaiting body?", key.url)
        }

        if !response_is_cacheable(&metadata) {
            resource.doomed = true;
        }

        resource.expires = get_response_expiry(&metadata);
        resource.last_validated = time::now();
        resource.consumers = AwaitingBody(metadata, vec!(), chans);
    }

    /// Handle a repsonse body payload for an incomplete cached response.
    /// All waiting consumers will see the new payload addition.
    pub fn process_payload(&mut self, key: &CacheKey, payload: Vec<u8>) {
        info!("storing partial response for {}", key.url);
        let resource = self.pending_entries.get_mut(key).unwrap();
        match resource.consumers {
            AwaitingBody(_, ref mut body, ref consumers) => {
                body.push_all(payload.as_slice());
                for consumer in consumers.iter() {
                    //FIXME: maybe remove consumer on failure to avoid extra clones?
                    let _ = consumer.send_opt(Payload(payload.clone()));
                }
            }
            AwaitingHeaders(_) => panic!("obtained body for {} but awaiting headers?", key.url)
        }
    }

    /// Handle a response body final payload for an incomplete cached response.
    /// All waiting consumers will see the new message. If the cache entry is
    /// doomed, it will not be transferred to the set of complete cache entries.
    pub fn process_done(&mut self, key: &CacheKey) {
        info!("finished fetching {}", key.url);
        let resource = self.pending_entries.remove(key).unwrap();
        match resource.consumers {
            AwaitingHeaders(_) => panic!("saw Done for {} but awaiting headers?", key.url),
            AwaitingBody(_, _, ref consumers) => {
                for consumer in consumers.iter() {
                    let _ = consumer.send_opt(Done(Ok(())));
                }
            }
        }

        if resource.doomed {
            info!("completing dooming of {}", key.url);
            return;
        }

        let (metadata, body) = match resource.consumers {
            AwaitingBody(metadata, body, _) => (metadata, body),
            _ => panic!("expected consumer list awaiting bodies"),
        };

        let complete = CachedResource {
            metadata: metadata,
            body: body,
            expires: resource.expires,
            last_validated: resource.last_validated,
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
    pub fn process_pending_request(&mut self, load_data: &LoadData, start_chan: Sender<LoadResponse>)
                                   -> CacheOperationResult {
        if load_data.method != Get {
            return Uncacheable("Only GET requests can be cached.");
        }

        let key = CacheKey::new(load_data.clone());
        match self.complete_entries.get(&key) {
            Some(resource) => {
                if self.base_time + resource.expires >= time::now().to_timespec() {
                    return Revalidate(key, ExpiryDate(time::at(self.base_time + resource.expires)));
                }

                let must_revalidate = resource.metadata.headers.as_ref().and_then(|headers| {
                    headers.cache_control.as_ref().map(|header| {
                        any_token_matches(header[], &["must-revalidate"])
                    })
                }).unwrap_or(false);

                if must_revalidate {
                    return Revalidate(key, ExpiryDate(resource.last_validated));
                }

                match resource.metadata.headers.as_ref().and_then(|headers| headers.etag.as_ref()) {
                    Some(etag) => return Revalidate(key, Etag(etag.clone())),
                    None => ()
                }

                //TODO: Revalidate once per session for response with no explicit expiry

                self.send_complete_entry(key, start_chan);
                return CachedContentPending;
            }

            None => ()
        }

        let new_entry = match self.pending_entries.get(&key) {
            Some(resource) if resource.doomed => return Uncacheable("Cache entry already doomed"),
            Some(_) => false,
            None => true,
        };

        if new_entry {
            self.add_pending_cache_entry(key.clone(), start_chan);
            NewCacheEntry(key)
        } else {
            self.send_partial_entry(key, start_chan);
            CachedContentPending
        }
    }

    /// Add a new pending request to the set of incomplete cache entries.
    fn add_pending_cache_entry(&mut self, key: CacheKey, start_chan: Sender<LoadResponse>) {
        let resource = PendingResource {
            consumers: AwaitingHeaders(vec!(start_chan)),
            expires: MAX,
            last_validated: time::now(),
            doomed: false,
        };
        info!("creating cache entry for {}", key.url);
        self.pending_entries.insert(key, resource);
    }

    /// Synchronously send the entire cached response body to the given consumer.
    fn send_complete_entry(&self, key: CacheKey, start_chan: Sender<LoadResponse>) {
        info!("returning full cache body for {}", key.url);
        let resource = self.complete_entries.get(&key).unwrap();
        let progress_chan = start_sending_opt(start_chan, resource.metadata.clone());
        match progress_chan {
            Ok(chan) => {
                let _ = chan.send_opt(Payload(resource.body.clone()));
                let _ = chan.send_opt(Done(Ok(())));
            }
            Err(_) => ()
        }
    }

    /// Synchronously send all partial stored response data for a cached request to the
    /// given consumer.
    fn send_partial_entry(&mut self, key: CacheKey, start_chan: Sender<LoadResponse>) {
        info!("returning partial cache data for {}", key.url);

        let resource = self.pending_entries.get_mut(&key).unwrap();

        match resource.consumers {
            AwaitingHeaders(ref mut consumers) => {
                consumers.push(start_chan);
            }
            AwaitingBody(ref metadata, ref body, ref mut consumers) => {
                info!("headers available for {}", key.url);
                let progress_chan = start_sending_opt(start_chan, metadata.clone());
                match progress_chan {
                    Ok(chan) => {
                        consumers.push(chan.clone());

                        if !body.is_empty() {
                            info!("partial body available for {}", key.url);
                            let _ = chan.send_opt(Payload(body.clone()));
                        }
                    }

                    Err(_) => ()
                }
            }
        }
    }
}
