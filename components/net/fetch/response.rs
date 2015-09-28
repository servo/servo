/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use hyper::header::Headers;
use hyper::status::StatusCode;
use net_traits::{Response, ResponseBody, ResponseType};
use std::ascii::AsciiExt;
use std::sync::mpsc::Receiver;
use url::Url;

pub trait ResponseMethods {
    fn new() -> Response;
    fn to_filtered(self, ResponseType) -> Response;
}

impl ResponseMethods for Response {
    fn new() -> Response {
        Response {
            response_type: ResponseType::Default,
            termination_reason: None,
            url: None,
            status: Some(StatusCode::Ok),
            headers: Headers::new(),
            body: ResponseBody::Empty,
            internal_response: None
        }
    }

    /// Convert to a filtered response, of type `filter_type`.
    /// Do not use with type Error or Default
    fn to_filtered(self, filter_type: ResponseType) -> Response {
        assert!(filter_type != ResponseType::Error);
        assert!(filter_type != ResponseType::Default);
        if self.is_network_error() {
            return self;
        }
        let old_headers = self.headers.clone();
        let mut response = self.clone();
        response.internal_response = Some(box self);
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
            ResponseType::Opaque => {
                response.headers = Headers::new();
                response.status = None;
                response.body = ResponseBody::Empty;
            }
        }
        response
    }
}
