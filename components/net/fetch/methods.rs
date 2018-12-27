/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use crate::data_loader::decode;
use crate::fetch::cors_cache::CorsCache;
use crate::filemanager_thread::{fetch_file_in_chunks, FileManager, FILE_CHUNK_SIZE};
use crate::http_loader::{determine_request_referrer, http_fetch, HttpState};
use crate::http_loader::{set_default_accept, set_default_accept_language};
use crate::subresource_integrity::is_response_integrity_valid;
use crossbeam_channel::{unbounded, Receiver, Sender};
use devtools_traits::DevtoolsControlMsg;
use headers_core::HeaderMapExt;
use headers_ext::{AccessControlExposeHeaders, ContentType, Range};
use http::header::{self, HeaderMap, HeaderName, HeaderValue};
use hyper::Method;
use hyper::StatusCode;
use ipc_channel::ipc::IpcReceiver;
use mime::{self, Mime};
use mime_guess::guess_mime_type;
use net_traits::blob_url_store::{parse_blob_url, BlobURLStoreError};
use net_traits::filemanager_thread::RelativePos;
use net_traits::request::{CredentialsMode, Destination, Referrer, Request, RequestMode};
use net_traits::request::{Origin, ResponseTainting, Window};
use net_traits::response::{Response, ResponseBody, ResponseType};
use net_traits::{FetchTaskTarget, NetworkError, ReferrerPolicy, ResourceFetchTiming};
use servo_url::ServoUrl;
use std::borrow::Cow;
use std::fs::File;
use std::io::{BufReader, Seek, SeekFrom};
use std::mem;
use std::ops::Bound;
use std::str;
use std::sync::atomic::Ordering;
use std::sync::{Arc, Mutex};

lazy_static! {
    static ref X_CONTENT_TYPE_OPTIONS: HeaderName =
        HeaderName::from_static("x-content-type-options");
}

pub type Target<'a> = &'a mut (dyn FetchTaskTarget + Send);

pub enum Data {
    Payload(Vec<u8>),
    Done,
    Cancelled,
}

pub struct FetchContext {
    pub state: Arc<HttpState>,
    pub user_agent: Cow<'static, str>,
    pub devtools_chan: Option<Sender<DevtoolsControlMsg>>,
    pub filemanager: FileManager,
    pub cancellation_listener: Arc<Mutex<CancellationListener>>,
    pub timing: Arc<Mutex<ResourceFetchTiming>>,
}

pub struct CancellationListener {
    cancel_chan: Option<IpcReceiver<()>>,
    cancelled: bool,
}

impl CancellationListener {
    pub fn new(cancel_chan: Option<IpcReceiver<()>>) -> Self {
        Self {
            cancel_chan: cancel_chan,
            cancelled: false,
        }
    }

    pub fn cancelled(&mut self) -> bool {
        if let Some(ref cancel_chan) = self.cancel_chan {
            if self.cancelled {
                true
            } else if cancel_chan.try_recv().is_ok() {
                self.cancelled = true;
                true
            } else {
                false
            }
        } else {
            false
        }
    }
}
pub type DoneChannel = Option<(Sender<Data>, Receiver<Data>)>;

/// [Fetch](https://fetch.spec.whatwg.org#concept-fetch)
pub fn fetch(request: &mut Request, target: Target, context: &FetchContext) {
    fetch_with_cors_cache(request, &mut CorsCache::new(), target, context);
}

pub fn fetch_with_cors_cache(
    request: &mut Request,
    cache: &mut CorsCache,
    target: Target,
    context: &FetchContext,
) {
    // Step 1.
    if request.window == Window::Client {
        // TODO: Set window to request's client object if client is a Window object
    } else {
        request.window = Window::NoWindow;
    }

    // Step 2.
    if request.origin == Origin::Client {
        // TODO: set request's origin to request's client's origin
        unimplemented!()
    }

    // Step 3.
    set_default_accept(request.destination, &mut request.headers);

    // Step 4.
    set_default_accept_language(&mut request.headers);

    // Step 5.
    // TODO: figure out what a Priority object is.

    // Step 6.
    // TODO: handle client hints headers.

    // Step 7.
    if request.is_subresource_request() {
        // TODO: handle client hints headers.
    }

    // Step 8.
    main_fetch(request, cache, false, false, target, &mut None, &context);
}

/// [Main fetch](https://fetch.spec.whatwg.org/#concept-main-fetch)
pub fn main_fetch(
    request: &mut Request,
    cache: &mut CorsCache,
    cors_flag: bool,
    recursive_flag: bool,
    target: Target,
    done_chan: &mut DoneChannel,
    context: &FetchContext,
) -> Response {
    // Step 1.
    let mut response = None;

    // Step 2.
    if request.local_urls_only {
        if !matches!(
            request.current_url().scheme(),
            "about" | "blob" | "data" | "filesystem"
        ) {
            response = Some(Response::network_error(NetworkError::Internal(
                "Non-local scheme".into(),
            )));
        }
    }

    // Step 3.
    // TODO: handle content security policy violations.

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

    // Step 6
    // TODO: handle request's client's referrer policy.

    // Step 7.
    request.referrer_policy = request
        .referrer_policy
        .or(Some(ReferrerPolicy::NoReferrerWhenDowngrade));

    // Step 8.
    {
        let referrer_url = match mem::replace(&mut request.referrer, Referrer::NoReferrer) {
            Referrer::NoReferrer => None,
            Referrer::Client => {
                // FIXME(#14507): We should never get this value here; it should
                //                already have been handled in the script thread.
                request.headers.remove(header::REFERER);
                None
            },
            Referrer::ReferrerUrl(url) => {
                request.headers.remove(header::REFERER);
                let current_url = request.current_url();
                determine_request_referrer(
                    &mut request.headers,
                    request.referrer_policy.unwrap(),
                    url,
                    current_url,
                )
            },
        };
        if let Some(referrer_url) = referrer_url {
            request.referrer = Referrer::ReferrerUrl(referrer_url);
        }
    }

    // Step 9.
    // TODO: handle FTP URLs.

    // Step 10.
    context
        .state
        .hsts_list
        .read()
        .unwrap()
        .switch_known_hsts_host_domain_url_to_https(request.current_url_mut());

    // Step 11.
    // Not applicable: see fetch_async.

    // Step 12.
    let mut response = response.unwrap_or_else(|| {
        let current_url = request.current_url();
        let same_origin = if let Origin::Origin(ref origin) = request.origin {
            *origin == current_url.origin()
        } else {
            false
        };

        if (same_origin && !cors_flag ) ||
            current_url.scheme() == "data" ||
            current_url.scheme() == "file" || // FIXME: Fetch spec has already dropped filtering against file:
                                              //        and about: schemes, but CSS tests will break on loading Ahem
                                              //        since we load them through a file: URL.
            current_url.scheme() == "about" ||
            request.mode == RequestMode::Navigate
        {
            // Substep 1.
            request.response_tainting = ResponseTainting::Basic;

            // Substep 2.
            scheme_fetch(request, cache, target, done_chan, context)
        } else if request.mode == RequestMode::SameOrigin {
            Response::network_error(NetworkError::Internal("Cross-origin response".into()))
        } else if request.mode == RequestMode::NoCors {
            // Substep 1.
            request.response_tainting = ResponseTainting::Opaque;

            // Substep 2.
            scheme_fetch(request, cache, target, done_chan, context)
        } else if !matches!(current_url.scheme(), "http" | "https") {
            Response::network_error(NetworkError::Internal("Non-http scheme".into()))
        } else if request.use_cors_preflight ||
            (request.unsafe_request &&
                (!is_cors_safelisted_method(&request.method) || request
                    .headers
                    .iter()
                    .any(|(name, value)| !is_cors_safelisted_request_header(&name, &value))))
        {
            // Substep 1.
            request.response_tainting = ResponseTainting::CorsTainting;
            // Substep 2.
            let response = http_fetch(
                request, cache, true, true, false, target, done_chan, context,
            );
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
                request, cache, true, false, false, target, done_chan, context,
            )
        }
    });

    // Step 13.
    if recursive_flag {
        return response;
    }

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
                Some(ref list) if request.credentials_mode != CredentialsMode::Include => {
                    if list.len() == 1 && list[0] == "*" {
                        response.cors_exposed_header_name_list = response
                            .headers
                            .iter()
                            .map(|(name, _)| name.as_str().to_owned())
                            .collect();
                    }
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
            internal_response.url_list = request.url_list.clone();
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
            (is_null_body_status(&internal_response.status) || match request.method {
                Method::HEAD | Method::CONNECT => true,
                _ => false,
            }) {
            // when Fetch is used only asynchronously, we will need to make sure
            // that nothing tries to write to the body at this point
            let mut body = internal_response.body.lock().unwrap();
            *body = ResponseBody::Empty;
        }

        internal_response.get_network_error().map(|e| e.clone())
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
        wait_for_response(&mut response, target, done_chan);
        response_loaded = true;

        // Step 19.2.
        let ref integrity_metadata = &request.integrity_metadata;
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
        target.process_response(&mut response);
        if !response_loaded {
            wait_for_response(&mut response, target, done_chan);
        }
        // overloaded similarly to process_response
        target.process_response_eof(&response);
        return response;
    }

    // Step 21.
    if request.body.is_some() && matches!(request.current_url().scheme(), "http" | "https") {
        // XXXManishearth: We actually should be calling process_request
        // in http_network_fetch. However, we can't yet follow the request
        // upload progress, so I'm keeping it here for now and pretending
        // the body got sent in one chunk
        target.process_request_body(&request);
        target.process_request_eof(&request);
    }

    // Step 22.
    target.process_response(&response);

    // Step 23.
    if !response_loaded {
        wait_for_response(&mut response, target, done_chan);
    }

    // Step 24.
    target.process_response_eof(&response);

    if !response.is_network_error() {
        if let Ok(mut http_cache) = context.state.http_cache.write() {
            http_cache.update_awaiting_consumers(&request, &response);
        }
    }

    // Steps 25-27.
    // TODO: remove this line when only asynchronous fetches are used
    response
}

fn wait_for_response(response: &mut Response, target: Target, done_chan: &mut DoneChannel) {
    if let Some(ref ch) = *done_chan {
        loop {
            match ch
                .1
                .recv()
                .expect("fetch worker should always send Done before terminating")
            {
                Data::Payload(vec) => {
                    target.process_response_chunk(vec);
                },
                Data::Done => break,
                Data::Cancelled => {
                    response.aborted.store(true, Ordering::Relaxed);
                    break;
                },
            }
        }
    } else {
        let body = response.actual_response().body.lock().unwrap();
        if let ResponseBody::Done(ref vec) = *body {
            // in case there was no channel to wait for, the body was
            // obtained synchronously via scheme_fetch for data/file/about/etc
            // We should still send the body across as a chunk
            target.process_response_chunk(vec.clone());
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
    pub fn get_final(&self, len: Option<u64>) -> Result<RelativePos, ()> {
        match self {
            RangeRequestBounds::Final(pos) => {
                if let Some(len) = len {
                    if pos.start <= len as i64 {
                        return Ok(pos.clone());
                    }
                }
                Err(())
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

/// Get the range bounds if the `Range` header is present.
fn get_range_request_bounds(range: Option<Range>) -> RangeRequestBounds {
    if let Some(ref range) = range {
        let (start, end) = match range
            .iter()
            .collect::<Vec<(Bound<u64>, Bound<u64>)>>()
            .first()
        {
            Some(&(Bound::Included(start), Bound::Unbounded)) => (start, None),
            Some(&(Bound::Included(start), Bound::Included(end))) => {
                // `end` should be less or equal to `start`.
                (start, Some(i64::max(start as i64, end as i64)))
            },
            Some(&(Bound::Unbounded, Bound::Included(offset))) => {
                return RangeRequestBounds::Pending(offset);
            },
            _ => (0, None),
        };
        RangeRequestBounds::Final(RelativePos::from_opts(Some(start as i64), end))
    } else {
        RangeRequestBounds::Final(RelativePos::from_opts(Some(0), None))
    }
}

fn partial_content(response: &mut Response) {
    let reason = "Partial Content".to_owned();
    response.status = Some((StatusCode::PARTIAL_CONTENT, reason.clone()));
    response.raw_status = Some((StatusCode::PARTIAL_CONTENT.as_u16(), reason.into()));
}

fn range_not_satisfiable_error(response: &mut Response) {
    let reason = "Range Not Satisfiable".to_owned();
    response.status = Some((StatusCode::RANGE_NOT_SATISFIABLE, reason.clone()));
    response.raw_status = Some((StatusCode::RANGE_NOT_SATISFIABLE.as_u16(), reason.into()));
}

/// [Scheme fetch](https://fetch.spec.whatwg.org#scheme-fetch)
fn scheme_fetch(
    request: &mut Request,
    cache: &mut CorsCache,
    target: Target,
    done_chan: &mut DoneChannel,
    context: &FetchContext,
) -> Response {
    let url = request.current_url();

    match url.scheme() {
        "about" if url.path() == "blank" => {
            let mut response = Response::new(url, ResourceFetchTiming::new(request.timing_type()));
            response
                .headers
                .typed_insert(ContentType::from(mime::TEXT_HTML_UTF_8));
            *response.body.lock().unwrap() = ResponseBody::Done(vec![]);
            response.status = Some((StatusCode::OK, "OK".to_string()));
            response.raw_status = Some((StatusCode::OK.as_u16(), b"OK".to_vec()));
            response
        },

        "http" | "https" => http_fetch(
            request, cache, false, false, false, target, done_chan, context,
        ),

        "data" => match decode(&url) {
            Ok((mime, bytes)) => {
                let mut response =
                    Response::new(url, ResourceFetchTiming::new(request.timing_type()));
                *response.body.lock().unwrap() = ResponseBody::Done(bytes);
                response.headers.typed_insert(ContentType::from(mime));
                response.status = Some((StatusCode::OK, "OK".to_string()));
                response.raw_status = Some((StatusCode::OK.as_u16(), b"OK".to_vec()));
                response
            },
            Err(_) => {
                Response::network_error(NetworkError::Internal("Decoding data URL failed".into()))
            },
        },

        "file" => {
            if request.method != Method::GET {
                return Response::network_error(NetworkError::Internal(
                    "Unexpected method for file".into(),
                ));
            }
            if let Ok(file_path) = url.to_file_path() {
                if let Ok(file) = File::open(file_path.clone()) {
                    // Get range bounds (if any) and try to seek to the requested offset.
                    // If seeking fails, bail out with a NetworkError.
                    let file_size = match file.metadata() {
                        Ok(metadata) => Some(metadata.len()),
                        Err(_) => None,
                    };

                    let mut response =
                        Response::new(url, ResourceFetchTiming::new(request.timing_type()));

                    let range_header = request.headers.typed_get::<Range>();
                    let is_range_request = range_header.is_some();
                    let range = match get_range_request_bounds(range_header).get_final(file_size) {
                        Ok(range) => range,
                        Err(_) => {
                            range_not_satisfiable_error(&mut response);
                            return response;
                        },
                    };
                    let mut reader = BufReader::with_capacity(FILE_CHUNK_SIZE, file);
                    if reader.seek(SeekFrom::Start(range.start as u64)).is_err() {
                        return Response::network_error(NetworkError::Internal(
                            "Unexpected method for file".into(),
                        ));
                    }

                    // Set response status to 206 if Range header is present.
                    // At this point we should have already validated the header.
                    if is_range_request {
                        partial_content(&mut response);
                    }

                    // Set Content-Type header.
                    let mime = guess_mime_type(file_path);
                    response.headers.typed_insert(ContentType::from(mime));

                    // Setup channel to receive cross-thread messages about the file fetch
                    // operation.
                    let (done_sender, done_receiver) = unbounded();
                    *done_chan = Some((done_sender.clone(), done_receiver));
                    *response.body.lock().unwrap() = ResponseBody::Receiving(vec![]);

                    fetch_file_in_chunks(
                        done_sender,
                        reader,
                        response.body.clone(),
                        context.cancellation_listener.clone(),
                        range,
                    );

                    response
                } else {
                    Response::network_error(NetworkError::Internal("Opening file failed".into()))
                }
            } else {
                Response::network_error(NetworkError::Internal(
                    "Constructing file path failed".into(),
                ))
            }
        },

        "blob" => {
            debug!("Loading blob {}", url.as_str());
            // Step 2.
            if request.method != Method::GET {
                return Response::network_error(NetworkError::Internal(
                    "Unexpected method for blob".into(),
                ));
            }

            let range_header = request.headers.typed_get::<Range>();
            let is_range_request = range_header.is_some();
            // We will get a final version of this range once we have
            // the length of the data backing the blob.
            let range = get_range_request_bounds(range_header);

            let (id, origin) = match parse_blob_url(&url) {
                Ok((id, origin)) => (id, origin),
                Err(()) => {
                    return Response::network_error(NetworkError::Internal(
                        "Invalid blob url".into(),
                    ));
                },
            };

            let mut response = Response::new(url, ResourceFetchTiming::new(request.timing_type()));
            response.status = Some((StatusCode::OK, "OK".to_string()));
            response.raw_status = Some((StatusCode::OK.as_u16(), b"OK".to_vec()));

            if is_range_request {
                partial_content(&mut response);
            }

            let (done_sender, done_receiver) = unbounded();
            *done_chan = Some((done_sender.clone(), done_receiver));
            *response.body.lock().unwrap() = ResponseBody::Receiving(vec![]);
            let check_url_validity = true;
            if let Err(err) = context.filemanager.fetch_file(
                &done_sender,
                context.cancellation_listener.clone(),
                id,
                check_url_validity,
                origin,
                &mut response,
                range,
            ) {
                let _ = done_sender.send(Data::Done);
                let err = match err {
                    BlobURLStoreError::InvalidRange => {
                        range_not_satisfiable_error(&mut response);
                        return response;
                    },
                    _ => format!("{:?}", err),
                };
                return Response::network_error(NetworkError::Internal(err));
            };

            response
        },

        "ftp" => {
            debug!("ftp is not implemented");
            Response::network_error(NetworkError::Internal("Unexpected scheme".into()))
        },

        _ => Response::network_error(NetworkError::Internal("Unexpected scheme".into())),
    }
}

/// <https://fetch.spec.whatwg.org/#cors-safelisted-request-header>
pub fn is_cors_safelisted_request_header(name: &HeaderName, value: &HeaderValue) -> bool {
    if name == header::CONTENT_TYPE {
        if let Some(m) = value.to_str().ok().and_then(|s| s.parse::<Mime>().ok()) {
            m.type_() == mime::TEXT && m.subtype() == mime::PLAIN ||
                m.type_() == mime::APPLICATION && m.subtype() == mime::WWW_FORM_URLENCODED ||
                m.type_() == mime::MULTIPART && m.subtype() == mime::FORM_DATA
        } else {
            false
        }
    } else {
        name == header::ACCEPT ||
            name == header::ACCEPT_LANGUAGE ||
            name == header::CONTENT_LANGUAGE
    }
}

/// <https://fetch.spec.whatwg.org/#cors-safelisted-method>
pub fn is_cors_safelisted_method(m: &Method) -> bool {
    match *m {
        Method::GET | Method::HEAD | Method::POST => true,
        _ => false,
    }
}

fn is_null_body_status(status: &Option<(StatusCode, String)>) -> bool {
    match *status {
        Some((status, _)) => match status {
            StatusCode::SWITCHING_PROTOCOLS |
            StatusCode::NO_CONTENT |
            StatusCode::RESET_CONTENT |
            StatusCode::NOT_MODIFIED => true,
            _ => false,
        },
        _ => false,
    }
}

/// <https://fetch.spec.whatwg.org/#should-response-to-request-be-blocked-due-to-nosniff?>
pub fn should_be_blocked_due_to_nosniff(
    destination: Destination,
    response_headers: &HeaderMap,
) -> bool {
    // Steps 1-3.
    // TODO(eijebong): Replace this once typed headers allow custom ones...
    if response_headers
        .get("x-content-type-options")
        .map_or(true, |val| {
            val.to_str().unwrap_or("").to_lowercase() != "nosniff"
        }) {
        return false;
    }

    // Step 4
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
        // Step 6
        Some(ref ct) if destination.is_script_like() => {
            !is_javascript_mime_type(&ct.clone().into())
        },

        // Step 7
        Some(ref ct) if destination == Destination::Style => {
            let m: mime::Mime = ct.clone().into();
            m.type_() != mime::TEXT && m.subtype() != mime::CSS
        },

        None if destination == Destination::Style || destination.is_script_like() => true,
        // Step 8
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
    destination.is_script_like() && match mime_type.type_() {
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
    static BAD_PORTS: [u16; 64] = [
        1, 7, 9, 11, 13, 15, 17, 19, 20, 21, 22, 23, 25, 37, 42, 43, 53, 77, 79, 87, 95, 101, 102,
        103, 104, 109, 110, 111, 113, 115, 117, 119, 123, 135, 139, 143, 179, 389, 465, 512, 513,
        514, 515, 526, 530, 531, 532, 540, 556, 563, 587, 601, 636, 993, 995, 2049, 3659, 4045,
        6000, 6665, 6666, 6667, 6668, 6669,
    ];

    BAD_PORTS.binary_search(&port).is_ok()
}
