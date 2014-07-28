/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use compositor_task::LayerProperties;
use events;
use pipeline::CompositionPipeline;

use azure::azure_hl::Color;
use geom::point::TypedPoint2D;
use geom::rect::Rect;
use geom::size::{TypedSize2D};
use gfx::render_task::{ReRenderRequest, RenderChan, UnusedBufferMsg};
use layers::layers::{Layer, LayerBufferSet};
use layers::platform::surface::NativeSurfaceMethods;
use servo_msg::compositor_msg::{Epoch, LayerId};
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

    /// The color to use for the unrendered-content void
    pub background_color: Color,

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
    pub fn new_layer(pipeline: CompositionPipeline,
                     layer_properties: LayerProperties,
                     wants_scroll_events: WantsScrollEventsFlag,
                     tile_size: uint)
                     -> Rc<Layer<CompositorData>> {
        let new_compositor_data = CompositorData {
            pipeline: pipeline,
            id: layer_properties.id,
            scroll_offset: TypedPoint2D(0f32, 0f32),
            wants_scroll_events: wants_scroll_events,
            scroll_policy: layer_properties.scroll_policy,
            background_color: layer_properties.background_color,
            epoch: layer_properties.epoch,
        };
        Rc::new(Layer::new(layer_properties.rect, tile_size, new_compositor_data))
    }

    /// Adds a child layer to the layer with the given ID and the given pipeline, if it doesn't
    /// exist yet. The child layer will have the same pipeline, tile size, memory limit, and CPU
    /// painting status as its parent.
    pub fn add_child(layer: Rc<Layer<CompositorData>>,
                     layer_properties: LayerProperties) {
        let new_kid = CompositorData::new_layer(layer.extra_data.borrow().pipeline.clone(),
                                                layer_properties,
                                                DoesntWantScrollEvents,
                                                layer.tile_size);
        layer.add_child(new_kid.clone());
    }

    // Given the current window size, determine which tiles need to be (re-)rendered and sends them
    // off the the appropriate renderer. Returns true if and only if the scene should be repainted.
    pub fn get_buffer_requests_recursively(requests: &mut HashMap<PipelineId,
                                                                  (RenderChan,
                                                                   Vec<ReRenderRequest>)>,
                                           layer: Rc<Layer<CompositorData>>,
                                           window_rect: Rect<f32>,
                                           scale: f32)
                                           -> bool {
        // Layers act as if they are rendered at (0,0), so we
        // subtract the layer's (x,y) coords in its containing page
        // to make the rect appear in coordinates local to it.
        let mut new_rect = window_rect;
        let offset = layer.extra_data.borrow().scroll_offset.to_untyped();
        new_rect.origin.x = new_rect.origin.x - offset.x;
        new_rect.origin.y = new_rect.origin.y - offset.y;

        let (request, unused) = layer.get_tile_rects_page(new_rect, scale);
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

        let get_child_buffer_request = |kid: &Rc<Layer<CompositorData>>| -> bool {
            match new_rect.intersection(&*kid.bounds.borrow()) {
                Some(new_rect) => {
                    let child_rect = Rect(new_rect.origin.sub(&kid.bounds.borrow().origin),
                                          new_rect.size);
                    CompositorData::get_buffer_requests_recursively(requests,
                                                                    kid.clone(),
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

    pub fn update_layer(layer: Rc<Layer<CompositorData>>,
                        window_size: TypedSize2D<PagePx, f32>,
                        layer_properties: LayerProperties) {

        layer.extra_data.borrow_mut().id = layer_properties.id;
        layer.extra_data.borrow_mut().epoch = layer_properties.epoch;
        layer.extra_data.borrow_mut().background_color = layer_properties.background_color;
        layer.extra_data.borrow_mut().scroll_policy = layer_properties.scroll_policy;
        layer.resize(layer_properties.rect.size);
        layer.contents_changed();

        // Call scroll for bounds checking if the page shrunk. Use (-1, -1) as the
        // cursor position to make sure the scroll isn't propagated downwards.
        events::handle_scroll_event(layer.clone(),
                                    TypedPoint2D(0f32, 0f32),
                                    TypedPoint2D(-1f32, -1f32),
                                    window_size);
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
        }

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
