/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use std::rc::Rc;

use dom_struct::dom_struct;
use js::rust::{HandleObject, MutableHandleValue};

use crate::dom::bindings::cell::DomRefCell;
use super::bindings::codegen::Bindings::IntersectionObserverBinding::{
    IntersectionObserverCallback, IntersectionObserverMethods,
};
use crate::dom::bindings::codegen::UnionTypes::ElementOrDocument;
use crate::dom::bindings::root::Dom;
use super::types::{Element, IntersectionObserverEntry};
use crate::dom::bindings::codegen::Bindings::IntersectionObserverBinding::IntersectionObserverInit;
use crate::dom::bindings::codegen::Bindings::PerformanceBinding::DOMHighResTimeStamp;
use crate::dom::bindings::reflector::{reflect_dom_object_with_proto, Reflector};
use crate::dom::bindings::root::DomRoot;
use crate::dom::bindings::str::DOMString;
use crate::dom::window::Window;
use crate::script_runtime::{CanGc, JSContext};
/// The Intersection Observer interface
///
/// > The IntersectionObserver interface can be used to observe changes in the intersection
/// > of an intersection root and one or more target Elements.
///
/// <https://w3c.github.io/IntersectionObserver/#intersection-observer-interface>
#[dom_struct]
pub(crate) struct IntersectionObserver {
    reflector_: Reflector,

    /// > This callback will be invoked when there are changes to a target’s intersection
    /// > with the intersection root, as per the processing model.
    /// <https://w3c.github.io/IntersectionObserver/#intersection-observer-callback>
    #[ignore_malloc_size_of = "Rc are hard"]
    callback: Rc<IntersectionObserverCallback>,

    /// <https://w3c.github.io/IntersectionObserver/#dom-intersectionobserver-queuedentries-slot>
    queued_entries: DomRefCell<Vec<DomRoot<IntersectionObserverEntry>>>,

    /// <https://w3c.github.io/IntersectionObserver/#dom-intersectionobserver-observationtargets-slot>
    observation_targets: DomRefCell<Vec<Dom<Element>>>,

    // A [[rootMargin]] slot which is a list of four pixel lengths or percentages.
    root_margin: MarginRect,

    // A [[scrollMargin]] slot which is a list of four pixel lengths or percentages.
    scroll_margin: MarginRect,

    // A [[thresholds]] slot which is initialized by IntersectionObserver(callback, options).
    thresholds: Vec<f64>,

    // A [[delay]] slot which is initialized by IntersectionObserver(callback, options).
    delay: i32,

    // A [[trackVisibility]] slot which is initialized by IntersectionObserver(callback, options).
    track_visibility: bool,
}

impl IntersectionObserver {
    pub(crate) fn new_inherited(
        callback: Rc<IntersectionObserverCallback>,
        _init: &IntersectionObserverInit,
    ) -> Self {
        Self {
            reflector_: Reflector::new(),
            callback,
            queued_entries: Default::default(),
            observation_targets: Default::default(),
            root_margin: MarginRect::zero(),
            scroll_margin: MarginRect::zero(),
            thresholds: Default::default(),
            delay: Default::default(),
            track_visibility: Default::default(),
        }
    }

    fn new(
        window: &Window,
        proto: Option<HandleObject>,
        callback: Rc<IntersectionObserverCallback>,
        init: &IntersectionObserverInit,
        can_gc: CanGc,
    ) -> DomRoot<Self> {
        let observer = Box::new(Self::new_inherited(callback, init));
        reflect_dom_object_with_proto(observer, window, proto, can_gc)
    }
}

impl IntersectionObserverMethods<crate::DomTypeHolder> for IntersectionObserver {
    /// > The root provided to the IntersectionObserver constructor, or null if none was provided.
    ///
    /// <https://w3c.github.io/IntersectionObserver/#dom-intersectionobserver-root>
    fn GetRoot(&self) -> Option<ElementOrDocument> {
        None
    }

    /// > Offsets applied to the root intersection rectangle, effectively growing or
    /// > shrinking the box that is used to calculate intersections. These offsets are only
    /// > applied when handling same-origin-domain targets; for cross-origin-domain targets
    /// > they are ignored.
    ///
    /// <https://w3c.github.io/IntersectionObserver/#dom-intersectionobserver-rootmargin>
    fn RootMargin(&self) -> DOMString {
        DOMString::new()
    }

    /// > Offsets are applied to scrollports on the path from intersection root to target,
    /// > effectively growing or shrinking the clip rects used to calculate intersections.
    ///
    /// <https://w3c.github.io/IntersectionObserver/#dom-intersectionobserver-scrollmargin>
    fn ScrollMargin(&self) -> DOMString {
        DOMString::new()
    }

    /// > A list of thresholds, sorted in increasing numeric order, where each threshold
    /// > is a ratio of intersection area to bounding box area of an observed target.
    /// > Notifications for a target are generated when any of the thresholds are crossed
    /// > for that target. If no options.threshold was provided to the IntersectionObserver
    /// > constructor, or the sequence is empty, the value of this attribute will be `[0]`.
    ///
    /// <https://w3c.github.io/IntersectionObserver/#dom-intersectionobserver-thresholds>
    fn Thresholds(&self, _context: JSContext, _retval: MutableHandleValue) {}

    /// > A number indicating the minimum delay in milliseconds between notifications from
    /// > this observer for a given target.
    ///
    /// <https://w3c.github.io/IntersectionObserver/#dom-intersectionobserver-delay>
    fn Delay(&self) -> i32 {
        0
    }

    /// > A boolean indicating whether this IntersectionObserver will track changes in a target’s visibility.
    ///
    /// <https://w3c.github.io/IntersectionObserver/#dom-intersectionobserver-trackvisibility>
    fn TrackVisibility(&self) -> bool {
        false
    }

    /// > Run the observe a target Element algorithm, providing this and target.
    ///
    /// <https://w3c.github.io/IntersectionObserver/#dom-intersectionobserver-observe>
    fn Observe(&self, _target: &Element) {}

    /// > Run the unobserve a target Element algorithm, providing this and target.
    ///
    /// <https://w3c.github.io/IntersectionObserver/#dom-intersectionobserver-unobserve>
    fn Unobserve(&self, _target: &Element) {}

    /// <https://w3c.github.io/IntersectionObserver/#dom-intersectionobserver-disconnect>
    fn Disconnect(&self) {}

    /// <https://w3c.github.io/IntersectionObserver/#dom-intersectionobserver-takerecords>
    fn TakeRecords(&self) -> Vec<DomRoot<IntersectionObserverEntry>> {
        vec![]
    }

    /// <https://w3c.github.io/IntersectionObserver/#dom-intersectionobserver-intersectionobserver>
    fn Constructor(
        window: &Window,
        proto: Option<HandleObject>,
        can_gc: CanGc,
        callback: Rc<IntersectionObserverCallback>,
        init: &IntersectionObserverInit,
    ) -> DomRoot<IntersectionObserver> {
        Self::new(window, proto, callback, init, can_gc)
    }
}

#[derive(JSTraceable, MallocSizeOf)]
struct MarginRect {
    top: LengthOrPercentage,
    right: LengthOrPercentage,
    bottom: LengthOrPercentage,
    left: LengthOrPercentage,
}

impl MarginRect {
    fn zero() -> Self {
        Self {
            top: LengthOrPercentage::zero(),
            right: LengthOrPercentage::zero(),
            bottom: LengthOrPercentage::zero(),
            left: LengthOrPercentage::zero(),
        }
    }
}

#[derive(JSTraceable, MallocSizeOf)]
enum LengthOrPercentage {
    // Length will be converted into pixel size on construction.
    AbsolutePixelLength(f32),
    Percentage(f32),
}

impl LengthOrPercentage {
    const fn zero() -> Self {
        Self::AbsolutePixelLength(0.0)
    }
}
/// <https://w3c.github.io/IntersectionObserver/#intersectionobserverregistration>
#[derive(JSTraceable, MallocSizeOf)]
pub(crate) struct IntersectionObserverRegistration {
    observer: DomRoot<IntersectionObserver>,
    previous_threshold_index: i32,
    previous_is_intersecting: bool,
    last_update_time: DOMHighResTimeStamp,
    previous_is_visible: bool,
}
