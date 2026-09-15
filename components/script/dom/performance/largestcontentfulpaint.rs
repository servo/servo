/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use dom_struct::dom_struct;
use js::context::JSContext;
use paint_api::display_list::PaintTimingInfo;
use script_bindings::reflector::reflect_dom_object;
use servo_base::cross_process_instant::CrossProcessInstant;
use servo_url::ServoUrl;
use time::Duration;

use super::painttimingmixin::PaintTimingMixin;
use super::performanceentry::{EntryType, PerformanceEntry};
use crate::dom::bindings::codegen::Bindings::ElementBinding::ElementMethods;
use crate::dom::bindings::codegen::Bindings::LargestContentfulPaintBinding::LargestContentfulPaintMethods;
use crate::dom::bindings::codegen::Bindings::PerformanceBinding::DOMHighResTimeStamp;
use crate::dom::bindings::inheritance::Castable;
use crate::dom::bindings::reflector::DomGlobal;
use crate::dom::bindings::root::{Dom, DomRoot};
use crate::dom::bindings::str::DOMString;
use crate::dom::element::Element;
use crate::dom::globalscope::GlobalScope;
use crate::dom::node::Node;

#[dom_struct]
pub(crate) struct LargestContentfulPaint {
    entry: PerformanceEntry,
    #[no_trace]
    load_time: Option<CrossProcessInstant>,
    #[no_trace]
    render_time: CrossProcessInstant,
    size: usize,
    url: DOMString,
    element: Option<Dom<Element>>,
    paint_timing_mixin: PaintTimingMixin,
}

impl LargestContentfulPaint {
    pub(crate) fn new_inherited(
        size: usize,
        url: Option<ServoUrl>,
        element: Option<&Element>,
        paint_timing_info: PaintTimingInfo,
    ) -> LargestContentfulPaint {
        // From: <https://www.w3.org/TR/largest-contentful-paint/#sec-largest-contentful-paint-interface>
        //
        // The renderTime attribute must return the default paint timestamp
        // given this’s paint timing info.
        let render_time = paint_timing_info.default_paint_timestamp();
        LargestContentfulPaint {
            entry: PerformanceEntry::new_inherited(
                DOMString::new(),
                EntryType::LargestContentfulPaint,
                Some(render_time),
                Duration::ZERO,
            ),
            load_time: None,
            render_time,
            size,
            url: url.map(|u| DOMString::from(u.as_str())).unwrap_or_default(),
            element: Some(Dom::from_ref(
                element.expect("Element for LCP entry should be non-null"),
            )),
            paint_timing_mixin: PaintTimingMixin::new_inherited(paint_timing_info),
        }
    }

    pub(crate) fn new(
        cx: &mut JSContext,
        global: &GlobalScope,
        size: usize,
        url: Option<ServoUrl>,
        element: Option<&Element>,
        paint_timing_info: PaintTimingInfo,
    ) -> DomRoot<LargestContentfulPaint> {
        reflect_dom_object(
            cx,
            Box::new(LargestContentfulPaint::new_inherited(
                size,
                url,
                element,
                paint_timing_info,
            )),
            global,
        )
    }
}

impl LargestContentfulPaintMethods<crate::DomTypeHolder> for LargestContentfulPaint {
    /// <https://www.w3.org/TR/largest-contentful-paint/#dom-largestcontentfulpaint-loadtime>
    fn LoadTime(&self, cx: &mut JSContext) -> DOMHighResTimeStamp {
        self.global()
            .performance(cx)
            .maybe_to_dom_high_res_time_stamp(self.load_time)
    }

    /// <https://www.w3.org/TR/largest-contentful-paint/#dom-largestcontentfulpaint-rendertime>
    fn RenderTime(&self, cx: &mut JSContext) -> DOMHighResTimeStamp {
        self.global()
            .performance(cx)
            .to_dom_high_res_time_stamp(self.render_time)
    }

    /// <https://www.w3.org/TR/largest-contentful-paint/#dom-largestcontentfulpaint-size>
    fn Size(&self) -> u32 {
        self.size as u32
    }

    /// <https://www.w3.org/TR/largest-contentful-paint/#dom-largestcontentfulpaint-url>
    fn Url(&self) -> DOMString {
        self.url.clone()
    }

    /// <https://www.w3.org/TR/largest-contentful-paint/#dom-largestcontentfulpaint-id>
    fn Id(&self) -> DOMString {
        self.GetElement()
            .map(|element| element.Id())
            .unwrap_or_default()
    }

    /// <https://www.w3.org/TR/largest-contentful-paint/#dom-largestcontentfulpaint-element>
    fn GetElement(&self) -> Option<DomRoot<Element>> {
        self.element
            .as_ref()
            .filter(|element| {
                element
                    .upcast::<Node>()
                    .is_connected_with_browsing_context()
            })
            .map(|element| element.as_rooted())
    }

    /// <https://www.w3.org/TR/paint-timing/#dom-painttimingmixin-presentationtime>
    fn GetPresentationTime(&self, cx: &mut JSContext) -> Option<DOMHighResTimeStamp> {
        Some(
            self.global()
                .performance(cx)
                .maybe_to_dom_high_res_time_stamp(self.paint_timing_mixin.presentation_time()),
        )
    }
}
