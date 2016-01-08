/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use hyper::header::Headers;
use hyper::status::StatusCode;
use net_traits::response::{CacheState, HttpsState, Response, ResponseBody, ResponseType};
use std::ascii::AsciiExt;
use std::cell::RefCell;
use std::rc::Rc;
use std::sync::mpsc::Receiver;
use url::Url;

pub trait ResponseMethods {
    fn new() -> Response;
    fn to_filtered(Rc<Response>, ResponseType) -> Response;
}

impl ResponseMethods for Response {
    fn new() -> Response {
        Response {
            response_type: ResponseType::Default,
            termination_reason: None,
            url: None,
            url_list: RefCell::new(Vec::new()),
            status: Some(StatusCode::Ok),
            headers: Headers::new(),
            body: ResponseBody::Empty,
            cache_state: CacheState::None,
            https_state: HttpsState::None,
            internal_response: None
        }
    }

    /// Convert to a filtered response, of type `filter_type`.
    /// Do not use with type Error or Default
    fn to_filtered(old_response: Rc<Response>, filter_type: ResponseType) -> Response {

        assert!(filter_type != ResponseType::Error);
        assert!(filter_type != ResponseType::Default);

        if old_response.response_type == ResponseType::Error {
            return Response::network_error();
        }

        let old_headers = old_response.headers.clone();
        let mut response = (*old_response).clone();
        response.internal_response = Some(old_response);

        match filter_type {

            ResponseType::Default | ResponseType::Error => unreachable!(),

            ResponseType::Basic => {
                let headers = old_headers.iter().filter(|header| {
                    match &*header.name().to_ascii_lowercase() {
                        "set-cookie" | "set-cookie2" => false,
                        _ => true
                    }
                }).collect();
                response.headers = headers;
                response.response_type = filter_type;
            },

            ResponseType::CORS => {
                let headers = old_headers.iter().filter(|header| {
                    match &*header.name().to_ascii_lowercase() {
                        "cache-control" | "content-language" |
                        "content-type" | "expires" | "last-modified" | "Pragma" => false,
                        // XXXManishearth handle Access-Control-Expose-Headers
                        _ => true
                    }
                }).collect();
                response.headers = headers;
                response.response_type = filter_type;
            },

            ResponseType::Opaque |
            ResponseType::OpaqueRedirect => {
                response.headers = Headers::new();
                response.status = None;
                response.body = ResponseBody::Empty;
            }
        }

        response
    }
}
