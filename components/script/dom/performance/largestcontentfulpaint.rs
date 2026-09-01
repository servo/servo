/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use dom_struct::dom_struct;
use js::context::JSContext;
use script_bindings::reflector::reflect_dom_object_with_cx;
use servo_base::cross_process_instant::CrossProcessInstant;
use servo_url::ServoUrl;
use time::Duration;

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
}

impl LargestContentfulPaint {
    pub(crate) fn new_inherited(
        render_time: CrossProcessInstant,
        size: usize,
        url: Option<ServoUrl>,
        element: Option<&Element>,
    ) -> LargestContentfulPaint {
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
        }
    }

    pub(crate) fn new(
        cx: &mut JSContext,
        global: &GlobalScope,
        render_time: CrossProcessInstant,
        size: usize,
        url: Option<ServoUrl>,
        element: Option<&Element>,
    ) -> DomRoot<LargestContentfulPaint> {
        reflect_dom_object_with_cx(
            Box::new(LargestContentfulPaint::new_inherited(
                render_time,
                size,
                url,
                element,
            )),
            global,
            cx,
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
}
