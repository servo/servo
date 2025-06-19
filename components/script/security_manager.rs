/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use std::sync::{Arc, Mutex};

use content_security_policy as csp;
use headers::{ContentType, HeaderMap, HeaderMapExt};
use net_traits::request::{
    CredentialsMode, RequestBody, RequestId, create_request_body_with_content,
};
use net_traits::{
    FetchMetadata, FetchResponseListener, NetworkError, ResourceFetchTiming, ResourceTimingType,
};
use servo_url::ServoUrl;
use stylo_atoms::Atom;

use crate::conversions::Convert;
use crate::dom::bindings::inheritance::Castable;
use crate::dom::bindings::refcounted::Trusted;
use crate::dom::bindings::root::DomRoot;
use crate::dom::csppolicyviolationreport::{
    CSPReportUriViolationReport, SecurityPolicyViolationReport,
};
use crate::dom::event::{Event, EventBubbles, EventCancelable, EventComposed};
use crate::dom::eventtarget::EventTarget;
use crate::dom::performanceresourcetiming::InitiatorType;
use crate::dom::securitypolicyviolationevent::SecurityPolicyViolationEvent;
use crate::dom::types::GlobalScope;
use crate::fetch::create_a_potential_cors_request;
use crate::network_listener::{PreInvoke, ResourceTimingListener, submit_timing};
use crate::script_runtime::CanGc;
use crate::task::TaskOnce;

pub(crate) struct CSPViolationReportTask {
    global: Trusted<GlobalScope>,
    event_target: Trusted<EventTarget>,
    violation_report: SecurityPolicyViolationReport,
    violation_policy: csp::Policy,
}

impl CSPViolationReportTask {
    pub fn new(
        global: Trusted<GlobalScope>,
        event_target: Trusted<EventTarget>,
        violation_report: SecurityPolicyViolationReport,
        violation_policy: csp::Policy,
    ) -> CSPViolationReportTask {
        CSPViolationReportTask {
            global,
            event_target,
            violation_report,
            violation_policy,
        }
    }

    fn fire_violation_event(&self, can_gc: CanGc) {
        let event = SecurityPolicyViolationEvent::new(
            &self.global.root(),
            Atom::from("securitypolicyviolation"),
            EventBubbles::Bubbles,
            EventCancelable::NotCancelable,
            EventComposed::Composed,
            &self.violation_report.clone().convert(),
            can_gc,
        );

        event
            .upcast::<Event>()
            .fire(&self.event_target.root(), can_gc);
    }

    /// <https://www.w3.org/TR/CSP/#deprecated-serialize-violation>
    fn serialize_violation(&self) -> Option<RequestBody> {
        let report_body = CSPReportUriViolationReport {
            // Steps 1-3.
            csp_report: self.violation_report.clone().into(),
        };
        // Step 4. Return the result of serialize an infra value to JSON bytes given «[ "csp-report" → body ]».
        Some(create_request_body_with_content(
            &serde_json::to_string(&report_body).unwrap_or("".to_owned()),
        ))
    }

    /// Step 3.4 of <https://www.w3.org/TR/CSP/#report-violation>
    fn post_csp_violation_to_report_uri(&self, report_uri_directive: &csp::Directive) {
        let global = self.global.root();
        // Step 3.4.1. If violation’s policy’s directive set contains a directive named
        // "report-to", skip the remaining substeps.
        if self
            .violation_policy
            .contains_a_directive_whose_name_is("report-to")
        {
            return;
        }
        // Step 3.4.2. For each token of directive’s value:
        for token in &report_uri_directive.value {
            // Step 3.4.2.1. Let endpoint be the result of executing the URL parser with token as the input,
            // and violation’s url as the base URL.
            let Ok(endpoint) = ServoUrl::parse_with_base(Some(&global.get_url()), token) else {
                // Step 3.4.2.2. If endpoint is not a valid URL, skip the remaining substeps.
                continue;
            };
            // Step 3.4.2.3. Let request be a new request, initialized as follows:
            let mut headers = HeaderMap::with_capacity(1);
            headers.typed_insert(ContentType::from(
                "application/csp-report".parse::<mime::Mime>().unwrap(),
            ));
            let request_body = self.serialize_violation();
            let request = create_a_potential_cors_request(
                None,
                endpoint.clone(),
                csp::Destination::Report,
                None,
                None,
                global.get_referrer(),
                global.insecure_requests_policy(),
                global.has_trustworthy_ancestor_or_current_origin(),
                global.policy_container(),
            )
            .method(http::Method::POST)
            .body(request_body)
            .origin(global.origin().immutable().clone())
            .credentials_mode(CredentialsMode::CredentialsSameOrigin)
            .headers(headers);
            // Step 3.4.2.4. Fetch request. The result will be ignored.
            global.fetch(
                request,
                Arc::new(Mutex::new(CSPReportUriFetchListener {
                    endpoint,
                    global: Trusted::new(&global),
                    resource_timing: ResourceFetchTiming::new(ResourceTimingType::None),
                })),
                global.task_manager().networking_task_source().into(),
            );
        }
    }
}

/// Corresponds to the operation in 5.5 Report Violation
/// <https://w3c.github.io/webappsec-csp/#report-violation>
/// > Queue a task to run the following steps:
impl TaskOnce for CSPViolationReportTask {
    fn run_once(self) {
        // > If target implements EventTarget, fire an event named securitypolicyviolation
        // > that uses the SecurityPolicyViolationEvent interface
        // > at target with its attributes initialized as follows:
        self.fire_violation_event(CanGc::note());
        // Step 3.4. If violation’s policy’s directive set contains a directive named "report-uri" directive:
        if let Some(report_uri_directive) = self
            .violation_policy
            .directive_set
            .iter()
            .find(|directive| directive.name == "report-uri")
        {
            self.post_csp_violation_to_report_uri(report_uri_directive);
        }
    }
}

struct CSPReportUriFetchListener {
    /// Endpoint URL of this request.
    endpoint: ServoUrl,
    /// Timing data for this resource.
    resource_timing: ResourceFetchTiming,
    /// The global object fetching the report uri violation
    global: Trusted<GlobalScope>,
}

impl FetchResponseListener for CSPReportUriFetchListener {
    fn process_request_body(&mut self, _: RequestId) {}

    fn process_request_eof(&mut self, _: RequestId) {}

    fn process_response(
        &mut self,
        _: RequestId,
        fetch_metadata: Result<FetchMetadata, NetworkError>,
    ) {
        _ = fetch_metadata;
    }

    fn process_response_chunk(&mut self, _: RequestId, chunk: Vec<u8>) {
        _ = chunk;
    }

    fn process_response_eof(
        &mut self,
        _: RequestId,
        response: Result<ResourceFetchTiming, NetworkError>,
    ) {
        _ = response;
    }

    fn resource_timing_mut(&mut self) -> &mut ResourceFetchTiming {
        &mut self.resource_timing
    }

    fn resource_timing(&self) -> &ResourceFetchTiming {
        &self.resource_timing
    }

    fn submit_resource_timing(&mut self) {
        submit_timing(self, CanGc::note())
    }

    fn process_csp_violations(&mut self, _request_id: RequestId, violations: Vec<csp::Violation>) {
        let global = &self.resource_timing_global();
        global.report_csp_violations(violations, None);
    }
}

impl ResourceTimingListener for CSPReportUriFetchListener {
    fn resource_timing_information(&self) -> (InitiatorType, ServoUrl) {
        (InitiatorType::Other, self.endpoint.clone())
    }

    fn resource_timing_global(&self) -> DomRoot<GlobalScope> {
        self.global.root()
    }
}

impl PreInvoke for CSPReportUriFetchListener {
    fn should_invoke(&self) -> bool {
        true
    }
}
