/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use std::sync::{Arc, Mutex};

use content_security_policy as csp;
use headers::{ContentType, HeaderMap, HeaderMapExt};
use net_traits::request::{
    CredentialsMode, Referrer, RequestBody, RequestId, create_request_body_with_content,
};
use net_traits::{
    FetchMetadata, FetchResponseListener, NetworkError, ResourceFetchTiming, ResourceTimingType,
};
use serde::Serialize;
use servo_url::ServoUrl;
use stylo_atoms::Atom;

use crate::conversions::Convert;
use crate::dom::bindings::codegen::Bindings::EventBinding::EventInit;
use crate::dom::bindings::codegen::Bindings::SecurityPolicyViolationEventBinding::{
    SecurityPolicyViolationEventDisposition, SecurityPolicyViolationEventInit,
};
use crate::dom::bindings::inheritance::Castable;
use crate::dom::bindings::refcounted::Trusted;
use crate::dom::bindings::root::DomRoot;
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

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct SecurityPolicyViolationReport {
    sample: Option<String>,
    #[serde(rename = "blockedURL")]
    blocked_url: String,
    referrer: String,
    status_code: u16,
    #[serde(rename = "documentURL")]
    document_url: String,
    source_file: String,
    violated_directive: String,
    effective_directive: String,
    line_number: u32,
    column_number: u32,
    original_policy: String,
    #[serde(serialize_with = "serialize_disposition")]
    disposition: SecurityPolicyViolationEventDisposition,
}

#[derive(Serialize)]
#[serde(rename_all = "kebab-case")]
struct CSPReportUriViolationReportBody {
    document_uri: String,
    referrer: String,
    blocked_uri: String,
    effective_directive: String,
    violated_directive: String,
    original_policy: String,
    #[serde(serialize_with = "serialize_disposition")]
    disposition: SecurityPolicyViolationEventDisposition,
    status_code: u16,
    script_sample: Option<String>,
    source_file: Option<String>,
    line_number: Option<u32>,
    column_number: Option<u32>,
}

#[derive(Serialize)]
#[serde(rename_all = "kebab-case")]
struct CSPReportUriViolationReport {
    csp_report: CSPReportUriViolationReportBody,
}

#[derive(Default)]
pub(crate) struct CSPViolationReportBuilder {
    pub report_only: bool,
    /// <https://www.w3.org/TR/CSP3/#violation-sample>
    pub sample: Option<String>,
    /// <https://www.w3.org/TR/CSP3/#violation-resource>
    pub resource: String,
    /// <https://www.w3.org/TR/CSP3/#violation-line-number>
    pub line_number: u32,
    /// <https://www.w3.org/TR/CSP3/#violation-column-number>
    pub column_number: u32,
    /// <https://www.w3.org/TR/CSP3/#violation-source-file>
    pub source_file: String,
    /// <https://www.w3.org/TR/CSP3/#violation-effective-directive>
    pub effective_directive: String,
    /// <https://www.w3.org/TR/CSP3/#violation-policy>
    pub original_policy: String,
}

impl CSPViolationReportBuilder {
    pub fn report_only(mut self, report_only: bool) -> CSPViolationReportBuilder {
        self.report_only = report_only;
        self
    }

    /// <https://www.w3.org/TR/CSP3/#violation-sample>
    pub fn sample(mut self, sample: Option<String>) -> CSPViolationReportBuilder {
        self.sample = sample;
        self
    }

    /// <https://www.w3.org/TR/CSP3/#violation-resource>
    pub fn resource(mut self, resource: String) -> CSPViolationReportBuilder {
        self.resource = resource;
        self
    }

    /// <https://www.w3.org/TR/CSP3/#violation-line-number>
    pub fn line_number(mut self, line_number: u32) -> CSPViolationReportBuilder {
        self.line_number = line_number;
        self
    }

    /// <https://www.w3.org/TR/CSP3/#violation-column-number>
    pub fn column_number(mut self, column_number: u32) -> CSPViolationReportBuilder {
        self.column_number = column_number;
        self
    }

    /// <https://www.w3.org/TR/CSP3/#violation-source-file>
    pub fn source_file(mut self, source_file: String) -> CSPViolationReportBuilder {
        self.source_file = source_file;
        self
    }

    /// <https://www.w3.org/TR/CSP3/#violation-effective-directive>
    pub fn effective_directive(mut self, effective_directive: String) -> CSPViolationReportBuilder {
        self.effective_directive = effective_directive;
        self
    }

    /// <https://www.w3.org/TR/CSP3/#violation-policy>
    pub fn original_policy(mut self, original_policy: String) -> CSPViolationReportBuilder {
        self.original_policy = original_policy;
        self
    }

    pub fn build(self, global: &GlobalScope) -> SecurityPolicyViolationReport {
        SecurityPolicyViolationReport {
            violated_directive: self.effective_directive.clone(),
            effective_directive: self.effective_directive.clone(),
            document_url: strip_url_for_reports(global.get_url()),
            disposition: match self.report_only {
                true => SecurityPolicyViolationEventDisposition::Report,
                false => SecurityPolicyViolationEventDisposition::Enforce,
            },
            // https://w3c.github.io/webappsec-csp/#violation-referrer
            referrer: match global.get_referrer() {
                Referrer::Client(url) => strip_url_for_reports(url),
                Referrer::ReferrerUrl(url) => strip_url_for_reports(url),
                _ => "".to_owned(),
            },
            sample: self.sample,
            blocked_url: self.resource,
            source_file: self.source_file,
            original_policy: self.original_policy,
            line_number: self.line_number,
            column_number: self.column_number,
            status_code: global.status_code().unwrap_or(0),
        }
    }
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

impl Convert<SecurityPolicyViolationEventInit> for SecurityPolicyViolationReport {
    fn convert(self) -> SecurityPolicyViolationEventInit {
        SecurityPolicyViolationEventInit {
            sample: self.sample.unwrap_or_default().into(),
            blockedURI: self.blocked_url.into(),
            referrer: self.referrer.into(),
            statusCode: self.status_code,
            documentURI: self.document_url.into(),
            sourceFile: self.source_file.into(),
            violatedDirective: self.violated_directive.into(),
            effectiveDirective: self.effective_directive.into(),
            lineNumber: self.line_number,
            columnNumber: self.column_number,
            originalPolicy: self.original_policy.into(),
            disposition: self.disposition,
            parent: EventInit::empty(),
        }
    }
}

/// <https://www.w3.org/TR/CSP/#deprecated-serialize-violation>
impl From<SecurityPolicyViolationReport> for CSPReportUriViolationReportBody {
    fn from(value: SecurityPolicyViolationReport) -> Self {
        // Step 1. Let body be a map with its keys initialized as follows:
        let mut converted = Self {
            document_uri: value.document_url,
            referrer: value.referrer,
            blocked_uri: value.blocked_url,
            effective_directive: value.effective_directive,
            violated_directive: value.violated_directive,
            original_policy: value.original_policy,
            disposition: value.disposition,
            status_code: value.status_code,
            script_sample: None,
            source_file: None,
            line_number: None,
            column_number: None,
        };

        // Step 2. If violation’s source file is not null:
        if !value.source_file.is_empty() {
            // Step 2.1. Set body["source-file'] to the result of
            // executing § 5.4 Strip URL for use in reports on violation’s source file.
            converted.source_file = ServoUrl::parse(&value.source_file)
                .map(strip_url_for_reports)
                .ok();
            // Step 2.2. Set body["line-number"] to violation’s line number.
            converted.line_number = Some(value.line_number);
            // Step 2.3. Set body["column-number"] to violation’s column number.
            converted.column_number = Some(value.column_number);
        }

        // Step 3. Assert: If body["blocked-uri"] is not "inline", then body["sample"] is the empty string.
        debug_assert!(converted.blocked_uri == "inline" || converted.script_sample.is_none());

        converted
    }
}

/// <https://w3c.github.io/webappsec-csp/#strip-url-for-use-in-reports>
fn strip_url_for_reports(mut url: ServoUrl) -> String {
    let scheme = url.scheme();
    // > Step 1: If url’s scheme is not an HTTP(S) scheme, then return url’s scheme.
    if scheme != "https" && scheme != "http" {
        return scheme.to_owned();
    }
    // > Step 2: Set url’s fragment to the empty string.
    url.set_fragment(None);
    // > Step 3: Set url’s username to the empty string.
    let _ = url.set_username("");
    // > Step 4: Set url’s password to the empty string.
    let _ = url.set_password(None);
    // > Step 5: Return the result of executing the URL serializer on url.
    url.into_string()
}

fn serialize_disposition<S: serde::Serializer>(
    val: &SecurityPolicyViolationEventDisposition,
    serializer: S,
) -> Result<S::Ok, S::Error> {
    match val {
        SecurityPolicyViolationEventDisposition::Report => serializer.serialize_str("report"),
        SecurityPolicyViolationEventDisposition::Enforce => serializer.serialize_str("enforce"),
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
