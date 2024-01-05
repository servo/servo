/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use dom_struct::dom_struct;
use metrics::ToMs;
use script_traits::ProgressiveWebMetricType;

use crate::dom::bindings::reflector::reflect_dom_object;
use crate::dom::bindings::root::DomRoot;
use crate::dom::bindings::str::DOMString;
use crate::dom::globalscope::GlobalScope;
use crate::dom::performanceentry::PerformanceEntry;

#[dom_struct]
pub struct PerformancePaintTiming {
    entry: PerformanceEntry,
}

impl PerformancePaintTiming {
    fn new_inherited(
        metric_type: ProgressiveWebMetricType,
        start_time: u64,
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
                start_time.to_ms(),
                0.,
            ),
        }
    }

    #[allow(crown::unrooted_must_root)]
    pub fn new(
        global: &GlobalScope,
        metric_type: ProgressiveWebMetricType,
        start_time: u64,
    ) -> DomRoot<PerformancePaintTiming> {
        let entry = PerformancePaintTiming::new_inherited(metric_type, start_time);
        reflect_dom_object(Box::new(entry), global)
    }
}
