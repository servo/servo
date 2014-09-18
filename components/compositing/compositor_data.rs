/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use compositor_task::LayerProperties;
use events;
use pipeline::CompositionPipeline;

use azure::azure_hl::Color;
use geom::point::TypedPoint2D;
use geom::scale_factor::ScaleFactor;
use geom::size::{Size2D, TypedSize2D};
use geom::rect::Rect;
use gfx::render_task::UnusedBufferMsg;
use layers::geometry::DevicePixel;
use layers::layers::{Layer, LayerBufferSet};
use layers::platform::surface::NativeSurfaceMethods;
use servo_msg::compositor_msg::{Epoch, LayerId};
use servo_msg::compositor_msg::ScrollPolicy;
use servo_msg::constellation_msg::PipelineId;
use std::rc::Rc;

pub struct CompositorData {
    /// This layer's pipeline. BufferRequests and mouse events will be sent through this.
    pub pipeline: CompositionPipeline,

    /// The ID of this layer within the pipeline.
    pub id: LayerId,

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
            wants_scroll_events: wants_scroll_events,
            scroll_policy: layer_properties.scroll_policy,
            background_color: layer_properties.background_color,
            epoch: layer_properties.epoch,
        };

        Rc::new(Layer::new(Rect::from_untyped(&layer_properties.rect),
                           tile_size, new_compositor_data))
    }

    pub fn update_layer(layer: Rc<Layer<CompositorData>>, layer_properties: LayerProperties) {
        layer.extra_data.borrow_mut().epoch = layer_properties.epoch;
        layer.extra_data.borrow_mut().background_color = layer_properties.background_color;

        let size: TypedSize2D<DevicePixel, f32> = Size2D::from_untyped(&layer_properties.rect.size);
        layer.resize(size);
        layer.contents_changed();

        // Call scroll for bounds checking if the page shrunk. Use (-1, -1) as the
        // cursor position to make sure the scroll isn't propagated downwards. The
        // scale doesn't matter here since 0, 0 is 0, 0 no matter the scene scale.
        events::handle_scroll_event(layer.clone(),
                                    TypedPoint2D(0f32, 0f32),
                                    TypedPoint2D(-1f32, -1f32),
                                    size,
                                    ScaleFactor(1.0) /* scene_scale */);
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
            for buffer in new_buffers.buffers.into_iter().rev() {
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
            for buffer in buffers.iter_mut() {
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
        for tile in tiles.into_iter() {
            let mut tile = tile;
            tile.mark_wont_leak()
        }

        for kid in layer.children().iter() {
            CompositorData::forget_all_tiles(kid.clone());
        }
    }
}

