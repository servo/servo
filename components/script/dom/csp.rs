/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use std::borrow::Cow;

use constellation_traits::{LoadData, LoadOrigin};
/// Used to determine which inline check to run
pub use content_security_policy::InlineCheckType;
/// Used to report CSP violations in Fetch handlers
pub use content_security_policy::Violation;
use content_security_policy::{
    CheckResult, CspList, Destination, Element as CspElement, Initiator, NavigationCheckType,
    Origin, ParserMetadata, PolicyDisposition, PolicySource, Request, ViolationResource,
};
use http::header::{HeaderMap, HeaderValue, ValueIter};
use hyper_serde::Serde;
use js::rust::describe_scripted_caller;
use log::warn;

use crate::dom::bindings::codegen::Bindings::WindowBinding::WindowMethods;
use crate::dom::bindings::inheritance::Castable;
use crate::dom::bindings::refcounted::Trusted;
use crate::dom::csppolicyviolationreport::CSPViolationReportBuilder;
use crate::dom::element::Element;
use crate::dom::globalscope::GlobalScope;
use crate::dom::node::{Node, NodeTraits};
use crate::dom::window::Window;
use crate::security_manager::CSPViolationReportTask;

pub(crate) trait CspReporting {
    fn is_js_evaluation_allowed(&self, global: &GlobalScope, source: &str) -> bool;
    fn is_wasm_evaluation_allowed(&self, global: &GlobalScope) -> bool;
    fn should_navigation_request_be_blocked(
        &self,
        global: &GlobalScope,
        load_data: &LoadData,
        element: Option<&Element>,
    ) -> bool;
    fn should_elements_inline_type_behavior_be_blocked(
        &self,
        global: &GlobalScope,
        el: &Element,
        type_: InlineCheckType,
        source: &str,
    ) -> bool;
    fn is_trusted_type_policy_creation_allowed(
        &self,
        global: &GlobalScope,
        policy_name: String,
        created_policy_names: Vec<String>,
    ) -> bool;
    fn does_sink_type_require_trusted_types(
        &self,
        sink_group: &str,
        include_report_only_policies: bool,
    ) -> bool;
    fn should_sink_type_mismatch_violation_be_blocked_by_csp(
        &self,
        global: &GlobalScope,
        sink: &str,
        sink_group: &str,
        source: &str,
    ) -> bool;
    fn concatenate(self, new_csp_list: Option<CspList>) -> Option<CspList>;
}

impl CspReporting for Option<CspList> {
    /// <https://www.w3.org/TR/CSP/#can-compile-strings>
    fn is_js_evaluation_allowed(&self, global: &GlobalScope, source: &str) -> bool {
        let Some(csp_list) = self else {
            return true;
        };

        let (is_js_evaluation_allowed, violations) = csp_list.is_js_evaluation_allowed(source);

        global.report_csp_violations(violations, None, None);

        is_js_evaluation_allowed == CheckResult::Allowed
    }

    /// <https://www.w3.org/TR/CSP/#can-compile-wasm-bytes>
    fn is_wasm_evaluation_allowed(&self, global: &GlobalScope) -> bool {
        let Some(csp_list) = self else {
            return true;
        };

        let (is_wasm_evaluation_allowed, violations) = csp_list.is_wasm_evaluation_allowed();

        global.report_csp_violations(violations, None, None);

        is_wasm_evaluation_allowed == CheckResult::Allowed
    }

    /// <https://www.w3.org/TR/CSP/#should-block-navigation-request>
    fn should_navigation_request_be_blocked(
        &self,
        global: &GlobalScope,
        load_data: &LoadData,
        element: Option<&Element>,
    ) -> bool {
        let Some(csp_list) = self else {
            return false;
        };
        let request = Request {
            url: load_data.url.clone().into_url(),
            origin: match &load_data.load_origin {
                LoadOrigin::Script(immutable_origin) => immutable_origin.clone().into_url_origin(),
                _ => Origin::new_opaque(),
            },
            // TODO: populate this field correctly
            redirect_count: 0,
            destination: Destination::None,
            initiator: Initiator::None,
            nonce: "".to_owned(),
            integrity_metadata: "".to_owned(),
            parser_metadata: ParserMetadata::None,
        };
        // TODO: set correct navigation check type for form submission if applicable
        let (result, violations) =
            csp_list.should_navigation_request_be_blocked(&request, NavigationCheckType::Other);

        global.report_csp_violations(violations, element, None);

        result == CheckResult::Blocked
    }

    /// <https://www.w3.org/TR/CSP/#should-block-inline>
    fn should_elements_inline_type_behavior_be_blocked(
        &self,
        global: &GlobalScope,
        el: &Element,
        type_: InlineCheckType,
        source: &str,
    ) -> bool {
        let Some(csp_list) = self else {
            return false;
        };
        let element = CspElement {
            nonce: el.nonce_value_if_nonceable().map(Cow::Owned),
        };
        let (result, violations) =
            csp_list.should_elements_inline_type_behavior_be_blocked(&element, type_, source);

        global.report_csp_violations(violations, Some(el), None);

        result == CheckResult::Blocked
    }

    /// <https://w3c.github.io/trusted-types/dist/spec/#should-block-create-policy>
    fn is_trusted_type_policy_creation_allowed(
        &self,
        global: &GlobalScope,
        policy_name: String,
        created_policy_names: Vec<String>,
    ) -> bool {
        let Some(csp_list) = self else {
            return true;
        };

        let (allowed_by_csp, violations) =
            csp_list.is_trusted_type_policy_creation_allowed(policy_name, created_policy_names);

        global.report_csp_violations(violations, None, None);

        allowed_by_csp == CheckResult::Allowed
    }

    /// <https://w3c.github.io/trusted-types/dist/spec/#abstract-opdef-does-sink-type-require-trusted-types>
    fn does_sink_type_require_trusted_types(
        &self,
        sink_group: &str,
        include_report_only_policies: bool,
    ) -> bool {
        let Some(csp_list) = self else {
            return false;
        };

        csp_list.does_sink_type_require_trusted_types(sink_group, include_report_only_policies)
    }

    /// <https://w3c.github.io/trusted-types/dist/spec/#should-block-sink-type-mismatch>
    fn should_sink_type_mismatch_violation_be_blocked_by_csp(
        &self,
        global: &GlobalScope,
        sink: &str,
        sink_group: &str,
        source: &str,
    ) -> bool {
        let Some(csp_list) = self else {
            return false;
        };

        let (allowed_by_csp, violations) = csp_list
            .should_sink_type_mismatch_violation_be_blocked_by_csp(sink, sink_group, source);

        global.report_csp_violations(violations, None, None);

        allowed_by_csp == CheckResult::Blocked
    }

    fn concatenate(self, new_csp_list: Option<CspList>) -> Option<CspList> {
        let Some(new_csp_list) = new_csp_list else {
            return self;
        };

        match self {
            None => Some(new_csp_list),
            Some(mut old_csp_list) => {
                old_csp_list.append(new_csp_list);
                Some(old_csp_list)
            },
        }
    }
}

pub(crate) struct SourcePosition {
    pub(crate) source_file: String,
    pub(crate) line_number: u32,
    pub(crate) column_number: u32,
}

pub(crate) trait GlobalCspReporting {
    fn report_csp_violations(
        &self,
        violations: Vec<Violation>,
        element: Option<&Element>,
        source_position: Option<SourcePosition>,
    );
}

#[allow(unsafe_code)]
fn compute_scripted_caller_source_position() -> SourcePosition {
    let scripted_caller =
        unsafe { describe_scripted_caller(*GlobalScope::get_cx()) }.unwrap_or_default();

    SourcePosition {
        source_file: scripted_caller.filename,
        line_number: scripted_caller.line,
        column_number: scripted_caller.col + 1,
    }
}

impl GlobalCspReporting for GlobalScope {
    /// <https://www.w3.org/TR/CSP/#report-violation>
    fn report_csp_violations(
        &self,
        violations: Vec<Violation>,
        element: Option<&Element>,
        source_position: Option<SourcePosition>,
    ) {
        if violations.is_empty() {
            return;
        }
        warn!("Reporting CSP violations: {:?}", violations);
        let source_position =
            source_position.unwrap_or_else(compute_scripted_caller_source_position);
        for violation in violations {
            let (sample, resource) = match violation.resource {
                ViolationResource::Inline { sample } => (sample, "inline".to_owned()),
                ViolationResource::Url(url) => (Some(String::new()), url.into()),
                ViolationResource::TrustedTypePolicy { sample } => {
                    (Some(sample), "trusted-types-policy".to_owned())
                },
                ViolationResource::TrustedTypeSink { sample } => {
                    (Some(sample), "trusted-types-sink".to_owned())
                },
                ViolationResource::Eval { sample } => (sample, "eval".to_owned()),
                ViolationResource::WasmEval => (None, "wasm-eval".to_owned()),
            };
            let report = CSPViolationReportBuilder::default()
                .resource(resource)
                .sample(sample)
                .effective_directive(violation.directive.name)
                .original_policy(violation.policy.to_string())
                .report_only(violation.policy.disposition == PolicyDisposition::Report)
                .source_file(source_position.source_file.clone())
                .line_number(source_position.line_number)
                .column_number(source_position.column_number)
                .build(self);
            // Step 1: Let global be violation’s global object.
            // We use `self` as `global`;
            // Step 2: Let target be violation’s element.
            let target = element.and_then(|event_target| {
                // Step 3.1: If target is not null, and global is a Window,
                // and target’s shadow-including root is not global’s associated Document, set target to null.
                if let Some(window) = self.downcast::<Window>() {
                    // If a node is connected, its owner document is always the shadow-including root.
                    // If it isn't connected, then it also doesn't have a corresponding document, hence
                    // it can't be this document.
                    if event_target.upcast::<Node>().owner_document() != window.Document() {
                        return None;
                    }
                }
                Some(event_target)
            });
            let target = match target {
                // Step 3.2: If target is null:
                None => {
                    // Step 3.2.2: If target is a Window, set target to target’s associated Document.
                    if let Some(window) = self.downcast::<Window>() {
                        Trusted::new(window.Document().upcast())
                    } else {
                        // Step 3.2.1: Set target to violation’s global object.
                        Trusted::new(self.upcast())
                    }
                },
                Some(event_target) => Trusted::new(event_target.upcast()),
            };
            // Step 3: Queue a task to run the following steps:
            let task =
                CSPViolationReportTask::new(Trusted::new(self), target, report, violation.policy);
            self.task_manager()
                .dom_manipulation_task_source()
                .queue(task);
        }
    }
}

fn parse_and_potentially_append_to_csp_list(
    old_csp_list: Option<CspList>,
    csp_header_iter: ValueIter<HeaderValue>,
    disposition: PolicyDisposition,
) -> Option<CspList> {
    let mut csp_list = old_csp_list;
    for header in csp_header_iter {
        // This silently ignores the CSP if it contains invalid Unicode.
        // We should probably report an error somewhere.
        let new_csp_list = header
            .to_str()
            .ok()
            .map(|value| CspList::parse(value, PolicySource::Header, disposition));
        csp_list = csp_list.concatenate(new_csp_list);
    }
    csp_list
}

/// <https://www.w3.org/TR/CSP/#parse-response-csp>
pub(crate) fn parse_csp_list_from_metadata(headers: &Option<Serde<HeaderMap>>) -> Option<CspList> {
    let headers = headers.as_ref()?;
    let csp_enforce_list = parse_and_potentially_append_to_csp_list(
        None,
        headers.get_all("content-security-policy").iter(),
        PolicyDisposition::Enforce,
    );

    parse_and_potentially_append_to_csp_list(
        csp_enforce_list,
        headers
            .get_all("content-security-policy-report-only")
            .iter(),
        PolicyDisposition::Report,
    )
}
