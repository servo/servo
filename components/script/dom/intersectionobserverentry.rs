/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use std::cell::Cell;

use dom_struct::dom_struct;
use js::rust::HandleObject;

use super::bindings::codegen::Bindings::IntersectionObserverEntryBinding::{
    IntersectionObserverEntryInit, IntersectionObserverEntryMethods,
};
use super::bindings::num::Finite;
use crate::dom::bindings::codegen::Bindings::DOMRectReadOnlyBinding::DOMRectInit;
use crate::dom::bindings::reflector::{Reflector, reflect_dom_object_with_proto};
use crate::dom::bindings::root::{Dom, DomRoot};
use crate::dom::domrectreadonly::DOMRectReadOnly;
use crate::dom::element::Element;
use crate::dom::window::Window;
use crate::script_runtime::CanGc;

/// An individual IntersectionObserver entry.
///
/// <https://w3c.github.io/IntersectionObserver/#intersection-observer-entry>
#[dom_struct]
pub(crate) struct IntersectionObserverEntry {
    reflector_: Reflector,
    // <https://w3c.github.io/IntersectionObserver/#dom-intersectionobserverentry-time>
    time: Cell<Finite<f64>>,
    // <https://w3c.github.io/IntersectionObserver/#dom-intersectionobserverentry-rootbounds>
    root_bounds: Option<Dom<DOMRectReadOnly>>,
    // <https://w3c.github.io/IntersectionObserver/#dom-intersectionobserverentry-boundingclientrect>
    bounding_client_rect: Dom<DOMRectReadOnly>,
    // <https://w3c.github.io/IntersectionObserver/#dom-intersectionobserverentry-intersectionrect>
    intersection_rect: Dom<DOMRectReadOnly>,
    // <https://w3c.github.io/IntersectionObserver/#dom-intersectionobserverentry-isintersecting>
    is_intersecting: Cell<bool>,
    // <https://w3c.github.io/IntersectionObserver/#dom-intersectionobserverentry-isvisible>
    is_visible: Cell<bool>,
    // <https://w3c.github.io/IntersectionObserver/#dom-intersectionobserverentry-intersectionratio>
    intersection_ratio: Cell<Finite<f64>>,
    // <https://w3c.github.io/IntersectionObserver/#dom-intersectionobserverentry-target>
    target: Dom<Element>,
}

impl IntersectionObserverEntry {
    #[allow(clippy::too_many_arguments)]
    fn new_inherited(
        time: Finite<f64>,
        root_bounds: Option<&DOMRectReadOnly>,
        bounding_client_rect: &DOMRectReadOnly,
        intersection_rect: &DOMRectReadOnly,
        is_intersecting: bool,
        is_visible: bool,
        intersection_ratio: Finite<f64>,
        target: &Element,
    ) -> Self {
        Self {
            reflector_: Reflector::new(),
            target: Dom::from_ref(target),
            time: Cell::new(time),
            root_bounds: root_bounds.map(Dom::from_ref),
            bounding_client_rect: Dom::from_ref(bounding_client_rect),
            intersection_rect: Dom::from_ref(intersection_rect),
            is_intersecting: Cell::new(is_intersecting),
            is_visible: Cell::new(is_visible),
            intersection_ratio: Cell::new(intersection_ratio),
        }
    }

    #[allow(clippy::too_many_arguments)]
    pub(crate) fn new(
        window: &Window,
        proto: Option<HandleObject>,
        time: Finite<f64>,
        root_bounds: Option<&DOMRectReadOnly>,
        bounding_client_rect: &DOMRectReadOnly,
        intersection_rect: &DOMRectReadOnly,
        is_intersecting: bool,
        is_visible: bool,
        intersection_ratio: Finite<f64>,
        target: &Element,
        can_gc: CanGc,
    ) -> DomRoot<Self> {
        let observer = Box::new(Self::new_inherited(
            time,
            root_bounds,
            bounding_client_rect,
            intersection_rect,
            is_intersecting,
            is_visible,
            intersection_ratio,
            target,
        ));
        reflect_dom_object_with_proto(observer, window, proto, can_gc)
    }

    fn new_from_dictionary(
        window: &Window,
        proto: Option<HandleObject>,
        init: &IntersectionObserverEntryInit,
        can_gc: CanGc,
    ) -> DomRoot<Self> {
        let domrectreadonly_from_dictionary = |dictionary: &DOMRectInit| {
            DOMRectReadOnly::new_from_dictionary(
                window.as_global_scope(),
                proto,
                dictionary,
                can_gc,
            )
        };
        let observer = Box::new(Self::new_inherited(
            init.time,
            Some(&*domrectreadonly_from_dictionary(&init.rootBounds)),
            &domrectreadonly_from_dictionary(&init.boundingClientRect),
            &domrectreadonly_from_dictionary(&init.intersectionRect),
            init.isIntersecting,
            init.isVisible,
            init.intersectionRatio,
            &init.target,
        ));
        reflect_dom_object_with_proto(observer, window, proto, can_gc)
    }
}

impl IntersectionObserverEntryMethods<crate::DomTypeHolder> for IntersectionObserverEntry {
    /// > The attribute must return a DOMHighResTimeStamp that corresponds to the time the
    /// > intersection was recorded, relative to the time origin of the global object
    /// > associated with the IntersectionObserver instance that generated the notification.
    ///
    /// <https://w3c.github.io/IntersectionObserver/#dom-intersectionobserverentry-time>
    fn Time(&self) -> Finite<f64> {
        self.time.get()
    }

    /// > For a same-origin-domain target, this will be the root intersection rectangle.
    /// > Otherwise, this will be null. Note that if the target is in a different browsing
    /// > context than the intersection root, this will be in a different coordinate system
    /// > than boundingClientRect and intersectionRect.
    ///
    /// <https://w3c.github.io/IntersectionObserver/#dom-intersectionobserverentry-rootbounds>
    fn GetRootBounds(&self) -> Option<DomRoot<DOMRectReadOnly>> {
        self.root_bounds.as_ref().map(|rect| rect.as_rooted())
    }

    /// > A DOMRectReadOnly obtained by getting the bounding box for target.
    ///
    /// <https://w3c.github.io/IntersectionObserver/#dom-intersectionobserverentry-boundingclientrect>
    fn BoundingClientRect(&self) -> DomRoot<DOMRectReadOnly> {
        self.bounding_client_rect.as_rooted()
    }

    /// > boundingClientRect, intersected by each of target's ancestors' clip rects (up to
    /// > but not including root), intersected with the root intersection rectangle. This
    /// > value represents the portion of target that intersects with the root intersection
    /// > rectangle.
    ///
    /// <https://w3c.github.io/IntersectionObserver/#dom-intersectionobserverentry-intersectionrect>
    fn IntersectionRect(&self) -> DomRoot<DOMRectReadOnly> {
        self.intersection_rect.as_rooted()
    }

    /// > True if the target intersects with the root; false otherwise. This flag makes it
    /// > possible to distinguish between an IntersectionObserverEntry signalling the
    /// > transition from intersecting to not-intersecting; and an IntersectionObserverEntry
    /// > signalling a transition from not-intersecting to intersecting with a zero-area
    /// > intersection rect (as will happen with edge-adjacent intersections, or when the
    /// > boundingClientRect has zero area).
    ///
    /// <https://w3c.github.io/IntersectionObserver/#dom-intersectionobserverentry-isintersecting>
    fn IsIntersecting(&self) -> bool {
        self.is_intersecting.get()
    }

    /// > Contains the result of running the visibility algorithm on target.
    ///
    /// <https://w3c.github.io/IntersectionObserver/#dom-intersectionobserverentry-isvisible>
    fn IsVisible(&self) -> bool {
        self.is_visible.get()
    }

    /// > If the boundingClientRect has non-zero area, this will be the ratio of
    /// > intersectionRect area to boundingClientRect area. Otherwise, this will be 1 if the
    /// > isIntersecting is true, and 0 if not.
    ///
    /// <https://w3c.github.io/IntersectionObserver/#dom-intersectionobserverentry-intersectionratio>
    fn IntersectionRatio(&self) -> Finite<f64> {
        self.intersection_ratio.get()
    }

    /// > The Element whose intersection with the intersection root changed.
    ///
    /// <https://w3c.github.io/IntersectionObserver/#dom-intersectionobserverentry-target>
    fn Target(&self) -> DomRoot<Element> {
        self.target.as_rooted()
    }

    /// <https://w3c.github.io/IntersectionObserver/#dom-intersectionobserverentry-intersectionobserverentry>
    fn Constructor(
        window: &Window,
        proto: Option<HandleObject>,
        can_gc: CanGc,
        init: &IntersectionObserverEntryInit,
    ) -> DomRoot<IntersectionObserverEntry> {
        Self::new_from_dictionary(window, proto, init, can_gc)
    }
}
