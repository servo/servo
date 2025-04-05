/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use base::cross_process_instant::CrossProcessInstant;
use dom_struct::dom_struct;
use script_traits::ProgressiveWebMetricType;
use time::Duration;

use crate::dom::bindings::reflector::reflect_dom_object;
use crate::dom::bindings::root::DomRoot;
use crate::dom::bindings::str::DOMString;
use crate::dom::globalscope::GlobalScope;
use crate::dom::performanceentry::PerformanceEntry;
use crate::script_runtime::CanGc;

#[dom_struct]
pub(crate) struct PerformancePaintTiming {
    entry: PerformanceEntry,
}

impl PerformancePaintTiming {
    fn new_inherited(
        metric_type: ProgressiveWebMetricType,
        start_time: CrossProcessInstant,
    ) -> PerformancePaintTiming {
        let name = match metric_type {
            ProgressiveWebMetricType::FirstPaint => DOMString::from("first-paint"),
            ProgressiveWebMetricType::FirstContentfulPaint => {
                DOMString::from("first-contentful-paint")
            },
            _ => DOMString::from(""),
        };
        PerformancePaintTiming {
            entry: PerformanceEntry::new_inherited(
                name,
                DOMString::from("paint"),
                Some(start_time),
                Duration::ZERO,
            ),
        }
    }

    #[cfg_attr(crown, allow(crown::unrooted_must_root))]
    pub(crate) fn new(
        global: &GlobalScope,
        metric_type: ProgressiveWebMetricType,
        start_time: CrossProcessInstant,
        can_gc: CanGc,
    ) -> DomRoot<PerformancePaintTiming> {
        let entry = PerformancePaintTiming::new_inherited(metric_type, start_time);
        reflect_dom_object(Box::new(entry), global, can_gc)
    }
}
