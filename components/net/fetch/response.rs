/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use hyper::header::{AccessControlExposeHeaders, Headers};
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
            body: RefCell::new(ResponseBody::Empty),
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

        if Response::is_network_error(&old_response) {
            return Response::network_error();
        }

        let old_headers = old_response.headers.clone();
        let mut response = (*old_response).clone();
        response.internal_response = Some(old_response);
        response.response_type = filter_type;

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
            },

            ResponseType::CORS => {

                let access = old_headers.get::<AccessControlExposeHeaders>();
                let allowed_headers = access.as_ref().map(|v| &v[..]).unwrap_or(&[]);

                let headers = old_headers.iter().filter(|header| {
                    match &*header.name().to_ascii_lowercase() {
                        "cache-control" | "content-language" |
                        "content-type" | "expires" | "last-modified" | "Pragma" => true,
                        "set-cookie" | "set-cookie2" => false,
                        header => {
                            let result =
                                allowed_headers.iter().find(|h| *header == *h.to_ascii_lowercase());
                            result.is_some()
                        }
                    }
                }).collect();
                response.headers = headers;
            },

            ResponseType::Opaque => {
                response.url_list = RefCell::new(vec![]);
                response.url = None;
                response.headers = Headers::new();
                response.status = None;
                response.body = RefCell::new(ResponseBody::Empty);
            },

            ResponseType::OpaqueRedirect => {
                response.headers = Headers::new();
                response.status = None;
                response.body = RefCell::new(ResponseBody::Empty);
            }
        }

        response
    }
}
