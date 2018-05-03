/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use dom::bindings::codegen::Bindings::PerformancePaintTimingBinding;
use dom::bindings::reflector::reflect_dom_object;
use dom::bindings::root::DomRoot;
use dom::bindings::str::DOMString;
use dom::globalscope::GlobalScope;
use dom::performanceentry::PerformanceEntry;
use dom_struct::dom_struct;
use metrics::ToMs;
use script_traits::ProgressiveWebMetricType;
use typeholder::TypeHolderTrait;

#[dom_struct]
pub struct PerformancePaintTiming<TH: TypeHolderTrait> {
    entry: PerformanceEntry<TH>,
}

impl<TH: TypeHolderTrait> PerformancePaintTiming<TH> {
    fn new_inherited(metric_type: ProgressiveWebMetricType, start_time: u64) -> PerformancePaintTiming<TH> {
        let name = match metric_type {
            ProgressiveWebMetricType::FirstPaint => DOMString::from("first-paint"),
            ProgressiveWebMetricType::FirstContentfulPaint => DOMString::from("first-contentful-paint"),
            _ => DOMString::from(""),
        };
        PerformancePaintTiming {
            entry: PerformanceEntry::new_inherited(name,
                                                   DOMString::from("paint"),
                                                   start_time.to_ms(),
                                                   0.)
        }
    }

    #[allow(unrooted_must_root)]
    pub fn new(global: &GlobalScope<TH>,
               metric_type: ProgressiveWebMetricType,
               start_time: u64) -> DomRoot<PerformancePaintTiming<TH>> {
        let entry = PerformancePaintTiming::new_inherited(metric_type, start_time);
        reflect_dom_object(Box::new(entry), global, PerformancePaintTimingBinding::Wrap)
    }
}
