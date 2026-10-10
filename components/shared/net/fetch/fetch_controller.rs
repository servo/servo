/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use malloc_size_of_derive::MallocSizeOf;

use crate::request::RequestId;
use crate::{CoreResourceThread, cancel_async_fetch};

/// <https://fetch.spec.whatwg.org/#fetch-controller-state>
#[derive(Default, MallocSizeOf, PartialEq)]
pub enum FetchControllerState {
    #[default]
    Ongoing,
    Terminated,
    Aborted,
}

/// Fetch controller object. By default initialized to having a
/// request associated with it, which can be aborted or terminated.
/// Calling `ignore` will sever the relationship with the request,
/// meaning it cannot be cancelled through this controller from that point on.
/// <https://fetch.spec.whatwg.org/#fetch-controller>
#[derive(Default, MallocSizeOf)]
pub struct FetchController {
    request_id: Option<RequestId>,
    core_resource_thread: Option<CoreResourceThread>,
    keep_alive: bool,
    /// <https://fetch.spec.whatwg.org/#fetch-controller-state>
    pub state: FetchControllerState,
}

impl FetchController {
    /// Create a FetchController associated with a request,
    /// and a particular(public vs private) resource thread.
    pub fn new(
        request_id: RequestId,
        keep_alive: bool,
        core_resource_thread: CoreResourceThread,
    ) -> Self {
        Self {
            request_id: Some(request_id),
            core_resource_thread: Some(core_resource_thread),
            keep_alive,
            state: Default::default(),
        }
    }

    pub fn keep_alive(&self) -> bool {
        self.keep_alive
    }

    fn cancel(&mut self) {
        if let Some(request_id) = self.request_id.take() {
            // stop trying to make fetch happen
            // it's not going to happen

            if let Some(ref core_resource_thread) = self.core_resource_thread {
                // No error handling here. Cancellation is a courtesy call,
                // we don't actually care if the other side heard.
                cancel_async_fetch(vec![request_id], core_resource_thread);
            }
        }
    }

    /// Use this if you don't want it to send a cancellation request
    /// on drop (e.g. if the fetch completes)
    pub fn ignore(&mut self) {
        let _ = self.request_id.take();
    }

    /// <https://fetch.spec.whatwg.org/#fetch-controller-abort>
    pub fn abort(&mut self) {
        // Step 1. Set controller’s state to "aborted".
        self.state = FetchControllerState::Aborted;

        self.cancel();
    }

    /// <https://fetch.spec.whatwg.org/#fetch-controller-terminate>
    pub fn terminate(&mut self) {
        // > To terminate a fetch controller controller,
        // > set controller’s state to "terminated".
        self.state = FetchControllerState::Terminated;

        self.cancel();
    }
}
