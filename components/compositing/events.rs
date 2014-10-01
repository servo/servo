/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use compositor_data::{CompositorData, WantsScrollEvents};
use windowing::{MouseWindowEvent, MouseWindowClickEvent, MouseWindowMouseDownEvent};
use windowing::MouseWindowMouseUpEvent;

use geom::length::Length;
use geom::point::{Point2D, TypedPoint2D};
use geom::rect::Rect;
use geom::size::TypedSize2D;
use layers::geometry::LayerPixel;
use layers::layers::Layer;
use script_traits::{ClickEvent, MouseDownEvent, MouseMoveEvent, MouseUpEvent, SendEventMsg};
use script_traits::{ScriptControlChan};
use servo_msg::compositor_msg::FixedPosition;
use std::rc::Rc;


use geom::matrix::identity;

trait Clampable {
    fn clamp(&self, mn: &Self, mx: &Self) -> Self;
}

impl Clampable for f32 {
    /// Returns the number constrained within the range `mn <= self <= mx`.
    /// If any of the numbers are `NAN` then `NAN` is returned.
    #[inline]
    fn clamp(&self, mn: &f32, mx: &f32) -> f32 {
        match () {
            _ if self.is_nan()   => *self,
            _ if !(*self <= *mx) => *mx,
            _ if !(*self >= *mn) => *mn,
            _                    => *self,
        }
    }
}

#[deriving(PartialEq)]
pub enum ScrollEventResult {
    ScrollEventUnhandled,
    ScrollPositionChanged,
    ScrollPositionUnchanged,
}

/// Move the layer's descendants that don't want scroll events and scroll by a relative
/// specified amount in page coordinates. This also takes in a cursor position to see if the
/// mouse is over child layers first. If a layer successfully scrolled, returns true; otherwise
/// returns false, so a parent layer can scroll instead.
pub fn handle_scroll_event(layer: Rc<Layer<CompositorData>>,
                           delta: TypedPoint2D<LayerPixel, f32>,
                           cursor: TypedPoint2D<LayerPixel, f32>)
                           -> ScrollEventResult {
    // If this layer doesn't want scroll events, neither it nor its children can handle scroll
    // events.
    if layer.extra_data.borrow().wants_scroll_events != WantsScrollEvents {
        return ScrollEventUnhandled;
    }

    // Allow children to scroll.
    let scroll_offset = layer.extra_data.borrow().scroll_offset;
    let new_cursor = cursor - scroll_offset;
    for child in layer.children().iter() {
        let child_bounds = child.bounds.borrow();
        if child_bounds.contains(&new_cursor) {
            let result = handle_scroll_event(child.clone(),
                                             delta,
                                             new_cursor - child_bounds.origin);
            if result != ScrollEventUnhandled {
                return result;
            }
        }
    }

    clamp_scroll_offset_and_scroll_layer(layer, scroll_offset + delta)
}

pub fn calculate_content_size_for_layer(layer: Rc<Layer<CompositorData>>)
                                         -> TypedSize2D<LayerPixel, f32> {
    layer.children().iter().fold(Rect::zero(),
                                 |unioned_rect, child_rect| {
                                    unioned_rect.union(&*child_rect.bounds.borrow())
                                 }).size
}

pub fn clamp_scroll_offset_and_scroll_layer(layer: Rc<Layer<CompositorData>>,
                                            new_offset: TypedPoint2D<LayerPixel, f32>)
                                            -> ScrollEventResult {
    let layer_size = layer.bounds.borrow().size;
    let content_size = calculate_content_size_for_layer(layer.clone());
    let min_x = (layer_size.width - content_size.width).get().min(0.0);
    let min_y = (layer_size.height - content_size.height).get().min(0.0);
    let new_offset : TypedPoint2D<LayerPixel, f32> =
        Point2D(Length(new_offset.x.get().clamp(&min_x, &0.0)),
                Length(new_offset.y.get().clamp(&min_y, &0.0)));

    if layer.extra_data.borrow().scroll_offset == new_offset {
        return ScrollPositionUnchanged;
    }

    // The scroll offset is just a record of the scroll position of this scrolling root,
    // but scroll_layer_and_all_child_layers actually moves the child layers.
    layer.extra_data.borrow_mut().scroll_offset = new_offset;

    let mut result = false;
    for child in layer.children().iter() {
        result |= scroll_layer_and_all_child_layers(child.clone(), new_offset);
    }

    if result {
        return ScrollPositionChanged;
    } else {
        return ScrollPositionUnchanged;
    }
}

fn scroll_layer_and_all_child_layers(layer: Rc<Layer<CompositorData>>,
                                     new_offset: TypedPoint2D<LayerPixel, f32>)
                                     -> bool {
    let mut result = false;

    // Only scroll this layer if it's not fixed-positioned.
    if layer.extra_data.borrow().scroll_policy != FixedPosition {
        let new_offset = new_offset.to_untyped();
        *layer.transform.borrow_mut() = identity().translate(new_offset.x,
                                                             new_offset.y,
                                                             0.0);
        *layer.content_offset.borrow_mut() = Point2D::from_untyped(&new_offset);
        result = true
    }

    let offset_for_children = new_offset + layer.extra_data.borrow().scroll_offset;
    for child in layer.children().iter() {
        result |= scroll_layer_and_all_child_layers(child.clone(), offset_for_children);
    }

    return result;
}

struct HitTestResult {
    layer: Rc<Layer<CompositorData>>,
    point: TypedPoint2D<LayerPixel, f32>,
}

pub fn find_topmost_layer_at_point(layer: Rc<Layer<CompositorData>>,
                                   point: TypedPoint2D<LayerPixel, f32>)
                                   -> Option<HitTestResult> {
    let child_point = point - layer.bounds.borrow().origin;
    for child in layer.children().iter().rev() {
        let result = find_topmost_layer_at_point(child.clone(), child_point);
        if result.is_some() {
            return result;
        }
    }

    let point = point - *layer.content_offset.borrow();
    if !layer.bounds.borrow().contains(&point) {
        return None;
    }

    return Some(HitTestResult { layer: layer, point: point });
}

// Takes in a MouseWindowEvent, determines if it should be passed to children, and
// sends the event off to the appropriate pipeline. NB: the cursor position is in
// page coordinates.
pub fn send_mouse_event(layer: Rc<Layer<CompositorData>>,
                        event: MouseWindowEvent,
                        cursor: TypedPoint2D<LayerPixel, f32>) {
    match find_topmost_layer_at_point(layer.clone(), cursor) {
        Some(result) => {
            let event_point = result.point.to_untyped();
            let message = match event {
                MouseWindowClickEvent(button, _) => ClickEvent(button, event_point),
                MouseWindowMouseDownEvent(button, _) => MouseDownEvent(button, event_point),
                MouseWindowMouseUpEvent(button, _) => MouseUpEvent(button, event_point),
            };
            let pipeline = &result.layer.extra_data.borrow().pipeline;
            let ScriptControlChan(ref chan) = pipeline.script_chan;
            let _ = chan.send_opt(SendEventMsg(pipeline.id.clone(), message));
        },
        None => {},
    }

}

pub fn send_mouse_move_event(layer: Rc<Layer<CompositorData>>,
                             cursor: TypedPoint2D<LayerPixel, f32>) {
    match find_topmost_layer_at_point(layer.clone(), cursor) {
        Some(result) => {
            let message = MouseMoveEvent(result.point.to_untyped());
            let pipeline = &result.layer.extra_data.borrow().pipeline;
            let ScriptControlChan(ref chan) = pipeline.script_chan;
            let _ = chan.send_opt(SendEventMsg(pipeline.id.clone(), message));
        },
        None => {},
    }
}

