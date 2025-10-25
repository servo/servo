/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use net_traits::request::Request;
use tokio::sync::oneshot::{Receiver as TokioReceiver};
use net_traits::response::Response;

/// <https://fetch.spec.whatwg.org/#fetch-params-preloaded-response-candidate>
pub(crate) enum PreloadResponseCandidate {
    None,
    Pending(TokioReceiver<Response>),
    Response(Box<Response>),
}

impl PreloadResponseCandidate {
    /// Part of Step 12 of <https://fetch.spec.whatwg.org/#concept-main-fetch>
    pub async fn response(&mut self) -> Option<Response> {
        // Step 3. Return fetchParams’s preloaded response candidate.
        match self {
            PreloadResponseCandidate::None => None,
            // Step 2. Assert: fetchParams’s preloaded response candidate is a response.
            PreloadResponseCandidate::Response(response) => Some(*response.clone()),
            // Step 1. Wait until fetchParams’s preloaded response candidate is not "pending".
            PreloadResponseCandidate::Pending(receiver) => {
                receiver.await.ok()
            },
        }
    }
}

/// <https://fetch.spec.whatwg.org/#fetch-params>
pub struct FetchParams {
    /// <https://fetch.spec.whatwg.org/#fetch-params-request>
    pub request: Request,
    /// <https://fetch.spec.whatwg.org/#fetch-params-preloaded-response-candidate>
    pub(crate) preload_response_candidate: PreloadResponseCandidate,
}

impl FetchParams {
    pub fn new(request: Request) -> FetchParams {
        FetchParams {
            request,
            preload_response_candidate: PreloadResponseCandidate::None,
        }
    }
}
