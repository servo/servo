/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use blob_loader::load_blob_sync;
use data_loader::decode;
use devtools_traits::DevtoolsControlMsg;
use fetch::cors_cache::CorsCache;
use filemanager_thread::FileManager;
use http_loader::{HttpState, determine_request_referrer, http_fetch, set_default_accept_language};
use hyper::header::{Accept, AcceptLanguage, ContentLanguage, ContentType};
use hyper::header::{HeaderView, QualityItem, Referer as RefererHeader, q, qitem};
use hyper::method::Method;
use hyper::mime::{Mime, SubLevel, TopLevel};
use hyper::status::StatusCode;
use mime_guess::guess_mime_type;
use net_traits::{FetchTaskTarget, NetworkError, ReferrerPolicy};
use net_traits::request::{RedirectMode, Referrer, Request, RequestMode, ResponseTainting};
use net_traits::request::{Type, Origin, Window};
use net_traits::response::{Response, ResponseBody, ResponseType};
use std::borrow::Cow;
use std::fs::File;
use std::io::Read;
use std::mem;
use std::rc::Rc;
use std::sync::mpsc::{Sender, Receiver};

pub type Target<'a> = &'a mut (FetchTaskTarget + Send);

pub enum Data {
    Payload(Vec<u8>),
    Done,
}

pub struct FetchContext {
    pub state: HttpState,
    pub user_agent: Cow<'static, str>,
    pub devtools_chan: Option<Sender<DevtoolsControlMsg>>,
    pub filemanager: FileManager,
}

pub type DoneChannel = Option<(Sender<Data>, Receiver<Data>)>;

/// [Fetch](https://fetch.spec.whatwg.org#concept-fetch)
pub fn fetch(request: Rc<Request>,
             target: Target,
             context: &FetchContext) {
    fetch_with_cors_cache(request, &mut CorsCache::new(), target, context);
}

pub fn fetch_with_cors_cache(request: Rc<Request>,
                             cache: &mut CorsCache,
                             target: Target,
                             context: &FetchContext) {
    // Step 1
    if request.window.get() == Window::Client {
        // TODO: Set window to request's client object if client is a Window object
    } else {
        request.window.set(Window::NoWindow);
    }

    // Step 2
    if *request.origin.borrow() == Origin::Client {
        // TODO: set request's origin to request's client's origin
        unimplemented!()
    }

    // Step 3
    if !request.headers.borrow().has::<Accept>() {
        let value = match request.type_ {
            // Substep 2
            _ if request.is_navigation_request() =>
                vec![qitem(mime!(Text / Html)),
                     // FIXME: This should properly generate a MimeType that has a
                     // SubLevel of xhtml+xml (https://github.com/hyperium/mime.rs/issues/22)
                     qitem(mime!(Application / ("xhtml+xml") )),
                     QualityItem::new(mime!(Application / Xml), q(0.9)),
                     QualityItem::new(mime!(_ / _), q(0.8))],

            // Substep 3
            Type::Image =>
                vec![qitem(mime!(Image / Png)),
                     // FIXME: This should properly generate a MimeType that has a
                     // SubLevel of svg+xml (https://github.com/hyperium/mime.rs/issues/22)
                     qitem(mime!(Image / ("svg+xml") )),
                     QualityItem::new(mime!(Image / _), q(0.8)),
                     QualityItem::new(mime!(_ / _), q(0.5))],

            // Substep 3
            Type::Style =>
                vec![qitem(mime!(Text / Css)),
                     QualityItem::new(mime!(_ / _), q(0.1))],
            // Substep 1
            _ => vec![qitem(mime!(_ / _))]
        };

        // Substep 4
        request.headers.borrow_mut().set(Accept(value));
    }

    // Step 4
    set_default_accept_language(&mut request.headers.borrow_mut());

    // Step 5
    // TODO: Figure out what a Priority object is

    // Step 6
    if request.is_subresource_request() {
        // TODO: create a fetch record and append it to request's client's fetch group list
    }

    // Step 7
    main_fetch(request, cache, false, false, target, &mut None, &context);
}

/// [Main fetch](https://fetch.spec.whatwg.org/#concept-main-fetch)
pub fn main_fetch(request: Rc<Request>,
                  cache: &mut CorsCache,
                  cors_flag: bool,
                  recursive_flag: bool,
                  target: Target,
                  done_chan: &mut DoneChannel,
                  context: &FetchContext)
                  -> Response {
    // TODO: Implement main fetch spec

    // Step 1
    let mut response = None;

    // Step 2
    if request.local_urls_only {
        match request.current_url().scheme() {
            "about" | "blob" | "data" | "filesystem" => (), // Ok, the URL is local.
            _ => response = Some(Response::network_error(NetworkError::Internal("Non-local scheme".into())))
        }
    }

    // Step 3
    // TODO be able to execute report CSP

    // Step 4
    // TODO this step, based off of http_loader.rs (upgrade)

    // Step 5
    // TODO this step (CSP port/content blocking)

    // Step 6
    // TODO this step (referrer policy)
    // currently the clients themselves set referrer policy in RequestInit

    // Step 7
    let referrer_policy = request.referrer_policy.get().unwrap_or(ReferrerPolicy::NoReferrerWhenDowngrade);
    request.referrer_policy.set(Some(referrer_policy));

    // Step 8
    {
        let mut referrer = request.referrer.borrow_mut();
        let referrer_url = match mem::replace(&mut *referrer, Referrer::NoReferrer) {
            Referrer::NoReferrer => None,
            Referrer::Client => {
                // FIXME(#14507): We should never get this value here; it should
                //                already have been handled in the script thread.
                request.headers.borrow_mut().remove::<RefererHeader>();
                None
            },
            Referrer::ReferrerUrl(url) => {
                request.headers.borrow_mut().remove::<RefererHeader>();
                determine_request_referrer(&mut *request.headers.borrow_mut(),
                                           referrer_policy,
                                           url,
                                           request.current_url().clone())
            }
        };
        if let Some(referrer_url) = referrer_url {
            *referrer = Referrer::ReferrerUrl(referrer_url);
        }
    }

    // Step 9
    // TODO this step (HSTS)

    // Step 10
    // this step is obsoleted by fetch_async

    // Step 11
    let response = match response {
        Some(response) => response,
        None => {
            let current_url = request.current_url();
            let same_origin = if let Origin::Origin(ref origin) = *request.origin.borrow() {
                *origin == current_url.origin()
            } else {
                false
            };

            if (same_origin && !cors_flag ) ||
                current_url.scheme() == "data" ||
                current_url.scheme() == "file" ||
                current_url.scheme() == "about" ||
                request.mode == RequestMode::Navigate {
                basic_fetch(request.clone(), cache, target, done_chan, context)

            } else if request.mode == RequestMode::SameOrigin {
                Response::network_error(NetworkError::Internal("Cross-origin response".into()))

            } else if request.mode == RequestMode::NoCors {
                request.response_tainting.set(ResponseTainting::Opaque);
                basic_fetch(request.clone(), cache, target, done_chan, context)

            } else if !matches!(current_url.scheme(), "http" | "https") {
                Response::network_error(NetworkError::Internal("Non-http scheme".into()))

            } else if request.use_cors_preflight ||
                (request.unsafe_request &&
                 (!is_simple_method(&request.method.borrow()) ||
                  request.headers.borrow().iter().any(|h| !is_simple_header(&h)))) {
                request.response_tainting.set(ResponseTainting::CorsTainting);
                request.redirect_mode.set(RedirectMode::Error);
                let response = http_fetch(request.clone(), cache, true, true, false,
                                          target, done_chan, context);
                if response.is_network_error() {
                    // TODO clear cache entries using request
                }
                response

            } else {
                request.response_tainting.set(ResponseTainting::CorsTainting);
                http_fetch(request.clone(), cache, true, false, false, target, done_chan, context)
            }
        }
    };

    // Step 12
    if recursive_flag {
        return response;
    }

    // Step 13
    // no need to check if response is a network error, since the type would not be `Default`
    let response = if response.response_type == ResponseType::Default {
        let response_type = match request.response_tainting.get() {
            ResponseTainting::Basic => ResponseType::Basic,
            ResponseTainting::CorsTainting => ResponseType::Cors,
            ResponseTainting::Opaque => ResponseType::Opaque,
        };
        response.to_filtered(response_type)
    } else {
        response
    };

    let internal_error = { // Inner scope to reborrow response
        // Step 14
        let network_error_response;
        let internal_response = if let Some(error) = response.get_network_error() {
            network_error_response = Response::network_error(error.clone());
            &network_error_response
        } else {
            response.actual_response()
        };

        // Step 15
        if internal_response.url_list.borrow().is_empty() {
            *internal_response.url_list.borrow_mut() = request.url_list.borrow().clone();
        }

        // Step 16
        // TODO Blocking for CSP, mixed content, MIME type
        let blocked_error_response;
        let internal_response = if !response.is_network_error() && should_block_nosniff(&request, &response) {
            // Defer rebinding result
            blocked_error_response = Response::network_error(NetworkError::Internal("Blocked by nosniff".into()));
            &blocked_error_response
        } else {
            internal_response
        };

        // Step 17
        // We use `internal_response` since we did not mutate `response` in the previous step.
        // Steps 14-18 may modify a response to become a network error, but not the reverse.
        // * Step 14: `internal_response.is_network_error()` <-> `response.is_network_error()`.
        //  `response.internal_response` may only be a non-network error type of the following:
        //  - basic filtered response
        //  - A CORS filtered response
        //  - An opaque filtered response
        //  - An opaque-redirect filtered response
        //  - None
        // * Step 15: No pertinent change.
        // * Step 16:
        //  - If *response* is already a network error, *response* and *internalResponse* are not modified.
        //  - If *response* becomes a network error, *internalResponse* also become a network error.
        //  - Otherwise  *response* and *internalResponse* are not modified.
        // Therefore: `internal_response.is_network_error() == response.is_network_error().`
        if !internal_response.is_network_error() && (is_null_body_status(&internal_response.status) ||
            match *request.method.borrow() {
                Method::Head | Method::Connect => true,
                _ => false })
            {
            // when Fetch is used only asynchronously, we will need to make sure
            // that nothing tries to write to the body at this point
            let mut body = internal_response.body.lock().unwrap();
            *body = ResponseBody::Empty;
        }

        // Step 18
        // TODO be able to compare response integrity against request integrity metadata
        // if !response.is_network_error() {

        //     // Substep 1
        //     response.wait_until_done();

        //     // Substep 2
        //     if response.termination_reason.is_none() {
        //         response = Response::network_error();
        //         internal_response = Response::network_error();
        //     }
        // }

        internal_response.get_network_error().map(|e| e.clone())
    };

    // Execute deferred rebinding of response
    let response = if let Some(error) = internal_error {
        Response::network_error(error)
    } else {
        response
    };

    // Step 19
    if request.synchronous {
        // process_response is not supposed to be used
        // by sync fetch, but we overload it here for simplicity
        target.process_response(&response);

        if let Some(ref ch) = *done_chan {
            loop {
                match ch.1.recv()
                        .expect("fetch worker should always send Done before terminating") {
                    Data::Payload(vec) => {
                        target.process_response_chunk(vec);
                    }
                    Data::Done => break,
                }
            }
        } else {
            let body = response.body.lock().unwrap();
            if let ResponseBody::Done(ref vec) = *body {
                // in case there was no channel to wait for, the body was
                // obtained synchronously via basic_fetch for data/file/about/etc
                // We should still send the body across as a chunk
                target.process_response_chunk(vec.clone());
            } else {
                assert!(*body == ResponseBody::Empty)
            }
        }

        // overloaded similarly to process_response
        target.process_response_eof(&response);
        return response;
    }

    // Step 20
    if request.body.borrow().is_some() && matches!(request.current_url().scheme(), "http" | "https") {
        // XXXManishearth: We actually should be calling process_request
        // in http_network_fetch. However, we can't yet follow the request
        // upload progress, so I'm keeping it here for now and pretending
        // the body got sent in one chunk
        target.process_request_body(&request);
        target.process_request_eof(&request);
    }

    // Step 21
    target.process_response(&response);

    // Step 22
    if let Some(ref ch) = *done_chan {
        loop {
            match ch.1.recv()
                    .expect("fetch worker should always send Done before terminating") {
                Data::Payload(vec) => {
                    target.process_response_chunk(vec);
                }
                Data::Done => break,
            }
        }
    } else {
        let body = response.body.lock().unwrap();
        if let ResponseBody::Done(ref vec) = *body {
            // in case there was no channel to wait for, the body was
            // obtained synchronously via basic_fetch for data/file/about/etc
            // We should still send the body across as a chunk
            target.process_response_chunk(vec.clone());
        } else {
            assert!(*body == ResponseBody::Empty)
        }
    }

    // Step 24
    target.process_response_eof(&response);

    // TODO remove this line when only asynchronous fetches are used
    return response;
}

/// [Basic fetch](https://fetch.spec.whatwg.org#basic-fetch)
fn basic_fetch(request: Rc<Request>,
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
            http_fetch(request.clone(), cache, false, false, false, target, done_chan, context)
        },

        "data" => {
            if *request.method.borrow() == Method::Get {
                match decode(&url) {
                    Ok((mime, bytes)) => {
                        let mut response = Response::new(url);
                        *response.body.lock().unwrap() = ResponseBody::Done(bytes);
                        response.headers.set(ContentType(mime));
                        response
                    },
                    Err(_) => Response::network_error(NetworkError::Internal("Decoding data URL failed".into()))
                }
            } else {
                Response::network_error(NetworkError::Internal("Unexpected method for data".into()))
            }
        },

        "file" => {
            if *request.method.borrow() == Method::Get {
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
            if *request.method.borrow() != Method::Get {
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

/// https://fetch.spec.whatwg.org/#cors-safelisted-request-header
pub fn is_simple_header(h: &HeaderView) -> bool {
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

pub fn is_simple_method(m: &Method) -> bool {
    match *m {
        Method::Get | Method::Head | Method::Post => true,
        _ => false
    }
}

// fn modify_request_headers(headers: &mut Headers) -> {
//     // TODO this function

// }

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

/// https://fetch.spec.whatwg.org/#should-response-to-request-be-blocked-due-to-nosniff?
#[inline]
fn should_block_nosniff(request: &Request, response: &Response) -> bool {
    header! { (XContentTypeOptions, "X-Content-Type-Options") => [String] };

    // Step 2
    if let Some(header) = response.headers.get::<XContentTypeOptions>() {
        let XContentTypeOptions(ref value) = *header;
        // Step 3
        if value != "nosniff" {
            return false;
        }

        // Step 4
        // TODO: What do we do if there is no MIME header?
        if let Some(content_type_header) = response.headers.get::<ContentType>() {
            let ContentType(ref mime_type) = *content_type_header;
            // Step 5
            let type_ = request.type_;

            /// This is a tricky one: it is not straightforward.
            /// The current practice is to use file sniffing
            /// to determine the actual font file type.
            ///
            /// There is a [proposal](https://www.w3.org/TR/WOFF2/#IMT)
            /// to add a top level "font/" MIME type.
            ///
            /// [These](https://docs.google.com/document/d/1Tsju6EOP4LqJ1RcFJRpqxryggbWZYbq5jvsIhVYNoyU/)
            /// are the most commonly used MIME types for fonts in practice.
            ///
            /// TODO determine how to handle font file acceptance with MIME types
            #[inline]
            fn is_font_mime_type(mime: &Mime) -> bool {
                // Declare that media MIME types are invalid for fonts
                match mime.0 {
                    TopLevel::Audio | TopLevel::Image | TopLevel::Video => false,
                    _ => true
                }
            }

            /// https://html.spec.whatwg.org/multipage/#scriptingLanguages
            #[inline]
            fn is_javascript_mime_type(mime_type: &Mime) -> bool {
                // TODO: try to replace with const
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

                javascript_mime_types.contains(mime_type)
            }

            // TODO: try to replace with const
            let text_css: Mime = mime!(Text / Css);
            let text_vtt: Mime = mime!(Text / ("vtt"));
            // Assumes str::starts_with is equivalent to mime::TopLevel
            return match type_ {
                // Step 7
                Type::Image => mime_type.0 != TopLevel::Image,
                // Step 8
                Type::Font => !is_font_mime_type(&mime_type),
                // Step 9
                Type::Script => !is_javascript_mime_type(&mime_type),
                // Step 10
                Type::Style => mime_type != &text_css,
                // Step 11
                Type::Track => mime_type != &text_vtt,
                // Step 12
                _ => false
            }
        }
    }

    // Step 1, 3
    false
}
