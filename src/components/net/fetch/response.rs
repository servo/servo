/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use url::Url;
use http::status::{Status, UnregisteredStatus};
use StatusOk = http::status::Ok;
use http::headers::HeaderEnum;
use http::headers::response::HeaderCollection;
use std::ascii::OwnedStrAsciiExt;

// [Response type](http://fetch.spec.whatwg.org/#concept-response-type)
#[deriving(Clone, PartialEq)]
pub enum ResponseType {
    Basic,
    CORS,
    Default,
    Error,
    Opaque
}

// [Response termination reason](http://fetch.spec.whatwg.org/#concept-response-termination-reason)
#[deriving(Clone)]
pub enum TerminationReason {
    EndUserAbort,
    Fatal,
    Timeout
}

// A [Response](http://fetch.spec.whatwg.org/#concept-response) as defined by the Fetch spec
#[deriving(Clone)]
pub struct Response {
    pub response_type: ResponseType,
    pub termination_reason: Option<TerminationReason>,
    pub url: Option<Url>,
    pub status: Status,
    pub headers: HeaderCollection,
    pub body: Option<Vec<u8>>,
    /// [Internal response](http://fetch.spec.whatwg.org/#concept-internal-response), only used if the Response is a filtered response
    pub internal_response: Option<Box<Response>>,
}

impl Response {
    pub fn new() -> Response {
        Response {
            response_type: Default,
            termination_reason: None,
            url: None,
            status: StatusOk,
            headers: HeaderCollection::new(),
            body: None,
            internal_response: None
        }
    }

    pub fn is_network_error(&self) -> bool {
        match self.response_type {
            Error => true,
            _ => false
        }
    }

    /// Convert to a filtered response, of type `filter_type`.
    /// Do not use with type Error or Default
    pub fn to_filtered(self, filter_type: ResponseType) -> Response {
        assert!(filter_type != Error);
        assert!(filter_type != Default);
        if self.is_network_error() {
            return self;
        }
        let old_headers = self.headers.clone();
        let mut response = self.clone();
        response.internal_response = Some(box self);
        match filter_type {
            Default | Error => unreachable!(),
            Basic => {
                let mut headers = HeaderCollection::new();
                for h in old_headers.iter() {
                    match h.header_name().into_ascii_lower().as_slice() {
                        "set-cookie" | "set-cookie2" => {},
                        _ => headers.insert(h)
                    }
                }
                response.headers = headers;
                response.response_type = filter_type;
            },
            CORS => {
                let mut headers = HeaderCollection::new();
                for h in old_headers.iter() {
                    match h.header_name().into_ascii_lower().as_slice() {
                        "cache-control" | "content-language" |
                        "content-type" | "expires" | "last-modified" | "Pragma" => {},
                        // XXXManishearth handle Access-Control-Expose-Headers
                        _ => headers.insert(h)
                    }
                }
                response.headers = headers;
                response.response_type = filter_type;
            },
            Opaque => {
                response.headers = HeaderCollection::new();
                response.status = UnregisteredStatus(0, "".to_string());
                response.body = None;
            }
        }
        response
    }
}
