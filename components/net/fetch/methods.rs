/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use fetch::cors_cache::{BasicCORSCache, CORSCache, CacheRequestDetails};
use fetch::response::ResponseMethods;
use http_loader::{NetworkHttpRequestFactory, WrappedHttpResponse};
use http_loader::{create_http_connector, obtain_response};
use hyper::client::response::Response as HyperResponse;
use hyper::header::{Accept, IfMatch, IfRange, IfUnmodifiedSince, Location};
use hyper::header::{AcceptLanguage, ContentLength, ContentLanguage, HeaderView};
use hyper::header::{Authorization, Basic, ContentEncoding, Encoding};
use hyper::header::{ContentType, Header, Headers, IfModifiedSince, IfNoneMatch};
use hyper::header::{QualityItem, q, qitem, Referer as RefererHeader, UserAgent};
use hyper::method::Method;
use hyper::mime::{Attr, Mime, SubLevel, TopLevel, Value};
use hyper::status::StatusCode;
use net_traits::request::{CacheMode, Context, ContextFrameType, CredentialsMode};
use net_traits::request::{RedirectMode, Referer, Request, RequestMode, ResponseTainting};
use net_traits::response::{CacheState, HttpsState, TerminationReason};
use net_traits::response::{Response, ResponseBody, ResponseType};
use net_traits::{AsyncFetchListener, Metadata};
use resource_thread::CancellationListener;
use std::ascii::AsciiExt;
use std::cell::RefCell;
use std::io::Read;
use std::rc::Rc;
use std::str::FromStr;
use std::thread;
use url::{Origin, Url, UrlParser};
use util::thread::spawn_named;

pub fn fetch_async(request: Request, cors_flag: bool, listener: Box<AsyncFetchListener + Send>) {
    spawn_named(format!("fetch for {:?}", request.get_last_url_string()), move || {
        let request = Rc::new(request);
        let res = fetch(request, cors_flag);
        listener.response_available(res);
    })
}

/// [Fetch](https://fetch.spec.whatwg.org#concept-fetch)
pub fn fetch(request: Rc<Request>, cors_flag: bool) -> Response {

    // Step 1
    if request.context != Context::Fetch && !request.headers.borrow().has::<Accept>() {

        // Substep 1
        let value = match request.context {

            Context::Favicon | Context::Image | Context::ImageSet
                => vec![qitem(Mime(TopLevel::Image, SubLevel::Png, vec![])),
                    // FIXME: This should properly generate a MimeType that has a
                    // SubLevel of svg+xml (https://github.com/hyperium/mime.rs/issues/22)
                    qitem(Mime(TopLevel::Image, SubLevel::Ext("svg+xml".to_owned()), vec![])),
                    QualityItem::new(Mime(TopLevel::Image, SubLevel::Star, vec![]), q(0.8)),
                    QualityItem::new(Mime(TopLevel::Star, SubLevel::Star, vec![]), q(0.5))],

            Context::Form | Context::Frame | Context::Hyperlink |
            Context::IFrame | Context::Location | Context::MetaRefresh |
            Context::PreRender
                => vec![qitem(Mime(TopLevel::Text, SubLevel::Html, vec![])),
                    // FIXME: This should properly generate a MimeType that has a
                    // SubLevel of xhtml+xml (https://github.com/hyperium/mime.rs/issues/22)
                    qitem(Mime(TopLevel::Application, SubLevel::Ext("xhtml+xml".to_owned()), vec![])),
                    QualityItem::new(Mime(TopLevel::Application, SubLevel::Xml, vec![]), q(0.9)),
                    QualityItem::new(Mime(TopLevel::Star, SubLevel::Star, vec![]), q(0.8))],

            Context::Internal if request.context_frame_type != ContextFrameType::ContextNone
                => vec![qitem(Mime(TopLevel::Text, SubLevel::Html, vec![])),
                    // FIXME: This should properly generate a MimeType that has a
                    // SubLevel of xhtml+xml (https://github.com/hyperium/mime.rs/issues/22)
                    qitem(Mime(TopLevel::Application, SubLevel::Ext("xhtml+xml".to_owned()), vec![])),
                    QualityItem::new(Mime(TopLevel::Application, SubLevel::Xml, vec![]), q(0.9)),
                    QualityItem::new(Mime(TopLevel::Star, SubLevel::Star, vec![]), q(0.8))],

            Context::Style
                => vec![qitem(Mime(TopLevel::Text, SubLevel::Css, vec![])),
                    QualityItem::new(Mime(TopLevel::Star, SubLevel::Star, vec![]), q(0.1))],
            _ => vec![qitem(Mime(TopLevel::Star, SubLevel::Star, vec![]))]
        };

        // Substep 2
        request.headers.borrow_mut().set(Accept(value));
    }

    // Step 2
    if request.context != Context::Fetch && !request.headers.borrow().has::<AcceptLanguage>() {
        request.headers.borrow_mut().set(AcceptLanguage(vec![qitem("en-US".parse().unwrap())]));
    }

    // TODO: Figure out what a Priority object is
    // Step 3
    // Step 4
    main_fetch(request, cors_flag, false)
}

/// [Main fetch](https://fetch.spec.whatwg.org/#concept-main-fetch)
fn main_fetch(request: Rc<Request>, cors_flag: bool, recursive_flag: bool) -> Response {
    // TODO: Implement main fetch spec

    // Step 1
    let mut response = None;

    // Step 2
    if request.local_urls_only {
        match &*request.current_url().scheme {
            "about" | "blob" | "data" | "filesystem" => response = Some(Response::network_error()),
            _ => { }
        };
    }

    // Step 3
    // TODO be able to execute report CSP

    // Step 4
    // TODO this step, based off of http_loader.rs

    // Step 5
    // TODO this step

    // Step 6
    if request.referer != Referer::NoReferer {
        // TODO be able to invoke "determine request's referer"
    }

    // Step 7
    // TODO this step

    // Step 8
    if !request.synchronous && !recursive_flag {
        // TODO run the remaining steps in parallel
    }

    // Step 9
    let mut response = if response.is_none() {

        let current_url = request.current_url();
        let origin_match = request.origin == current_url.origin();

        if (!cors_flag && origin_match) ||
            (current_url.scheme == "data" && request.same_origin_data.get()) ||
            current_url.scheme == "about" ||
            request.mode == RequestMode::Navigate {

            basic_fetch(request.clone())

        } else if request.mode == RequestMode::SameOrigin {
            Response::network_error()

        } else if request.mode == RequestMode::NoCORS {
            request.response_tainting.set(ResponseTainting::Opaque);
            basic_fetch(request.clone())

        } else if current_url.scheme != "http" && current_url.scheme != "https" {
            Response::network_error()

        } else if request.use_cors_preflight ||
            (request.unsafe_request &&
            (!is_simple_method(&request.method.borrow()) ||
            request.headers.borrow().iter().any(|h| !is_simple_header(&h)))) {

            request.response_tainting.set(ResponseTainting::CORSTainting);
            request.redirect_mode.set(RedirectMode::Error);
            let response = http_fetch(request.clone(), BasicCORSCache::new(), true, true, false);
            if Response::is_network_error(&response) {
                // TODO clear cache entries using request
            }
            response

        } else {
            request.response_tainting.set(ResponseTainting::CORSTainting);
            http_fetch(request.clone(), BasicCORSCache::new(), true, false, false)
        }
    } else {
        response.unwrap()
    };

    // Step 10
    if recursive_flag {
        return response;
    }

    // Step 11
    // no need to check if response is a network error, since the type would not be `Default`
    let mut response = if response.response_type == ResponseType::Default {
        let old_response = Rc::new(response);
        let response_type = match request.response_tainting.get() {
            ResponseTainting::Basic => ResponseType::Basic,
            ResponseTainting::CORSTainting => ResponseType::CORS,
            ResponseTainting::Opaque => ResponseType::Opaque,
        };
        Response::to_filtered(old_response, response_type)
    } else {
        response
    };

    // Step 12
    let mut internal_response = if Response::is_network_error(&response) {
        Rc::new(Response::network_error())
    } else {
        response.internal_response.clone().unwrap()
    };

    // Step 13
    // TODO this step

    // Step 14
    if !Response::is_network_error(&response) && (is_null_body_status(&internal_response.status) ||
        match *request.method.borrow() {
            Method::Head | Method::Connect => true,
            _ => false })
        {
        // when the Fetch implementation does asynchronous retrieval of the body,
        // we will need to make sure nothing tries to write to the body at this point
        *internal_response.body.borrow_mut() = ResponseBody::Empty;
    }

    // Step 15
    // TODO be able to compare response integrity against request integrity metadata
    // if !Response::is_network_error(&response) {

    //     // Substep 1
    //     // TODO wait for response

    //     // Substep 2
    //     if response.termination_reason.is_none() {
    //         response = Response::network_error();
    //         internal_response = Response::network_error();
    //     }
    // }

    // Step 16
    if request.synchronous {
        // TODO wait for internal_response
        return response;
    }

    // Step 17
    if request.body.is_some() && match &*request.current_url().scheme {
        "http" | "https" => true,
        _ => false }
        {
        // TODO queue a fetch task on request to process end-of-file
    }

    // Step 18
    // TODO this step

    match *internal_response.body.borrow() {
        // Step 20
        ResponseBody::Empty => {
            // Substep 1
            // Substep 2
        },

        // Step 19
        _ => {
            // Substep 1
            // Substep 2
        }
    };

    // TODO remove this line when asynchronous fetches are supported
    return response;
}

/// [Basic fetch](https://fetch.spec.whatwg.org#basic-fetch)
fn basic_fetch(request: Rc<Request>) -> Response {

    let url = request.current_url();
    let scheme = url.scheme.clone();

    match &*scheme {

        "about" => {
            match url.non_relative_scheme_data() {
                Some(s) if &*s == "blank" => {
                    let mut response = Response::new();
                    response.headers.set(ContentType(Mime(
                        TopLevel::Text, SubLevel::Html,
                        vec![(Attr::Charset, Value::Utf8)])));
                    response
                },
                _ => Response::network_error()
            }
        },

        "http" | "https" => {
            http_fetch(request.clone(), BasicCORSCache::new(), false, false, false)
        },

        "blob" | "data" | "file" | "ftp" => {
            // XXXManishearth handle these
            panic!("Unimplemented scheme for Fetch")
        },

        _ => Response::network_error()
    }
}

fn http_fetch_async(request: Request,
                    cors_flag: bool,
                    cors_preflight_flag: bool,
                    authentication_fetch_flag: bool,
                    listener: Box<AsyncFetchListener + Send>) {

    spawn_named(format!("http_fetch for {:?}", request.get_last_url_string()), move || {
        let request = Rc::new(request);
        let res = http_fetch(request, BasicCORSCache::new(),
                             cors_flag, cors_preflight_flag,
                             authentication_fetch_flag);
        listener.response_available(res);
    });
}

/// [HTTP fetch](https://fetch.spec.whatwg.org#http-fetch)
fn http_fetch(request: Rc<Request>,
              mut cache: BasicCORSCache,
              cors_flag: bool,
              cors_preflight_flag: bool,
              authentication_fetch_flag: bool) -> Response {

    // Step 1
    let mut response: Option<Rc<Response>> = None;

    // Step 2
    let mut actual_response: Option<Rc<Response>> = None;

    // Step 3
    if !request.skip_service_worker.get() && !request.is_service_worker_global_scope {

        // Substep 1
        // TODO (handle fetch unimplemented)

        if let Some(ref res) = response {

            // Substep 2
            actual_response = match res.internal_response {
                Some(ref internal_res) => Some(internal_res.clone()),
                None => Some(res.clone())
            };

            // Substep 3
            if (res.response_type == ResponseType::Opaque &&
                request.mode != RequestMode::NoCORS) ||
               (res.response_type == ResponseType::OpaqueRedirect &&
                request.redirect_mode.get() != RedirectMode::Manual) ||
               res.response_type == ResponseType::Error {
                return Response::network_error();
            }
        }

        // Substep 4
        if let Some(ref res) = actual_response {
            if res.url_list.borrow().is_empty() {
                *res.url_list.borrow_mut() = request.url_list.borrow().clone();
            }
        }

        // Substep 5
        // TODO: set response's CSP list on actual_response
    }

    // Step 4
    if response.is_none() {

        // Substep 1
        if cors_preflight_flag {
            let mut method_mismatch = false;
            let mut header_mismatch = false;

            let origin = request.origin.clone();
            let url = request.current_url();
            let credentials = request.credentials_mode == CredentialsMode::Include;
            let method_cache_match = cache.match_method(CacheRequestDetails {
                origin: origin.clone(),
                destination: url.clone(),
                credentials: credentials
            }, request.method.borrow().clone());

            method_mismatch = !method_cache_match && (!is_simple_method(&request.method.borrow()) ||
                request.use_cors_preflight);
            header_mismatch = request.headers.borrow().iter().any(|view|
                !cache.match_header(CacheRequestDetails {
                    origin: origin.clone(),
                    destination: url.clone(),
                    credentials: credentials
                }, view.name()) && !is_simple_header(&view)
            );

            // Sub-substep 1
            if method_mismatch || header_mismatch {
                let preflight_result = preflight_fetch(request.clone());
                // Sub-substep 2
                if preflight_result.response_type == ResponseType::Error {
                    return Response::network_error();
                }
            }
        }

        // Substep 2
        request.skip_service_worker.set(true);

        // Substep 3
        let credentials = match request.credentials_mode {
            CredentialsMode::Include => true,
            CredentialsMode::CredentialsSameOrigin if (!cors_flag ||
                request.response_tainting.get() == ResponseTainting::Opaque)
                => true,
            _ => false
        };

        // Substep 4
        let fetch_result = http_network_or_cache_fetch(request.clone(), credentials, authentication_fetch_flag);

        // Substep 5
        if cors_flag && cors_check(request.clone(), &fetch_result).is_err() {
            return Response::network_error();
        }

        response = Some(Rc::new(fetch_result));
        actual_response = response.clone();
    }

    // response and actual_response are guaranteed to be something by now
    let mut response = response.unwrap();
    let actual_response = actual_response.unwrap();

    // Step 5
    match actual_response.status.unwrap() {

        // Code 301, 302, 303, 307, 308
        StatusCode::MovedPermanently | StatusCode::Found | StatusCode::SeeOther |
        StatusCode::TemporaryRedirect | StatusCode::PermanentRedirect => {

            // Step 1
            if request.redirect_mode.get() == RedirectMode::Error {
                return Response::network_error();
            }

            // Step 3
            if !actual_response.headers.has::<Location>() {
                drop(actual_response);
                return Rc::try_unwrap(response).ok().unwrap();
            }

            // Step 2
            let location = match actual_response.headers.get::<Location>() {
                Some(&Location(ref location)) => location.clone(),
                // Step 4
                _ => return Response::network_error(),
            };

            // Step 5
            let location_url = UrlParser::new().base_url(&request.current_url()).parse(&*location);

            // Step 6
            let location_url = match location_url {
                Ok(url) => url,
                _ => { return Response::network_error(); }
            };

            // Step 7
            if request.redirect_count.get() >= 20 {
                return Response::network_error();
            }

            // Step 8
            request.redirect_count.set(request.redirect_count.get() + 1);

            // Step 9
            request.same_origin_data.set(false);

            match request.redirect_mode.get() {

                // Step 10
                RedirectMode::Manual => {
                    response = Rc::new(Response::to_filtered(actual_response, ResponseType::OpaqueRedirect));
                }

                // Step 11
                RedirectMode::Follow => {

                    // Substep 1
                    // FIXME: Use Url::origin
                    // if (request.mode == RequestMode::CORSMode ||
                    //     request.mode == RequestMode::ForcedPreflightMode) &&
                    //     location_url.origin() != request.url.origin() &&
                    //     has_credentials(&location_url) {
                    //     return Response::network_error();
                    // }

                    // Substep 2
                    if cors_flag && has_credentials(&location_url) {
                        return Response::network_error();
                    }

                    // Substep 3
                    // FIXME: Use Url::origin
                    // if cors_flag && location_url.origin() != request.url.origin() {
                    //     request.borrow_mut().origin = Origin::UID(OpaqueOrigin);
                    // }

                    // Substep 4
                    if actual_response.status.unwrap() == StatusCode::SeeOther ||
                       ((actual_response.status.unwrap() == StatusCode::MovedPermanently ||
                         actual_response.status.unwrap() == StatusCode::Found) &&
                        *request.method.borrow() == Method::Post) {
                        *request.method.borrow_mut() = Method::Get;
                    }

                    // Substep 5
                    request.url_list.borrow_mut().push(location_url);

                    // Substep 6
                    return main_fetch(request.clone(), cors_flag, true);
                }
                RedirectMode::Error => { panic!("RedirectMode is Error after step 8") }
            }
        }

        // Code 401
        StatusCode::Unauthorized => {

            // Step 1
            // FIXME: Figure out what to do with request window objects
            if cors_flag {
                drop(actual_response);
                return Rc::try_unwrap(response).ok().unwrap();
            }

            // Step 2
            // TODO: Spec says requires testing on multiple WWW-Authenticate headers

            // Step 3
            if !request.use_url_credentials || authentication_fetch_flag {
                // TODO: Prompt the user for username and password from the window
            }

            // Step 4
            return http_fetch(request, BasicCORSCache::new(), cors_flag, cors_preflight_flag, true);
        }

        // Code 407
        StatusCode::ProxyAuthenticationRequired => {

            // Step 1
            // TODO: Figure out what to do with request window objects

            // Step 2
            // TODO: Spec says requires testing on Proxy-Authenticate headers

            // Step 3
            // TODO: Prompt the user for proxy authentication credentials

            // Step 4
            return http_fetch(request, BasicCORSCache::new(),
                              cors_flag, cors_preflight_flag,
                              authentication_fetch_flag);
        }

        _ => drop(actual_response)
    }

    // Step 6
    if authentication_fetch_flag {
        // TODO: Create authentication entry for this request
    }

    // Step 7
    Rc::try_unwrap(response).ok().unwrap()
}

/// [HTTP network or cache fetch](https://fetch.spec.whatwg.org#http-network-or-cache-fetch)
fn http_network_or_cache_fetch(request: Rc<Request>,
                               credentials_flag: bool,
                               authentication_fetch_flag: bool) -> Response {

    // TODO: Implement Window enum for Request
    let request_has_no_window = true;

    // Step 1
    let http_request = if request_has_no_window &&
        request.redirect_mode.get() != RedirectMode::Follow {
        request.clone()
    } else {
        Rc::new((*request).clone())
    };

    let content_length_value = match http_request.body {
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
    match http_request.referer {
        Referer::NoReferer =>
            http_request.headers.borrow_mut().set(RefererHeader("".to_owned())),
        Referer::RefererUrl(ref http_request_referer) =>
            http_request.headers.borrow_mut().set(RefererHeader(http_request_referer.serialize())),
        Referer::Client =>
            // it should be impossible for referer to be anything else during fetching
            // https://fetch.spec.whatwg.org/#concept-request-referrer
            unreachable!()
    };

    // Step 7
    if http_request.omit_origin_header == false {
        // TODO update this when https://github.com/hyperium/hyper/pull/691 is finished
        // http_request.headers.borrow_mut().set_raw("origin", origin);
    }

    // Step 8
    if !http_request.headers.borrow().has::<UserAgent>() {
        http_request.headers.borrow_mut().set(UserAgent(global_user_agent().to_owned()));
    }

    // Step 9
    if http_request.cache_mode.get() == CacheMode::Default && is_no_store_cache(&http_request.headers.borrow()) {
        http_request.cache_mode.set(CacheMode::NoStore);
    }

    // Step 10
    // modify_request_headers(http_request.headers.borrow());

    // Step 11
    // TODO some of this step can't be implemented yet
    if credentials_flag {
        // Substep 1
        // TODO http://mxr.mozilla.org/servo/source/components/net/http_loader.rs#504

        // Substep 2
        let mut authorization_value = None;

        // Substep 3
        // TODO be able to retrieve https://fetch.spec.whatwg.org/#authentication-entry

        // Substep 4
        if authentication_fetch_flag {

            let current_url = http_request.current_url();

            authorization_value = if includes_credentials(&current_url) {
                Some(Basic {
                    username: current_url.username().unwrap_or("").to_owned(),
                    password: current_url.password().map(str::to_owned)
                })

            } else {
                None
            }
        }

        // Substep 5
        if let Some(basic) = authorization_value {
            http_request.headers.borrow_mut().set(Authorization(basic));
        }
    }

    // Step 12
    // TODO this step can't be implemented

    // Step 13
    let mut response: Option<Response> = None;

    // Step 14
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

    // Step 15
    // TODO have a HTTP cache to check for a partial response
    } else if http_request.cache_mode.get() == CacheMode::Default ||
        http_request.cache_mode.get() == CacheMode::ForceCache {
        // TODO this substep
    }

    // Step 16
    if response.is_none() {
        response = Some(http_network_fetch(request.clone(), http_request.clone(), credentials_flag));
    }
    let response = response.unwrap();

    // Step 17
    if let Some(status) = response.status {
        if status == StatusCode::NotModified &&
            (http_request.cache_mode.get() == CacheMode::Default ||
            http_request.cache_mode.get() == CacheMode::NoCache) {

            // Substep 1
            // TODO this substep
            let cached_response: Option<Response> = None;

            // Substep 2
            if cached_response.is_none() {
                return Response::network_error();
            }

            // Substep 3

            // Substep 4
            // response = cached_response;

            // Substep 5
            // TODO cache_state is immutable?
            // response.cache_state = CacheState::Validated;
        }
    }

    // Step 18
    response
}

/// [HTTP network fetch](https://fetch.spec.whatwg.org/#http-network-fetch)
fn http_network_fetch(request: Rc<Request>,
                      http_request: Rc<Request>,
                      credentials_flag: bool) -> Response {
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

    let wrapped_response = obtain_response(&factory, &url, &request.method.borrow(),
                                           &request.headers.borrow(),
                                           &cancellation_listener, &None, &request.method.borrow(),
                                           &None, request.redirect_count.get(), &None, "");

    let mut response = Response::new();
    match wrapped_response {
        Ok(mut res) => {
            // is it okay for res.version to be unused?
            response.url = Some(res.response.url.clone());
            response.status = Some(res.response.status);
            response.headers = res.response.headers.clone();

            let mut body = vec![];
            res.response.read_to_end(&mut body);
            *response.body.borrow_mut() = ResponseBody::Done(body);
        },
        Err(e) =>
            response.termination_reason = Some(TerminationReason::Fatal)
    };

        // TODO these substeps aren't possible yet
        // Substep 1

        // Substep 2

    // TODO how can I tell if response was retrieved over HTTPS?
    // TODO Servo needs to decide what ciphers are to be treated as "deprecated"
    response.https_state = HttpsState::None;

    // TODO how do I read request?

    // Step 5
    // TODO when https://bugzilla.mozilla.org/show_bug.cgi?id=1030660
    // is resolved, this step will become uneccesary
    // TODO this step
    if let Some(encoding) = response.headers.get::<ContentEncoding>() {
        if encoding.contains(&Encoding::Gzip) {

        }

        else if encoding.contains(&Encoding::Compress) {

        }
    };

    // Step 6
    *response.url_list.borrow_mut() = request.url_list.borrow().clone();

    // Step 7
    // TODO this step isn't possible yet

    // Step 8
    if Response::is_network_error(&response) && request.cache_mode.get() == CacheMode::NoStore {
        // TODO update response in the HTTP cache for request
    }

    // TODO these steps aren't possible yet
    // Step 9
        // Substep 1
        // Substep 2
        // Substep 3
        // Substep 4

    // TODO these steps
    // Step 10
        // Substep 1
        // Substep 2
            // Sub-substep 1
            // Sub-substep 2
            // Sub-substep 3
            // Sub-substep 4
        // Substep 3

    // Step 11
    response
}

/// [CORS preflight fetch](https://fetch.spec.whatwg.org#cors-preflight-fetch)
fn preflight_fetch(request: Rc<Request>) -> Response {
    // TODO: Implement preflight fetch spec
    Response::network_error()
}

/// [CORS check](https://fetch.spec.whatwg.org#concept-cors-check)
fn cors_check(request: Rc<Request>, response: &Response) -> Result<(), ()> {
    // TODO: Implement CORS check spec
    Err(())
}

fn global_user_agent() -> String {
    // TODO have a better useragent string
    const USER_AGENT_STRING: &'static str = "Servo";
    USER_AGENT_STRING.to_owned()
}

fn has_credentials(url: &Url) -> bool {
    !url.username().unwrap_or("").is_empty() || url.password().is_some()
}

fn is_no_store_cache(headers: &Headers) -> bool {
    headers.has::<IfModifiedSince>() | headers.has::<IfNoneMatch>() |
    headers.has::<IfUnmodifiedSince>() | headers.has::<IfMatch>() |
    headers.has::<IfRange>()
}

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

fn includes_credentials(url: &Url) -> bool {

    if url.password().is_some() {
        return true
    }

    if let Some(name) = url.username() {
        return name.len() > 0
    }

    false
}

fn response_needs_revalidation(response: &Response) -> bool {
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
