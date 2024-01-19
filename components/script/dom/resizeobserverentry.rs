/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use dom_struct::dom_struct;
use js::jsval::JSVal;

use crate::dom::bindings::cell::DomRefCell;
use crate::dom::bindings::codegen::Bindings::ResizeObserverEntryBinding::ResizeObserverEntryMethods;
use crate::dom::bindings::reflector::Reflector;
use crate::dom::bindings::root::{Dom, DomRoot};
use crate::dom::bindings::utils::to_frozen_array;
use crate::dom::domrectreadonly::DOMRectReadOnly;
use crate::dom::element::Element;
use crate::dom::resizeobserversize::ResizeObserverSize;
use crate::script_runtime::JSContext as SafeJSContext;

#[dom_struct]
pub struct ResizeObserverEntry {
    reflector_: Reflector,
    target: Dom<Element>,
    content_rect: Dom<DOMRectReadOnly>,
    border_box_size: Vec<ResizeObserverSize>,
    content_box_size: Vec<ResizeObserverSize>,
    device_pixel_content_box_size: Vec<ResizeObserverSize>,
}

impl ResizeObserverEntryMethods for ResizeObserverEntry {
    fn Target(&self) -> DomRoot<Element> {
        DomRoot::from_ref(&*self.target)
    }
    fn ContentRect(&self) -> DomRoot<DOMRectReadOnly> {
        DomRoot::from_ref(&*self.content_rect)
    }
    fn BorderBoxSize(&self, cx: SafeJSContext) -> JSVal {
        to_frozen_array(&self.border_box_size, cx)
    }
    fn ContentBoxSize(&self, cx: SafeJSContext) -> JSVal {
        to_frozen_array(&self.content_box_size, cx)
    }
    fn DevicePixelContentBoxSize(&self, cx: SafeJSContext) -> JSVal {
        to_frozen_array(&self.device_pixel_content_box_size, cx)
    }
}
