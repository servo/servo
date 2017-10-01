/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

#![deny(missing_docs)]

//! A non-validating memory cache that only evicts expired entries and grows
//! without bound. Implements the logic specified in http://tools.ietf.org/html/rfc7234
//! and http://tools.ietf.org/html/rfc7232.

use hyper::header;
use hyper::header::{ContentType, EntityTag};
use hyper::header::Headers;
use hyper::method::Method;
use hyper::status::StatusCode;
use hyper_serde::Serde;
use net_traits::{Metadata, FetchMetadata, FilteredMetadata};
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
    metadata: CachedMetadata,
    body: ResponseBody,
    expires: Duration,
    last_validated: Tm,
}

/// Metadata about a loaded resource, such as is obtained from HTTP headers.
#[derive(Clone)]
pub struct CachedMetadata {
    /// Final URL after redirects.
    pub final_url: ServoUrl,

    /// MIME type / subtype.
    pub content_type: Option<Serde<ContentType>>,

    /// Character set.
    pub charset: Option<String>,

    /// Headers
    pub headers: Option<Vec<(String, String)>>,

    /// HTTP Status
    pub status: Option<(u16, Vec<u8>)>
}

/// A memory cache that tracks incomplete and complete responses, differentiated by
/// the initial request.
pub struct HttpCache {
    /// cached responses.
    entries: HashMap<CacheKey, CachedResource>,
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
    if let Some((_, ref status)) = metadata.status {
        if status != &b"OK".to_vec() {
            return false;
        }
    }

    if metadata.headers.is_none() {
        return true;
    }

    let headers = metadata.headers.as_ref().unwrap();
    match headers.get::<header::CacheControl>() {
        Some(&header::CacheControl(ref directive)) => {
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
        },
        _ => ()
    }

    return true;
}

/// Determine the expiry date of the given response headers.
/// Returns a far-future date if the response does not expire.
fn get_response_expiry_from_headers(headers: &Headers) -> Duration {
    if let Some(&header::CacheControl(ref directives)) = headers.get::<header::CacheControl>() {
        for directive in directives {
            match directive {
                &header::CacheDirective::MaxAge(secs) => {
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

impl HttpCache {
    /// Create a new memory cache instance.
    pub fn new() -> HttpCache {
        HttpCache {
            entries: HashMap::new(),
            base_time: time::now().to_timespec(),
        }
    }

    /// Try to fetch a cached response.
    pub fn try_cache_fetch(&self, request: &Request) -> Option<Response> {
        None
    }

    /// Update the cache with a new response.
    pub fn update_cache(&mut self, request: &Request, response: &Response) {
        match response.metadata() {
            Ok(FetchMetadata::Filtered {
               filtered: FilteredMetadata::Basic(metadata),
               unsafe_: unsafe_metadata }) |
            Ok(FetchMetadata::Filtered {
                filtered: FilteredMetadata::Cors(metadata),
                unsafe_: unsafe_metadata }) => {
                if response_is_cacheable(&metadata) {
                    let entry_key = CacheKey::new(request.clone());
                    let raw_headers = metadata.headers.map_or(None, |headers|
                        Some(headers.iter()
                            .map(|header|
                                    (String::from_str(header.name()).unwrap_or(String::from("None")),
                                    header.value_string()))
                            .collect()));
                    let cacheable_metadata = CachedMetadata {
                        final_url: metadata.final_url,
                        content_type: metadata.content_type,
                        charset: metadata.charset,
                        status: metadata.status,
                        headers: raw_headers
                    };
                    let entry_resource = CachedResource {
                        metadata: cacheable_metadata,
                        body: response.body.lock().unwrap().clone(),
                        expires: get_response_expiry_from_headers(&response.headers),
                        last_validated: time::now()
                    };
                    self.entries.insert(entry_key, entry_resource);
                }
            },
            _ => {}
        }
    }

}
