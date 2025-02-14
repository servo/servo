/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use dom_struct::dom_struct;
use js::rust::HandleObject;

use super::bindings::codegen::Bindings::IntersectionObserverEntryBinding::{
    IntersectionObserverEntryInit, IntersectionObserverEntryMethods,
};
use super::bindings::num::Finite;
use crate::dom::bindings::reflector::{reflect_dom_object_with_proto, DomGlobal, Reflector};
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
    target: Dom<Element>,
}

impl IntersectionObserverEntry {
    pub(crate) fn new_inherited(init: &IntersectionObserverEntryInit) -> Self {
        Self {
            reflector_: Reflector::new(),
            target: init.target.as_traced(),
        }
    }

    fn new(
        window: &Window,
        proto: Option<HandleObject>,
        init: &IntersectionObserverEntryInit,
        can_gc: CanGc,
    ) -> DomRoot<Self> {
        let observer = Box::new(Self::new_inherited(init));
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
        Finite::new(0.).unwrap()
    }

    /// > For a same-origin-domain target, this will be the root intersection rectangle.
    /// > Otherwise, this will be null. Note that if the target is in a different browsing
    /// > context than the intersection root, this will be in a different coordinate system
    /// > than boundingClientRect and intersectionRect.
    ///
    /// <https://w3c.github.io/IntersectionObserver/#dom-intersectionobserverentry-rootbounds>
    fn GetRootBounds(&self) -> Option<DomRoot<DOMRectReadOnly>> {
        None
    }

    /// > A DOMRectReadOnly obtained by getting the bounding box for target.
    ///
    /// <https://w3c.github.io/IntersectionObserver/#dom-intersectionobserverentry-boundingclientrect>
    fn BoundingClientRect(&self) -> DomRoot<DOMRectReadOnly> {
        DOMRectReadOnly::new(&self.global(), None, 0., 0., 0., 0., CanGc::note())
    }

    /// > boundingClientRect, intersected by each of target's ancestors' clip rects (up to
    /// > but not including root), intersected with the root intersection rectangle. This
    /// > value represents the portion of target that intersects with the root intersection
    /// > rectangle.
    ///
    /// <https://w3c.github.io/IntersectionObserver/#dom-intersectionobserverentry-intersectionrect>
    fn IntersectionRect(&self) -> DomRoot<DOMRectReadOnly> {
        DOMRectReadOnly::new(&self.global(), None, 0., 0., 0., 0., CanGc::note())
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
        false
    }

    /// > Contains the result of running the visibility algorithm on target.
    ///
    /// <https://w3c.github.io/IntersectionObserver/#dom-intersectionobserverentry-isvisible>
    fn IsVisible(&self) -> bool {
        false
    }

    /// > If the boundingClientRect has non-zero area, this will be the ratio of
    /// > intersectionRect area to boundingClientRect area. Otherwise, this will be 1 if the
    /// > isIntersecting is true, and 0 if not.
    ///
    /// <https://w3c.github.io/IntersectionObserver/#dom-intersectionobserverentry-intersectionratio>
    fn IntersectionRatio(&self) -> Finite<f64> {
        Finite::new(0.).unwrap()
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
        Self::new(window, proto, init, can_gc)
    }
}
