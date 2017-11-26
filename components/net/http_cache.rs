/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

#![deny(missing_docs)]

//! A memory cache implementing the logic specified in http://tools.ietf.org/html/rfc7234
//! and <http://tools.ietf.org/html/rfc7232>.

use fetch::methods::{Data, DoneChannel};
use hyper::header;
use hyper::header::ContentType;
use hyper::header::Headers;
use hyper::method::Method;
use hyper::status::StatusCode;
use hyper_serde::Serde;
use net_traits::{Metadata, FetchMetadata};
use net_traits::request::Request;
use net_traits::response::{HttpsState, Response, ResponseBody};
use servo_config::prefs::PREFS;
use servo_url::ServoUrl;
use std::collections::HashMap;
use std::str;
use std::sync::{Arc, Mutex};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::mpsc::{channel, Sender};
use time;
use time::{Duration, Tm};


/// The key used to differentiate requests in the cache.
#[derive(Clone, Eq, Hash, PartialEq)]
pub struct CacheKey {
    url: ServoUrl
}

impl CacheKey {
    fn new(request: Request) -> CacheKey {
        CacheKey {
            url: request.current_url().clone()
        }
    }

    fn from_servo_url(servo_url: &ServoUrl) -> CacheKey {
        CacheKey {
            url: servo_url.clone()
        }
    }

    /// Retrieve the URL associated with this key
    pub fn url(&self) -> ServoUrl {
        self.url.clone()
    }
}

/// A complete cached resource.
#[derive(Clone)]
struct CachedResource {
    metadata: CachedMetadata,
    request_headers: Arc<Mutex<Headers>>,
    body: Arc<Mutex<ResponseBody>>,
    location_url: Option<Result<ServoUrl, String>>,
    https_state: HttpsState,
    status: Option<StatusCode>,
    raw_status: Option<(u16, Vec<u8>)>,
    url_list: Vec<ServoUrl>,
    expires: Duration,
    last_validated: Tm,
    aborted: Arc<AtomicBool>,
    awaiting_body: Arc<Mutex<Vec<Sender<Data>>>>
}

/// Metadata about a loaded resource, such as is obtained from HTTP headers.
#[derive(Clone)]
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

/// A memory cache.
pub struct HttpCache {
    /// cached responses.
    entries: HashMap<CacheKey, Vec<CachedResource>>,
}


/// Determine if a given response is cacheable based on the initial metadata received.
/// Based on <https://tools.ietf.org/html/rfc7234#section-3>
fn response_is_cacheable(metadata: &Metadata) -> bool {
    // TODO: if we determine that this cache should be considered shared:
    // 1. check for absence of private response directive <https://tools.ietf.org/html/rfc7234#section-5.2.2.6>
    // 2. check for absence of the Authorization header field.
    let mut is_cacheable = false;
    let headers = metadata.headers.as_ref().unwrap();
    if headers.has::<header::Expires>() ||
        headers.has::<header::LastModified>() ||
        headers.has::<header::ETag>() {
        is_cacheable = true;
    }
    if let Some(&header::CacheControl(ref directive)) = headers.get::<header::CacheControl>() {
        for directive in directive.iter() {
            match *directive {
                header::CacheDirective::NoStore => return false,
                header::CacheDirective::Public | header::CacheDirective::SMaxAge(_)
                | header::CacheDirective::MaxAge(_) | header::CacheDirective::NoCache => is_cacheable = true,
                _ => {},
            }
        }
    }
    if let Some(&header::Pragma::NoCache) = headers.get::<header::Pragma>() {
        return false;
    }
    is_cacheable
}

/// Calculating Age
/// <https://tools.ietf.org/html/rfc7234#section-4.2.3>
fn calculate_response_age(response: &Response) -> Duration {
    // TODO: follow the spec more closely (Date headers, request/response lag, ...)
    if let Some(secs) = response.headers.get_raw("Age") {
        let seconds_string = String::from_utf8_lossy(&secs[0]);
        if let Ok(secs) = seconds_string.parse::<i64>() {
            return Duration::seconds(secs);
        }
    }
    Duration::seconds(0i64)
}

/// Determine the expiry date from relevant headers,
/// or uses a heuristic if none are present.
fn get_response_expiry(response: &Response) -> Duration {
    // Calculating Freshness Lifetime <https://tools.ietf.org/html/rfc7234#section-4.2.1>
    let age = calculate_response_age(&response);
    if let Some(&header::CacheControl(ref directives)) = response.headers.get::<header::CacheControl>() {
        let has_no_cache_directive = directives.iter().any(|directive| {
            header::CacheDirective::NoCache == *directive
        });
        if has_no_cache_directive {
            // Requires validation on first use.
            return Duration::seconds(0i64);
        } else {
            for directive in directives {
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
    // Calculating Heuristic Freshness
    // <https://tools.ietf.org/html/rfc7234#section-4.2.2>
    if let Some((ref code, _)) = response.raw_status {
        // <https://tools.ietf.org/html/rfc7234#section-5.5.4>
        // Since presently we do not generate a Warning header field with a 113 warn-code,
        // 24 hours minus response age is the max for heuristic calculation.
        let max_heuristic = Duration::hours(24) - age;
        let heuristic_freshness = if let Some(&header::LastModified(header::HttpDate(t))) =
            // If the response has a Last-Modified header field,
            // caches are encouraged to use a heuristic expiration value
            // that is no more than some fraction of the interval since that time.
            response.headers.get::<header::LastModified>() {
            let last_modified = t.to_timespec();
            let current = time::now().to_timespec();
            // A typical setting of this fraction might be 10%.
            let raw_heuristic_calc = (current - last_modified) / 10;
            let result = if raw_heuristic_calc < max_heuristic {
                raw_heuristic_calc
            } else {
                max_heuristic
            };
            result
        } else {
            max_heuristic
        };
        match *code {
            200 | 203 | 204 | 206 | 300 | 301 | 404 | 405 | 410 | 414 | 501 => {
                // Status codes that are cacheable by default <https://tools.ietf.org/html/rfc7231#section-6.1>
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

/// Request Cache-Control Directives
/// <https://tools.ietf.org/html/rfc7234#section-5.2.1>
fn get_expiry_adjustment_from_request_headers(request: &Request, expires: Duration) -> Duration {
    let directive_data = match request.headers.get_raw("cache-control") {
        Some(data) => data,
        None => return expires,
    };
    let directives_string = String::from_utf8_lossy(&directive_data[0]);
    for directive in directives_string.split(",") {
        let mut directive_info = directive.split("=");
        match (directive_info.next(), directive_info.next()) {
            (Some("max-stale"), Some(sec_str)) => {
                if let Ok(secs) = sec_str.parse::<i64>() {
                    return expires + Duration::seconds(secs);
                }
            },
            (Some("max-age"), Some(sec_str)) => {
                if let Ok(secs) = sec_str.parse::<i64>() {
                    let max_age = Duration::seconds(secs);
                    if expires > max_age {
                        return Duration::min_value();
                    }
                    return expires - max_age;
                }
            },
            (Some("min-fresh"), Some(sec_str)) => {
                if let Ok(secs) = sec_str.parse::<i64>() {
                    let min_fresh = Duration::seconds(secs);
                    if expires < min_fresh {
                        return Duration::min_value();
                    }
                    return expires - min_fresh;
                }
            },
            (Some("no-cache"), _) | (Some("no-store"), _) => return Duration::min_value(),
            _ => {}
        }
    }
    expires
}

/// Create a CachedResponse from a request and a CachedResource.
fn create_cached_response(request: &Request,
    cached_resource: &CachedResource,
    cached_headers: &Headers,
    done_chan: &mut DoneChannel)
    -> CachedResponse {
    let mut response = Response::new(cached_resource.metadata.final_url.clone());
    response.headers = cached_headers.clone();
    response.body = cached_resource.body.clone();
    if let ResponseBody::Receiving(_) = *cached_resource.body.lock().unwrap() {
        let (done_sender, done_receiver) = channel();
        *done_chan = Some((done_sender.clone(), done_receiver));
        cached_resource.awaiting_body.lock().unwrap().push(done_sender);
    }
    response.location_url = cached_resource.location_url.clone();
    response.status = cached_resource.status.clone();
    response.raw_status = cached_resource.raw_status.clone();
    response.url_list = cached_resource.url_list.clone();
    response.https_state = cached_resource.https_state.clone();
    response.referrer = request.referrer.to_url().cloned();
    response.referrer_policy = request.referrer_policy.clone();
    response.aborted = cached_resource.aborted.clone();
    let expires = cached_resource.expires;
    let adjusted_expires = get_expiry_adjustment_from_request_headers(request, expires);
    let now = Duration::seconds(time::now().to_timespec().sec);
    let last_validated = Duration::seconds(cached_resource.last_validated.to_timespec().sec);
    let time_since_validated = now - last_validated;
    // TODO: take must-revalidate into account <https://tools.ietf.org/html/rfc7234#section-5.2.2.1>
    // TODO: if this cache is to be considered shared, take proxy-revalidate into account
    // <https://tools.ietf.org/html/rfc7234#section-5.2.2.7>
    let has_expired = (adjusted_expires < time_since_validated) ||
        (adjusted_expires == time_since_validated);
    CachedResponse { response: response, needs_validation: has_expired }
}

/// Create a new resource, based on the bytes requested, and an existing resource,
/// with a status-code of 206.
fn create_resource_with_bytes_from_resource(bytes: &[u8], resource: &CachedResource)
    -> CachedResource {
    CachedResource {
        metadata: resource.metadata.clone(),
        request_headers: resource.request_headers.clone(),
        body: Arc::new(Mutex::new(ResponseBody::Done(bytes.to_owned()))),
        location_url: resource.location_url.clone(),
        https_state: resource.https_state.clone(),
        status: Some(StatusCode::PartialContent),
        raw_status: Some((206, b"Partial Content".to_vec())),
        url_list: resource.url_list.clone(),
        expires: resource.expires.clone(),
        last_validated: resource.last_validated.clone(),
        aborted: Arc::new(AtomicBool::new(false)),
        awaiting_body: Arc::new(Mutex::new(vec![]))
    }
}

/// Support for range requests <https://tools.ietf.org/html/rfc7233>.
fn handle_range_request(request: &Request,
    candidates: Vec<&CachedResource>,
    range_spec: &[header::ByteRangeSpec],
    done_chan: &mut DoneChannel)
    -> Option<CachedResponse> {
    let mut complete_cached_resources = candidates.iter().filter(|resource| {
        match resource.raw_status {
            Some((ref code, _)) => *code == 200,
            None => false
        }
    });
    let partial_cached_resources = candidates.iter().filter(|resource| {
        match resource.raw_status {
            Some((ref code, _)) => *code == 206,
            None => false
        }
    });
    match (range_spec.first().unwrap(), complete_cached_resources.next()) {
        // TODO: take the full range spec into account.
        // If we have a complete resource, take the request range from the body.
        // When there isn't a complete resource available, we loop over cached partials,
        // and see if any individual partial response can fulfill the current request for a bytes range.
        // TODO: combine partials that in combination could satisfy the requested range?
        // see <https://tools.ietf.org/html/rfc7233#section-4.3>.
        // TODO: add support for complete and partial resources,
        // whose body is in the ResponseBody::Receiving state.
        (&header::ByteRangeSpec::FromTo(beginning, end), Some(ref complete_resource)) => {
            if let ResponseBody::Done(ref body) = *complete_resource.body.lock().unwrap() {
                let b = beginning as usize;
                let e = end as usize + 1;
                let requested = body.get(b..e);
                if let Some(bytes) = requested {
                    let new_resource = create_resource_with_bytes_from_resource(bytes, complete_resource);
                    let cached_headers = new_resource.metadata.headers.lock().unwrap();
                    let cached_response = create_cached_response(request, &new_resource, &*cached_headers, done_chan);
                    return Some(cached_response);
                }
            }
        },
        (&header::ByteRangeSpec::FromTo(beginning, end), None) => {
            for partial_resource in partial_cached_resources {
                let headers = partial_resource.metadata.headers.lock().unwrap();
                let content_range = headers.get::<header::ContentRange>();
                let (res_beginning, res_end) = match content_range {
                    Some(&header::ContentRange(
                        header::ContentRangeSpec::Bytes {
                            range: Some((res_beginning, res_end)), .. })) => (res_beginning, res_end),
                    _ => continue,
                };
                if res_beginning - 1 < beginning && res_end + 1 > end {
                    let resource_body = &*partial_resource.body.lock().unwrap();
                    let requested = match resource_body {
                        &ResponseBody::Done(ref body) => {
                            let b = beginning as usize - res_beginning as usize;
                            let e = end as usize - res_beginning as usize + 1;
                            body.get(b..e)
                        },
                        _ => continue,
                    };
                    if let Some(bytes) = requested {
                        let new_resource = create_resource_with_bytes_from_resource(&bytes, partial_resource);
                        let cached_response = create_cached_response(request, &new_resource, &*headers, done_chan);
                        return Some(cached_response);
                    }
                }
            }
        },
        (&header::ByteRangeSpec::AllFrom(beginning), Some(ref complete_resource)) => {
            if let ResponseBody::Done(ref body) = *complete_resource.body.lock().unwrap() {
                let b = beginning as usize;
                let requested = body.get(b..);
                if let Some(bytes) = requested {
                    let new_resource = create_resource_with_bytes_from_resource(bytes, complete_resource);
                    let cached_headers = new_resource.metadata.headers.lock().unwrap();
                    let cached_response = create_cached_response(request, &new_resource, &*cached_headers, done_chan);
                    return Some(cached_response);
                }
            }
        },
        (&header::ByteRangeSpec::AllFrom(beginning), None) => {
            for partial_resource in partial_cached_resources {
                let headers = partial_resource.metadata.headers.lock().unwrap();
                let content_range = headers.get::<header::ContentRange>();
                let (res_beginning, res_end, total) = match content_range {
                    Some(&header::ContentRange(
                        header::ContentRangeSpec::Bytes {
                            range: Some((res_beginning, res_end)),
                            instance_length: Some(total) })) => (res_beginning, res_end, total),
                    _ => continue,
                };
                if res_beginning < beginning && res_end == total - 1 {
                    let resource_body = &*partial_resource.body.lock().unwrap();
                    let requested = match resource_body {
                        &ResponseBody::Done(ref body) => {
                            let from_byte = beginning as usize - res_beginning as usize;
                            body.get(from_byte..)
                        },
                        _ => continue,
                    };
                    if let Some(bytes) = requested {
                        let new_resource = create_resource_with_bytes_from_resource(&bytes, partial_resource);
                        let cached_response = create_cached_response(request, &new_resource, &*headers, done_chan);
                        return Some(cached_response);
                    }
                }
            }
        },
        (&header::ByteRangeSpec::Last(offset), Some(ref complete_resource)) => {
            if let ResponseBody::Done(ref body) = *complete_resource.body.lock().unwrap() {
                let from_byte = body.len() - offset as usize;
                let requested = body.get(from_byte..);
                if let Some(bytes) = requested {
                    let new_resource = create_resource_with_bytes_from_resource(bytes, complete_resource);
                    let cached_headers = new_resource.metadata.headers.lock().unwrap();
                    let cached_response = create_cached_response(request, &new_resource, &*cached_headers, done_chan);
                    return Some(cached_response);
                }
            }
        },
        (&header::ByteRangeSpec::Last(offset), None) => {
            for partial_resource in partial_cached_resources {
                let headers = partial_resource.metadata.headers.lock().unwrap();
                let content_range = headers.get::<header::ContentRange>();
                let (res_beginning, res_end, total) = match content_range {
                    Some(&header::ContentRange(
                        header::ContentRangeSpec::Bytes {
                            range: Some((res_beginning, res_end)),
                            instance_length: Some(total) })) => (res_beginning, res_end, total),
                    _ => continue,
                };
                if (total - res_beginning) > (offset - 1 ) && (total - res_end) < offset + 1 {
                    let resource_body = &*partial_resource.body.lock().unwrap();
                    let requested = match resource_body {
                        &ResponseBody::Done(ref body) => {
                            let from_byte = body.len() - offset as usize;
                            body.get(from_byte..)
                        },
                        _ => continue,
                    };
                    if let Some(bytes) = requested {
                        let new_resource = create_resource_with_bytes_from_resource(&bytes, partial_resource);
                        let cached_response = create_cached_response(request, &new_resource, &*headers, done_chan);
                        return Some(cached_response);
                    }
                }
            }
        }
    }
    None
}


impl HttpCache {
    /// Create a new memory cache instance.
    pub fn new() -> HttpCache {
        HttpCache {
            entries: HashMap::new()
        }
    }

    /// Constructing Responses from Caches.
    /// <https://tools.ietf.org/html/rfc7234#section-4>
    pub fn construct_response(&self, request: &Request, done_chan: &mut DoneChannel) -> Option<CachedResponse> {
        // TODO: generate warning headers as appropriate <https://tools.ietf.org/html/rfc7234#section-5.5>
        if request.method != Method::Get {
            // Only Get requests are cached, avoid a url based match for others.
            return None;
        }
        let entry_key = CacheKey::new(request.clone());
        let resources = self.entries.get(&entry_key)?.into_iter().filter(|r| { !r.aborted.load(Ordering::Relaxed) });
        let mut candidates = vec![];
        for cached_resource in resources {
            let mut can_be_constructed = true;
            let cached_headers = cached_resource.metadata.headers.lock().unwrap();
            let original_request_headers = cached_resource.request_headers.lock().unwrap();
            if let Some(vary_data) = cached_headers.get_raw("Vary") {
                // Calculating Secondary Keys with Vary <https://tools.ietf.org/html/rfc7234#section-4.1>
                let vary_data_string = String::from_utf8_lossy(&vary_data[0]);
                let vary_values = vary_data_string.split(",").map(|val| val.trim());
                for vary_val in vary_values {
                    // For every header name found in the Vary header of the stored response.
                    if vary_val == "*" {
                        // A Vary header field-value of "*" always fails to match.
                        can_be_constructed = false;
                        break;
                    }
                    match request.headers.get_raw(vary_val) {
                        Some(header_data) => {
                            // If the header is present in the request.
                            let request_header_data_string = String::from_utf8_lossy(&header_data[0]);
                            if let Some(original_header_data) = original_request_headers.get_raw(vary_val) {
                                // Check that the value of the nominated header field,
                                // in the original request, matches the value in the current request.
                                let original_request_header_data_string =
                                    String::from_utf8_lossy(&original_header_data[0]);
                                if original_request_header_data_string != request_header_data_string {
                                    can_be_constructed = false;
                                    break;
                                }
                            }
                        },
                        None => {
                            // If a header field is absent from a request,
                            // it can only match a stored response if those headers,
                            // were also absent in the original request.
                            can_be_constructed = original_request_headers.get_raw(vary_val).is_none();
                        },
                    }
                    if !can_be_constructed {
                        break;
                    }
                }
            }
            if can_be_constructed {
                candidates.push(cached_resource);
            }
        }
        // Support for range requests
        if let Some(&header::Range::Bytes(ref range_spec)) = request.headers.get::<header::Range>() {
            return handle_range_request(request, candidates, &range_spec, done_chan);
        } else {
            // Not a Range request.
            if let Some(ref cached_resource) = candidates.first() {
                // Returning the first response that can be constructed
                // TODO: select the most appropriate one, using a known mechanism from a selecting header field,
                // or using the Date header to return the most recent one.
                let cached_headers = cached_resource.metadata.headers.lock().unwrap();
                let cached_response = create_cached_response(request, cached_resource, &*cached_headers, done_chan);
                return Some(cached_response);
            }
        }
        None
    }

    /// Updating consumers who received a response constructed with a ResponseBody::Receiving.
    pub fn update_awaiting_consumers(&mut self, request: &Request, response: &Response) {
        if let ResponseBody::Done(ref completed_body) = *response.body.lock().unwrap() {
            let entry_key = CacheKey::new(request.clone());
            if let Some(cached_resources) = self.entries.get(&entry_key) {
                for cached_resource in cached_resources.iter() {
                    let mut awaiting_consumers = cached_resource.awaiting_body.lock().unwrap();
                    for done_sender in awaiting_consumers.drain(..) {
                        if cached_resource.aborted.load(Ordering::Relaxed) {
                            let _ = done_sender.send(Data::Cancelled);
                        } else {
                            let _ = done_sender.send(Data::Payload(completed_body.clone()));
                            let _ = done_sender.send(Data::Done);
                        }
                    };
                }
            }
        }
    }

    /// Freshening Stored Responses upon Validation.
    /// <https://tools.ietf.org/html/rfc7234#section-4.3.4>
    pub fn refresh(&mut self, request: &Request, response: Response, done_chan: &mut DoneChannel) -> Option<Response> {
        assert!(response.status == Some(StatusCode::NotModified));
        let entry_key = CacheKey::new(request.clone());
        if let Some(cached_resources) = self.entries.get_mut(&entry_key) {
            for cached_resource in cached_resources.iter_mut() {
                let mut stored_headers = cached_resource.metadata.headers.lock().unwrap();
                // Received a response with 304 status code, in response to a request that matches a cached resource.
                // 1. update the headers of the cached resource.
                // 2. return a response, constructed from the cached resource.
                stored_headers.extend(response.headers.iter());
                let mut constructed_response = Response::new(cached_resource.metadata.final_url.clone());
                constructed_response.headers = stored_headers.clone();
                constructed_response.body = cached_resource.body.clone();
                constructed_response.status = cached_resource.status.clone();
                constructed_response.https_state = cached_resource.https_state.clone();
                constructed_response.referrer = request.referrer.to_url().cloned();
                constructed_response.referrer_policy = request.referrer_policy.clone();
                constructed_response.raw_status = cached_resource.raw_status.clone();
                constructed_response.url_list = cached_resource.url_list.clone();
                // done_chan will have been set to Some by http_network_fetch,
                // set it back to None since the response returned here replaces the 304 one from the network.
                *done_chan = None;
                cached_resource.expires = get_response_expiry(&constructed_response);
                return Some(constructed_response);
            }
        }
        None
    }

    fn invalidate_for_url(&mut self, url: &ServoUrl) {
        let entry_key = CacheKey::from_servo_url(url);
        if let Some(cached_resources) = self.entries.get_mut(&entry_key) {
            for cached_resource in cached_resources.iter_mut() {
                cached_resource.expires = Duration::seconds(0i64);
            }
        }
    }

    /// Invalidation.
    /// <https://tools.ietf.org/html/rfc7234#section-4.4>
    pub fn invalidate(&mut self, request: &Request, response: &Response) {
        if let Some(&header::Location(ref location)) = response.headers.get::<header::Location>() {
            if let Ok(url) = request.current_url().join(location) {
                self.invalidate_for_url(&url);
            }
        }
        // TODO: update hyper to use typed getter.
        if let Some(url_data) = response.headers.get_raw("Content-Location") {
            if let Ok(content_location) = str::from_utf8(&url_data[0]) {
                if let Ok(url) = request.current_url().join(content_location) {
                    self.invalidate_for_url(&url);
                }
            }
        }
        self.invalidate_for_url(&request.url());
    }

    /// Storing Responses in Caches.
    /// <https://tools.ietf.org/html/rfc7234#section-3>
    pub fn store(&mut self, request: &Request, response: &Response) {
        if PREFS.get("network.http-cache.disabled").as_boolean().unwrap_or(false) {
            return
        }
        if request.method != Method::Get {
            // Only Get requests are cached.
            return
        }
        let entry_key = CacheKey::new(request.clone());
        let metadata = match response.metadata() {
            Ok(FetchMetadata::Filtered {
               filtered: _,
               unsafe_: metadata }) |
            Ok(FetchMetadata::Unfiltered(metadata)) => metadata,
            _ => return,
        };
        if !response_is_cacheable(&metadata) {
            return;
        }
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
            request_headers: Arc::new(Mutex::new(request.headers.clone())),
            body: response.body.clone(),
            location_url: response.location_url.clone(),
            https_state: response.https_state.clone(),
            status: response.status.clone(),
            raw_status: response.raw_status.clone(),
            url_list: response.url_list.clone(),
            expires: expiry,
            last_validated: time::now(),
            aborted: response.aborted.clone(),
            awaiting_body: Arc::new(Mutex::new(vec![]))
        };
        let entry = self.entries.entry(entry_key).or_insert(vec![]);
        entry.push(entry_resource);
    }

}
