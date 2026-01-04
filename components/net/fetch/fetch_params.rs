/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use std::sync::{Arc, Mutex};

use content_security_policy as csp;
use net_traits::request::{PreloadEntry, PreloadId, PreloadKey, Request, RequestClient};
use net_traits::response::Response;
use rustc_hash::FxHashMap;
use tokio::sync::oneshot::Receiver as TokioReceiver;

/// <https://fetch.spec.whatwg.org/#fetch-params-preloaded-response-candidate>
pub enum PreloadResponseCandidate {
    None,
    Pending(TokioReceiver<Response>, PreloadId),
    Response(Box<Response>, PreloadId),
}

impl PreloadResponseCandidate {
    /// Part of Step 12 of <https://fetch.spec.whatwg.org/#concept-main-fetch>
    pub(crate) async fn response(&mut self) -> Option<(Response, PreloadId)> {
        // Step 3. Return fetchParams’s preloaded response candidate.
        match self {
            PreloadResponseCandidate::None => None,
            // Step 2. Assert: fetchParams’s preloaded response candidate is a response.
            PreloadResponseCandidate::Response(response, preload_id) => {
                Some((*response.clone(), preload_id.clone()))
            },
            // Step 1. Wait until fetchParams’s preloaded response candidate is not "pending".
            PreloadResponseCandidate::Pending(receiver, preload_id) => receiver
                .await
                .ok()
                .map(|response| (response, preload_id.clone())),
        }
    }
}

/// <https://fetch.spec.whatwg.org/#fetch-params>
pub struct FetchParams {
    /// <https://fetch.spec.whatwg.org/#fetch-params-request>
    pub request: Request,
    /// <https://fetch.spec.whatwg.org/#fetch-params-preloaded-response-candidate>
    pub preload_response_candidate: PreloadResponseCandidate,
}

impl FetchParams {
    pub fn new(request: Request) -> FetchParams {
        FetchParams {
            request,
            preload_response_candidate: PreloadResponseCandidate::None,
        }
    }
}

pub type SharedPreloadedResources = Arc<Mutex<FxHashMap<PreloadId, PreloadEntry>>>;

pub trait ConsumePreloadedResources {
    fn consume_preloaded_resource(
        &self,
        request: &Request,
        preloaded_resources: SharedPreloadedResources,
    ) -> Option<PreloadResponseCandidate>;
}

impl ConsumePreloadedResources for RequestClient {
    /// <https://html.spec.whatwg.org/multipage/#consume-a-preloaded-resource>
    fn consume_preloaded_resource(
        &self,
        request: &Request,
        preloaded_resources: SharedPreloadedResources,
    ) -> Option<PreloadResponseCandidate> {
        // Step 1. Let key be a preload key whose URL is url,
        // destination is destination, mode is mode, and credentials mode is credentialsMode.
        let key = PreloadKey {
            url: request.url().clone(),
            destination: request.destination,
            mode: request.mode.clone(),
            credentials_mode: request.credentials_mode,
        };
        // Step 2. Let preloads be window's associated Document's map of preloaded resources.
        // Step 3. If key does not exist in preloads, then return false.
        let preload_id = self.preloaded_resources.get(&key)?;

        // Step 4. Let entry be preloads[key].
        let mut preloaded_resources_lock = preloaded_resources.lock();
        let preloads = preloaded_resources_lock.as_mut().unwrap();
        let preload_entry = preloads.get_mut(preload_id)?;

        // Step 5. Let consumerIntegrityMetadata be the result of parsing integrityMetadata.
        let consumer_integrity_metadata =
            csp::parse_subresource_integrity_metadata(&request.integrity_metadata);
        // Step 6. Let preloadIntegrityMetadata be the result of parsing entry's integrity metadata.
        let preload_integrity_metadata =
            csp::parse_subresource_integrity_metadata(&preload_entry.integrity_metadata);
        // Step 7. If none of the following conditions apply:
        if !(
            // consumerIntegrityMetadata is no metadata;
            consumer_integrity_metadata == csp::SubresourceIntegrityMetadata::NoMetadata
            // consumerIntegrityMetadata is equal to preloadIntegrityMetadata; or
            || consumer_integrity_metadata == preload_integrity_metadata
        ) {
            // then return false.
            return None;
        }

        // Step 8. Remove preloads[key].
        // Note: `self.preloads` is an immutable copy of the list of preloads
        //       that is not shared with any other running fetch.
        //       Instead, we defer removing the associated PreloadId->PreloadEntry
        //       map entry until the entire response has been preloaded, since
        //       it stores the Sender which will be used to transmit the complete
        //       response.

        // Step 10. Otherwise, call onResponseAvailable with entry's response.
        let result = if let Some(response) = preload_entry.response.as_ref() {
            Some(PreloadResponseCandidate::Response(
                Box::new(response.clone()),
                preload_id.clone(),
            ))
        } else {
            // Step 9. If entry's response is null, then set entry's on response available to onResponseAvailable.
            let (sender, receiver) = tokio::sync::oneshot::channel();
            preload_entry.on_response_available = Some(sender);
            // Step 11. Return true.
            Some(PreloadResponseCandidate::Pending(
                receiver,
                preload_id.clone(),
            ))
        };

        result
    }
}
