/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use std::sync::{Arc, Mutex};

use content_security_policy::Violation;
use net_traits::request::RequestId;
use net_traits::{
    BoxedFetchCallback, FetchMetadata, FetchResponseMsg, NetworkError, ResourceFetchTiming,
    ResourceTimingType,
};
use servo_url::ServoUrl;

use crate::dom::bindings::inheritance::Castable;
use crate::dom::bindings::root::DomRoot;
use crate::dom::globalscope::GlobalScope;
use crate::dom::performance::performanceentry::PerformanceEntry;
use crate::dom::performance::performanceresourcetiming::{
    InitiatorType, PerformanceResourceTiming,
};
use crate::script_runtime::CanGc;
use crate::task_source::SendableTaskSource;

pub(crate) trait ResourceTimingListener {
    fn resource_timing_information(&self) -> (InitiatorType, ServoUrl);
    fn resource_timing_global(&self) -> DomRoot<GlobalScope>;
}

pub(crate) fn submit_timing<T: ResourceTimingListener + FetchResponseListener>(
    listener: &T,
    can_gc: CanGc,
) {
    if listener.resource_timing().timing_type != ResourceTimingType::Resource {
        warn!(
            "Submitting non-resource ({:?}) timing as resource",
            listener.resource_timing().timing_type
        );
        return;
    }

    let (initiator_type, url) = listener.resource_timing_information();
    if initiator_type == InitiatorType::Other {
        warn!("Ignoring InitiatorType::Other resource {:?}", url);
        return;
    }

    submit_timing_data(
        &listener.resource_timing_global(),
        url,
        initiator_type,
        listener.resource_timing(),
        can_gc,
    );
}

pub(crate) fn submit_timing_data(
    global: &GlobalScope,
    url: ServoUrl,
    initiator_type: InitiatorType,
    resource_timing: &ResourceFetchTiming,
    can_gc: CanGc,
) {
    let performance_entry =
        PerformanceResourceTiming::new(global, url, initiator_type, None, resource_timing, can_gc);
    global
        .performance()
        .queue_entry(performance_entry.upcast::<PerformanceEntry>(), can_gc);
}

pub(crate) trait FetchResponseListener: Send + 'static {
    /// A gating mechanism that runs before invoking the listener methods on the target
    /// thread. If the `should_invoke` method returns false, the listener does not receive
    /// the notification.
    fn should_invoke(&self) -> bool {
        true
    }

    fn process_request_body(&mut self, request_id: RequestId);
    fn process_request_eof(&mut self, request_id: RequestId);
    fn process_response(
        &mut self,
        request_id: RequestId,
        metadata: Result<FetchMetadata, NetworkError>,
    );
    fn process_response_chunk(&mut self, request_id: RequestId, chunk: Vec<u8>);
    fn process_response_eof(
        &mut self,
        request_id: RequestId,
        response: Result<ResourceFetchTiming, NetworkError>,
    );
    fn resource_timing(&self) -> &ResourceFetchTiming;
    fn resource_timing_mut(&mut self) -> &mut ResourceFetchTiming;
    fn submit_resource_timing(&mut self);
    fn process_csp_violations(&mut self, request_id: RequestId, violations: Vec<Violation>);
}

/// An off-thread sink for async network event tasks. All such events are forwarded to
/// a target thread, where they are invoked on the provided context object.
pub(crate) struct NetworkListener<Listener: FetchResponseListener> {
    pub(crate) context: Arc<Mutex<Listener>>,
    pub(crate) task_source: SendableTaskSource,
}

impl<Listener: FetchResponseListener> NetworkListener<Listener> {
    pub(crate) fn notify(&mut self, message: FetchResponseMsg) {
        let context = self.context.clone();
        self.task_source.queue(task!(network_listener_response: move || {
            let mut context = context.lock().unwrap();

            let fetch_listener = &mut *context;
            if !fetch_listener.should_invoke() {
                return;
            }

            match message {
                FetchResponseMsg::ProcessRequestBody(request_id) => {
                    fetch_listener.process_request_body(request_id)
                },
                FetchResponseMsg::ProcessRequestEOF(request_id) => {
                    fetch_listener.process_request_eof(request_id)
                },
                FetchResponseMsg::ProcessResponse(request_id, meta) => {
                    fetch_listener.process_response(request_id, meta)
                },
                FetchResponseMsg::ProcessResponseChunk(request_id, data) => {
                    fetch_listener.process_response_chunk(request_id, data.0)
                },
                FetchResponseMsg::ProcessResponseEOF(request_id, data) => {
                    match data {
                        Ok(ref response_resource_timing) => {
                            // update listener with values from response
                            *fetch_listener.resource_timing_mut() = response_resource_timing.clone();
                            fetch_listener
                                .process_response_eof(request_id, Ok(response_resource_timing.clone()));
                            // TODO timing check https://w3c.github.io/resource-timing/#dfn-timing-allow-check

                            fetch_listener.submit_resource_timing();
                        },
                        // TODO Resources for which the fetch was initiated, but was later aborted
                        // (e.g. due to a network error) MAY be included as PerformanceResourceTiming
                        // objects in the Performance Timeline and MUST contain initialized attribute
                        // values for processed substeps of the processing model.
                        Err(error) => fetch_listener.process_response_eof(request_id, Err(error)),
                    }
                },
                FetchResponseMsg::ProcessCspViolations(request_id, violations) => {
                    fetch_listener.process_csp_violations(request_id, violations)
                },
            }
        }));
    }

    pub(crate) fn into_callback(mut self) -> BoxedFetchCallback {
        Box::new(move |response_msg| self.notify(response_msg))
    }
}
