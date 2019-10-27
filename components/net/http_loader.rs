/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use crate::connector::{create_http_client, Connector};
use crate::cookie;
use crate::cookie_storage::CookieStorage;
use crate::decoder::Decoder;
use crate::fetch::cors_cache::CorsCache;
use crate::fetch::methods::{
    is_cors_safelisted_method, is_cors_safelisted_request_header, main_fetch,
};
use crate::fetch::methods::{Data, DoneChannel, FetchContext, Target};
use crate::hsts::HstsList;
use crate::http_cache::{CacheKey, HttpCache};
use crate::resource_thread::AuthCache;
use crossbeam_channel::{unbounded, Sender};
use devtools_traits::{
    ChromeToDevtoolsControlMsg, DevtoolsControlMsg, HttpRequest as DevtoolsHttpRequest,
};
use devtools_traits::{HttpResponse as DevtoolsHttpResponse, NetworkEvent};
use headers::authorization::Basic;
use headers::{AccessControlAllowCredentials, AccessControlAllowHeaders, HeaderMapExt};
use headers::{
    AccessControlAllowMethods, AccessControlRequestHeaders, AccessControlRequestMethod,
    Authorization,
};
use headers::{AccessControlAllowOrigin, AccessControlMaxAge};
use headers::{CacheControl, ContentEncoding, ContentLength};
use headers::{
    Host, IfModifiedSince, LastModified, Origin as HyperOrigin, Pragma, Referer, UserAgent,
};
use http::header::{self, HeaderName, HeaderValue};
use http::uri::Authority;
use http::{HeaderMap, Request as HyperRequest};
use hyper::{Body, Client, Method, Response as HyperResponse, StatusCode};
use hyper_serde::Serde;
use msg::constellation_msg::{HistoryStateId, PipelineId};
use net_traits::quality::{quality_to_value, Quality, QualityItem};
use net_traits::request::Origin::Origin as SpecificOrigin;
use net_traits::request::{CacheMode, CredentialsMode, Destination, Origin};
use net_traits::request::{RedirectMode, Referrer, Request, RequestBuilder, RequestMode};
use net_traits::request::{ResponseTainting, ServiceWorkersMode};
use net_traits::response::{HttpsState, Response, ResponseBody, ResponseType};
use net_traits::{CookieSource, FetchMetadata, NetworkError, ReferrerPolicy};
use net_traits::{
    RedirectEndValue, RedirectStartValue, ResourceAttribute, ResourceFetchTiming, ResourceTimeValue,
};
use openssl::ssl::SslConnectorBuilder;
use servo_arc::Arc;
use servo_url::{ImmutableOrigin, ServoUrl};
use std::collections::{HashMap, HashSet};
use std::error::Error;
use std::iter::FromIterator;
use std::mem;
use std::ops::Deref;
use std::str::FromStr;
use std::sync::{Condvar, Mutex, RwLock};
use std::time::{Duration, SystemTime};
use time::{self, Tm};
use tokio::prelude::{future, Future, Stream};
use tokio::runtime::Runtime;

lazy_static! {
    pub static ref HANDLE: Mutex<Runtime> = { Mutex::new(Runtime::new().unwrap()) };
}

/// The various states an entry of the HttpCache can be in.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum HttpCacheEntryState {
    /// The entry is fully up-to-date,
    /// there are no pending concurrent stores,
    /// and it is ready to construct cached responses.
    ReadyToConstruct,
    /// The entry is pending a number of concurrent stores.
    PendingStore(usize),
}

pub struct HttpState {
    pub hsts_list: RwLock<HstsList>,
    pub cookie_jar: RwLock<CookieStorage>,
    pub http_cache: RwLock<HttpCache>,
    /// A map of cache key to entry state,
    /// reflecting whether the cache entry is ready to read from,
    /// or whether a concurrent pending store should be awaited.
    pub http_cache_state: Mutex<HashMap<CacheKey, Arc<(Mutex<HttpCacheEntryState>, Condvar)>>>,
    pub auth_cache: RwLock<AuthCache>,
    pub history_states: RwLock<HashMap<HistoryStateId, Vec<u8>>>,
    pub client: Client<Connector, Body>,
}

impl HttpState {
    pub fn new(ssl_connector_builder: SslConnectorBuilder) -> HttpState {
        HttpState {
            hsts_list: RwLock::new(HstsList::new()),
            cookie_jar: RwLock::new(CookieStorage::new(150)),
            auth_cache: RwLock::new(AuthCache::new()),
            history_states: RwLock::new(HashMap::new()),
            http_cache: RwLock::new(HttpCache::new()),
            http_cache_state: Mutex::new(HashMap::new()),
            client: create_http_client(ssl_connector_builder, HANDLE.lock().unwrap().executor()),
        }
    }
}

fn precise_time_ms() -> u64 {
    time::precise_time_ns() / (1000 * 1000)
}

// Step 3 of https://fetch.spec.whatwg.org/#concept-fetch.
pub fn set_default_accept(destination: Destination, headers: &mut HeaderMap) {
    if headers.contains_key(header::ACCEPT) {
        return;
    }
    let value = match destination {
        // Step 3.2.
        Destination::Document => vec![
            QualityItem::new(mime::TEXT_HTML, Quality::from_u16(1000)),
            QualityItem::new(
                "application/xhtml+xml".parse().unwrap(),
                Quality::from_u16(1000),
            ),
            QualityItem::new("application/xml".parse().unwrap(), Quality::from_u16(900)),
            QualityItem::new(mime::STAR_STAR, Quality::from_u16(800)),
        ],
        // Step 3.3.
        Destination::Image => vec![
            QualityItem::new(mime::IMAGE_PNG, Quality::from_u16(1000)),
            QualityItem::new(mime::IMAGE_SVG, Quality::from_u16(1000)),
            QualityItem::new(mime::IMAGE_STAR, Quality::from_u16(800)),
            QualityItem::new(mime::STAR_STAR, Quality::from_u16(500)),
        ],
        // Step 3.3.
        Destination::Style => vec![
            QualityItem::new(mime::TEXT_CSS, Quality::from_u16(1000)),
            QualityItem::new(mime::STAR_STAR, Quality::from_u16(100)),
        ],
        // Step 3.1.
        _ => vec![QualityItem::new(mime::STAR_STAR, Quality::from_u16(1000))],
    };

    // Step 3.4.
    // TODO(eijebong): Change this once typed headers are done
    headers.insert(header::ACCEPT, quality_to_value(value));
}

fn set_default_accept_encoding(headers: &mut HeaderMap) {
    if headers.contains_key(header::ACCEPT_ENCODING) {
        return;
    }

    // TODO(eijebong): Change this once typed headers are done
    headers.insert(
        header::ACCEPT_ENCODING,
        HeaderValue::from_static("gzip, deflate, br"),
    );
}

pub fn set_default_accept_language(headers: &mut HeaderMap) {
    if headers.contains_key(header::ACCEPT_LANGUAGE) {
        return;
    }

    // TODO(eijebong): Change this once typed headers are done
    headers.insert(
        header::ACCEPT_LANGUAGE,
        HeaderValue::from_static("en-US, en; q=0.5"),
    );
}

/// <https://w3c.github.io/webappsec-referrer-policy/#referrer-policy-state-no-referrer-when-downgrade>
fn no_referrer_when_downgrade_header(referrer_url: ServoUrl, url: ServoUrl) -> Option<ServoUrl> {
    if referrer_url.scheme() == "https" && url.scheme() != "https" {
        return None;
    }
    return strip_url(referrer_url, false);
}

/// <https://w3c.github.io/webappsec-referrer-policy/#referrer-policy-strict-origin>
fn strict_origin(referrer_url: ServoUrl, url: ServoUrl) -> Option<ServoUrl> {
    if referrer_url.scheme() == "https" && url.scheme() != "https" {
        return None;
    }
    strip_url(referrer_url, true)
}

/// <https://w3c.github.io/webappsec-referrer-policy/#referrer-policy-strict-origin-when-cross-origin>
fn strict_origin_when_cross_origin(referrer_url: ServoUrl, url: ServoUrl) -> Option<ServoUrl> {
    if referrer_url.scheme() == "https" && url.scheme() != "https" {
        return None;
    }
    let cross_origin = referrer_url.origin() != url.origin();
    strip_url(referrer_url, cross_origin)
}

/// <https://w3c.github.io/webappsec-referrer-policy/#strip-url>
fn strip_url(mut referrer_url: ServoUrl, origin_only: bool) -> Option<ServoUrl> {
    const MAX_REFERRER_URL_LENGTH: usize = 4096;
    if referrer_url.scheme() == "https" || referrer_url.scheme() == "http" {
        {
            let referrer = referrer_url.as_mut_url();
            referrer.set_username("").unwrap();
            referrer.set_password(None).unwrap();
            referrer.set_fragment(None);
            // Limit `referer` header's value to 4k <https://github.com/w3c/webappsec-referrer-policy/pull/122>
            if origin_only || referrer.as_str().len() > MAX_REFERRER_URL_LENGTH {
                referrer.set_path("");
                referrer.set_query(None);
            }
        }
        return Some(referrer_url);
    }
    return None;
}

/// <https://w3c.github.io/webappsec-referrer-policy/#determine-requests-referrer>
/// Steps 4-6.
pub fn determine_request_referrer(
    headers: &mut HeaderMap,
    referrer_policy: ReferrerPolicy,
    referrer_source: ServoUrl,
    current_url: ServoUrl,
) -> Option<ServoUrl> {
    assert!(!headers.contains_key(header::REFERER));
    // FIXME(#14505): this does not seem to be the correct way of checking for
    //                same-origin requests.
    let cross_origin = referrer_source.origin() != current_url.origin();
    // FIXME(#14506): some of these cases are expected to consider whether the
    //                request's client is "TLS-protected", whatever that means.
    match referrer_policy {
        ReferrerPolicy::NoReferrer => None,
        ReferrerPolicy::Origin => strip_url(referrer_source, true),
        ReferrerPolicy::SameOrigin => {
            if cross_origin {
                None
            } else {
                strip_url(referrer_source, false)
            }
        },
        ReferrerPolicy::UnsafeUrl => strip_url(referrer_source, false),
        ReferrerPolicy::OriginWhenCrossOrigin => strip_url(referrer_source, cross_origin),
        ReferrerPolicy::StrictOrigin => strict_origin(referrer_source, current_url),
        ReferrerPolicy::StrictOriginWhenCrossOrigin => {
            strict_origin_when_cross_origin(referrer_source, current_url)
        },
        ReferrerPolicy::NoReferrerWhenDowngrade => {
            no_referrer_when_downgrade_header(referrer_source, current_url)
        },
    }
}

pub fn set_request_cookies(
    url: &ServoUrl,
    headers: &mut HeaderMap,
    cookie_jar: &RwLock<CookieStorage>,
) {
    let mut cookie_jar = cookie_jar.write().unwrap();
    if let Some(cookie_list) = cookie_jar.cookies_for_url(url, CookieSource::HTTP) {
        headers.insert(
            header::COOKIE,
            HeaderValue::from_bytes(cookie_list.as_bytes()).unwrap(),
        );
    }
}

fn set_cookie_for_url(cookie_jar: &RwLock<CookieStorage>, request: &ServoUrl, cookie_val: &str) {
    let mut cookie_jar = cookie_jar.write().unwrap();
    let source = CookieSource::HTTP;

    if let Some(cookie) = cookie::Cookie::from_cookie_string(cookie_val.into(), request, source) {
        cookie_jar.push(cookie, request, source);
    }
}

fn set_cookies_from_headers(
    url: &ServoUrl,
    headers: &HeaderMap,
    cookie_jar: &RwLock<CookieStorage>,
) {
    for cookie in headers.get_all(header::SET_COOKIE) {
        if let Ok(cookie_str) = cookie.to_str() {
            set_cookie_for_url(&cookie_jar, &url, &cookie_str);
        }
    }
}

fn prepare_devtools_request(
    request_id: String,
    url: ServoUrl,
    method: Method,
    headers: HeaderMap,
    body: Option<Vec<u8>>,
    pipeline_id: PipelineId,
    now: Tm,
    connect_time: u64,
    send_time: u64,
    is_xhr: bool,
) -> ChromeToDevtoolsControlMsg {
    let request = DevtoolsHttpRequest {
        url: url,
        method: method,
        headers: headers,
        body: body,
        pipeline_id: pipeline_id,
        startedDateTime: now,
        timeStamp: now.to_timespec().sec,
        connect_time: connect_time,
        send_time: send_time,
        is_xhr: is_xhr,
    };
    let net_event = NetworkEvent::HttpRequest(request);

    ChromeToDevtoolsControlMsg::NetworkEvent(request_id, net_event)
}

fn send_request_to_devtools(
    msg: ChromeToDevtoolsControlMsg,
    devtools_chan: &Sender<DevtoolsControlMsg>,
) {
    devtools_chan
        .send(DevtoolsControlMsg::FromChrome(msg))
        .unwrap();
}

fn send_response_to_devtools(
    devtools_chan: &Sender<DevtoolsControlMsg>,
    request_id: String,
    headers: Option<HeaderMap>,
    status: Option<(u16, Vec<u8>)>,
    pipeline_id: PipelineId,
) {
    let response = DevtoolsHttpResponse {
        headers: headers,
        status: status,
        body: None,
        pipeline_id: pipeline_id,
    };
    let net_event_response = NetworkEvent::HttpResponse(response);

    let msg = ChromeToDevtoolsControlMsg::NetworkEvent(request_id, net_event_response);
    let _ = devtools_chan.send(DevtoolsControlMsg::FromChrome(msg));
}

fn auth_from_cache(
    auth_cache: &RwLock<AuthCache>,
    origin: &ImmutableOrigin,
) -> Option<Authorization<Basic>> {
    if let Some(ref auth_entry) = auth_cache
        .read()
        .unwrap()
        .entries
        .get(&origin.ascii_serialization())
    {
        let user_name = &auth_entry.user_name;
        let password = &auth_entry.password;
        Some(Authorization::basic(user_name, password))
    } else {
        None
    }
}

fn obtain_response(
    client: &Client<Connector, Body>,
    url: &ServoUrl,
    method: &Method,
    request_headers: &HeaderMap,
    data: &Option<Vec<u8>>,
    load_data_method: &Method,
    pipeline_id: &Option<PipelineId>,
    iters: u32,
    request_id: Option<&str>,
    is_xhr: bool,
    context: &FetchContext,
) -> Box<
    dyn Future<
        Item = (HyperResponse<Decoder>, Option<ChromeToDevtoolsControlMsg>),
        Error = NetworkError,
    >,
> {
    let mut headers = request_headers.clone();

    // Avoid automatically sending request body if a redirect has occurred.
    //
    // TODO - This is the wrong behaviour according to the RFC. However, I'm not
    // sure how much "correctness" vs. real-world is important in this case.
    //
    // https://tools.ietf.org/html/rfc7231#section-6.4
    let is_redirected_request = iters != 1;
    let request_body;
    match data {
        &Some(ref d) if !is_redirected_request => {
            headers.typed_insert(ContentLength(d.len() as u64));
            request_body = d.clone();
        },
        _ => {
            if *load_data_method != Method::GET && *load_data_method != Method::HEAD {
                headers.typed_insert(ContentLength(0))
            }
            request_body = vec![];
        },
    }

    context
        .timing
        .lock()
        .unwrap()
        .set_attribute(ResourceAttribute::DomainLookupStart);

    // TODO(#21261) connect_start: set if a persistent connection is *not* used and the last non-redirected
    // fetch passes the timing allow check
    let connect_start = precise_time_ms();
    context
        .timing
        .lock()
        .unwrap()
        .set_attribute(ResourceAttribute::ConnectStart(connect_start));

    // https://url.spec.whatwg.org/#percent-encoded-bytes
    let request = HyperRequest::builder()
        .method(method)
        .uri(
            url.clone()
                .into_url()
                .as_ref()
                .replace("|", "%7C")
                .replace("{", "%7B")
                .replace("}", "%7D"),
        )
        .body(request_body.clone().into());

    // TODO: We currently don't know when the handhhake before the connection is done
    // so our best bet would be to set `secure_connection_start` here when we are currently
    // fetching on a HTTPS url.
    if url.scheme() == "https" {
        context
            .timing
            .lock()
            .unwrap()
            .set_attribute(ResourceAttribute::SecureConnectionStart);
    }

    let mut request = match request {
        Ok(request) => request,
        Err(e) => return Box::new(future::result(Err(NetworkError::from_http_error(&e)))),
    };
    *request.headers_mut() = headers.clone();

    let connect_end = precise_time_ms();
    context
        .timing
        .lock()
        .unwrap()
        .set_attribute(ResourceAttribute::ConnectEnd(connect_end));

    let request_id = request_id.map(|v| v.to_owned());
    let pipeline_id = pipeline_id.clone();
    let closure_url = url.clone();
    let method = method.clone();
    let send_start = precise_time_ms();

    Box::new(
        client
            .request(request)
            .and_then(move |res| {
                let send_end = precise_time_ms();

                // TODO(#21271) response_start: immediately after receiving first byte of response

                let msg = if let Some(request_id) = request_id {
                    if let Some(pipeline_id) = pipeline_id {
                        Some(prepare_devtools_request(
                            request_id,
                            closure_url,
                            method.clone(),
                            headers,
                            Some(request_body.clone()),
                            pipeline_id,
                            time::now(),
                            connect_end - connect_start,
                            send_end - send_start,
                            is_xhr,
                        ))
                    // TODO: ^This is not right, connect_start is taken before contructing the
                    // request and connect_end at the end of it. send_start is takend before the
                    // connection too. I'm not sure it's currently possible to get the time at the
                    // point between the connection and the start of a request.
                    } else {
                        debug!("Not notifying devtools (no pipeline_id)");
                        None
                    }
                } else {
                    debug!("Not notifying devtools (no request_id)");
                    None
                };
                Ok((Decoder::detect(res), msg))
            })
            .map_err(move |e| NetworkError::from_hyper_error(&e)),
    )
}

/// [HTTP fetch](https://fetch.spec.whatwg.org#http-fetch)
pub fn http_fetch(
    request: &mut Request,
    cache: &mut CorsCache,
    cors_flag: bool,
    cors_preflight_flag: bool,
    authentication_fetch_flag: bool,
    target: Target,
    done_chan: &mut DoneChannel,
    context: &FetchContext,
) -> Response {
    // This is a new async fetch, reset the channel we are waiting on
    *done_chan = None;
    // Step 1
    let mut response: Option<Response> = None;

    // Step 2
    // nothing to do, since actual_response is a function on response

    // Step 3
    if request.service_workers_mode == ServiceWorkersMode::All {
        // TODO: Substep 1
        // Set response to the result of invoking handle fetch for request.

        // Substep 2
        if let Some(ref res) = response {
            // Subsubstep 1
            // TODO: transmit body for request

            // Subsubstep 2
            // nothing to do, since actual_response is a function on response

            // Subsubstep 3
            if (res.response_type == ResponseType::Opaque && request.mode != RequestMode::NoCors) ||
                (res.response_type == ResponseType::OpaqueRedirect &&
                    request.redirect_mode != RedirectMode::Manual) ||
                (res.url_list.len() > 1 && request.redirect_mode != RedirectMode::Follow) ||
                res.is_network_error()
            {
                return Response::network_error(NetworkError::Internal("Request failed".into()));
            }

            // Subsubstep 4
            // TODO: set response's CSP list on actual_response
        }
    }

    // Step 4
    if response.is_none() {
        // Substep 1
        if cors_preflight_flag {
            let method_cache_match = cache.match_method(&*request, request.method.clone());

            let method_mismatch = !method_cache_match &&
                (!is_cors_safelisted_method(&request.method) || request.use_cors_preflight);
            let header_mismatch = request.headers.iter().any(|(name, value)| {
                !cache.match_header(&*request, &name) &&
                    !is_cors_safelisted_request_header(&name, &value)
            });

            // Sub-substep 1
            if method_mismatch || header_mismatch {
                let preflight_result = cors_preflight_fetch(&request, cache, context);
                // Sub-substep 2
                if let Some(e) = preflight_result.get_network_error() {
                    return Response::network_error(e.clone());
                }
            }
        }

        // Substep 2
        if request.redirect_mode == RedirectMode::Follow {
            request.service_workers_mode = ServiceWorkersMode::None;
        }

        // Generally, we use a persistent connection, so we will also set other PerformanceResourceTiming
        //   attributes to this as well (domain_lookup_start, domain_lookup_end, connect_start, connect_end,
        //   secure_connection_start)
        context
            .timing
            .lock()
            .unwrap()
            .set_attribute(ResourceAttribute::RequestStart);

        let mut fetch_result = http_network_or_cache_fetch(
            request,
            authentication_fetch_flag,
            cors_flag,
            done_chan,
            context,
        );

        // Substep 4
        if cors_flag && cors_check(&request, &fetch_result).is_err() {
            return Response::network_error(NetworkError::Internal("CORS check failed".into()));
        }

        fetch_result.return_internal = false;
        response = Some(fetch_result);
    }

    // response is guaranteed to be something by now
    let mut response = response.unwrap();

    // Step 5
    if response
        .actual_response()
        .status
        .as_ref()
        .map_or(false, is_redirect_status)
    {
        // Substep 1.
        if response
            .actual_response()
            .status
            .as_ref()
            .map_or(true, |s| s.0 != StatusCode::SEE_OTHER)
        {
            // TODO: send RST_STREAM frame
        }

        // Substep 2-3.
        let location = response
            .actual_response()
            .headers
            .get(header::LOCATION)
            .and_then(|v| {
                HeaderValue::to_str(v)
                    .map(|l| {
                        ServoUrl::parse_with_base(response.actual_response().url(), &l)
                            .map_err(|err| err.description().into())
                    })
                    .ok()
            });

        // Substep 4.
        response.actual_response_mut().location_url = location;

        // Substep 5.
        response = match request.redirect_mode {
            RedirectMode::Error => {
                Response::network_error(NetworkError::Internal("Redirect mode error".into()))
            },
            RedirectMode::Manual => response.to_filtered(ResponseType::OpaqueRedirect),
            RedirectMode::Follow => {
                // set back to default
                response.return_internal = true;
                http_redirect_fetch(
                    request, cache, response, cors_flag, target, done_chan, context,
                )
            },
        };
    }

    // set back to default
    response.return_internal = true;
    context
        .timing
        .lock()
        .unwrap()
        .set_attribute(ResourceAttribute::RedirectCount(
            request.redirect_count as u16,
        ));

    response.resource_timing = Arc::clone(&context.timing);

    // Step 6
    response
}

// Convenience struct that implements Drop, for setting redirectEnd on function return
struct RedirectEndTimer(Option<Arc<Mutex<ResourceFetchTiming>>>);

impl RedirectEndTimer {
    fn neuter(&mut self) {
        self.0 = None;
    }
}

impl Drop for RedirectEndTimer {
    fn drop(&mut self) {
        let RedirectEndTimer(resource_fetch_timing_opt) = self;

        resource_fetch_timing_opt.as_ref().map_or((), |t| {
            t.lock()
                .unwrap()
                .set_attribute(ResourceAttribute::RedirectEnd(RedirectEndValue::Zero));
        })
    }
}

/// [HTTP redirect fetch](https://fetch.spec.whatwg.org#http-redirect-fetch)
pub fn http_redirect_fetch(
    request: &mut Request,
    cache: &mut CorsCache,
    response: Response,
    cors_flag: bool,
    target: Target,
    done_chan: &mut DoneChannel,
    context: &FetchContext,
) -> Response {
    let mut redirect_end_timer = RedirectEndTimer(Some(context.timing.clone()));

    // Step 1
    assert!(response.return_internal);

    let location_url = response.actual_response().location_url.clone();
    let location_url = match location_url {
        // Step 2
        None => return response,
        // Step 3
        Some(Err(err)) => {
            return Response::network_error(NetworkError::Internal(
                "Location URL parse failure: ".to_owned() + &err,
            ));
        },
        // Step 4
        Some(Ok(ref url)) if !matches!(url.scheme(), "http" | "https") => {
            return Response::network_error(NetworkError::Internal(
                "Location URL not an HTTP(S) scheme".into(),
            ));
        },
        Some(Ok(url)) => url,
    };

    // Step 1 of https://w3c.github.io/resource-timing/#dom-performanceresourcetiming-fetchstart
    // TODO: check origin and timing allow check
    context
        .timing
        .lock()
        .unwrap()
        .set_attribute(ResourceAttribute::RedirectStart(
            RedirectStartValue::FetchStart,
        ));

    context
        .timing
        .lock()
        .unwrap()
        .set_attribute(ResourceAttribute::FetchStart);

    // start_time should equal redirect_start if nonzero; else fetch_start
    context
        .timing
        .lock()
        .unwrap()
        .set_attribute(ResourceAttribute::StartTime(ResourceTimeValue::FetchStart));

    context
        .timing
        .lock()
        .unwrap()
        .set_attribute(ResourceAttribute::StartTime(
            ResourceTimeValue::RedirectStart,
        )); // updates start_time only if redirect_start is nonzero (implying TAO)

    // Step 5
    if request.redirect_count >= 20 {
        return Response::network_error(NetworkError::Internal("Too many redirects".into()));
    }

    // Step 6
    request.redirect_count += 1;

    // Step 7
    let same_origin = match request.origin {
        Origin::Origin(ref origin) => *origin == location_url.origin(),
        Origin::Client => panic!(
            "Request origin should not be client for {}",
            request.current_url()
        ),
    };
    let has_credentials = has_credentials(&location_url);

    if request.mode == RequestMode::CorsMode && !same_origin && has_credentials {
        return Response::network_error(NetworkError::Internal(
            "Cross-origin credentials check failed".into(),
        ));
    }

    // Step 8
    if cors_flag && has_credentials {
        return Response::network_error(NetworkError::Internal("Credentials check failed".into()));
    }

    // Step 9
    if response
        .actual_response()
        .status
        .as_ref()
        .map_or(true, |s| s.0 != StatusCode::SEE_OTHER) &&
        request.body.as_ref().map_or(false, |b| b.is_empty())
    {
        return Response::network_error(NetworkError::Internal("Request body is not done".into()));
    }

    // Step 10
    if cors_flag && location_url.origin() != request.current_url().origin() {
        request.origin = Origin::Origin(ImmutableOrigin::new_opaque());
    }

    // Step 11
    if response
        .actual_response()
        .status
        .as_ref()
        .map_or(false, |(code, _)| {
            ((*code == StatusCode::MOVED_PERMANENTLY || *code == StatusCode::FOUND) &&
                request.method == Method::POST) ||
                (*code == StatusCode::SEE_OTHER && request.method != Method::HEAD)
        })
    {
        request.method = Method::GET;
        request.body = None;
    }

    // Step 12
    if let Some(_) = request.body {
        // TODO: extract request's body's source
    }

    // Step 13
    request.url_list.push(location_url);

    // Step 14
    // TODO implement referrer policy

    // Step 15
    let recursive_flag = request.redirect_mode != RedirectMode::Manual;

    let fetch_response = main_fetch(
        request,
        cache,
        cors_flag,
        recursive_flag,
        target,
        done_chan,
        context,
    );

    // TODO: timing allow check
    context
        .timing
        .lock()
        .unwrap()
        .set_attribute(ResourceAttribute::RedirectEnd(
            RedirectEndValue::ResponseEnd,
        ));
    redirect_end_timer.neuter();

    fetch_response
}

fn try_immutable_origin_to_hyper_origin(url_origin: &ImmutableOrigin) -> Option<HyperOrigin> {
    match *url_origin {
        ImmutableOrigin::Opaque(_) => Some(HyperOrigin::NULL),
        ImmutableOrigin::Tuple(ref scheme, ref host, ref port) => {
            let port = match (scheme.as_ref(), port) {
                ("http", 80) | ("https", 443) => None,
                _ => Some(*port),
            };
            HyperOrigin::try_from_parts(&scheme, &host.to_string(), port).ok()
        },
    }
}

/// [HTTP network or cache fetch](https://fetch.spec.whatwg.org#http-network-or-cache-fetch)
fn http_network_or_cache_fetch(
    request: &mut Request,
    authentication_fetch_flag: bool,
    cors_flag: bool,
    done_chan: &mut DoneChannel,
    context: &FetchContext,
) -> Response {
    // Step 2
    let mut response: Option<Response> = None;
    // Step 4
    let mut revalidating_flag = false;

    // TODO: Implement Window enum for Request
    let request_has_no_window = true;

    // Step 5.1
    let mut http_request;
    let http_request = if request_has_no_window && request.redirect_mode == RedirectMode::Error {
        request
    } else {
        // Step 5.2
        // TODO Implement body source
        http_request = request.clone();
        &mut http_request
    };

    // Step 5.3
    let credentials_flag = match http_request.credentials_mode {
        CredentialsMode::Include => true,
        CredentialsMode::CredentialsSameOrigin
            if http_request.response_tainting == ResponseTainting::Basic =>
        {
            true
        },
        _ => false,
    };

    let content_length_value = match http_request.body {
        None => match http_request.method {
            // Step 5.5
            Method::POST | Method::PUT => Some(0),
            // Step 5.4
            _ => None,
        },
        // Step 5.6
        Some(ref http_request_body) => Some(http_request_body.len() as u64),
    };

    // Step 5.7
    if let Some(content_length_value) = content_length_value {
        http_request
            .headers
            .typed_insert(ContentLength(content_length_value));
        if http_request.keep_alive {
            // Step 5.8 TODO: needs request's client object
        }
    }

    // Step 5.9
    match http_request.referrer {
        Referrer::NoReferrer => (),
        Referrer::ReferrerUrl(ref http_request_referrer) => {
            if let Ok(referer) = http_request_referrer.to_string().parse::<Referer>() {
                http_request.headers.typed_insert(referer);
            } else {
                // This error should only happen in cases where hyper and rust-url disagree
                // about how to parse a referer.
                // https://github.com/servo/servo/issues/24175
                error!("Failed to parse {} as referer", http_request_referrer);
            }
        },
        Referrer::Client =>
        // it should be impossible for referrer to be anything else during fetching
        // https://fetch.spec.whatwg.org/#concept-request-referrer
        {
            unreachable!()
        },
    };

    // Step 5.10
    if cors_flag || (http_request.method != Method::GET && http_request.method != Method::HEAD) {
        debug_assert_ne!(http_request.origin, Origin::Client);
        if let Origin::Origin(ref url_origin) = http_request.origin {
            if let Some(hyper_origin) = try_immutable_origin_to_hyper_origin(url_origin) {
                http_request.headers.typed_insert(hyper_origin)
            }
        }
    }

    // Step 5.11
    if !http_request.headers.contains_key(header::USER_AGENT) {
        let user_agent = context.user_agent.clone().into_owned();
        http_request
            .headers
            .typed_insert::<UserAgent>(user_agent.parse().unwrap());
    }

    match http_request.cache_mode {
        // Step 5.12
        CacheMode::Default if is_no_store_cache(&http_request.headers) => {
            http_request.cache_mode = CacheMode::NoStore;
        },

        // Step 5.13
        CacheMode::NoCache if !http_request.headers.contains_key(header::CACHE_CONTROL) => {
            http_request
                .headers
                .typed_insert(CacheControl::new().with_max_age(Duration::from_secs(0)));
        },

        // Step 5.14
        CacheMode::Reload | CacheMode::NoStore => {
            // Substep 1
            if !http_request.headers.contains_key(header::PRAGMA) {
                http_request.headers.typed_insert(Pragma::no_cache());
            }

            // Substep 2
            if !http_request.headers.contains_key(header::CACHE_CONTROL) {
                http_request
                    .headers
                    .typed_insert(CacheControl::new().with_no_cache());
            }
        },

        _ => {},
    }

    // Step 5.15
    // TODO: if necessary append `Accept-Encoding`/`identity` to headers

    // Step 5.16
    let current_url = http_request.current_url();
    if let Ok(host) = format!(
        "{}{}",
        current_url.host_str().unwrap(),
        current_url
            .port()
            .map(|v| format!(":{}", v))
            .unwrap_or("".into())
    )
    .parse::<Authority>()
    .map(Host::from)
    {
        http_request.headers.typed_insert(host);
    } else {
        // This error should only happen in cases where hyper and rust-url disagree
        // about how to parse an authority.
        // https://github.com/servo/servo/issues/24175
        error!("Failed to parse {} as authority", current_url);
    }

    // unlike http_loader, we should not set the accept header
    // here, according to the fetch spec
    set_default_accept_encoding(&mut http_request.headers);

    // Step 5.17
    // TODO some of this step can't be implemented yet
    if credentials_flag {
        // Substep 1
        // TODO http://mxr.mozilla.org/servo/source/components/net/http_loader.rs#504
        // XXXManishearth http_loader has block_cookies: support content blocking here too
        set_request_cookies(
            &current_url,
            &mut http_request.headers,
            &context.state.cookie_jar,
        );
        // Substep 2
        if !http_request.headers.contains_key(header::AUTHORIZATION) {
            // Substep 3
            let mut authorization_value = None;

            // Substep 4
            if let Some(basic) = auth_from_cache(&context.state.auth_cache, &current_url.origin()) {
                if !http_request.use_url_credentials || !has_credentials(&current_url) {
                    authorization_value = Some(basic);
                }
            }

            // Substep 5
            if authentication_fetch_flag && authorization_value.is_none() {
                if has_credentials(&current_url) {
                    authorization_value = Some(Authorization::basic(
                        current_url.username(),
                        current_url.password().unwrap_or(""),
                    ));
                }
            }

            // Substep 6
            if let Some(basic) = authorization_value {
                http_request.headers.typed_insert(basic);
            }
        }
    }

    // Step 5.18
    // TODO If there’s a proxy-authentication entry, use it as appropriate.

    // If the cache is not ready to construct a response, wait.
    //
    // The cache is not ready if a previous fetch checked the cache, found nothing,
    // and moved on to a network fetch, and hasn't updated the cache yet with a pending resource.
    //
    // Note that this is a different workflow from the one involving `wait_for_cached_response`.
    // That one happens when a fetch gets a cache hit, and the resource is pending completion from the network.
    {
        let (lock, cvar) = {
            let entry_key = CacheKey::new(&http_request);
            let mut state_map = context.state.http_cache_state.lock().unwrap();
            &*state_map
                .entry(entry_key)
                .or_insert_with(|| {
                    Arc::new((
                        Mutex::new(HttpCacheEntryState::ReadyToConstruct),
                        Condvar::new(),
                    ))
                })
                .clone()
        };

        // Start of critical section on http-cache state.
        let mut state = lock.lock().unwrap();
        while let HttpCacheEntryState::PendingStore(_) = *state {
            let (current_state, time_out) = cvar
                .wait_timeout(state, Duration::from_millis(500))
                .unwrap();
            state = current_state;
            if time_out.timed_out() {
                // After a timeout, ignore the pending store.
                break;
            }
        }

        // Step 5.19
        if let Ok(http_cache) = context.state.http_cache.read() {
            if let Some(response_from_cache) =
                http_cache.construct_response(&http_request, done_chan)
            {
                let response_headers = response_from_cache.response.headers.clone();
                // Substep 1, 2, 3, 4
                let (cached_response, needs_revalidation) =
                    match (http_request.cache_mode, &http_request.mode) {
                        (CacheMode::ForceCache, _) => (Some(response_from_cache.response), false),
                        (CacheMode::OnlyIfCached, &RequestMode::SameOrigin) => {
                            (Some(response_from_cache.response), false)
                        },
                        (CacheMode::OnlyIfCached, _) |
                        (CacheMode::NoStore, _) |
                        (CacheMode::Reload, _) => (None, false),
                        (_, _) => (
                            Some(response_from_cache.response),
                            response_from_cache.needs_validation,
                        ),
                    };
                if needs_revalidation {
                    revalidating_flag = true;
                    // Substep 5
                    if let Some(http_date) = response_headers.typed_get::<LastModified>() {
                        let http_date: SystemTime = http_date.into();
                        http_request
                            .headers
                            .typed_insert(IfModifiedSince::from(http_date));
                    }
                    if let Some(entity_tag) = response_headers.get(header::ETAG) {
                        http_request
                            .headers
                            .insert(header::IF_NONE_MATCH, entity_tag.clone());
                    }
                } else {
                    // Substep 6
                    response = cached_response;
                }
                if response.is_none() {
                    // Ensure the done chan is not set if we're not using the cached response,
                    // as the cache might have set it to Some if it constructed a pending response.
                    *done_chan = None;

                    // Update the cache state, incrementing the pending store count,
                    // or starting the count.
                    if let HttpCacheEntryState::PendingStore(i) = *state {
                        let new = i + 1;
                        *state = HttpCacheEntryState::PendingStore(new);
                    } else {
                        *state = HttpCacheEntryState::PendingStore(1);
                    }
                }
            }
        }
        // Notify the next thread waiting in line, if there is any.
        if *state == HttpCacheEntryState::ReadyToConstruct {
            cvar.notify_one();
        }
        // End of critical section on http-cache state.
    }

    // Decrement the number of pending stores,
    // and set the state to ready to construct,
    // if no stores are pending.
    fn update_http_cache_state(context: &FetchContext, http_request: &Request) {
        let (lock, cvar) = {
            let entry_key = CacheKey::new(&http_request);
            let mut state_map = context.state.http_cache_state.lock().unwrap();
            &*state_map
                .get_mut(&entry_key)
                .expect("Entry in http-cache state to have been previously inserted")
                .clone()
        };
        let mut state = lock.lock().unwrap();
        if let HttpCacheEntryState::PendingStore(i) = *state {
            let new = i - 1;
            if new == 0 {
                *state = HttpCacheEntryState::ReadyToConstruct;
                // Notify the next thread waiting in line, if there is any.
                cvar.notify_one();
            } else {
                *state = HttpCacheEntryState::PendingStore(new);
            }
        }
    }

    fn wait_for_cached_response(done_chan: &mut DoneChannel, response: &mut Option<Response>) {
        if let Some(ref ch) = *done_chan {
            // The cache constructed a response with a body of ResponseBody::Receiving.
            // We wait for the response in the cache to "finish",
            // with a body of either Done or Cancelled.
            assert!(response.is_some());
            loop {
                match ch
                    .1
                    .recv()
                    .expect("HTTP cache should always send Done or Cancelled")
                {
                    Data::Payload(_) => {},
                    Data::Done => break, // Return the full response as if it was initially cached as such.
                    Data::Cancelled => {
                        // The response was cancelled while the fetch was ongoing.
                        // Set response to None, which will trigger a network fetch below.
                        *response = None;
                        break;
                    },
                }
            }
        }
        // Set done_chan back to None, it's cache-related usefulness ends here.
        *done_chan = None;
    }

    wait_for_cached_response(done_chan, &mut response);

    // Step 6
    // TODO: https://infra.spec.whatwg.org/#if-aborted

    // Step 7
    if response.is_none() {
        // Substep 1
        if http_request.cache_mode == CacheMode::OnlyIfCached {
            // The cache will not be updated,
            // set its state to ready to construct.
            update_http_cache_state(context, &http_request);
            return Response::network_error(NetworkError::Internal(
                "Couldn't find response in cache".into(),
            ));
        }
    }
    // More Step 7
    if response.is_none() {
        // Substep 2
        let forward_response =
            http_network_fetch(http_request, credentials_flag, done_chan, context);
        // Substep 3
        if let Some((200..=399, _)) = forward_response.raw_status {
            if !http_request.method.is_safe() {
                if let Ok(mut http_cache) = context.state.http_cache.write() {
                    http_cache.invalidate(&http_request, &forward_response);
                }
            }
        }
        // Substep 4
        if revalidating_flag &&
            forward_response
                .status
                .as_ref()
                .map_or(false, |s| s.0 == StatusCode::NOT_MODIFIED)
        {
            if let Ok(mut http_cache) = context.state.http_cache.write() {
                // Ensure done_chan is None,
                // since the network response will be replaced by the revalidated stored one.
                *done_chan = None;
                response = http_cache.refresh(&http_request, forward_response.clone(), done_chan);
                wait_for_cached_response(done_chan, &mut response);
            }
        }

        // Substep 5
        if response.is_none() {
            if http_request.cache_mode != CacheMode::NoStore {
                // Subsubstep 2, doing it first to avoid a clone of forward_response.
                if let Ok(mut http_cache) = context.state.http_cache.write() {
                    http_cache.store(&http_request, &forward_response);
                }
            }
            // Subsubstep 1
            response = Some(forward_response);
        }
    }

    let mut response = response.unwrap();

    // The cache has been updated, set its state to ready to construct.
    update_http_cache_state(context, &http_request);

    // Step 8
    // TODO: if necessary set response's range-requested flag

    // Step 9
    // TODO: handle CORS not set and cross-origin blocked

    // Step 10
    // FIXME: Figure out what to do with request window objects
    if let (Some((StatusCode::UNAUTHORIZED, _)), false, true) =
        (response.status.as_ref(), cors_flag, credentials_flag)
    {
        // Substep 1
        // TODO: Spec says requires testing on multiple WWW-Authenticate headers

        // Substep 2
        if http_request.body.is_some() {
            // TODO Implement body source
        }

        // Substep 3
        if !http_request.use_url_credentials || authentication_fetch_flag {
            // FIXME: Prompt the user for username and password from the window

            // Wrong, but will have to do until we are able to prompt the user
            // otherwise this creates an infinite loop
            // We basically pretend that the user declined to enter credentials
            return response;
        }

        // Make sure this is set to None,
        // since we're about to start a new `http_network_or_cache_fetch`.
        *done_chan = None;

        // Substep 4
        response = http_network_or_cache_fetch(
            http_request,
            true, /* authentication flag */
            cors_flag,
            done_chan,
            context,
        );
    }

    // Step 11
    if let Some((StatusCode::PROXY_AUTHENTICATION_REQUIRED, _)) = response.status.as_ref() {
        // Step 1
        if request_has_no_window {
            return Response::network_error(NetworkError::Internal(
                "Can't find Window object".into(),
            ));
        }

        // Step 2
        // TODO: Spec says requires testing on Proxy-Authenticate headers

        // Step 3
        // FIXME: Prompt the user for proxy authentication credentials

        // Wrong, but will have to do until we are able to prompt the user
        // otherwise this creates an infinite loop
        // We basically pretend that the user declined to enter credentials
        return response;

        // Step 4
        // return http_network_or_cache_fetch(request, authentication_fetch_flag,
        //                                    cors_flag, done_chan, context);
    }

    // Step 12
    if authentication_fetch_flag {
        // TODO Create the authentication entry for request and the given realm
    }

    // Step 13
    response
}

// Convenience struct that implements Done, for setting responseEnd on function return
struct ResponseEndTimer(Option<Arc<Mutex<ResourceFetchTiming>>>);

impl ResponseEndTimer {
    fn neuter(&mut self) {
        self.0 = None;
    }
}

impl Drop for ResponseEndTimer {
    fn drop(&mut self) {
        let ResponseEndTimer(resource_fetch_timing_opt) = self;

        resource_fetch_timing_opt.as_ref().map_or((), |t| {
            t.lock()
                .unwrap()
                .set_attribute(ResourceAttribute::ResponseEnd);
        })
    }
}

/// [HTTP network fetch](https://fetch.spec.whatwg.org/#http-network-fetch)
fn http_network_fetch(
    request: &Request,
    credentials_flag: bool,
    done_chan: &mut DoneChannel,
    context: &FetchContext,
) -> Response {
    let mut response_end_timer = ResponseEndTimer(Some(context.timing.clone()));

    // Step 1
    // nothing to do here, since credentials_flag is already a boolean

    // Step 2
    // TODO be able to create connection using current url's origin and credentials

    // Step 3
    // TODO be able to tell if the connection is a failure

    // Step 4
    // TODO: check whether the connection is HTTP/2

    // Step 5
    let url = request.current_url();

    let request_id = context
        .devtools_chan
        .as_ref()
        .map(|_| uuid::Uuid::new_v4().to_simple().to_string());

    if log_enabled!(log::Level::Info) {
        info!("request for {} ({:?})", url, request.method);
        for header in request.headers.iter() {
            info!(" - {:?}", header);
        }
    }

    // XHR uses the default destination; other kinds of fetches (which haven't been implemented yet)
    // do not. Once we support other kinds of fetches we'll need to be more fine grained here
    // since things like image fetches are classified differently by devtools
    let is_xhr = request.destination == Destination::None;
    let response_future = obtain_response(
        &context.state.client,
        &url,
        &request.method,
        &request.headers,
        &request.body,
        &request.method,
        &request.pipeline_id,
        request.redirect_count + 1,
        request_id.as_ref().map(Deref::deref),
        is_xhr,
        context,
    );

    let pipeline_id = request.pipeline_id;
    // This will only get the headers, the body is read later
    let (res, msg) = match response_future.wait() {
        Ok(wrapped_response) => wrapped_response,
        Err(error) => return Response::network_error(error),
    };

    if log_enabled!(log::Level::Info) {
        info!("response for {}", url);
        for header in res.headers().iter() {
            info!(" - {:?}", header);
        }
    }

    let header_strings: Vec<&str> = res
        .headers()
        .get_all("Timing-Allow-Origin")
        .iter()
        .map(|header_value| header_value.to_str().unwrap_or(""))
        .collect();
    let wildcard_present = header_strings.iter().any(|header_str| *header_str == "*");
    // The spec: https://www.w3.org/TR/resource-timing-2/#sec-timing-allow-origin
    // says that a header string is either an origin or a wildcard so we can just do a straight
    // check against the document origin
    let req_origin_in_timing_allow = header_strings
        .iter()
        .any(|header_str| match request.origin {
            SpecificOrigin(ref immutable_request_origin) => {
                *header_str == immutable_request_origin.ascii_serialization()
            },
            _ => false,
        });

    let is_same_origin = request.url_list.iter().any(|url| match request.origin {
        SpecificOrigin(ref immutable_request_origin) => {
            url.clone().into_url().origin().ascii_serialization() ==
                immutable_request_origin.ascii_serialization()
        },
        _ => false,
    });

    if !(is_same_origin || req_origin_in_timing_allow || wildcard_present) {
        context.timing.lock().unwrap().mark_timing_check_failed();
    }

    let timing = context.timing.lock().unwrap().clone();
    let mut response = Response::new(url.clone(), timing);

    response.status = Some((
        res.status(),
        res.status().canonical_reason().unwrap_or("").into(),
    ));
    debug!("got {:?} response for {:?}", res.status(), request.url());
    response.raw_status = Some((
        res.status().as_u16(),
        res.status().canonical_reason().unwrap_or("").into(),
    ));
    response.headers = res.headers().clone();
    response.referrer = request.referrer.to_url().cloned();
    response.referrer_policy = request.referrer_policy.clone();

    let res_body = response.body.clone();

    // We're about to spawn a future to be waited on here
    let (done_sender, done_receiver) = unbounded();
    *done_chan = Some((done_sender.clone(), done_receiver));
    let meta = match response
        .metadata()
        .expect("Response metadata should exist at this stage")
    {
        FetchMetadata::Unfiltered(m) => m,
        FetchMetadata::Filtered { unsafe_, .. } => unsafe_,
    };

    let devtools_sender = context.devtools_chan.clone();
    let meta_status = meta.status;
    let meta_headers = meta.headers;
    let cancellation_listener = context.cancellation_listener.clone();
    if cancellation_listener.lock().unwrap().cancelled() {
        return Response::network_error(NetworkError::Internal("Fetch aborted".into()));
    }

    *res_body.lock().unwrap() = ResponseBody::Receiving(vec![]);
    let res_body2 = res_body.clone();

    if let Some(ref sender) = devtools_sender {
        if let Some(m) = msg {
            send_request_to_devtools(m, &sender);
        }

        // --- Tell devtools that we got a response
        // Send an HttpResponse message to devtools with the corresponding request_id
        if let Some(pipeline_id) = pipeline_id {
            send_response_to_devtools(
                &sender,
                request_id.unwrap(),
                meta_headers.map(Serde::into_inner),
                meta_status,
                pipeline_id,
            );
        }
    }

    let done_sender2 = done_sender.clone();
    let done_sender3 = done_sender.clone();
    let timing_ptr2 = context.timing.clone();
    let timing_ptr3 = context.timing.clone();
    let url1 = request.url();
    let url2 = url1.clone();
    HANDLE.lock().unwrap().spawn(
        res.into_body()
            .map_err(|_| ())
            .fold(res_body, move |res_body, chunk| {
                if cancellation_listener.lock().unwrap().cancelled() {
                    *res_body.lock().unwrap() = ResponseBody::Done(vec![]);
                    let _ = done_sender.send(Data::Cancelled);
                    return future::failed(());
                }
                if let ResponseBody::Receiving(ref mut body) = *res_body.lock().unwrap() {
                    let bytes = chunk.into_bytes();
                    body.extend_from_slice(&*bytes);
                    let _ = done_sender.send(Data::Payload(bytes.to_vec()));
                }
                future::ok(res_body)
            })
            .and_then(move |res_body| {
                debug!("successfully finished response for {:?}", url1);
                let mut body = res_body.lock().unwrap();
                let completed_body = match *body {
                    ResponseBody::Receiving(ref mut body) => mem::replace(body, vec![]),
                    _ => vec![],
                };
                *body = ResponseBody::Done(completed_body);
                timing_ptr2
                    .lock()
                    .unwrap()
                    .set_attribute(ResourceAttribute::ResponseEnd);
                let _ = done_sender2.send(Data::Done);
                future::ok(())
            })
            .map_err(move |_| {
                debug!("finished response for {:?} with error", url2);
                let mut body = res_body2.lock().unwrap();
                let completed_body = match *body {
                    ResponseBody::Receiving(ref mut body) => mem::replace(body, vec![]),
                    _ => vec![],
                };
                *body = ResponseBody::Done(completed_body);
                timing_ptr3
                    .lock()
                    .unwrap()
                    .set_attribute(ResourceAttribute::ResponseEnd);
                let _ = done_sender3.send(Data::Done);
            }),
    );

    // TODO these substeps aren't possible yet
    // Substep 1

    // Substep 2

    // TODO Determine if response was retrieved over HTTPS
    // TODO Servo needs to decide what ciphers are to be treated as "deprecated"
    response.https_state = HttpsState::None;

    // TODO Read request

    // Step 6-11
    // (needs stream bodies)

    // Step 12
    // TODO when https://bugzilla.mozilla.org/show_bug.cgi?id=1030660
    // is resolved, this step will become uneccesary
    // TODO this step
    if let Some(encoding) = response.headers.typed_get::<ContentEncoding>() {
        if encoding.contains("gzip") {
        } else if encoding.contains("compress") {
        }
    };

    // Step 13
    // TODO this step isn't possible yet (CSP)

    // Step 14, update the cached response, done via the shared response body.

    // TODO this step isn't possible yet
    // Step 15
    if credentials_flag {
        set_cookies_from_headers(&url, &response.headers, &context.state.cookie_jar);
    }

    // TODO these steps
    // Step 16
    // Substep 1
    // Substep 2
    // Sub-substep 1
    // Sub-substep 2
    // Sub-substep 3
    // Sub-substep 4
    // Substep 3

    // Step 16

    // Ensure we don't override "responseEnd" on successful return of this function
    response_end_timer.neuter();

    response
}

/// [CORS preflight fetch](https://fetch.spec.whatwg.org#cors-preflight-fetch)
fn cors_preflight_fetch(
    request: &Request,
    cache: &mut CorsCache,
    context: &FetchContext,
) -> Response {
    // Step 1
    let mut preflight = RequestBuilder::new(request.current_url())
        .method(Method::OPTIONS)
        .origin(match &request.origin {
            Origin::Client => {
                unreachable!("We shouldn't get Client origin in cors_preflight_fetch.")
            },
            Origin::Origin(origin) => origin.clone(),
        })
        .pipeline_id(request.pipeline_id)
        .initiator(request.initiator.clone())
        .destination(request.destination.clone())
        .referrer(Some(request.referrer.clone()))
        .referrer_policy(request.referrer_policy)
        .build();

    // Step 2
    preflight
        .headers
        .typed_insert::<AccessControlRequestMethod>(AccessControlRequestMethod::from(
            request.method.clone(),
        ));

    // Step 3
    let mut headers = request
        .headers
        .iter()
        .filter(|(name, value)| !is_cors_safelisted_request_header(&name, &value))
        .map(|(name, _)| name.as_str())
        .collect::<Vec<&str>>();
    headers.sort();
    let headers = headers
        .iter()
        .filter_map(|name| HeaderName::from_str(name).ok())
        .collect::<Vec<HeaderName>>();

    // Step 4
    if !headers.is_empty() {
        preflight
            .headers
            .typed_insert(AccessControlRequestHeaders::from_iter(headers));
    }

    // Step 5
    let response = http_network_or_cache_fetch(&mut preflight, false, false, &mut None, context);

    // Step 6
    if cors_check(&request, &response).is_ok() &&
        response
            .status
            .as_ref()
            .map_or(false, |(status, _)| status.is_success())
    {
        // Substep 1, 2
        let mut methods = if response
            .headers
            .contains_key(header::ACCESS_CONTROL_ALLOW_METHODS)
        {
            match response.headers.typed_get::<AccessControlAllowMethods>() {
                Some(methods) => methods.iter().collect(),
                // Substep 4
                None => {
                    return Response::network_error(NetworkError::Internal(
                        "CORS ACAM check failed".into(),
                    ));
                },
            }
        } else {
            vec![]
        };

        // Substep 3
        let header_names = if response
            .headers
            .contains_key(header::ACCESS_CONTROL_ALLOW_HEADERS)
        {
            match response.headers.typed_get::<AccessControlAllowHeaders>() {
                Some(names) => names.iter().collect(),
                // Substep 4
                None => {
                    return Response::network_error(NetworkError::Internal(
                        "CORS ACAH check failed".into(),
                    ));
                },
            }
        } else {
            vec![]
        };

        // Substep 5
        if (methods.iter().any(|m| m.as_ref() == "*") ||
            header_names.iter().any(|hn| hn.as_str() == "*")) &&
            request.credentials_mode == CredentialsMode::Include
        {
            return Response::network_error(NetworkError::Internal(
                "CORS ACAH/ACAM and request credentials mode mismatch".into(),
            ));
        }

        // Substep 6
        if methods.is_empty() && request.use_cors_preflight {
            methods = vec![request.method.clone()];
        }

        // Substep 7
        debug!(
            "CORS check: Allowed methods: {:?}, current method: {:?}",
            methods, request.method
        );
        if methods.iter().all(|method| *method != request.method) &&
            !is_cors_safelisted_method(&request.method) &&
            methods.iter().all(|m| m.as_ref() != "*")
        {
            return Response::network_error(NetworkError::Internal(
                "CORS method check failed".into(),
            ));
        }

        // Substep 8
        if request.headers.iter().any(|(name, _)| {
            name == header::AUTHORIZATION && header_names.iter().all(|hn| hn != name)
        }) {
            return Response::network_error(NetworkError::Internal(
                "CORS authorization check failed".into(),
            ));
        }

        // Substep 9
        debug!(
            "CORS check: Allowed headers: {:?}, current headers: {:?}",
            header_names, request.headers
        );
        let set: HashSet<&HeaderName> = HashSet::from_iter(header_names.iter());
        if request.headers.iter().any(|(name, value)| {
            !set.contains(name) && !is_cors_safelisted_request_header(&name, &value)
        }) {
            return Response::network_error(NetworkError::Internal(
                "CORS headers check failed".into(),
            ));
        }

        // Substep 10, 11
        let max_age: Duration = response
            .headers
            .typed_get::<AccessControlMaxAge>()
            .map(|acma| acma.into())
            .unwrap_or(Duration::from_secs(0));
        let max_age = max_age.as_secs() as u32;
        // Substep 12
        // TODO: Need to define what an imposed limit on max-age is

        // Substep 13 ignored, we do have a CORS cache

        // Substep 14, 15
        for method in &methods {
            cache.match_method_and_update(&*request, method.clone(), max_age);
        }

        // Substep 16, 17
        for header_name in &header_names {
            cache.match_header_and_update(&*request, &*header_name, max_age);
        }

        // Substep 18
        return response;
    }

    // Step 7
    Response::network_error(NetworkError::Internal("CORS check failed".into()))
}

/// [CORS check](https://fetch.spec.whatwg.org#concept-cors-check)
fn cors_check(request: &Request, response: &Response) -> Result<(), ()> {
    // Step 1
    let origin = response.headers.typed_get::<AccessControlAllowOrigin>();

    // Step 2
    let origin = origin.ok_or(())?;

    // Step 3
    if request.credentials_mode != CredentialsMode::Include &&
        origin == AccessControlAllowOrigin::ANY
    {
        return Ok(());
    }

    // Step 4
    let origin = match origin.origin() {
        Some(origin) => origin,
        // if it's Any or Null at this point, there's nothing to do but return Err(())
        None => return Err(()),
    };

    match request.origin {
        Origin::Origin(ref o) if o.ascii_serialization() == origin.to_string().trim() => {},
        _ => return Err(()),
    }

    // Step 5
    if request.credentials_mode != CredentialsMode::Include {
        return Ok(());
    }

    // Step 6
    let credentials = response
        .headers
        .typed_get::<AccessControlAllowCredentials>();

    // Step 7
    if credentials.is_some() {
        return Ok(());
    }

    // Step 8
    Err(())
}

fn has_credentials(url: &ServoUrl) -> bool {
    !url.username().is_empty() || url.password().is_some()
}

fn is_no_store_cache(headers: &HeaderMap) -> bool {
    headers.contains_key(header::IF_MODIFIED_SINCE) |
        headers.contains_key(header::IF_NONE_MATCH) |
        headers.contains_key(header::IF_UNMODIFIED_SINCE) |
        headers.contains_key(header::IF_MATCH) |
        headers.contains_key(header::IF_RANGE)
}

/// <https://fetch.spec.whatwg.org/#redirect-status>
pub fn is_redirect_status(status: &(StatusCode, String)) -> bool {
    match status.0 {
        StatusCode::MOVED_PERMANENTLY |
        StatusCode::FOUND |
        StatusCode::SEE_OTHER |
        StatusCode::TEMPORARY_REDIRECT |
        StatusCode::PERMANENT_REDIRECT => true,
        _ => false,
    }
}
