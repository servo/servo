/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use blob_loader::load_blob_sync;
use data_loader::decode;
use devtools_traits::DevtoolsControlMsg;
use fetch::cors_cache::CorsCache;
use filemanager_thread::FileManager;
use http_loader::{HttpState, determine_request_referrer, http_fetch};
use http_loader::{set_default_accept, set_default_accept_language};
use hyper::{Error, Result as HyperResult};
use hyper::header::{Accept, AcceptLanguage, AccessControlExposeHeaders, ContentLanguage, ContentType};
use hyper::header::{Header, HeaderFormat, HeaderView, Headers, Referer as RefererHeader};
use hyper::method::Method;
use hyper::mime::{Mime, SubLevel, TopLevel};
use hyper::status::StatusCode;
use ipc_channel::ipc::IpcReceiver;
use mime_guess::guess_mime_type;
use net_traits::{FetchTaskTarget, NetworkError, ReferrerPolicy};
use net_traits::request::{CredentialsMode, Destination, Referrer, Request, RequestMode};
use net_traits::request::{ResponseTainting, Origin, Window};
use net_traits::response::{Response, ResponseBody, ResponseType};
use servo_url::ServoUrl;
use std::borrow::Cow;
use std::fmt;
use std::fs::File;
use std::io::Read;
use std::mem;
use std::str;
use std::sync::{Arc, Mutex};
use std::sync::atomic::Ordering;
use std::sync::mpsc::{Sender, Receiver};
use subresource_integrity::is_response_integrity_valid;

pub type Target<'a> = &'a mut (FetchTaskTarget + Send);

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
pub fn fetch(request: &mut Request,
             target: Target,
             context: &FetchContext) {
    fetch_with_cors_cache(request, &mut CorsCache::new(), target, context);
}

pub fn fetch_with_cors_cache(request: &mut Request,
                             cache: &mut CorsCache,
                             target: Target,
                             context: &FetchContext) {
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
pub fn main_fetch(request: &mut Request,
                  cache: &mut CorsCache,
                  cors_flag: bool,
                  recursive_flag: bool,
                  target: Target,
                  done_chan: &mut DoneChannel,
                  context: &FetchContext)
                  -> Response {
    // Step 1.
    let mut response = None;

    // Step 2.
    if request.local_urls_only {
        if !matches!(request.current_url().scheme(), "about" | "blob" | "data" | "filesystem") {
            response = Some(Response::network_error(NetworkError::Internal("Non-local scheme".into())));
        }
    }

    // Step 3.
    // TODO: handle content security policy violations.

    // Step 4.
    // TODO: handle upgrade to a potentially secure URL.

    // Step 5.
    if should_be_blocked_due_to_bad_port(&request.current_url()) {
        response = Some(Response::network_error(NetworkError::Internal("Request attempted on bad port".into())));
    }
    // TODO: handle blocking as mixed content.
    // TODO: handle blocking by content security policy.

    // Step 6
    // TODO: handle request's client's referrer policy.

    // Step 7.
    request.referrer_policy = request.referrer_policy.or(Some(ReferrerPolicy::NoReferrerWhenDowngrade));

    // Step 8.
    {
        let referrer_url = match mem::replace(&mut request.referrer, Referrer::NoReferrer) {
            Referrer::NoReferrer => None,
            Referrer::Client => {
                // FIXME(#14507): We should never get this value here; it should
                //                already have been handled in the script thread.
                request.headers.remove::<RefererHeader>();
                None
            },
            Referrer::ReferrerUrl(url) => {
                request.headers.remove::<RefererHeader>();
                let current_url = request.current_url().clone();
                determine_request_referrer(&mut request.headers,
                                           request.referrer_policy.unwrap(),
                                           url,
                                           current_url)
            }
        };
        if let Some(referrer_url) = referrer_url {
            request.referrer = Referrer::ReferrerUrl(referrer_url);
        }
    }

    // Step 9.
    // TODO: handle FTP URLs.

    // Step 10.
    context.state.hsts_list.read().unwrap().switch_known_hsts_host_domain_url_to_https(
        request.current_url_mut());

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
            request.mode == RequestMode::Navigate {
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
                (!is_cors_safelisted_method(&request.method) ||
                request.headers.iter().any(|h| !is_cors_safelisted_request_header(&h)))) {
            // Substep 1.
            request.response_tainting = ResponseTainting::CorsTainting;
            // Substep 2.
            let response = http_fetch(request, cache, true, true, false,
                                        target, done_chan, context);
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
            http_fetch(request, cache, true, false, false, target, done_chan, context)
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
            let header_names = response.headers.get::<AccessControlExposeHeaders>();
            match header_names {
                // Subsubstep 2.
                Some(list) if request.credentials_mode != CredentialsMode::Include => {
                    if list.len() == 1 && list[0] == "*" {
                        response.cors_exposed_header_name_list =
                            response.headers.iter().map(|h| h.name().to_owned()).collect();
                    }
                },
                // Subsubstep 3.
                Some(list) => {
                    response.cors_exposed_header_name_list = list.iter().map(|h| (**h).clone()).collect();
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
        let should_replace_with_nosniff_error =
            !response_is_network_error && should_be_blocked_due_to_nosniff(request.destination, &response.headers);
        let should_replace_with_mime_type_error =
            !response_is_network_error && should_be_blocked_due_to_mime_type(request.destination, &response.headers);

        // Step 15.
        let mut network_error_response = response.get_network_error().cloned().map(Response::network_error);
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
        let internal_response =
            if should_replace_with_nosniff_error {
                // Defer rebinding result
                blocked_error_response = Response::network_error(NetworkError::Internal("Blocked by nosniff".into()));
                &blocked_error_response
            } else if should_replace_with_mime_type_error {
                // Defer rebinding result
                blocked_error_response = Response::network_error(NetworkError::Internal("Blocked by mime type".into()));
                &blocked_error_response
            } else {
                internal_response
            };

        // Step 18.
        // We check `internal_response` since we did not mutate `response`
        // in the previous step.
        let not_network_error = !response_is_network_error && !internal_response.is_network_error();
        if not_network_error && (is_null_body_status(&internal_response.status) ||
            match request.method {
                Method::Head | Method::Connect => true,
                _ => false }) {
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
           !is_response_integrity_valid(integrity_metadata, &response) {
            Response::network_error(NetworkError::Internal("Subresource integrity validation failed".into()))
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
            match ch.1.recv()
                    .expect("fetch worker should always send Done before terminating") {
                Data::Payload(vec) => {
                    target.process_response_chunk(vec);
                },
                Data::Done => break,
                Data::Cancelled => {
                    response.aborted.store(true, Ordering::Relaxed);
                    break;
                }
            }
        }
    } else {
        let body = response.body.lock().unwrap();
        if let ResponseBody::Done(ref vec) = *body {
            // in case there was no channel to wait for, the body was
            // obtained synchronously via scheme_fetch for data/file/about/etc
            // We should still send the body across as a chunk
            target.process_response_chunk(vec.clone());
        } else {
            assert!(*body == ResponseBody::Empty)
        }
    }
}

/// [Scheme fetch](https://fetch.spec.whatwg.org#scheme-fetch)
fn scheme_fetch(request: &mut Request,
               cache: &mut CorsCache,
               target: Target,
               done_chan: &mut DoneChannel,
               context: &FetchContext)
               -> Response {
    let url = request.current_url();

    match url.scheme() {
        "about" if url.path() == "blank" => {
            let mut response = Response::new(url);
            response.headers.set(ContentType(mime!(Text / Html; Charset = Utf8)));
            *response.body.lock().unwrap() = ResponseBody::Done(vec![]);
            response
        },

        "http" | "https" => {
            http_fetch(request, cache, false, false, false, target, done_chan, context)
        },

        "data" => {
            match decode(&url) {
                Ok((mime, bytes)) => {
                    let mut response = Response::new(url);
                    *response.body.lock().unwrap() = ResponseBody::Done(bytes);
                    response.headers.set(ContentType(mime));
                    response
                },
                Err(_) => Response::network_error(NetworkError::Internal("Decoding data URL failed".into()))
            }
        },

        "file" => {
            if request.method == Method::Get {
                match url.to_file_path() {
                    Ok(file_path) => {
                        match File::open(file_path.clone()) {
                            Ok(mut file) => {
                                let mut bytes = vec![];
                                let _ = file.read_to_end(&mut bytes);
                                let mime = guess_mime_type(file_path);

                                let mut response = Response::new(url);
                                *response.body.lock().unwrap() = ResponseBody::Done(bytes);
                                response.headers.set(ContentType(mime));
                                response
                            },
                            _ => Response::network_error(NetworkError::Internal("Opening file failed".into())),
                        }
                    },
                    _ => Response::network_error(NetworkError::Internal("Constructing file path failed".into()))
                }
            } else {
                Response::network_error(NetworkError::Internal("Unexpected method for file".into()))
            }
        },

        "blob" => {
            println!("Loading blob {}", url.as_str());
            // Step 2.
            if request.method != Method::Get {
                return Response::network_error(NetworkError::Internal("Unexpected method for blob".into()));
            }

            match load_blob_sync(url.clone(), context.filemanager.clone()) {
                Ok((headers, bytes)) => {
                    let mut response = Response::new(url);
                    response.headers = headers;
                    *response.body.lock().unwrap() = ResponseBody::Done(bytes);
                    response
                },
                Err(e) => {
                    debug!("Failed to load {}: {:?}", url, e);
                    Response::network_error(e)
                },
            }
        },

        "ftp" => {
            debug!("ftp is not implemented");
            Response::network_error(NetworkError::Internal("Unexpected scheme".into()))
        },

        _ => Response::network_error(NetworkError::Internal("Unexpected scheme".into()))
    }
}

/// <https://fetch.spec.whatwg.org/#cors-safelisted-request-header>
pub fn is_cors_safelisted_request_header(h: &HeaderView) -> bool {
    if h.is::<ContentType>() {
        match h.value() {
            Some(&ContentType(Mime(TopLevel::Text, SubLevel::Plain, _))) |
            Some(&ContentType(Mime(TopLevel::Application, SubLevel::WwwFormUrlEncoded, _))) |
            Some(&ContentType(Mime(TopLevel::Multipart, SubLevel::FormData, _))) => true,
            _ => false

        }
    } else {
        h.is::<Accept>() || h.is::<AcceptLanguage>() || h.is::<ContentLanguage>()
    }
}

/// <https://fetch.spec.whatwg.org/#cors-safelisted-method>
pub fn is_cors_safelisted_method(m: &Method) -> bool {
    match *m {
        Method::Get | Method::Head | Method::Post => true,
        _ => false
    }
}

fn is_null_body_status(status: &Option<StatusCode>) -> bool {
    match *status {
        Some(status) => match status {
            StatusCode::SwitchingProtocols | StatusCode::NoContent |
                StatusCode::ResetContent | StatusCode::NotModified => true,
            _ => false
        },
        _ => false
    }
}

/// <https://fetch.spec.whatwg.org/#should-response-to-request-be-blocked-due-to-nosniff?>
pub fn should_be_blocked_due_to_nosniff(destination: Destination, response_headers: &Headers) -> bool {
    /// <https://fetch.spec.whatwg.org/#x-content-type-options-header>
    /// This is needed to parse `X-Content-Type-Options` according to spec,
    /// which requires that we inspect only the first value.
    ///
    /// A [unit-like struct](https://doc.rust-lang.org/book/structs.html#unit-like-structs)
    /// is sufficient since a valid header implies that we use `nosniff`.
    #[derive(Clone, Copy, Debug)]
    struct XContentTypeOptions;

    impl Header for XContentTypeOptions {
        fn header_name() -> &'static str {
            "X-Content-Type-Options"
        }

        /// https://fetch.spec.whatwg.org/#should-response-to-request-be-blocked-due-to-nosniff%3F #2
        fn parse_header(raw: &[Vec<u8>]) -> HyperResult<Self> {
            raw.first()
                .and_then(|v| str::from_utf8(v).ok())
                .and_then(|s| if s.trim().eq_ignore_ascii_case("nosniff") {
                    Some(XContentTypeOptions)
                } else {
                    None
                })
                .ok_or(Error::Header)
        }
    }

    impl HeaderFormat for XContentTypeOptions {
        fn fmt_header(&self, f: &mut fmt::Formatter) -> fmt::Result {
            f.write_str("nosniff")
        }
    }

    // Steps 1-3.
    if response_headers.get::<XContentTypeOptions>().is_none() {
        return false;
    }

    // Step 4
    // Note: an invalid MIME type will produce a `None`.
    let content_type_header = response_headers.get::<ContentType>();

    /// <https://html.spec.whatwg.org/multipage/#scriptingLanguages>
    #[inline]
    fn is_javascript_mime_type(mime_type: &Mime) -> bool {
        let javascript_mime_types: [Mime; 16] = [
            mime!(Application / ("ecmascript")),
            mime!(Application / ("javascript")),
            mime!(Application / ("x-ecmascript")),
            mime!(Application / ("x-javascript")),
            mime!(Text / ("ecmascript")),
            mime!(Text / ("javascript")),
            mime!(Text / ("javascript1.0")),
            mime!(Text / ("javascript1.1")),
            mime!(Text / ("javascript1.2")),
            mime!(Text / ("javascript1.3")),
            mime!(Text / ("javascript1.4")),
            mime!(Text / ("javascript1.5")),
            mime!(Text / ("jscript")),
            mime!(Text / ("livescript")),
            mime!(Text / ("x-ecmascript")),
            mime!(Text / ("x-javascript")),
        ];

        javascript_mime_types.iter()
            .any(|mime| mime.0 == mime_type.0 && mime.1 == mime_type.1)
    }

    // Assumes str::starts_with is equivalent to mime::TopLevel
    match content_type_header {
        // Step 6
        Some(&ContentType(ref mime_type)) if destination.is_script_like()
            => !is_javascript_mime_type(mime_type),

        // Step 7
        Some(&ContentType(Mime(ref tl, ref sl, _))) if destination == Destination::Style
            => *tl != TopLevel::Text && *sl != SubLevel::Css,

        None if destination == Destination::Style || destination.is_script_like() => true,
        // Step 8
        _ => false
    }
}

/// <https://fetch.spec.whatwg.org/#should-response-to-request-be-blocked-due-to-mime-type?>
fn should_be_blocked_due_to_mime_type(destination: Destination, response_headers: &Headers) -> bool {
    // Step 1
    let mime_type = match response_headers.get::<ContentType>() {
        Some(header) => header,
        None => return false,
    };

    // Step 2-3
    destination.is_script_like() && match *mime_type {
        ContentType(Mime(TopLevel::Audio, _, _)) |
        ContentType(Mime(TopLevel::Video, _, _)) |
        ContentType(Mime(TopLevel::Image, _, _)) => true,
        ContentType(Mime(TopLevel::Text, SubLevel::Ext(ref ext), _)) => ext == "csv",

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
    let port = if let Some(port) = url.port() { port } else { return false };

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
        1, 7, 9, 11, 13, 15, 17, 19, 20, 21, 22, 23, 25, 37, 42,
        43, 53, 77, 79, 87, 95, 101, 102, 103, 104, 109, 110, 111,
        113, 115, 117, 119, 123, 135, 139, 143, 179, 389, 465, 512,
        513, 514, 515, 526, 530, 531, 532, 540, 556, 563, 587, 601,
        636, 993, 995, 2049, 3659, 4045, 6000, 6665, 6666, 6667,
        6668, 6669
    ];

    BAD_PORTS.binary_search(&port).is_ok()
}
