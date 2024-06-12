/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

#![deny(missing_docs)]

//! A memory cache implementing the logic specified in <http://tools.ietf.org/html/rfc7234>
//! and <http://tools.ietf.org/html/rfc7232>.

use std::collections::HashMap;
use std::ops::Bound;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Mutex;
use std::time::SystemTime;

use headers::{
    CacheControl, ContentRange, Expires, HeaderMapExt, LastModified, Pragma, Range, Vary,
};
use http::header::HeaderValue;
use http::{header, HeaderMap, Method, StatusCode};
use log::debug;
use malloc_size_of::{
    MallocSizeOf, MallocSizeOfOps, MallocUnconditionalShallowSizeOf, MallocUnconditionalSizeOf,
    Measurable,
};
use malloc_size_of_derive::MallocSizeOf;
use net_traits::request::Request;
use net_traits::response::{HttpsState, Response, ResponseBody};
use net_traits::{FetchMetadata, Metadata, ResourceFetchTiming};
use servo_arc::Arc;
use servo_config::pref;
use servo_url::ServoUrl;
use time::{Duration, Timespec, Tm};
use tokio::sync::mpsc::{unbounded_channel as unbounded, UnboundedSender as TokioSender};

use crate::fetch::methods::{Data, DoneChannel};

/// The key used to differentiate requests in the cache.
#[derive(Clone, Eq, Hash, MallocSizeOf, PartialEq)]
pub struct CacheKey {
    url: ServoUrl,
}

impl CacheKey {
    /// Create a cache-key from a request.
    pub(crate) fn new(request: &Request) -> CacheKey {
        CacheKey {
            url: request.current_url(),
        }
    }

    fn from_servo_url(servo_url: &ServoUrl) -> CacheKey {
        CacheKey {
            url: servo_url.clone(),
        }
    }
}

/// A complete cached resource.
#[derive(Clone)]
struct CachedResource {
    request_headers: Arc<Mutex<HeaderMap>>,
    body: Arc<Mutex<ResponseBody>>,
    aborted: Arc<AtomicBool>,
    awaiting_body: Arc<Mutex<Vec<TokioSender<Data>>>>,
    data: Measurable<MeasurableCachedResource>,
}

#[derive(Clone, MallocSizeOf)]
struct MeasurableCachedResource {
    metadata: CachedMetadata,
    location_url: Option<Result<ServoUrl, String>>,
    https_state: HttpsState,
    status: Option<(StatusCode, String)>,
    raw_status: Option<(u16, Vec<u8>)>,
    url_list: Vec<ServoUrl>,
    expires: Duration,
    last_validated: Tm,
}

impl MallocSizeOf for CachedResource {
    fn size_of(&self, ops: &mut MallocSizeOfOps) -> usize {
        // TODO: self.request_headers.unconditional_size_of(ops) +
        self.body.unconditional_size_of(ops) +
            self.aborted.unconditional_size_of(ops) +
            self.awaiting_body.unconditional_size_of(ops) +
            self.data.size_of(ops)
    }
}

/// Metadata about a loaded resource, such as is obtained from HTTP headers.
#[derive(Clone)]
struct CachedMetadata {
    /// Headers
    pub headers: Arc<Mutex<HeaderMap>>,
    /// Fields that implement MallocSizeOf
    pub data: Measurable<MeasurableCachedMetadata>,
}

#[derive(Clone, MallocSizeOf)]
struct MeasurableCachedMetadata {
    /// Final URL after redirects.
    pub final_url: ServoUrl,
    /// MIME type / subtype.
    pub content_type: Option<String>,
    /// Character set.
    pub charset: Option<String>,
    /// HTTP Status
    pub status: Option<(u16, Vec<u8>)>,
}

impl MallocSizeOf for CachedMetadata {
    fn size_of(&self, ops: &mut MallocSizeOfOps) -> usize {
        self.headers.unconditional_shallow_size_of(ops) +
        // TODO: self.headers.size_of(ops) +
        self.data.size_of(ops)
    }
}

/// Wrapper around a cached response, including information on re-validation needs
pub struct CachedResponse {
    /// The response constructed from the cached resource
    pub response: Response,
    /// The revalidation flag for the stored response
    pub needs_validation: bool,
}

/// A memory cache.
#[derive(Default, MallocSizeOf)]
pub struct HttpCache {
    /// cached responses.
    entries: HashMap<CacheKey, Vec<CachedResource>>,
}

/// Determine if a response is cacheable by default <https://tools.ietf.org/html/rfc7231#section-6.1>
fn is_cacheable_by_default(status_code: u16) -> bool {
    matches!(
        status_code,
        200 | 203 | 204 | 206 | 300 | 301 | 404 | 405 | 410 | 414 | 501
    )
}

/// Determine if a given response is cacheable.
/// Based on <https://tools.ietf.org/html/rfc7234#section-3>
fn response_is_cacheable(metadata: &Metadata) -> bool {
    // TODO: if we determine that this cache should be considered shared:
    // 1. check for absence of private response directive <https://tools.ietf.org/html/rfc7234#section-5.2.2.6>
    // 2. check for absence of the Authorization header field.
    let mut is_cacheable = false;
    let headers = metadata.headers.as_ref().unwrap();
    if headers.contains_key(header::EXPIRES) ||
        headers.contains_key(header::LAST_MODIFIED) ||
        headers.contains_key(header::ETAG)
    {
        is_cacheable = true;
    }
    if let Some(ref directive) = headers.typed_get::<CacheControl>() {
        if directive.no_store() {
            return false;
        }
        if directive.public() ||
            directive.s_max_age().is_some() ||
            directive.max_age().is_some() ||
            directive.no_cache()
        {
            is_cacheable = true;
        }
    }
    if let Some(pragma) = headers.typed_get::<Pragma>() {
        if pragma.is_no_cache() {
            return false;
        }
    }
    is_cacheable
}

/// Calculating Age
/// <https://tools.ietf.org/html/rfc7234#section-4.2.3>
fn calculate_response_age(response: &Response) -> Duration {
    // TODO: follow the spec more closely (Date headers, request/response lag, ...)
    if let Some(secs) = response.headers.get(header::AGE) {
        if let Ok(seconds_string) = secs.to_str() {
            if let Ok(secs) = seconds_string.parse::<i64>() {
                return Duration::seconds(secs);
            }
        }
    }
    Duration::seconds(0i64)
}

/// Determine the expiry date from relevant headers,
/// or uses a heuristic if none are present.
fn get_response_expiry(response: &Response) -> Duration {
    // Calculating Freshness Lifetime <https://tools.ietf.org/html/rfc7234#section-4.2.1>
    let age = calculate_response_age(response);
    if let Some(directives) = response.headers.typed_get::<CacheControl>() {
        if directives.no_cache() {
            // Requires validation on first use.
            return Duration::seconds(0i64);
        }
        if let Some(secs) = directives.max_age().or(directives.s_max_age()) {
            let max_age = Duration::from_std(secs).unwrap();
            if max_age < age {
                return Duration::seconds(0i64);
            }
            return max_age - age;
        }
    }
    match response.headers.typed_get::<Expires>() {
        Some(t) => {
            // store the period of time from now until expiry
            let t: SystemTime = t.into();
            let t = t.duration_since(SystemTime::UNIX_EPOCH).unwrap();
            let desired = Timespec::new(t.as_secs() as i64, 0);
            let current = time::now().to_timespec();

            if desired > current {
                return desired - current;
            }
            return Duration::seconds(0i64);
        },
        // Malformed Expires header, shouldn't be used to construct a valid response.
        None if response.headers.contains_key(header::EXPIRES) => return Duration::seconds(0i64),
        _ => {},
    }
    // Calculating Heuristic Freshness
    // <https://tools.ietf.org/html/rfc7234#section-4.2.2>
    if let Some((ref code, _)) = response.raw_status {
        // <https://tools.ietf.org/html/rfc7234#section-5.5.4>
        // Since presently we do not generate a Warning header field with a 113 warn-code,
        // 24 hours minus response age is the max for heuristic calculation.
        let max_heuristic = Duration::hours(24) - age;
        let heuristic_freshness = if let Some(last_modified) =
            // If the response has a Last-Modified header field,
            // caches are encouraged to use a heuristic expiration value
            // that is no more than some fraction of the interval since that time.
            response.headers.typed_get::<LastModified>()
        {
            let current = time::now().to_timespec();
            let last_modified: SystemTime = last_modified.into();
            let last_modified = last_modified
                .duration_since(SystemTime::UNIX_EPOCH)
                .unwrap();
            let last_modified = Timespec::new(last_modified.as_secs() as i64, 0);
            // A typical setting of this fraction might be 10%.
            let raw_heuristic_calc = (current - last_modified) / 10;

            if raw_heuristic_calc < max_heuristic {
                raw_heuristic_calc
            } else {
                max_heuristic
            }
        } else {
            max_heuristic
        };
        if is_cacheable_by_default(*code) {
            // Status codes that are cacheable by default can use heuristics to determine freshness.
            return heuristic_freshness;
        }
        // Other status codes can only use heuristic freshness if the public cache directive is present.
        if let Some(ref directives) = response.headers.typed_get::<CacheControl>() {
            if directives.public() {
                return heuristic_freshness;
            }
        }
    }
    // Requires validation upon first use as default.
    Duration::seconds(0i64)
}

/// Request Cache-Control Directives
/// <https://tools.ietf.org/html/rfc7234#section-5.2.1>
fn get_expiry_adjustment_from_request_headers(request: &Request, expires: Duration) -> Duration {
    let directive = match request.headers.typed_get::<CacheControl>() {
        Some(data) => data,
        None => return expires,
    };

    if let Some(max_age) = directive.max_stale() {
        return expires + Duration::from_std(max_age).unwrap();
    }
    if let Some(max_age) = directive.max_age() {
        let max_age = Duration::from_std(max_age).unwrap();
        if expires > max_age {
            return Duration::min_value();
        }
        return expires - max_age;
    }
    if let Some(min_fresh) = directive.min_fresh() {
        let min_fresh = Duration::from_std(min_fresh).unwrap();
        if expires < min_fresh {
            return Duration::min_value();
        }
        return expires - min_fresh;
    }
    if directive.no_cache() || directive.no_store() {
        return Duration::min_value();
    }

    expires
}

/// Create a CachedResponse from a request and a CachedResource.
fn create_cached_response(
    request: &Request,
    cached_resource: &CachedResource,
    cached_headers: &HeaderMap,
    done_chan: &mut DoneChannel,
) -> Option<CachedResponse> {
    debug!("creating a cached response for {:?}", request.url());
    if cached_resource.aborted.load(Ordering::Acquire) {
        return None;
    }
    let resource_timing = ResourceFetchTiming::new(request.timing_type());
    let mut response = Response::new(
        cached_resource.data.metadata.data.final_url.clone(),
        resource_timing,
    );
    response.headers = cached_headers.clone();
    response.body = cached_resource.body.clone();
    if let ResponseBody::Receiving(_) = *cached_resource.body.lock().unwrap() {
        debug!("existing body is in progress");
        let (done_sender, done_receiver) = unbounded();
        *done_chan = Some((done_sender.clone(), done_receiver));
        cached_resource
            .awaiting_body
            .lock()
            .unwrap()
            .push(done_sender);
    }
    response
        .location_url
        .clone_from(&cached_resource.data.location_url);
    response.status.clone_from(&cached_resource.data.status);
    response
        .raw_status
        .clone_from(&cached_resource.data.raw_status);
    response.url_list.clone_from(&cached_resource.data.url_list);
    response.https_state = cached_resource.data.https_state;
    response.referrer = request.referrer.to_url().cloned();
    response.referrer_policy = request.referrer_policy;
    response.aborted = cached_resource.aborted.clone();

    let expires = cached_resource.data.expires;
    let adjusted_expires = get_expiry_adjustment_from_request_headers(request, expires);
    let now = Duration::seconds(time::now().to_timespec().sec);
    let last_validated = Duration::seconds(cached_resource.data.last_validated.to_timespec().sec);
    let time_since_validated = now - last_validated;
    // TODO: take must-revalidate into account <https://tools.ietf.org/html/rfc7234#section-5.2.2.1>
    // TODO: if this cache is to be considered shared, take proxy-revalidate into account
    // <https://tools.ietf.org/html/rfc7234#section-5.2.2.7>
    let has_expired = adjusted_expires <= time_since_validated;
    let cached_response = CachedResponse {
        response,
        needs_validation: has_expired,
    };
    Some(cached_response)
}

/// Create a new resource, based on the bytes requested, and an existing resource,
/// with a status-code of 206.
fn create_resource_with_bytes_from_resource(
    bytes: &[u8],
    resource: &CachedResource,
) -> CachedResource {
    CachedResource {
        request_headers: resource.request_headers.clone(),
        body: Arc::new(Mutex::new(ResponseBody::Done(bytes.to_owned()))),
        aborted: Arc::new(AtomicBool::new(false)),
        awaiting_body: Arc::new(Mutex::new(vec![])),
        data: Measurable(MeasurableCachedResource {
            metadata: resource.data.metadata.clone(),
            location_url: resource.data.location_url.clone(),
            https_state: resource.data.https_state,
            status: Some((StatusCode::PARTIAL_CONTENT, "Partial Content".into())),
            raw_status: Some((206, b"Partial Content".to_vec())),
            url_list: resource.data.url_list.clone(),
            expires: resource.data.expires,
            last_validated: resource.data.last_validated,
        }),
    }
}

/// Support for range requests <https://tools.ietf.org/html/rfc7233>.
fn handle_range_request(
    request: &Request,
    candidates: &[&CachedResource],
    range_spec: Vec<(Bound<u64>, Bound<u64>)>,
    done_chan: &mut DoneChannel,
) -> Option<CachedResponse> {
    let mut complete_cached_resources =
        candidates
            .iter()
            .filter(|resource| match resource.data.raw_status {
                Some((ref code, _)) => *code == 200,
                None => false,
            });
    let partial_cached_resources =
        candidates
            .iter()
            .filter(|resource| match resource.data.raw_status {
                Some((ref code, _)) => *code == 206,
                None => false,
            });
    match (
        range_spec.first().unwrap(),
        complete_cached_resources.next(),
    ) {
        // TODO: take the full range spec into account.
        // If we have a complete resource, take the request range from the body.
        // When there isn't a complete resource available, we loop over cached partials,
        // and see if any individual partial response can fulfill the current request for a bytes range.
        // TODO: combine partials that in combination could satisfy the requested range?
        // see <https://tools.ietf.org/html/rfc7233#section-4.3>.
        // TODO: add support for complete and partial resources,
        // whose body is in the ResponseBody::Receiving state.
        (&(Bound::Included(beginning), Bound::Included(end)), Some(complete_resource)) => {
            if let ResponseBody::Done(ref body) = *complete_resource.body.lock().unwrap() {
                if end == u64::max_value() {
                    // Prevent overflow on the addition below.
                    return None;
                }
                let b = beginning as usize;
                let e = end as usize + 1;
                let requested = body.get(b..e);
                if let Some(bytes) = requested {
                    let new_resource =
                        create_resource_with_bytes_from_resource(bytes, complete_resource);
                    let cached_headers = new_resource.data.metadata.headers.lock().unwrap();
                    let cached_response =
                        create_cached_response(request, &new_resource, &cached_headers, done_chan);
                    if let Some(cached_response) = cached_response {
                        return Some(cached_response);
                    }
                }
            }
        },
        (&(Bound::Included(beginning), Bound::Included(end)), None) => {
            for partial_resource in partial_cached_resources {
                let headers = partial_resource.data.metadata.headers.lock().unwrap();
                let content_range = headers.typed_get::<ContentRange>();
                let (res_beginning, res_end) = match content_range {
                    Some(range) => {
                        if let Some(bytes_range) = range.bytes_range() {
                            bytes_range
                        } else {
                            continue;
                        }
                    },
                    _ => continue,
                };
                if res_beginning <= beginning && res_end >= end {
                    let resource_body = &*partial_resource.body.lock().unwrap();
                    let requested = match resource_body {
                        ResponseBody::Done(body) => {
                            let b = beginning as usize - res_beginning as usize;
                            let e = end as usize - res_beginning as usize + 1;
                            body.get(b..e)
                        },
                        _ => continue,
                    };
                    if let Some(bytes) = requested {
                        let new_resource =
                            create_resource_with_bytes_from_resource(bytes, partial_resource);
                        let cached_response =
                            create_cached_response(request, &new_resource, &headers, done_chan);
                        if let Some(cached_response) = cached_response {
                            return Some(cached_response);
                        }
                    }
                }
            }
        },
        (&(Bound::Included(beginning), Bound::Unbounded), Some(complete_resource)) => {
            if let ResponseBody::Done(ref body) = *complete_resource.body.lock().unwrap() {
                let b = beginning as usize;
                let requested = body.get(b..);
                if let Some(bytes) = requested {
                    let new_resource =
                        create_resource_with_bytes_from_resource(bytes, complete_resource);
                    let cached_headers = new_resource.data.metadata.headers.lock().unwrap();
                    let cached_response =
                        create_cached_response(request, &new_resource, &cached_headers, done_chan);
                    if let Some(cached_response) = cached_response {
                        return Some(cached_response);
                    }
                }
            }
        },
        (&(Bound::Included(beginning), Bound::Unbounded), None) => {
            for partial_resource in partial_cached_resources {
                let headers = partial_resource.data.metadata.headers.lock().unwrap();
                let content_range = headers.typed_get::<ContentRange>();
                let (res_beginning, res_end, total) = if let Some(range) = content_range {
                    match (range.bytes_range(), range.bytes_len()) {
                        (Some(bytes_range), Some(total)) => (bytes_range.0, bytes_range.1, total),
                        _ => continue,
                    }
                } else {
                    continue;
                };
                if total == 0 {
                    // Prevent overflow in the below operations from occuring.
                    continue;
                };
                if res_beginning < beginning && res_end == total - 1 {
                    let resource_body = &*partial_resource.body.lock().unwrap();
                    let requested = match resource_body {
                        ResponseBody::Done(body) => {
                            let from_byte = beginning as usize - res_beginning as usize;
                            body.get(from_byte..)
                        },
                        _ => continue,
                    };
                    if let Some(bytes) = requested {
                        let new_resource =
                            create_resource_with_bytes_from_resource(bytes, partial_resource);
                        let cached_response =
                            create_cached_response(request, &new_resource, &headers, done_chan);
                        if let Some(cached_response) = cached_response {
                            return Some(cached_response);
                        }
                    }
                }
            }
        },
        (&(Bound::Unbounded, Bound::Included(offset)), Some(complete_resource)) => {
            if let ResponseBody::Done(ref body) = *complete_resource.body.lock().unwrap() {
                let from_byte = body.len() - offset as usize;
                let requested = body.get(from_byte..);
                if let Some(bytes) = requested {
                    let new_resource =
                        create_resource_with_bytes_from_resource(bytes, complete_resource);
                    let cached_headers = new_resource.data.metadata.headers.lock().unwrap();
                    let cached_response =
                        create_cached_response(request, &new_resource, &cached_headers, done_chan);
                    if let Some(cached_response) = cached_response {
                        return Some(cached_response);
                    }
                }
            }
        },
        (&(Bound::Unbounded, Bound::Included(offset)), None) => {
            for partial_resource in partial_cached_resources {
                let headers = partial_resource.data.metadata.headers.lock().unwrap();
                let content_range = headers.typed_get::<ContentRange>();
                let (res_beginning, res_end, total) = if let Some(range) = content_range {
                    match (range.bytes_range(), range.bytes_len()) {
                        (Some(bytes_range), Some(total)) => (bytes_range.0, bytes_range.1, total),
                        _ => continue,
                    }
                } else {
                    continue;
                };
                if total < res_beginning ||
                    total < res_end ||
                    offset == 0 ||
                    offset == u64::max_value()
                {
                    // Prevent overflow in the below operations from occuring.
                    continue;
                }
                if (total - res_beginning) > (offset - 1) && (total - res_end) < offset + 1 {
                    let resource_body = &*partial_resource.body.lock().unwrap();
                    let requested = match resource_body {
                        ResponseBody::Done(body) => {
                            let from_byte = body.len() - offset as usize;
                            body.get(from_byte..)
                        },
                        _ => continue,
                    };
                    if let Some(bytes) = requested {
                        let new_resource =
                            create_resource_with_bytes_from_resource(bytes, partial_resource);
                        let cached_response =
                            create_cached_response(request, &new_resource, &headers, done_chan);
                        if let Some(cached_response) = cached_response {
                            return Some(cached_response);
                        }
                    }
                }
            }
        },
        // All the cases with Bound::Excluded should be unreachable anyway
        _ => return None,
    }
    None
}

impl HttpCache {
    /// Constructing Responses from Caches.
    /// <https://tools.ietf.org/html/rfc7234#section-4>
    pub fn construct_response(
        &self,
        request: &Request,
        done_chan: &mut DoneChannel,
    ) -> Option<CachedResponse> {
        // TODO: generate warning headers as appropriate <https://tools.ietf.org/html/rfc7234#section-5.5>
        debug!("trying to construct cache response for {:?}", request.url());
        if request.method != Method::GET {
            // Only Get requests are cached, avoid a url based match for others.
            debug!("non-GET method, not caching");
            return None;
        }
        let entry_key = CacheKey::new(request);
        let resources = self
            .entries
            .get(&entry_key)?
            .iter()
            .filter(|r| !r.aborted.load(Ordering::Relaxed));
        let mut candidates = vec![];
        for cached_resource in resources {
            let mut can_be_constructed = true;
            let cached_headers = cached_resource.data.metadata.headers.lock().unwrap();
            let original_request_headers = cached_resource.request_headers.lock().unwrap();
            if let Some(vary_value) = cached_headers.typed_get::<Vary>() {
                if vary_value.is_any() {
                    debug!("vary value is any, not caching");
                    can_be_constructed = false
                } else {
                    // For every header name found in the Vary header of the stored response.
                    // Calculating Secondary Keys with Vary <https://tools.ietf.org/html/rfc7234#section-4.1>
                    for vary_val in vary_value.iter_strs() {
                        match request.headers.get(vary_val) {
                            Some(header_data) => {
                                // If the header is present in the request.
                                if let Some(original_header_data) =
                                    original_request_headers.get(vary_val)
                                {
                                    // Check that the value of the nominated header field,
                                    // in the original request, matches the value in the current request.
                                    if original_header_data != header_data {
                                        debug!("headers don't match, not caching");
                                        can_be_constructed = false;
                                        break;
                                    }
                                }
                            },
                            None => {
                                // If a header field is absent from a request,
                                // it can only match a stored response if those headers,
                                // were also absent in the original request.
                                can_be_constructed =
                                    original_request_headers.get(vary_val).is_none();
                                if !can_be_constructed {
                                    debug!("vary header present, not caching");
                                }
                            },
                        }
                        if !can_be_constructed {
                            break;
                        }
                    }
                }
            }
            if can_be_constructed {
                candidates.push(cached_resource);
            }
        }
        // Support for range requests
        if let Some(range_spec) = request.headers.typed_get::<Range>() {
            return handle_range_request(
                request,
                candidates.as_slice(),
                range_spec.iter().collect(),
                done_chan,
            );
        }
        while let Some(cached_resource) = candidates.pop() {
            // Not a Range request.
            // Do not allow 206 responses to be constructed.
            //
            // See https://tools.ietf.org/html/rfc7234#section-3.1
            //
            // A cache MUST NOT use an incomplete response to answer requests unless the
            // response has been made complete or the request is partial and
            // specifies a range that is wholly within the incomplete response.
            //
            // TODO: Combining partial content to fulfill a non-Range request
            // see https://tools.ietf.org/html/rfc7234#section-3.3
            match cached_resource.data.raw_status {
                Some((ref code, _)) => {
                    if *code == 206 {
                        continue;
                    }
                },
                None => continue,
            }
            // Returning a response that can be constructed
            // TODO: select the most appropriate one, using a known mechanism from a selecting header field,
            // or using the Date header to return the most recent one.
            let cached_headers = cached_resource.data.metadata.headers.lock().unwrap();
            let cached_response =
                create_cached_response(request, cached_resource, &cached_headers, done_chan);
            if let Some(cached_response) = cached_response {
                return Some(cached_response);
            }
        }
        debug!("couldn't find an appropriate response, not caching");
        // The cache wasn't able to construct anything.
        None
    }

    /// Wake-up consumers of cached resources
    /// whose response body was still receiving data when the resource was constructed,
    /// and whose response has now either been completed or cancelled.
    pub fn update_awaiting_consumers(&self, request: &Request, response: &Response) {
        let entry_key = CacheKey::new(request);

        let cached_resources = match self.entries.get(&entry_key) {
            None => return,
            Some(resources) => resources,
        };

        // Ensure we only wake-up consumers of relevant resources,
        // ie we don't want to wake-up 200 awaiting consumers with a 206.
        let relevant_cached_resources = cached_resources.iter().filter(|resource| {
            if response.actual_response().is_network_error() {
                return *resource.body.lock().unwrap() == ResponseBody::Empty;
            }
            resource.data.raw_status == response.raw_status
        });

        for cached_resource in relevant_cached_resources {
            let mut awaiting_consumers = cached_resource.awaiting_body.lock().unwrap();
            if awaiting_consumers.is_empty() {
                continue;
            }
            let to_send = if cached_resource.aborted.load(Ordering::Acquire) {
                // In the case of an aborted fetch,
                // wake-up all awaiting consumers.
                // Each will then start a new network request.
                // TODO: Wake-up only one consumer, and make it the producer on which others wait.
                Data::Cancelled
            } else {
                match *cached_resource.body.lock().unwrap() {
                    ResponseBody::Done(_) | ResponseBody::Empty => Data::Done,
                    ResponseBody::Receiving(_) => {
                        continue;
                    },
                }
            };
            for done_sender in awaiting_consumers.drain(..) {
                let _ = done_sender.send(to_send.clone());
            }
        }
    }

    /// Freshening Stored Responses upon Validation.
    /// <https://tools.ietf.org/html/rfc7234#section-4.3.4>
    pub fn refresh(
        &mut self,
        request: &Request,
        response: Response,
        done_chan: &mut DoneChannel,
    ) -> Option<Response> {
        assert_eq!(response.status.map(|s| s.0), Some(StatusCode::NOT_MODIFIED));
        let entry_key = CacheKey::new(request);
        if let Some(cached_resources) = self.entries.get_mut(&entry_key) {
            if let Some(cached_resource) = cached_resources.iter_mut().next() {
                // done_chan will have been set to Some(..) by http_network_fetch.
                // If the body is not receiving data, set the done_chan back to None.
                // Otherwise, create a new dedicated channel to update the consumer.
                // The response constructed here will replace the 304 one from the network.
                let in_progress_channel = match *cached_resource.body.lock().unwrap() {
                    ResponseBody::Receiving(..) => Some(unbounded()),
                    ResponseBody::Empty | ResponseBody::Done(..) => None,
                };
                match in_progress_channel {
                    Some((done_sender, done_receiver)) => {
                        *done_chan = Some((done_sender.clone(), done_receiver));
                        cached_resource
                            .awaiting_body
                            .lock()
                            .unwrap()
                            .push(done_sender);
                    },
                    None => *done_chan = None,
                }
                // Received a response with 304 status code, in response to a request that matches a cached resource.
                // 1. update the headers of the cached resource.
                // 2. return a response, constructed from the cached resource.
                let resource_timing = ResourceFetchTiming::new(request.timing_type());
                let mut constructed_response = Response::new(
                    cached_resource.data.metadata.data.final_url.clone(),
                    resource_timing,
                );
                constructed_response.body = cached_resource.body.clone();
                constructed_response
                    .status
                    .clone_from(&cached_resource.data.status);
                constructed_response.https_state = cached_resource.data.https_state;
                constructed_response.referrer = request.referrer.to_url().cloned();
                constructed_response.referrer_policy = request.referrer_policy;
                constructed_response
                    .raw_status
                    .clone_from(&cached_resource.data.raw_status);
                constructed_response
                    .url_list
                    .clone_from(&cached_resource.data.url_list);
                cached_resource.data.expires = get_response_expiry(&constructed_response);
                let mut stored_headers = cached_resource.data.metadata.headers.lock().unwrap();
                stored_headers.extend(response.headers);
                constructed_response.headers = stored_headers.clone();
                return Some(constructed_response);
            }
        }
        None
    }

    fn invalidate_for_url(&mut self, url: &ServoUrl) {
        let entry_key = CacheKey::from_servo_url(url);
        if let Some(cached_resources) = self.entries.get_mut(&entry_key) {
            for cached_resource in cached_resources.iter_mut() {
                cached_resource.data.expires = Duration::seconds(0i64);
            }
        }
    }

    /// Invalidation.
    /// <https://tools.ietf.org/html/rfc7234#section-4.4>
    pub fn invalidate(&mut self, request: &Request, response: &Response) {
        // TODO(eijebong): Once headers support typed_get, update this to use them
        if let Some(Ok(location)) = response
            .headers
            .get(header::LOCATION)
            .map(HeaderValue::to_str)
        {
            if let Ok(url) = request.current_url().join(location) {
                self.invalidate_for_url(&url);
            }
        }
        if let Some(Ok(content_location)) = response
            .headers
            .get(header::CONTENT_LOCATION)
            .map(HeaderValue::to_str)
        {
            if let Ok(url) = request.current_url().join(content_location) {
                self.invalidate_for_url(&url);
            }
        }
        self.invalidate_for_url(&request.url());
    }

    /// Storing Responses in Caches.
    /// <https://tools.ietf.org/html/rfc7234#section-3>
    pub fn store(&mut self, request: &Request, response: &Response) {
        if pref!(network.http_cache.disabled) {
            return;
        }
        if request.method != Method::GET {
            // Only Get requests are cached.
            return;
        }
        if request.headers.contains_key(header::AUTHORIZATION) {
            // https://tools.ietf.org/html/rfc7234#section-3.1
            // A shared cache MUST NOT use a cached response
            // to a request with an Authorization header field
            //
            // TODO: unless a cache directive that allows such
            // responses to be stored is present in the response.
            return;
        };
        let entry_key = CacheKey::new(request);
        let metadata = match response.metadata() {
            Ok(FetchMetadata::Filtered {
                filtered: _,
                unsafe_: metadata,
            }) |
            Ok(FetchMetadata::Unfiltered(metadata)) => metadata,
            _ => return,
        };
        if !response_is_cacheable(&metadata) {
            return;
        }
        let expiry = get_response_expiry(response);
        let cacheable_metadata = CachedMetadata {
            headers: Arc::new(Mutex::new(response.headers.clone())),
            data: Measurable(MeasurableCachedMetadata {
                final_url: metadata.final_url,
                content_type: metadata.content_type.map(|v| v.0.to_string()),
                charset: metadata.charset,
                status: metadata.status,
            }),
        };
        let entry_resource = CachedResource {
            request_headers: Arc::new(Mutex::new(request.headers.clone())),
            body: response.body.clone(),
            aborted: response.aborted.clone(),
            awaiting_body: Arc::new(Mutex::new(vec![])),
            data: Measurable(MeasurableCachedResource {
                metadata: cacheable_metadata,
                location_url: response.location_url.clone(),
                https_state: response.https_state,
                status: response.status.clone(),
                raw_status: response.raw_status.clone(),
                url_list: response.url_list.clone(),
                expires: expiry,
                last_validated: time::now(),
            }),
        };
        let entry = self.entries.entry(entry_key).or_default();
        entry.push(entry_resource);
        // TODO: Complete incomplete responses, including 206 response, when stored here.
        // See A cache MAY complete a stored incomplete response by making a subsequent range request
        // https://tools.ietf.org/html/rfc7234#section-3.1
    }

    /// Clear the contents of this cache.
    pub fn clear(&mut self) {
        self.entries.clear();
    }
}
