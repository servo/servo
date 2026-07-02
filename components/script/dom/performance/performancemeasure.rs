/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use dom_struct::dom_struct;
use js::gc::HandleValue;
use js::jsapi::Heap;
use js::jsval::JSVal;
use js::rust::MutableHandleValue;
use script_bindings::reflector::reflect_dom_object;
use servo_base::cross_process_instant::CrossProcessInstant;
use time::Duration;

use crate::dom::bindings::codegen::Bindings::PerformanceMeasureBinding::PerformanceMeasureMethods;
use crate::dom::bindings::root::DomRoot;
use crate::dom::bindings::str::DOMString;
use crate::dom::globalscope::GlobalScope;
use crate::dom::performance::performanceentry::{EntryType, PerformanceEntry};
use crate::script_runtime::{CanGc, JSContext};

#[dom_struct]
pub(crate) struct PerformanceMeasure {
    entry: PerformanceEntry,
    #[ignore_malloc_size_of = "Defined in rust-mozjs"]
    detail: Heap<JSVal>,
}

impl PerformanceMeasure {
    fn new_inherited(
        name: DOMString,
        start_time: CrossProcessInstant,
        duration: Duration,
    ) -> PerformanceMeasure {
        PerformanceMeasure {
            entry: PerformanceEntry::new_inherited(
                name,
                EntryType::Measure,
                Some(start_time),
                duration,
            ),
            detail: Default::default(),
        }
    }

    pub(crate) fn new(
        global: &GlobalScope,
        name: DOMString,
        start_time: CrossProcessInstant,
        duration: Duration,
    ) -> DomRoot<PerformanceMeasure> {
        reflect_dom_object(
            Box::new(PerformanceMeasure::new_inherited(
                name, start_time, duration,
            )),
            global,
            CanGc::deprecated_note(),
        )
    }

    pub(crate) fn set_detail(&self, handle: HandleValue<'_>) {
        self.detail.set(handle.get());
    }
}

impl PerformanceMeasureMethods<crate::DomTypeHolder> for PerformanceMeasure {
    /// <https://w3c.github.io/user-timing/#dom-performancemeasure-detail>
    fn Detail(&self, _cx: JSContext, mut retval: MutableHandleValue) {
        retval.set(self.detail.get())
    }
}
