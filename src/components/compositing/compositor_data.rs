/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use compositor_task::LayerProperties;
use pipeline::CompositionPipeline;
use windowing::{MouseWindowEvent, MouseWindowClickEvent, MouseWindowMouseDownEvent};
use windowing::{MouseWindowMouseUpEvent};

use azure::azure_hl::Color;
use geom::length::Length;
use geom::matrix::identity;
use geom::point::{Point2D, TypedPoint2D};
use geom::rect::{Rect, TypedRect};
use geom::size::{Size2D, TypedSize2D};
use gfx::render_task::{ReRenderMsg, UnusedBufferMsg};
use layers::layers::{Layer, Flip, LayerBuffer, LayerBufferSet, NoFlip, TextureLayer};
use layers::quadtree::{Tile, Normal, Hidden};
use layers::platform::surface::{NativeCompositingGraphicsContext, NativeSurfaceMethods};
use layers::texturegl::{Texture, TextureTarget};
use script::dom::event::{ClickEvent, MouseDownEvent, MouseMoveEvent, MouseUpEvent};
use script::script_task::{ScriptChan, SendEventMsg};
use servo_msg::compositor_msg::{Epoch, FixedPosition, LayerId};
use servo_msg::compositor_msg::ScrollPolicy;
use servo_msg::constellation_msg::PipelineId;
use servo_util::geometry::PagePx;
use std::rc::Rc;

#[cfg(target_os="macos")]
#[cfg(target_os="android")]
use layers::layers::VerticalFlip;
#[cfg(not(target_os="macos"))]
use layers::texturegl::TextureTarget2D;
#[cfg(target_os="macos")]
use layers::texturegl::TextureTargetRectangle;

pub struct CompositorData {
    /// This layer's pipeline. BufferRequests and mouse events will be sent through this.
    pub pipeline: CompositionPipeline,

    /// The ID of this layer within the pipeline.
    pub id: LayerId,

    /// The offset of the page due to scrolling. (0,0) is when the window sees the
    /// top left corner of the page.
    pub scroll_offset: TypedPoint2D<PagePx, f32>,

    /// The bounds of this layer in terms of its parent (a.k.a. the scissor box).
    pub bounds: Rect<f32>,

    /// The size of the underlying page in page coordinates. This is an option
    /// because we may not know the size of the page until layout is finished completely.
    /// if we have no size yet, the layer is hidden until a size message is recieved.
    pub page_size: Option<Size2D<f32>>,

    /// When set to true, this layer is ignored by its parents. This is useful for
    /// soft deletion or when waiting on a page size.
    pub hidden: bool,

    /// The behavior of this layer when a scroll message is received.
    pub wants_scroll_events: WantsScrollEventsFlag,

    /// Whether an ancestor layer that receives scroll events moves this layer.
    pub scroll_policy: ScrollPolicy,

    /// True if CPU rendering is enabled, false if we're using GPU rendering.
    pub cpu_painting: bool,

    /// The color to use for the unrendered-content void
    pub unrendered_color: Color,

    pub scissor: Option<Rect<f32>>,

    /// A monotonically increasing counter that keeps track of the current epoch.
    /// add_buffer() calls that don't match the current epoch will be ignored.
    pub epoch: Epoch,
}

#[deriving(PartialEq, Clone)]
pub enum WantsScrollEventsFlag {
    WantsScrollEvents,
    DoesntWantScrollEvents,
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

impl CompositorData {
    pub fn new(pipeline: CompositionPipeline,
               layer_id: LayerId,
               epoch: Epoch,
               bounds: Rect<f32>,
               page_size: Option<Size2D<f32>>,
               cpu_painting: bool,
               wants_scroll_events: WantsScrollEventsFlag,
               scroll_policy: ScrollPolicy,
               unrendered_color: Color)
               -> CompositorData {
        CompositorData {
            pipeline: pipeline,
            id: layer_id,
            scroll_offset: TypedPoint2D(0f32, 0f32),
            bounds: bounds,
            page_size: page_size,
            hidden: false,
            wants_scroll_events: wants_scroll_events,
            scroll_policy: scroll_policy,
            cpu_painting: cpu_painting,
            unrendered_color: unrendered_color,
            scissor: None,
            epoch: epoch,
        }
    }

    pub fn new_root(pipeline: CompositionPipeline,
                    epoch: Epoch,
                    page_size: Size2D<f32>,
                    cpu_painting: bool,
                    unrendered_color: Color) -> CompositorData {
        CompositorData::new(pipeline,
                            LayerId::null(),
                            epoch,
                            Rect(Point2D(0f32, 0f32), page_size),
                            Some(page_size),
                            cpu_painting,
                            WantsScrollEvents,
                            FixedPosition,
                            unrendered_color)
    }

    /// Adds a child layer to the layer with the given ID and the given pipeline, if it doesn't
    /// exist yet. The child layer will have the same pipeline, tile size, memory limit, and CPU
    /// painting status as its parent.
    pub fn add_child(layer: Rc<Layer<CompositorData>>,
                     layer_properties: LayerProperties,
                     page_size: Size2D<f32>) {
        let new_compositor_data = CompositorData::new(layer.extra_data.borrow().pipeline.clone(),
                                                      layer_properties.id,
                                                      layer_properties.epoch,
                                                      layer_properties.rect,
                                                      Some(page_size),
                                                      layer.extra_data.borrow().cpu_painting,
                                                      DoesntWantScrollEvents,
                                                      layer_properties.scroll_policy,
                                                      layer_properties.background_color);
        let new_kid = Rc::new(Layer::new(page_size,
                                         Layer::tile_size(layer.clone()),
                                         new_compositor_data));

        new_kid.extra_data.borrow_mut().scissor = Some(layer_properties.rect);
        *new_kid.origin.borrow_mut() = layer_properties.rect.origin;

        // Place the kid's layer in the container passed in.
        Layer::add_child(layer.clone(), new_kid.clone());
    }

    /// Move the layer's descendants that don't want scroll events and scroll by a relative
    /// specified amount in page coordinates. This also takes in a cursor position to see if the
    /// mouse is over child layers first. If a layer successfully scrolled, returns true; otherwise
    /// returns false, so a parent layer can scroll instead.
    pub fn handle_scroll_event(layer: Rc<Layer<CompositorData>>,
                               delta: TypedPoint2D<PagePx, f32>,
                               cursor: TypedPoint2D<PagePx, f32>,
                               window_size: TypedSize2D<PagePx, f32>)
                               -> bool {
        // If this layer is hidden, neither it nor its children will scroll.
        if layer.extra_data.borrow().hidden {
            return false
        }

        // If this layer doesn't want scroll events, neither it nor its children can handle scroll
        // events.
        if layer.extra_data.borrow().wants_scroll_events != WantsScrollEvents {
            return false
        }

        // Allow children to scroll.
        let cursor = cursor - layer.extra_data.borrow().scroll_offset;
        for child in layer.children().iter() {
            match child.extra_data.borrow().scissor {
                None => {
                    error!("CompositorData: unable to perform cursor hit test for layer");
                }
                Some(rect) => {
                    let rect: TypedRect<PagePx, f32> = Rect::from_untyped(&rect);
                    if rect.contains(&cursor) &&
                       CompositorData::handle_scroll_event(child.clone(),
                                                           delta,
                                                           cursor - rect.origin,
                                                           rect.size) {
                        return true
                    }
                }
            }
        }

        // This scroll event is mine!
        // Scroll this layer!
        let old_origin = layer.extra_data.borrow().scroll_offset.clone();
        layer.extra_data.borrow_mut().scroll_offset = old_origin + delta;

        // bounds checking
        let page_size = match layer.extra_data.borrow().page_size {
            Some(size) => size,
            None => fail!("CompositorData: tried to scroll with no page size set"),
        };

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
        CompositorData::scroll(layer.clone(), offset)
    }

    /// Actually scrolls the descendants of a layer that scroll. This is called by
    /// `handle_scroll_event` above when it determines that a layer wants to scroll.
    fn scroll(layer: Rc<Layer<CompositorData>>,
              scroll_offset: TypedPoint2D<PagePx, f32>)
              -> bool {
        let mut result = false;

        // Only scroll this layer if it's not fixed-positioned.
        if layer.extra_data.borrow().scroll_policy != FixedPosition {
            // Scroll this layer!
            layer.extra_data.borrow_mut().scroll_offset = scroll_offset;

            let scroll_offset = layer.extra_data.borrow().scroll_offset.clone();
            *layer.transform.borrow_mut() = identity().translate(scroll_offset.x.get(), scroll_offset.y.get(), 0.0);

            result = true
        }

        for child in layer.children().iter() {
            result = CompositorData::scroll(child.clone(), scroll_offset) || result;
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
            if child.extra_data.borrow().hidden {
                continue;
            }

            match child.extra_data.borrow().scissor {
                None => {
                    error!("CompositorData: unable to perform cursor hit test for layer");
                }
                Some(rect) => {
                    let rect: TypedRect<PagePx, f32> = Rect::from_untyped(&rect);
                    if rect.contains(&cursor) {
                        CompositorData::send_mouse_event(child.clone(), event, cursor - rect.origin);
                        return;
                    }
                }
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

    // Given the current window size, determine which tiles need to be (re-)rendered and sends them
    // off the the appropriate renderer. Returns true if and only if the scene should be repainted.
    pub fn send_buffer_requests_recursively(layer: Rc<Layer<CompositorData>>,
                                            graphics_context: &NativeCompositingGraphicsContext,
                                            window_rect: Rect<f32>,
                                            scale: f32)
                                            -> bool {
        let (request, unused) = Layer::get_tile_rects_page(layer.clone(), window_rect, scale);
        let redisplay = !unused.is_empty();
        if redisplay {
            // Send back unused tiles.
            let msg = UnusedBufferMsg(unused);
            let _ = layer.extra_data.borrow().pipeline.render_chan.send_opt(msg);
        }
        if !request.is_empty() {
            // Ask for tiles.
            //
            // FIXME(#2003, pcwalton): We may want to batch these up in the case in which
            // one page has multiple layers, to avoid the user seeing inconsistent states.
            let msg = ReRenderMsg(request,
                                  scale,
                                  layer.extra_data.borrow().id,
                                  layer.extra_data.borrow().epoch);
            let _ = layer.extra_data.borrow().pipeline.render_chan.send_opt(msg);
        }

        if redisplay {
            CompositorData::build_layer_tree(layer.clone(), graphics_context);
        }

        let send_child_buffer_request = |kid: &Rc<Layer<CompositorData>>| -> bool {
            match kid.extra_data.borrow().scissor {
                Some(scissor) => {
                    let mut new_rect = window_rect;
                    let offset = kid.extra_data.borrow().scroll_offset.to_untyped();
                    new_rect.origin.x = new_rect.origin.x - offset.x;
                    new_rect.origin.y = new_rect.origin.y - offset.y;
                    match new_rect.intersection(&scissor) {
                        Some(new_rect) => {
                            // Child layers act as if they are rendered at (0,0), so we
                            // subtract the layer's (x,y) coords in its containing page
                            // to make the child_rect appear in coordinates local to it.
                            let child_rect = Rect(new_rect.origin.sub(&scissor.origin),
                                                  new_rect.size);
                            CompositorData::send_buffer_requests_recursively(kid.clone(),
                                                                             graphics_context,
                                                                             child_rect,
                                                                             scale)
                        }
                        None => {
                            false // Layer is offscreen
                        }
                    }
                }
                None => fail!("child layer not clipped!"),
            }
        };

        layer.children().iter().filter(|x| !x.extra_data.borrow().hidden)
            .map(send_child_buffer_request)
            .any(|b| b) || redisplay
    }

    // Move the sublayer to an absolute position in page coordinates relative to its parent,
    // and clip the layer to the specified size in page coordinates.
    // If the layer is hidden and has a defined page size, unhide it.
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
                *child_node.origin.borrow_mut() = new_rect.origin;
                let old_rect = child_node.extra_data.borrow().scissor.clone();
                child_node.extra_data.borrow_mut().scissor = Some(new_rect);
                match old_rect {
                    Some(old_rect) => {
                        // Rect is unhidden
                        Layer::set_status_page(layer.clone(), old_rect, Normal, false);
                    }
                    None => {} // Nothing to do
                }
                // Hide the new rect
                Layer::set_status_page(layer.clone(), new_rect, Hidden, false);

                // If possible, unhide child
                let mut child_data = child_node.extra_data.borrow_mut();
                if child_data.hidden && child_data.page_size.is_some() {
                    child_data.hidden = false;
                }
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
        CompositorData::resize(layer.clone(), layer_properties.rect.size);
    }

    // Resize and unhide a pre-existing layer. A new layer's size is set during creation.
    fn resize(layer: Rc<Layer<CompositorData>>,
              new_size: Size2D<f32>) {
        debug!("compositor_data: starting resize_helper()");

        debug!("compositor_data: layer found for resize_helper()");
        layer.extra_data.borrow_mut().page_size = Some(new_size);

        let unused_buffers = Layer::resize(layer.clone(), new_size);
        if !unused_buffers.is_empty() {
            let msg = UnusedBufferMsg(unused_buffers);
            let _ = layer.extra_data.borrow().pipeline.render_chan.send_opt(msg);
        }

        let scissor_clone = layer.extra_data.borrow().scissor.clone();
        match scissor_clone {
            Some(scissor) => {
                // Call scroll for bounds checking if the page shrunk. Use (-1, -1) as the
                // cursor position to make sure the scroll isn't propagated downwards.
                let size: TypedSize2D<PagePx, f32> = Size2D::from_untyped(&scissor.size);
                CompositorData::handle_scroll_event(layer.clone(),
                                                    TypedPoint2D(0f32, 0f32),
                                                    TypedPoint2D(-1f32, -1f32),
                                                    size);
                layer.extra_data.borrow_mut().hidden = false;
            }
            None => {} // Nothing to do
        }

        CompositorData::set_occlusions(layer.clone());
    }

    pub fn move(layer: Rc<Layer<CompositorData>>,
                pipeline_id: PipelineId,
                layer_id: LayerId,
                origin: Point2D<f32>,
                window_size: TypedSize2D<PagePx, f32>)
                -> bool {
        // Search children for the right layer to move.
        if layer.extra_data.borrow().pipeline.id != pipeline_id ||
           layer.extra_data.borrow().id != layer_id {
            return layer.children().iter().any(|kid| {
                CompositorData::move(kid.clone(), pipeline_id, layer_id, origin, window_size)
            });
        }

        if layer.extra_data.borrow().wants_scroll_events != WantsScrollEvents {
            return false
        }

        // Scroll this layer!
        let old_origin = layer.extra_data.borrow().scroll_offset;
        layer.extra_data.borrow_mut().scroll_offset = Point2D::from_untyped(&(origin * -1.0));

        // bounds checking
        let page_size = match layer.extra_data.borrow().page_size {
            Some(size) => size,
            None => fail!("CompositorData: tried to scroll with no page size set"),
        };
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
        CompositorData::scroll(layer.clone(), offset)
    }

    // Returns whether the layer should be vertically flipped.
    #[cfg(target_os="macos")]
    fn texture_flip_and_target(cpu_painting: bool, size: Size2D<uint>) -> (Flip, TextureTarget) {
        let flip = if cpu_painting {
            NoFlip
        } else {
            VerticalFlip
        };

        (flip, TextureTargetRectangle(size))
    }

    #[cfg(target_os="android")]
    fn texture_flip_and_target(cpu_painting: bool, size: Size2D<uint>) -> (Flip, TextureTarget) {
        let flip = if cpu_painting {
            NoFlip
        } else {
            VerticalFlip
        };

        (flip, TextureTarget2D)
    }

    #[cfg(target_os="linux")]
    fn texture_flip_and_target(_: bool, _: Size2D<uint>) -> (Flip, TextureTarget) {
        (NoFlip, TextureTarget2D)
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

    // Collect buffers from the quadtree. This method IS NOT recursive, so child layers
    // are not rebuilt directly from this method.
    pub fn build_layer_tree(layer: Rc<Layer<CompositorData>>,
                            graphics_context: &NativeCompositingGraphicsContext) {
        // Clear all old textures.
        layer.tiles.borrow_mut().clear();

        // Add new tiles.
        Layer::do_for_all_tiles(layer.clone(), |buffer: &Box<LayerBuffer>| {
            debug!("osmain: compositing buffer rect {}", buffer.rect);

            let size = Size2D(buffer.screen_pos.size.width as int,
                              buffer.screen_pos.size.height as int);

            debug!("osmain: adding new texture layer");

            // Determine, in a platform-specific way, whether we should flip the texture
            // and which target to use.
            let (flip, target) =
                    CompositorData::texture_flip_and_target(layer.extra_data.borrow().cpu_painting,
                                                            buffer.screen_pos.size);

            // Make a new texture and bind the layer buffer's surface to it.
            let texture = Texture::new(target);
            debug!("COMPOSITOR binding to native surface {:d}",
                   buffer.native_surface.get_id() as int);
            buffer.native_surface.bind_to_texture(graphics_context, &texture, size);

            // Set the layer's transform.
            let rect = buffer.rect;
            let transform = identity().translate(rect.origin.x, rect.origin.y, 0.0);
            let transform = transform.scale(rect.size.width, rect.size.height, 1.0);

            // Make a texture layer and add it.
            let texture_layer = Rc::new(TextureLayer::new(texture, buffer.screen_pos.size,
                                                          flip, transform));
            layer.tiles.borrow_mut().push(texture_layer);
        });
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
            let mut unused_tiles = vec!();
            for buffer in new_buffers.buffers.move_iter().rev() {
                unused_tiles.push_all_move(Layer::add_tile_pixel(layer.clone(), buffer));
            }
            if !unused_tiles.is_empty() { // send back unused buffers
                let msg = UnusedBufferMsg(unused_tiles);
                let _ = layer.extra_data.borrow().pipeline.render_chan.send_opt(msg);
            }
        }

        CompositorData::build_layer_tree(layer.clone(), graphics_context);
        return true;
    }

    // Recursively sets occluded portions of quadtrees to Hidden, so that they do not ask for
    // tile requests. If layers are moved, resized, or deleted, these portions may be updated.
    fn set_occlusions(layer: Rc<Layer<CompositorData>>) {
        for kid in layer.children().iter() {
            if !kid.extra_data.borrow().hidden {
                match kid.extra_data.borrow().scissor {
                    None => {} // Nothing to do
                    Some(rect) => {
                        Layer::set_status_page(layer.clone(), rect, Hidden, false);
                    }
                }
            }
        }

        for kid in layer.children().iter() {
            if !kid.extra_data.borrow().hidden {
                CompositorData::set_occlusions(kid.clone());
            }
        }
    }

    /// Destroys all quadtree tiles, sending the buffers back to the renderer to be destroyed or
    /// reused.
    fn clear(layer: Rc<Layer<CompositorData>>) {
        let mut tiles = Layer::collect_tiles(layer.clone());

        if !tiles.is_empty() {
            // We have no way of knowing without a race whether the render task is even up and
            // running, but mark the tiles as not leaking. If the render task died, then the
            // tiles are going to be cleaned up.
            for tile in tiles.mut_iter() {
                tile.mark_wont_leak()
            }

            let _ = layer.extra_data.borrow().pipeline.render_chan.send_opt(UnusedBufferMsg(tiles));
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
        let tiles = Layer::collect_tiles(layer.clone());
        for tile in tiles.move_iter() {
            let mut tile = tile;
            tile.mark_wont_leak()
        }

        for kid in layer.children().iter() {
            CompositorData::forget_all_tiles(kid.clone());
        }
    }
}

