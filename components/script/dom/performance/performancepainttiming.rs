/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use dom_struct::dom_struct;
use js::context::JSContext;
use script_bindings::reflector::reflect_dom_object_with_cx;
use script_traits::ProgressiveWebMetricType;
use servo_base::cross_process_instant::CrossProcessInstant;
use time::Duration;

use super::painttimingmixin::PaintTimingMixin;
use super::performanceentry::{EntryType, PerformanceEntry};
use crate::dom::bindings::codegen::Bindings::PerformanceBinding::DOMHighResTimeStamp;
use crate::dom::bindings::codegen::Bindings::PerformancePaintTimingBinding::PerformancePaintTimingMethods;
use crate::dom::bindings::reflector::DomGlobal;
use crate::dom::bindings::root::DomRoot;
use crate::dom::bindings::str::DOMString;
use crate::dom::globalscope::GlobalScope;

#[dom_struct]
pub(crate) struct PerformancePaintTiming {
    entry: PerformanceEntry,
    paint_timing_mixin: PaintTimingMixin,
}

impl PerformancePaintTiming {
    fn new_inherited(
        metric_type: ProgressiveWebMetricType,
        start_time: CrossProcessInstant,
        paint_timing_mixin: PaintTimingMixin,
    ) -> PerformancePaintTiming {
        let name = match metric_type {
            ProgressiveWebMetricType::FirstPaint => DOMString::from_static("first-paint"),
            ProgressiveWebMetricType::FirstContentfulPaint => {
                DOMString::from_static("first-contentful-paint")
            },
            _ => DOMString::new(),
        };
        PerformancePaintTiming {
            entry: PerformanceEntry::new_inherited(
                name,
                EntryType::Paint,
                Some(start_time),
                Duration::ZERO,
            ),
            paint_timing_mixin,
        }
    }

    #[cfg_attr(crown, expect(crown::unrooted_must_root))]
    pub(crate) fn new(
        cx: &mut JSContext,
        global: &GlobalScope,
        metric_type: ProgressiveWebMetricType,
        start_time: CrossProcessInstant,
    ) -> DomRoot<PerformancePaintTiming> {
        let paint_timing_mixin = PaintTimingMixin::new(Some(start_time));
        let entry =
            PerformancePaintTiming::new_inherited(metric_type, start_time, paint_timing_mixin);
        reflect_dom_object_with_cx(Box::new(entry), global, cx)
    }
}

impl PerformancePaintTimingMethods<crate::DomTypeHolder> for PerformancePaintTiming {
    /// <https://www.w3.org/TR/paint-timing/#dom-painttimingmixin-presentationtime>
    fn GetPresentationTime(&self, cx: &mut JSContext) -> Option<DOMHighResTimeStamp> {
        Some(
            self.global()
                .performance(cx)
                .maybe_to_dom_high_res_time_stamp(self.paint_timing_mixin.presentation_time()),
        )
    }
}
