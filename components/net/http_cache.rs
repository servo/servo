/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

#![deny(missing_docs)]

//! A non-validating memory cache that only evicts expired entries and grows
//! without bound. Implements the logic specified in http://tools.ietf.org/html/rfc7234
//! and http://tools.ietf.org/html/rfc7232.

use fetch::methods::{Data, DoneChannel};
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
use std::sync::mpsc::{channel, Sender};
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
#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub struct CacheKey {
    url_list: Vec<ServoUrl>,
    request_headers: Vec<(String, String)>,
}

impl CacheKey {
    fn new(request: Request) -> CacheKey {
        CacheKey {
            url_list: request.url_list.clone(),
            request_headers: request.headers
                                      .iter()
                                      .map(|header| (String::from_str(header.name()).unwrap_or(String::from("None")),
                                                      header.value_string()))
                                      .collect(),
        }
    }
}

/// A complete cached resource.
#[derive(Debug)]
struct CachedResource {
    metadata: CachedMetadata,
    body: Arc<Mutex<ResponseBody>>,
    status: Option<StatusCode>,
    raw_status: Option<(u16, Vec<u8>)>,
    url_list: Vec<ServoUrl>,
    expires: Duration,
    last_validated: Tm,
    awaiting_body: Arc<Mutex<Vec<Sender<Data>>>>
}

/// Metadata about a loaded resource, such as is obtained from HTTP headers.
#[derive(Debug)]
struct CachedMetadata {
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

/// Wrapper around a cached response, including information on re-validation needs
pub struct CachedResponse {
    /// The stored response
    pub response: Response,

    /// The revalidation flag for the stored response
    pub needs_validation: bool
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

    if let Some((ref code, _)) = metadata.status {
        // Status codes that are cacheable by default https://tools.ietf.org/html/rfc7231#section-6.1
        match *code {
            200 | 203 | 204 | 206 | 300 | 301 | 404 | 405 | 410 | 414 | 501 => return true,
            _ => {},
        }
    }

    return true;
}

/// Determine the expiry date of the given response headers,
/// or uses a heuristic if none are present.
fn get_response_expiry_from_headers(headers: &Headers) -> Duration {
    // Calculating Freshness Lifetime https://tools.ietf.org/html/rfc7234#section-4.2.1
    if let Some(&header::CacheControl(ref directives)) = headers.get::<header::CacheControl>() {
        for directive in directives {
            match directive {
                &header::CacheDirective::SMaxAge(secs) => {
                    return Duration::seconds(secs as i64);
                },
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
    // Calculating Heuristic Freshness https://tools.ietf.org/html/rfc7234#section-4.2.2
    if let Some(&header::LastModified(header::HttpDate(t))) = headers.get::<header::LastModified>() {
        let last_modified = t.to_timespec();
        let current = time::now().to_timespec();
        return (current - last_modified) / 10;
    }
    // https://tools.ietf.org/html/rfc7234#section-5.5.4
    // Since we do not generate such a warning, 24 hours is the max for heuristic calculation.
    Duration::hours(24)
}

impl HttpCache {
    /// Create a new memory cache instance.
    pub fn new() -> HttpCache {
        HttpCache {
            entries: HashMap::new(),
            base_time: time::now().to_timespec(),
        }
    }

    /// https://tools.ietf.org/html/rfc7234#section-4 Constructing Responses from Caches.
    pub fn construct_response(&self, request: &Request, done_chan: &mut DoneChannel)
        -> Option<CachedResponse> {
        let entry_key = CacheKey::new(request.clone());
        println!("received construct_response for {:?}", entry_key);
        if let Some(cached_resource) = self.entries.get(&entry_key) {
            let mut response = Response::new(cached_resource.metadata.final_url.clone());
            let mut headers = Headers::new();
            if let Some(ref header_list) = cached_resource.metadata.headers {
                for &(ref name, ref value) in header_list {
                    let header_values: Vec<Vec<u8>> = value.split(",").map(|val| String::from(val).into_bytes())
                        .collect();
                    headers.set_raw(name.clone(), header_values);
                }
            };
            response.headers = headers;
            response.body = cached_resource.body.clone();
            response.status = cached_resource.status.clone();
            response.raw_status = cached_resource.raw_status.clone();
            response.url_list = cached_resource.url_list.clone();
            if let ResponseBody::Receiving(_) = *response.body.lock().unwrap() {
                let (done_sender, done_receiver) = channel();
                *done_chan = Some((done_sender.clone(), done_receiver));
                cached_resource.awaiting_body.lock().unwrap().push(done_sender);
            }
            let has_expired = self.base_time + cached_resource.expires < time::now().to_timespec();
            println!("constructing for: {:?} {:?}", response, has_expired);
            return Some(CachedResponse { response: response, needs_validation: has_expired });
        }
        None
    }

    /// https://tools.ietf.org/html/rfc7234#section-4.3.4 Freshening Stored Responses upon Validation.
    pub fn refresh(&mut self, request: &Request, response: &Response) {}

    /// https://tools.ietf.org/html/rfc7234#section-4.4 Invalidation.
    pub fn invalidate(&mut self, request: &Request) {}

    /// Updating the cached response body from ResponseBody::Receiving to ResponseBody::Done.
    pub fn update_response_body(&mut self, request: &Request, response: &Response) {
        let entry_key = CacheKey::new(request.clone());
        if let Some(mut cached_resource) = self.entries.get(&entry_key) {
            println!("updating response for {:?}", entry_key);
            if let ResponseBody::Done(ref completed_body) = *response.body.lock().unwrap() {
                let mut body = cached_resource.body.lock().unwrap();
                match *body {
                    ResponseBody::Receiving(_) => {
                        *body = ResponseBody::Done(completed_body.clone());
                        let mut awaiting_consumers = cached_resource.awaiting_body.lock().unwrap();
                        for done_sender in awaiting_consumers.drain(..) {
                            done_sender.send(Data::Payload(completed_body.clone()));
                            done_sender.send(Data::Done);
                        };
                    },
                    _ => {},
                }

            }
        }
    }


    /// https://tools.ietf.org/html/rfc7234#section-3 Storing Responses in Caches.
    pub fn store(&mut self, request: &Request, response: &Response) {
        println!("received store: {:?}", response);
        println!("received metatdata: {:?}", response.metadata());
        match request.method {
            // Only cache Get requests https://tools.ietf.org/html/rfc7234#section-2
            Method::Get => {},
            _ => return,
        }
        match response.metadata() {
            Ok(FetchMetadata::Filtered {
               filtered: FilteredMetadata::Basic(metadata),
               unsafe_: _ }) |
            Ok(FetchMetadata::Filtered {
                filtered: FilteredMetadata::Cors(metadata),
                unsafe_: _ }) |
            Ok(FetchMetadata::Unfiltered(metadata)) => {
                println!("checking if response is cacheable: {:?}", response);
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
                        body: response.body.clone(),
                        status: response.status,
                        raw_status: response.raw_status.clone(),
                        url_list: response.url_list.clone(),
                        expires: get_response_expiry_from_headers(&response.headers),
                        last_validated: time::now(),
                        awaiting_body: Arc::new(Mutex::new(vec![]))
                    };
                    println!("storing: {:?} {:?}", entry_key, entry_resource);
                    self.entries.insert(entry_key, entry_resource);
                }
            },
            _ => { println!("not storing: {:?}", response); }
        }
    }

}
