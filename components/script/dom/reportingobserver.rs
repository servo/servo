/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use std::cell::RefCell;
use std::rc::Rc;
use std::time::{SystemTime, UNIX_EPOCH};

use dom_struct::dom_struct;
use js::rust::HandleObject;
use script_bindings::str::DOMString;
use servo_url::ServoUrl;

use crate::dom::bindings::callback::ExceptionHandling;
use crate::dom::bindings::cell::DomRefCell;
use crate::dom::bindings::codegen::Bindings::CSPViolationReportBodyBinding::CSPViolationReportBody;
use crate::dom::bindings::codegen::Bindings::ReportingObserverBinding::{
    Report, ReportList, ReportingObserverCallback, ReportingObserverMethods,
    ReportingObserverOptions,
};
use crate::dom::bindings::num::Finite;
use crate::dom::bindings::refcounted::Trusted;
use crate::dom::bindings::reflector::{DomGlobal, Reflector, reflect_dom_object_with_proto};
use crate::dom::bindings::root::DomRoot;
use crate::dom::globalscope::GlobalScope;
use crate::script_runtime::CanGc;

#[dom_struct]
pub(crate) struct ReportingObserver {
    reflector_: Reflector,

    #[ignore_malloc_size_of = "Rc has unclear ownership"]
    callback: Rc<ReportingObserverCallback>,
    buffered: RefCell<bool>,
    types: DomRefCell<Vec<DOMString>>,
    report_queue: DomRefCell<Vec<Report>>,
}

impl ReportingObserver {
    fn new_inherited(
        callback: Rc<ReportingObserverCallback>,
        options: &ReportingObserverOptions,
    ) -> Self {
        Self {
            reflector_: Reflector::new(),
            callback,
            buffered: RefCell::new(options.buffered),
            types: DomRefCell::new(options.types.clone().unwrap_or_default()),
            report_queue: Default::default(),
        }
    }

    #[cfg_attr(crown, allow(crown::unrooted_must_root))]
    pub(crate) fn new_with_proto(
        callback: Rc<ReportingObserverCallback>,
        options: &ReportingObserverOptions,
        global: &GlobalScope,
        proto: Option<HandleObject>,
        can_gc: CanGc,
    ) -> DomRoot<Self> {
        reflect_dom_object_with_proto(
            Box::new(Self::new_inherited(callback, options)),
            global,
            proto,
            can_gc,
        )
    }

    fn report_is_visible_to_reporting_observers(report: &Report) -> bool {
        match report.type_.str() {
            // https://w3c.github.io/webappsec-csp/#reporting
            "csp-violation" => true,
            _ => false,
        }
    }

    /// <https://w3c.github.io/reporting/#add-report>
    fn add_report_to_observer(&self, report: &Report) {
        // Step 1. If report’s type is not visible to ReportingObservers, return.
        if !Self::report_is_visible_to_reporting_observers(report) {
            return;
        }
        // Step 2. If observer’s options has a non-empty types member which does not contain report’s type, return.
        let types = self.types.borrow();
        if !types.is_empty() && !types.contains(&report.type_) {
            return;
        }
        // Step 3. Create a new Report r with type initialized to report’s type,
        // url initialized to report’s url, and body initialized to report’s body.
        let report = Report {
            type_: report.type_.clone(),
            url: report.url.clone(),
            body: report.body.clone(),
            destination: report.destination.clone(),
            attempts: report.attempts,
            timestamp: report.timestamp,
        };
        // Step 4. Append r to observer’s report queue.
        self.report_queue.borrow_mut().push(report);
        // Step 5. If the size of observer’s report queue is 1:
        if self.report_queue.borrow().len() == 1 {
            // Step 5.1. Let global be observer’s relevant global object.
            let global = self.global();
            // Step 5.2. Queue a task to § 4.4 Invoke reporting observers with notify list
            // with a copy of global’s registered reporting observer list.
            let observers_global = Trusted::new(&*global);
            global.task_manager().dom_manipulation_task_source().queue(
                task!(notify_reporting_observers: move || {
                    Self::invoke_reporting_observers_with_notify_list(
                        observers_global.root().registered_reporting_observers()
                    );
                }),
            );
        }
    }

    /// <https://w3c.github.io/reporting/#notify-observers>
    pub(crate) fn notify_reporting_observers_on_scope(global: &GlobalScope, report: &Report) {
        // Step 1. For each ReportingObserver observer registered with scope,
        // execute § 4.3 Add report to observer on report and observer.
        for observer in global.registered_reporting_observers().iter() {
            observer.add_report_to_observer(report);
        }
        // Step 2. Append report to scope’s report buffer.
        global.append_report(report.clone());
        // Step 3. Let type be report’s type.
        // TODO(37328)
        // Step 4. If scope’s report buffer now contains more than 100 reports with
        // type equal to type, remove the earliest item with type equal to type in the report buffer.
        // TODO(37328)
    }

    /// <https://w3c.github.io/reporting/#invoke-observers>
    fn invoke_reporting_observers_with_notify_list(notify_list: Vec<DomRoot<ReportingObserver>>) {
        // Step 1. For each ReportingObserver observer in notify list:
        for observer in notify_list.iter() {
            // Step 1.1. If observer’s report queue is empty, then continue.
            if observer.report_queue.borrow().is_empty() {
                continue;
            }
            // Step 1.2. Let reports be a copy of observer’s report queue
            // Step 1.3. Empty observer’s report queue
            let reports = std::mem::take(&mut *observer.report_queue.borrow_mut());
            // Step 1.4. Invoke observer’s callback with « reports, observer » and "report",
            // and with observer as the callback this value.
            let _ = observer.callback.Call_(
                &**observer,
                reports,
                observer,
                ExceptionHandling::Report,
                CanGc::note(),
            );
        }
    }

    /// <https://w3c.github.io/reporting/#generate-a-report>
    fn generate_a_report(
        global: &GlobalScope,
        type_: DOMString,
        url: Option<ServoUrl>,
        body: Option<CSPViolationReportBody>,
        destination: DOMString,
    ) -> Report {
        // Step 2. If url was not provided by the caller, let url be settings’s creation URL.
        let url = url.unwrap_or(global.creation_url().clone());
        // Step 3. Set url’s username to the empty string, and its password to null.
        // Step 4. Set report’s url to the result of executing the URL serializer
        // on url with the exclude fragment flag set.
        let url = Self::strip_url_for_reports(url).into();
        // Step 1. Let report be a new report object with its values initialized as follows:
        // Step 5. Return report.
        Report {
            type_,
            url,
            body,
            destination,
            timestamp: Finite::wrap(
                SystemTime::now()
                    .duration_since(UNIX_EPOCH)
                    .unwrap_or_default()
                    .as_millis() as f64,
            ),
            attempts: 0,
        }
    }

    /// <https://w3c.github.io/reporting/#generate-and-queue-a-report>
    pub(crate) fn generate_and_queue_a_report(
        global: &GlobalScope,
        type_: DOMString,
        body: Option<CSPViolationReportBody>,
        destination: DOMString,
    ) {
        // Step 1. Let settings be context’s relevant settings object.
        // Step 2. Let report be the result of running generate a report with data, type, destination and settings.
        let report = Self::generate_a_report(global, type_, None, body, destination);
        // Step 3. If settings is given, then
        // Step 3.1. Let scope be settings’s global object.
        // Step 3.2. If scope is an object implementing WindowOrWorkerGlobalScope, then
        // execute § 4.2 Notify reporting observers on scope with report with scope and report.
        Self::notify_reporting_observers_on_scope(global, &report);
        // Step 4. Append report to context’s reports.
        global.append_report(report);
    }

    /// <https://w3c.github.io/webappsec-csp/#strip-url-for-use-in-reports>
    pub(crate) fn strip_url_for_reports(mut url: ServoUrl) -> String {
        let scheme = url.scheme();
        // Step 1: If url’s scheme is not an HTTP(S) scheme, then return url’s scheme.
        if scheme != "https" && scheme != "http" {
            return scheme.to_owned();
        }
        // Step 2: Set url’s fragment to the empty string.
        url.set_fragment(None);
        // Step 3: Set url’s username to the empty string.
        let _ = url.set_username("");
        // Step 4: Set url’s password to the empty string.
        let _ = url.set_password(None);
        // Step 5: Return the result of executing the URL serializer on url.
        url.into_string()
    }
}

impl ReportingObserverMethods<crate::DomTypeHolder> for ReportingObserver {
    /// <https://w3c.github.io/reporting/#dom-reportingobserver-reportingobserver>
    fn Constructor(
        global: &GlobalScope,
        proto: Option<HandleObject>,
        can_gc: CanGc,
        callback: Rc<ReportingObserverCallback>,
        options: &ReportingObserverOptions,
    ) -> DomRoot<ReportingObserver> {
        // Step 1. Create a new ReportingObserver object observer.
        // Step 2. Set observer’s callback to callback.
        // Step 3. Set observer’s options to options.
        // Step 4. Return observer.
        ReportingObserver::new_with_proto(callback, options, global, proto, can_gc)
    }

    /// <https://w3c.github.io/reporting/#dom-reportingobserver-observe>
    fn Observe(&self) {
        // Step 1. Let global be the be the relevant global object of this.
        let global = &self.global();
        // Step 2. Append this to the global’s registered reporting observer list.
        global.append_reporting_observer(self);
        // Step 3. If this’s buffered option is false, return.
        if !*self.buffered.borrow() {
            return;
        }
        // Step 4. Set this’s buffered option to false.
        *self.buffered.borrow_mut() = false;
        // Step 5.For each report in global’s report buffer, queue a task to
        // execute § 4.3 Add report to observer with report and this.
        for report in global.buffered_reports() {
            // TODO(37328): Figure out how to put this in a task
            self.add_report_to_observer(&report);
        }
    }

    /// <https://w3c.github.io/reporting/#dom-reportingobserver-disconnect>
    fn Disconnect(&self) {
        // Step 1. If this is not registered, return.
        // Skipped, as this is handled in `remove_reporting_observer`

        // Step 2. Let global be the relevant global object of this.
        let global = &self.global();
        // Step 3. Remove this from global’s registered reporting observer list.
        global.remove_reporting_observer(self);
    }

    /// <https://w3c.github.io/reporting/#dom-reportingobserver-takerecords>
    fn TakeRecords(&self) -> ReportList {
        // Step 1. Let reports be a copy of this’s report queue.
        // Step 2. Empty this’s report queue.
        // Step 3. Return reports.
        std::mem::take(&mut *self.report_queue.borrow_mut())
    }
}
