/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use std::rc::Rc;

use dom_struct::dom_struct;
use js::rust::HandleObject;

use crate::dom::bindings::codegen::Bindings::ResizeObserverBinding::{
    ResizeObserverCallback, ResizeObserverMethods, ResizeObserverOptions,
};
use crate::dom::bindings::reflector::{reflect_dom_object_with_proto, Reflector};
use crate::dom::bindings::root::DomRoot;
use crate::dom::element::Element;
use crate::dom::window::Window;

/// https://drafts.csswg.org/resize-observer/#resize-observer-slots
#[dom_struct]
pub struct ResizeObserver {
    reflector_: Reflector,
    #[ignore_malloc_size_of = "Rc are hard"]
    callback: Rc<ResizeObserverCallback>,
}

impl ResizeObserver {
    pub fn new_inherited(callback: Rc<ResizeObserverCallback>) -> ResizeObserver {
        ResizeObserver {
            reflector_: Reflector::new(),
            callback,
        }
    }

    fn new(
        window: &Window,
        proto: Option<HandleObject>,
        callback: Rc<ResizeObserverCallback>,
    ) -> DomRoot<ResizeObserver> {
        reflect_dom_object_with_proto(
            Box::new(ResizeObserver::new_inherited(callback)),
            window,
            proto,
        )
    }

    #[allow(non_snake_case)]
    pub fn Constructor(
        window: &Window,
        proto: Option<HandleObject>,
        callback: Rc<ResizeObserverCallback>,
    ) -> DomRoot<ResizeObserver> {
        ResizeObserver::new(window, proto, callback)
    }
}

impl ResizeObserverMethods for ResizeObserver {
    fn Observe(&self, target: &Element, options: &ResizeObserverOptions) {}

    fn Unobserve(&self, target: &Element) {}

    fn Disconnect(&self) {}
}
