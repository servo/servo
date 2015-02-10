/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use compositor_task::LayerProperties;
use compositor::IOCompositor;
use windowing::{MouseWindowEvent, WindowMethods};

use azure::azure_hl;
use geom::length::Length;
use geom::matrix::identity;
use geom::point::{Point2D, TypedPoint2D};
use geom::size::TypedSize2D;
use geom::rect::Rect;
use gfx::paint_task::Msg as PaintMsg;
use layers::color::Color;
use layers::geometry::LayerPixel;
use layers::layers::{Layer, LayerBufferSet};
use script_traits::CompositorEvent::{ClickEvent, MouseDownEvent, MouseMoveEvent, MouseUpEvent};
use script_traits::{ScriptControlChan, ConstellationControlMsg};
use msg::compositor_msg::{Epoch, LayerId, ScrollPolicy};
use msg::constellation_msg::PipelineId;
use std::num::Float;
use std::rc::Rc;

pub struct CompositorData {
    /// This layer's pipeline id. The compositor can associate this id with an
    /// actual CompositionPipeline.
    pub pipeline_id: PipelineId,

    /// The ID of this layer within the pipeline.
    pub id: LayerId,

    /// The behavior of this layer when a scroll message is received.
    pub wants_scroll_events: WantsScrollEventsFlag,

    /// Whether an ancestor layer that receives scroll events moves this layer.
    pub scroll_policy: ScrollPolicy,

    /// A monotonically increasing counter that keeps track of the current epoch.
    /// add_buffer() calls that don't match the current epoch will be ignored.
    pub epoch: Epoch,

    /// The scroll offset originating from this scrolling root. This allows scrolling roots
    /// to track their current scroll position even while their content_offset does not change.
    pub scroll_offset: TypedPoint2D<LayerPixel, f32>,
}

impl CompositorData {
    pub fn new_layer(layer_properties: LayerProperties,
                     wants_scroll_events: WantsScrollEventsFlag,
                     tile_size: uint)
                     -> Rc<Layer<CompositorData>> {
        let new_compositor_data = CompositorData {
            pipeline_id: layer_properties.pipeline_id,
            id: layer_properties.id,
            wants_scroll_events: wants_scroll_events,
            scroll_policy: layer_properties.scroll_policy,
            epoch: layer_properties.epoch,
            scroll_offset: TypedPoint2D(0., 0.),
        };

        Rc::new(Layer::new(Rect::from_untyped(&layer_properties.rect),
                           tile_size,
                           to_layers_color(&layer_properties.background_color),
                           new_compositor_data))
    }
}

pub trait CompositorLayer {
    fn update_layer_except_bounds(&self, layer_properties: LayerProperties);

    fn update_layer(&self, layer_properties: LayerProperties);

    fn add_buffers<Window>(&self,
                           compositor: &IOCompositor<Window>,
                           new_buffers: Box<LayerBufferSet>,
                           epoch: Epoch)
                           -> bool
                           where Window: WindowMethods;

    /// Destroys all layer tiles, sending the buffers back to the painter to be destroyed or
    /// reused.
    fn clear<Window>(&self, compositor: &IOCompositor<Window>) where Window: WindowMethods;

    /// Destroys tiles for this layer and all descendent layers, sending the buffers back to the
    /// painter to be destroyed or reused.
    fn clear_all_tiles<Window>(&self, compositor: &IOCompositor<Window>) where Window: WindowMethods;

    /// Destroys all tiles of all layers, including children, *without* sending them back to the
    /// painter. You must call this only when the paint task is destined to be going down;
    /// otherwise, you will leak tiles.
    ///
    /// This is used during shutdown, when we know the paint task is going away.
    fn forget_all_tiles(&self);

    /// Move the layer's descendants that don't want scroll events and scroll by a relative
    /// specified amount in page coordinates. This also takes in a cursor position to see if the
    /// mouse is over child layers first. If a layer successfully scrolled returns either
    /// ScrollPositionUnchanged or ScrollPositionChanged. If no layer was targeted by the event
    /// returns ScrollEventUnhandled.
    fn handle_scroll_event(&self,
                           delta: TypedPoint2D<LayerPixel, f32>,
                           cursor: TypedPoint2D<LayerPixel, f32>)
                           -> ScrollEventResult;

    // Takes in a MouseWindowEvent, determines if it should be passed to children, and
    // sends the event off to the appropriate pipeline. NB: the cursor position is in
    // page coordinates.
    fn send_mouse_event<Window>(&self,
                                compositor: &IOCompositor<Window>,
                                event: MouseWindowEvent,
                                cursor: TypedPoint2D<LayerPixel, f32>)
                                where Window: WindowMethods;

    fn send_mouse_move_event<Window>(&self,
                                     compositor: &IOCompositor<Window>,
                                     cursor: TypedPoint2D<LayerPixel, f32>)
                                     where Window: WindowMethods;

    fn clamp_scroll_offset_and_scroll_layer(&self,
                                            new_offset: TypedPoint2D<LayerPixel, f32>)
                                            -> ScrollEventResult;

    fn scroll_layer_and_all_child_layers(&self,
                                         new_offset: TypedPoint2D<LayerPixel, f32>)
                                         -> bool;

    /// Return a flag describing how this layer deals with scroll events.
    fn wants_scroll_events(&self) -> WantsScrollEventsFlag;

    /// Return the pipeline id associated with this layer.
    fn get_pipeline_id(&self) -> PipelineId;
}

#[derive(Copy, PartialEq, Clone)]
pub enum WantsScrollEventsFlag {
    WantsScrollEvents,
    DoesntWantScrollEvents,
}

fn to_layers_color(color: &azure_hl::Color) -> Color {
    Color { r: color.r, g: color.g, b: color.b, a: color.a }
}

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

fn calculate_content_size_for_layer(layer: &Layer<CompositorData>)
                                    -> TypedSize2D<LayerPixel, f32> {
    layer.children().iter().fold(Rect::zero(),
                                 |unioned_rect, child_rect| {
                                    unioned_rect.union(&*child_rect.bounds.borrow())
                                 }).size
}

#[derive(PartialEq)]
pub enum ScrollEventResult {
    ScrollEventUnhandled,
    ScrollPositionChanged,
    ScrollPositionUnchanged,
}

impl CompositorLayer for Layer<CompositorData> {
    fn update_layer_except_bounds(&self, layer_properties: LayerProperties) {
        self.extra_data.borrow_mut().epoch = layer_properties.epoch;
        self.extra_data.borrow_mut().scroll_policy = layer_properties.scroll_policy;

        *self.background_color.borrow_mut() = to_layers_color(&layer_properties.background_color);

        self.contents_changed();
    }

    fn update_layer(&self, layer_properties: LayerProperties) {
        *self.bounds.borrow_mut() = Rect::from_untyped(&layer_properties.rect);

        // Call scroll for bounds checking if the page shrunk. Use (-1, -1) as the
        // cursor position to make sure the scroll isn't propagated downwards.
        self.handle_scroll_event(TypedPoint2D(0f32, 0f32), TypedPoint2D(-1f32, -1f32));
        self.update_layer_except_bounds(layer_properties);
    }

    // Add LayerBuffers to the specified layer. Returns the layer buffer set back if the layer that
    // matches the given pipeline ID was not found; otherwise returns None and consumes the layer
    // buffer set.
    //
    // If the epoch of the message does not match the layer's epoch, the message is ignored, the
    // layer buffer set is consumed, and None is returned.
    fn add_buffers<Window>(&self,
                           compositor: &IOCompositor<Window>,
                           new_buffers: Box<LayerBufferSet>,
                           epoch: Epoch)
                           -> bool
                           where Window: WindowMethods {
        if self.extra_data.borrow().epoch != epoch {
            debug!("add_buffers: compositor epoch mismatch: {:?} != {:?}, id: {:?}",
                   self.extra_data.borrow().epoch,
                   epoch,
                   self.get_pipeline_id());
            let pipeline = compositor.get_pipeline(self.get_pipeline_id());
            let _ = pipeline.paint_chan.send(PaintMsg::UnusedBuffer(new_buffers.buffers));
            return false;
        }

        for buffer in new_buffers.buffers.into_iter().rev() {
            self.add_buffer(buffer);
        }

        let unused_buffers = self.collect_unused_buffers();
        if !unused_buffers.is_empty() { // send back unused buffers
            let pipeline = compositor.get_pipeline(self.get_pipeline_id());
            let _ = pipeline.paint_chan.send(PaintMsg::UnusedBuffer(unused_buffers));
        }

        return true;
    }

    fn clear<Window>(&self, compositor: &IOCompositor<Window>) where Window: WindowMethods {
        let mut buffers = self.collect_buffers();

        if !buffers.is_empty() {
            // We have no way of knowing without a race whether the paint task is even up and
            // running, but mark the buffers as not leaking. If the paint task died, then the
            // buffers are going to be cleaned up.
            for buffer in buffers.iter_mut() {
                buffer.mark_wont_leak()
            }

            let pipeline = compositor.get_pipeline(self.get_pipeline_id());
            let _ = pipeline.paint_chan.send(PaintMsg::UnusedBuffer(buffers));
        }
    }

    /// Destroys tiles for this layer and all descendent layers, sending the buffers back to the
    /// painter to be destroyed or reused.
    fn clear_all_tiles<Window>(&self,
                               compositor: &IOCompositor<Window>)
                               where Window: WindowMethods {
        self.clear(compositor);
        for kid in self.children().iter() {
            kid.clear_all_tiles(compositor);
        }
    }

    /// Destroys all tiles of all layers, including children, *without* sending them back to the
    /// painter. You must call this only when the paint task is destined to be going down;
    /// otherwise, you will leak tiles.
    ///
    /// This is used during shutdown, when we know the paint task is going away.
    fn forget_all_tiles(&self) {
        let tiles = self.collect_buffers();
        for tile in tiles.into_iter() {
            let mut tile = tile;
            tile.mark_wont_leak()
        }

        for kid in self.children().iter() {
            kid.forget_all_tiles();
        }
    }

    fn handle_scroll_event(&self,
                           delta: TypedPoint2D<LayerPixel, f32>,
                           cursor: TypedPoint2D<LayerPixel, f32>)
                           -> ScrollEventResult {
        // If this layer doesn't want scroll events, neither it nor its children can handle scroll
        // events.
        if self.wants_scroll_events() != WantsScrollEventsFlag::WantsScrollEvents {
            return ScrollEventResult::ScrollEventUnhandled;
        }

        //// Allow children to scroll.
        let scroll_offset = self.extra_data.borrow().scroll_offset;
        let new_cursor = cursor - scroll_offset;
        for child in self.children().iter() {
            let child_bounds = child.bounds.borrow();
            if child_bounds.contains(&new_cursor) {
                let result = child.handle_scroll_event(delta, new_cursor - child_bounds.origin);
                if result != ScrollEventResult::ScrollEventUnhandled {
                    return result;
                }
            }
        }

        self.clamp_scroll_offset_and_scroll_layer(scroll_offset + delta)
    }

    fn clamp_scroll_offset_and_scroll_layer(&self, new_offset: TypedPoint2D<LayerPixel, f32>)
                                            -> ScrollEventResult {
        let layer_size = self.bounds.borrow().size;
        let content_size = calculate_content_size_for_layer(self);
        let min_x = (layer_size.width - content_size.width).get().min(0.0);
        let min_y = (layer_size.height - content_size.height).get().min(0.0);
        let new_offset : TypedPoint2D<LayerPixel, f32> =
            Point2D(Length(new_offset.x.get().clamp(&min_x, &0.0)),
                    Length(new_offset.y.get().clamp(&min_y, &0.0)));

        if self.extra_data.borrow().scroll_offset == new_offset {
            return ScrollEventResult::ScrollPositionUnchanged;
        }

        // The scroll offset is just a record of the scroll position of this scrolling root,
        // but scroll_layer_and_all_child_layers actually moves the child layers.
        self.extra_data.borrow_mut().scroll_offset = new_offset;

        let mut result = false;
        for child in self.children().iter() {
            result |= child.scroll_layer_and_all_child_layers(new_offset);
        }

        if result {
            return ScrollEventResult::ScrollPositionChanged;
        } else {
            return ScrollEventResult::ScrollPositionUnchanged;
        }
    }

    fn send_mouse_event<Window>(&self,
                                compositor: &IOCompositor<Window>,
                                event: MouseWindowEvent,
                                cursor: TypedPoint2D<LayerPixel, f32>)
                                where Window: WindowMethods {
        let event_point = cursor.to_untyped();
        let message = match event {
            MouseWindowEvent::Click(button, _) =>
                ClickEvent(button, event_point),
            MouseWindowEvent::MouseDown(button, _) =>
                MouseDownEvent(button, event_point),
            MouseWindowEvent::MouseUp(button, _) =>
                MouseUpEvent(button, event_point),
        };

        let pipeline = compositor.get_pipeline(self.get_pipeline_id());
        let ScriptControlChan(ref chan) = pipeline.script_chan;
        let _ = chan.send(ConstellationControlMsg::SendEvent(pipeline.id.clone(), message));
    }

    fn send_mouse_move_event<Window>(&self,
                                     compositor: &IOCompositor<Window>,
                                     cursor: TypedPoint2D<LayerPixel, f32>)
                                     where Window: WindowMethods {
        let message = MouseMoveEvent(cursor.to_untyped());
        let pipeline = compositor.get_pipeline(self.get_pipeline_id());
        let ScriptControlChan(ref chan) = pipeline.script_chan;
        let _ = chan.send(ConstellationControlMsg::SendEvent(pipeline.id.clone(), message));
    }

    fn scroll_layer_and_all_child_layers(&self, new_offset: TypedPoint2D<LayerPixel, f32>)
                                         -> bool {
        let mut result = false;

        // Only scroll this layer if it's not fixed-positioned.
        if self.extra_data.borrow().scroll_policy != ScrollPolicy::FixedPosition {
            let new_offset = new_offset.to_untyped();
            *self.transform.borrow_mut() = identity().translate(new_offset.x, new_offset.y, 0.0);
            *self.content_offset.borrow_mut() = Point2D::from_untyped(&new_offset);
            result = true
        }

        let offset_for_children = new_offset + self.extra_data.borrow().scroll_offset;
        for child in self.children().iter() {
            result |= child.scroll_layer_and_all_child_layers(offset_for_children);
        }

        return result;
    }

    fn wants_scroll_events(&self) -> WantsScrollEventsFlag {
        self.extra_data.borrow().wants_scroll_events
    }

    fn get_pipeline_id(&self) -> PipelineId {
        self.extra_data.borrow().pipeline_id
    }
}
