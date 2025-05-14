/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use std::cell::{Cell, RefCell};
use std::rc::Rc;
use std::time::Duration;

use app_units::Au;
use base::cross_process_instant::CrossProcessInstant;
use cssparser::{Parser, ParserInput};
use dom_struct::dom_struct;
use euclid::default::{Rect, Size2D};
use js::rust::{HandleObject, MutableHandleValue};
use style::context::QuirksMode;
use style::parser::{Parse, ParserContext};
use style::stylesheets::{CssRuleType, Origin};
use style_traits::{ParsingMode, ToCss};
use url::Url;

use crate::dom::bindings::callback::ExceptionHandling;
use crate::dom::bindings::cell::DomRefCell;
use crate::dom::bindings::codegen::Bindings::IntersectionObserverBinding::{
    IntersectionObserverCallback, IntersectionObserverInit, IntersectionObserverMethods,
};
use crate::dom::bindings::codegen::Bindings::WindowBinding::WindowMethods;
use crate::dom::bindings::codegen::UnionTypes::{DoubleOrDoubleSequence, ElementOrDocument};
use crate::dom::bindings::error::{Error, Fallible};
use crate::dom::bindings::inheritance::Castable;
use crate::dom::bindings::num::Finite;
use crate::dom::bindings::reflector::{Reflector, reflect_dom_object_with_proto};
use crate::dom::bindings::root::{Dom, DomRoot};
use crate::dom::bindings::str::DOMString;
use crate::dom::bindings::utils::to_frozen_array;
use crate::dom::document::Document;
use crate::dom::domrectreadonly::DOMRectReadOnly;
use crate::dom::element::Element;
use crate::dom::intersectionobserverentry::IntersectionObserverEntry;
use crate::dom::intersectionobserverrootmargin::IntersectionObserverRootMargin;
use crate::dom::node::{Node, NodeTraits};
use crate::dom::window::Window;
use crate::script_runtime::{CanGc, JSContext};

/// > The intersection root for an IntersectionObserver is the value of its root attribute if the attribute is non-null;
/// > otherwise, it is the top-level browsing context’s document node, referred to as the implicit root.
///
/// <https://w3c.github.io/IntersectionObserver/#intersectionobserver-intersection-root>
pub type IntersectionRoot = Option<ElementOrDocument>;

/// The Intersection Observer interface
///
/// > The IntersectionObserver interface can be used to observe changes in the intersection
/// > of an intersection root and one or more target Elements.
///
/// <https://w3c.github.io/IntersectionObserver/#intersection-observer-interface>
#[dom_struct]
pub(crate) struct IntersectionObserver {
    reflector_: Reflector,

    /// [`Document`] that should process this observer's observation steps.
    /// Following Chrome and Firefox, it is the current document on construction.
    /// <https://github.com/w3c/IntersectionObserver/issues/525>
    owner_doc: Dom<Document>,

    /// > The root provided to the IntersectionObserver constructor, or null if none was provided.
    /// <https://w3c.github.io/IntersectionObserver/#dom-intersectionobserver-root>
    root: IntersectionRoot,

    /// > This callback will be invoked when there are changes to a target’s intersection
    /// > with the intersection root, as per the processing model.
    ///
    /// <https://w3c.github.io/IntersectionObserver/#intersection-observer-callback>
    #[ignore_malloc_size_of = "Rc are hard"]
    callback: Rc<IntersectionObserverCallback>,

    /// <https://w3c.github.io/IntersectionObserver/#dom-intersectionobserver-queuedentries-slot>
    queued_entries: DomRefCell<Vec<Dom<IntersectionObserverEntry>>>,

    /// <https://w3c.github.io/IntersectionObserver/#dom-intersectionobserver-observationtargets-slot>
    observation_targets: DomRefCell<Vec<Dom<Element>>>,

    /// <https://w3c.github.io/IntersectionObserver/#dom-intersectionobserver-rootmargin-slot>
    #[no_trace]
    #[ignore_malloc_size_of = "Defined in style"]
    root_margin: RefCell<IntersectionObserverRootMargin>,

    /// <https://w3c.github.io/IntersectionObserver/#dom-intersectionobserver-scrollmargin-slot>
    #[no_trace]
    #[ignore_malloc_size_of = "Defined in style"]
    scroll_margin: RefCell<IntersectionObserverRootMargin>,

    /// <https://w3c.github.io/IntersectionObserver/#dom-intersectionobserver-thresholds-slot>
    thresholds: RefCell<Vec<Finite<f64>>>,

    /// <https://w3c.github.io/IntersectionObserver/#dom-intersectionobserver-delay-slot>
    delay: Cell<i32>,

    /// <https://w3c.github.io/IntersectionObserver/#dom-intersectionobserver-trackvisibility-slot>
    track_visibility: Cell<bool>,
}

impl IntersectionObserver {
    fn new_inherited(
        window: &Window,
        callback: Rc<IntersectionObserverCallback>,
        root: IntersectionRoot,
        root_margin: IntersectionObserverRootMargin,
        scroll_margin: IntersectionObserverRootMargin,
    ) -> Self {
        Self {
            reflector_: Reflector::new(),
            owner_doc: window.Document().as_traced(),
            root,
            callback,
            queued_entries: Default::default(),
            observation_targets: Default::default(),
            root_margin: RefCell::new(root_margin),
            scroll_margin: RefCell::new(scroll_margin),
            thresholds: Default::default(),
            delay: Default::default(),
            track_visibility: Default::default(),
        }
    }

    /// <https://w3c.github.io/IntersectionObserver/#initialize-new-intersection-observer>
    fn new(
        window: &Window,
        proto: Option<HandleObject>,
        callback: Rc<IntersectionObserverCallback>,
        init: &IntersectionObserverInit,
        can_gc: CanGc,
    ) -> Fallible<DomRoot<Self>> {
        // Step 3.
        // > Attempt to parse a margin from options.rootMargin. If a list is returned,
        // > set this’s internal [[rootMargin]] slot to that. Otherwise, throw a SyntaxError exception.
        let root_margin = if let Ok(margin) = parse_a_margin(init.rootMargin.as_ref()) {
            margin
        } else {
            return Err(Error::Syntax);
        };

        // Step 4.
        // > Attempt to parse a margin from options.scrollMargin. If a list is returned,
        // > set this’s internal [[scrollMargin]] slot to that. Otherwise, throw a SyntaxError exception.
        let scroll_margin = if let Ok(margin) = parse_a_margin(init.scrollMargin.as_ref()) {
            margin
        } else {
            return Err(Error::Syntax);
        };

        // Step 1 and step 2, 3, 4 setter
        // > 1. Let this be a new IntersectionObserver object
        // > 2. Set this’s internal [[callback]] slot to callback.
        // > 3. ... set this’s internal [[rootMargin]] slot to that.
        // > 4. ... set this’s internal [[scrollMargin]] slot to that.
        let observer = reflect_dom_object_with_proto(
            Box::new(Self::new_inherited(
                window,
                callback,
                init.root.clone(),
                root_margin,
                scroll_margin,
            )),
            window,
            proto,
            can_gc,
        );

        // Step 5-13
        observer.init_observer(init)?;

        Ok(observer)
    }

    /// Step 5-13 of <https://w3c.github.io/IntersectionObserver/#initialize-new-intersection-observer>
    fn init_observer(&self, init: &IntersectionObserverInit) -> Fallible<()> {
        // Step 5
        // > Let thresholds be a list equal to options.threshold.
        //
        // Non-sequence value should be converted into Vec.
        // Default value of thresholds is [0].
        let mut thresholds = match &init.threshold {
            Some(DoubleOrDoubleSequence::Double(num)) => vec![*num],
            Some(DoubleOrDoubleSequence::DoubleSequence(sequence)) => sequence.clone(),
            None => vec![Finite::wrap(0.)],
        };

        // Step 6
        // > If any value in thresholds is less than 0.0 or greater than 1.0, throw a RangeError exception.
        for num in &thresholds {
            if **num < 0.0 || **num > 1.0 {
                return Err(Error::Range(
                    "Value in thresholds should not be less than 0.0 or greater than 1.0"
                        .to_owned(),
                ));
            }
        }

        // Step 7
        // > Sort thresholds in ascending order.
        thresholds.sort_by(|lhs, rhs| lhs.partial_cmp(&**rhs).unwrap());

        // Step 8
        // > If thresholds is empty, append 0 to thresholds.
        if thresholds.is_empty() {
            thresholds.push(Finite::wrap(0.));
        }

        // Step 9
        // > The thresholds attribute getter will return this sorted thresholds list.
        //
        // Set this internal [[thresholds]] slot to the sorted thresholds list
        // and getter will return the internal [[thresholds]] slot.
        self.thresholds.replace(thresholds);

        // Step 10
        // > Let delay be the value of options.delay.
        //
        // Default value of delay is 0.
        let mut delay = init.delay.unwrap_or(0);

        // Step 11
        // > If options.trackVisibility is true and delay is less than 100, set delay to 100.
        //
        // In Chromium, the minimum delay required is 100 milliseconds for observation that consider trackVisibilty.
        // Currently, visibility is not implemented.
        if init.trackVisibility {
            delay = delay.max(100);
        }

        // Step 12
        // > Set this’s internal [[delay]] slot to options.delay to delay.
        self.delay.set(delay);

        // Step 13
        // > Set this’s internal [[trackVisibility]] slot to options.trackVisibility.
        self.track_visibility.set(init.trackVisibility);

        Ok(())
    }

    /// <https://w3c.github.io/IntersectionObserver/#intersectionobserver-implicit-root>
    fn root_is_implicit_root(&self) -> bool {
        self.root.is_none()
    }

    /// Return unwrapped root if it was an element, None if otherwise.
    fn maybe_element_root(&self) -> Option<&Element> {
        match &self.root {
            Some(ElementOrDocument::Element(element)) => Some(element),
            _ => None,
        }
    }

    /// <https://w3c.github.io/IntersectionObserver/#observe-target-element>
    fn observe_target_element(&self, target: &Element) {
        // Step 1
        // > If target is in observer’s internal [[ObservationTargets]] slot, return.
        let is_present = self
            .observation_targets
            .borrow()
            .iter()
            .any(|element| &**element == target);
        if is_present {
            return;
        }

        // Step 2
        // > Let intersectionObserverRegistration be an IntersectionObserverRegistration record with
        // > an observer property set to observer, a previousThresholdIndex property set to -1,
        // > a previousIsIntersecting property set to false, and a previousIsVisible property set to false.
        // Step 3
        // > Append intersectionObserverRegistration to target’s internal [[RegisteredIntersectionObservers]] slot.
        target.add_initial_intersection_observer_registration(self);

        if self.observation_targets.borrow().is_empty() {
            self.connect_to_owner_unchecked();
        }

        // Step 4
        // > Add target to observer’s internal [[ObservationTargets]] slot.
        self.observation_targets
            .borrow_mut()
            .push(Dom::from_ref(target));
    }

    /// <https://w3c.github.io/IntersectionObserver/#unobserve-target-element>
    fn unobserve_target_element(&self, target: &Element) {
        // Step 1
        // > Remove the IntersectionObserverRegistration record whose observer property is equal to
        // > this from target’s internal [[RegisteredIntersectionObservers]] slot, if present.
        target
            .registered_intersection_observers_mut()
            .retain(|registration| &*registration.observer != self);

        // Step 2
        // > Remove target from this’s internal [[ObservationTargets]] slot, if present
        self.observation_targets
            .borrow_mut()
            .retain(|element| &**element != target);

        // Should disconnect from owner if it is not observing anything.
        if self.observation_targets.borrow().is_empty() {
            self.disconnect_from_owner_unchecked();
        }
    }

    /// <https://w3c.github.io/IntersectionObserver/#queue-an-intersectionobserverentry>
    #[allow(clippy::too_many_arguments)]
    fn queue_an_intersectionobserverentry(
        &self,
        document: &Document,
        time: CrossProcessInstant,
        root_bounds: Rect<Au>,
        bounding_client_rect: Rect<Au>,
        intersection_rect: Rect<Au>,
        is_intersecting: bool,
        is_visible: bool,
        intersection_ratio: f64,
        target: &Element,
        can_gc: CanGc,
    ) {
        let rect_to_domrectreadonly = |rect: Rect<Au>| {
            DOMRectReadOnly::new(
                self.owner_doc.window().as_global_scope(),
                None,
                rect.origin.x.to_f64_px(),
                rect.origin.y.to_f64_px(),
                rect.size.width.to_f64_px(),
                rect.size.height.to_f64_px(),
                can_gc,
            )
        };

        let root_bounds = rect_to_domrectreadonly(root_bounds);
        let bounding_client_rect = rect_to_domrectreadonly(bounding_client_rect);
        let intersection_rect = rect_to_domrectreadonly(intersection_rect);

        // Step 1-2
        // > 1. Construct an IntersectionObserverEntry, passing in time, rootBounds,
        // >    boundingClientRect, intersectionRect, isIntersecting, and target.
        // > 2. Append it to observer’s internal [[QueuedEntries]] slot.
        self.queued_entries.borrow_mut().push(
            IntersectionObserverEntry::new(
                self.owner_doc.window(),
                None,
                document
                    .owner_global()
                    .performance()
                    .to_dom_high_res_time_stamp(time),
                Some(&root_bounds),
                &bounding_client_rect,
                &intersection_rect,
                is_intersecting,
                is_visible,
                Finite::wrap(intersection_ratio),
                target,
                can_gc,
            )
            .as_traced(),
        );
        // > Step 3
        // Queue an intersection observer task for document.
        document.queue_an_intersection_observer_task();
    }

    /// Step 3.1-3.5 of <https://w3c.github.io/IntersectionObserver/#notify-intersection-observers-algo>
    pub(crate) fn invoke_callback_if_necessary(&self, can_gc: CanGc) {
        // Step 1
        // > If observer’s internal [[QueuedEntries]] slot is empty, continue.
        if self.queued_entries.borrow().is_empty() {
            return;
        }

        // Step 2-3
        // We trivially moved the entries and root them.
        let queued_entries = self
            .queued_entries
            .take()
            .iter_mut()
            .map(|entry| entry.as_rooted())
            .collect();

        // Step 4-5
        let _ = self.callback.Call_(
            self,
            queued_entries,
            self,
            ExceptionHandling::Report,
            can_gc,
        );
    }

    /// Connect the observer itself into owner doc if it is unconnected.
    /// It would not check whether the observer is already connected or not inside the doc.
    fn connect_to_owner_unchecked(&self) {
        self.owner_doc.add_intersection_observer(self);
    }

    /// Disconnect the observer itself from owner doc.
    /// It would not check whether the observer is already disconnected or not inside the doc.
    fn disconnect_from_owner_unchecked(&self) {
        self.owner_doc.remove_intersection_observer(self);
    }

    /// > The root intersection rectangle for an IntersectionObserver is
    /// > the rectangle we’ll use to check against the targets.
    ///
    /// <https://w3c.github.io/IntersectionObserver/#intersectionobserver-root-intersection-rectangle>
    pub(crate) fn root_intersection_rectangle(&self, document: &Document) -> Option<Rect<Au>> {
        let intersection_rectangle = match &self.root {
            // Handle if root is an element.
            Some(ElementOrDocument::Element(element)) => {
                // TODO: recheck scrollbar approach and clip-path clipping from Chromium implementation.

                // > Otherwise, if the intersection root has a content clip,
                // > it’s the element’s padding area.
                // TODO(stevennovaryo): check for content clip

                // > Otherwise, it’s the result of getting the bounding box for the intersection root.
                // TODO: replace this once getBoundingBox() is implemented correctly.
                DomRoot::upcast::<Node>(element.clone()).bounding_content_box_no_reflow()
            },
            // Handle if root is a Document, which includes implicit root and explicit Document root.
            _ => {
                let document = if self.root.is_none() {
                    // > If the IntersectionObserver is an implicit root observer,
                    // > it’s treated as if the root were the top-level browsing context’s document,
                    // > according to the following rule for document.
                    //
                    // There are uncertainties whether the browsing context we should consider is the browsing
                    // context of the target or observer. <https://github.com/w3c/IntersectionObserver/issues/456>
                    document
                        .window()
                        .webview_window_proxy()
                        .and_then(|window_proxy| window_proxy.document())
                } else if let Some(ElementOrDocument::Document(document)) = &self.root {
                    Some(document.clone())
                } else {
                    None
                };

                // > If the intersection root is a document, it’s the size of the document's viewport
                // > (note that this processing step can only be reached if the document is fully active).
                // TODO: viewport should consider native scrollbar if exist. Recheck Servo's scrollbar approach.
                document.map(|document| {
                    let viewport = document.window().viewport_details().size;
                    Rect::from_size(Size2D::new(
                        Au::from_f32_px(viewport.width),
                        Au::from_f32_px(viewport.height),
                    ))
                })
            },
        };

        // > When calculating the root intersection rectangle for a same-origin-domain target,
        // > the rectangle is then expanded according to the offsets in the IntersectionObserver’s
        // > [[rootMargin]] slot in a manner similar to CSS’s margin property, with the four values
        // > indicating the amount the top, right, bottom, and left edges, respectively, are offset by,
        // > with positive lengths indicating an outward offset. Percentages are resolved relative to
        // > the width of the undilated rectangle.
        // TODO(stevennovaryo): add check for same-origin-domain
        intersection_rectangle.map(|intersection_rectangle| {
            let margin = self
                .root_margin
                .borrow()
                .resolve_percentages_with_basis(intersection_rectangle);
            intersection_rectangle.outer_rect(margin)
        })
    }

    /// Step 2.2.4-2.2.21 of <https://w3c.github.io/IntersectionObserver/#update-intersection-observations-algo>
    ///
    /// If some conditions require to skips "processing further", we will skips those steps and
    /// return default values conformant to step 2.2.4. See [`IntersectionObservationOutput::default_skipped`].
    ///
    /// Note that current draft specs skipped wrong steps, as it should skip computing fields that
    /// would result in different intersection entry other than the default entry per published spec.
    /// <https://www.w3.org/TR/intersection-observer/>
    fn maybe_compute_intersection_output(
        &self,
        document: &Document,
        target: &Element,
        maybe_root_bounds: Option<Rect<Au>>,
    ) -> IntersectionObservationOutput {
        // Step 5
        // > If the intersection root is not the implicit root, and target is not in
        // > the same document as the intersection root, skip to step 11.
        if !self.root_is_implicit_root() && *target.owner_document() != *document {
            return IntersectionObservationOutput::default_skipped();
        }

        // Step 6
        // > If the intersection root is an Element, and target is not a descendant of
        // > the intersection root in the containing block chain, skip to step 11.
        // TODO(stevennovaryo): implement LayoutThread query that support this.
        if let Some(_element) = self.maybe_element_root() {
            debug!("descendant of containing block chain is not implemented");
        }

        // Step 7
        // > Set targetRect to the DOMRectReadOnly obtained by getting the bounding box for target.
        // This is what we are currently using for getBoundingBox(). However, it is not correct,
        // mainly because it is not considering transform and scroll offset.
        // TODO: replace this once getBoundingBox() is implemented correctly.
        let maybe_target_rect = target.upcast::<Node>().bounding_content_box_no_reflow();

        // Following the implementation of Gecko, we will skip further processing if these
        // information not available. This would also handle display none element.
        if maybe_root_bounds.is_none() || maybe_target_rect.is_none() {
            return IntersectionObservationOutput::default_skipped();
        }
        let root_bounds = maybe_root_bounds.unwrap();
        let target_rect = maybe_target_rect.unwrap();

        // TODO(stevennovaryo): we should probably also consider adding visibity check, ideally
        //                      it would require new query from LayoutThread.

        // Step 8
        // > Let intersectionRect be the result of running the compute the intersection algorithm on
        // > target and observer’s intersection root.
        let intersection_rect =
            compute_the_intersection(document, target, &self.root, root_bounds, target_rect);

        // Step 9
        // > Let targetArea be targetRect’s area.
        // Step 10
        // > Let intersectionArea be intersectionRect’s area.
        // These steps are folded in Step 12, rewriting (w1 * h1) / (w2 * h2) as (w1 / w2) * (h1 / h2)
        // to avoid multiplication overflows.

        // Step 11
        // > Let isIntersecting be true if targetRect and rootBounds intersect or are edge-adjacent,
        // > even if the intersection has zero area (because rootBounds or targetRect have zero area).
        // Because we are considering edge-adjacent, instead of checking whether the rectangle is empty,
        // we are checking whether the rectangle is negative or not.
        // TODO(stevennovaryo): there is a dicussion regarding isIntersecting definition, we should update
        //                      it accordingly. https://github.com/w3c/IntersectionObserver/issues/432
        let is_intersecting = !target_rect
            .to_box2d()
            .intersection_unchecked(&root_bounds.to_box2d())
            .is_negative();

        // Step 12
        // > If targetArea is non-zero, let intersectionRatio be intersectionArea divided by targetArea.
        // > Otherwise, let intersectionRatio be 1 if isIntersecting is true, or 0 if isIntersecting is false.
        let intersection_ratio = if target_rect.size.width.0 == 0 || target_rect.size.height.0 == 0
        {
            is_intersecting.into()
        } else {
            (intersection_rect.size.width.0 as f64 / target_rect.size.width.0 as f64) *
                (intersection_rect.size.height.0 as f64 / target_rect.size.height.0 as f64)
        };

        // Step 13
        // > Set thresholdIndex to the index of the first entry in observer.thresholds whose value is
        // > greater than intersectionRatio, or the length of observer.thresholds if intersectionRatio is
        // > greater than or equal to the last entry in observer.thresholds.
        let threshold_index = self
            .thresholds
            .borrow()
            .iter()
            .position(|threshold| **threshold > intersection_ratio)
            .unwrap_or(self.thresholds.borrow().len()) as i32;

        // Step 14
        // > Let isVisible be the result of running the visibility algorithm on target.
        // TODO: Implement visibility algorithm
        let is_visible = false;

        IntersectionObservationOutput::new_computed(
            threshold_index,
            is_intersecting,
            target_rect,
            intersection_rect,
            intersection_ratio,
            is_visible,
            root_bounds,
        )
    }

    /// Step 2.2.1-2.2.21 of <https://w3c.github.io/IntersectionObserver/#update-intersection-observations-algo>
    pub(crate) fn update_intersection_observations_steps(
        &self,
        document: &Document,
        time: CrossProcessInstant,
        root_bounds: Option<Rect<Au>>,
        can_gc: CanGc,
    ) {
        for target in &*self.observation_targets.borrow() {
            // Step 1
            // > Let registration be the IntersectionObserverRegistration record in target’s internal
            // > [[RegisteredIntersectionObservers]] slot whose observer property is equal to observer.
            let registration = target.get_intersection_observer_registration(self).unwrap();

            // Step 2
            // > If (time - registration.lastUpdateTime < observer.delay), skip further processing for target.
            if time - registration.last_update_time.get() <
                Duration::from_millis(self.delay.get().max(0) as u64)
            {
                return;
            }

            // Step 3
            // > Set registration.lastUpdateTime to time.
            registration.last_update_time.set(time);

            // step 4-14
            let intersection_output =
                self.maybe_compute_intersection_output(document, target, root_bounds);

            // Step 15-17
            // > 15. Let previousThresholdIndex be the registration’s previousThresholdIndex property.
            // > 16. Let previousIsIntersecting be the registration’s previousIsIntersecting property.
            // > 17. Let previousIsVisible be the registration’s previousIsVisible property.
            let previous_threshold_index = registration.previous_threshold_index.get();
            let previous_is_intersecting = registration.previous_is_intersecting.get();
            let previous_is_visible = registration.previous_is_visible.get();

            // Step 18
            // > If thresholdIndex does not equal previousThresholdIndex, or
            // > if isIntersecting does not equal previousIsIntersecting, or
            // > if isVisible does not equal previousIsVisible,
            // > queue an IntersectionObserverEntry, passing in observer, time, rootBounds,
            // > targetRect, intersectionRect, isIntersecting, isVisible, and target.
            if intersection_output.threshold_index != previous_threshold_index ||
                intersection_output.is_intersecting != previous_is_intersecting ||
                intersection_output.is_visible != previous_is_visible
            {
                // TODO(stevennovaryo): Per IntersectionObserverEntry interface, the rootBounds
                //                      should be null for cross-origin-domain target.
                self.queue_an_intersectionobserverentry(
                    document,
                    time,
                    intersection_output.root_bounds,
                    intersection_output.target_rect,
                    intersection_output.intersection_rect,
                    intersection_output.is_intersecting,
                    intersection_output.is_visible,
                    intersection_output.intersection_ratio,
                    target,
                    can_gc,
                );
            }

            // Step 19-21
            // > 19. Assign thresholdIndex to registration’s previousThresholdIndex property.
            // > 20. Assign isIntersecting to registration’s previousIsIntersecting property.
            // > 21. Assign isVisible to registration’s previousIsVisible property.
            registration
                .previous_threshold_index
                .set(intersection_output.threshold_index);
            registration
                .previous_is_intersecting
                .set(intersection_output.is_intersecting);
            registration
                .previous_is_visible
                .set(intersection_output.is_visible);
        }
    }
}

impl IntersectionObserverMethods<crate::DomTypeHolder> for IntersectionObserver {
    /// > The root provided to the IntersectionObserver constructor, or null if none was provided.
    ///
    /// <https://w3c.github.io/IntersectionObserver/#dom-intersectionobserver-root>
    fn GetRoot(&self) -> Option<ElementOrDocument> {
        self.root.clone()
    }

    /// > Offsets applied to the root intersection rectangle, effectively growing or
    /// > shrinking the box that is used to calculate intersections. These offsets are only
    /// > applied when handling same-origin-domain targets; for cross-origin-domain targets
    /// > they are ignored.
    ///
    /// <https://w3c.github.io/IntersectionObserver/#dom-intersectionobserver-rootmargin>
    fn RootMargin(&self) -> DOMString {
        DOMString::from_string(self.root_margin.borrow().to_css_string())
    }

    /// > Offsets are applied to scrollports on the path from intersection root to target,
    /// > effectively growing or shrinking the clip rects used to calculate intersections.
    ///
    /// <https://w3c.github.io/IntersectionObserver/#dom-intersectionobserver-scrollmargin>
    fn ScrollMargin(&self) -> DOMString {
        DOMString::from_string(self.scroll_margin.borrow().to_css_string())
    }

    /// > A list of thresholds, sorted in increasing numeric order, where each threshold
    /// > is a ratio of intersection area to bounding box area of an observed target.
    /// > Notifications for a target are generated when any of the thresholds are crossed
    /// > for that target. If no options.threshold was provided to the IntersectionObserver
    /// > constructor, or the sequence is empty, the value of this attribute will be `[0]`.
    ///
    /// <https://w3c.github.io/IntersectionObserver/#dom-intersectionobserver-thresholds>
    fn Thresholds(&self, context: JSContext, can_gc: CanGc, retval: MutableHandleValue) {
        to_frozen_array(&self.thresholds.borrow(), context, retval, can_gc);
    }

    /// > A number indicating the minimum delay in milliseconds between notifications from
    /// > this observer for a given target.
    ///
    /// <https://w3c.github.io/IntersectionObserver/#dom-intersectionobserver-delay>
    fn Delay(&self) -> i32 {
        self.delay.get()
    }

    /// > A boolean indicating whether this IntersectionObserver will track changes in a target’s visibility.
    ///
    /// <https://w3c.github.io/IntersectionObserver/#dom-intersectionobserver-trackvisibility>
    fn TrackVisibility(&self) -> bool {
        self.track_visibility.get()
    }

    /// > Run the observe a target Element algorithm, providing this and target.
    ///
    /// <https://w3c.github.io/IntersectionObserver/#dom-intersectionobserver-observe>
    fn Observe(&self, target: &Element) {
        self.observe_target_element(target);

        // Connect to owner doc to be accessed in the event loop.
        self.connect_to_owner_unchecked();
    }

    /// > Run the unobserve a target Element algorithm, providing this and target.
    ///
    /// <https://w3c.github.io/IntersectionObserver/#dom-intersectionobserver-unobserve>
    fn Unobserve(&self, target: &Element) {
        self.unobserve_target_element(target);
    }

    /// <https://w3c.github.io/IntersectionObserver/#dom-intersectionobserver-disconnect>
    fn Disconnect(&self) {
        // > For each target in this’s internal [[ObservationTargets]] slot:
        self.observation_targets.borrow().iter().for_each(|target| {
            // > 1. Remove the IntersectionObserverRegistration record whose observer property is equal to
            // >    this from target’s internal [[RegisteredIntersectionObservers]] slot.
            target.remove_intersection_observer(self);
        });
        // > 2. Remove target from this’s internal [[ObservationTargets]] slot.
        self.observation_targets.borrow_mut().clear();

        // We should remove this observer from the event loop.
        self.disconnect_from_owner_unchecked();
    }

    /// <https://w3c.github.io/IntersectionObserver/#dom-intersectionobserver-takerecords>
    fn TakeRecords(&self) -> Vec<DomRoot<IntersectionObserverEntry>> {
        // Step 1-3.
        self.queued_entries
            .take()
            .iter()
            .map(|entry| entry.as_rooted())
            .collect()
    }

    /// <https://w3c.github.io/IntersectionObserver/#dom-intersectionobserver-intersectionobserver>
    fn Constructor(
        window: &Window,
        proto: Option<HandleObject>,
        can_gc: CanGc,
        callback: Rc<IntersectionObserverCallback>,
        init: &IntersectionObserverInit,
    ) -> Fallible<DomRoot<IntersectionObserver>> {
        Self::new(window, proto, callback, init, can_gc)
    }
}

/// <https://w3c.github.io/IntersectionObserver/#intersectionobserverregistration>
#[derive(Clone, JSTraceable, MallocSizeOf)]
#[cfg_attr(crown, crown::unrooted_must_root_lint::must_root)]
pub(crate) struct IntersectionObserverRegistration {
    pub(crate) observer: Dom<IntersectionObserver>,
    pub(crate) previous_threshold_index: Cell<i32>,
    pub(crate) previous_is_intersecting: Cell<bool>,
    #[no_trace]
    pub(crate) last_update_time: Cell<CrossProcessInstant>,
    pub(crate) previous_is_visible: Cell<bool>,
}

impl IntersectionObserverRegistration {
    /// Initial value of [`IntersectionObserverRegistration`] according to
    /// step 2 of <https://w3c.github.io/IntersectionObserver/#observe-target-element>.
    /// > Let intersectionObserverRegistration be an IntersectionObserverRegistration record with
    /// > an observer property set to observer, a previousThresholdIndex property set to -1,
    /// > a previousIsIntersecting property set to false, and a previousIsVisible property set to false.
    pub(crate) fn new_initial(observer: &IntersectionObserver) -> Self {
        IntersectionObserverRegistration {
            observer: Dom::from_ref(observer),
            previous_threshold_index: Cell::new(-1),
            previous_is_intersecting: Cell::new(false),
            last_update_time: Cell::new(CrossProcessInstant::epoch()),
            previous_is_visible: Cell::new(false),
        }
    }
}

/// <https://w3c.github.io/IntersectionObserver/#parse-a-margin>
fn parse_a_margin(value: Option<&DOMString>) -> Result<IntersectionObserverRootMargin, ()> {
    // <https://w3c.github.io/IntersectionObserver/#dom-intersectionobserverinit-rootmargin> &&
    // <https://w3c.github.io/IntersectionObserver/#dom-intersectionobserverinit-scrollmargin>
    // > ... defaulting to "0px".
    let value = match value {
        Some(str) => str.str(),
        _ => "0px",
    };

    // Create necessary style ParserContext and utilize stylo's IntersectionObserverRootMargin
    let mut input = ParserInput::new(value);
    let mut parser = Parser::new(&mut input);

    let url = Url::parse("about:blank").unwrap().into();
    let context = ParserContext::new(
        Origin::Author,
        &url,
        Some(CssRuleType::Style),
        ParsingMode::DEFAULT,
        QuirksMode::NoQuirks,
        /* namespaces = */ Default::default(),
        None,
        None,
    );

    parser
        .parse_entirely(|p| IntersectionObserverRootMargin::parse(&context, p))
        .map_err(|_| ())
}

/// <https://w3c.github.io/IntersectionObserver/#compute-the-intersection>
fn compute_the_intersection(
    _document: &Document,
    _target: &Element,
    _root: &IntersectionRoot,
    root_bounds: Rect<Au>,
    mut intersection_rect: Rect<Au>,
) -> Rect<Au> {
    // > 1. Let intersectionRect be the result of getting the bounding box for target.
    // We had delegated the computation of this to the caller of the function.

    // > 2. Let container be the containing block of target.
    // > 3. While container is not root:
    // >    1. If container is the document of a nested browsing context, update intersectionRect
    // >       by clipping to the viewport of the document,
    // >       and update container to be the browsing context container of container.
    // >    2. Map intersectionRect to the coordinate space of container.
    // >    3. If container is a scroll container, apply the IntersectionObserver’s [[scrollMargin]]
    // >       to the container’s clip rect as described in apply scroll margin to a scrollport.
    // >    4. If container has a content clip or a css clip-path property, update intersectionRect
    // >       by applying container’s clip.
    // >    5. If container is the root element of a browsing context, update container to be the
    // >       browsing context’s document; otherwise, update container to be the containing block
    // >       of container.
    // TODO: Implement rest of step 2 and 3, which will consider transform matrix, window scroll, etc.

    // Step 4
    // > Map intersectionRect to the coordinate space of root.
    // TODO: implement this by considering the transform matrix, window scroll, etc.

    // Step 5
    // > Update intersectionRect by intersecting it with the root intersection rectangle.
    // Note that we also consider the edge-adjacent intersection.
    let intersection_box = intersection_rect
        .to_box2d()
        .intersection_unchecked(&root_bounds.to_box2d());
    // Although not specified, the result for non-intersecting rectangle should be zero rectangle.
    // So we should give zero rectangle immediately without modifying it.
    if intersection_box.is_negative() {
        return Rect::zero();
    }
    intersection_rect = intersection_box.to_rect();

    // Step 6
    // > Map intersectionRect to the coordinate space of the viewport of the document containing target.
    // TODO: implement this by considering the transform matrix, window scroll, etc.

    // Step 7
    // > Return intersectionRect.
    intersection_rect
}

/// The values from computing step 2.2.4-2.2.14 in
/// <https://w3c.github.io/IntersectionObserver/#update-intersection-observations-algo>.
/// See [`IntersectionObserver::maybe_compute_intersection_output`].
struct IntersectionObservationOutput {
    pub(crate) threshold_index: i32,
    pub(crate) is_intersecting: bool,
    pub(crate) target_rect: Rect<Au>,
    pub(crate) intersection_rect: Rect<Au>,
    pub(crate) intersection_ratio: f64,
    pub(crate) is_visible: bool,

    /// The root intersection rectangle [`IntersectionObserver::root_intersection_rectangle`].
    /// If the processing is skipped, computation should report the default zero value.
    pub(crate) root_bounds: Rect<Au>,
}

impl IntersectionObservationOutput {
    /// Default values according to
    /// <https://w3c.github.io/IntersectionObserver/#update-intersection-observations-algo>.
    /// Step 4.
    /// > Let:
    /// > - thresholdIndex be 0.
    /// > - isIntersecting be false.
    /// > - targetRect be a DOMRectReadOnly with x, y, width, and height set to 0.
    /// > - intersectionRect be a DOMRectReadOnly with x, y, width, and height set to 0.
    ///
    /// For fields that the default values is not directly mentioned, the values conformant
    /// to current browser implementation or WPT test is used instead.
    fn default_skipped() -> Self {
        Self {
            threshold_index: 0,
            is_intersecting: false,
            target_rect: Rect::zero(),
            intersection_rect: Rect::zero(),
            intersection_ratio: 0.,
            is_visible: false,
            root_bounds: Rect::zero(),
        }
    }

    fn new_computed(
        threshold_index: i32,
        is_intersecting: bool,
        target_rect: Rect<Au>,
        intersection_rect: Rect<Au>,
        intersection_ratio: f64,
        is_visible: bool,
        root_bounds: Rect<Au>,
    ) -> Self {
        Self {
            threshold_index,
            is_intersecting,
            target_rect,
            intersection_rect,
            intersection_ratio,
            is_visible,
            root_bounds,
        }
    }
}
