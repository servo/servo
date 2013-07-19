/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use geom::point::Point2D;
use geom::size::Size2D;
use geom::rect::Rect;
use geom::matrix::identity;
use gfx::render_task::BufferRequest;
use servo_msg::compositor_msg::{LayerBuffer, LayerBufferSet};
use compositing::quadtree::Quadtree;
use layers::layers::{ContainerLayer, TextureLayerKind, TextureLayer, TextureManager};

pub struct CompositorLayer {
    id: uint,
    page_rect: Rect<f32>,
    z_order: int,
    quadtree: Quadtree<~LayerBuffer>,
    root_layer: @mut ContainerLayer,
}

pub fn CompositorLayer(id: uint, page_size: Size2D<f32>, init_offset: Point2D<f32>, z_order: int, tile_size: uint, max_mem: Option<uint>) -> CompositorLayer {
    CompositorLayer {
        id: id,
        page_rect: Rect(init_offset, page_size),
        z_order: z_order,
        quadtree: Quadtree::new(page_size.width as uint, page_size.height as uint,
                                tile_size,
                                max_mem),
        root_layer: @mut ContainerLayer(),
    }
}

impl CompositorLayer {
    // Given the current window size, determine which tiles need to be redisplayed.
    // Returns a BufferRequest array as well as a bool that is true if the layer tree should
    // be rebuilt for this layer.
    fn get_buffer_request(&mut self, window_size: Size2D<int>, scale: f32) -> (~[BufferRequest], bool) {
        let rect = Rect(Point2D(-(self.page_rect.origin.x as int),
                                -(self.page_rect.origin.y as int)),
                        window_size);
        self.quadtree.get_tile_rects(rect, scale)
    }
    
    // Move the layer by as relative specified amount. Called during a scroll event.
    fn translate(&mut self, delta: Point2D<f32>) {
        self.page_rect.origin = self.page_rect.origin + delta;
    }
    
    // Move the layer to an absolute position.
    fn set_offset(&mut self, new_offset: Point2D<f32>) {
        self.page_rect.origin = new_offset;
    }

    // Called when the layer changes size (NOT as a result of a zoom event).
    fn resize(&mut self, new_size: Size2D<f32>) {
        self.page_rect.size = new_size;
        // TODO: might get buffers back here
        self.quadtree.resize(new_size.width as uint, new_size.height as uint);
    }
    
    fn get_layer_tree(&mut self, scale: f32) -> @mut ContainerLayer {
        // Iterate over the children of the container layer.
        let mut current_layer_child = self.root_layer.first_child;
        
        // Delete old layer
        while current_layer_child.is_some() {
            let trash = current_layer_child.get();
            do current_layer_child.get().with_common |common| {
                current_layer_child = common.next_sibling;
            }
            self.root_layer.remove_child(trash);
        }

        let all_tiles = self.quadtree.get_all_tiles();
        for all_tiles.iter().advance |buffer| {
            let width = buffer.screen_pos.size.width as uint;
            let height = buffer.screen_pos.size.height as uint;
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
            
            let origin = buffer.rect.origin;
            let origin = Point2D(origin.x as f32, origin.y as f32);
            
            // Set the layer's transform.
            let transform = identity().translate(origin.x * scale + self.page_rect.origin.x, origin.y * scale + self.page_rect.origin.y, 0.0);
            let transform = transform.scale(width as f32 * scale / buffer.resolution, height as f32 * scale / buffer.resolution, 1.0);
            texture_layer.common.set_transform(transform);
        }
        
        self.root_layer
    }    
    
    // Add LayerBuffers to this layer.
    // TODO: This may return old buffers, which should be sent back to the renderer.
    fn add_buffers(&mut self, new_buffers: &mut LayerBufferSet) {
        for new_buffers.buffers.iter().advance |buffer| {
            self.quadtree.add_tile(buffer.screen_pos.origin.x, buffer.screen_pos.origin.y,
                                   buffer.resolution, ~buffer.clone());
        }
        
    }

    // TODO: send buffers back to renderer when later is deleted
    
}