/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use std::sync::{Arc, Mutex};

use net_traits::request::Request;

/// https://fetch.spec.whatwg.org/#fetch-params
#[derive(Clone)]
pub struct FetchParams {
    /// https://fetch.spec.whatwg.org/#concept-request
    pub request: Request,
    /// https://fetch.spec.whatwg.org/#fetch-controller
    pub controller: Arc<Mutex<FetchController>>,
}

impl FetchParams {
    pub fn new(request: Request) -> FetchParams {
        FetchParams {
            request,
            controller: Arc::new(Mutex::new(FetchController::default())),
        }
    }
}

/// https://fetch.spec.whatwg.org/#fetch-controller
#[derive(Clone, Debug, Default)]
pub struct FetchController {
    /// https://fetch.spec.whatwg.org/#fetch-controller-state
    pub state: FetchControllerState,
}

/// https://fetch.spec.whatwg.org/#fetch-controller-state
#[derive(Clone, Copy, Debug, Default)]
pub enum FetchControllerState {
    #[default]
    Ongoing,
    Terminated,
    Aborted,
}
