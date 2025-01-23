/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use std::borrow::Cow;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};
use std::{io, mem, str};

use base64::engine::general_purpose;
use base64::Engine as _;
use content_security_policy as csp;
use crossbeam_channel::Sender;
use devtools_traits::DevtoolsControlMsg;
use headers::{AccessControlExposeHeaders, ContentType, HeaderMapExt};
use http::header::{self, HeaderMap, HeaderName};
use http::{Method, StatusCode};
use ipc_channel::ipc;
use log::warn;
use mime::{self, Mime};
use net_traits::filemanager_thread::{FileTokenCheck, RelativePos};
use net_traits::http_status::HttpStatus;
use net_traits::policy_container::{PolicyContainer, RequestPolicyContainer};
use net_traits::request::{
    is_cors_safelisted_method, is_cors_safelisted_request_header, BodyChunkRequest,
    BodyChunkResponse, CredentialsMode, Destination, Origin, RedirectMode, Referrer, Request,
    RequestMode, ResponseTainting, Window,
};
use net_traits::response::{Response, ResponseBody, ResponseType};
use net_traits::{
    set_default_accept_language, FetchTaskTarget, NetworkError, ReferrerPolicy, ResourceAttribute,
    ResourceFetchTiming, ResourceTimeValue, ResourceTimingType,
};
use rustls_pki_types::CertificateDer;
use serde::{Deserialize, Serialize};
use servo_arc::Arc as ServoArc;
use servo_url::ServoUrl;
use tokio::sync::mpsc::{UnboundedReceiver as TokioReceiver, UnboundedSender as TokioSender};

use super::fetch_params::FetchParams;
use crate::fetch::cors_cache::CorsCache;
use crate::fetch::headers::determine_nosniff;
use crate::filemanager_thread::FileManager;
use crate::http_loader::{determine_requests_referrer, http_fetch, set_default_accept, HttpState};
use crate::protocols::ProtocolRegistry;
use crate::request_intercepter::RequestIntercepter;
use crate::subresource_integrity::is_response_integrity_valid;

pub type Target<'a> = &'a mut (dyn FetchTaskTarget + Send);

#[derive(Clone, Deserialize, Serialize)]
pub enum Data {
    Payload(Vec<u8>),
    Done,
    Cancelled,
}

pub struct FetchContext {
    pub state: Arc<HttpState>,
    pub user_agent: Cow<'static, str>,
    pub devtools_chan: Option<Arc<Mutex<Sender<DevtoolsControlMsg>>>>,
    pub filemanager: Arc<Mutex<FileManager>>,
    pub file_token: FileTokenCheck,
    pub request_intercepter: Arc<Mutex<RequestIntercepter>>,
    pub cancellation_listener: Arc<CancellationListener>,
    pub timing: ServoArc<Mutex<ResourceFetchTiming>>,
    pub protocols: Arc<ProtocolRegistry>,
}

#[derive(Default)]
pub struct CancellationListener {
    cancelled: AtomicBool,
}

impl CancellationListener {
    pub(crate) fn cancelled(&self) -> bool {
        self.cancelled.load(Ordering::Relaxed)
    }

    pub(crate) fn cancel(&self) {
        self.cancelled.store(true, Ordering::Relaxed)
    }
}
pub type DoneChannel = Option<(TokioSender<Data>, TokioReceiver<Data>)>;

/// [Fetch](https://fetch.spec.whatwg.org#concept-fetch)
pub async fn fetch(request: Request, target: Target<'_>, context: &FetchContext) {
    // Steps 7,4 of https://w3c.github.io/resource-timing/#processing-model
    // rev order okay since spec says they're equal - https://w3c.github.io/resource-timing/#dfn-starttime
    {
        let mut timing_guard = context.timing.lock().unwrap();
        timing_guard.set_attribute(ResourceAttribute::FetchStart);
        timing_guard.set_attribute(ResourceAttribute::StartTime(ResourceTimeValue::FetchStart));
    }
    fetch_with_cors_cache(request, &mut CorsCache::default(), target, context).await;
}

/// Continuation of fetch from step 8.
///
/// <https://fetch.spec.whatwg.org#concept-fetch>
pub async fn fetch_with_cors_cache(
    request: Request,
    cache: &mut CorsCache,
    target: Target<'_>,
    context: &FetchContext,
) {
    // Step 8: Let fetchParams be a new fetch params whose request is request
    let mut fetch_params = FetchParams::new(request);
    let request = &mut fetch_params.request;

    // Step 9: If request’s window is "client", then set request’s window to request’s client, if
    // request’s client’s global object is a Window object; otherwise "no-window".
    if request.window == Window::Client {
        // TODO: Set window to request's client object if client is a Window object
    } else {
        request.window = Window::NoWindow;
    }

    // Step 10: If request’s origin is "client", then set request’s origin to request’s client’s
    // origin.
    if request.origin == Origin::Client {
        // TODO: set request's origin to request's client's origin
        unimplemented!()
    }

    // Step 11: If all of the following conditions are true:
    // - request’s URL’s scheme is an HTTP(S) scheme
    // - request’s mode is "same-origin", "cors", or "no-cors"
    // - request’s window is an environment settings object
    // - request’s method is `GET`
    // - request’s unsafe-request flag is not set or request’s header list is empty
    // TODO: evaluate these conditions when we have an an environment settings object

    // Step 12: If request’s policy container is "client", then:
    if let RequestPolicyContainer::Client = request.policy_container {
        // Step 12.1: If request’s client is non-null, then set request’s policy container to a clone
        // of request’s client’s policy container.
        // TODO: Requires request's client to support PolicyContainer

        // Step 12.2: Otherwise, set request’s policy container to a new policy container.
        request.policy_container =
            RequestPolicyContainer::PolicyContainer(PolicyContainer::default());
    }

    // Step 13: If request’s header list does not contain `Accept`:
    set_default_accept(request);

    // Step 14: If request’s header list does not contain `Accept-Language`, then user agents should
    // append (`Accept-Language, an appropriate header value) to request’s header list.
    set_default_accept_language(&mut request.headers);

    // Step 15. If request’s internal priority is null, then use request’s priority, initiator,
    // destination, and render-blocking in an implementation-defined manner to set request’s
    // internal priority to an implementation-defined object.
    // TODO: figure out what a Priority object is.

    // Step 16: If request is a subresource request, then:
    if request.is_subresource_request() {
        // TODO: requires keepalive.
    }

    // Step 17: Run main fetch given fetchParams.
    main_fetch(&mut fetch_params, cache, false, target, &mut None, context).await;

    // Step 18: Return fetchParams’s controller.
    // TODO: We don't implement fetchParams as defined in the spec
}

/// <https://www.w3.org/TR/CSP/#should-block-request>
pub fn should_request_be_blocked_by_csp(
    request: &Request,
    policy_container: &PolicyContainer,
) -> csp::CheckResult {
    let origin = match &request.origin {
        Origin::Client => return csp::CheckResult::Allowed,
        Origin::Origin(origin) => origin,
    };

    let csp_request = csp::Request {
        url: request.url().into_url(),
        origin: origin.clone().into_url_origin(),
        redirect_count: request.redirect_count,
        destination: request.destination,
        initiator: csp::Initiator::None,
        nonce: String::new(),
        integrity_metadata: request.integrity_metadata.clone(),
        parser_metadata: csp::ParserMetadata::None,
    };

    // TODO: Instead of ignoring violations, report them.
    policy_container
        .csp_list
        .as_ref()
        .map(|c| c.should_request_be_blocked(&csp_request).0)
        .unwrap_or(csp::CheckResult::Allowed)
}

pub fn maybe_intercept_request(
    request: &mut Request,
    context: &FetchContext,
    response: &mut Option<Response>,
) {
    context
        .request_intercepter
        .lock()
        .unwrap()
        .intercept_request(request, response, context);
}

/// [Main fetch](https://fetch.spec.whatwg.org/#concept-main-fetch)
pub async fn main_fetch(
    fetch_params: &mut FetchParams,
    cache: &mut CorsCache,
    recursive_flag: bool,
    target: Target<'_>,
    done_chan: &mut DoneChannel,
    context: &FetchContext,
) -> Response {
    // Step 1: Let request be fetchParam's request.
    let request = &mut fetch_params.request;

    // Step 2: Let response be null.
    let mut response = None;

    // Servo internal: return a crash error when a crash error page is needed
    if let Some(ref details) = request.crash {
        response = Some(Response::network_error(NetworkError::Crash(
            details.clone(),
        )));
    }

    // Step 3: If request’s local-URLs-only flag is set and request’s
    // current URL is not local, then set response to a network error.
    if request.local_urls_only &&
        !matches!(
            request.current_url().scheme(),
            "about" | "blob" | "data" | "filesystem"
        )
    {
        response = Some(Response::network_error(NetworkError::Internal(
            "Non-local scheme".into(),
        )));
    }

    // Step 2.2.
    // TODO: Report violations.

    // The request should have a valid policy_container associated with it.
    // TODO: This should not be `Client` here
    let policy_container = match &request.policy_container {
        RequestPolicyContainer::Client => PolicyContainer::default(),
        RequestPolicyContainer::PolicyContainer(container) => container.to_owned(),
    };

    // Step 2.4.
    if should_request_be_blocked_by_csp(request, &policy_container) == csp::CheckResult::Blocked {
        warn!("Request blocked by CSP");
        response = Some(Response::network_error(NetworkError::Internal(
            "Blocked by Content-Security-Policy".into(),
        )))
    }

    // Step 3.
    // TODO: handle request abort.

    // Step 4.
    // TODO: handle upgrade to a potentially secure URL.

    // Step 5.
    if should_be_blocked_due_to_bad_port(&request.current_url()) {
        response = Some(Response::network_error(NetworkError::Internal(
            "Request attempted on bad port".into(),
        )));
    }
    // TODO: handle blocking as mixed content.
    // TODO: handle blocking by content security policy.

    // Step 8: If request’s referrer policy is the empty string, then set request’s referrer policy
    // to request’s policy container’s referrer policy.
    if request.referrer_policy == ReferrerPolicy::EmptyString {
        request.referrer_policy = policy_container.get_referrer_policy();
    }

    let referrer_url = match mem::replace(&mut request.referrer, Referrer::NoReferrer) {
        Referrer::NoReferrer => None,
        Referrer::ReferrerUrl(referrer_source) | Referrer::Client(referrer_source) => {
            request.headers.remove(header::REFERER);
            determine_requests_referrer(
                request.referrer_policy,
                referrer_source,
                request.current_url(),
            )
        },
    };
    request.referrer = referrer_url.map_or(Referrer::NoReferrer, Referrer::ReferrerUrl);

    // Step 9.
    // TODO: handle FTP URLs.

    // Step 10.
    context
        .state
        .hsts_list
        .read()
        .unwrap()
        .apply_hsts_rules(request.current_url_mut());

    // Step 11.
    // Not applicable: see fetch_async.

    // Step 12.

    let current_url = request.current_url();
    let current_scheme = current_url.scheme();

    // Intercept the request and maybe override the response.
    maybe_intercept_request(request, context, &mut response);

    let mut response = match response {
        Some(res) => res,
        None => {
            let same_origin = if let Origin::Origin(ref origin) = request.origin {
                *origin == current_url.origin()
            } else {
                false
            };

            // request's current URL's origin is same origin with request's origin, and request's
            // response tainting is "basic"
            if (same_origin && request.response_tainting == ResponseTainting::Basic) ||
                // request's current URL's scheme is "data"
                current_scheme == "data" ||
                // request's mode is "navigate" or "websocket"
                matches!(
                    request.mode,
                    RequestMode::Navigate | RequestMode::WebSocket { .. }
                )
            {
                // Substep 1. Set request’s response tainting to "basic".
                request.response_tainting = ResponseTainting::Basic;

                // Substep 2. Return the result of running scheme fetch given fetchParams.
                scheme_fetch(fetch_params, cache, target, done_chan, context).await
            } else if request.mode == RequestMode::SameOrigin {
                Response::network_error(NetworkError::Internal("Cross-origin response".into()))
            } else if request.mode == RequestMode::NoCors {
                // Substep 1. If request’s redirect mode is not "follow", then return a network error.
                if request.redirect_mode != RedirectMode::Follow {
                    Response::network_error(NetworkError::Internal(
                        "NoCors requests must follow redirects".into(),
                    ))
                } else {
                    // Substep 2. Set request’s response tainting to "opaque".
                    request.response_tainting = ResponseTainting::Opaque;

                    // Substep 3. Return the result of running scheme fetch given fetchParams.
                    scheme_fetch(fetch_params, cache, target, done_chan, context).await
                }
            } else if !matches!(current_scheme, "http" | "https") {
                Response::network_error(NetworkError::Internal("Non-http scheme".into()))
            } else if request.use_cors_preflight ||
                (request.unsafe_request &&
                    (!is_cors_safelisted_method(&request.method) ||
                        request.headers.iter().any(|(name, value)| {
                            !is_cors_safelisted_request_header(&name, &value)
                        })))
            {
                // Substep 1.
                request.response_tainting = ResponseTainting::CorsTainting;
                // Substep 2.
                let response = http_fetch(
                    fetch_params,
                    cache,
                    true,
                    true,
                    false,
                    target,
                    done_chan,
                    context,
                )
                .await;
                // Substep 3.
                if response.is_network_error() {
                    // TODO clear cache entries using request
                }
                // Substep 4.
                response
            } else {
                // Substep 1.
                request.response_tainting = ResponseTainting::CorsTainting;
                // Substep 2.
                http_fetch(
                    fetch_params,
                    cache,
                    true,
                    false,
                    false,
                    target,
                    done_chan,
                    context,
                )
                .await
            }
        },
    };

    // Step 13.
    if recursive_flag {
        return response;
    }

    // reborrow request to avoid double mutable borrow
    let request = &mut fetch_params.request;

    // Step 14.
    let mut response = if !response.is_network_error() && response.internal_response.is_none() {
        // Substep 1.
        if request.response_tainting == ResponseTainting::CorsTainting {
            // Subsubstep 1.
            let header_names: Option<Vec<HeaderName>> = response
                .headers
                .typed_get::<AccessControlExposeHeaders>()
                .map(|v| v.iter().collect());
            match header_names {
                // Subsubstep 2.
                Some(ref list)
                    if request.credentials_mode != CredentialsMode::Include &&
                        list.iter().any(|header| header == "*") =>
                {
                    response.cors_exposed_header_name_list = response
                        .headers
                        .iter()
                        .map(|(name, _)| name.as_str().to_owned())
                        .collect();
                },
                // Subsubstep 3.
                Some(list) => {
                    response.cors_exposed_header_name_list =
                        list.iter().map(|h| h.as_str().to_owned()).collect();
                },
                _ => (),
            }
        }

        // Substep 2.
        let response_type = match request.response_tainting {
            ResponseTainting::Basic => ResponseType::Basic,
            ResponseTainting::CorsTainting => ResponseType::Cors,
            ResponseTainting::Opaque => ResponseType::Opaque,
        };
        response.to_filtered(response_type)
    } else {
        response
    };

    let internal_error = {
        // Tests for steps 17 and 18, before step 15 for borrowing concerns.
        let response_is_network_error = response.is_network_error();
        let should_replace_with_nosniff_error = !response_is_network_error &&
            should_be_blocked_due_to_nosniff(request.destination, &response.headers);
        let should_replace_with_mime_type_error = !response_is_network_error &&
            should_be_blocked_due_to_mime_type(request.destination, &response.headers);

        // Step 15.
        let mut network_error_response = response
            .get_network_error()
            .cloned()
            .map(Response::network_error);
        let internal_response = if let Some(error_response) = network_error_response.as_mut() {
            error_response
        } else {
            response.actual_response_mut()
        };

        // Step 16.
        if internal_response.url_list.is_empty() {
            internal_response.url_list.clone_from(&request.url_list)
        }

        // Step 17.
        // TODO: handle blocking as mixed content.
        // TODO: handle blocking by content security policy.
        let blocked_error_response;
        let internal_response = if should_replace_with_nosniff_error {
            // Defer rebinding result
            blocked_error_response =
                Response::network_error(NetworkError::Internal("Blocked by nosniff".into()));
            &blocked_error_response
        } else if should_replace_with_mime_type_error {
            // Defer rebinding result
            blocked_error_response =
                Response::network_error(NetworkError::Internal("Blocked by mime type".into()));
            &blocked_error_response
        } else {
            internal_response
        };

        // Step 18.
        // We check `internal_response` since we did not mutate `response`
        // in the previous step.
        let not_network_error = !response_is_network_error && !internal_response.is_network_error();
        if not_network_error &&
            (is_null_body_status(&internal_response.status) ||
                matches!(request.method, Method::HEAD | Method::CONNECT))
        {
            // when Fetch is used only asynchronously, we will need to make sure
            // that nothing tries to write to the body at this point
            let mut body = internal_response.body.lock().unwrap();
            *body = ResponseBody::Empty;
        }

        internal_response.get_network_error().cloned()
    };

    // Execute deferred rebinding of response.
    let mut response = if let Some(error) = internal_error {
        Response::network_error(error)
    } else {
        response
    };

    // Step 19.
    let mut response_loaded = false;
    let mut response = if !response.is_network_error() && !request.integrity_metadata.is_empty() {
        // Step 19.1.
        wait_for_response(request, &mut response, target, done_chan).await;
        response_loaded = true;

        // Step 19.2.
        let integrity_metadata = &request.integrity_metadata;
        if response.termination_reason.is_none() &&
            !is_response_integrity_valid(integrity_metadata, &response)
        {
            Response::network_error(NetworkError::Internal(
                "Subresource integrity validation failed".into(),
            ))
        } else {
            response
        }
    } else {
        response
    };

    // Step 20.
    if request.synchronous {
        // process_response is not supposed to be used
        // by sync fetch, but we overload it here for simplicity
        target.process_response(request, &response);
        if !response_loaded {
            wait_for_response(request, &mut response, target, done_chan).await;
        }
        // overloaded similarly to process_response
        target.process_response_eof(request, &response);
        return response;
    }

    // Step 21.
    if request.body.is_some() && matches!(current_scheme, "http" | "https") {
        // XXXManishearth: We actually should be calling process_request
        // in http_network_fetch. However, we can't yet follow the request
        // upload progress, so I'm keeping it here for now and pretending
        // the body got sent in one chunk
        target.process_request_body(request);
        target.process_request_eof(request);
    }

    // Step 22.
    target.process_response(request, &response);

    // Step 23.
    if !response_loaded {
        wait_for_response(request, &mut response, target, done_chan).await;
    }

    // Step 24.
    target.process_response_eof(request, &response);

    if let Ok(http_cache) = context.state.http_cache.write() {
        http_cache.update_awaiting_consumers(request, &response);
    }

    // Steps 25-27.
    // TODO: remove this line when only asynchronous fetches are used
    response
}

async fn wait_for_response(
    request: &Request,
    response: &mut Response,
    target: Target<'_>,
    done_chan: &mut DoneChannel,
) {
    if let Some(ref mut ch) = *done_chan {
        loop {
            match ch.1.recv().await {
                Some(Data::Payload(vec)) => {
                    target.process_response_chunk(request, vec);
                },
                Some(Data::Done) => {
                    break;
                },
                Some(Data::Cancelled) => {
                    response.aborted.store(true, Ordering::Release);
                    break;
                },
                _ => {
                    panic!("fetch worker should always send Done before terminating");
                },
            }
        }
    } else {
        let body = response.actual_response().body.lock().unwrap();
        if let ResponseBody::Done(ref vec) = *body {
            // in case there was no channel to wait for, the body was
            // obtained synchronously via scheme_fetch for data/file/about/etc
            // We should still send the body across as a chunk
            target.process_response_chunk(request, vec.clone());
        } else {
            assert_eq!(*body, ResponseBody::Empty)
        }
    }
}

/// Range header start and end values.
pub enum RangeRequestBounds {
    /// The range bounds are known and set to final values.
    Final(RelativePos),
    /// We need extra information to set the range bounds.
    /// i.e. buffer or file size.
    Pending(u64),
}

impl RangeRequestBounds {
    pub fn get_final(&self, len: Option<u64>) -> Result<RelativePos, &'static str> {
        match self {
            RangeRequestBounds::Final(pos) => {
                if let Some(len) = len {
                    if pos.start <= len as i64 {
                        return Ok(pos.clone());
                    }
                }
                Err("Tried to process RangeRequestBounds::Final without len")
            },
            RangeRequestBounds::Pending(offset) => Ok(RelativePos::from_opts(
                if let Some(len) = len {
                    Some((len - u64::min(len, *offset)) as i64)
                } else {
                    Some(0)
                },
                None,
            )),
        }
    }
}

fn create_blank_reply(url: ServoUrl, timing_type: ResourceTimingType) -> Response {
    let mut response = Response::new(url, ResourceFetchTiming::new(timing_type));
    response
        .headers
        .typed_insert(ContentType::from(mime::TEXT_HTML_UTF_8));
    *response.body.lock().unwrap() = ResponseBody::Done(vec![]);
    response.status = HttpStatus::default();
    response
}

/// Handle a request from the user interface to ignore validation errors for a certificate.
fn handle_allowcert_request(request: &mut Request, context: &FetchContext) -> io::Result<()> {
    let error = |string| Err(io::Error::new(io::ErrorKind::Other, string));

    let body = match request.body.as_mut() {
        Some(body) => body,
        None => return error("No body found"),
    };

    let stream = body.take_stream();
    let stream = stream.lock().unwrap();
    let (body_chan, body_port) = ipc::channel().unwrap();
    let _ = stream.send(BodyChunkRequest::Connect(body_chan));
    let _ = stream.send(BodyChunkRequest::Chunk);
    let body_bytes = match body_port.recv().ok() {
        Some(BodyChunkResponse::Chunk(bytes)) => bytes,
        _ => return error("Certificate not sent in a single chunk"),
    };

    let split_idx = match body_bytes.iter().position(|b| *b == b'&') {
        Some(split_idx) => split_idx,
        None => return error("Could not find ampersand in data"),
    };
    let (secret, cert_base64) = body_bytes.split_at(split_idx);

    let secret = str::from_utf8(secret).ok().and_then(|s| s.parse().ok());
    if secret != Some(*net_traits::PRIVILEGED_SECRET) {
        return error("Invalid secret sent. Ignoring request");
    }

    let cert_bytes = match general_purpose::STANDARD_NO_PAD.decode(&cert_base64[1..]) {
        Ok(bytes) => bytes,
        Err(_) => return error("Could not decode certificate base64"),
    };

    context
        .state
        .override_manager
        .add_override(&CertificateDer::from_slice(&cert_bytes).into_owned());
    Ok(())
}

/// [Scheme fetch](https://fetch.spec.whatwg.org#scheme-fetch)
async fn scheme_fetch(
    fetch_params: &mut FetchParams,
    cache: &mut CorsCache,
    target: Target<'_>,
    done_chan: &mut DoneChannel,
    context: &FetchContext,
) -> Response {
    // Step 1: If fetchParams is canceled, then return the appropriate network error for fetchParams.

    // Step 2: Let request be fetchParams’s request.
    let request = &mut fetch_params.request;
    let url = request.current_url();

    let scheme = url.scheme();
    match scheme {
        "about" if url.path() == "blank" => create_blank_reply(url, request.timing_type()),

        "chrome" if url.path() == "allowcert" => {
            if let Err(error) = handle_allowcert_request(request, context) {
                warn!("Could not handle allowcert request: {error}");
            }
            create_blank_reply(url, request.timing_type())
        },

        "http" | "https" => {
            http_fetch(
                fetch_params,
                cache,
                false,
                false,
                false,
                target,
                done_chan,
                context,
            )
            .await
        },

        _ => match context.protocols.get(scheme) {
            Some(handler) => handler.load(request, done_chan, context).await,
            None => Response::network_error(NetworkError::Internal("Unexpected scheme".into())),
        },
    }
}

fn is_null_body_status(status: &HttpStatus) -> bool {
    matches!(
        status.try_code(),
        Some(StatusCode::SWITCHING_PROTOCOLS) |
            Some(StatusCode::NO_CONTENT) |
            Some(StatusCode::RESET_CONTENT) |
            Some(StatusCode::NOT_MODIFIED)
    )
}

/// <https://fetch.spec.whatwg.org/#should-response-to-request-be-blocked-due-to-nosniff?>
pub fn should_be_blocked_due_to_nosniff(
    destination: Destination,
    response_headers: &HeaderMap,
) -> bool {
    // Step 1
    if !determine_nosniff(response_headers) {
        return false;
    }

    // Step 2
    // Note: an invalid MIME type will produce a `None`.
    let content_type_header = response_headers.typed_get::<ContentType>();

    /// <https://html.spec.whatwg.org/multipage/#scriptingLanguages>
    #[inline]
    fn is_javascript_mime_type(mime_type: &Mime) -> bool {
        let javascript_mime_types: [Mime; 16] = [
            "application/ecmascript".parse().unwrap(),
            "application/javascript".parse().unwrap(),
            "application/x-ecmascript".parse().unwrap(),
            "application/x-javascript".parse().unwrap(),
            "text/ecmascript".parse().unwrap(),
            "text/javascript".parse().unwrap(),
            "text/javascript1.0".parse().unwrap(),
            "text/javascript1.1".parse().unwrap(),
            "text/javascript1.2".parse().unwrap(),
            "text/javascript1.3".parse().unwrap(),
            "text/javascript1.4".parse().unwrap(),
            "text/javascript1.5".parse().unwrap(),
            "text/jscript".parse().unwrap(),
            "text/livescript".parse().unwrap(),
            "text/x-ecmascript".parse().unwrap(),
            "text/x-javascript".parse().unwrap(),
        ];

        javascript_mime_types
            .iter()
            .any(|mime| mime.type_() == mime_type.type_() && mime.subtype() == mime_type.subtype())
    }

    match content_type_header {
        // Step 4
        Some(ref ct) if destination.is_script_like() => {
            !is_javascript_mime_type(&ct.clone().into())
        },

        // Step 5
        Some(ref ct) if destination == Destination::Style => {
            let m: mime::Mime = ct.clone().into();
            m.type_() != mime::TEXT && m.subtype() != mime::CSS
        },

        None if destination == Destination::Style || destination.is_script_like() => true,
        // Step 6
        _ => false,
    }
}

/// <https://fetch.spec.whatwg.org/#should-response-to-request-be-blocked-due-to-mime-type?>
fn should_be_blocked_due_to_mime_type(
    destination: Destination,
    response_headers: &HeaderMap,
) -> bool {
    // Step 1
    let mime_type: mime::Mime = match response_headers.typed_get::<ContentType>() {
        Some(header) => header.into(),
        None => return false,
    };

    // Step 2-3
    destination.is_script_like() &&
        match mime_type.type_() {
            mime::AUDIO | mime::VIDEO | mime::IMAGE => true,
            mime::TEXT if mime_type.subtype() == mime::CSV => true,
            // Step 4
            _ => false,
        }
}

/// <https://fetch.spec.whatwg.org/#block-bad-port>
pub fn should_be_blocked_due_to_bad_port(url: &ServoUrl) -> bool {
    // Step 1 is not applicable, this function just takes the URL directly.

    // Step 2.
    let scheme = url.scheme();

    // Step 3.
    // If there is no explicit port, this means the default one is used for
    // the given scheme, and thus this means the request should not be blocked
    // due to a bad port.
    let port = if let Some(port) = url.port() {
        port
    } else {
        return false;
    };

    // Step 4.
    if scheme == "ftp" && (port == 20 || port == 21) {
        return false;
    }

    // Step 5.
    if is_network_scheme(scheme) && is_bad_port(port) {
        return true;
    }

    // Step 6.
    false
}

/// <https://fetch.spec.whatwg.org/#network-scheme>
fn is_network_scheme(scheme: &str) -> bool {
    scheme == "ftp" || scheme == "http" || scheme == "https"
}

/// <https://fetch.spec.whatwg.org/#bad-port>
fn is_bad_port(port: u16) -> bool {
    static BAD_PORTS: [u16; 78] = [
        1, 7, 9, 11, 13, 15, 17, 19, 20, 21, 22, 23, 25, 37, 42, 43, 53, 69, 77, 79, 87, 95, 101,
        102, 103, 104, 109, 110, 111, 113, 115, 117, 119, 123, 135, 137, 139, 143, 161, 179, 389,
        427, 465, 512, 513, 514, 515, 526, 530, 531, 532, 540, 548, 554, 556, 563, 587, 601, 636,
        993, 995, 1719, 1720, 1723, 2049, 3659, 4045, 5060, 5061, 6000, 6566, 6665, 6666, 6667,
        6668, 6669, 6697, 10080,
    ];

    BAD_PORTS.binary_search(&port).is_ok()
}
