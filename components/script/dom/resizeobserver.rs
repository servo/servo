/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use std::rc::Rc;

use app_units::Au;
use dom_struct::dom_struct;
use euclid::default::Rect;
use js::rust::HandleObject;

use crate::dom::bindings::callback::ExceptionHandling;
use crate::dom::bindings::cell::DomRefCell;
use crate::dom::bindings::codegen::Bindings::ResizeObserverBinding::{
    ResizeObserverBoxOptions, ResizeObserverCallback, ResizeObserverMethods, ResizeObserverOptions,
};
use crate::dom::bindings::codegen::Bindings::WindowBinding::WindowMethods;
use crate::dom::bindings::inheritance::Castable;
use crate::dom::bindings::reflector::{reflect_dom_object_with_proto, Reflector};
use crate::dom::bindings::root::{Dom, DomRoot};
use crate::dom::domrectreadonly::DOMRectReadOnly;
use crate::dom::element::Element;
use crate::dom::node::{window_from_node, Node};
use crate::dom::resizeobserverentry::ResizeObserverEntry;
use crate::dom::resizeobserversize::{ResizeObserverSize, ResizeObserverSizeImpl};
use crate::dom::window::Window;
use crate::script_thread::ScriptThread;

/// <https://drafts.csswg.org/resize-observer/#calculate-depth-for-node>
#[derive(Debug, Default, PartialEq, PartialOrd)]
pub struct ResizeObservationDepth(usize);

impl ResizeObservationDepth {
    pub fn max() -> ResizeObservationDepth {
        ResizeObservationDepth(usize::MAX)
    }
}

/// <https://drafts.csswg.org/resize-observer/#resize-observer-slots>
/// See `ObservationState` for active and skipped observation targets.
#[dom_struct]
pub struct ResizeObserver {
    reflector_: Reflector,
    /// <https://drafts.csswg.org/resize-observer/#dom-resizeobserver-callback-slot>
    #[ignore_malloc_size_of = "Rc are hard"]
    callback: Rc<ResizeObserverCallback>,
    /// <https://drafts.csswg.org/resize-observer/#dom-resizeobserver-observationtargets-slot>
    observation_targets: DomRefCell<Vec<(ResizeObservation, Dom<Element>)>>,
}

impl ResizeObserver {
    pub fn new_inherited(callback: Rc<ResizeObserverCallback>) -> ResizeObserver {
        ResizeObserver {
            reflector_: Reflector::new(),
            callback,
            observation_targets: Default::default(),
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

    /// <https://drafts.csswg.org/resize-observer/#dom-resizeobserver-resizeobserver>
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

    /// <https://drafts.csswg.org/resize-observer/#gather-active-observations-h>
    /// <https://drafts.csswg.org/resize-observer/#has-active-resize-observations>
    pub fn gather_active_resize_observations_at_depth(
        &self,
        depth: &ResizeObservationDepth,
        has_active: &mut bool,
    ) {
        for (observation, target) in self.observation_targets.borrow_mut().iter_mut() {
            observation.state = Default::default();
            if let Some(size) = observation.is_active(target) {
                let target_depth = calculate_depth_for_node(target);
                if target_depth > *depth {
                    observation.state = ObservationState::Active(size);
                    *has_active = true;
                } else {
                    observation.state = ObservationState::Skipped;
                }
            }
        }
    }

    /// <https://drafts.csswg.org/resize-observer/#broadcast-active-resize-observations>
    pub fn broadcast_active_resize_observations(
        &self,
        shallowest_target_depth: &mut ResizeObservationDepth,
    ) {
        let mut entries: Vec<DomRoot<ResizeObserverEntry>> = Default::default();
        for (observation, target) in self.observation_targets.borrow_mut().iter_mut() {
            let ObservationState::Active(box_size) = observation.state else {
                continue;
            };

            // #create-and-populate-a-resizeobserverentry

            // Note: only calculating content box size.
            let width = box_size.width().to_f64_px();
            let height = box_size.height().to_f64_px();
            let size_impl = ResizeObserverSizeImpl::new(width, height);
            let window = window_from_node(&**target);
            let observer_size = ResizeObserverSize::new(&window, size_impl);

            // Note: content rect is built from content box size.
            let content_rect = DOMRectReadOnly::new(
                window.upcast(),
                None,
                box_size.origin.x.to_f64_px(),
                box_size.origin.y.to_f64_px(),
                width,
                height,
            );
            let entry = ResizeObserverEntry::new(
                &window,
                target,
                &content_rect,
                &[],
                &[&*observer_size],
                &[],
            );
            entries.push(entry);

            // Note: this is safe because an observation is
            // initialized with one reported size (zero).
            // The spec plans to store multiple reported sizes,
            // but for now there can be only one.
            observation.last_reported_sizes[0] = size_impl;
            observation.state = ObservationState::Done;
            let target_depth = calculate_depth_for_node(target);
            if target_depth < *shallowest_target_depth {
                *shallowest_target_depth = target_depth;
            }
        }
        let _ = self
            .callback
            .Call_(self, entries, self, ExceptionHandling::Report);
    }

    /// <https://drafts.csswg.org/resize-observer/#has-skipped-observations-h>
    pub fn has_skipped_resize_observations(&self) -> bool {
        self.observation_targets
            .borrow()
            .iter()
            .any(|(observation, _)| observation.state == ObservationState::Skipped)
    }
}

impl ResizeObserverMethods for ResizeObserver {
    /// <https://drafts.csswg.org/resize-observer/#dom-resizeobserver-observe>
    fn Observe(&self, target: &Element, options: &ResizeObserverOptions) {
        let is_present = self
            .observation_targets
            .borrow()
            .iter()
            .any(|(_obs, other)| &**other == target);
        if is_present {
            self.Unobserve(target);
        }

        let resize_observation = ResizeObservation::new(options.box_);

        self.observation_targets
            .borrow_mut()
            .push((resize_observation, Dom::from_ref(target)));

        // Note: noting a rendering opportunity here is necessary
        // to make /resize-observer/iframe-same-origin.html PASS.
        ScriptThread::note_rendering_opportunity(window_from_node(target).pipeline_id());
    }

    /// <https://drafts.csswg.org/resize-observer/#dom-resizeobserver-unobserve>
    fn Unobserve(&self, target: &Element) {
        self.observation_targets
            .borrow_mut()
            .retain_mut(|(_obs, other)| !(&**other == target));
    }

    /// <https://drafts.csswg.org/resize-observer/#dom-resizeobserver-disconnect>
    fn Disconnect(&self) {
        self.observation_targets.borrow_mut().clear();
    }
}

/// State machine equivalent of active and skipped observations.
#[derive(Default, MallocSizeOf, PartialEq)]
enum ObservationState {
    #[default]
    Done,
    /// <https://drafts.csswg.org/resize-observer/#dom-resizeobserver-activetargets-slot>
    /// With the result of the box size calculated when setting the state to active,
    /// in order to avoid recalculating it in the subsequent broadcast.
    Active(Rect<Au>),
    /// <https://drafts.csswg.org/resize-observer/#dom-resizeobserver-skippedtargets-slot>
    Skipped,
}

/// https://drafts.csswg.org/resize-observer/#resizeobservation
#[derive(JSTraceable, MallocSizeOf)]
struct ResizeObservation {
    /// <https://drafts.csswg.org/resize-observer/#dom-resizeobservation-target>
    /// Note: `target` is kept out of here, to avoid having to root the `ResizeObservation`.

    /// <https://drafts.csswg.org/resize-observer/#dom-resizeobservation-observedbox>
    observed_box: ResizeObserverBoxOptions,
    /// <https://drafts.csswg.org/resize-observer/#dom-resizeobservation-lastreportedsizes>
    last_reported_sizes: Vec<ResizeObserverSizeImpl>,
    /// State machine mimicking the "active" and "skipped" targets slots of the observer.
    #[no_trace]
    state: ObservationState,
}

impl ResizeObservation {
    /// <https://drafts.csswg.org/resize-observer/#dom-resizeobservation-resizeobservation>
    pub fn new(observed_box: ResizeObserverBoxOptions) -> ResizeObservation {
        let size_impl = ResizeObserverSizeImpl::new(0.0, 0.0);
        ResizeObservation {
            observed_box,
            last_reported_sizes: vec![size_impl],
            state: Default::default(),
        }
    }

    /// <https://drafts.csswg.org/resize-observer/#dom-resizeobservation-isactive>
    /// Returning an optional calculated size, instead of a boolean,
    /// to avoid recalculating the size in the subsequent broadcast.
    fn is_active(&self, target: &Element) -> Option<Rect<Au>> {
        let last_reported_size = self.last_reported_sizes[0];
        let box_size = calculate_box_size(target, &self.observed_box);
        let is_active = box_size.width().to_f64_px() != last_reported_size.inline_size() ||
            box_size.height().to_f64_px() != last_reported_size.block_size();
        if is_active {
            Some(box_size)
        } else {
            None
        }
    }
}

/// <https://drafts.csswg.org/resize-observer/#calculate-depth-for-node>
fn calculate_depth_for_node(target: &Element) -> ResizeObservationDepth {
    let node = target.upcast::<Node>();
    let depth = node.ancestors().count();
    ResizeObservationDepth(depth)
}

/// <https://drafts.csswg.org/resize-observer/#calculate-box-size>
fn calculate_box_size(target: &Element, observed_box: &ResizeObserverBoxOptions) -> Rect<Au> {
    match observed_box {
        ResizeObserverBoxOptions::Content_box => {
            // Note: only taking first fragment,
            // but the spec will expand to cover all fragments.
            target
                .upcast::<Node>()
                .content_boxes()
                .pop()
                .unwrap_or_else(Rect::zero)
        },
        // TODO(#31182): add support for border box, and device pixel size, calculations.
        _ => Rect::zero(),
    }
}
