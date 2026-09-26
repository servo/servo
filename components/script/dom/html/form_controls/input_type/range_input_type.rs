/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */
use std::cell::{Cell, Ref};

use app_units::Au;
use embedder_traits::MouseButton;
use euclid::num::Zero;
use euclid::{Point2D, Rect};
use html5ever::{local_name, ns};
use js::context::JSContext;
use markup5ever::QualName;
use script_bindings::cell::DomRefCell;
use script_bindings::codegen::GenericBindings::HTMLInputElementBinding::HTMLInputElementMethods;
use script_bindings::domstring::parse_floating_point_number;
use script_bindings::root::{Dom, DomRoot};
use style::attr::AttrValue;
use style::logical_geometry::PhysicalSide;
use style::selector_parser::PseudoElement;
use style_traits::CSSPixel;

use crate::dom::bindings::codegen::Bindings::ElementBinding::ElementMethods;
use crate::dom::bindings::codegen::Bindings::NodeBinding::NodeMethods;
use crate::dom::bindings::codegen::Bindings::PointerEventBinding::PointerEventMethods;
use crate::dom::bindings::inheritance::Castable;
use crate::dom::bindings::str::DOMString;
use crate::dom::element::{CustomElementCreationMode, Element, ElementCreator};
use crate::dom::event::Event;
use crate::dom::html::form_controls::htmlinputelement::HTMLInputElement;
use crate::dom::html::form_controls::input_type::{SpecificInputType, ValueChangeEvents};
use crate::dom::htmlformelement::HTMLFormElement;
use crate::dom::input_type::text_input_widget::TextInputWidget;
use crate::dom::node::{Node, NodeTraits, UnbindContext};
use crate::dom::pointerevent::PointerEvent;
use crate::dom::types::MouseEvent;

#[derive(Default, JSTraceable, MallocSizeOf, PartialEq)]
#[cfg_attr(crown, crown::unrooted_must_root_lint::must_root)]
pub(crate) struct RangeInputType {
    shadow_tree: DomRefCell<Option<RangeInputShadowTree>>,

    /// The identifier of the pointer that is currently dragging the slider thumb.
    dragging_pointer_id: Cell<Option<i32>>,

    /// The distance along the inline axis between the pointer and the center of the thumb
    /// when the drag started.
    #[no_trace]
    grab_offset: Cell<Au>,

    /// The value the element had when the drag started. Used to tell whether the drag has
    /// to fire a `change` event once it ends, and which value to restore if it is cancelled.
    value_before_drag: Cell<f64>,
}

impl RangeInputType {
    /// Get the shadow tree for this [`HTMLInputElement`], if it is created and valid, otherwise
    /// recreate the shadow tree and return it.
    fn get_or_create_shadow_tree(
        &self,
        cx: &mut JSContext,
        input: &HTMLInputElement,
    ) -> Ref<'_, RangeInputShadowTree> {
        {
            if let Ok(shadow_tree) = Ref::filter_map(self.shadow_tree.borrow(), |shadow_tree| {
                shadow_tree.as_ref()
            }) {
                return shadow_tree;
            }
        }

        let element = input.upcast::<Element>();
        let shadow_root = element
            .shadow_root()
            .unwrap_or_else(|| element.attach_ua_shadow_root(cx, true));
        let shadow_root = shadow_root.upcast();
        *self.shadow_tree.borrow_mut() = Some(RangeInputShadowTree::new(cx, shadow_root));
        self.get_or_create_shadow_tree(cx, input)
    }

    /// Start dragging the thumb with the pointer that just went down.
    fn handle_pointer_down(
        &self,
        cx: &mut JSContext,
        input: &HTMLInputElement,
        event: &PointerEvent,
    ) -> ValueChangeEvents {
        // Only the primary pointer pressing the primary button drags the thumb, and only
        // one drag can be in flight at a time.
        if !event.IsPrimary() ||
            event.upcast::<MouseEvent>().button() != MouseButton::Primary ||
            self.dragging_pointer_id.get().is_some() ||
            input.upcast::<Element>().disabled_state()
        {
            return ValueChangeEvents::empty();
        }

        // Don't capture the pointer for a widget that has no box to drag a thumb along.
        let Some(axis) = SliderAxis::new(input) else {
            return ValueChangeEvents::empty();
        };

        // Capture the pointer to allow moving the thumb even when leaving the
        // input box. The capture is released implicitly on `pointerup`.
        let pointer_id = event.PointerId();
        if input
            .upcast::<Element>()
            .SetPointerCapture(pointer_id)
            .is_err()
        {
            return ValueChangeEvents::empty();
        }

        self.dragging_pointer_id.set(Some(pointer_id));
        self.value_before_drag.set(input.ValueAsNumber());
        self.grab_offset.set(self.compute_grab_offset(&axis, event));

        self.update_value_from_pointer(cx, input, event, &axis)
    }

    /// End the drag and commit whichever value the thumb was left at.
    fn commit_drag(&self, input: &HTMLInputElement) -> ValueChangeEvents {
        self.dragging_pointer_id.set(None);

        if input.ValueAsNumber() == self.value_before_drag.get() {
            ValueChangeEvents::empty()
        } else {
            ValueChangeEvents::Change
        }
    }

    /// Abandon the drag
    fn cancel_drag(&self, cx: &mut JSContext, input: &HTMLInputElement) -> ValueChangeEvents {
        self.dragging_pointer_id.set(None);

        let value_before_drag = self.value_before_drag.get();
        if input.ValueAsNumber() == value_before_drag {
            return ValueChangeEvents::empty();
        }
        let _ = input.SetValueAsNumber(cx, value_before_drag);
        ValueChangeEvents::Input
    }

    /// Move the thumb to `event`'s position and update the element's value to match.
    fn update_value_from_pointer(
        &self,
        cx: &mut JSContext,
        input: &HTMLInputElement,
        event: &PointerEvent,
        axis: &SliderAxis,
    ) -> ValueChangeEvents {
        let offset = axis.offset_of(client_point(event)) - self.grab_offset.get();
        let old_value = input.ValueAsNumber();
        // Setting the value runs the value sanitization algorithm, which is what snaps the
        // value the pointer landed on to the closest allowed step.
        let _ = input.SetValueAsNumber(cx, axis.value_at(offset));

        if input.ValueAsNumber() == old_value {
            ValueChangeEvents::empty()
        } else {
            ValueChangeEvents::Input
        }
    }

    /// Continue a drag already in flight: re-read the geometry without forcing a layout and
    /// move the thumb to the pointer.
    fn continue_drag(
        &self,
        cx: &mut JSContext,
        input: &HTMLInputElement,
        event: &PointerEvent,
    ) -> ValueChangeEvents {
        let Some(axis) = SliderAxis::new_without_reflow(input) else {
            return ValueChangeEvents::empty();
        };
        self.update_value_from_pointer(cx, input, event, &axis)
    }

    /// The distance to keep between the pointer and the center of the thumb for the whole
    /// duration of a drag.
    fn compute_grab_offset(&self, axis: &SliderAxis, event: &PointerEvent) -> Au {
        let Some(thumb) = self
            .shadow_tree
            .borrow()
            .as_ref()
            .map(|shadow_tree| DomRoot::from_ref(&*shadow_tree.slider_thumb))
        else {
            return Au::zero();
        };

        let Some(thumb_box) = thumb.upcast::<Node>().border_box_without_reflow() else {
            return Au::zero();
        };

        let client_point = client_point(event);
        if !thumb_box.contains(client_point) {
            return Au::zero();
        }
        let thumb_center = (axis.offset_of(thumb_box.min()) + axis.offset_of(thumb_box.max())) / 2;
        axis.offset_of(client_point) - thumb_center
    }
}

impl SpecificInputType for RangeInputType {
    fn text_input_widget(&self) -> Option<&DomRefCell<TextInputWidget>> {
        None
    }

    /// <https://html.spec.whatwg.org/multipage/#range-state-(type=range):value-sanitization-algorithm>
    fn sanitize_value(&self, input: &HTMLInputElement, value: &mut DOMString) {
        if !value.is_valid_floating_point_number_string() {
            *value = DOMString::from(input.default_range_value().to_string());
        }
        if let Ok(fval) = &value.parse::<f64>() {
            let mut fval = *fval;
            // comparing max first, because if they contradict
            // the spec wants min to be the one that applies
            if let Some(max) = input.maximum() &&
                fval > max
            {
                fval = max;
            }
            if let Some(min) = input.minimum() &&
                fval < min
            {
                fval = min;
            }
            // https://html.spec.whatwg.org/multipage/#range-state-(type=range):suffering-from-a-step-mismatch
            // Spec does not describe this in a way that lends itself to
            // reproducible handling of floating-point rounding;
            // Servo may fail a WPT test because .1 * 6 == 6.000000000000001
            if let Some(allowed_value_step) = input.allowed_value_step() {
                let step_base = input.step_base();
                let steps_from_base = (fval - step_base) / allowed_value_step;
                if steps_from_base.fract() != 0.0 {
                    // not an integer number of steps, there's a mismatch
                    // round the number of steps...
                    let int_steps = round_halves_positive(steps_from_base);
                    // and snap the value to that rounded value...
                    fval = int_steps * allowed_value_step + step_base;

                    // but if after snapping we're now outside min..max
                    // we have to adjust! (adjusting to min last because
                    // that "wins" over max in the spec)
                    if let Some(stepped_maximum) = input.stepped_maximum() &&
                        fval > stepped_maximum
                    {
                        fval = stepped_maximum;
                    }
                    if let Some(stepped_minimum) = input.stepped_minimum() &&
                        fval < stepped_minimum
                    {
                        fval = stepped_minimum;
                    }
                }
            }
            *value = DOMString::from(fval.to_string());
        };
    }

    /// <https://html.spec.whatwg.org/multipage/#range-state-(type=range):concept-input-value-string-number>
    fn convert_string_to_number(&self, input: &str) -> Option<f64> {
        parse_floating_point_number(input)
    }

    /// <https://html.spec.whatwg.org/multipage/#range-state-(type=range):concept-input-value-string-number>
    fn convert_number_to_string(&self, input: f64) -> Option<DOMString> {
        let mut value = DOMString::from(input.to_string());
        value.set_best_representation_of_the_floating_point_number();
        Some(value)
    }

    /// <https://html.spec.whatwg.org/multipage/#range-state-(type=range):suffering-from-bad-input>
    fn suffers_from_bad_input(&self, value: &DOMString) -> bool {
        !value.is_valid_floating_point_number_string()
    }

    fn update_shadow_tree(&self, cx: &mut JSContext, input: &HTMLInputElement) {
        self.get_or_create_shadow_tree(cx, input).update(cx, input)
    }

    /// The user changes the value of a range input by dragging its thumb. Pointer
    /// capture keeps a drag alive while the pointer is outside of the widget.
    ///
    /// <https://html.spec.whatwg.org/multipage/#range-state-(type=range)>
    fn handle_event(
        &self,
        cx: &mut JSContext,
        input: &HTMLInputElement,
        event: &Event,
    ) -> ValueChangeEvents {
        let event_type = event.type_();

        // Dragging the thumb with a finger must not scroll the page as well.
        if self.dragging_pointer_id.get().is_some() && &*event_type == "touchmove" {
            event.mark_as_handled();
            return ValueChangeEvents::empty();
        }

        let Some(pointer_event) = event.downcast::<PointerEvent>() else {
            return ValueChangeEvents::empty();
        };

        if &*event_type == "pointerdown" {
            return self.handle_pointer_down(cx, input, pointer_event);
        }

        if self.dragging_pointer_id.get() != Some(pointer_event.PointerId()) {
            return ValueChangeEvents::empty();
        }

        match &*event_type {
            "pointermove" => self.continue_drag(cx, input, pointer_event),
            "pointerup" => self.continue_drag(cx, input, pointer_event) | self.commit_drag(input),
            "pointercancel" => self.cancel_drag(cx, input),
            "lostpointercapture" => self.commit_drag(input),
            _ => ValueChangeEvents::empty(),
        }
    }

    fn unbind_from_tree(
        &self,
        _cx: &mut JSContext,
        _input: &HTMLInputElement,
        _form_owner: Option<DomRoot<HTMLFormElement>>,
        _context: &UnbindContext,
    ) {
        self.dragging_pointer_id.set(None);
    }
}

fn round_halves_positive(n: f64) -> f64 {
    // WHATWG specs about input steps say to round to the nearest step,
    // rounding halves always to positive infinity.
    // This differs from Rust's .round() in the case of -X.5.
    if n.fract() == -0.5 {
        n.ceil()
    } else {
        n.round()
    }
}

/// The position of a [`PointerEvent`] in the same coordinate space as the boxes returned by
/// layout queries.
fn client_point(event: &PointerEvent) -> Point2D<Au, CSSPixel> {
    event.upcast::<MouseEvent>().client_point().map(Au::from_px)
}

/// The `min` and `max` of a range input, which always both have a default.
fn range_bounds(input: &HTMLInputElement) -> (f64, f64) {
    let expectation = "This value should be available for range input.";
    (
        input.minimum().expect(expectation),
        input.maximum().expect(expectation),
    )
}

/// The axis a slider's thumb travels along, in client coordinates, together with the value
/// domain it maps onto.
struct SliderAxis {
    /// The client coordinate of the inline start edge of the slider, on whichever physical
    /// axis `is_vertical` selects.
    inline_start: Au,

    /// The length of the slider along its inline axis. Always greater than zero.
    length: Au,

    /// Whether the inline axis is the vertical one.
    is_vertical: bool,

    /// `1` when the inline axis grows with that coordinate, `-1` when it grows against it.
    direction: i32,

    /// The `min` and `max` the axis maps onto, read once so that a drag does not re-parse
    /// the attributes on every move.
    bounds: (f64, f64),
}

impl SliderAxis {
    /// The axis at the start of a drag, flushing layout so the geometry is current.
    fn new(input: &HTMLInputElement) -> Option<Self> {
        Self::from_padding_box(input, input.upcast::<Node>().padding_box()?)
    }

    /// The axis for the moves that follow, without forcing a layout.
    fn new_without_reflow(input: &HTMLInputElement) -> Option<Self> {
        Self::from_padding_box(input, input.upcast::<Node>().padding_box_without_reflow()?)
    }

    fn from_padding_box(input: &HTMLInputElement, padding_box: Rect<Au, CSSPixel>) -> Option<Self> {
        // The caller has just queried a box, so the style data is current either way.
        let inline_start_side = input
            .upcast::<Element>()
            .style_without_reflow()?
            .writing_mode
            .inline_start_physical_side();

        let (inline_start, length, is_vertical, direction) = match inline_start_side {
            PhysicalSide::Left => (padding_box.min_x(), padding_box.width(), false, 1),
            PhysicalSide::Right => (padding_box.max_x(), padding_box.width(), false, -1),
            PhysicalSide::Top => (padding_box.min_y(), padding_box.height(), true, 1),
            PhysicalSide::Bottom => (padding_box.max_y(), padding_box.height(), true, -1),
        };

        (length > Au::zero()).then_some(Self {
            inline_start,
            length,
            is_vertical,
            direction,
            bounds: range_bounds(input),
        })
    }

    /// How far `point` is from the inline start edge of the slider. Negative when the point
    /// is before that edge, larger than the slider's length when it is past its inline end.
    fn offset_of(&self, point: Point2D<Au, CSSPixel>) -> Au {
        let coordinate = if self.is_vertical { point.y } else { point.x };
        (coordinate - self.inline_start) * self.direction
    }

    /// The value a thumb centered `offset` away from the inline start edge represents.
    fn value_at(&self, offset: Au) -> f64 {
        let (min, max) = self.bounds;
        if max <= min {
            return min;
        }

        let fraction = (offset.to_f64_px() / self.length.to_f64_px()).clamp(0.0, 1.0);
        min + fraction * (max - min)
    }
}

#[derive(Clone, JSTraceable, MallocSizeOf, PartialEq)]
#[cfg_attr(crown, crown::unrooted_must_root_lint::must_root)]
/// Contains references to the elements in the shadow tree for `<input type=range>`.
///
/// The shadow tree consists of three div in the following structure:
/// <input type=range>
/// ├─ ::slider-track
/// │ └─ ::slider-fill
/// └─ ::slider-thumb
pub(crate) struct RangeInputShadowTree {
    slider_fill: Dom<Element>,
    slider_thumb: Dom<Element>,
    slider_track: Dom<Element>,

    /// How far along the slider the thumb was last placed, so that an update that resolves
    /// to the same position can skip updating the inline styles below.
    /// `NaN` until the first update, so that it never compares equal.
    last_percent: Cell<f64>,
}

impl RangeInputShadowTree {
    pub(crate) fn new(cx: &mut JSContext, shadow_root: &Node) -> Self {
        Node::replace_all(cx, None, shadow_root.upcast::<Node>());

        let slider_fill = Element::create(
            cx,
            QualName::new(None, ns!(html), local_name!("div")),
            None,
            &shadow_root.owner_document(),
            ElementCreator::ScriptCreated,
            CustomElementCreationMode::Asynchronous,
            None,
        );

        let slider_thumb = Element::create(
            cx,
            QualName::new(None, ns!(html), local_name!("div")),
            None,
            &shadow_root.owner_document(),
            ElementCreator::ScriptCreated,
            CustomElementCreationMode::Asynchronous,
            None,
        );

        let slider_track = Element::create(
            cx,
            QualName::new(None, ns!(html), local_name!("div")),
            None,
            &shadow_root.owner_document(),
            ElementCreator::ScriptCreated,
            CustomElementCreationMode::Asynchronous,
            None,
        );

        shadow_root
            .upcast::<Node>()
            .AppendChild(cx, slider_track.upcast::<Node>())
            .unwrap();
        slider_track
            .upcast::<Node>()
            .AppendChild(cx, slider_fill.upcast::<Node>())
            .unwrap();
        shadow_root
            .upcast::<Node>()
            .AppendChild(cx, slider_thumb.upcast::<Node>())
            .unwrap();

        slider_fill
            .upcast::<Node>()
            .set_implemented_pseudo_element(PseudoElement::SliderFill);
        slider_thumb
            .upcast::<Node>()
            .set_implemented_pseudo_element(PseudoElement::SliderThumb);
        slider_track
            .upcast::<Node>()
            .set_implemented_pseudo_element(PseudoElement::SliderTrack);

        Self {
            slider_fill: slider_fill.as_traced(),
            slider_thumb: slider_thumb.as_traced(),
            slider_track: slider_track.as_traced(),
            last_percent: Cell::new(f64::NAN),
        }
    }

    pub(crate) fn update(&self, cx: &mut JSContext, input_element: &HTMLInputElement) {
        let value = input_element.Value();
        let (min, max) = range_bounds(input_element);
        let value_num = input_element
            .convert_string_to_number(&value.str())
            .unwrap_or(input_element.default_range_value());

        let percent = if min > max || (max - min).abs() < f64::EPSILON {
            0.0
        } else {
            let clamped_value = value_num.clamp(min, max);
            (clamped_value - min) / (max - min) * 100.0
        };

        if self.last_percent.replace(percent) == percent {
            return;
        }

        self.slider_thumb.set_attribute(
            cx,
            &local_name!("style"),
            AttrValue::String(format!("inset-inline-start: {percent}% !important;")),
        );
        self.slider_fill.set_attribute(
            cx,
            &local_name!("style"),
            AttrValue::String(format!("width: {percent}% !important;")),
        );
    }
}
