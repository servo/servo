/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

#![deny(missing_docs)]

//! A non-validating memory cache that only evicts expired entries and grows
//! without bound. Implements the logic specified in http://tools.ietf.org/html/rfc7234
//! and http://tools.ietf.org/html/rfc7232.

use hyper::header;
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
    /// The time at which this cache was created for use by expiry checks.
    base_time: Timespec,
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


/// Determine if a given response is cacheable based on the initial metadata received.
/// Based on http://tools.ietf.org/html/rfc7234#section-5
fn response_is_cacheable(metadata: &Metadata) -> bool {
    if let Some((_, status)) = metadata.status {
        if status != b"OK".to_vec() {
            return false;
        }
    }

    if metadata.headers.is_none() {
        return true;
    }

    let headers = metadata.headers.as_ref().unwrap();
    match headers.get::<header::CacheControl>() {
        Some(&header::CacheControl(directive)) => {
            let has_no_cache_directives = directive.iter().any(|directive|
                match *directive {
                    header::CacheDirective::NoCache |
                    header::CacheDirective::NoStore |
                    header::CacheDirective::MaxAge(0u32) => {
                        true
                    },
                    _ => false,
            });
            if has_no_cache_directives {
                return false;
            }
        },
        None => ()
    }

    match headers.get::<header::Pragma>() {
        Some(&header::Pragma::NoCache) => {
            return false;
        }
        None => ()
    }

    return true;
}

/// Determine the expiry date of the given response headers.
/// Returns a far-future date if the response does not expire.
fn get_response_expiry_from_headers(headers: &Headers) -> Duration {
    if let Some(&header::CacheControl(directives)) = headers.get::<header::CacheControl>() {
        for directive in directives {
            match directive {
                header::CacheDirective::MaxAge(secs) => {
                    return Duration::seconds(secs as i64);
                },
                _ => (),
            }
        }
    }
    if let Some(&header::Expires(header::HttpDate(t))) = headers.get::<header::Expires>() {
        // store the period of time from now until expiry
        let desired = t.to_timespec();
        let current = time::now().to_timespec();
        if desired > current {
            return desired - current;
        } else {
            return Duration::min_value();
        }
    }
    Duration::max_value()
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
            base_time: time::now().to_timespec(),
        }
    }

    /// Process a revalidation that returned new content for an expired entry.
    pub fn process_revalidation_failed(&mut self, key: &CacheKey) {
        debug!("recreating entry for {} (cache entry expired)", key.url);
        let resource = self.complete_entries.remove(key).unwrap();
        self.add_pending_cache_entry(key.clone(), resource.revalidating_consumers);
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

    /// Handle a response body final payload for response.
    pub fn add_cache_entry(&mut self, key: &CacheKey) {
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
        self.add_cache_entry(key.clone());
        CacheOperationResult::NewCacheEntry(key)
    }

    /// Synchronously send the entire cached response body to the given consumer.
    fn send_complete_entry(&self, key: CacheKey, start_chan: Sender<Response>) {
        debug!("returning full cache body for {}", key.url);
        let resource = self.complete_entries.get(&key).unwrap();
        MemoryCache::send_complete_resource(resource, start_chan)
    }
}
