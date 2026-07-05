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
use crate::dom::bindings::codegen::Bindings::LargestContentfulPaintBinding::LargestContentfulPaintMethods;
use crate::dom::bindings::codegen::Bindings::PerformanceBinding::DOMHighResTimeStamp;
use crate::dom::bindings::reflector::DomGlobal;
use crate::dom::bindings::root::{Dom, DomRoot};
use crate::dom::bindings::str::DOMString;
use crate::dom::element::Element;
use crate::dom::globalscope::GlobalScope;

#[dom_struct]
pub(crate) struct LargestContentfulPaint {
    entry: PerformanceEntry,
    #[no_trace]
    load_time: CrossProcessInstant,
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
    ) -> LargestContentfulPaint {
        LargestContentfulPaint {
            entry: PerformanceEntry::new_inherited(
                DOMString::from(""),
                EntryType::LargestContentfulPaint,
                Some(render_time),
                Duration::ZERO,
            ),
            load_time: CrossProcessInstant::epoch(),
            render_time,
            size,
            url: url.map(|u| DOMString::from(u.as_str())).unwrap_or_default(),
            element: None,
        }
    }

    pub(crate) fn new(
        cx: &mut JSContext,
        global: &GlobalScope,
        render_time: CrossProcessInstant,
        size: usize,
        url: Option<ServoUrl>,
    ) -> DomRoot<LargestContentfulPaint> {
        reflect_dom_object_with_cx(
            Box::new(LargestContentfulPaint::new_inherited(
                render_time,
                size,
                url,
            )),
            global,
            cx,
        )
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

    /// <https://www.w3.org/TR/largest-contentful-paint/#dom-largestcontentfulpaint-url>
    fn Url(&self) -> DOMString {
        self.url.clone()
    }

    /// <https://www.w3.org/TR/largest-contentful-paint/#dom-largestcontentfulpaint-element>
    fn GetElement(&self) -> Option<DomRoot<Element>> {
        self.element.as_ref().map(|element| element.as_rooted())
    }
}
