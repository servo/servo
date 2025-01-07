/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use net_traits::request::Request;

/// <https://fetch.spec.whatwg.org/#fetch-params>
#[derive(Clone)]
pub struct FetchParams {
    /// <https://fetch.spec.whatwg.org/#concept-request>
    pub request: Request,
}

impl FetchParams {
    pub fn new(request: Request) -> FetchParams {
        FetchParams { request }
    }
}
