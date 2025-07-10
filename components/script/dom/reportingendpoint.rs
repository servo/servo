/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use std::collections::HashMap;
use std::sync::{Arc, Mutex};

use headers::{ContentType, HeaderMapExt};
use http::HeaderMap;
use hyper_serde::Serde;
use malloc_size_of_derive::MallocSizeOf;
use net_traits::request::{
    CredentialsMode, Destination, RequestBody, RequestId, RequestMode,
    create_request_body_with_content,
};
use net_traits::{
    FetchMetadata, FetchResponseListener, NetworkError, ResourceFetchTiming, ResourceTimingType,
};
use script_bindings::str::DOMString;
use serde::Serialize;
use servo_url::{ImmutableOrigin, ServoUrl};

use crate::dom::bindings::codegen::Bindings::CSPViolationReportBodyBinding::CSPViolationReportBody;
use crate::dom::bindings::codegen::Bindings::ReportingObserverBinding::Report;
use crate::dom::bindings::codegen::Bindings::SecurityPolicyViolationEventBinding::SecurityPolicyViolationEventDisposition;
use crate::dom::bindings::refcounted::Trusted;
use crate::dom::bindings::root::DomRoot;
use crate::dom::csp::Violation;
use crate::dom::csppolicyviolationreport::serialize_disposition;
use crate::dom::globalscope::GlobalScope;
use crate::dom::performanceresourcetiming::InitiatorType;
use crate::fetch::create_a_potential_cors_request;
use crate::network_listener::{PreInvoke, ResourceTimingListener, submit_timing};
use crate::script_runtime::CanGc;

/// <https://w3c.github.io/reporting/#endpoint>
#[derive(Clone, Eq, Hash, MallocSizeOf, PartialEq)]
pub(crate) struct ReportingEndpoint {
    /// <https://w3c.github.io/reporting/#dom-endpoint-name>
    name: DOMString,
    /// <https://w3c.github.io/reporting/#dom-endpoint-url>
    url: ServoUrl,
    /// <https://w3c.github.io/reporting/#dom-endpoint-failures>
    failures: u32,
}

impl ReportingEndpoint {
    /// <https://w3c.github.io/reporting/#process-header>
    pub(crate) fn parse_reporting_endpoints_header(
        response_url: &ServoUrl,
        headers: &Option<Serde<HeaderMap>>,
    ) -> Option<Vec<ReportingEndpoint>> {
        let headers = headers.as_ref()?;
        let reporting_headers = headers.get_all("reporting-endpoints");
        // Step 2. Let parsed header be the result of executing get a structured field value
        // given "Reporting-Endpoints" and "dictionary" from response’s header list.
        let mut parsed_header = Vec::new();
        for header in reporting_headers.iter() {
            let Some(header_value) = header.to_str().ok() else {
                continue;
            };
            parsed_header.append(&mut header_value.split(",").map(|s| s.trim()).collect());
        }
        // Step 3. If parsed header is null, abort these steps.
        if parsed_header.is_empty() {
            return None;
        }
        // Step 4. Let endpoints be an empty list.
        let mut endpoints = Vec::new();
        // Step 5. For each name → value_and_parameters of parsed header:
        for header in parsed_header {
            // There could be a '=' in the URL itself (for example query parameters). Therefore, we can't
            // split on '=', but instead look for the first one.
            let Some(split_index) = header.find('=') else {
                continue;
            };
            // Step 5.1. Let endpoint url string be the first element of the tuple value_and_parameters.
            // If endpoint url string is not a string, then continue.
            let (name, endpoint_url_string) = header.split_at(split_index);
            let length = endpoint_url_string.len();
            let endpoint_bytes = endpoint_url_string.as_bytes();
            // Note that the first character is the '=' and we check for the next and last character to be '"'
            if length < 3 || endpoint_bytes[1] != b'"' || endpoint_bytes[length - 1] != b'"' {
                continue;
            }
            // The '="' at the start and '"' at the end removed
            let endpoint_url_value = &endpoint_url_string[2..length - 1];
            // Step 5.2. Let endpoint url be the result of executing the URL parser on endpoint url string,
            // with base URL set to response’s url. If endpoint url is failure, then continue.
            let Ok(endpoint_url) =
                ServoUrl::parse_with_base(Some(response_url), endpoint_url_value)
            else {
                continue;
            };
            // Step 5.3. If endpoint url’s origin is not potentially trustworthy, then continue.
            if !endpoint_url.is_potentially_trustworthy() {
                continue;
            }
            // Step 5.4. Let endpoint be a new endpoint whose properties are set as follows:
            // Step 5.5. Add endpoint to endpoints.
            endpoints.push(ReportingEndpoint {
                name: name.into(),
                url: endpoint_url,
                failures: 0,
            });
        }
        Some(endpoints)
    }
}

pub(crate) trait SendReportsToEndpoints {
    /// <https://w3c.github.io/reporting/#send-reports>
    fn send_reports_to_endpoints(&self, reports: Vec<Report>, endpoints: Vec<ReportingEndpoint>);
    /// <https://w3c.github.io/reporting/#try-delivery>
    fn attempt_to_deliver_reports_to_endpoints(
        &self,
        endpoint: &ServoUrl,
        origin: ImmutableOrigin,
        reports: &[&Report],
    );
    /// <https://w3c.github.io/reporting/#serialize-a-list-of-reports-to-json>
    fn serialize_list_of_reports(reports: &[&Report]) -> Option<RequestBody>;
}

impl SendReportsToEndpoints for GlobalScope {
    fn send_reports_to_endpoints(
        &self,
        mut reports: Vec<Report>,
        endpoints: Vec<ReportingEndpoint>,
    ) {
        // Step 1. Let endpoint map be an empty map of endpoint objects to lists of report objects.
        let mut endpoint_map: HashMap<&ReportingEndpoint, Vec<Report>> = HashMap::new();
        // Step 2. For each report in reports:
        reports.retain(|report| {
            // Step 2.1. If there exists an endpoint (endpoint) in context’s endpoints
            // list whose name is report’s destination:
            if let Some(endpoint) = endpoints.iter().find(|e| e.name == report.destination) {
                // Step 2.1.1. Append report to endpoint map’s list of reports for endpoint.
                endpoint_map
                    .entry(endpoint)
                    .or_default()
                    .push(report.clone());
                true
            } else {
                // Step 2.1.2. Otherwise, remove report from reports.
                false
            }
        });
        // Step 3. For each (endpoint, report list) pair in endpoint map:
        for (endpoint, report_list) in endpoint_map.iter() {
            // Step 3.1. Let origin map be an empty map of origins to lists of report objects.
            let mut origin_map: HashMap<ImmutableOrigin, Vec<&Report>> = HashMap::new();
            // Step 3.2. For each report in report list:
            for report in report_list {
                let Ok(url) = ServoUrl::parse(&report.url) else {
                    continue;
                };
                // Step 3.2.1. Let origin be the origin of report’s url.
                let origin = url.origin();
                // Step 3.2.2. Append report to origin map’s list of reports for origin.
                origin_map.entry(origin).or_default().push(report);
            }
            // Step 3.3. For each (origin, per-origin reports) pair in origin map,
            // execute the following steps asynchronously:
            for (origin, origin_report_list) in origin_map.iter() {
                // Step 3.3.1. Let result be the result of executing
                // § 3.5.2 Attempt to deliver reports to endpoint on endpoint, origin, and per-origin reports.
                self.attempt_to_deliver_reports_to_endpoints(
                    &endpoint.url,
                    origin.clone(),
                    origin_report_list,
                );
                // Step 3.3.2. If result is "Failure":
                // TODO(37238)
                // Step 3.3.2.1. Increment endpoint’s failures.
                // TODO(37238)
                // Step 3.3.3. If result is "Remove Endpoint":
                // TODO(37238)
                // Step 3.3.3.1 Remove endpoint from context’s endpoints list.
                // TODO(37238)
                // Step 3.3.4. Remove each report from reports.
                // TODO(37238)
            }
        }
    }

    fn attempt_to_deliver_reports_to_endpoints(
        &self,
        endpoint: &ServoUrl,
        origin: ImmutableOrigin,
        reports: &[&Report],
    ) {
        // Step 1. Let body be the result of executing serialize a list of reports to JSON on reports.
        let request_body = Self::serialize_list_of_reports(reports);
        // Step 2. Let request be a new request with the following properties [FETCH]:
        let mut headers = HeaderMap::with_capacity(1);
        headers.typed_insert(ContentType::from(
            "application/reports+json".parse::<mime::Mime>().unwrap(),
        ));
        let request = create_a_potential_cors_request(
            None,
            endpoint.clone(),
            Destination::Report,
            None,
            None,
            self.get_referrer(),
            self.insecure_requests_policy(),
            self.has_trustworthy_ancestor_or_current_origin(),
            self.policy_container(),
        )
        .method(http::Method::POST)
        .body(request_body)
        .origin(origin)
        .mode(RequestMode::CorsMode)
        .credentials_mode(CredentialsMode::CredentialsSameOrigin)
        .unsafe_request(true)
        .headers(headers);
        // Step 3. Queue a task to fetch request.
        self.fetch(
            request,
            Arc::new(Mutex::new(CSPReportEndpointFetchListener {
                endpoint: endpoint.clone(),
                global: Trusted::new(self),
                resource_timing: ResourceFetchTiming::new(ResourceTimingType::None),
            })),
            self.task_manager().networking_task_source().into(),
        );
        // Step 4. Wait for a response (response).
        // TODO(37238)
        // Step 5. If response’s status is an OK status (200-299), return "Success".
        // TODO(37238)
        // Step 6. If response’s status is 410 Gone [RFC9110], return "Remove Endpoint".
        // TODO(37238)
        // Step 7. Return "Failure".
        // TODO(37238)
    }

    fn serialize_list_of_reports(reports: &[&Report]) -> Option<RequestBody> {
        // Step 1. Let collection be an empty list.
        // Step 2. For each report in reports:
        let report_body: Vec<SerializedReport> = reports
            .iter()
            // Step 2.1. Let data be a map with the following key/value pairs:
            .map(|r| SerializedReport {
                // TODO(37238)
                age: 0,
                type_: r.type_.to_string(),
                url: r.url.to_string(),
                user_agent: "".to_owned(),
                body: r.body.clone().map(|b| b.into()),
            })
            // Step 2.2. Increment report’s attempts.
            // TODO(37238)
            // Step 2.3. Append data to collection.
            .collect();
        // Step 3. Return the byte sequence resulting from executing serialize an
        // Infra value to JSON bytes on collection.
        Some(create_request_body_with_content(
            &serde_json::to_string(&report_body).unwrap_or("".to_owned()),
        ))
    }
}

#[derive(Serialize)]
struct SerializedReport {
    age: u64,
    #[serde(rename = "type")]
    type_: String,
    url: String,
    user_agent: String,
    body: Option<CSPReportingEndpointBody>,
}

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct CSPReportingEndpointBody {
    sample: Option<String>,
    #[serde(rename = "blockedURL")]
    blocked_url: Option<String>,
    referrer: Option<String>,
    status_code: u16,
    #[serde(rename = "documentURL")]
    document_url: String,
    source_file: Option<String>,
    effective_directive: String,
    line_number: Option<u32>,
    column_number: Option<u32>,
    original_policy: String,
    #[serde(serialize_with = "serialize_disposition")]
    disposition: SecurityPolicyViolationEventDisposition,
}

impl From<CSPViolationReportBody> for CSPReportingEndpointBody {
    fn from(value: CSPViolationReportBody) -> Self {
        CSPReportingEndpointBody {
            sample: value.sample.map(|s| s.to_string()),
            blocked_url: value.blockedURL.map(|s| s.to_string()),
            referrer: value.referrer.map(|s| s.to_string()),
            status_code: value.statusCode,
            document_url: value.documentURL.to_string(),
            source_file: value.sourceFile.map(|s| s.to_string()),
            effective_directive: value.effectiveDirective.to_string(),
            line_number: value.lineNumber,
            column_number: value.columnNumber,
            original_policy: value.originalPolicy.into(),
            disposition: value.disposition,
        }
    }
}

struct CSPReportEndpointFetchListener {
    /// Endpoint URL of this request.
    endpoint: ServoUrl,
    /// Timing data for this resource.
    resource_timing: ResourceFetchTiming,
    /// The global object fetching the report uri violation
    global: Trusted<GlobalScope>,
}

impl FetchResponseListener for CSPReportEndpointFetchListener {
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

    fn process_csp_violations(&mut self, _request_id: RequestId, _violations: Vec<Violation>) {}
}

impl ResourceTimingListener for CSPReportEndpointFetchListener {
    fn resource_timing_information(&self) -> (InitiatorType, ServoUrl) {
        (InitiatorType::Other, self.endpoint.clone())
    }

    fn resource_timing_global(&self) -> DomRoot<GlobalScope> {
        self.global.root()
    }
}

impl PreInvoke for CSPReportEndpointFetchListener {
    fn should_invoke(&self) -> bool {
        true
    }
}
