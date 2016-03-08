/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

//! The [Response](https://fetch.spec.whatwg.org/#responses) object
//! resulting from a [fetch operation](https://fetch.spec.whatwg.org/#concept-fetch)
use hyper::header::{AccessControlExposeHeaders, Headers};
use hyper::status::StatusCode;
use std::ascii::AsciiExt;
use std::cell::{Cell, RefCell};
use std::sync::{Arc, Mutex};
use url::Url;

/// [Response type](https://fetch.spec.whatwg.org/#concept-response-type)
#[derive(Clone, PartialEq, Copy, Debug)]
pub enum ResponseType {
    Basic,
    CORS,
    Default,
    Error,
    Opaque,
    OpaqueRedirect
}

/// [Response termination reason](https://fetch.spec.whatwg.org/#concept-response-termination-reason)
#[derive(Clone, Copy)]
pub enum TerminationReason {
    EndUserAbort,
    Fatal,
    Timeout
}

/// The response body can still be pushed to after fetch
/// This provides a way to store unfinished response bodies
#[derive(Clone, Debug, PartialEq)]
pub enum ResponseBody {
    Empty, // XXXManishearth is this necessary, or is Done(vec![]) enough?
    Receiving(Vec<u8>),
    Done(Vec<u8>),
}

impl ResponseBody {
    pub fn is_done(&self) -> bool {
        match *self {
            ResponseBody::Done(..) => true,
            ResponseBody::Empty | ResponseBody::Receiving(..) => false
        }
    }
}


/// [Cache state](https://fetch.spec.whatwg.org/#concept-response-cache-state)
#[derive(Clone, Debug)]
pub enum CacheState {
    None,
    Local,
    Validated,
    Partial
}

/// [Https state](https://fetch.spec.whatwg.org/#concept-response-https-state)
#[derive(Clone, Copy, HeapSizeOf, Deserialize, Serialize)]
pub enum HttpsState {
    None,
    Deprecated,
    Modern
}

pub enum ResponseMsg {
    Chunk(Vec<u8>),
    Finished,
    Errored
}

/// A [Response](https://fetch.spec.whatwg.org/#concept-response) as defined by the Fetch spec
#[derive(Clone)]
pub struct Response {
    pub response_type: ResponseType,
    pub termination_reason: Option<TerminationReason>,
    pub url: Option<Url>,
    pub url_list: RefCell<Vec<Url>>,
    /// `None` can be considered a StatusCode of `0`.
    pub status: Option<StatusCode>,
    pub headers: Headers,
    pub body: Arc<Mutex<ResponseBody>>,
    pub cache_state: CacheState,
    pub https_state: HttpsState,
    /// [Internal response](https://fetch.spec.whatwg.org/#concept-internal-response), only used if the Response
    /// is a filtered response
    pub internal_response: Option<Box<Response>>,
    /// whether or not to try to return the internal_response when asked for actual_response
    pub return_internal: Cell<bool>,
}

impl Response {
    pub fn new() -> Response {
        Response {
            response_type: ResponseType::Default,
            termination_reason: None,
            url: None,
            url_list: RefCell::new(Vec::new()),
            status: Some(StatusCode::Ok),
            headers: Headers::new(),
            body: Arc::new(Mutex::new(ResponseBody::Empty)),
            cache_state: CacheState::None,
            https_state: HttpsState::None,
            internal_response: None,
            return_internal: Cell::new(true)
        }
    }

    pub fn network_error() -> Response {
        Response {
            response_type: ResponseType::Error,
            termination_reason: None,
            url: None,
            url_list: RefCell::new(vec![]),
            status: None,
            headers: Headers::new(),
            body: Arc::new(Mutex::new(ResponseBody::Empty)),
            cache_state: CacheState::None,
            https_state: HttpsState::None,
            internal_response: None,
            return_internal: Cell::new(true)
        }
    }

    pub fn is_network_error(&self) -> bool {
        match self.response_type {
            ResponseType::Error => true,
            _ => false
        }
    }

    pub fn wait_until_done(&self) {
        while !self.body.lock().unwrap().is_done() && !self.is_network_error() {
        }
    }

    pub fn get_actual_response(&self) -> &Response {
        if self.return_internal.get() && self.internal_response.is_some() {
            &**self.internal_response.as_ref().unwrap()
        } else {
            self
        }
    }

    pub fn to_actual(self) -> Response {
        if self.return_internal.get() && self.internal_response.is_some() {
            *self.internal_response.unwrap()
        } else {
            self
        }
    }

    /// Convert to a filtered response, of type `filter_type`.
    /// Do not use with type Error or Default
    pub fn to_filtered(self, filter_type: ResponseType) -> Response {

        assert!(filter_type != ResponseType::Error);
        assert!(filter_type != ResponseType::Default);

        let old_response = self.to_actual();

        if Response::is_network_error(&old_response) {
            return Response::network_error();
        }

        let old_headers = old_response.headers.clone();
        let mut response = old_response.clone();
        response.internal_response = Some(Box::new(old_response));
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
                        "cache-control" | "content-language" | "content-type" |
                        "expires" | "last-modified" | "pragma" => true,
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
                response.body = Arc::new(Mutex::new(ResponseBody::Empty));
                response.cache_state = CacheState::None;
            },

            ResponseType::OpaqueRedirect => {
                response.headers = Headers::new();
                response.status = None;
                response.body = Arc::new(Mutex::new(ResponseBody::Empty));
                response.cache_state = CacheState::None;
            }
        }

        response
    }
}
