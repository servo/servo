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
use script_traits::PaintMetricType;

#[dom_struct]
pub struct PerformancePaintTiming {
    entry: PerformanceEntry,
}

impl PerformancePaintTiming {
    fn new_inherited(metric_type: PaintMetricType, start_time: f64)
        -> PerformancePaintTiming {
        let name = match metric_type {
            PaintMetricType::FirstPaint => DOMString::from("first-paint"),
            PaintMetricType::FirstContentfulPaint => DOMString::from("first-contentful-paint"),
        };
        PerformancePaintTiming {
            entry: PerformanceEntry::new_inherited(name,
                                                   DOMString::from("paint"),
                                                   start_time,
                                                   0.)
        }
    }

    #[allow(unrooted_must_root)]
    pub fn new(global: &GlobalScope,
               metric_type: PaintMetricType,
               start_time: f64) -> DomRoot<PerformancePaintTiming> {
        let entry = PerformancePaintTiming::new_inherited(metric_type, start_time);
        reflect_dom_object(Box::new(entry), global, PerformancePaintTimingBinding::Wrap)
    }
}
