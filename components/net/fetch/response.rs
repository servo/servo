/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use hyper::header::Headers;
use hyper::status::StatusCode;
use net_traits::response::{CacheState, HttpsState, Response, ResponseBody, ResponseType};
use std::ascii::AsciiExt;
use std::cell::{Cell, RefCell};
use std::rc::Rc;
use std::sync::mpsc::Receiver;
use std::sync::{Arc, Mutex};
use url::Url;

pub trait ResponseMethods {
    fn new() -> Response;
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
            body: Arc::new(Mutex::new(ResponseBody::Empty)),
            cache_state: CacheState::None,
            https_state: HttpsState::None,
            internal_response: None,
            return_internal: Cell::new(true)
        }
    }
}

