/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

#![deny(missing_docs)]

//! A memory cache Implements the logic specified in http://tools.ietf.org/html/rfc7234
//! and http://tools.ietf.org/html/rfc7232.

use fetch::methods::DoneChannel;
use http_loader::is_redirect_status;
use hyper::header;
use hyper::header::ContentType;
use hyper::header::Headers;
use hyper::method::Method;
use hyper::status::StatusCode;
use hyper_serde::Serde;
use net_traits::{Metadata, FetchMetadata, FilteredMetadata, ReferrerPolicy};
use net_traits::request::Request;
use net_traits::response::{HttpsState, Response, ResponseBody};
use servo_url::ServoUrl;
use std::collections::HashMap;
use std::str;
use std::str::FromStr;
use std::sync::{Arc, Mutex};
use time;
use time::{Duration, Tm};
use url::Url;



/// The key used to differentiate requests in the cache.
#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub struct CacheKey {
    url: ServoUrl
}

impl CacheKey {
    fn new(request: Request) -> CacheKey {
        CacheKey {
            url: request.url().clone()
        }
    }

    fn from_url_string(url: &str) -> Result<CacheKey, ()> {
        if let Ok(url) = Url::parse(&url) {
            let key = CacheKey {
                url: ServoUrl::from_url(url)
            };
            return Ok(key);
        }
        return Err(());
    }

    /// Retrieve the URL associated with this key
    pub fn url(&self) -> ServoUrl {
        self.url.clone()
    }
}

/// A complete cached resource.
#[derive(Clone, Debug)]
struct CachedResource {
    metadata: CachedMetadata,
    request_headers: Vec<(String, String)>,
    body: Arc<Mutex<ResponseBody>>,
    https_state: HttpsState,
    referrer: Option<ServoUrl>,
    referrer_policy: Option<ReferrerPolicy>,
    status: Option<StatusCode>,
    raw_status: Option<(u16, Vec<u8>)>,
    url_list: Vec<ServoUrl>,
    expires: Duration,
    last_validated: Tm
}

/// Metadata about a loaded resource, such as is obtained from HTTP headers.
#[derive(Clone, Debug)]
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
/// Based on http://tools.ietf.org/html/rfc7234#section-5
fn response_is_cacheable(metadata: &Metadata) -> bool {
    // Note: this cache should be treated as shared for now.
    // TODO: check for absence of private response directive https://tools.ietf.org/html/rfc7234#section-5.2.2.6
    // TODO: check for absence of the Authorization header field.
    // TODO: check that the response either:
    // *  contains an Expires header field (see Section 5.3), or
    // *  contains a max-age response directive (see Section 5.2.2.8), or
    // *  contains a s-maxage response directive (see Section 5.2.2.9) and the cache is shared, or
    // *  contains a Cache Control Extension (see Section 5.2.3) that allows it to be cached, or
    // *  has a status code that is defined as cacheable by default (see Section 4.2.2), or
    // *  contains a public response directive (see Section 5.2.2.5).
    // TODO write a new http-cache/shared_cache.html wpt test suite for the above.
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
        let seconds_string = String::from_utf8_lossy(&secs[0]);
        if let Ok(secs) = seconds_string.parse::<i64>() {
            return Duration::seconds(secs);
        }
    }
    Duration::seconds(0i64)
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
fn get_expiry_adjustment_from_request_headers(request: &Request, expires: Duration) -> Duration {
    let directive_data = match request.headers.get_raw("cache-control") {
        Some(data) => data,
        None => return expires,
    };
    let directives_string = String::from_utf8_lossy(&directive_data[0]);
    for directive in directives_string.split(",") {
        let mut directive_info = directive.split("=");
        match (directive_info.nth(0), directive_info.nth(1)) {
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
fn create_cached_response(request: &Request, cached_resource: &CachedResource, cached_headers: &Headers)
    -> CachedResponse {
    let mut response = Response::new(cached_resource.metadata.final_url.clone());
    response.headers = cached_headers.clone();
    response.body = cached_resource.body.clone();
    response.status = cached_resource.status.clone();
    response.raw_status = cached_resource.raw_status.clone();
    response.url_list = cached_resource.url_list.clone();
    response.https_state = cached_resource.https_state.clone();
    response.referrer = cached_resource.referrer.clone();
    response.referrer_policy = cached_resource.referrer_policy.clone();
    let expires = cached_resource.expires;
    let adjusted_expires = get_expiry_adjustment_from_request_headers(request, expires);
    let now = Duration::seconds(time::now().to_timespec().sec);
    let last_validated = Duration::seconds(cached_resource.last_validated.to_timespec().sec);
    let time_since_validated = now - last_validated;
    // TODO: take must-revalidate into account https://tools.ietf.org/html/rfc7234#section-5.2.2.1
    // TODO: since this cache is shared, taking proxy-revalidate into account
    // https://tools.ietf.org/html/rfc7234#section-5.2.2.7
    let has_expired = (adjusted_expires < time_since_validated) ||
        (adjusted_expires == time_since_validated);
    CachedResponse { response: response, needs_validation: has_expired }
}


impl HttpCache {
    /// Create a new memory cache instance.
    pub fn new() -> HttpCache {
        HttpCache {
            entries: HashMap::new()
        }
    }

    /// https://tools.ietf.org/html/rfc7234#section-4 Constructing Responses from Caches.
    pub fn construct_response(&self, request: &Request) -> Option<CachedResponse> {
        // TODO: generate warning headers as appropriate https://tools.ietf.org/html/rfc7234#section-5.5
       if request.method != Method::Get {
            // Only Get requests are cached, avoid a url based match for others.
            return None;
        }
        let entry_key = CacheKey::new(request.clone());
        let resources = match self.entries.get(&entry_key) {
            Some(ref resources) => resources.clone(),
            None => return None,
        };
        for cached_resource in resources.iter() {
            let mut can_be_constructed = true;
            let stored_headers = cached_resource.metadata.headers.lock().unwrap();
            if let Some(vary_data) = stored_headers.get_raw("Vary") {
                // Calculating Secondary Keys with Vary https://tools.ietf.org/html/rfc7234#section-4.1
                let vary_data_string = String::from_utf8_lossy(&vary_data[0]);
                let vary_values = vary_data_string.split(",").map(|val| val.trim());
                for vary_val in vary_values {
                    // For every header name found in the Vary header field in the stored response.
                    if vary_val == "*" {
                        // A Vary header field-value of "*" always fails to match.
                        can_be_constructed = false;
                        break;
                    }
                    match request.headers.get_raw(vary_val) {
                        Some(header_data) => {
                            // If the header is present in the request.
                            let request_header_data_string = String::from_utf8_lossy(&header_data[0]);
                            for &(ref name, ref value) in cached_resource.request_headers.iter() {
                                if name.to_lowercase() == vary_val.to_lowercase() {
                                    // Check that the value of the nominated header field,
                                    // in the original request, matches the value in the current request.
                                    let original_vary_values = value.split(",");
                                    let request_vary_values = request_header_data_string.split(",");
                                    if !original_vary_values.eq(request_vary_values) {
                                        can_be_constructed = false;
                                        break;
                                    }
                                }
                            }
                        },
                        None => {
                            // If a header field is absent from a request,
                            // it can only match another request if it is also absent there.
                            can_be_constructed = cached_resource.request_headers.iter().all(|&(ref name, _)| {
                                name.to_lowercase() != vary_val.to_lowercase()
                            });
                        },
                    }
                    if !can_be_constructed {
                        break;
                    }
                }
            }
            if can_be_constructed {
                // Returning the first response that can be constructed
                // TODO: select the most appropriate one, using a known mechanism from a selecting header field,
                // or using the Date header to return the most recent one.
                let cached_response = create_cached_response(request, cached_resource, &*stored_headers);
                return Some(cached_response);
            }
        }
        None
    }

    /// https://tools.ietf.org/html/rfc7234#section-4.3.4 Freshening Stored Responses upon Validation.
    pub fn refresh(&mut self, request: &Request, response: Response, done_chan: &mut DoneChannel) -> Option<Response> {
        assert!(response.status == Some(StatusCode::NotModified));
        let entry_key = CacheKey::new(request.clone());
        if let Some(cached_resources) = self.entries.get_mut(&entry_key) {
            for cached_resource in cached_resources.iter_mut() {
                if let Ok(ref mut stored_headers) = cached_resource.metadata.headers.try_lock() {
                    // Received a response with 304 status code whose url matches a stored resource.
                    // 1. update the headers of the stored resource.
                    // 2. return a response, constructed from the updated stored resource.
                    stored_headers.extend(response.headers.iter());
                    let mut constructed_response = Response::new(cached_resource.metadata.final_url.clone());
                    constructed_response.headers = stored_headers.clone();
                    constructed_response.body = cached_resource.body.clone();
                    constructed_response.status = cached_resource.status.clone();
                    constructed_response.https_state = cached_resource.https_state.clone();
                    constructed_response.referrer = cached_resource.referrer.clone();
                    constructed_response.referrer_policy = cached_resource.referrer_policy.clone();
                    constructed_response.raw_status = cached_resource.raw_status.clone();
                    constructed_response.url_list = cached_resource.url_list.clone();
                    // done_chan will have been set to Some by http_network_fetch,
                    // set it back to None since the response returned here replaces the 304 one from the network.
                    *done_chan = None;
                    cached_resource.expires = get_response_expiry(&constructed_response);
                    return Some(constructed_response);
                }
            }
        }
        None
    }

    fn invalidate_for_url(&mut self, url: &str) {
        if let Ok(entry_key) = CacheKey::from_url_string(url) {
            if let Some(cached_resources) = self.entries.get_mut(&entry_key) {
                for cached_resource in cached_resources.iter_mut() {
                    cached_resource.expires = Duration::seconds(0i64);
                }
            }
        }
    }

    /// https://tools.ietf.org/html/rfc7234#section-4.4 Invalidation.
    pub fn invalidate(&mut self, request: &Request, response: &Response) {
        if let Some(&header::Location(ref location)) = response.headers.get::<header::Location>() {
            self.invalidate_for_url(location);
        }
        // TODO: update hyper to use typed getter.
        if let Some(url_data) = response.headers.get_raw("Content-Location") {
            if let Ok(content_location) = str::from_utf8(&url_data[0]) {
                self.invalidate_for_url(content_location);
            }
        }
        self.invalidate_for_url(&request.url().as_str());
    }

    /// https://tools.ietf.org/html/rfc7234#section-3 Storing Responses in Caches.
    pub fn store(&mut self, request: &Request, response: &Response) {
        let entry_key = CacheKey::new(request.clone());
        if request.method != Method::Get {
             // For simplicity, only cache Get requests https://tools.ietf.org/html/rfc7234#section-2
             return
        }
        if let Some(status) = response.status {
            // Not caching redirects.
            if is_redirect_status(status) {
                return
            }
        }
        if let Some((ref code, _)) = response.raw_status {
            // Not caching Not Modified.
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
                    let request_headers = request.headers
                        .iter()
                        .map(|header| (String::from_str(header.name()).unwrap_or(String::from("None")),
                                                                  header.value_string()))
                        .collect();
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
                        request_headers: request_headers,
                        body: response.body.clone(),
                        https_state: response.https_state.clone(),
                        referrer: response.referrer.clone(),
                        referrer_policy: response.referrer_policy.clone(),
                        status: response.status.clone(),
                        raw_status: response.raw_status.clone(),
                        url_list: response.url_list.clone(),
                        expires: expiry,
                        last_validated: time::now()
                    };
                    let entry = self.entries.entry(entry_key).or_insert(vec![]);
                    entry.push(entry_resource);
                }
            },
            _ => {}
        }
    }

}
