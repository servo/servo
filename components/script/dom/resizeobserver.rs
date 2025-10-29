/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use std::rc::Rc;

use dom_struct::dom_struct;
use euclid::Size2D;
use euclid::default::Rect;
use js::rust::HandleObject;

use crate::dom::bindings::callback::ExceptionHandling;
use crate::dom::bindings::cell::DomRefCell;
use crate::dom::bindings::codegen::Bindings::ResizeObserverBinding::{
    ResizeObserverBoxOptions, ResizeObserverCallback, ResizeObserverMethods, ResizeObserverOptions,
};
use crate::dom::bindings::codegen::Bindings::WindowBinding::WindowMethods;
use crate::dom::bindings::inheritance::Castable;
use crate::dom::bindings::reflector::{Reflector, reflect_dom_object_with_proto};
use crate::dom::bindings::root::{Dom, DomRoot};
use crate::dom::document::RenderingUpdateReason;
use crate::dom::domrectreadonly::DOMRectReadOnly;
use crate::dom::element::Element;
use crate::dom::node::{Node, NodeTraits};
use crate::dom::resizeobserverentry::ResizeObserverEntry;
use crate::dom::resizeobserversize::{ResizeObserverSize, ResizeObserverSizeImpl};
use crate::dom::window::Window;
use crate::script_runtime::CanGc;

/// <https://drafts.csswg.org/resize-observer/#calculate-depth-for-node>
#[derive(Debug, Default, PartialEq, PartialOrd)]
pub(crate) struct ResizeObservationDepth(usize);

impl ResizeObservationDepth {
    pub(crate) fn max() -> ResizeObservationDepth {
        ResizeObservationDepth(usize::MAX)
    }
}

/// <https://drafts.csswg.org/resize-observer/#resize-observer-slots>
/// See `ObservationState` for active and skipped observation targets.
#[dom_struct]
pub(crate) struct ResizeObserver {
    reflector_: Reflector,

    /// <https://drafts.csswg.org/resize-observer/#dom-resizeobserver-callback-slot>
    #[conditional_malloc_size_of]
    callback: Rc<ResizeObserverCallback>,

    /// <https://drafts.csswg.org/resize-observer/#dom-resizeobserver-observationtargets-slot>
    ///
    /// This list simultaneously also represents the
    /// [`[[activeTargets]]`](https://drafts.csswg.org/resize-observer/#dom-resizeobserver-activetargets-slot)
    /// and [`[[skippedTargets]]`](https://drafts.csswg.org/resize-observer/#dom-resizeobserver-skippedtargets-slot)
    /// internal slots.
    observation_targets: DomRefCell<Vec<(ResizeObservation, Dom<Element>)>>,
}

impl ResizeObserver {
    pub(crate) fn new_inherited(callback: Rc<ResizeObserverCallback>) -> ResizeObserver {
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
        can_gc: CanGc,
    ) -> DomRoot<ResizeObserver> {
        let observer = Box::new(ResizeObserver::new_inherited(callback));
        reflect_dom_object_with_proto(observer, window, proto, can_gc)
    }

    /// Step 2 of <https://drafts.csswg.org/resize-observer/#gather-active-observations-h>
    ///
    /// <https://drafts.csswg.org/resize-observer/#has-active-resize-observations>
    pub(crate) fn gather_active_resize_observations_at_depth(
        &self,
        depth: &ResizeObservationDepth,
        has_active: &mut bool,
    ) {
        // Step 2.1 Clear observer’s [[activeTargets]], and [[skippedTargets]].
        // NOTE: This happens as part of Step 2.2

        // Step 2.2 For each observation in observer.[[observationTargets]] run this step:
        for (observation, target) in self.observation_targets.borrow_mut().iter_mut() {
            observation.state = Default::default();

            // Step 2.2.1 If observation.isActive() is true
            if observation.is_active(target) {
                // Step 2.2.1.1 Let targetDepth be result of calculate depth for node for observation.target.
                let target_depth = calculate_depth_for_node(target);

                // Step 2.2.1.2 If targetDepth is greater than depth then add observation to [[activeTargets]].
                if target_depth > *depth {
                    observation.state = ObservationState::Active;
                    *has_active = true;
                }
                // Step 2.2.1.3 Else add observation to [[skippedTargets]].
                else {
                    observation.state = ObservationState::Skipped;
                }
            }
        }
    }

    /// Step 2 of <https://drafts.csswg.org/resize-observer/#broadcast-active-resize-observations>
    pub(crate) fn broadcast_active_resize_observations(
        &self,
        shallowest_target_depth: &mut ResizeObservationDepth,
        can_gc: CanGc,
    ) {
        // Step 2.1 If observer.[[activeTargets]] slot is empty, continue.
        // NOTE: Due to the way we implement the activeTarges internal slot we can't easily
        // know if it's empty. Instead we remember whether there were any active observation
        // targets during the following traversal and return if there were none.
        let mut has_active_observation_targets = false;

        // Step 2.2 Let entries be an empty list of ResizeObserverEntryies.
        let mut entries: Vec<DomRoot<ResizeObserverEntry>> = Default::default();

        // Step 2.3 For each observation in [[activeTargets]] perform these steps:
        for (observation, target) in self.observation_targets.borrow_mut().iter_mut() {
            let ObservationState::Active = observation.state else {
                continue;
            };
            has_active_observation_targets = true;

            let window = target.owner_window();
            let entry =
                create_and_populate_a_resizeobserverentry(&window, target, observation, can_gc);
            entries.push(entry);
            observation.state = ObservationState::Done;

            let target_depth = calculate_depth_for_node(target);
            if target_depth < *shallowest_target_depth {
                *shallowest_target_depth = target_depth;
            }
        }

        if !has_active_observation_targets {
            return;
        }

        // Step 2.4 Invoke observer.[[callback]] with entries.
        let _ = self
            .callback
            .Call_(self, entries, self, ExceptionHandling::Report, can_gc);

        // Step 2.5 Clear observer.[[activeTargets]].
        // NOTE: The observation state was modified in Step 2.2
    }

    /// <https://drafts.csswg.org/resize-observer/#has-skipped-observations-h>
    pub(crate) fn has_skipped_resize_observations(&self) -> bool {
        self.observation_targets
            .borrow()
            .iter()
            .any(|(observation, _)| observation.state == ObservationState::Skipped)
    }
}

/// <https://drafts.csswg.org/resize-observer/#create-and-populate-a-resizeobserverentry>
fn create_and_populate_a_resizeobserverentry(
    window: &Window,
    target: &Element,
    observation: &mut ResizeObservation,
    can_gc: CanGc,
) -> DomRoot<ResizeObserverEntry> {
    let border_box_size = calculate_box_size(target, &ResizeObserverBoxOptions::Border_box);
    let content_box_size = calculate_box_size(target, &ResizeObserverBoxOptions::Content_box);
    let (padding_top, padding_left) = target
        .upcast::<Node>()
        .padding_top_and_left()
        .unwrap_or_default();

    let device_pixel_content_box =
        calculate_box_size(target, &ResizeObserverBoxOptions::Device_pixel_content_box);

    // Note: this is safe because an observation is
    // initialized with one reported size (zero).
    // The spec plans to store multiple reported sizes,
    // but for now there can be only one.
    let last_reported_size =
        ResizeObserverSizeImpl::new(content_box_size.width(), content_box_size.height());
    if observation.last_reported_sizes.is_empty() {
        observation.last_reported_sizes.push(last_reported_size);
    } else {
        observation.last_reported_sizes[0] = last_reported_size;
    }

    let content_rect = DOMRectReadOnly::new(
        window.upcast(),
        None,
        padding_left.to_f64_px(),
        padding_top.to_f64_px(),
        content_box_size.width(),
        content_box_size.height(),
        can_gc,
    );

    let border_box_size = ResizeObserverSize::new(
        window,
        ResizeObserverSizeImpl::new(border_box_size.width(), border_box_size.height()),
        can_gc,
    );
    let content_box_size = ResizeObserverSize::new(
        window,
        ResizeObserverSizeImpl::new(content_box_size.width(), content_box_size.height()),
        can_gc,
    );
    let device_pixel_content_box = ResizeObserverSize::new(
        window,
        ResizeObserverSizeImpl::new(
            device_pixel_content_box.width(),
            device_pixel_content_box.height(),
        ),
        can_gc,
    );

    ResizeObserverEntry::new(
        window,
        target,
        &content_rect,
        &[&*border_box_size],
        &[&*content_box_size],
        &[&*device_pixel_content_box],
        can_gc,
    )
}

impl ResizeObserverMethods<crate::DomTypeHolder> for ResizeObserver {
    /// <https://drafts.csswg.org/resize-observer/#dom-resizeobserver-resizeobserver>
    fn Constructor(
        window: &Window,
        proto: Option<HandleObject>,
        can_gc: CanGc,
        callback: Rc<ResizeObserverCallback>,
    ) -> DomRoot<ResizeObserver> {
        let rooted_observer = ResizeObserver::new(window, proto, callback, can_gc);
        let document = window.Document();
        document.add_resize_observer(&rooted_observer);
        rooted_observer
    }

    /// <https://drafts.csswg.org/resize-observer/#dom-resizeobserver-observe>
    fn Observe(&self, target: &Element, options: &ResizeObserverOptions) {
        // Step 1. If target is in [[observationTargets]] slot, call unobserve() with argument target.
        let is_present = self
            .observation_targets
            .borrow()
            .iter()
            .any(|(_obs, other)| &**other == target);
        if is_present {
            self.Unobserve(target);
        }

        // Step 2. Let observedBox be the value of the box dictionary member of options.
        // Step 3. Let resizeObservation be new ResizeObservation(target, observedBox).
        let resize_observation = ResizeObservation::new(options.box_);

        // Step 4. Add the resizeObservation to the [[observationTargets]] slot.
        self.observation_targets
            .borrow_mut()
            .push((resize_observation, Dom::from_ref(target)));
        target
            .owner_window()
            .Document()
            .add_rendering_update_reason(
                RenderingUpdateReason::ResizeObserverStartedObservingTarget,
            );
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
    Active,
    /// <https://drafts.csswg.org/resize-observer/#dom-resizeobserver-skippedtargets-slot>
    Skipped,
}

/// <https://drafts.csswg.org/resize-observer/#resizeobservation>
///
/// Note: `target` is kept out of here, to avoid having to root the `ResizeObservation`.
/// <https://drafts.csswg.org/resize-observer/#dom-resizeobservation-target>
#[derive(JSTraceable, MallocSizeOf)]
struct ResizeObservation {
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
    pub(crate) fn new(observed_box: ResizeObserverBoxOptions) -> ResizeObservation {
        ResizeObservation {
            observed_box,
            last_reported_sizes: vec![],
            state: Default::default(),
        }
    }

    /// <https://drafts.csswg.org/resize-observer/#dom-resizeobservation-isactive>
    fn is_active(&self, target: &Element) -> bool {
        let Some(last_reported_size) = self.last_reported_sizes.first() else {
            return true;
        };
        let box_size = calculate_box_size(target, &self.observed_box);
        box_size.width() != last_reported_size.inline_size() ||
            box_size.height() != last_reported_size.block_size()
    }
}

/// <https://drafts.csswg.org/resize-observer/#calculate-depth-for-node>
fn calculate_depth_for_node(target: &Element) -> ResizeObservationDepth {
    let node = target.upcast::<Node>();
    let depth = node.inclusive_ancestors_in_flat_tree().count();
    ResizeObservationDepth(depth)
}

/// <https://drafts.csswg.org/resize-observer/#calculate-box-size>
///
/// The dimensions of the returned `Rect` depend on the type of box being observed.
/// For `ResizeObserverBoxOptions::Content_box` and `ResizeObserverBoxOptions::Border_box`,
/// the values will be in `px`. For `ResizeObserverBoxOptions::Device_pixel_content_box` they
/// will be in integral device pixels.
fn calculate_box_size(target: &Element, observed_box: &ResizeObserverBoxOptions) -> Rect<f64> {
    match observed_box {
        ResizeObserverBoxOptions::Content_box => {
            // Note: only taking first fragment,
            // but the spec will expand to cover all fragments.
            let content_box = target
                .upcast::<Node>()
                .content_box()
                .unwrap_or_else(Rect::zero);

            Rect::new(
                content_box.origin.map(|coordinate| coordinate.to_f64_px()),
                Size2D::new(
                    content_box.size.width.to_f64_px(),
                    content_box.size.height.to_f64_px(),
                ),
            )
        },
        ResizeObserverBoxOptions::Border_box => {
            // Note: only taking first fragment,
            // but the spec will expand to cover all fragments.
            let border_box = target
                .upcast::<Node>()
                .border_box()
                .unwrap_or_else(Rect::zero);

            Rect::new(
                border_box.origin.map(|coordinate| coordinate.to_f64_px()),
                Size2D::new(
                    border_box.size.width.to_f64_px(),
                    border_box.size.height.to_f64_px(),
                ),
            )
        },
        ResizeObserverBoxOptions::Device_pixel_content_box => {
            let device_pixel_ratio = target.owner_window().device_pixel_ratio();
            let content_box = target
                .upcast::<Node>()
                .content_box()
                .unwrap_or_else(Rect::zero);

            Rect::new(
                content_box
                    .origin
                    .map(|coordinate| coordinate.to_nearest_pixel(device_pixel_ratio.get()) as f64),
                Size2D::new(
                    content_box
                        .size
                        .width
                        .to_nearest_pixel(device_pixel_ratio.get()) as f64,
                    content_box
                        .size
                        .height
                        .to_nearest_pixel(device_pixel_ratio.get()) as f64,
                ),
            )
        },
    }
}
