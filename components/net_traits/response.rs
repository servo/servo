/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

//! The [Response](https://fetch.spec.whatwg.org/#responses) object
//! resulting from a [fetch operation](https://fetch.spec.whatwg.org/#concept-fetch)
use hyper::header::Headers;
use hyper::status::StatusCode;
use std::cell::RefCell;
use std::rc::Rc;
use url::Url;

/// [Response type](https://fetch.spec.whatwg.org/#concept-response-type)
#[derive(Clone, PartialEq, Copy)]
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
#[derive(Clone)]
pub enum ResponseBody {
    Empty, // XXXManishearth is this necessary, or is Done(vec![]) enough?
    Receiving(Vec<u8>),
    Done(Vec<u8>),
}

/// [Cache state](https://fetch.spec.whatwg.org/#concept-response-cache-state)
#[derive(Clone)]
pub enum CacheState {
    None,
    Local,
    Validated,
    Partial
}

/// [Https state](https://fetch.spec.whatwg.org/#concept-response-https-state)
#[derive(Clone)]
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
    pub body: ResponseBody,
    pub cache_state: CacheState,
    pub https_state: HttpsState,
    /// [Internal response](https://fetch.spec.whatwg.org/#concept-internal-response), only used if the Response
    /// is a filtered response
    pub internal_response: Option<Rc<Response>>,
}

impl Response {
    pub fn network_error() -> Response {
        Response {
            response_type: ResponseType::Error,
            termination_reason: None,
            url: None,
            url_list: RefCell::new(vec![]),
            status: None,
            headers: Headers::new(),
            body: ResponseBody::Empty,
            cache_state: CacheState::None,
            https_state: HttpsState::None,
            internal_response: None
        }
    }

    pub fn is_network_error(&self) -> bool {
        match self.response_type {
            ResponseType::Error => true,
            _ => false
        }
    }
}
