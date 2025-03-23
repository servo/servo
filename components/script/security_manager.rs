/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use js::jsapi::RuntimeCode;
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

pub(crate) struct CSPViolationReporter {
    sample: Option<String>,
    filename: String,
    report_only: bool,
    runtime_code: RuntimeCode,
    line_number: u32,
    column_number: u32,
    target: Trusted<EventTarget>,
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

impl CSPViolationReporter {
    pub(crate) fn new(
        global: &GlobalScope,
        sample: Option<String>,
        report_only: bool,
        runtime_code: RuntimeCode,
        filename: String,
        line_number: u32,
        column_number: u32,
    ) -> CSPViolationReporter {
        CSPViolationReporter {
            sample,
            filename,
            report_only,
            runtime_code,
            line_number,
            column_number,
            target: Trusted::new(global.upcast::<EventTarget>()),
        }
    }

    fn get_report(&self, global: &GlobalScope) -> SecurityPolicyViolationReport {
        SecurityPolicyViolationReport {
            sample: self.sample.clone(),
            disposition: match self.report_only {
                true => SecurityPolicyViolationEventDisposition::Report,
                false => SecurityPolicyViolationEventDisposition::Enforce,
            },
            // https://w3c.github.io/webappsec-csp/#violation-resource
            blocked_url: match self.runtime_code {
                RuntimeCode::JS => "eval".to_owned(),
                RuntimeCode::WASM => "wasm-eval".to_owned(),
            },
            // https://w3c.github.io/webappsec-csp/#violation-referrer
            referrer: match global.get_referrer() {
                Referrer::Client(url) => self.strip_url_for_reports(url),
                Referrer::ReferrerUrl(url) => self.strip_url_for_reports(url),
                _ => "".to_owned(),
            },
            status_code: global.status_code().unwrap_or(200),
            document_url: self.strip_url_for_reports(global.get_url()),
            source_file: self.filename.clone(),
            violated_directive: "script-src".to_owned(),
            effective_directive: "script-src".to_owned(),
            line_number: self.line_number,
            column_number: self.column_number,
            original_policy: String::default(),
        }
    }

    fn fire_violation_event(&self, can_gc: CanGc) {
        let target = self.target.root();
        let global = &target.global();
        let report = self.get_report(global);

        let event = SecurityPolicyViolationEvent::new(
            global,
            Atom::from("securitypolicyviolation"),
            EventBubbles::Bubbles,
            EventCancelable::Cancelable,
            &report.convert(),
            can_gc,
        );

        event.upcast::<Event>().fire(&target, can_gc);
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
}

/// Corresponds to the operation in 5.5 Report Violation
/// <https://w3c.github.io/webappsec-csp/#report-violation>
/// > Queue a task to run the following steps:
impl TaskOnce for CSPViolationReporter {
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
