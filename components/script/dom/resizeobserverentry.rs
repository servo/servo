/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use dom_struct::dom_struct;
use js::jsval::JSVal;

use crate::dom::bindings::codegen::Bindings::ResizeObserverEntryBinding::ResizeObserverEntryMethods;
use crate::dom::bindings::reflector::{reflect_dom_object_with_proto, Reflector};
use crate::dom::bindings::root::{Dom, DomRoot};
use crate::dom::bindings::utils::to_frozen_array;
use crate::dom::domrectreadonly::DOMRectReadOnly;
use crate::dom::element::Element;
use crate::dom::resizeobserversize::ResizeObserverSize;
use crate::dom::window::Window;
use crate::script_runtime::JSContext as SafeJSContext;

/// <https://drafts.csswg.org/resize-observer/#resize-observer-entry-interface>
#[dom_struct]
pub struct ResizeObserverEntry {
    reflector_: Reflector,
    /// <https://drafts.csswg.org/resize-observer/#dom-resizeobserverentry-target>
    target: Dom<Element>,
    /// <https://drafts.csswg.org/resize-observer/#dom-resizeobserverentry-contentrect>
    content_rect: Dom<DOMRectReadOnly>,
    /// <https://drafts.csswg.org/resize-observer/#dom-resizeobserverentry-borderboxsize>
    border_box_size: Vec<Dom<ResizeObserverSize>>,
    /// <https://drafts.csswg.org/resize-observer/#dom-resizeobserverentry-contentboxsize>
    content_box_size: Vec<Dom<ResizeObserverSize>>,
    /// <https://drafts.csswg.org/resize-observer/#dom-resizeobserverentry-devicepixelcontentboxsize>
    device_pixel_content_box_size: Vec<Dom<ResizeObserverSize>>,
}

impl ResizeObserverEntry {
    fn new_inherited(
        target: &Element,
        content_rect: &DOMRectReadOnly,
        border_box_size: &[&ResizeObserverSize],
        content_box_size: &[&ResizeObserverSize],
        device_pixel_content_box_size: &[&ResizeObserverSize],
    ) -> ResizeObserverEntry {
        ResizeObserverEntry {
            reflector_: Reflector::new(),
            target: Dom::from_ref(target),
            content_rect: Dom::from_ref(content_rect),
            border_box_size: border_box_size
                .iter()
                .map(|size| Dom::from_ref(*size))
                .collect(),
            content_box_size: content_box_size
                .iter()
                .map(|size| Dom::from_ref(*size))
                .collect(),
            device_pixel_content_box_size: device_pixel_content_box_size
                .iter()
                .map(|size| Dom::from_ref(*size))
                .collect(),
        }
    }

    pub fn new(
        window: &Window,
        target: &Element,
        content_rect: &DOMRectReadOnly,
        border_box_size: &[&ResizeObserverSize],
        content_box_size: &[&ResizeObserverSize],
        device_pixel_content_box_size: &[&ResizeObserverSize],
    ) -> DomRoot<ResizeObserverEntry> {
        let entry = Box::new(ResizeObserverEntry::new_inherited(
            target,
            content_rect,
            border_box_size,
            content_box_size,
            device_pixel_content_box_size,
        ));
        reflect_dom_object_with_proto(entry, window, None)
    }
}

impl ResizeObserverEntryMethods for ResizeObserverEntry {
    /// https://drafts.csswg.org/resize-observer/#dom-resizeobserverentry-target
    fn Target(&self) -> DomRoot<Element> {
        DomRoot::from_ref(&*self.target)
    }

    /// https://drafts.csswg.org/resize-observer/#dom-resizeobserverentry-contentrect
    fn ContentRect(&self) -> DomRoot<DOMRectReadOnly> {
        DomRoot::from_ref(&*self.content_rect)
    }

    /// https://drafts.csswg.org/resize-observer/#dom-resizeobserverentry-borderboxsize
    fn BorderBoxSize(&self, cx: SafeJSContext) -> JSVal {
        let sizes: Vec<DomRoot<ResizeObserverSize>> = self
            .border_box_size
            .iter()
            .map(|size| DomRoot::from_ref(&**size))
            .collect();
        to_frozen_array(sizes.as_slice(), cx)
    }

    /// https://drafts.csswg.org/resize-observer/#dom-resizeobserverentry-contentboxsize
    fn ContentBoxSize(&self, cx: SafeJSContext) -> JSVal {
        let sizes: Vec<DomRoot<ResizeObserverSize>> = self
            .content_box_size
            .iter()
            .map(|size| DomRoot::from_ref(&**size))
            .collect();
        to_frozen_array(sizes.as_slice(), cx)
    }

    /// https://drafts.csswg.org/resize-observer/#dom-resizeobserverentry-devicepixelcontentboxsize
    fn DevicePixelContentBoxSize(&self, cx: SafeJSContext) -> JSVal {
        let sizes: Vec<DomRoot<ResizeObserverSize>> = self
            .device_pixel_content_box_size
            .iter()
            .map(|size| DomRoot::from_ref(&**size))
            .collect();
        to_frozen_array(sizes.as_slice(), cx)
    }
}
