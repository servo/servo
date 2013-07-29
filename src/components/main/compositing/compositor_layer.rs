/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use geom::point::Point2D;
use geom::size::Size2D;
use geom::rect::Rect;
use geom::matrix::identity;
use gfx::render_task::ReRenderMsg;
use servo_msg::compositor_msg::{LayerBuffer, LayerBufferSet};
use servo_msg::constellation_msg::PipelineId;
use script::dom::event::{ClickEvent, MouseDownEvent, MouseUpEvent};
use script::script_task::SendEventMsg;
use windowing::{MouseWindowEvent, MouseWindowClickEvent, MouseWindowMouseDownEvent, MouseWindowMouseUpEvent};
use compositing::quadtree::Quadtree;
use layers::layers::{ContainerLayerKind, ContainerLayer, TextureLayerKind, TextureLayer, TextureManager};
use pipeline::Pipeline;

/// The CompositorLayer represents an element on a page that has a unique scroll
/// or animation behavior. This can include absolute positioned elements, iframes, etc.
/// Each layer can also have child elements.
pub struct CompositorLayer {
    /// This layer's pipeline. BufferRequests will be sent through this.
    pipeline: Pipeline,
    /// The rect that this layer occupies in page coordinates. The offset
    /// is blind to any parents this layer may have; it only refers to the
    /// scroll position of the page.
    page_rect: Rect<f32>,
    /// This layer's children. These could be iframes or any element which
    /// differs in scroll behavior from its parent. Each is associated with a
    /// ContainerLayer which determines its position relative to its parent and
    /// clipping rect. Children are stored in the order in which they are drawn.
    children: ~[CompositorLayerChild],
    /// This layer's quadtree. This is where all buffers are stored for this layer.
    quadtree: Quadtree<~LayerBuffer>,
    /// The root layer of this CompositorLayer's layer tree. Buffers are collected
    /// from the quadtree and inserted here when the layer is painted to the screen.
    root_layer: @mut ContainerLayer,
    
    // TODO: Eventually, it may be useful to have an ID associated with each layer.
}

/// Helper struct for keeping CompositorLayer children organized.
struct CompositorLayerChild {
    /// The child itself.
    child: ~CompositorLayer, 
    /// A ContainerLayer managed by the parent node. This deals with clipping and
    /// positioning, and is added above the child's layer tree.
    container: @mut ContainerLayer,
}

pub fn CompositorLayer(pipeline: Pipeline, page_size: Size2D<f32>, tile_size: uint,
                       max_mem: Option<uint>) -> CompositorLayer {
    CompositorLayer {
        pipeline: pipeline,
        page_rect: Rect(Point2D(0f32, 0f32), page_size),
        children: ~[],
        quadtree: Quadtree::new(page_size.width as uint, page_size.height as uint,
                                tile_size,
                                max_mem),
        root_layer: @mut ContainerLayer(),
    }
}

impl CompositorLayer {    
    // Move the layer by as relative specified amount in page coordinates. Does not change
    // the position of the layer relative to its parent. This also takes in a cursor position
    // to see if the mouse is over child layers first. If a layer successfully scrolled, returns
    // true; otherwise returns false, so a parent layer can scroll instead.
    pub fn scroll(&mut self, delta: Point2D<f32>, cursor: Point2D<f32>, window_size: Size2D<f32>) -> bool {
        let cursor = cursor - self.page_rect.origin;
        for self.children.mut_iter().advance |child| {
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

        let old_origin = self.page_rect.origin;
        self.page_rect.origin = self.page_rect.origin + delta;
        
        // bounds checking
        let min_x = (window_size.width - self.page_rect.size.width).min(&0.0);
        self.page_rect.origin.x = self.page_rect.origin.x.clamp(&min_x, &0.0);
        let min_y = (window_size.height - self.page_rect.size.height).min(&0.0);
        self.page_rect.origin.y = self.page_rect.origin.y.clamp(&min_y, &0.0);
        
        // check to see if we scrolled
        if old_origin - self.page_rect.origin == Point2D(0f32, 0f32) {
            return false;
        }

        self.root_layer.common.set_transform(identity().translate(self.page_rect.origin.x,
                                                                  self.page_rect.origin.y,
                                                                  0.0));
        true
    }

    // Takes in a MouseWindowEvent, determines if it should be passed to children, and 
    // sends the event off to the appropriate pipeline. NB: the cursor position is in
    // page coordinates.
    pub fn send_mouse_event(&self, event: MouseWindowEvent, cursor: Point2D<f32>) {
        let cursor = cursor - self.page_rect.origin;
        // FIXME: maybe we want rev_iter() instead? Depends on draw order
        for self.children.iter().advance |child| {
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
    
    // Given the current window size, determine which tiles need to be redisplayed
    // and sends them off the the appropriate renderer.
    // Returns a bool that is true if the scene should be repainted.
    pub fn get_buffer_request(&mut self, window_rect: Rect<f32>, scale: f32) -> bool {
        let rect = Rect(Point2D(-self.page_rect.origin.x + window_rect.origin.x,
                                -self.page_rect.origin.y + window_rect.origin.y),
                        window_rect.size);
        let (request, redisplay) = self.quadtree.get_tile_rects_page(rect, scale);
        if !request.is_empty() {
            self.pipeline.render_chan.send(ReRenderMsg(request, scale, self.pipeline.id.clone()));
        }
        if redisplay {
            self.build_layer_tree();
        }
        let transform = |x: &mut CompositorLayerChild| -> bool {
            match x.container.scissor {
                Some(scissor) => {
                    let new_rect = window_rect.intersection(&scissor);
                    match new_rect {
                        Some(new_rect) => {
                            x.child.get_buffer_request(new_rect, scale)
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
        self.children.mut_iter()
            .transform(transform)
            .fold(false, |a, b| a || b) || redisplay
    }


    // Move the sublayer to an absolute position in page coordinates relative to its parent,
    // and clip the layer to the specified size in page coordinates.
    // This method returns false if the specified layer is not found.
    pub fn set_clipping_rect(&mut self, pipeline_id: PipelineId, new_rect: Rect<f32>) -> bool {
            for self.children.iter().advance |child_node| {
            if pipeline_id != child_node.child.pipeline.id {
                loop;
            }
            let con = child_node.container;
            con.common.set_transform(identity().translate(new_rect.origin.x,
                                                          new_rect.origin.y,
                                                          0.0));
            con.scissor = Some(new_rect);
            return true;
        }
        
        // ID does not match any of our immediate children, so recurse on descendents.
        self.children.mut_iter().transform(|x| &mut x.child).any(|x| x.set_clipping_rect(pipeline_id, new_rect))
    }


    // Called when the layer changes size (NOT as a result of a zoom event).
    // This method returns false if the specified layer is not found.
    pub fn resize(&mut self, pipeline_id: PipelineId, new_size: Size2D<f32>, window_size: Size2D<f32>) -> bool {
        if self.pipeline.id == pipeline_id {
            self.page_rect.size = new_size;
            // TODO: might get buffers back here
            self.quadtree.resize(new_size.width as uint, new_size.height as uint);
            // Call scroll for bounds checking of the page shrunk.
            self.scroll(Point2D(0f32, 0f32), Point2D(-1f32, -1f32), window_size);
            return true;
        }

        // ID does not match ours, so recurse on descendents.
        let transform = |x: &mut CompositorLayerChild| -> bool {
            match x.container.scissor {
                Some(scissor) => {
                    x.child.resize(pipeline_id, new_size, scissor.size)
                }
                None => {
                    fail!("CompositorLayer: Child layer not clipped");
                }
            }
        };

        self.children.mut_iter().any(transform)
    }
    
    // Collect buffers from the quadtree. This method IS NOT recursive, so child CompositorLayers
    // are not rebuilt directly from this method.
    pub fn build_layer_tree(&mut self) {
        // Iterate over the children of the container layer.
        let mut current_layer_child = self.root_layer.first_child;
        
        // Delete old layer.
        while current_layer_child.is_some() {
            let trash = current_layer_child.get();
            do current_layer_child.get().with_common |common| {
                current_layer_child = common.next_sibling;
            }
            self.root_layer.remove_child(trash);
        }

        // Add child layers.
        for self.children.mut_iter().advance |child| {
            current_layer_child = match current_layer_child {
                None => {
                    child.container.common.parent = None;
                    child.container.common.prev_sibling = None;
                    child.container.common.next_sibling = None;
                    self.root_layer.add_child(ContainerLayerKind(child.container));
                    None
                }
                Some(_) => {
                    fail!("CompositorLayer: Layer tree failed to delete");
                }
            };
        }
        
        // Add new tiles.
        let all_tiles = self.quadtree.get_all_tiles();
        for all_tiles.iter().advance |buffer| {
            debug!("osmain: compositing buffer rect %?", &buffer.rect);
            
            // Find or create a texture layer.
            let texture_layer;
            current_layer_child = match current_layer_child {
                None => {
                    debug!("osmain: adding new texture layer");
                    texture_layer = @mut TextureLayer::new(@buffer.draw_target.clone() as @TextureManager,
                                                           buffer.screen_pos.size);
                    self.root_layer.add_child(TextureLayerKind(texture_layer));
                    None
                }
                Some(TextureLayerKind(existing_texture_layer)) => {
                    texture_layer = existing_texture_layer;
                    texture_layer.manager = @buffer.draw_target.clone() as @TextureManager;
                    
                    // Move on to the next sibling.
                    do current_layer_child.get().with_common |common| {
                        common.next_sibling
                    }
                }
                Some(_) => fail!(~"found unexpected layer kind"),
            };
            

            let rect = buffer.rect;
            // Set the layer's transform.
            let transform = identity().translate(rect.origin.x, rect.origin.y, 0.0);
            let transform = transform.scale(rect.size.width, rect.size.height, 1.0);
            texture_layer.common.set_transform(transform);
        }

    }
    
    // Add LayerBuffers to the specified layer. Returns false if the layer is not found.
    pub fn add_buffers(&mut self, pipeline_id: PipelineId, new_buffers: &LayerBufferSet) -> bool {
        if self.pipeline.id == pipeline_id {
            for new_buffers.buffers.iter().advance |buffer| {
                // TODO: This may return old buffers, which should be sent back to the renderer.
                self.quadtree.add_tile_pixel(buffer.screen_pos.origin.x, buffer.screen_pos.origin.y,
                                             buffer.resolution, ~buffer.clone());
            }
            self.build_layer_tree();
            return true;
        }
        // ID does not match ours, so recurse on descendents.
        self.children.mut_iter().transform(|x| &mut x.child).any(|x| x.add_buffers(pipeline_id, new_buffers))
    }

    // Deletes a specified sublayer. Returns false if the layer is not found.
    pub fn delete(&mut self, pipeline_id: PipelineId) -> bool {
        match self.children.rposition(|x| x.child.pipeline.id == pipeline_id) {
            Some(index) => {
                // TODO: send buffers back to renderer when layer is deleted
                self.children.remove(index);
                self.build_layer_tree();
                true
            }
            None => {
                self.children.mut_iter().transform(|x| &mut x.child).any(|x| x.delete(pipeline_id))
            }
        }
    }
    

    // Adds a child.
    pub fn add_child(&mut self, pipeline: Pipeline, page_size: Size2D<f32>, tile_size: uint,
                     max_mem: Option<uint>, clipping_rect: Rect<f32>) {
        let container = @mut ContainerLayer();
        container.scissor = Some(clipping_rect);
        container.common.set_transform(identity().translate(clipping_rect.origin.x,
                                                            clipping_rect.origin.y,
                                                            0.0));
        let child = ~CompositorLayer(pipeline, page_size, tile_size, max_mem);
        container.add_child(ContainerLayerKind(child.root_layer));
        self.children.push(CompositorLayerChild {
            child: child,
            container: container,
        });
        
    }
}
