/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use compositor_data::{CompositorData, WantsScrollEvents};
use windowing::{MouseWindowEvent, MouseWindowClickEvent, MouseWindowMouseDownEvent};
use windowing::MouseWindowMouseUpEvent;

use geom::length::Length;
use geom::point::{Point2D, TypedPoint2D};
use geom::scale_factor::ScaleFactor;
use geom::size::TypedSize2D;
use layers::geometry::DevicePixel;
use layers::layers::Layer;
use script_traits::{ClickEvent, MouseDownEvent, MouseMoveEvent, MouseUpEvent, SendEventMsg};
use script_traits::{ScriptControlChan};
use servo_msg::compositor_msg::FixedPosition;
use servo_util::geometry::PagePx;
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

/// Move the layer's descendants that don't want scroll events and scroll by a relative
/// specified amount in page coordinates. This also takes in a cursor position to see if the
/// mouse is over child layers first. If a layer successfully scrolled, returns true; otherwise
/// returns false, so a parent layer can scroll instead.
pub fn handle_scroll_event(layer: Rc<Layer<CompositorData>>,
                           delta: TypedPoint2D<DevicePixel, f32>,
                           cursor: TypedPoint2D<DevicePixel, f32>,
                           window_size: TypedSize2D<DevicePixel, f32>,
                           scale: ScaleFactor<PagePx, DevicePixel, f32>)
                           -> bool {
    // If this layer doesn't want scroll events, neither it nor its children can handle scroll
    // events.
    if layer.extra_data.borrow().wants_scroll_events != WantsScrollEvents {
        return false
    }

    // Allow children to scroll.
    let content_offset_in_page_pixels : TypedPoint2D<PagePx, f32> =
        Point2D::from_untyped(&*layer.content_offset.borrow());
    let content_offset : TypedPoint2D<DevicePixel, f32> = content_offset_in_page_pixels * scale;
    let new_cursor = cursor - content_offset;
    for child in layer.children().iter() {
        let child_bounds = child.bounds.borrow();
        if child_bounds.contains(&new_cursor) &&
           handle_scroll_event(child.clone(),
                               delta,
                               new_cursor - child_bounds.origin,
                               child_bounds.size,
                               scale) {
            return true
        }
    }

    clamp_scroll_offset_and_scroll_layer(layer, content_offset + delta, window_size, scale)

}

pub fn clamp_scroll_offset_and_scroll_layer(layer: Rc<Layer<CompositorData>>,
                                            new_offset: TypedPoint2D<DevicePixel, f32>,
                                            window_size: TypedSize2D<DevicePixel, f32>,
                                            scale: ScaleFactor<PagePx, DevicePixel, f32>)
                                            -> bool {
    let layer_size = layer.bounds.borrow().size;
    let min_x = (window_size.width - layer_size.width).get().min(0.0);
    let min_y = (window_size.height - layer_size.height).get().min(0.0);
    let new_offset : TypedPoint2D<DevicePixel, f32> =
        Point2D(Length(new_offset.x.get().clamp(&min_x, &0.0)),
                Length(new_offset.y.get().clamp(&min_y, &0.0)));

    let new_offset_in_page_px = new_offset / scale;
    let untyped_new_offset = new_offset_in_page_px.to_untyped();
    if *layer.content_offset.borrow() == untyped_new_offset {
        return false
    }

    // FIXME: This allows the base layer to record the current content offset without
    // updating its transform. This should be replaced with something less strange.
    *layer.content_offset.borrow_mut() = untyped_new_offset;
    scroll_layer_and_all_child_layers(layer.clone(), new_offset_in_page_px)
}

fn scroll_layer_and_all_child_layers(layer: Rc<Layer<CompositorData>>,
                                     new_offset: TypedPoint2D<PagePx, f32>)
                                     -> bool {
    let mut result = false;

    // Only scroll this layer if it's not fixed-positioned.
    if layer.extra_data.borrow().scroll_policy != FixedPosition {
        let new_offset = new_offset.to_untyped();
        *layer.transform.borrow_mut() = identity().translate(new_offset.x,
                                                             new_offset.y,
                                                             0.0);
        *layer.content_offset.borrow_mut() = new_offset;
        result = true
    }

    for child in layer.children().iter() {
        result |= scroll_layer_and_all_child_layers(child.clone(), new_offset);
    }

    return result;
}

// Takes in a MouseWindowEvent, determines if it should be passed to children, and
// sends the event off to the appropriate pipeline. NB: the cursor position is in
// page coordinates.
pub fn send_mouse_event(layer: Rc<Layer<CompositorData>>,
                        event: MouseWindowEvent,
                        cursor: TypedPoint2D<DevicePixel, f32>,
                        device_pixels_per_page_px: ScaleFactor<PagePx, DevicePixel, f32>) {
    let content_offset = *layer.content_offset.borrow() * device_pixels_per_page_px.get();
    let content_offset : TypedPoint2D<DevicePixel, f32> = Point2D::from_untyped(&content_offset);
    let cursor = cursor - content_offset;
    for child in layer.children().iter() {
        let child_bounds = child.bounds.borrow();
        if child_bounds.contains(&cursor) {
            send_mouse_event(child.clone(),
                             event,
                             cursor - child_bounds.origin,
                             device_pixels_per_page_px);
            return;
        }
    }

    // This mouse event is mine!
    let cursor = cursor / device_pixels_per_page_px;
    let message = match event {
        MouseWindowClickEvent(button, _) => ClickEvent(button, cursor.to_untyped()),
        MouseWindowMouseDownEvent(button, _) => MouseDownEvent(button, cursor.to_untyped()),
        MouseWindowMouseUpEvent(button, _) => MouseUpEvent(button, cursor.to_untyped()),
    };
    let ScriptControlChan(ref chan) = layer.extra_data.borrow().pipeline.script_chan;
    let _ = chan.send_opt(SendEventMsg(layer.extra_data.borrow().pipeline.id.clone(), message));
}

pub fn send_mouse_move_event(layer: Rc<Layer<CompositorData>>,
                             cursor: TypedPoint2D<PagePx, f32>) {
    let message = MouseMoveEvent(cursor.to_untyped());
    let ScriptControlChan(ref chan) = layer.extra_data.borrow().pipeline.script_chan;
    let _ = chan.send_opt(SendEventMsg(layer.extra_data.borrow().pipeline.id.clone(), message));
}

