/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use std::cell::Cell;
use std::sync::atomic::{AtomicUsize, Ordering};

use app_units::Au;
use dom_struct::dom_struct;
use euclid::Rect;
use euclid::num::Zero;
use html5ever::ns;
use js::context::{JSContext, NoGC};
use js::rust::HandleObject;
use layout_api::BoxAreaType;
use script_bindings::callback::{RootedCallback, TracedCallback};
use script_bindings::cell::DomRefCell;
use script_bindings::reflector::{Reflector, reflect_dom_object_with_proto};
use style::logical_geometry::WritingMode;

use crate::dom::bindings::callback::ExceptionHandling;
use crate::dom::bindings::codegen::Bindings::ResizeObserverBinding::{
    ResizeObserverBoxOptions, ResizeObserverCallback, ResizeObserverMethods, ResizeObserverOptions,
};
use crate::dom::bindings::inheritance::Castable;
use crate::dom::bindings::root::{Dom, DomRoot};
use crate::dom::document::RenderingUpdateReason;
use crate::dom::domrectreadonly::DOMRectReadOnly;
use crate::dom::element::Element;
use crate::dom::node::{Node, NodeTraits};
use crate::dom::resizeobserverentry::ResizeObserverEntry;
use crate::dom::resizeobserversize::{ResizeObserverSize, ResizeObserverSizeImpl};
use crate::dom::window::Window;

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

    /// The [`Window`] that this [`ResizeObserver`] was created in.
    window: Dom<Window>,

    /// <https://drafts.csswg.org/resize-observer/#dom-resizeobserver-callback-slot>
    callback: TracedCallback<ResizeObserverCallback>,

    /// <https://drafts.csswg.org/resize-observer/#dom-resizeobserver-observationtargets-slot>
    ///
    /// This list simultaneously also represents the
    /// [`[[activeTargets]]`](https://drafts.csswg.org/resize-observer/#dom-resizeobserver-activetargets-slot)
    /// and [`[[skippedTargets]]`](https://drafts.csswg.org/resize-observer/#dom-resizeobserver-skippedtargets-slot)
    /// internal slots.
    observation_targets: DomRefCell<Vec<(ResizeObservation, Dom<Element>)>>,

    /// An index that indicates what order this [`ResizeObsever`] was created in. This is used for
    /// keeping them in sorted order in the `Document`'s collection of observers.
    creation_order: usize,
}

impl ResizeObserver {
    pub(crate) fn new_inherited(
        window: &Window,
        callback: RootedCallback<ResizeObserverCallback>,
    ) -> ResizeObserver {
        static CREATION_ORDER: AtomicUsize = AtomicUsize::new(0);
        ResizeObserver {
            reflector_: Reflector::new(),
            window: Dom::from_ref(window),
            callback: callback.to_traced(),
            observation_targets: Default::default(),
            creation_order: CREATION_ORDER.fetch_add(1, Ordering::Relaxed),
        }
    }

    fn new(
        cx: &mut JSContext,
        window: &Window,
        proto: Option<HandleObject>,
        callback: RootedCallback<ResizeObserverCallback>,
    ) -> DomRoot<ResizeObserver> {
        let observer = Box::new(ResizeObserver::new_inherited(window, callback));
        reflect_dom_object_with_proto(cx, observer, window, proto)
    }

    /// Step 2 of <https://drafts.csswg.org/resize-observer/#gather-active-observations-h>
    ///
    /// <https://drafts.csswg.org/resize-observer/#has-active-resize-observations>
    pub(crate) fn gather_active_resize_observations_at_depth(
        &self,
        no_gc: &NoGC,
        depth: &ResizeObservationDepth,
        has_active: &mut bool,
    ) {
        // Step 2.1 Clear observer’s [[activeTargets]], and [[skippedTargets]].
        // NOTE: This happens as part of Step 2.2

        // Step 2.2 For each observation in observer.[[observationTargets]] run this step:
        for (observation, target) in self.observation_targets.borrow().iter() {
            observation.state.set(Default::default());

            // Step 2.2.1 If observation.isActive() is true
            if observation.is_active(target) {
                // Step 2.2.1.1 Let targetDepth be result of calculate depth for node for observation.target.
                let target_depth = calculate_depth_for_node(no_gc, target);

                // Step 2.2.1.2 If targetDepth is greater than depth then add observation to [[activeTargets]].
                if target_depth > *depth {
                    observation.state.set(ObservationState::Active);
                    *has_active = true;
                }
                // Step 2.2.1.3 Else add observation to [[skippedTargets]].
                else {
                    observation.state.set(ObservationState::Skipped);
                }
            }
        }
    }

    /// Step 2 of <https://drafts.csswg.org/resize-observer/#broadcast-active-resize-observations>
    pub(crate) fn broadcast_active_resize_observations(
        &self,
        cx: &mut JSContext,
        shallowest_target_depth: &mut ResizeObservationDepth,
    ) {
        // Step 2.1 If observer.[[activeTargets]] slot is empty, continue.
        // NOTE: Due to the way we implement the activeTarges internal slot we can't easily
        // know if it's empty. Instead we remember whether there were any active observation
        // targets during the following traversal and return if there were none.
        let mut has_active_observation_targets = false;

        // Step 2.2 Let entries be an empty list of ResizeObserverEntryies.
        let mut entries: Vec<DomRoot<ResizeObserverEntry>> = Default::default();

        // Step 2.3 For each observation in [[activeTargets]] perform these steps:
        for (observation, target) in self.observation_targets.borrow().iter() {
            let ObservationState::Active = observation.state.get() else {
                continue;
            };
            has_active_observation_targets = true;

            let window = target.owner_window();
            let entry = create_and_populate_a_resizeobserverentry(cx, &window, target, observation);
            entries.push(entry);
            observation.state.set(ObservationState::Done);

            let target_depth = calculate_depth_for_node(cx.no_gc(), target);
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
            .Call_(cx, self, entries, self, ExceptionHandling::Report);

        // Step 2.5 Clear observer.[[activeTargets]].
        // NOTE: The observation state was modified in Step 2.2
    }

    /// <https://drafts.csswg.org/resize-observer/#has-skipped-observations-h>
    pub(crate) fn has_skipped_resize_observations(&self) -> bool {
        self.observation_targets
            .borrow()
            .iter()
            .any(|(observation, _)| observation.state.get() == ObservationState::Skipped)
    }

    /// Whether or not the given [`ResizeObserver`] was created after this one.
    pub(crate) fn created_after(&self, other: &ResizeObserver) -> bool {
        self.creation_order > other.creation_order
    }
}

/// <https://drafts.csswg.org/resize-observer/#create-and-populate-a-resizeobserverentry>
fn create_and_populate_a_resizeobserverentry(
    cx: &mut JSContext,
    window: &Window,
    target: &Element,
    observation: &ResizeObservation,
) -> DomRoot<ResizeObserverEntry> {
    // Step 3. Set this.borderBoxSize slot to result of calculating box size given target and observedBox of "border-box".
    let border_box_size = calculate_box_size(target, &ResizeObserverBoxOptions::Border_box);
    // Step 4. Set this.contentBoxSize slot to result of calculating box size given target and observedBox of "content-box".
    let content_box_size = calculate_box_size(target, &ResizeObserverBoxOptions::Content_box);

    // Step 5. Set this.devicePixelContentBoxSize slot to result of calculating box size given target and observedBox of "device-pixel-content-box".
    let device_pixel_content_box =
        calculate_box_size(target, &ResizeObserverBoxOptions::Device_pixel_content_box);

    // Note: this is safe because an observation is
    // initialized with one reported size (zero).
    // The spec plans to store multiple reported sizes,
    // but for now there can be only one.
    let last_reported_size = match observation.observed_box {
        ResizeObserverBoxOptions::Content_box => content_box_size,
        ResizeObserverBoxOptions::Border_box => border_box_size,
        ResizeObserverBoxOptions::Device_pixel_content_box => device_pixel_content_box,
    };

    {
        let mut sizes = observation.last_reported_sizes.safe_borrow_mut(cx.no_gc());
        if sizes.is_empty() {
            sizes.push(last_reported_size);
        } else {
            sizes[0] = last_reported_size;
        }
    }

    // Step 7. If target is not an SVG element or target is an SVG element with an associated CSS
    // layout box do these steps:
    let use_padding = *target.namespace() != ns!(svg) || target.has_css_layout_box();
    let (padding_top, padding_left) = if use_padding {
        // Step 7.1. Set this.contentRect.top to target.padding top.
        // Step 7.2. Set this.contentRect.left to target.padding left.
        let padding = target.upcast::<Node>().padding().unwrap_or_default();
        (padding.top, padding.left)
    } else {
        // Step 8. If target is an SVG element without an associated CSS layout box do these steps:
        // Step 8.1. Set this.contentRect.top and this.contentRect.left to 0.
        (Au::zero(), Au::zero())
    };

    let writing_mode = target
        .style_without_reflow()
        .map_or_else(WritingMode::empty, |style| style.writing_mode);
    let (width, height) = if writing_mode.is_horizontal() {
        (content_box_size.inline_size, content_box_size.block_size)
    } else {
        (content_box_size.block_size, content_box_size.inline_size)
    };

    // Step 6. Set this.contentRect to logical this.contentBoxSize given target and observedBox of "content-box".
    let content_rect = DOMRectReadOnly::new(
        cx,
        window.upcast(),
        None,
        padding_left.to_f64_px(),
        padding_top.to_f64_px(),
        width,
        height,
    );

    let border_box_size = ResizeObserverSize::new(cx, window, border_box_size);
    let content_box_size = ResizeObserverSize::new(cx, window, content_box_size);
    let device_pixel_content_box = ResizeObserverSize::new(cx, window, device_pixel_content_box);

    // Step 1. Let this be a new ResizeObserverEntry.
    // Step 2. Set this.target slot to target.
    ResizeObserverEntry::new(
        cx,
        window,
        target,
        &content_rect,
        &[&*border_box_size],
        &[&*content_box_size],
        &[&*device_pixel_content_box],
    )
}

impl ResizeObserverMethods<crate::DomTypeHolder> for ResizeObserver {
    /// <https://drafts.csswg.org/resize-observer/#dom-resizeobserver-resizeobserver>
    fn Constructor(
        cx: &mut JSContext,
        window: &Window,
        proto: Option<HandleObject>,
        callback: RootedCallback<ResizeObserverCallback>,
    ) -> DomRoot<ResizeObserver> {
        ResizeObserver::new(cx, window, proto, callback)
    }

    /// <https://drafts.csswg.org/resize-observer/#dom-resizeobserver-observe>
    fn Observe(&self, no_gc: &NoGC, target: &Element, options: &ResizeObserverOptions) {
        let observed_box = self
            .observation_targets
            .borrow()
            .iter()
            .find(|(_, other)| &**other == target)
            .map(|(observation, _)| observation.observed_box);
        // Propoposed specification text from <https://github.com/web-platform-tests/wpt/issues/30708>.
        // >  If preexistingTarget exists and its observedBox.box equals options.box return
        // Both Chrome and Firefox do this and likely WebKit too.
        if observed_box == Some(options.box_) {
            return;
        }

        // Step 1. If target is in [[observationTargets]] slot, call unobserve() with argument target.
        if observed_box.is_some() {
            self.Unobserve(no_gc, target);
        }

        // Step 2. Let observedBox be the value of the box dictionary member of options.
        // Step 3. Let resizeObservation be new ResizeObservation(target, observedBox).
        let resize_observation = ResizeObservation::new(options.box_);

        // Step 4. Add the resizeObservation to the [[observationTargets]] slot.
        self.observation_targets
            .safe_borrow_mut(no_gc)
            .push((resize_observation, Dom::from_ref(target)));

        let document = self.window.document_unrooted(no_gc);
        document.add_rendering_update_reason(
            RenderingUpdateReason::ResizeObserverStartedObservingTarget,
        );

        // This isn't specified, but this is what Servo does to ensure that ResizeObservers can be
        // garbage collected properly. `Unobserve` will remove the observer from the Document.
        document.add_resize_observer(no_gc, self);
    }

    /// <https://drafts.csswg.org/resize-observer/#dom-resizeobserver-unobserve>
    fn Unobserve(&self, no_gc: &NoGC, target: &Element) {
        self.observation_targets
            .safe_borrow_mut(no_gc)
            .retain_mut(|(_, other)| !(&**other == target));
        if self.observation_targets.borrow().is_empty() {
            self.window
                .document_unrooted(no_gc)
                .remove_resize_observer(no_gc, self);
        }
    }

    /// <https://drafts.csswg.org/resize-observer/#dom-resizeobserver-disconnect>
    fn Disconnect(&self, no_gc: &NoGC) {
        self.observation_targets.safe_borrow_mut(no_gc).clear();
        self.window
            .document_unrooted(no_gc)
            .remove_resize_observer(no_gc, self);
    }
}

/// State machine equivalent of active and skipped observations.
#[derive(Copy, Clone, Default, MallocSizeOf, PartialEq)]
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
    last_reported_sizes: DomRefCell<Vec<ResizeObserverSizeImpl>>,
    /// State machine mimicking the "active" and "skipped" targets slots of the observer.
    #[no_trace]
    state: Cell<ObservationState>,
}

impl ResizeObservation {
    /// <https://drafts.csswg.org/resize-observer/#dom-resizeobservation-resizeobservation>
    pub(crate) fn new(observed_box: ResizeObserverBoxOptions) -> ResizeObservation {
        ResizeObservation {
            observed_box,
            last_reported_sizes: DomRefCell::new(vec![]),
            state: Default::default(),
        }
    }

    /// <https://drafts.csswg.org/resize-observer/#dom-resizeobservation-isactive>
    fn is_active(&self, target: &Element) -> bool {
        let Some(last_reported_size) = self.last_reported_sizes.borrow().first().copied() else {
            return true;
        };
        calculate_box_size(target, &self.observed_box) != last_reported_size
    }
}

/// <https://drafts.csswg.org/resize-observer/#calculate-depth-for-node>
fn calculate_depth_for_node(no_gc: &NoGC, target: &Element) -> ResizeObservationDepth {
    let node = target.upcast::<Node>();
    let depth = node
        .inclusive_ancestors_in_flat_tree_unrooted(no_gc)
        .count();
    ResizeObservationDepth(depth)
}

/// <https://drafts.csswg.org/resize-observer/#calculate-box-size>
///
/// The dimensions of the returned `ResizeObserverSizeImpl` depend on the type of box being
/// observed. For `ResizeObserverBoxOptions::Content_box` and
/// `ResizeObserverBoxOptions::Border_box`, the values will be in `px`. For
/// `ResizeObserverBoxOptions::Device_pixel_content_box` they will be in integral device pixels.
fn calculate_box_size(
    target: &Element,
    observed_box: &ResizeObserverBoxOptions,
) -> ResizeObserverSizeImpl {
    let (width, height) = match observed_box {
        ResizeObserverBoxOptions::Content_box => {
            // Note: only taking first fragment,
            // but the spec will expand to cover all fragments.
            let content_box = target
                .owner_window()
                .box_area_query_without_reflow(target.upcast(), BoxAreaType::Content, true)
                .unwrap_or_else(Rect::zero);

            (
                content_box.size.width.to_f64_px(),
                content_box.size.height.to_f64_px(),
            )
        },
        ResizeObserverBoxOptions::Border_box => {
            // Note: only taking first fragment,
            // but the spec will expand to cover all fragments.
            let border_box = target
                .owner_window()
                .box_area_query_without_reflow(target.upcast(), BoxAreaType::Border, true)
                .unwrap_or_else(Rect::zero);
            (
                border_box.size.width.to_f64_px(),
                border_box.size.height.to_f64_px(),
            )
        },
        ResizeObserverBoxOptions::Device_pixel_content_box => {
            let device_pixel_ratio = target.owner_window().device_pixel_ratio().get() as f64;
            let content_box = target
                .owner_window()
                .box_area_query_without_reflow(target.upcast(), BoxAreaType::Content, true)
                .unwrap_or_else(Rect::zero);
            // This code is snapping both edges of the rect to device pixels and then calculating
            // the width. This gives the exact number of device pixels the rectangle takes up on
            // the screen.
            let to_device_px = |length: Au| (length.to_f64_px() * device_pixel_ratio).round();
            let width = to_device_px(content_box.max_x()) - to_device_px(content_box.origin.x);
            let height = to_device_px(content_box.max_y()) - to_device_px(content_box.origin.y);
            (width, height)
        },
    };

    let writing_mode = target
        .style_without_reflow()
        .map_or_else(WritingMode::empty, |style| style.writing_mode);
    let (inline_size, block_size) = if writing_mode.is_horizontal() {
        (width, height)
    } else {
        (height, width)
    };

    ResizeObserverSizeImpl {
        inline_size,
        block_size,
    }
}

impl script_bindings::callback::OwnerWindow<crate::DomTypeHolder> for ResizeObserver {}
