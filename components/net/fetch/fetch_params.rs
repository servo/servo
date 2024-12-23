/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use std::sync::{Arc, Mutex};

use net_traits::request::Request;

#[derive(Clone)]
pub struct FetchParams {
    pub request: Request,
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

#[derive(Clone, Debug, Default)]
pub struct FetchController {
    pub state: FetchControllerState,
}

#[derive(Clone, Copy, Debug, Default)]
pub enum FetchControllerState {
    #[default]
    Ongoing,
    Terminated,
    Aborted,
}
