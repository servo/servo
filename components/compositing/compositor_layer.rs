/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use compositor::IOCompositor;
use windowing::{MouseWindowEvent, WindowMethods};

use azure::azure_hl;
use euclid::length::Length;
use euclid::point::{Point2D, TypedPoint2D};
use euclid::size::TypedSize2D;
use euclid::rect::Rect;
use layers::color::Color;
use layers::geometry::LayerPixel;
use layers::layers::{Layer, LayerBufferSet};
use script_traits::CompositorEvent::{ClickEvent, MouseDownEvent, MouseMoveEvent, MouseUpEvent};
use script_traits::ConstellationControlMsg;
use msg::compositor_msg::{Epoch, LayerId, LayerProperties, ScrollPolicy};
use msg::constellation_msg::PipelineId;
use std::rc::Rc;

#[derive(Debug)]
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

    /// The epoch that has been requested for this layer (via send_buffer_requests).
    pub requested_epoch: Epoch,

    /// The last accepted painted buffer for this layer (via assign_pained_buffers).
    pub painted_epoch: Epoch,

    /// The scroll offset originating from this scrolling root. This allows scrolling roots
    /// to track their current scroll position even while their content_offset does not change.
    pub scroll_offset: TypedPoint2D<LayerPixel, f32>,
}

impl CompositorData {
    pub fn new_layer(pipeline_id: PipelineId,
                     layer_properties: LayerProperties,
                     wants_scroll_events: WantsScrollEventsFlag,
                     tile_size: usize)
                     -> Rc<Layer<CompositorData>> {
        let new_compositor_data = CompositorData {
            pipeline_id: pipeline_id,
            id: layer_properties.id,
            wants_scroll_events: wants_scroll_events,
            scroll_policy: layer_properties.scroll_policy,
            requested_epoch: Epoch(0),
            painted_epoch: Epoch(0),
            scroll_offset: Point2D::typed(0., 0.),
        };

        Rc::new(Layer::new(Rect::from_untyped(&layer_properties.rect),
                           tile_size,
                           to_layers_color(&layer_properties.background_color),
                           1.0,
                           layer_properties.establishes_3d_context,
                           new_compositor_data))
    }
}

pub trait CompositorLayer {
    fn update_layer_except_bounds(&self, layer_properties: LayerProperties);

    fn update_layer(&self, layer_properties: LayerProperties);

    fn add_buffers<Window>(&self,
                           compositor: &mut IOCompositor<Window>,
                           new_buffers: Box<LayerBufferSet>,
                           epoch: Epoch)
                           where Window: WindowMethods;

    /// Destroys all layer tiles, sending the buffers back to the painter to be destroyed or
    /// reused.
    fn clear<Window>(&self, compositor: &mut IOCompositor<Window>) where Window: WindowMethods;

    /// Destroys tiles for this layer and all descendent layers, sending the buffers back to the
    /// painter to be destroyed or reused.
    fn clear_all_tiles<Window>(&self, compositor: &mut IOCompositor<Window>)
                               where Window: WindowMethods;

    /// Removes the root layer (and any children) for a given pipeline from the
    /// compositor. Buffers that the compositor is holding are returned to the
    /// owning paint task.
    fn remove_root_layer_with_pipeline_id<Window>(&self,
                                                  compositor: &mut IOCompositor<Window>,
                                                  pipeline_id: PipelineId)
                                                  where Window: WindowMethods;

    /// Traverses the existing layer hierarchy and removes any layers that
    /// currently exist but which are no longer required.
    fn collect_old_layers<Window>(&self,
                                  compositor: &mut IOCompositor<Window>,
                                  pipeline_id: PipelineId,
                                  new_layers: &Vec<LayerProperties>)
                                  where Window: WindowMethods;

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
    fn pipeline_id(&self) -> PipelineId;
}

#[derive(Copy, PartialEq, Clone, Debug)]
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
        self.extra_data.borrow_mut().scroll_policy = layer_properties.scroll_policy;
        *self.transform.borrow_mut() = layer_properties.transform;
        *self.perspective.borrow_mut() = layer_properties.perspective;

        *self.background_color.borrow_mut() = to_layers_color(&layer_properties.background_color);

        self.contents_changed();
    }

    fn update_layer(&self, layer_properties: LayerProperties) {
        *self.bounds.borrow_mut() = Rect::from_untyped(&layer_properties.rect);

        // Call scroll for bounds checking if the page shrunk. Use (-1, -1) as the
        // cursor position to make sure the scroll isn't propagated downwards.
        self.handle_scroll_event(Point2D::typed(0f32, 0f32), Point2D::typed(-1f32, -1f32));
        self.update_layer_except_bounds(layer_properties);
    }

    // Add LayerBuffers to the specified layer. Returns the layer buffer set back if the layer that
    // matches the given pipeline ID was not found; otherwise returns None and consumes the layer
    // buffer set.
    //
    // If the epoch of the message does not match the layer's epoch, the message is ignored, the
    // layer buffer set is consumed, and None is returned.
    fn add_buffers<Window>(&self,
                           compositor: &mut IOCompositor<Window>,
                           new_buffers: Box<LayerBufferSet>,
                           epoch: Epoch)
                           where Window: WindowMethods {
        self.extra_data.borrow_mut().painted_epoch = epoch;
        assert!(self.extra_data.borrow().painted_epoch == self.extra_data.borrow().requested_epoch);

        for buffer in new_buffers.buffers.into_iter().rev() {
            self.add_buffer(buffer);
        }

        compositor.cache_unused_buffers(self.collect_unused_buffers())
    }

    fn clear<Window>(&self, compositor: &mut IOCompositor<Window>) where Window: WindowMethods {
        let buffers = self.collect_buffers();

        if !buffers.is_empty() {
            compositor.cache_unused_buffers(buffers);
        }
    }

    /// Destroys tiles for this layer and all descendent layers, sending the buffers back to the
    /// painter to be destroyed or reused.
    fn clear_all_tiles<Window>(&self,
                               compositor: &mut IOCompositor<Window>)
                               where Window: WindowMethods {
        self.clear(compositor);
        for kid in &*self.children() {
            kid.clear_all_tiles(compositor);
        }
    }

    fn remove_root_layer_with_pipeline_id<Window>(&self,
                                                  compositor: &mut IOCompositor<Window>,
                                                  pipeline_id: PipelineId)
                                                  where Window: WindowMethods {
        // Find the child that is the root layer for this pipeline.
        let index = self.children().iter().position(|kid| {
            let extra_data = kid.extra_data.borrow();
            extra_data.pipeline_id == pipeline_id && extra_data.id == LayerId::null()
        });

        match index {
            Some(index) => {
                // Remove the root layer, and return buffers to the paint task
                let child = self.children().remove(index);
                child.clear_all_tiles(compositor);
            }
            None => {
                // Wasn't found, recurse into child layers
                for kid in &*self.children() {
                    kid.remove_root_layer_with_pipeline_id(compositor, pipeline_id);
                }
            }
        }
    }

    fn collect_old_layers<Window>(&self,
                                  compositor: &mut IOCompositor<Window>,
                                  pipeline_id: PipelineId,
                                  new_layers: &Vec<LayerProperties>)
                                  where Window: WindowMethods {
        // Traverse children first so that layers are removed
        // bottom up - allowing each layer being removed to properly
        // clean up any tiles it owns.
        for kid in &*self.children() {
            kid.collect_old_layers(compositor, pipeline_id, new_layers);
        }

        // Retain child layers that also exist in the new layer list.
        self.children().retain(|child| {
            let extra_data = child.extra_data.borrow();

            // Never remove root layers or layers from other pipelines.
            if pipeline_id != extra_data.pipeline_id ||
               extra_data.id == LayerId::null() {
                true
            } else {
                // Keep this layer if it exists in the new layer list.
                let keep_layer = new_layers.iter().any(|properties| {
                    properties.id == extra_data.id
                });

                // When removing a layer, clear any tiles and surfaces
                // associated with the layer.
                if !keep_layer {
                    child.clear_all_tiles(compositor);
                }

                keep_layer
            }
        });
    }

    /// Destroys all tiles of all layers, including children, *without* sending them back to the
    /// painter. You must call this only when the paint task is destined to be going down;
    /// otherwise, you will leak tiles.
    ///
    /// This is used during shutdown, when we know the paint task is going away.
    fn forget_all_tiles(&self) {
        let tiles = self.collect_buffers();
        for tile in tiles {
            let mut tile = tile;
            tile.mark_wont_leak()
        }

        for kid in &*self.children() {
            kid.forget_all_tiles();
        }
    }

    fn handle_scroll_event(&self,
                           delta: TypedPoint2D<LayerPixel, f32>,
                           cursor: TypedPoint2D<LayerPixel, f32>)
                           -> ScrollEventResult {
        // Allow children to scroll.
        let scroll_offset = self.extra_data.borrow().scroll_offset;
        let new_cursor = cursor - scroll_offset;
        for child in &*self.children() {
            let child_bounds = child.bounds.borrow();
            if child_bounds.contains(&new_cursor) {
                let result = child.handle_scroll_event(delta, new_cursor - child_bounds.origin);
                if result != ScrollEventResult::ScrollEventUnhandled {
                    return result;
                }
            }
        }

        // If this layer doesn't want scroll events, it can't handle scroll events.
        if self.wants_scroll_events() != WantsScrollEventsFlag::WantsScrollEvents {
            return ScrollEventResult::ScrollEventUnhandled;
        }

        self.clamp_scroll_offset_and_scroll_layer(scroll_offset + delta)
    }

    fn clamp_scroll_offset_and_scroll_layer(&self, new_offset: TypedPoint2D<LayerPixel, f32>)
                                            -> ScrollEventResult {
        let layer_size = self.bounds.borrow().size;
        let content_size = calculate_content_size_for_layer(self);
        let min_x = (layer_size.width - content_size.width).get().min(0.0);
        let min_y = (layer_size.height - content_size.height).get().min(0.0);
        let new_offset: TypedPoint2D<LayerPixel, f32> =
            Point2D::new(Length::new(new_offset.x.get().clamp(&min_x, &0.0)),
                         Length::new(new_offset.y.get().clamp(&min_y, &0.0)));

        if self.extra_data.borrow().scroll_offset == new_offset {
            return ScrollEventResult::ScrollPositionUnchanged;
        }

        // The scroll offset is just a record of the scroll position of this scrolling root,
        // but scroll_layer_and_all_child_layers actually moves the child layers.
        self.extra_data.borrow_mut().scroll_offset = new_offset;

        let mut result = false;
        for child in &*self.children() {
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

        let pipeline = compositor.get_pipeline(self.pipeline_id());
        let _ = pipeline.script_chan.send(ConstellationControlMsg::SendEvent(pipeline.id.clone(), message));
    }

    fn send_mouse_move_event<Window>(&self,
                                     compositor: &IOCompositor<Window>,
                                     cursor: TypedPoint2D<LayerPixel, f32>)
                                     where Window: WindowMethods {
        let message = MouseMoveEvent(cursor.to_untyped());
        let pipeline = compositor.get_pipeline(self.pipeline_id());
        let _ = pipeline.script_chan.send(ConstellationControlMsg::SendEvent(pipeline.id.clone(), message));
    }

    fn scroll_layer_and_all_child_layers(&self, new_offset: TypedPoint2D<LayerPixel, f32>)
                                         -> bool {
        let mut result = false;

        // Only scroll this layer if it's not fixed-positioned.
        if self.extra_data.borrow().scroll_policy != ScrollPolicy::FixedPosition {
            let new_offset = new_offset.to_untyped();
            *self.content_offset.borrow_mut() = Point2D::from_untyped(&new_offset);
            result = true
        }

        let offset_for_children = new_offset + self.extra_data.borrow().scroll_offset;
        for child in &*self.children() {
            result |= child.scroll_layer_and_all_child_layers(offset_for_children);
        }

        return result;
    }

    fn wants_scroll_events(&self) -> WantsScrollEventsFlag {
        self.extra_data.borrow().wants_scroll_events
    }

    fn pipeline_id(&self) -> PipelineId {
        self.extra_data.borrow().pipeline_id
    }
}
