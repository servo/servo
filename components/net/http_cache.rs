/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

#![deny(missing_docs)]

//! A memory cache Implements the logic specified in http://tools.ietf.org/html/rfc7234
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


/// The key used to differentiate requests in the cache.
#[derive(Clone, Debug, Eq, Hash, PartialEq)]
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
                                      .filter(|header| {
                                          match header.name().to_lowercase().as_ref() {
                                              "cache-control" | "vary" | "expires" => false,
                                              _ => true
                                          }
                                      })
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
#[derive(Debug)]
struct CachedResource {
    metadata: CachedMetadata,
    body: Arc<Mutex<ResponseBody>>,
    status: Option<StatusCode>,
    raw_status: Option<(u16, Vec<u8>)>,
    url_list: Vec<ServoUrl>,
    expires: Arc<Mutex<Duration>>,
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
    pub headers: Arc<Mutex<Headers>>,

    /// HTTP Status
    pub status: Option<(u16, Vec<u8>)>
}

/// Wrapper around a cached response, including information on re-validation needs
pub struct CachedResponse {
    /// The response constructed from the cached resource
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


/// Determine if a given response is cacheable based on the initial metadata received.
/// Based on http://tools.ietf.org/html/rfc7234#section-5
fn response_is_cacheable(metadata: &Metadata) -> bool {
    if metadata.headers.is_none() {
        return true;
    }

    let headers = metadata.headers.as_ref().unwrap();
    match headers.get::<header::CacheControl>() {
        Some(&header::CacheControl(ref directive)) => {
            for directive in directive.iter() {
                if header::CacheDirective::NoStore == *directive {
                    return false;
                }
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

/// Calculating Age https://tools.ietf.org/html/rfc7234#section-4.2.3
fn calculate_response_age(response: &Response) -> Duration {
    // TODO: follow the spec more closely (Date headers, request/response lag, ...)
    if let Some(secs) = response.headers.get_raw("Age") {
        let seconds = String::from_utf8(secs[0].to_vec()).unwrap();
        return Duration::seconds(seconds.parse::<i64>().unwrap());
    } else {
        return Duration::seconds(0i64);
    }
}

/// Determine the expiry date of the given response headers,
/// or uses a heuristic if none are present.
fn get_response_expiry(response: &Response) -> Duration {
    // Calculating Freshness Lifetime https://tools.ietf.org/html/rfc7234#section-4.2.1
    if let Some(&header::CacheControl(ref directives)) = response.headers.get::<header::CacheControl>() {
        let has_no_cache_directive = directives.iter().any(|directive| {
            header::CacheDirective::NoCache == *directive
        });
        if has_no_cache_directive {
            // Requires validation on first use.
            return Duration::seconds(0i64);
        } else {
            for directive in directives {
                let age = calculate_response_age(&response);
                match *directive {
                    header::CacheDirective::SMaxAge(secs) | header::CacheDirective::MaxAge(secs) => {
                        let max_age = Duration::seconds(secs as i64);
                        if max_age < age {
                            return return Duration::seconds(0i64);
                        }
                        return max_age - age;
                    },
                    _ => (),
                }
            }
        }

    }
    if let Some(&header::Expires(header::HttpDate(t))) = response.headers.get::<header::Expires>() {
        println!("Expires headers for {:?}", t);
        // store the period of time from now until expiry
        let desired = t.to_timespec();
        let current = time::now().to_timespec();
        if desired > current {
            return desired - current;
        } else {
            return Duration::seconds(0i64);
        }
    } else {
       if let Some(val) = response.headers.get_raw("Expires") {
           // Malformed Expires header, shouldn't be used to construct a valid response.
           return Duration::seconds(0i64);
       }
    }
    // Calculating Heuristic Freshness https://tools.ietf.org/html/rfc7234#section-4.2.2
    if let Some((ref code, _)) = response.raw_status {
        let heuristic_freshness = if let Some(&header::LastModified(header::HttpDate(t))) =
            response.headers.get::<header::LastModified>() {
            let last_modified = t.to_timespec();
            let current = time::now().to_timespec();
            (current - last_modified) / 10
        } else {
            // https://tools.ietf.org/html/rfc7234#section-5.5.4
            // Since we do not generate such a warning, 24 hours is the max for heuristic calculation.
            Duration::hours(24)
        };
        match *code {
            200 | 203 | 204 | 206 | 300 | 301 | 404 | 405 | 410 | 414 | 501 => {
                // Status codes that are cacheable by default https://tools.ietf.org/html/rfc7231#section-6.1
                return heuristic_freshness
            },
            _ => {
                // Other status codes can only use heuristic freshness if the public cache directive is present.
                if let Some(&header::CacheControl(ref directives)) = response.headers.get::<header::CacheControl>() {
                    let has_public_directive = directives.iter().any(|directive| {
                        header::CacheDirective::Public == *directive
                    });
                    if has_public_directive {
                        return heuristic_freshness;
                    }
                }
            },
        }
    }
    // Requires validation upon first use as default.
    Duration::seconds(0i64)
}

fn get_expire_adjustment_from_request_headers(request: &Request, expires: Duration) -> Duration {
    if let Some(directive_data) = request.headers.get_raw("cache-control") {
        let directives_string = String::from_utf8(directive_data[0].to_vec()).unwrap();
        println!("received request cache controle {:?}", directives_string);
        let directives: Vec<&str> = directives_string.split(",").collect();
        println!("received request cache controle split{:?}", directives);
        for directive in directives {
            let directive_info: Vec<&str> = directive.split("=").collect();
            println!("received request cache controle directive {:?}", directive_info);
            match directive_info[0] {
                "max-stale" => {
                    println!("received request max-stale {:?}", directive_info[1]);
                    let seconds = String::from_str(directive_info[1]).unwrap();
                    return expires + Duration::seconds(seconds.parse::<i64>().unwrap());
                },
                "max-age" => {
                    println!("received request max-age {:?}", directive_info[1]);
                    let seconds = String::from_str(directive_info[1]).unwrap();
                    let max_age = Duration::seconds(seconds.parse::<i64>().unwrap());
                    if expires > max_age {
                        return Duration::min_value();
                    }
                    return expires - max_age;
                },
                "min-fresh" => {
                    println!("received request min-fresh {:?}", directive_info[1]);
                    let seconds = String::from_str(directive_info[1]).unwrap();
                    let min_fresh = Duration::seconds(seconds.parse::<i64>().unwrap());
                    if expires < min_fresh {
                        return Duration::min_value();
                    }
                    return expires - min_fresh;
                },
                "no-cache" | "no-store" => return Duration::min_value(),
                _ => {}
            }
        }
    }
    expires
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
            response.headers = cached_resource.metadata.headers.lock().unwrap().clone();
            response.body = cached_resource.body.clone();
            response.status = cached_resource.status.clone();
            response.raw_status = cached_resource.raw_status.clone();
            response.url_list = cached_resource.url_list.clone();
            if let ResponseBody::Receiving(_) = *response.body.lock().unwrap() {
                let (done_sender, done_receiver) = channel();
                *done_chan = Some((done_sender.clone(), done_receiver));
                cached_resource.awaiting_body.lock().unwrap().push(done_sender);
            }
            let expires = *cached_resource.expires.lock().unwrap();
            println!("expires: {:?}", expires);
            let adjusted_expires = get_expire_adjustment_from_request_headers(request, expires);
            println!("adjusted_expires: {:?}", adjusted_expires);
            let now = Duration::seconds(time::now().to_timespec().sec);
            println!("now: {:?}", now);
            let last_validated = Duration::seconds(cached_resource.last_validated.to_timespec().sec);
            println!("now: {:?}", last_validated);
            let time_since_validated = now - last_validated;
            let has_expired = (adjusted_expires < time_since_validated)
                | (adjusted_expires == time_since_validated);
            println!("constructing for: {:?} {:?}", response, has_expired);
            return Some(CachedResponse { response: response, needs_validation: has_expired });
        }
        None
    }

    /// https://tools.ietf.org/html/rfc7234#section-4.3.4 Freshening Stored Responses upon Validation.
    pub fn refresh(&mut self, request: &Request, response: Response, done_chan: &mut DoneChannel) -> Option<Response> {
        for (key, cached_resource) in self.entries.iter_mut() {
            println!("comparing: {:?} with  {:?}", key.url(), request.url());
            println!("result: {:?}", key.url() == request.url());
            if key.url() == request.url() {
                if let Ok(ref mut stored_headers) = cached_resource.metadata.headers.try_lock() {
                    println!("starting refreshing: {:?}", stored_headers);
                    stored_headers.extend(response.headers.iter());
                    let mut response_200 = Response::new(cached_resource.metadata.final_url.clone());
                    response_200.headers = stored_headers.clone();
                    response_200.body = cached_resource.body.clone();
                    response_200.status = cached_resource.status.clone();
                    response_200.raw_status = cached_resource.raw_status.clone();
                    response_200.url_list = cached_resource.url_list.clone();
                    *done_chan = None;
                    let mut expires = cached_resource.expires.lock().unwrap();
                    *expires = get_response_expiry(&response_200);
                    println!("refreshing: {:?}", response_200);
                    return Some(response_200);
                }
            }
        }
        None
    }

    /// https://tools.ietf.org/html/rfc7234#section-4.4 Invalidation.
    pub fn invalidate(&mut self, request: &Request, response: &Response) {
        let mut location = String::new();
        let mut content_location = String::new();
        if let Some(&header::Location(ref url)) = response.headers.get::<header::Location>() {
            location = url.clone();
        }
        if let Some(url_data) = response.headers.get_raw("Content-Location") {
            content_location = String::from_utf8(url_data[0].to_vec()).unwrap();
        }
        for (key, cached_resource) in self.entries.iter_mut() {
            let string_resource_url = key.url().into_string();
            let matches = (key.url() == request.url()) |
                (string_resource_url == location) |
                (string_resource_url == content_location);
            if matches {
                println!("invalidating: {:?}", key);
                let mut expires = cached_resource.expires.lock().unwrap();
                *expires = Duration::seconds(0i64);
            }
        }
    }

    /// Updating the cached response body from ResponseBody::Receiving to ResponseBody::Done.
    pub fn update_response_body(&mut self, request: &Request, response: &Response) {
        if let Some((ref code, _)) = response.raw_status {
            println!("updating for code {:?}", code);
            if *code == 304 {
                println!("not updating because 304: response: {:?}", response);
                return
            }
        }
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
        let entry_key = CacheKey::new(request.clone());
        println!("received store for key: {:?} response: {:?}", entry_key, response);
        match request.method {
            // Only cache Get requests https://tools.ietf.org/html/rfc7234#section-2
            Method::Get => {},
            _ => return,
        }
        match response.status {
            // Not caching redirects.
            Some(StatusCode::SeeOther) | Some(StatusCode::MovedPermanently)
            | Some(StatusCode::TemporaryRedirect) | Some(StatusCode::Found) => return,
            _ => {}
        }
        if let Some((ref code, _)) = response.raw_status {
            println!("store for code {:?}", code);
            if *code == 304 {
                println!("not storing key because 304: {:?} response: {:?}", entry_key, response);
                return
            }
        }
        match response.metadata() {
            Ok(FetchMetadata::Filtered {
               filtered: FilteredMetadata::Basic(metadata),
               unsafe_: _ }) |
            Ok(FetchMetadata::Filtered {
                filtered: FilteredMetadata::Cors(metadata),
                unsafe_: _ }) |
            Ok(FetchMetadata::Unfiltered(metadata)) => {
                if response_is_cacheable(&metadata) {
                    let expiry = get_response_expiry(&response);
                    let cacheable_metadata = CachedMetadata {
                        final_url: metadata.final_url,
                        content_type: metadata.content_type,
                        charset: metadata.charset,
                        status: metadata.status,
                        headers: Arc::new(Mutex::new(response.headers.clone()))
                    };
                    let entry_resource = CachedResource {
                        metadata: cacheable_metadata,
                        body: response.body.clone(),
                        status: response.status,
                        raw_status: response.raw_status.clone(),
                        url_list: response.url_list.clone(),
                        expires: Arc::new(Mutex::new(expiry)),
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
