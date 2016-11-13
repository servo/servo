/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use blob_loader::load_blob_sync;
use connector::create_http_connector;
use data_loader::decode;
use devtools_traits::DevtoolsControlMsg;
use fetch::cors_cache::CorsCache;
use filemanager_thread::{FileManager, UIProvider};
use http_loader::{HttpState, set_default_accept_encoding, set_default_accept_language, set_request_cookies};
use http_loader::{NetworkHttpRequestFactory, ReadResult, StreamedResponse, obtain_response, read_block};
use http_loader::{auth_from_cache, determine_request_referrer, set_cookies_from_headers};
use http_loader::{send_response_to_devtools, send_request_to_devtools, LoadErrorType};
use hyper::header::{Accept, AcceptLanguage, Authorization, AccessControlAllowCredentials};
use hyper::header::{AccessControlAllowOrigin, AccessControlAllowHeaders, AccessControlAllowMethods};
use hyper::header::{AccessControlRequestHeaders, AccessControlMaxAge, AccessControlRequestMethod, Basic};
use hyper::header::{CacheControl, CacheDirective, ContentEncoding, ContentLength, ContentLanguage, ContentType};
use hyper::header::{Encoding, HeaderView, Headers, Host, IfMatch, IfRange, IfUnmodifiedSince, IfModifiedSince};
use hyper::header::{IfNoneMatch, Pragma, Location, QualityItem, Referer as RefererHeader, UserAgent, q, qitem};
use hyper::method::Method;
use hyper::mime::{Mime, SubLevel, TopLevel};
use hyper::status::StatusCode;
use hyper_serde::Serde;
use mime_guess::guess_mime_type;
use net_traits::{FetchTaskTarget, FetchMetadata, NetworkError, ReferrerPolicy};
use net_traits::request::{CacheMode, CredentialsMode, Destination};
use net_traits::request::{RedirectMode, Referrer, Request, RequestMode, ResponseTainting};
use net_traits::request::{Type, Origin, Window};
use net_traits::response::{HttpsState, Response, ResponseBody, ResponseType};
use resource_thread::CancellationListener;
use std::borrow::Cow;
use std::collections::HashSet;
use std::error::Error;
use std::fs::File;
use std::io::Read;
use std::iter::FromIterator;
use std::mem::swap;
use std::ops::Deref;
use std::rc::Rc;
use std::sync::mpsc::{channel, Sender, Receiver};
use unicase::UniCase;
use url::{Origin as UrlOrigin, Url};
use util::thread::spawn_named;
use uuid;

pub type Target = Option<Box<FetchTaskTarget + Send>>;

enum Data {
    Payload(Vec<u8>),
    Done,
}

pub struct FetchContext<UI: 'static + UIProvider> {
    pub state: HttpState,
    pub user_agent: Cow<'static, str>,
    pub devtools_chan: Option<Sender<DevtoolsControlMsg>>,
    pub filemanager: FileManager<UI>,
}

type DoneChannel = Option<(Sender<Data>, Receiver<Data>)>;

/// [Fetch](https://fetch.spec.whatwg.org#concept-fetch)
pub fn fetch<UI: 'static + UIProvider>(request: Rc<Request>,
                                       target: &mut Target,
                                       context: &FetchContext<UI>)
                                       -> Response {
    fetch_with_cors_cache(request, &mut CorsCache::new(), target, context)
}

pub fn fetch_with_cors_cache<UI: 'static + UIProvider>(request: Rc<Request>,
                                                       cache: &mut CorsCache,
                                                       target: &mut Target,
                                                       context: &FetchContext<UI>)
                                                       -> Response {
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
    main_fetch(request, cache, false, false, target, &mut None, &context)
}

/// [Main fetch](https://fetch.spec.whatwg.org/#concept-main-fetch)
fn main_fetch<UI: 'static + UIProvider>(request: Rc<Request>,
                                        cache: &mut CorsCache,
                                        cors_flag: bool,
                                        recursive_flag: bool,
                                        target: &mut Target,
                                        done_chan: &mut DoneChannel,
                                        context: &FetchContext<UI>)
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
    if request.referrer_policy.get().is_none() {
        request.referrer_policy.set(Some(ReferrerPolicy::NoReferrerWhenDowngrade));
    }

    // Step 8
    if *request.referrer.borrow() != Referrer::NoReferrer {
        // remove Referrer headers set in past redirects/preflights
        // this stops the assertion in determine_request_referrer from failing
        request.headers.borrow_mut().remove::<RefererHeader>();
        let referrer_url = determine_request_referrer(&mut *request.headers.borrow_mut(),
                                                      request.referrer_policy.get(),
                                                      request.referrer.borrow_mut().take(),
                                                      request.current_url().clone());
        *request.referrer.borrow_mut() = Referrer::from_url(referrer_url);
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

    {
        // Step 14
        let network_error_res;
        let internal_response = if let Some(error) = response.get_network_error() {
            network_error_res = Response::network_error(error.clone());
            &network_error_res
        } else {
            response.actual_response()
        };

        // Step 15
        if internal_response.url_list.borrow().is_empty() {
            *internal_response.url_list.borrow_mut() = request.url_list.borrow().clone();
        }

        // Step 16
        // TODO this step (CSP/blocking)

        // Step 17
        if !response.is_network_error() && (is_null_body_status(&internal_response.status) ||
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
    }

    // Step 19
    if request.synchronous {
        if let Some(ref mut target) = *target {
            // process_response is not supposed to be used
            // by sync fetch, but we overload it here for simplicity
            target.process_response(&response);
        }

        if let Some(ref ch) = *done_chan {
            loop {
                match ch.1.recv()
                        .expect("fetch worker should always send Done before terminating") {
                    Data::Payload(vec) => {
                        if let Some(ref mut target) = *target {
                            target.process_response_chunk(vec);
                        }
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
                if let Some(ref mut target) = *target {
                    target.process_response_chunk(vec.clone());
                }
            } else {
                assert!(*body == ResponseBody::Empty)
            }
        }

        // overloaded similarly to process_response
        if let Some(ref mut target) = *target {
            target.process_response_eof(&response);
        }
        return response;
    }

    // Step 20
    if request.body.borrow().is_some() && matches!(request.current_url().scheme(), "http" | "https") {
        if let Some(ref mut target) = *target {
            // XXXManishearth: We actually should be calling process_request
            // in http_network_fetch. However, we can't yet follow the request
            // upload progress, so I'm keeping it here for now and pretending
            // the body got sent in one chunk
            target.process_request_body(&request);
            target.process_request_eof(&request);
        }
    }

    // Step 21
    if let Some(ref mut target) = *target {
        target.process_response(&response);
    }

    // Step 22
    if let Some(ref ch) = *done_chan {
        loop {
            match ch.1.recv()
                    .expect("fetch worker should always send Done before terminating") {
                Data::Payload(vec) => {
                    if let Some(ref mut target) = *target {
                        target.process_response_chunk(vec);
                    }
                }
                Data::Done => break,
            }
        }
    } else if let Some(ref mut target) = *target {
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

    // Step 23
    request.done.set(true);

    // Step 24
    if let Some(ref mut target) = *target {
        target.process_response_eof(&response);
    }

    // TODO remove this line when only asynchronous fetches are used
    return response;
}

/// [Basic fetch](https://fetch.spec.whatwg.org#basic-fetch)
fn basic_fetch<UI: 'static + UIProvider>(request: Rc<Request>,
                                         cache: &mut CorsCache,
                                         target: &mut Target,
                                         done_chan: &mut DoneChannel,
                                         context: &FetchContext<UI>)
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

/// [HTTP fetch](https://fetch.spec.whatwg.org#http-fetch)
fn http_fetch<UI: 'static + UIProvider>(request: Rc<Request>,
                                        cache: &mut CorsCache,
                                        cors_flag: bool,
                                        cors_preflight_flag: bool,
                                        authentication_fetch_flag: bool,
                                        target: &mut Target,
                                        done_chan: &mut DoneChannel,
                                        context: &FetchContext<UI>)
                                        -> Response {
    // This is a new async fetch, reset the channel we are waiting on
    *done_chan = None;
    // Step 1
    let mut response: Option<Response> = None;

    // Step 2
    // nothing to do, since actual_response is a function on response

    // Step 3
    if !request.skip_service_worker.get() && !request.is_service_worker_global_scope {
        // Substep 1
        // TODO (handle fetch unimplemented)

        if let Some(ref res) = response {
            // Substep 2
            // nothing to do, since actual_response is a function on response

            // Substep 3
            if (res.response_type == ResponseType::Opaque &&
                request.mode != RequestMode::NoCors) ||
               (res.response_type == ResponseType::OpaqueRedirect &&
                request.redirect_mode.get() != RedirectMode::Manual) ||
               (res.url_list.borrow().len() > 1 &&
                request.redirect_mode.get() != RedirectMode::Follow) ||
               res.is_network_error() {
                return Response::network_error(NetworkError::Internal("Request failed".into()));
            }

            // Substep 4
            // TODO: set response's CSP list on actual_response
        }
    }

    // Step 4
    let credentials = match request.credentials_mode {
        CredentialsMode::Include => true,
        CredentialsMode::CredentialsSameOrigin if request.response_tainting.get() == ResponseTainting::Basic
            => true,
        _ => false
    };
    // Step 5
    if response.is_none() {
        // Substep 1
        if cors_preflight_flag {
            let method_cache_match = cache.match_method(&*request,
                                                        request.method.borrow().clone());

            let method_mismatch = !method_cache_match && (!is_simple_method(&request.method.borrow()) ||
                                                          request.use_cors_preflight);
            let header_mismatch = request.headers.borrow().iter().any(|view|
                !cache.match_header(&*request, view.name()) && !is_simple_header(&view)
            );

            // Sub-substep 1
            if method_mismatch || header_mismatch {
                let preflight_result = cors_preflight_fetch(request.clone(), cache, context);
                // Sub-substep 2
                if let Some(e) = preflight_result.get_network_error() {
                    return Response::network_error(e.clone());
                }
            }
        }

        // Substep 2
        request.skip_service_worker.set(true);

        // Substep 3
        let fetch_result = http_network_or_cache_fetch(request.clone(), credentials, authentication_fetch_flag,
                                                       done_chan, context);

        // Substep 4
        if cors_flag && cors_check(request.clone(), &fetch_result).is_err() {
            return Response::network_error(NetworkError::Internal("CORS check failed".into()));
        }

        fetch_result.return_internal.set(false);
        response = Some(fetch_result);
    }

    // response is guaranteed to be something by now
    let mut response = response.unwrap();

    // Step 5
    match response.actual_response().status {
        // Code 301, 302, 303, 307, 308
        Some(StatusCode::MovedPermanently) |
        Some(StatusCode::Found) |
        Some(StatusCode::SeeOther) |
        Some(StatusCode::TemporaryRedirect) |
        Some(StatusCode::PermanentRedirect) => {
            response = match request.redirect_mode.get() {
                RedirectMode::Error => Response::network_error(NetworkError::Internal("Redirect mode error".into())),
                RedirectMode::Manual => {
                    response.to_filtered(ResponseType::OpaqueRedirect)
                },
                RedirectMode::Follow => {
                    // set back to default
                    response.return_internal.set(true);
                    http_redirect_fetch(request, cache, response,
                                        cors_flag, target, done_chan, context)
                }
            }
        },

        // Code 401
        Some(StatusCode::Unauthorized) => {
            // Step 1
            // FIXME: Figure out what to do with request window objects
            if cors_flag || !credentials {
                return response;
            }

            // Step 2
            // TODO: Spec says requires testing on multiple WWW-Authenticate headers

            // Step 3
            if !request.use_url_credentials || authentication_fetch_flag {
                // TODO: Prompt the user for username and password from the window
                // Wrong, but will have to do until we are able to prompt the user
                // otherwise this creates an infinite loop
                // We basically pretend that the user declined to enter credentials
                return response;
            }

            // Step 4
            return http_fetch(request, cache, cors_flag, cors_preflight_flag,
                              true, target, done_chan, context);
        }

        // Code 407
        Some(StatusCode::ProxyAuthenticationRequired) => {
            // Step 1
            // TODO: Figure out what to do with request window objects

            // Step 2
            // TODO: Spec says requires testing on Proxy-Authenticate headers

            // Step 3
            // TODO: Prompt the user for proxy authentication credentials
            // Wrong, but will have to do until we are able to prompt the user
            // otherwise this creates an infinite loop
            // We basically pretend that the user declined to enter credentials
            return response;

            // Step 4
            // return http_fetch(request, cache,
            //                   cors_flag, cors_preflight_flag,
            //                   authentication_fetch_flag, target,
            //                   done_chan, context);
        }

        _ => { }
    }

    // Step 6
    if authentication_fetch_flag {
        // TODO: Create authentication entry for this request
    }

    // set back to default
    response.return_internal.set(true);
    // Step 7
    response
}

/// [HTTP redirect fetch](https://fetch.spec.whatwg.org#http-redirect-fetch)
fn http_redirect_fetch<UI: 'static + UIProvider>(request: Rc<Request>,
                                                 cache: &mut CorsCache,
                                                 response: Response,
                                                 cors_flag: bool,
                                                 target: &mut Target,
                                                 done_chan: &mut DoneChannel,
                                                 context: &FetchContext<UI>)
                                                 -> Response {
    // Step 1
    assert_eq!(response.return_internal.get(), true);

    // Step 2
    if !response.actual_response().headers.has::<Location>() {
        return response;
    }

    // Step 3
    let location = match response.actual_response().headers.get::<Location>() {
        Some(&Location(ref location)) => location.clone(),
        _ => return Response::network_error(NetworkError::Internal("Location header parsing failure".into()))
    };
    let response_url = response.actual_response().url().unwrap();
    let location_url = response_url.join(&*location);
    let location_url = match location_url {
        Ok(url) => url,
        _ => return Response::network_error(NetworkError::Internal("Location URL parsing failure".into()))
    };

    // Step 4
    // TODO implement return network_error if not HTTP(S)

    // Step 5
    if request.redirect_count.get() >= 20 {
        return Response::network_error(NetworkError::Internal("Too many redirects".into()));
    }

    // Step 6
    request.redirect_count.set(request.redirect_count.get() + 1);

    // Step 7
    let same_origin = if let Origin::Origin(ref origin) = *request.origin.borrow() {
        *origin == request.current_url().origin()
    } else {
        false
    };
    let has_credentials = has_credentials(&location_url);

    if request.mode == RequestMode::CorsMode && !same_origin && has_credentials {
        return Response::network_error(NetworkError::Internal("Cross-origin credentials check failed".into()));
    }

    // Step 8
    if cors_flag && has_credentials {
        return Response::network_error(NetworkError::Internal("Credentials check failed".into()));
    }

    // Step 9
    if cors_flag && !same_origin {
        *request.origin.borrow_mut() = Origin::Origin(UrlOrigin::new_opaque());
    }

    // Step 10
    let status_code = response.actual_response().status.unwrap();
    if ((status_code == StatusCode::MovedPermanently || status_code == StatusCode::Found) &&
        *request.method.borrow() == Method::Post) ||
        status_code == StatusCode::SeeOther {
        *request.method.borrow_mut() = Method::Get;
        *request.body.borrow_mut() = None;
    }

    // Step 11
    request.url_list.borrow_mut().push(location_url);

    // Step 12
    // TODO implement referrer policy

    // Step 13
    main_fetch(request, cache, cors_flag, true, target, done_chan, context)
}

/// [HTTP network or cache fetch](https://fetch.spec.whatwg.org#http-network-or-cache-fetch)
fn http_network_or_cache_fetch<UI: 'static + UIProvider>(request: Rc<Request>,
                                                         credentials_flag: bool,
                                                         authentication_fetch_flag: bool,
                                                         done_chan: &mut DoneChannel,
                                                         context: &FetchContext<UI>)
                                                         -> Response {
    // TODO: Implement Window enum for Request
    let request_has_no_window = true;

    // Step 1
    let http_request = if request_has_no_window &&
        request.redirect_mode.get() == RedirectMode::Error {
        request
    } else {
        Rc::new((*request).clone())
    };

    let content_length_value = match *http_request.body.borrow() {
        None =>
            match *http_request.method.borrow() {
                // Step 3
                Method::Head | Method::Post | Method::Put =>
                    Some(0),
                // Step 2
                _ => None
            },
        // Step 4
        Some(ref http_request_body) => Some(http_request_body.len() as u64)
    };

    // Step 5
    if let Some(content_length_value) = content_length_value {
        http_request.headers.borrow_mut().set(ContentLength(content_length_value));
    }

    // Step 6
    match *http_request.referrer.borrow() {
        Referrer::NoReferrer => (),
        Referrer::ReferrerUrl(ref http_request_referrer) =>
            http_request.headers.borrow_mut().set(RefererHeader(http_request_referrer.to_string())),
        Referrer::Client =>
            // it should be impossible for referrer to be anything else during fetching
            // https://fetch.spec.whatwg.org/#concept-request-referrer
            unreachable!()
    };

    // Step 7
    if http_request.omit_origin_header.get() == false {
        // TODO update this when https://github.com/hyperium/hyper/pull/691 is finished
        // http_request.headers.borrow_mut().set_raw("origin", origin);
    }

    // Step 8
    if !http_request.headers.borrow().has::<UserAgent>() {
        let user_agent = context.user_agent.clone().into_owned();
        http_request.headers.borrow_mut().set(UserAgent(user_agent));
    }

    match http_request.cache_mode.get() {
        // Step 9
        CacheMode::Default if is_no_store_cache(&http_request.headers.borrow()) => {
            http_request.cache_mode.set(CacheMode::NoStore);
        },

        // Step 10
        CacheMode::NoCache if !http_request.headers.borrow().has::<CacheControl>() => {
            http_request.headers.borrow_mut().set(CacheControl(vec![CacheDirective::MaxAge(0)]));
        },

        // Step 11
        CacheMode::Reload => {
            // Substep 1
            if !http_request.headers.borrow().has::<Pragma>() {
                http_request.headers.borrow_mut().set(Pragma::NoCache);
            }

            // Substep 2
            if !http_request.headers.borrow().has::<CacheControl>() {
                http_request.headers.borrow_mut().set(CacheControl(vec![CacheDirective::NoCache]));
            }
        },

        _ => {}
    }

    let current_url = http_request.current_url();
    // Step 12
    // todo: pass referrer url and policy
    // this can only be uncommented when the referrer header is set, else it crashes
    // in the meantime, we manually set the headers in the block below
    // modify_request_headers(&mut http_request.headers.borrow_mut(), &current_url,
    //                        None, None, None);
    {
        let headers = &mut *http_request.headers.borrow_mut();
        let host = Host {
            hostname: current_url.host_str().unwrap().to_owned(),
            port: current_url.port_or_known_default()
        };
        headers.set(host);
        // unlike http_loader, we should not set the accept header
        // here, according to the fetch spec
        set_default_accept_encoding(headers);
    }

    // Step 13
    // TODO some of this step can't be implemented yet
    if credentials_flag {
        // Substep 1
        // TODO http://mxr.mozilla.org/servo/source/components/net/http_loader.rs#504
        // XXXManishearth http_loader has block_cookies: support content blocking here too
        set_request_cookies(&current_url,
                            &mut *http_request.headers.borrow_mut(),
                            &context.state.cookie_jar);
        // Substep 2
        if !http_request.headers.borrow().has::<Authorization<String>>() {
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
                    authorization_value = Some(Basic {
                        username: current_url.username().to_owned(),
                        password: current_url.password().map(str::to_owned)
                    })
                }
            }

            // Substep 6
            if let Some(basic) = authorization_value {
                http_request.headers.borrow_mut().set(Authorization(basic));
            }
        }
    }

    // Step 14
    // TODO this step can't be implemented yet

    // Step 15
    let mut response: Option<Response> = None;

    // Step 16
    // TODO have a HTTP cache to check for a completed response
    let complete_http_response_from_cache: Option<Response> = None;
    if http_request.cache_mode.get() != CacheMode::NoStore &&
        http_request.cache_mode.get() != CacheMode::Reload &&
        complete_http_response_from_cache.is_some() {
        // Substep 1
        if http_request.cache_mode.get() == CacheMode::ForceCache {
            // TODO pull response from HTTP cache
            // response = http_request
        }

        let revalidation_needed = match response {
            Some(ref response) => response_needs_revalidation(&response),
            _ => false
        };

        // Substep 2
        if !revalidation_needed && http_request.cache_mode.get() == CacheMode::Default {
            // TODO pull response from HTTP cache
            // response = http_request
            // response.cache_state = CacheState::Local;
        }

        // Substep 3
        if revalidation_needed && http_request.cache_mode.get() == CacheMode::Default ||
            http_request.cache_mode.get() == CacheMode::NoCache {
            // TODO this substep
        }

    // Step 17
    // TODO have a HTTP cache to check for a partial response
    } else if http_request.cache_mode.get() == CacheMode::Default ||
        http_request.cache_mode.get() == CacheMode::ForceCache {
        // TODO this substep
    }

    // Step 18
    if response.is_none() {
        response = Some(http_network_fetch(http_request.clone(), credentials_flag,
                                           done_chan, context));
    }
    let response = response.unwrap();

    // Step 19
    if let Some(status) = response.status {
        if status == StatusCode::NotModified &&
            (http_request.cache_mode.get() == CacheMode::Default ||
            http_request.cache_mode.get() == CacheMode::NoCache) {
            // Substep 1
            // TODO this substep
            // let cached_response: Option<Response> = None;

            // Substep 2
            // if cached_response.is_none() {
            //     return Response::network_error();
            // }

            // Substep 3

            // Substep 4
            // response = cached_response;

            // Substep 5
            // TODO cache_state is immutable?
            // response.cache_state = CacheState::Validated;
        }
    }

    // Step 20
    response
}

/// [HTTP network fetch](https://fetch.spec.whatwg.org/#http-network-fetch)
fn http_network_fetch<UI: 'static + UIProvider>(request: Rc<Request>,
                                                credentials_flag: bool,
                                                done_chan: &mut DoneChannel,
                                                context: &FetchContext<UI>)
                                                -> Response {
    // TODO: Implement HTTP network fetch spec

    // Step 1
    // nothing to do here, since credentials_flag is already a boolean

    // Step 2
    // TODO be able to create connection using current url's origin and credentials
    let connection = create_http_connector();

    // Step 3
    // TODO be able to tell if the connection is a failure

    // Step 4
    let factory = NetworkHttpRequestFactory {
        connector: connection,
    };
    let url = request.current_url();
    let cancellation_listener = CancellationListener::new(None);

    let request_id = context.devtools_chan.as_ref().map(|_| {
        uuid::Uuid::new_v4().simple().to_string()
    });

    // XHR uses the default destination; other kinds of fetches (which haven't been implemented yet)
    // do not. Once we support other kinds of fetches we'll need to be more fine grained here
    // since things like image fetches are classified differently by devtools
    let is_xhr = request.destination == Destination::None;
    let wrapped_response = obtain_response(&factory, &url, &request.method.borrow(),
                                           &request.headers.borrow(),
                                           &cancellation_listener, &request.body.borrow(), &request.method.borrow(),
                                           &request.pipeline_id.get(), request.redirect_count.get() + 1,
                                           request_id.as_ref().map(Deref::deref), is_xhr);

    let pipeline_id = request.pipeline_id.get();
    let (res, msg) = match wrapped_response {
        Ok(wrapped_response) => wrapped_response,
        Err(error) => {
            let error = match error.error {
                LoadErrorType::ConnectionAborted { .. } => unreachable!(),
                LoadErrorType::Ssl { reason } => NetworkError::SslValidation(error.url, reason),
                LoadErrorType::Cancelled => NetworkError::LoadCancelled,
                e => NetworkError::Internal(e.description().to_owned())
            };
            return Response::network_error(error);
        }
    };

    let mut response = Response::new(url.clone());
    response.status = Some(res.response.status);
    response.raw_status = Some((res.response.status_raw().0,
                                res.response.status_raw().1.as_bytes().to_vec()));
    response.headers = res.response.headers.clone();
    response.referrer = request.referrer.borrow().to_url().cloned();

    let res_body = response.body.clone();

    // We're about to spawn a thread to be waited on here
    *done_chan = Some(channel());
    let meta = match response.metadata().expect("Response metadata should exist at this stage") {
        FetchMetadata::Unfiltered(m) => m,
        FetchMetadata::Filtered { unsafe_, .. } => unsafe_
    };
    let done_sender = done_chan.as_ref().map(|ch| ch.0.clone());
    let devtools_sender = context.devtools_chan.clone();
    let meta_status = meta.status.clone();
    let meta_headers = meta.headers.clone();
    spawn_named(format!("fetch worker thread"), move || {
        match StreamedResponse::from_http_response(box res, meta) {
            Ok(mut res) => {
                *res_body.lock().unwrap() = ResponseBody::Receiving(vec![]);

                if let Some(ref sender) = devtools_sender {
                    if let Some(m) = msg {
                        send_request_to_devtools(m, &sender);
                    }

                    // --- Tell devtools that we got a response
                    // Send an HttpResponse message to devtools with the corresponding request_id
                    if let Some(pipeline_id) = pipeline_id {
                        send_response_to_devtools(
                            &sender, request_id.unwrap(),
                            meta_headers.map(Serde::into_inner),
                            meta_status,
                            pipeline_id);
                    }
                }

                loop {
                    match read_block(&mut res) {
                        Ok(ReadResult::Payload(chunk)) => {
                            if let ResponseBody::Receiving(ref mut body) = *res_body.lock().unwrap() {
                                body.extend_from_slice(&chunk);
                                if let Some(ref sender) = done_sender {
                                    let _ = sender.send(Data::Payload(chunk));
                                }
                            }
                        },
                        Ok(ReadResult::EOF) | Err(_) => {
                            let mut empty_vec = Vec::new();
                            let completed_body = match *res_body.lock().unwrap() {
                                ResponseBody::Receiving(ref mut body) => {
                                    // avoid cloning the body
                                    swap(body, &mut empty_vec);
                                    empty_vec
                                },
                                _ => empty_vec,
                            };
                            *res_body.lock().unwrap() = ResponseBody::Done(completed_body);
                            if let Some(ref sender) = done_sender {
                                let _ = sender.send(Data::Done);
                            }
                            break;
                        }
                    }
                }
            }
            Err(_) => {
                // XXXManishearth we should propagate this error somehow
                *res_body.lock().unwrap() = ResponseBody::Done(vec![]);
                if let Some(ref sender) = done_sender {
                    let _ = sender.send(Data::Done);
                }
            }
        }
    });

        // TODO these substeps aren't possible yet
        // Substep 1

        // Substep 2

    // TODO Determine if response was retrieved over HTTPS
    // TODO Servo needs to decide what ciphers are to be treated as "deprecated"
    response.https_state = HttpsState::None;

    // TODO Read request

    // Step 5-9
    // (needs stream bodies)

    // Step 10
    // TODO when https://bugzilla.mozilla.org/show_bug.cgi?id=1030660
    // is resolved, this step will become uneccesary
    // TODO this step
    if let Some(encoding) = response.headers.get::<ContentEncoding>() {
        if encoding.contains(&Encoding::Gzip) {
        }

        else if encoding.contains(&Encoding::Compress) {
        }
    };

    // Step 11
    // TODO this step isn't possible yet (CSP)

    // Step 12
    if response.is_network_error() && request.cache_mode.get() == CacheMode::NoStore {
        // TODO update response in the HTTP cache for request
    }

    // TODO this step isn't possible yet
    // Step 13

    // Step 14.
    if credentials_flag {
        set_cookies_from_headers(&url, &response.headers, &context.state.cookie_jar);
    }

    // TODO these steps
    // Step 15
        // Substep 1
        // Substep 2
            // Sub-substep 1
            // Sub-substep 2
            // Sub-substep 3
            // Sub-substep 4
        // Substep 3

    // Step 16
    response
}

/// [CORS preflight fetch](https://fetch.spec.whatwg.org#cors-preflight-fetch)
fn cors_preflight_fetch<UI: 'static + UIProvider>(request: Rc<Request>,
                                                  cache: &mut CorsCache,
                                                  context: &FetchContext<UI>)
                                                  -> Response {
    // Step 1
    let mut preflight = Request::new(request.current_url(), Some(request.origin.borrow().clone()),
                                     request.is_service_worker_global_scope, request.pipeline_id.get());
    *preflight.method.borrow_mut() = Method::Options;
    preflight.initiator = request.initiator.clone();
    preflight.type_ = request.type_.clone();
    preflight.destination = request.destination.clone();
    *preflight.referrer.borrow_mut() = request.referrer.borrow().clone();
    preflight.referrer_policy.set(request.referrer_policy.get());

    // Step 2
    preflight.headers.borrow_mut().set::<AccessControlRequestMethod>(
        AccessControlRequestMethod(request.method.borrow().clone()));

    // Step 3, 4
    let mut value = request.headers.borrow().iter()
                                            .filter_map(|ref view| if is_simple_header(view) {
                                                None
                                            } else {
                                                Some(UniCase(view.name().to_owned()))
                                            }).collect::<Vec<UniCase<String>>>();
    value.sort();

    // Step 5
    preflight.headers.borrow_mut().set::<AccessControlRequestHeaders>(
        AccessControlRequestHeaders(value));

    // Step 6
    let preflight = Rc::new(preflight);
    let response = http_network_or_cache_fetch(preflight.clone(), false, false, &mut None, context);

    // Step 7
    if cors_check(request.clone(), &response).is_ok() &&
       response.status.map_or(false, |status| status.is_success()) {
        // Substep 1
        let mut methods = if response.headers.has::<AccessControlAllowMethods>() {
            match response.headers.get::<AccessControlAllowMethods>() {
                Some(&AccessControlAllowMethods(ref m)) => m.clone(),
                // Substep 3
                None => return Response::network_error(NetworkError::Internal("CORS ACAM check failed".into()))
            }
        } else {
            vec![]
        };

        // Substep 2
        let header_names = if response.headers.has::<AccessControlAllowHeaders>() {
            match response.headers.get::<AccessControlAllowHeaders>() {
                Some(&AccessControlAllowHeaders(ref hn)) => hn.clone(),
                // Substep 3
                None => return Response::network_error(NetworkError::Internal("CORS ACAH check failed".into()))
            }
        } else {
            vec![]
        };

        // Substep 4
        if methods.is_empty() && request.use_cors_preflight {
            methods = vec![request.method.borrow().clone()];
        }

        // Substep 5
        debug!("CORS check: Allowed methods: {:?}, current method: {:?}",
                methods, request.method.borrow());
        if methods.iter().all(|method| *method != *request.method.borrow()) &&
            !is_simple_method(&*request.method.borrow()) {
            return Response::network_error(NetworkError::Internal("CORS method check failed".into()));
        }

        // Substep 6
        debug!("CORS check: Allowed headers: {:?}, current headers: {:?}",
                header_names, request.headers.borrow());
        let set: HashSet<&UniCase<String>> = HashSet::from_iter(header_names.iter());
        if request.headers.borrow().iter().any(|ref hv| !set.contains(&UniCase(hv.name().to_owned())) &&
                                                        !is_simple_header(hv)) {
            return Response::network_error(NetworkError::Internal("CORS headers check failed".into()));
        }

        // Substep 7, 8
        let max_age = response.headers.get::<AccessControlMaxAge>().map(|acma| acma.0).unwrap_or(0);

        // TODO: Substep 9 - Need to define what an imposed limit on max-age is

        // Substep 11, 12
        for method in &methods {
            cache.match_method_and_update(&*request, method.clone(), max_age);
        }

        // Substep 13, 14
        for header_name in &header_names {
            cache.match_header_and_update(&*request, &*header_name, max_age);
        }

        // Substep 15
        return response;
    }

    // Step 8
    Response::network_error(NetworkError::Internal("CORS check failed".into()))
}

/// [CORS check](https://fetch.spec.whatwg.org#concept-cors-check)
fn cors_check(request: Rc<Request>, response: &Response) -> Result<(), ()> {
    // Step 1
    let origin = response.headers.get::<AccessControlAllowOrigin>().cloned();

    // Step 2
    let origin = try!(origin.ok_or(()));

    // Step 3
    if request.credentials_mode != CredentialsMode::Include &&
       origin == AccessControlAllowOrigin::Any {
        return Ok(());
    }

    // Step 4
    let origin = match origin {
        AccessControlAllowOrigin::Value(origin) => origin,
        // if it's Any or Null at this point, there's nothing to do but return Err(())
        _ => return Err(())
    };

    match *request.origin.borrow() {
        Origin::Origin(ref o) if o.ascii_serialization() == origin => {},
        _ => return Err(())
    }

    // Step 5
    if request.credentials_mode != CredentialsMode::Include {
        return Ok(());
    }

    // Step 6
    let credentials = request.headers.borrow().get::<AccessControlAllowCredentials>().cloned();

    // Step 7
    if credentials.is_some() {
        return Ok(());
    }

    // Step 8
    Err(())
}

fn has_credentials(url: &Url) -> bool {
    !url.username().is_empty() || url.password().is_some()
}

fn is_no_store_cache(headers: &Headers) -> bool {
    headers.has::<IfModifiedSince>() | headers.has::<IfNoneMatch>() |
    headers.has::<IfUnmodifiedSince>() | headers.has::<IfMatch>() |
    headers.has::<IfRange>()
}

/// https://fetch.spec.whatwg.org/#cors-safelisted-request-header
fn is_simple_header(h: &HeaderView) -> bool {
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

fn is_simple_method(m: &Method) -> bool {
    match *m {
        Method::Get | Method::Head | Method::Post => true,
        _ => false
    }
}

fn response_needs_revalidation(_response: &Response) -> bool {
    // TODO this function
    false
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
