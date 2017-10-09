/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

#![deny(missing_docs)]

//! A memory cache Implements the logic specified in http://tools.ietf.org/html/rfc7234
//! and http://tools.ietf.org/html/rfc7232.

use fetch::methods::{Data, DoneChannel};
use hyper::header;
use hyper::header::ContentType;
use hyper::header::Headers;
use hyper::method::Method;
use hyper::status::StatusCode;
use hyper_serde::Serde;
use net_traits::{Metadata, FetchMetadata, FilteredMetadata};
use net_traits::request::Request;
use net_traits::response::{Response, ResponseBody};
use servo_url::ServoUrl;
use std::collections::HashMap;
use std::str::FromStr;
use std::sync::{Arc, Mutex};
use std::sync::mpsc::{channel, Sender};
use time;
use time::{Duration, Tm};


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
                                              "cache-control" | "expires" => false,
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

    /// Retrieve the request headers associated with this key
    pub fn request_headers(&self) -> Vec<(String, String)> {
        self.request_headers.clone()
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
                            return Duration::seconds(0i64);
                        }
                        return max_age - age;
                    },
                    _ => (),
                }
            }
        }
    }
    if let Some(&header::Expires(header::HttpDate(t))) = response.headers.get::<header::Expires>() {
        // store the period of time from now until expiry
        let desired = t.to_timespec();
        let current = time::now().to_timespec();
        if desired > current {
            return desired - current;
        } else {
            return Duration::seconds(0i64);
        }
    } else {
       if let Some(_) = response.headers.get_raw("Expires") {
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

/// Request Cache-Control Directives https://tools.ietf.org/html/rfc7234#section-5.2.1
fn get_expire_adjustment_from_request_headers(request: &Request, expires: Duration) -> Duration {
    if let Some(directive_data) = request.headers.get_raw("cache-control") {
        let directives_string = String::from_utf8(directive_data[0].to_vec()).unwrap();
        let directives: Vec<&str> = directives_string.split(",").collect();
        for directive in directives {
            let directive_info: Vec<&str> = directive.split("=").collect();
            match directive_info[0] {
                "max-stale" => {
                    let seconds = String::from_str(directive_info[1]).unwrap();
                    return expires + Duration::seconds(seconds.parse::<i64>().unwrap());
                },
                "max-age" => {
                    let seconds = String::from_str(directive_info[1]).unwrap();
                    let max_age = Duration::seconds(seconds.parse::<i64>().unwrap());
                    if expires > max_age {
                        return Duration::min_value();
                    }
                    return expires - max_age;
                },
                "min-fresh" => {
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

/// Create a CachedResponse from a request and a CachedResource.
fn create_cached_response(request: &Request, cached_resource: &CachedResource, done_chan: &mut DoneChannel)
    -> CachedResponse {
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
    let adjusted_expires = get_expire_adjustment_from_request_headers(request, expires);
    let now = Duration::seconds(time::now().to_timespec().sec);
    let last_validated = Duration::seconds(cached_resource.last_validated.to_timespec().sec);
    let time_since_validated = now - last_validated;
    let has_expired = (adjusted_expires < time_since_validated)
        | (adjusted_expires == time_since_validated);
    CachedResponse { response: response, needs_validation: has_expired }
}


impl HttpCache {
    /// Create a new memory cache instance.
    pub fn new() -> HttpCache {
        HttpCache {
            entries: HashMap::new()
        }
    }

    /// Calculating Secondary Keys with Vary https://tools.ietf.org/html/rfc7234#section-4.1
    fn calculate_secondary_keys_with_vary(&self, request: &Request) -> Option<&CachedResource> {
        let mut can_be_constructed = vec![];
        for (key, cached_resource) in self.entries.iter() {
            if key.url() == request.url() {
                if let Ok(ref mut stored_headers) = cached_resource.metadata.headers.try_lock() {
                    if let Some(vary_data) = stored_headers.get_raw("vary") {
                        let vary_data_string = String::from_utf8(vary_data[0].to_vec()).unwrap();
                        let vary_values: Vec<&str> = vary_data_string.split(",").collect();
                        for vary_val in vary_values {
                            if let Some(header_data) = request.headers.get_raw(vary_val) {
                                let request_header_data_string = String::from_utf8(header_data[0].to_vec()).unwrap();
                                let request_vary_values: Vec<&str> = request_header_data_string.split(",").collect();
                                let mut ok = true;
                                for (name, value) in key.request_headers() {
                                    if name.to_lowercase() == vary_val.to_lowercase() {
                                        let stored_vary_values: Vec<&str> = value.split(",").collect();
                                        ok = request_vary_values == stored_vary_values;
                                    }
                                }
                                if ok {
                                    can_be_constructed.push(cached_resource.clone());
                                }
                            }
                        }
                    }
                }
            }
        }
        // TODO: select most recent resource using the Date header.
        if let Some(resource) = can_be_constructed.first() {
            return Some(resource.clone());
        }
        None
    }

    /// https://tools.ietf.org/html/rfc7234#section-4 Constructing Responses from Caches.
    pub fn construct_response(&self, request: &Request, done_chan: &mut DoneChannel)
        -> Option<CachedResponse> {
        let entry_key = CacheKey::new(request.clone());
        return match (self.entries.get(&entry_key), self.calculate_secondary_keys_with_vary(request)) {
            (None, Some(cached_resource)) => {
                let cached_response = create_cached_response(request, cached_resource, done_chan);
                Some(cached_response)
            },
            (Some(ref cached_resource), Some(_)) | (Some(ref cached_resource), None) => {
                let cached_response = create_cached_response(request, cached_resource, done_chan);
                Some(cached_response)
            },
            _ => None,
        };
    }

    /// https://tools.ietf.org/html/rfc7234#section-4.3.4 Freshening Stored Responses upon Validation.
    pub fn refresh(&mut self, request: &Request, response: Response, done_chan: &mut DoneChannel) -> Option<Response> {
        for (key, cached_resource) in self.entries.iter_mut() {
            if key.url() == request.url() {
                if let Ok(ref mut stored_headers) = cached_resource.metadata.headers.try_lock() {
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
                let mut expires = cached_resource.expires.lock().unwrap();
                *expires = Duration::seconds(0i64);
            }
        }
    }

    /// Updating the cached response body from ResponseBody::Receiving to ResponseBody::Done.
    pub fn update_response_body(&mut self, request: &Request, response: &Response) {
        if let Some((ref code, _)) = response.raw_status {
            if *code == 304 {
                return
            }
        }
        let entry_key = CacheKey::new(request.clone());
        if let Some(cached_resource) = self.entries.get(&entry_key) {
            if let ResponseBody::Done(ref completed_body) = *response.body.lock().unwrap() {
                let mut body = cached_resource.body.lock().unwrap();
                match *body {
                    ResponseBody::Receiving(_) => {
                        *body = ResponseBody::Done(completed_body.clone());
                        let mut awaiting_consumers = cached_resource.awaiting_body.lock().unwrap();
                        for done_sender in awaiting_consumers.drain(..) {
                            let _ = done_sender.send(Data::Payload(completed_body.clone()));
                            let _ = done_sender.send(Data::Done);
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
            if *code == 304 {
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

                    self.entries.insert(entry_key, entry_resource);
                }
            },
            _ => {}
        }
    }

}
