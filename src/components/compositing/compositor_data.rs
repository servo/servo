/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use compositor_task::LayerProperties;
use events;
use pipeline::CompositionPipeline;

use azure::azure_hl::Color;
use geom::point::TypedPoint2D;
use geom::rect::Rect;
use geom::size::{Size2D, TypedSize2D};
use gfx::render_task::{ReRenderRequest, ReRenderMsg, RenderChan, UnusedBufferMsg};
use layers::layers::{Layer, LayerBufferSet};
use layers::platform::surface::{NativeCompositingGraphicsContext, NativeSurfaceMethods};
use servo_msg::compositor_msg::{Epoch, FixedPosition, LayerId};
use servo_msg::compositor_msg::ScrollPolicy;
use servo_msg::constellation_msg::PipelineId;
use servo_util::geometry::PagePx;
use std::collections::hashmap::HashMap;
use std::rc::Rc;

pub struct CompositorData {
    /// This layer's pipeline. BufferRequests and mouse events will be sent through this.
    pub pipeline: CompositionPipeline,

    /// The ID of this layer within the pipeline.
    pub id: LayerId,

    /// The offset of the page due to scrolling. (0,0) is when the window sees the
    /// top left corner of the page.
    pub scroll_offset: TypedPoint2D<PagePx, f32>,

    /// The behavior of this layer when a scroll message is received.
    pub wants_scroll_events: WantsScrollEventsFlag,

    /// Whether an ancestor layer that receives scroll events moves this layer.
    pub scroll_policy: ScrollPolicy,

    /// True if CPU rendering is enabled, false if we're using GPU rendering.
    pub cpu_painting: bool,

    /// The color to use for the unrendered-content void
    pub unrendered_color: Color,

    /// A monotonically increasing counter that keeps track of the current epoch.
    /// add_buffer() calls that don't match the current epoch will be ignored.
    pub epoch: Epoch,
}

#[deriving(PartialEq, Clone)]
pub enum WantsScrollEventsFlag {
    WantsScrollEvents,
    DoesntWantScrollEvents,
}

impl CompositorData {
    pub fn new(pipeline: CompositionPipeline,
               layer_id: LayerId,
               epoch: Epoch,
               cpu_painting: bool,
               wants_scroll_events: WantsScrollEventsFlag,
               scroll_policy: ScrollPolicy,
               unrendered_color: Color)
               -> CompositorData {
        CompositorData {
            pipeline: pipeline,
            id: layer_id,
            scroll_offset: TypedPoint2D(0f32, 0f32),
            wants_scroll_events: wants_scroll_events,
            scroll_policy: scroll_policy,
            cpu_painting: cpu_painting,
            unrendered_color: unrendered_color,
            epoch: epoch,
        }
    }

    pub fn new_root(pipeline: CompositionPipeline,
                    epoch: Epoch,
                    cpu_painting: bool,
                    unrendered_color: Color) -> CompositorData {
        CompositorData::new(pipeline,
                            LayerId::null(),
                            epoch,
                            cpu_painting,
                            WantsScrollEvents,
                            FixedPosition,
                            unrendered_color)
    }

    /// Adds a child layer to the layer with the given ID and the given pipeline, if it doesn't
    /// exist yet. The child layer will have the same pipeline, tile size, memory limit, and CPU
    /// painting status as its parent.
    pub fn add_child(layer: Rc<Layer<CompositorData>>,
                     layer_properties: LayerProperties) {
        let new_compositor_data = CompositorData::new(layer.extra_data.borrow().pipeline.clone(),
                                                      layer_properties.id,
                                                      layer_properties.epoch,
                                                      layer.extra_data.borrow().cpu_painting,
                                                      DoesntWantScrollEvents,
                                                      layer_properties.scroll_policy,
                                                      layer_properties.background_color);
        let new_kid = Rc::new(Layer::new(layer_properties.rect,
                                         layer.tile_size,
                                         new_compositor_data));
        layer.add_child(new_kid.clone());
    }

    // Given the current window size, determine which tiles need to be (re-)rendered and sends them
    // off the the appropriate renderer. Returns true if and only if the scene should be repainted.
    pub fn get_buffer_requests_recursively(requests: &mut HashMap<PipelineId,
                                                                  (RenderChan,
                                                                   Vec<ReRenderRequest>)>,
                                           layer: Rc<Layer<CompositorData>>,
                                           graphics_context: &NativeCompositingGraphicsContext,
                                           window_rect: Rect<f32>,
                                           scale: f32)
                                           -> bool {
        let (request, unused) = layer.get_tile_rects_page(window_rect, scale);
        let redisplay = !unused.is_empty();
        if redisplay {
            // Send back unused tiles.
            let msg = UnusedBufferMsg(unused);
            let _ = layer.extra_data.borrow().pipeline.render_chan.send_opt(msg);
        }
        if !request.is_empty() {
            // Ask for tiles.
            let pipeline_id = layer.extra_data.borrow().pipeline.id;
            let msg = ReRenderRequest {
                buffer_requests: request,
                scale: scale,
                layer_id: layer.extra_data.borrow().id,
                epoch: layer.extra_data.borrow().epoch,
            };
            let &(_, ref mut vec) = requests.find_or_insert_with(pipeline_id, |_| {
                (layer.extra_data.borrow().pipeline.render_chan.clone(), Vec::new())
            });
            vec.push(msg);
        }

        if redisplay {
            layer.create_textures(graphics_context);
        }

        let get_child_buffer_request = |kid: &Rc<Layer<CompositorData>>| -> bool {
            let mut new_rect = window_rect;
            let offset = kid.extra_data.borrow().scroll_offset.to_untyped();
            new_rect.origin.x = new_rect.origin.x - offset.x;
            new_rect.origin.y = new_rect.origin.y - offset.y;
            match new_rect.intersection(&*kid.bounds.borrow()) {
                Some(new_rect) => {
                    // Child layers act as if they are rendered at (0,0), so we
                    // subtract the layer's (x,y) coords in its containing page
                    // to make the child_rect appear in coordinates local to it.
                    let child_rect = Rect(new_rect.origin.sub(&kid.bounds.borrow().origin),
                                          new_rect.size);
                    CompositorData::get_buffer_requests_recursively(requests,
                                                                    kid.clone(),
                                                                    graphics_context,
                                                                    child_rect,
                                                                    scale)
                }
                None => {
                    false // Layer is offscreen
                }
            }
        };

        layer.children().iter().map(get_child_buffer_request).any(|b| b) || redisplay
    }

    // Move the sublayer to an absolute position in page coordinates relative to its parent,
    // and clip the layer to the specified size in page coordinates.
    // This method returns false if the specified layer is not found.
    pub fn set_clipping_rect(layer: Rc<Layer<CompositorData>>,
                             pipeline_id: PipelineId,
                             layer_id: LayerId,
                             new_rect: Rect<f32>)
                             -> bool {
        debug!("compositor_data: starting set_clipping_rect()");
        match CompositorData::find_child_with_pipeline_and_layer_id(layer.clone(),
                                                                    pipeline_id,
                                                                    layer_id) {
            Some(child_node) => {
                debug!("compositor_data: node found for set_clipping_rect()");
                *child_node.bounds.borrow_mut() = new_rect;
                true
            }
            None => {
                layer.children().iter()
                    .any(|kid| CompositorData::set_clipping_rect(kid.clone(),
                                                                 pipeline_id,
                                                                 layer_id,
                                                                 new_rect))

            }
        }
    }

    pub fn update_layer(layer: Rc<Layer<CompositorData>>, layer_properties: LayerProperties) {
        layer.extra_data.borrow_mut().epoch = layer_properties.epoch;
        layer.extra_data.borrow_mut().unrendered_color = layer_properties.background_color;

        layer.resize(layer_properties.rect.size);
        layer.contents_changed();

        // Call scroll for bounds checking if the page shrunk. Use (-1, -1) as the
        // cursor position to make sure the scroll isn't propagated downwards.
        let size: TypedSize2D<PagePx, f32> = Size2D::from_untyped(&layer.bounds.borrow().size);
        events::handle_scroll_event(layer.clone(),
                                            TypedPoint2D(0f32, 0f32),
                                            TypedPoint2D(-1f32, -1f32),
                                            size);
    }

    fn find_child_with_pipeline_and_layer_id(layer: Rc<Layer<CompositorData>>,
                                             pipeline_id: PipelineId,
                                             layer_id: LayerId)
                                             -> Option<Rc<Layer<CompositorData>>> {
        for kid in layer.children().iter() {
            if pipeline_id == kid.extra_data.borrow().pipeline.id &&
               layer_id == kid.extra_data.borrow().id {
                return Some(kid.clone());
            }
        }
        return None
    }

    pub fn find_layer_with_pipeline_and_layer_id(layer: Rc<Layer<CompositorData>>,
                                                 pipeline_id: PipelineId,
                                                 layer_id: LayerId)
                                                 -> Option<Rc<Layer<CompositorData>>> {
        if layer.extra_data.borrow().pipeline.id == pipeline_id &&
           layer.extra_data.borrow().id == layer_id {
            return Some(layer.clone());
        }

        for kid in layer.children().iter() {
            match CompositorData::find_layer_with_pipeline_and_layer_id(kid.clone(),
                                                                        pipeline_id,
                                                                        layer_id) {
                v @ Some(_) => { return v; }
                None => { }
            }
        }

        return None;
    }

    // Add LayerBuffers to the specified layer. Returns the layer buffer set back if the layer that
    // matches the given pipeline ID was not found; otherwise returns None and consumes the layer
    // buffer set.
    //
    // If the epoch of the message does not match the layer's epoch, the message is ignored, the
    // layer buffer set is consumed, and None is returned.
    pub fn add_buffers(layer: Rc<Layer<CompositorData>>,
                       graphics_context: &NativeCompositingGraphicsContext,
                       new_buffers: Box<LayerBufferSet>,
                       epoch: Epoch)
                       -> bool {
        if layer.extra_data.borrow().epoch != epoch {
            debug!("add_buffers: compositor epoch mismatch: {:?} != {:?}, id: {:?}",
                   layer.extra_data.borrow().epoch,
                   epoch,
                   layer.extra_data.borrow().pipeline.id);
            let msg = UnusedBufferMsg(new_buffers.buffers);
            let _ = layer.extra_data.borrow().pipeline.render_chan.send_opt(msg);
            return false;
        }

        {
            for buffer in new_buffers.buffers.move_iter().rev() {
                layer.add_buffer(buffer);
            }

            let unused_buffers = layer.collect_unused_buffers();
            if !unused_buffers.is_empty() { // send back unused buffers
                let msg = UnusedBufferMsg(unused_buffers);
                let _ = layer.extra_data.borrow().pipeline.render_chan.send_opt(msg);
            }

            let (pending_buffer_requests, scale) = layer.flush_pending_buffer_requests();
            if !pending_buffer_requests.is_empty() {
                let mut requests = Vec::new();
                requests.push(ReRenderRequest {
                    buffer_requests: pending_buffer_requests,
                    scale: scale,
                    layer_id: layer.extra_data.borrow().id,
                    epoch: layer.extra_data.borrow().epoch,
                });
                let _ = layer.extra_data.borrow().pipeline.render_chan.send_opt(ReRenderMsg(requests));
            }
        }

        layer.create_textures(graphics_context);
        return true;
    }

    /// Destroys all layer tiles, sending the buffers back to the renderer to be destroyed or
    /// reused.
    fn clear(layer: Rc<Layer<CompositorData>>) {
        let mut buffers = layer.collect_buffers();

        if !buffers.is_empty() {
            // We have no way of knowing without a race whether the render task is even up and
            // running, but mark the buffers as not leaking. If the render task died, then the
            // buffers are going to be cleaned up.
            for buffer in buffers.mut_iter() {
                buffer.mark_wont_leak()
            }

            let _ = layer.extra_data.borrow().pipeline.render_chan.send_opt(UnusedBufferMsg(buffers));
        }
    }

    /// Destroys tiles for this layer and all descendent layers, sending the buffers back to the
    /// renderer to be destroyed or reused.
    pub fn clear_all_tiles(layer: Rc<Layer<CompositorData>>) {
        CompositorData::clear(layer.clone());
        for kid in layer.children().iter() {
            CompositorData::clear_all_tiles(kid.clone());
        }
    }

    /// Destroys all tiles of all layers, including children, *without* sending them back to the
    /// renderer. You must call this only when the render task is destined to be going down;
    /// otherwise, you will leak tiles.
    ///
    /// This is used during shutdown, when we know the render task is going away.
    pub fn forget_all_tiles(layer: Rc<Layer<CompositorData>>) {
        let tiles = layer.collect_buffers();
        for tile in tiles.move_iter() {
            let mut tile = tile;
            tile.mark_wont_leak()
        }

        for kid in layer.children().iter() {
            CompositorData::forget_all_tiles(kid.clone());
        }
    }
}

