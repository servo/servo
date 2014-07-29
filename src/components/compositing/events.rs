/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use compositor_data::{CompositorData, WantsScrollEvents};
use windowing::{MouseWindowEvent, MouseWindowClickEvent, MouseWindowMouseDownEvent};
use windowing::MouseWindowMouseUpEvent;

use geom::length::Length;
use geom::matrix::identity;
use geom::point::{Point2D, TypedPoint2D};
use geom::rect::{Rect, TypedRect};
use geom::scale_factor::ScaleFactor;
use geom::size::TypedSize2D;
use layers::layers::Layer;
use script::dom::event::{ClickEvent, MouseDownEvent, MouseMoveEvent, MouseUpEvent};
use script::script_task::{ScriptChan, SendEventMsg};
use servo_msg::compositor_msg::{FixedPosition, LayerId};
use servo_msg::constellation_msg::PipelineId;
use servo_util::geometry::{DevicePixel, PagePx};
use std::rc::Rc;

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
                           delta: TypedPoint2D<PagePx, f32>,
                           cursor: TypedPoint2D<PagePx, f32>,
                           window_size: TypedSize2D<PagePx, f32>,
                           page_to_device_pixels_scale: ScaleFactor<PagePx, DevicePixel, f32>)
                           -> bool {
    // If this layer doesn't want scroll events, neither it nor its children can handle scroll
    // events.
    if layer.extra_data.borrow().wants_scroll_events != WantsScrollEvents {
        return false
    }

    // Allow children to scroll.
    let cursor = cursor - layer.extra_data.borrow().scroll_offset;
    for child in layer.children().iter() {
        let rect: TypedRect<PagePx, f32> = Rect::from_untyped(&*child.bounds.borrow());
        if rect.contains(&cursor) &&
           handle_scroll_event(child.clone(),
                               delta,
                               cursor - rect.origin,
                               rect.size,
                               page_to_device_pixels_scale) {
            return true
        }
    }

    // This scroll event is mine!
    // Scroll this layer!
    let old_origin = layer.extra_data.borrow().scroll_offset.clone();
    layer.extra_data.borrow_mut().scroll_offset = old_origin + delta;

    // bounds checking
    let page_size = layer.bounds.borrow().size;
    let window_size = window_size.to_untyped();
    let scroll_offset = layer.extra_data.borrow().scroll_offset.to_untyped();

    let min_x = (window_size.width - page_size.width).min(0.0);
    layer.extra_data.borrow_mut().scroll_offset.x = Length(scroll_offset.x.clamp(&min_x, &0.0));

    let min_y = (window_size.height - page_size.height).min(0.0);
    layer.extra_data.borrow_mut().scroll_offset.y = Length(scroll_offset.y.clamp(&min_y, &0.0));

    if old_origin - layer.extra_data.borrow().scroll_offset == TypedPoint2D(0f32, 0f32) {
        return false
    }

    let offset = layer.extra_data.borrow().scroll_offset.clone();
    scroll(layer.clone(), offset, page_to_device_pixels_scale)
}

/// Actually scrolls the descendants of a layer that scroll. This is called by
/// `handle_scroll_event` above when it determines that a layer wants to scroll.
fn scroll(layer: Rc<Layer<CompositorData>>,
          scroll_offset: TypedPoint2D<PagePx, f32>,
          page_to_device_pixels_scale: ScaleFactor<PagePx, DevicePixel, f32>)
          -> bool {
    let mut result = false;

    // Only scroll this layer if it's not fixed-positioned.
    if layer.extra_data.borrow().scroll_policy != FixedPosition {
        // Scroll this layer!
        layer.extra_data.borrow_mut().scroll_offset = scroll_offset;

        let scroll_offset = layer.extra_data.borrow().scroll_offset.clone();
        *layer.transform.borrow_mut() = identity().translate(scroll_offset.x.get(), scroll_offset.y.get(), 0.0);
        *layer.content_offset.borrow_mut() = (scroll_offset * page_to_device_pixels_scale).to_untyped();

        result = true
    }

    for child in layer.children().iter() {
        result = scroll(child.clone(), scroll_offset, page_to_device_pixels_scale) || result;
    }

    result
}

// Takes in a MouseWindowEvent, determines if it should be passed to children, and
// sends the event off to the appropriate pipeline. NB: the cursor position is in
// page coordinates.
pub fn send_mouse_event(layer: Rc<Layer<CompositorData>>,
                        event: MouseWindowEvent, cursor: TypedPoint2D<PagePx, f32>) {
    let cursor = cursor - layer.extra_data.borrow().scroll_offset;
    for child in layer.children().iter() {
        let rect: TypedRect<PagePx, f32> = Rect::from_untyped(&*child.bounds.borrow());
        if rect.contains(&cursor) {
            send_mouse_event(child.clone(), event, cursor - rect.origin);
            return;
        }
    }

    // This mouse event is mine!
    let message = match event {
        MouseWindowClickEvent(button, _) => ClickEvent(button, cursor.to_untyped()),
        MouseWindowMouseDownEvent(button, _) => MouseDownEvent(button, cursor.to_untyped()),
        MouseWindowMouseUpEvent(button, _) => MouseUpEvent(button, cursor.to_untyped()),
    };
    let ScriptChan(ref chan) = layer.extra_data.borrow().pipeline.script_chan;
    let _ = chan.send_opt(SendEventMsg(layer.extra_data.borrow().pipeline.id.clone(), message));
}

pub fn send_mouse_move_event(layer: Rc<Layer<CompositorData>>,
                             cursor: TypedPoint2D<PagePx, f32>) {
    let message = MouseMoveEvent(cursor.to_untyped());
    let ScriptChan(ref chan) = layer.extra_data.borrow().pipeline.script_chan;
    let _ = chan.send_opt(SendEventMsg(layer.extra_data.borrow().pipeline.id.clone(), message));
}

pub fn move(layer: Rc<Layer<CompositorData>>,
            pipeline_id: PipelineId,
            layer_id: LayerId,
            origin: Point2D<f32>,
            window_size: TypedSize2D<PagePx, f32>,
            page_to_device_pixels_scale: ScaleFactor<PagePx, DevicePixel, f32>)
            -> bool {
    // Search children for the right layer to move.
    if layer.extra_data.borrow().pipeline.id != pipeline_id ||
       layer.extra_data.borrow().id != layer_id {
        return layer.children().iter().any(|kid| {
            move(kid.clone(),
                 pipeline_id,
                 layer_id,
                 origin,
                 window_size,
                 page_to_device_pixels_scale)
        });
    }

    if layer.extra_data.borrow().wants_scroll_events != WantsScrollEvents {
        return false
    }

    // Scroll this layer!
    let old_origin = layer.extra_data.borrow().scroll_offset;
    layer.extra_data.borrow_mut().scroll_offset = Point2D::from_untyped(&(origin * -1.0));

    // bounds checking
    let page_size = layer.bounds.borrow().size;
    let window_size = window_size.to_untyped();
    let scroll_offset = layer.extra_data.borrow().scroll_offset.to_untyped();

    let min_x = (window_size.width - page_size.width).min(0.0);
    layer.extra_data.borrow_mut().scroll_offset.x = Length(scroll_offset.x.clamp(&min_x, &0.0));
    let min_y = (window_size.height - page_size.height).min(0.0);
    layer.extra_data.borrow_mut().scroll_offset.y = Length(scroll_offset.y.clamp(&min_y, &0.0));

    // check to see if we scrolled
    if old_origin - layer.extra_data.borrow().scroll_offset == TypedPoint2D(0f32, 0f32) {
        return false;
    }

    let offset = layer.extra_data.borrow().scroll_offset.clone();
    scroll(layer.clone(), offset, page_to_device_pixels_scale)
}
