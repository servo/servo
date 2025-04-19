/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use net_traits::request::Referrer;
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
use crate::dom::bindings::reflector::DomGlobal;
use crate::dom::event::{Event, EventBubbles, EventCancelable};
use crate::dom::eventtarget::EventTarget;
use crate::dom::securitypolicyviolationevent::SecurityPolicyViolationEvent;
use crate::dom::types::GlobalScope;
use crate::script_runtime::CanGc;
use crate::task::TaskOnce;

pub(crate) struct CSPViolationReportTask {
    event_target: Trusted<EventTarget>,
    violation_report: SecurityPolicyViolationReport,
}

#[derive(Debug, Serialize)]
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

    /// <https://w3c.github.io/webappsec-csp/#strip-url-for-use-in-reports>
    fn strip_url_for_reports(&self, mut url: ServoUrl) -> String {
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

    pub fn build(self, global: &GlobalScope) -> SecurityPolicyViolationReport {
        SecurityPolicyViolationReport {
            violated_directive: self.effective_directive.clone(),
            effective_directive: self.effective_directive.clone(),
            document_url: self.strip_url_for_reports(global.get_url()),
            disposition: match self.report_only {
                true => SecurityPolicyViolationEventDisposition::Report,
                false => SecurityPolicyViolationEventDisposition::Enforce,
            },
            // https://w3c.github.io/webappsec-csp/#violation-referrer
            referrer: match global.get_referrer() {
                Referrer::Client(url) => self.strip_url_for_reports(url),
                Referrer::ReferrerUrl(url) => self.strip_url_for_reports(url),
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
        global: &GlobalScope,
        report: SecurityPolicyViolationReport,
    ) -> CSPViolationReportTask {
        CSPViolationReportTask {
            violation_report: report,
            event_target: Trusted::new(global.upcast::<EventTarget>()),
        }
    }

    fn fire_violation_event(self, can_gc: CanGc) {
        let target = self.event_target.root();
        let global = &target.global();
        let event = SecurityPolicyViolationEvent::new(
            global,
            Atom::from("securitypolicyviolation"),
            EventBubbles::Bubbles,
            EventCancelable::Cancelable,
            &self.violation_report.convert(),
            can_gc,
        );

        event.upcast::<Event>().fire(&target, can_gc);
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
        // TODO: Support `report-to` directive that corresponds to 5.5.3.5.
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

fn serialize_disposition<S: serde::Serializer>(
    val: &SecurityPolicyViolationEventDisposition,
    serializer: S,
) -> Result<S::Ok, S::Error> {
    match val {
        SecurityPolicyViolationEventDisposition::Report => serializer.serialize_str("report"),
        SecurityPolicyViolationEventDisposition::Enforce => serializer.serialize_str("enforce"),
    }
}
