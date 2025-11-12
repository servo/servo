/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use base::cross_process_instant::CrossProcessInstant;
use dom_struct::dom_struct;
use script_traits::ProgressiveWebMetricType;
use time::Duration;

use super::performanceentry::PerformanceEntry;
use crate::dom::bindings::codegen::Bindings::LargestContentfulPaintBinding::LargestContentfulPaintMethods;
use crate::dom::bindings::codegen::Bindings::PerformanceBinding::DOMHighResTimeStamp;
use crate::dom::bindings::reflector::{DomGlobal, reflect_dom_object};
use crate::dom::bindings::root::DomRoot;
use crate::dom::bindings::str::DOMString;
use crate::dom::element::Element;
use crate::dom::globalscope::GlobalScope;
use crate::script_runtime::CanGc;

#[dom_struct]
pub(crate) struct LargestContentfulPaint {
    entry: PerformanceEntry,
    #[no_trace]
    load_time: CrossProcessInstant,
    #[no_trace]
    render_time: CrossProcessInstant,
    size: usize,
    element: Option<DomRoot<Element>>,
}

impl LargestContentfulPaint {
    pub(crate) fn new_inherited(
        metric_type: ProgressiveWebMetricType,
        render_time: CrossProcessInstant,
    ) -> LargestContentfulPaint {
        LargestContentfulPaint {
            entry: PerformanceEntry::new_inherited(
                DOMString::from(""),
                DOMString::from("largest-contentful-paint"),
                Some(render_time),
                Duration::ZERO,
            ),
            load_time: CrossProcessInstant::epoch(),
            render_time,
            size: metric_type.area(),
            element: None,
        }
    }

    #[cfg_attr(crown, allow(crown::unrooted_must_root))]
    pub(crate) fn new(
        global: &GlobalScope,
        metric_type: ProgressiveWebMetricType,
        render_time: CrossProcessInstant,
        can_gc: CanGc,
    ) -> DomRoot<LargestContentfulPaint> {
        let entry = LargestContentfulPaint::new_inherited(metric_type, render_time);
        reflect_dom_object(Box::new(entry), global, can_gc)
    }
}

impl LargestContentfulPaintMethods<crate::DomTypeHolder> for LargestContentfulPaint {
    /// <https://www.w3.org/TR/largest-contentful-paint/#dom-largestcontentfulpaint-loadtime>
    fn LoadTime(&self) -> DOMHighResTimeStamp {
        self.global()
            .performance()
            .to_dom_high_res_time_stamp(self.load_time)
    }

    /// <https://www.w3.org/TR/largest-contentful-paint/#dom-largestcontentfulpaint-rendertime>
    fn RenderTime(&self) -> DOMHighResTimeStamp {
        self.global()
            .performance()
            .to_dom_high_res_time_stamp(self.render_time)
    }

    /// <https://www.w3.org/TR/largest-contentful-paint/#dom-largestcontentfulpaint-size>
    fn Size(&self) -> u32 {
        self.size as u32
    }

    /// <https://www.w3.org/TR/largest-contentful-paint/#dom-largestcontentfulpaint-element>
    fn GetElement(&self) -> Option<DomRoot<Element>> {
        self.element.clone()
    }
}
