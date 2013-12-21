/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use compositing::quadtree::{Quadtree, Normal, Invalid, Hidden};
use constellation::{SendableChildFrameTree, SendableFrameTree};
use geom::matrix::identity;
use geom::point::Point2D;
use geom::rect::Rect;
use geom::size::Size2D;
use gfx::render_task::{ReRenderMsg, UnusedBufferMsg};
use layers::layers::{ContainerLayerKind, ContainerLayer, Flip, NoFlip, TextureLayer};
use layers::layers::TextureLayerKind;
#[cfg(target_os="macos")] 
#[cfg(target_os="android")]
use layers::layers::VerticalFlip;
use layers::platform::surface::{NativeCompositingGraphicsContext, NativeSurfaceMethods};
use layers::texturegl::{Texture, TextureTarget};
#[cfg(target_os="macos")] use layers::texturegl::TextureTargetRectangle;
use pipeline::CompositionPipeline;
use script::dom::event::{ClickEvent, MouseDownEvent, MouseUpEvent};
use script::script_task::SendEventMsg;
use servo_msg::compositor_msg::{LayerBuffer, LayerBufferSet, Epoch, Tile};
use servo_msg::constellation_msg::PipelineId;
use windowing::{MouseWindowEvent, MouseWindowClickEvent, MouseWindowMouseDownEvent};
use windowing::{MouseWindowMouseUpEvent};
use azure::azure_hl::Color;
use gfx;

#[cfg(not(target_os="macos"))]
use layers::texturegl::TextureTarget2D;

/// The CompositorLayer represents an element on a page that has a unique scroll
/// or animation behavior. This can include absolute positioned elements, iframes, etc.
/// Each layer can also have child layers.
pub struct CompositorLayer {
    /// This layer's pipeline. BufferRequests and mouse events will be sent through this.
    pipeline: CompositionPipeline,

    /// The size of the underlying page in page coordinates. This is an option
    /// because we may not know the size of the page until layout is finished completely.
    /// if we have no size yet, the layer is hidden until a size message is recieved.
    page_size: Option<Size2D<f32>>,

    /// The offset of the page due to scrolling. (0,0) is when the window sees the
    /// top left corner of the page.
    scroll_offset: Point2D<f32>,

    /// This layer's children. These could be iframes or any element which
    /// differs in scroll behavior from its parent. Each is associated with a
    /// ContainerLayer which determines its position relative to its parent and
    /// clipping rect. Children are stored in the order in which they are drawn.
    children: ~[CompositorLayerChild],

    /// This layer's quadtree. This is where all buffers are stored for this layer.
    quadtree: MaybeQuadtree,

    /// The root layer of this CompositorLayer's layer tree. Buffers are collected
    /// from the quadtree and inserted here when the layer is painted to the screen.
    root_layer: @mut ContainerLayer,

    /// When set to true, this layer is ignored by its parents. This is useful for
    /// soft deletion or when waiting on a page size.
    hidden: bool,

    /// A monotonically increasing counter that keeps track of the current epoch.
    /// add_buffer() calls that don't match the current epoch will be ignored.
    epoch: Epoch,

    /// The behavior of this layer when a scroll message is received. 
    scroll_behavior: ScrollBehavior,

    /// True if CPU rendering is enabled, false if we're using GPU rendering.
    cpu_painting: bool,

    /// The color to use for the unrendered-content void
    unrendered_color: Color
}

/// Helper struct for keeping CompositorLayer children organized.
struct CompositorLayerChild {
    /// The child itself.
    child: ~CompositorLayer, 
    /// A ContainerLayer managed by the parent node. This deals with clipping and
    /// positioning, and is added above the child's layer tree.
    container: @mut ContainerLayer,
}

/// Helper enum for storing quadtrees. Either contains a quadtree, or contains
/// information from which a quadtree can be built.
enum MaybeQuadtree {
    Tree(Quadtree<~LayerBuffer>),
    NoTree(uint, Option<uint>),
}

/// Determines the behavior of the layer when a scroll message is recieved.
enum ScrollBehavior {
    /// Normal scrolling behavior.
    Scroll,
    /// Scrolling messages targeted at this layer are ignored, but can be
    /// passed on to child layers.
    FixedPosition,
}

impl CompositorLayer {
    /// Creates a new CompositorLayer with an optional page size. If no page size is given,
    /// the layer is initially hidden and initialized without a quadtree.
    pub fn new(pipeline: CompositionPipeline,
               page_size: Option<Size2D<f32>>,
               tile_size: uint,
               max_mem: Option<uint>,
               cpu_painting: bool)
               -> CompositorLayer {
        CompositorLayer {
            pipeline: pipeline,
            page_size: page_size,
            scroll_offset: Point2D(0f32, 0f32),
            children: ~[],
            quadtree: match page_size {
                None => NoTree(tile_size, max_mem),
                Some(page_size) => Tree(Quadtree::new(Size2D(page_size.width as uint,
                                                             page_size.height as uint),
                                                      tile_size,
                                                      max_mem)),
            },
            root_layer: @mut ContainerLayer(),
            hidden: true,
            epoch: Epoch(0),
            scroll_behavior: Scroll,
            cpu_painting: cpu_painting,
            unrendered_color: gfx::color::rgba(0.0, 0.0, 0.0, 0.0),
        }
    }
    
    /// Constructs a CompositorLayer tree from a frame tree.
    pub fn from_frame_tree(frame_tree: SendableFrameTree,
                           tile_size: uint,
                           max_mem: Option<uint>,
                           cpu_painting: bool)
                           -> CompositorLayer {
        let SendableFrameTree { pipeline, children } = frame_tree;
        let mut layer = CompositorLayer::new(pipeline, None, tile_size, max_mem, cpu_painting);
        layer.children = (children.move_iter().map(|child| {
            let SendableChildFrameTree { frame_tree, rect } = child;
            let container = @mut ContainerLayer();
            match rect {
                Some(rect) => {
                     container.scissor = Some(rect);
                     container.common.transform = identity().translate(rect.origin.x,
                                                                       rect.origin.y,
                                                                       0f32);
                    
                }
                None => {}
            }
            
            let child_layer = ~CompositorLayer::from_frame_tree(frame_tree,
                                                                tile_size,
                                                                max_mem,
                                                                cpu_painting);
            container.add_child_start(ContainerLayerKind(child_layer.root_layer));
            
            CompositorLayerChild {
                child: child_layer,
                container: container,
            }
        })).collect();
        layer.set_occlusions();
        layer
    }

    // Move the layer by as relative specified amount in page coordinates. Does not change
    // the position of the layer relative to its parent. This also takes in a cursor position
    // to see if the mouse is over child layers first. If a layer successfully scrolled, returns
    // true; otherwise returns false, so a parent layer can scroll instead.
    pub fn scroll(&mut self, delta: Point2D<f32>, cursor: Point2D<f32>, window_size: Size2D<f32>)
                  -> bool {
        let cursor = cursor - self.scroll_offset;
        for child in self.children.mut_iter().filter(|x| !x.child.hidden) {
            match child.container.scissor {
                None => {
                    error!("CompositorLayer: unable to perform cursor hit test for layer");
                }
                Some(rect) => {
                    if cursor.x >= rect.origin.x && cursor.x < rect.origin.x + rect.size.width
                        && cursor.y >= rect.origin.y && cursor.y < rect.origin.y + rect.size.height
                        && child.child.scroll(delta, cursor - rect.origin, rect.size) {
                        return true;
                    }
                }
            }
        }

        // This scroll event is mine!
        match self.scroll_behavior {
            Scroll => {
                // Scroll this layer!
                let old_origin = self.scroll_offset;
                self.scroll_offset = self.scroll_offset + delta;

                // bounds checking
                let page_size = match self.page_size {
                    Some(size) => size,
                    None => fail!("CompositorLayer: tried to scroll with no page size set"),
                };
                let min_x = (window_size.width - page_size.width).min(&0.0);
                self.scroll_offset.x = self.scroll_offset.x.clamp(&min_x, &0.0);
                let min_y = (window_size.height - page_size.height).min(&0.0);
                self.scroll_offset.y = self.scroll_offset.y.clamp(&min_y, &0.0);

                // check to see if we scrolled
                if old_origin - self.scroll_offset == Point2D(0f32, 0f32) {
                    return false;
                }

                self.root_layer.common.set_transform(identity().translate(self.scroll_offset.x,
                                                                          self.scroll_offset.y,
                                                                          0.0));
                true
            }
            FixedPosition => false, // Ignore this scroll event.
        }
    }

    // Takes in a MouseWindowEvent, determines if it should be passed to children, and 
    // sends the event off to the appropriate pipeline. NB: the cursor position is in
    // page coordinates.
    pub fn send_mouse_event(&self, event: MouseWindowEvent, cursor: Point2D<f32>) {
        let cursor = cursor - self.scroll_offset;
        for child in self.children.iter().filter(|&x| !x.child.hidden) {
            match child.container.scissor {
                None => {
                    error!("CompositorLayer: unable to perform cursor hit test for layer");
                }
                Some(rect) => {
                    if cursor.x >= rect.origin.x && cursor.x < rect.origin.x + rect.size.width
                        && cursor.y >= rect.origin.y && cursor.y < rect.origin.y + rect.size.height {
                        child.child.send_mouse_event(event, cursor - rect.origin);
                        return;
                    }
                }
            }
        }
        
        // This mouse event is mine!
        let message = match event {
            MouseWindowClickEvent(button, _) => ClickEvent(button, cursor),
            MouseWindowMouseDownEvent(button, _) => MouseDownEvent(button, cursor),
            MouseWindowMouseUpEvent(button, _) => MouseUpEvent(button, cursor),
        };
        
        self.pipeline.script_chan.send(SendEventMsg(self.pipeline.id.clone(), message));
    }
    
    // Given the current window size, determine which tiles need to be (re)rendered
    // and sends them off the the appropriate renderer.
    // Returns a bool that is true if the scene should be repainted.
    pub fn get_buffer_request(&mut self,
                              graphics_context: &NativeCompositingGraphicsContext,
                              window_rect: Rect<f32>,
                              scale: f32)
                              -> bool {
        let rect = Rect(Point2D(-self.scroll_offset.x + window_rect.origin.x,
                                -self.scroll_offset.y + window_rect.origin.y),
                        window_rect.size);
        let mut redisplay: bool;
        { // block here to prevent double mutable borrow of self
            let quadtree = match self.quadtree {
                NoTree(..) => fail!("CompositorLayer: cannot get buffer request for {:?},
                                   no quadtree initialized", self.pipeline.id),
                Tree(ref mut quadtree) => quadtree,
            };
            let (request, unused) = quadtree.get_tile_rects_page(rect, scale);
            redisplay = !unused.is_empty(); // workaround to make redisplay visible outside block
            if redisplay { // send back unused tiles
                self.pipeline.render_chan.send(UnusedBufferMsg(unused));
            }
            if !request.is_empty() { // ask for tiles
                self.pipeline.render_chan.try_send(ReRenderMsg(request, scale, self.epoch));
            }
        }
        if redisplay {
            self.build_layer_tree(graphics_context);
        }
        let transform = |x: &mut CompositorLayerChild| -> bool {
            match x.container.scissor {
                Some(scissor) => {
                    let new_rect = rect.intersection(&scissor);
                    match new_rect {
                        Some(new_rect) => {
                            // Child layers act as if they are rendered at (0,0), so we
                            // subtract the layer's (x,y) coords in its containing page
                            // to make the child_rect appear in coordinates local to it.
                            let child_rect = Rect(new_rect.origin.sub(&scissor.origin),
                                                  new_rect.size);
                            x.child.get_buffer_request(graphics_context, child_rect, scale)
                        }
                        None => {
                            false //Layer is offscreen
                        }
                    }
                }
                None => {
                    fail!("CompositorLayer: Child layer not clipped");
                }
            }
        };
        self.children.mut_iter().filter(|x| !x.child.hidden)
            .map(transform)
            .fold(false, |a, b| a || b) || redisplay
    }


    // Move the sublayer to an absolute position in page coordinates relative to its parent,
    // and clip the layer to the specified size in page coordinates.
    // If the layer is hidden and has a defined page size, unhide it.
    // This method returns false if the specified layer is not found.
    pub fn set_clipping_rect(&mut self, pipeline_id: PipelineId, new_rect: Rect<f32>) -> bool {
        match self.children.iter().position(|x| pipeline_id == x.child.pipeline.id) {
            Some(i) => {
                let child_node = &mut self.children[i];
                let con = child_node.container;
                con.common.set_transform(identity().translate(new_rect.origin.x,
                                                              new_rect.origin.y,
                                                              0.0));
                let old_rect = con.scissor;
                con.scissor = Some(new_rect);
                match self.quadtree {
                    NoTree(..) => {} // Nothing to do
                        Tree(ref mut quadtree) => {
                        match old_rect {
                            Some(old_rect) => {
                                quadtree.set_status_page(old_rect, Normal, false); // Rect is unhidden
                            }
                            None => {} // Nothing to do
                        }
                        quadtree.set_status_page(new_rect, Hidden, false); // Hide the new rect
                    }
                }
                // If possible, unhide child
                if child_node.child.hidden && child_node.child.page_size.is_some() {
                    child_node.child.hidden = false;
                }
                true
            }
            None => {
                // ID does not match any of our immediate children, so recurse on 
                // descendents (including hidden children)
                self.children.mut_iter().map(|x| &mut x.child).any(|x| x.set_clipping_rect(pipeline_id, new_rect))
            }
        }
    }


    // Set the layer's page size. This signals that the renderer is ready for BufferRequests.
    // If the layer is hidden and has a defined clipping rect, unhide it.
    // This method returns false if the specified layer is not found.
    pub fn resize(&mut self, pipeline_id: PipelineId, new_size: Size2D<f32>, window_size: Size2D<f32>, epoch: Epoch) -> bool {
        if self.pipeline.id == pipeline_id {
            self.epoch = epoch;
            self.page_size = Some(new_size);
            match self.quadtree {
                Tree(ref mut quadtree) => {
                    self.pipeline.render_chan.send(UnusedBufferMsg(quadtree.resize(new_size.width as uint,
                                                                                   new_size.height as uint)));
                }
                NoTree(tile_size, max_mem) => {
                    self.quadtree = Tree(Quadtree::new(Size2D(new_size.width as uint,
                                                              new_size.height as uint),
                                                       tile_size,
                                                       max_mem))
                }
            }
            // Call scroll for bounds checking if the page shrunk. Use (-1, -1) as the cursor position
            // to make sure the scroll isn't propagated downwards.
            self.scroll(Point2D(0f32, 0f32), Point2D(-1f32, -1f32), window_size);
            self.hidden = false;
            self.set_occlusions();
            true
        } else {
            self.resize_helper(pipeline_id, new_size, epoch)
        }
    }

    pub fn move(&mut self, origin: Point2D<f32>, window_size: Size2D<f32>) -> bool {
        match self.scroll_behavior {
            Scroll => {
                // Scroll this layer!
                let old_origin = self.scroll_offset;
                self.scroll_offset = Point2D(0f32, 0f32) - origin;

                // bounds checking
                let page_size = match self.page_size {
                    Some(size) => size,
                    None => fail!("CompositorLayer: tried to scroll with no page size set"),
                };
                let min_x = (window_size.width - page_size.width).min(&0.0);
                self.scroll_offset.x = self.scroll_offset.x.clamp(&min_x, &0.0);
                let min_y = (window_size.height - page_size.height).min(&0.0);
                self.scroll_offset.y = self.scroll_offset.y.clamp(&min_y, &0.0);

                // check to see if we scrolled
                if old_origin - self.scroll_offset == Point2D(0f32, 0f32) {
                    return false;
                }

                self.root_layer.common.set_transform(identity().translate(self.scroll_offset.x,
                                                                          self.scroll_offset.y,
                                                                          0.0));
                true
            }
            FixedPosition => false  // Ignore this scroll event.
        }
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
    
    // A helper method to resize sublayers.
    fn resize_helper(&mut self, pipeline_id: PipelineId, new_size: Size2D<f32>, epoch: Epoch) -> bool {
        let found = match self.children.iter().position(|x| pipeline_id == x.child.pipeline.id) {
            Some(i) => {
                let child_node = &mut self.children[i];
                let child = &mut child_node.child;
                child.epoch = epoch;
                child.page_size = Some(new_size);
                match child.quadtree {
                    Tree(ref mut quadtree) => {
                        child.pipeline.render_chan.send(UnusedBufferMsg(quadtree.resize(new_size.width as uint,
                                                                                        new_size.height as uint)));
                    }
                    NoTree(tile_size, max_mem) => {
                        child.quadtree = Tree(Quadtree::new(Size2D(new_size.width as uint,
                                                                   new_size.height as uint),
                                                            tile_size,
                                                            max_mem))
                    }
                }
                match child_node.container.scissor {
                    Some(scissor) => {
                        // Call scroll for bounds checking if the page shrunk. Use (-1, -1) as the cursor position
                        // to make sure the scroll isn't propagated downwards.
                        child.scroll(Point2D(0f32, 0f32), Point2D(-1f32, -1f32), scissor.size);
                        child.hidden = false;
                    }
                    None => {} // Nothing to do
                }
                true
            }
            None => false,
        }; 
                
        if found { // Boolean flag to get around double borrow of self
            self.set_occlusions();
            true
        } else {
            // ID does not match ours, so recurse on descendents (including hidden children)
            self.children.mut_iter().map(|x| &mut x.child).any(|x| x.resize_helper(pipeline_id, new_size, epoch))
        }
    }

    // Collect buffers from the quadtree. This method IS NOT recursive, so child CompositorLayers
    // are not rebuilt directly from this method.
    pub fn build_layer_tree(&mut self, graphics_context: &NativeCompositingGraphicsContext) {
        // Iterate over the children of the container layer.
        let mut current_layer_child = self.root_layer.first_child;
        
        // Delete old layer.
        while current_layer_child.is_some() {
            let trash = current_layer_child.unwrap();
            current_layer_child.unwrap().with_common(|common| {
                current_layer_child = common.next_sibling;
            });
            self.root_layer.remove_child(trash);
        }

        // Add new tiles.
        let quadtree = match self.quadtree {
            NoTree(..) => fail!("CompositorLayer: cannot build layer tree for {:?},
                               no quadtree initialized", self.pipeline.id),
            Tree(ref mut quadtree) => quadtree,
        };

        let all_tiles = quadtree.get_all_tiles();
        for buffer in all_tiles.iter() {
            debug!("osmain: compositing buffer rect {}", buffer.rect);

            let size = Size2D(buffer.screen_pos.size.width as int,
                              buffer.screen_pos.size.height as int);

            // Find or create a texture layer.
            let texture_layer;
            current_layer_child = match current_layer_child {
                None => {
                    debug!("osmain: adding new texture layer");

                    // Determine, in a platform-specific way, whether we should flip the texture
                    // and which target to use.
                    let (flip, target) =
                            CompositorLayer::texture_flip_and_target(self.cpu_painting,
                                                                     buffer.screen_pos.size);

                    // Make a new texture and bind the layer buffer's surface to it.
                    let texture = Texture::new(target);
                    debug!("COMPOSITOR binding to native surface {:d}",
                           buffer.native_surface.get_id() as int);
                    buffer.native_surface.bind_to_texture(graphics_context, &texture, size);

                    // Make a texture layer and add it.
                    texture_layer = @mut TextureLayer::new(texture, buffer.screen_pos.size, flip);
                    self.root_layer.add_child_end(TextureLayerKind(texture_layer));
                    None
                }
                Some(TextureLayerKind(existing_texture_layer)) => {
                    texture_layer = existing_texture_layer;

                    let texture = &texture_layer.texture;
                    buffer.native_surface.bind_to_texture(graphics_context, texture, size);

                    // Move on to the next sibling.
                    current_layer_child.unwrap().with_common(|common| {
                        common.next_sibling
                    })
                }
                Some(_) => fail!(~"found unexpected layer kind"),
            };

            // Set the layer's transform.
            let rect = buffer.rect;
            let transform = identity().translate(rect.origin.x, rect.origin.y, 0.0);
            let transform = transform.scale(rect.size.width, rect.size.height, 1.0);
            texture_layer.common.set_transform(transform);
        }

        // Add child layers.
        for child in self.children.mut_iter().filter(|x| !x.child.hidden) {
            current_layer_child = match current_layer_child {
                None => {
                    child.container.common.parent = None;
                    child.container.common.prev_sibling = None;
                    child.container.common.next_sibling = None;
                    self.root_layer.add_child_end(ContainerLayerKind(child.container));
                    None
                }
                Some(_) => {
                    fail!("CompositorLayer: Layer tree failed to delete");
                }
            };
        }
    }
    
    // Add LayerBuffers to the specified layer. Returns the layer buffer set back if the layer that
    // matches the given pipeline ID was not found; otherwise returns None and consumes the layer
    // buffer set.
    //
    // If the epoch of the message does not match the layer's epoch, the message is ignored, the
    // layer buffer set is consumed, and None is returned.
    pub fn add_buffers(&mut self,
                       graphics_context: &NativeCompositingGraphicsContext,
                       pipeline_id: PipelineId,
                       mut new_buffers: ~LayerBufferSet,
                       epoch: Epoch)
                       -> Option<~LayerBufferSet> {
        if self.pipeline.id == pipeline_id {
            if self.epoch != epoch {
                debug!("compositor epoch mismatch: {:?} != {:?}, id: {:?}",
                       self.epoch,
                       epoch,
                       self.pipeline.id);
                self.pipeline.render_chan.send(UnusedBufferMsg(new_buffers.buffers));
                return None;
            }

            {
                // Block here to prevent double mutable borrow of self.
                let quadtree = match self.quadtree {
                    NoTree(..) => fail!("CompositorLayer: cannot add buffers, no quadtree initialized"),
                    Tree(ref mut quadtree) => quadtree,
                };
                
                let mut unused_tiles = ~[];
                // move_rev_iter is more efficient
                for buffer in new_buffers.buffers.move_rev_iter() {
                    unused_tiles.push_all_move(quadtree.add_tile_pixel(buffer.screen_pos.origin.x,
                                                                       buffer.screen_pos.origin.y,
                                                                       buffer.resolution, buffer));
                }
                if !unused_tiles.is_empty() { // send back unused buffers
                    self.pipeline.render_chan.send(UnusedBufferMsg(unused_tiles));
                }
            }
            self.build_layer_tree(graphics_context);
            return None;
        }

        // ID does not match ours, so recurse on descendents (including hidden children).
        for child_layer in self.children.mut_iter() {
            match child_layer.child.add_buffers(graphics_context,
                                                pipeline_id,
                                                new_buffers,
                                                epoch) {
                None => return None,
                Some(buffers) => new_buffers = buffers,
            }
        }

        // Not found. Give the caller the buffers back.
        Some(new_buffers)
    }

    // Deletes a specified sublayer, including hidden children. Returns false if the layer is not found.
    pub fn delete(&mut self,
                  graphics_context: &NativeCompositingGraphicsContext,
                  pipeline_id: PipelineId)
                  -> bool {
        match self.children.iter().position(|x| x.child.pipeline.id == pipeline_id) {
            Some(i) => {
                let mut child = self.children.remove(i);
                match self.quadtree {
                    NoTree(..) => {} // Nothing to do
                    Tree(ref mut quadtree) => {
                        match child.container.scissor {
                            Some(rect) => {
                                quadtree.set_status_page(rect, Normal, false); // Unhide this rect
                            }
                            None => {} // Nothing to do
                        }
                    }
                }

                // Send back all tiles to renderer.
                child.child.clear();

                self.build_layer_tree(graphics_context);
                true
            }
            None => {
                self.children.mut_iter().map(|x| &mut x.child)
                                        .any(|x| x.delete(graphics_context, pipeline_id))
            }
        }
    }

    pub fn invalidate_rect(&mut self, pipeline_id: PipelineId, rect: Rect<f32>) -> bool {
        if self.pipeline.id == pipeline_id {
            let quadtree = match self.quadtree {
                NoTree(..) => return true, // Nothing to do
                Tree(ref mut quadtree) => quadtree,
            };
            quadtree.set_status_page(rect, Invalid, true);
            true
        } else {
            // ID does not match ours, so recurse on descendents (including hidden children).
            self.children.mut_iter().map(|x| &mut x.child).any(|x| x.invalidate_rect(pipeline_id, rect))
        }
    }
    
    // Recursively sets occluded portions of quadtrees to Hidden, so that they do not ask for
    // tile requests. If layers are moved, resized, or deleted, these portions may be updated.
    fn set_occlusions(&mut self) {
        let quadtree = match self.quadtree {
            NoTree(..) => return, // Cannot calculate occlusions
            Tree(ref mut quadtree) => quadtree,
        };
        for child in self.children.iter().filter(|x| !x.child.hidden) {
            match child.container.scissor {
                None => {} // Nothing to do
                Some(rect) => {
                    quadtree.set_status_page(rect, Hidden, false);
                }
            }
        }
        for child in self.children.mut_iter().filter(|x| !x.child.hidden) {
            child.child.set_occlusions();
        }
    }

    /// Destroys all quadtree tiles, sending the buffers back to the renderer to be destroyed or
    /// reused.
    fn clear(&mut self) {
        match self.quadtree {
            NoTree(..) => {}
            Tree(ref mut quadtree) => {
                let mut tiles = quadtree.collect_tiles();

                // We have no way of knowing without a race whether the render task is even up and
                // running, but mark the tiles as not leaking. If the render task died, then the
                // tiles are going to be cleaned up.
                for tile in tiles.mut_iter() {
                    tile.mark_wont_leak()
                }

                self.pipeline.render_chan.send(UnusedBufferMsg(tiles))
            }
        }
    }

    /// Destroys all quadtree tiles of all layers, including child layers, sending the buffers
    /// back to the renderer to be destroyed or reused.
    pub fn clear_all(&mut self) {
        self.clear();

        for kid in self.children.mut_iter() {
            kid.child.clear_all()
        }
    }

    /// Destroys all tiles of all layers, including children, *without* sending them back to the
    /// renderer. You must call this only when the render task is destined to be going down;
    /// otherwise, you will leak tiles.
    ///
    /// This is used during shutdown, when we know the render task is going away.
    pub fn forget_all_tiles(&mut self) {
        match self.quadtree {
            NoTree(..) => {}
            Tree(ref mut quadtree) => {
                let tiles = quadtree.collect_tiles();
                for tile in tiles.move_iter() {
                    let mut tile = tile;
                    tile.mark_wont_leak()
                }
            }
        }

        for kid in self.children.mut_iter() {
            kid.child.forget_all_tiles();
        }
    }
}

