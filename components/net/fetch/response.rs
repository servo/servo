/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use url::Url;
use hyper::status::StatusCode;
use hyper::status::Ok as StatusOk;
use hyper::header::Headers;
use std::ascii::AsciiExt;
use std::comm::Receiver;

/// [Response type](http://fetch.spec.whatwg.org/#concept-response-type)
#[deriving(Clone, PartialEq)]
pub enum ResponseType {
    Basic,
    CORS,
    Default,
    Error,
    Opaque
}

/// [Response termination reason](http://fetch.spec.whatwg.org/#concept-response-termination-reason)
#[deriving(Clone)]
pub enum TerminationReason {
    EndUserAbort,
    Fatal,
    Timeout
}

/// The response body can still be pushed to after fetch
/// This provides a way to store unfinished response bodies
#[unstable = "I haven't yet decided exactly how the interface for this will be"]
#[deriving(Clone)]
pub enum ResponseBody {
    Empty, // XXXManishearth is this necessary, or is Done(vec![]) enough?
    Receiving(Vec<u8>),
    Done(Vec<u8>),
}

#[unstable = "I haven't yet decided exactly how the interface for this will be"]
pub enum ResponseMsg {
    Chunk(Vec<u8>),
    Finished,
    Errored
}

#[unstable = "I haven't yet decided exactly how the interface for this will be"]
pub struct ResponseLoader {
    response: Response,
    chan: Receiver<ResponseMsg>
}

/// A [Response](http://fetch.spec.whatwg.org/#concept-response) as defined by the Fetch spec
#[deriving(Clone)]
pub struct Response {
    pub response_type: ResponseType,
    pub termination_reason: Option<TerminationReason>,
    pub url: Option<Url>,
    /// `None` can be considered a StatusCode of `0`.
    pub status: Option<StatusCode>,
    pub headers: Headers,
    pub body: ResponseBody,
    /// [Internal response](http://fetch.spec.whatwg.org/#concept-internal-response), only used if the Response is a filtered response
    pub internal_response: Option<Box<Response>>,
}

impl Response {
    pub fn new() -> Response {
        Response {
            response_type: Default,
            termination_reason: None,
            url: None,
            status: Some(StatusOk),
            headers: Headers::new(),
            body: Empty,
            internal_response: None
        }
    }

    pub fn network_error() -> Response {
        Response {
            response_type: Error,
            termination_reason: None,
            url: None,
            status: None,
            headers: Headers::new(),
            body: Empty,
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
                let headers = old_headers.iter().filter(|header| {
                    match header.name().to_ascii_lower().as_slice() {
                        "set-cookie" | "set-cookie2" => false,
                        _ => true
                    }
                }).collect();
                response.headers = headers;
                response.response_type = filter_type;
            },
            CORS => {
                let headers = old_headers.iter().filter(|header| {
                    match header.name().to_ascii_lower().as_slice() {
                        "cache-control" | "content-language" |
                        "content-type" | "expires" | "last-modified" | "Pragma" => false,
                        // XXXManishearth handle Access-Control-Expose-Headers
                        _ => true
                    }
                }).collect();
                response.headers = headers;
                response.response_type = filter_type;
            },
            Opaque => {
                response.headers = Headers::new();
                response.status = None;
                response.body = Empty;
            }
        }
        response
    }
}
