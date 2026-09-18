/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use dom_struct::dom_struct;
use euclid::Box2D;
use js::context::JSContext;
use script_bindings::reflector::reflect_dom_object_with_cx;
use servo_base::cross_process_instant::CrossProcessInstant;
use time::Duration;
use webrender_api::units::LayoutPixel;

use super::performanceentry::{EntryType, PerformanceEntry};
use crate::dom::bindings::codegen::Bindings::PerformanceBinding::DOMHighResTimeStamp;
use crate::dom::bindings::codegen::Bindings::PerformanceContainerTimingBinding::PerformanceContainerTimingMethods;
use crate::dom::bindings::reflector::DomGlobal;
use crate::dom::bindings::root::{Dom, DomRoot};
use crate::dom::bindings::str::DOMString;
use crate::dom::domrectreadonly::DOMRectReadOnly;
use crate::dom::element::Element;
use crate::dom::globalscope::GlobalScope;

/// <https://wicg.github.io/container-timing/#sec-performance-container-timing>
#[dom_struct]
pub(crate) struct PerformanceContainerTiming {
    entry: PerformanceEntry,
    identifier: DOMString,
    intersection_rect_x: f64,
    intersection_rect_y: f64,
    intersection_rect_width: f64,
    intersection_rect_height: f64,
    size: u64,
    #[no_trace]
    first_render_time: CrossProcessInstant,
    #[no_trace]
    paint_time: CrossProcessInstant,
    /// The descendant whose paint produced this entry.
    last_painted_element: Option<Dom<Element>>,
    /// The element carrying the `containertiming` attribute.
    root_element: Option<Dom<Element>>,
}

impl PerformanceContainerTiming {
    fn new_inherited(
        identifier: DOMString,
        intersection_rect: Box2D<f32, LayoutPixel>,
        size: f32,
        first_render_time: CrossProcessInstant,
        paint_time: CrossProcessInstant,
        last_painted_element: Option<&Element>,
        root_element: Option<&Element>,
    ) -> PerformanceContainerTiming {
        PerformanceContainerTiming {
            entry: PerformanceEntry::new_inherited(
                DOMString::new(),
                EntryType::ContainerTiming,
                Some(paint_time),
                Duration::ZERO,
            ),
            identifier,
            intersection_rect_x: intersection_rect.min.x as f64,
            intersection_rect_y: intersection_rect.min.y as f64,
            intersection_rect_width: intersection_rect.width() as f64,
            intersection_rect_height: intersection_rect.height() as f64,
            size: size.round() as u64,
            first_render_time,
            paint_time,
            last_painted_element: last_painted_element.map(Dom::from_ref),
            root_element: root_element.map(Dom::from_ref),
        }
    }

    #[expect(clippy::too_many_arguments)]
    pub(crate) fn new(
        cx: &mut JSContext,
        global: &GlobalScope,
        identifier: DOMString,
        intersection_rect: Box2D<f32, LayoutPixel>,
        size: f32,
        first_render_time: CrossProcessInstant,
        paint_time: CrossProcessInstant,
        last_painted_element: Option<&Element>,
        root_element: Option<&Element>,
    ) -> DomRoot<PerformanceContainerTiming> {
        reflect_dom_object_with_cx(
            Box::new(PerformanceContainerTiming::new_inherited(
                identifier,
                intersection_rect,
                size,
                first_render_time,
                paint_time,
                last_painted_element,
                root_element,
            )),
            global,
            cx,
        )
    }
}

impl PerformanceContainerTimingMethods<crate::DomTypeHolder> for PerformanceContainerTiming {
    /// <https://wicg.github.io/container-timing/#dom-performancecontainertiming-identifier>
    fn Identifier(&self) -> DOMString {
        self.identifier.clone()
    }

    /// <https://wicg.github.io/container-timing/#dom-performancecontainertiming-intersectionrect>
    fn IntersectionRect(&self, cx: &mut JSContext) -> DomRoot<DOMRectReadOnly> {
        DOMRectReadOnly::new(
            cx,
            &self.global(),
            None,
            self.intersection_rect_x,
            self.intersection_rect_y,
            self.intersection_rect_width,
            self.intersection_rect_height,
        )
    }

    /// <https://wicg.github.io/container-timing/#dom-performancecontainertiming-size>
    fn Size(&self) -> u64 {
        self.size
    }

    /// <https://wicg.github.io/container-timing/#dom-performancecontainertiming-firstrendertime>
    fn FirstRenderTime(&self, cx: &mut JSContext) -> DOMHighResTimeStamp {
        self.global()
            .performance(cx)
            .to_dom_high_res_time_stamp(self.first_render_time)
    }

    /// <https://wicg.github.io/container-timing/#dom-performancecontainertiming-painttime>
    fn PaintTime(&self, cx: &mut JSContext) -> DOMHighResTimeStamp {
        self.global()
            .performance(cx)
            .to_dom_high_res_time_stamp(self.paint_time)
    }

    /// <https://wicg.github.io/container-timing/#dom-performancecontainertiming-lastpaintedelement>
    fn GetLastPaintedElement(&self) -> Option<DomRoot<Element>> {
        self.last_painted_element.as_deref().map(DomRoot::from_ref)
    }

    /// <https://wicg.github.io/container-timing/#dom-performancecontainertiming-rootelement>
    fn GetRootElement(&self) -> Option<DomRoot<Element>> {
        self.root_element.as_deref().map(DomRoot::from_ref)
    }
}
