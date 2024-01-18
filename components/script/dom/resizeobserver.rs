/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use std::rc::Rc;

use dom_struct::dom_struct;
use js::rust::HandleObject;

use crate::dom::bindings::cell::DomRefCell;
use crate::dom::bindings::codegen::Bindings::ResizeObserverBinding::{
    ResizeObserverCallback, ResizeObserverMethods, ResizeObserverOptions,
};
use crate::dom::bindings::codegen::Bindings::WindowBinding::WindowMethods;
use crate::dom::bindings::reflector::{reflect_dom_object_with_proto, Reflector};
use crate::dom::bindings::root::DomRoot;
use crate::dom::element::Element;
use crate::dom::window::Window;

/// https://drafts.csswg.org/resize-observer/#resize-observer-slots
#[dom_struct]
pub struct ResizeObserver {
    reflector_: Reflector,
    /// https://drafts.csswg.org/resize-observer/#dom-resizeobserver-callback-slot
    #[ignore_malloc_size_of = "Rc are hard"]
    callback: Rc<ResizeObserverCallback>,
    /// https://drafts.csswg.org/resize-observer/#dom-resizeobserver-observationtargets-slot
    observation_targets: DomRefCell<Vec<ResizeObservation>>,
    /// https://drafts.csswg.org/resize-observer/#dom-resizeobserver-activetargets-slot
    active_targets: DomRefCell<Vec<ResizeObservation>>,
    /// https://drafts.csswg.org/resize-observer/#dom-resizeobserver-skippedtargets-slot
    skipped_targets: DomRefCell<Vec<ResizeObservation>>,
}

impl ResizeObserver {
    pub fn new_inherited(callback: Rc<ResizeObserverCallback>) -> ResizeObserver {
        ResizeObserver {
            reflector_: Reflector::new(),
            callback,
            observation_targets: Default::default(),
            active_targets: Default::default(),
            skipped_targets: Default::default(),
        }
    }

    fn new(
        window: &Window,
        proto: Option<HandleObject>,
        callback: Rc<ResizeObserverCallback>,
    ) -> DomRoot<ResizeObserver> {
        let observer = Box::new(ResizeObserver::new_inherited(callback));
        reflect_dom_object_with_proto(observer, window, proto)
    }

    /// https://drafts.csswg.org/resize-observer/#dom-resizeobserver-resizeobserver
    #[allow(non_snake_case)]
    pub fn Constructor(
        window: &Window,
        proto: Option<HandleObject>,
        callback: Rc<ResizeObserverCallback>,
    ) -> DomRoot<ResizeObserver> {
        let rooted_observer = ResizeObserver::new(window, proto, callback);
        let document = window.Document();
        document.add_resize_observer(&rooted_observer);
        rooted_observer
    }
}

impl ResizeObserverMethods for ResizeObserver {
    /// https://drafts.csswg.org/resize-observer/#dom-resizeobserver-observe
    fn Observe(&self, target: &Element, options: &ResizeObserverOptions) {}

    /// https://drafts.csswg.org/resize-observer/#dom-resizeobserver-unobserve
    fn Unobserve(&self, target: &Element) {}

    /// https://drafts.csswg.org/resize-observer/#dom-resizeobserver-disconnect
    fn Disconnect(&self) {}
}

/// https://drafts.csswg.org/resize-observer/#resizeobservation
#[dom_struct]
struct ResizeObservation {
    reflector_: Reflector,
}
