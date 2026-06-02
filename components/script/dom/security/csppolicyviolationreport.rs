/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use net_traits::request::Referrer;
use serde::Serialize;
use servo_url::ServoUrl;

use crate::conversions::Convert;
use crate::dom::bindings::codegen::Bindings::CSPViolationReportBodyBinding::CSPViolationReportBody;
use crate::dom::bindings::codegen::Bindings::EventBinding::EventInit;
use crate::dom::bindings::codegen::Bindings::ReportingObserverBinding::ReportBody;
use crate::dom::bindings::codegen::Bindings::SecurityPolicyViolationEventBinding::{
    SecurityPolicyViolationEventDisposition, SecurityPolicyViolationEventInit,
};
use crate::dom::globalscope::GlobalScope;
use crate::dom::reportingobserver::ReportingObserver;

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
pub(crate) struct CSPReportUriViolationReportBody {
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
pub(crate) struct CSPReportUriViolationReport {
    pub(crate) csp_report: CSPReportUriViolationReportBody,
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

impl Convert<CSPViolationReportBody> for SecurityPolicyViolationReport {
    fn convert(self) -> CSPViolationReportBody {
        CSPViolationReportBody {
            sample: self.sample.map(|s| s.into()),
            blockedURL: Some(self.blocked_url.into()),
            // TODO(37328): Why does /content-security-policy/reporting-api/
            // report-to-directive-allowed-in-meta.https.sub.html expect this to be
            // empty, yet the spec expects us to copy referrer from SecurityPolicyViolationReport
            referrer: Some("".to_owned().into()),
            statusCode: self.status_code,
            documentURL: self.document_url.into(),
            sourceFile: Some(self.source_file.into()),
            effectiveDirective: self.effective_directive.into(),
            lineNumber: Some(self.line_number),
            columnNumber: Some(self.column_number),
            originalPolicy: self.original_policy.into(),
            disposition: self.disposition,
            parent: ReportBody::empty(),
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
                .map(ReportingObserver::strip_url_for_reports)
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
            document_url: ReportingObserver::strip_url_for_reports(global.get_url()),
            disposition: match self.report_only {
                true => SecurityPolicyViolationEventDisposition::Report,
                false => SecurityPolicyViolationEventDisposition::Enforce,
            },
            // https://w3c.github.io/webappsec-csp/#violation-referrer
            referrer: match global.get_referrer() {
                Referrer::Client(url) => ReportingObserver::strip_url_for_reports(url),
                Referrer::ReferrerUrl(url) => ReportingObserver::strip_url_for_reports(url),
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

pub(crate) fn serialize_disposition<S: serde::Serializer>(
    val: &SecurityPolicyViolationEventDisposition,
    serializer: S,
) -> Result<S::Ok, S::Error> {
    match val {
        SecurityPolicyViolationEventDisposition::Report => serializer.serialize_str("report"),
        SecurityPolicyViolationEventDisposition::Enforce => serializer.serialize_str("enforce"),
    }
}
