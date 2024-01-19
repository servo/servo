/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use std::rc::Rc;

use dom_struct::dom_struct;
use js::rust::HandleObject;

use crate::dom::bindings::cell::DomRefCell;
use crate::dom::bindings::codegen::Bindings::ResizeObserverBinding::{
    ResizeObserverBoxOptions, ResizeObserverCallback, ResizeObserverMethods, ResizeObserverOptions,
};
use crate::dom::bindings::codegen::Bindings::WindowBinding::WindowMethods;
use crate::dom::bindings::reflector::{reflect_dom_object_with_proto, DomObjectWrap, Reflector};
use crate::dom::bindings::root::{Dom, DomRoot};
use crate::dom::element::Element;
use crate::dom::node::window_from_node;
use crate::dom::resizeobserversize::ResizeObserverSize;
use crate::dom::window::Window;
use crate::script_runtime::JSContext as SafeJSContext;

/// https://drafts.csswg.org/resize-observer/#resize-observer-slots
/// See `ObservationState` below for active and skipped observation targets.
#[dom_struct]
pub struct ResizeObserver {
    reflector_: Reflector,
    /// https://drafts.csswg.org/resize-observer/#dom-resizeobserver-callback-slot
    #[ignore_malloc_size_of = "Rc are hard"]
    callback: Rc<ResizeObserverCallback>,
    /// https://drafts.csswg.org/resize-observer/#dom-resizeobserver-observationtargets-slot
    observation_targets: DomRefCell<Vec<ResizeObservation>>,
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

    /// https://drafts.csswg.org/resize-observer/#gather-active-observations-h
    pub fn gather_active_resize_observations_at_depth(&self, depth: u32) {
        // Step 2.2
        for observation in self.observation_targets.borrow_mut().iter_mut() {
            if observation.is_active() {
                // TODO https://drafts.csswg.org/resize-observer/#calculate-depth-for-node
                let target_depth = 0;
                if target_depth > depth {
                    observation.state = ObservationState::Active;
                } else {
                    observation.state = ObservationState::Skipped;
                }
            }
        }
    }

    /// https://drafts.csswg.org/resize-observer/#broadcast-active-resize-observations
    pub fn broadcast_active_resize_observations(&self, shallowest_target_depth: u32) -> u32 {
        shallowest_target_depth
    }
}

impl ResizeObserverMethods for ResizeObserver {
    /// https://drafts.csswg.org/resize-observer/#dom-resizeobserver-observe
    fn Observe(&self, target: &Element, options: &ResizeObserverOptions) {
        // Step 1.
        let is_present = self
            .observation_targets
            .borrow()
            .iter()
            .any(|obs| &*obs.target == target);
        if is_present {
            return self.Unobserve(target);
        }

        // Step 2 and 3.
        let resize_observation = ResizeObservation::new(target, options.box_);

        // Step 4.
        self.observation_targets
            .borrow_mut()
            .push(resize_observation);
    }

    /// https://drafts.csswg.org/resize-observer/#dom-resizeobserver-unobserve
    fn Unobserve(&self, target: &Element) {
        self.observation_targets
            .borrow_mut()
            .retain_mut(|obs| !(&*obs.target == target));
    }

    /// https://drafts.csswg.org/resize-observer/#dom-resizeobserver-disconnect
    fn Disconnect(&self) {
        self.observation_targets.borrow_mut().clear();
    }
}

/// State machine equivalent of active and skipped observations.
#[derive(Default, JSTraceable, MallocSizeOf)]
enum ObservationState {
    #[default]
    Start,
    /// https://drafts.csswg.org/resize-observer/#dom-resizeobserver-activetargets-slot
    Active,
    /// https://drafts.csswg.org/resize-observer/#dom-resizeobserver-skippedtargets-slot
    Skipped,
}

/// https://drafts.csswg.org/resize-observer/#resizeobservation
#[derive(JSTraceable, MallocSizeOf)]
struct ResizeObservation {
    /// https://drafts.csswg.org/resize-observer/#dom-resizeobservation-target
    target: DomRoot<Element>,
    /// https://drafts.csswg.org/resize-observer/#dom-resizeobservation-observedbox
    observed_box: ResizeObserverBoxOptions,
    /// https://drafts.csswg.org/resize-observer/#dom-resizeobservation-lastreportedsizes
    last_reported_sizes: Vec<DomRoot<ResizeObserverSize>>,
    /// State machine mimicking the "active" and "skipped" targets slots of the observer.
    state: ObservationState,
}

impl ResizeObservation {
    /// https://drafts.csswg.org/resize-observer/#dom-resizeobservation-resizeobservation
    pub fn new(target: &Element, observed_box: ResizeObserverBoxOptions) -> ResizeObservation {
        ResizeObservation {
            target: DomRoot::from_ref(target),
            observed_box,
            last_reported_sizes: Default::default(),
            state: Default::default(),
        }
    }

    /// https://drafts.csswg.org/resize-observer/#dom-resizeobservation-isactive
    pub fn is_active(&self) -> bool {
        // TODO: https://drafts.csswg.org/resize-observer/#calculate-box-size
        true
    }
}
