/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

// Implements a Quadtree data structure to keep track of which tiles have
// been rasterized and which have not.

use geom::point::Point2D;
use geom::size::Size2D;
use geom::rect::Rect;
use gfx::render_task::BufferRequest;
use std::uint::{div_ceil, next_power_of_two};
use std::vec;
use std::util::replace;
use std::vec::build;
use servo_msg::compositor_msg::Tile;

#[cfg(test)]
use layers::platform::surface::NativePaintingGraphicsContext;

/// Parent to all quadtree nodes. Stores variables needed at all levels. All method calls
/// at this level are in pixel coordinates.
pub struct Quadtree<T> {
    // The root node of the quadtree
    root: ~QuadtreeNode<T>,
    // The size of the layer in pixels. Tiles will be clipped to this size.
    // Note that the underlying quadtree has a potentailly larger size, since it is rounded
    // to the next highest power of two.
    clip_size: Size2D<uint>,
    // The maximum size of the tiles requested in pixels. Tiles requested will be
    // of a size anywhere between half this value and this value.
    max_tile_size: uint,
    // The maximum allowed total memory of tiles in the tree. If this limit is reached, tiles
    // will be removed from the tree. Set this to None to prevent this behavior.
    max_mem: Option<uint>,
}

/// A node in the tree. All method calls at this level are in page coordinates.
struct QuadtreeNode<T> {
    /// The tile belonging to this node. Note that parent nodes can have tiles.
    tile: Option<T>,
    /// The position of the node in page coordinates.
    origin: Point2D<f32>,
    /// The width and height of the node in page coordinates.
    size: f32,
    /// The node's children.
    quadrants: [Option<~QuadtreeNode<T>>, ..4],
    /// Combined size of self.tile and tiles of all descendants
    tile_mem: uint,
    /// The current status of this node. See below for details.
    status: NodeStatus,
}

/// The status of a QuadtreeNode. This determines the behavior of the node
/// when querying for tile requests.
#[deriving(Eq)]
pub enum NodeStatus {
    /// If we have no valid tile, request one; otherwise, don't send a request.
    Normal,
    /// Render request has been sent; ignore this node until tile is inserted.
    Rendering,
    /// Do not send tile requests. Overrides Invalid.
    Hidden,
    /// Send tile requests, even if the node has (or child nodes have) a valid tile.
    Invalid,
}

enum Quadrant {
    TL = 0,
    TR = 1,
    BL = 2,
    BR = 3,
}

impl<T: Tile> Quadtree<T> {
    /// Public method to create a new Quadtree
    /// Takes in the initial width and height of the space, a maximum tile size, and
    /// a maximum amount of memory. Tiles will be deleted if this memory is exceeded.
    /// Set max_mem to None to turn off automatic tile removal.
    pub fn new(clip_size: Size2D<uint>, tile_size: uint, max_mem: Option<uint>) -> Quadtree<T> {
        // Spaces must be squares and powers of 2, so expand the space until it is
        let longer = clip_size.width.max(&clip_size.height);
        let num_tiles = div_ceil(longer, tile_size);
        let power_of_two = next_power_of_two(num_tiles);
        let size = power_of_two * tile_size;
        
        Quadtree {
            root: ~QuadtreeNode {
                tile: None,
                origin: Point2D(0f32, 0f32),
                size: size as f32,
                quadrants: [None, None, None, None],
                tile_mem: 0,
                status: Normal,
            },
            clip_size: clip_size,
            max_tile_size: tile_size,
            max_mem: max_mem,
        }
    }

    /// Add a tile associated with a given pixel position and scale.
    /// If the tile pushes the total memory over its maximum, tiles will be removed
    /// until total memory is below the maximum again. These tiles are returned.
    pub fn add_tile_pixel(&mut self, x: uint, y: uint, scale: f32, tile: T) -> ~[T] {
        let (_, tiles) = self.root.add_tile(x as f32 / scale, y as f32 / scale, tile,
                                            self.max_tile_size as f32 / scale);
        let mut tiles = tiles;
        match self.max_mem {
            Some(max) => {
                while self.root.tile_mem > max {
                    let r = self.root.remove_tile(x as f32 / scale, y as f32 / scale);
                    match r {
                        (Some(tile), _, _) => tiles.push(tile),
                        _ => fail!("Quadtree: No valid tiles to remove"),
                    }
                }
            }
            None => {} // Nothing to do
        }
        tiles
    }

    /// Get all the tiles in the tree.
    pub fn get_all_tiles<'r>(&'r self) -> ~[&'r T] {
        self.root.get_all_tiles()
    }

    /// Given a window rect in pixel coordinates, this function returns a list of BufferRequests for tiles that
    /// need to be rendered. It also returns a vector of tiles if the window needs to be redisplayed, i.e. if
    /// no tiles need to be rendered, but the display tree needs to be rebuilt. This can occur when the
    /// user zooms out and cached tiles need to be displayed on top of higher resolution tiles.
    /// When this happens, higher resolution tiles will be removed from the quadtree.
    #[cfg(test)]
    pub fn get_tile_rects_pixel(&mut self, window: Rect<int>, scale: f32) -> (~[BufferRequest], ~[T]) {
        let (ret, unused, _) = self.root.get_tile_rects(
            Rect(Point2D(window.origin.x as f32 / scale, window.origin.y as f32 / scale),
                 Size2D(window.size.width as f32 / scale, window.size.height as f32 / scale)),
            Size2D(self.clip_size.width as f32, self.clip_size.height as f32),
            scale, self.max_tile_size as f32 / scale, false);
        (ret, unused)
    }

    /// Same function as above, using page coordinates for the window.
    pub fn get_tile_rects_page(&mut self, window: Rect<f32>, scale: f32) -> (~[BufferRequest], ~[T]) {
        let (ret, unused, _) = self.root.get_tile_rects(
            window,
            Size2D(self.clip_size.width as f32, self.clip_size.height as f32),
            scale, self.max_tile_size as f32 / scale, false);
        (ret, unused)
    }

    /// Creates a new quadtree at the specified size. This should be called when the window changes size.
    pub fn resize(&mut self, width: uint, height: uint) -> ~[T] {
        // Spaces must be squares and powers of 2, so expand the space until it is
        let longer = width.max(&height);
        let num_tiles = div_ceil(longer, self.max_tile_size);
        let power_of_two = next_power_of_two(num_tiles);
        let size = power_of_two * self.max_tile_size;
        let ret = self.root.collect_tiles();
        self.root = ~QuadtreeNode {
            tile: None,
            origin: Point2D(0f32, 0f32),
            size: size as f32,
            quadrants: [None, None, None, None],
            status: Normal,
            tile_mem: 0,
        };
        self.clip_size = Size2D(width, height);
        ret
    }

    /// Resize the underlying quadtree without removing tiles already in place.
    /// Might be useful later on, but resize() should be used for now.
    /// TODO: return tiles after shrinking
    #[cfg(test)]
    pub fn bad_resize(&mut self, width: uint, height: uint) {
        self.clip_size = Size2D(width, height);
        let longer = width.max(&height);
        let new_num_tiles = div_ceil(longer, self.max_tile_size);
        let new_size = next_power_of_two(new_num_tiles);
        // difference here indicates the number of times the underlying size of the quadtree needs
        // to be doubled or halved. It will recursively add a new root if it is positive, or
        // recursivly make a child the new root if it is negative.
        let difference = (new_size as f32 / self.root.size as f32).log2() as int;
        if difference > 0 { // doubling
            let difference = difference as uint;
            for i in range(0, difference) {
                let new_root = ~QuadtreeNode {
                    tile: None,
                    origin: Point2D(0f32, 0f32),
                    size: new_size as f32 / ((difference - i - 1) as f32).exp2(),
                    quadrants: [None, None, None, None],
                    tile_mem: self.root.tile_mem,
                    status: Normal,
                };
                self.root.quadrants[TL as int] = Some(replace(&mut self.root, new_root));
            }
        } else if difference < 0 { // halving
            let difference = difference.abs() as uint;
            for _ in range(0, difference) {
                let remove = replace(&mut self.root.quadrants[TL as int], None);
                match remove {
                    Some(child) => self.root = child,
                    None => {
                        self.root = ~QuadtreeNode {
                            tile: None,
                            origin: Point2D(0f32, 0f32),
                            size: new_size as f32,
                            quadrants: [None, None, None, None],
                            tile_mem: 0,
                            status: Normal,
                        };
                        break;
                    }
                }
            }
        }
    }

    /// Set the status of all quadtree nodes within the given rect in page coordinates. If
    /// include_border is true, then nodes on the edge of the rect will be included; otherwise,
    /// only nodes completely occluded by the rect will be changed.
    pub fn set_status_page(&mut self, rect: Rect<f32>, status: NodeStatus, include_border: bool) {
        self.root.set_status(rect, status, include_border);
    }

    /// Remove and return all tiles in the tree. Use this before deleting the quadtree to prevent
    /// a GC pause.
    pub fn collect_tiles(&mut self) -> ~[T] {
        self.root.collect_tiles()
    }
}

impl<T: Tile> QuadtreeNode<T> {
    /// Private method to create new children
    fn new_child(x: f32, y: f32, size: f32) -> QuadtreeNode<T> {
        QuadtreeNode {
            tile: None,
            origin: Point2D(x, y),
            size: size,
            quadrants: [None, None, None, None],
            tile_mem: 0,
            status: Normal,
        }
    }

    /// Determine which child contains a given point in page coords.
    fn get_quadrant(&self, x: f32, y: f32) -> Quadrant {
        if x < self.origin.x + self.size / 2.0 {
            if y < self.origin.y + self.size / 2.0 { 
                TL
            } else { 
                BL
            }
        } else if y < self.origin.y + self.size / 2.0 { 
            TR
        } else { 
            BR
        }
    }

    /// Get all tiles in the tree, parents first.
    fn get_all_tiles<'r>(&'r self) -> ~[&'r T] {
        let mut ret = ~[];

        match self.tile {
            Some(ref tile) => ret = ret + ~[tile],
            None => {}
        }

        for quad in self.quadrants.iter() {
            match *quad {
                Some(ref child) => ret = ret + child.get_all_tiles(),
                None => {}
            }
        }
        
        return ret;
    }

    /// Add a tile associated with a given position in page coords. If the tile size exceeds the maximum,
    /// the node will be split and the method will recurse until the tile size is within limits.
    /// Returns an the difference in tile memory between the new quadtree node and the old quadtree node,
    /// along with any deleted tiles.
    fn add_tile(&mut self, x: f32, y: f32, tile: T, tile_size: f32) -> (int, ~[T]) {
        debug!("Quadtree: Adding: ({}, {}) size:{}px", self.origin.x, self.origin.y, self.size);

        if x >= self.origin.x + self.size || x < self.origin.x
            || y >= self.origin.y + self.size || y < self.origin.y {
            fail!("Quadtree: Tried to add tile to invalid region");
        }
        
        if self.size <= tile_size { // We are the child
            let old_size = self.tile_mem;
            self.tile_mem = tile.get_mem();
            let mut unused_tiles = match replace(&mut self.tile, Some(tile)) {
                Some(old_tile) => ~[old_tile],
                None => ~[],
            };
            for child in self.quadrants.mut_iter() {
                match *child {
                    Some(ref mut node) => {
                        unused_tiles.push_all_move(node.collect_tiles());
                    }
                    None => {} // Nothing to do
                }
                *child = None;
            }
            self.status = Normal;
            (self.tile_mem as int - old_size as int, unused_tiles)
        } else { // Send tile to children            
            let quad = self.get_quadrant(x, y);
            match self.quadrants[quad as int] {
                Some(ref mut child) => {
                    let (delta, unused) = child.add_tile(x, y, tile, tile_size);
                    self.tile_mem = (self.tile_mem as int + delta) as uint;
                    (delta, unused)
                }
                None => { // Make new child      
                    let new_size = self.size / 2.0;
                    let new_x = match quad {
                        TL | BL => self.origin.x,
                        TR | BR => self.origin.x + new_size,
                    };
                    let new_y = match quad {
                        TL | TR => self.origin.y,
                        BL | BR => self.origin.y + new_size,
                    };
                    let mut c = ~QuadtreeNode::new_child(new_x, new_y, new_size);
                    let (delta, unused) = c.add_tile(x, y, tile, tile_size);
                    self.tile_mem = (self.tile_mem as int + delta) as uint;    
                    self.quadrants[quad as int] = Some(c);
                    (delta, unused)
                }
            }
        }
    }

    /// Get a tile rect in screen and page coords for a given position in page coords
    fn get_tile_rect(&mut self, x: f32, y: f32, clip_x: f32, clip_y: f32, scale: f32,
                     tile_size: f32) -> BufferRequest {
        if x >= self.origin.x + self.size || x < self.origin.x
            || y >= self.origin.y + self.size || y < self.origin.y {
            fail!("Quadtree: Tried to query a tile rect outside of range");
        }
        
        if self.size <= tile_size {
            let pix_x = (self.origin.x * scale).ceil() as uint;
            let pix_y = (self.origin.y * scale).ceil() as uint;
            let page_width = (clip_x - self.origin.x).min(&self.size);
            let page_height = (clip_y - self.origin.y).min(&self.size);
            let pix_width = (page_width * scale).ceil() as uint;
            let pix_height = (page_height * scale).ceil() as uint;
            self.status = Rendering;
            return BufferRequest(Rect(Point2D(pix_x, pix_y), Size2D(pix_width, pix_height)),
                                 Rect(Point2D(self.origin.x, self.origin.y), Size2D(page_width, page_height)));
        }
        
        let quad = self.get_quadrant(x,y);
        match self.quadrants[quad as int] {
            None => {
                let new_size = self.size / 2.0;
                let new_x = match quad {
                    TL | BL => self.origin.x,
                    TR | BR => self.origin.x + new_size,
                };
                let new_y = match quad {
                    TL | TR => self.origin.y,
                    BL | BR => self.origin.y + new_size,
                };
                let mut c = ~QuadtreeNode::new_child(new_x, new_y, new_size);
                let result = c.get_tile_rect(x, y, clip_x, clip_y, scale, tile_size);
                self.quadrants[quad as int] = Some(c);
                result
            }
            Some(ref mut child) => child.get_tile_rect(x, y, clip_x, clip_y, scale, tile_size),
        }
    }

    /// Removes a tile that is far from the given input point in page coords. Returns the tile removed,
    /// a bool that is true if the child has no tiles and needs to be deleted, and an integer showing the
    /// amount of memory changed by the operation. Unfortunately, the tile has to be an option, because
    /// there are occasionally leaves without tiles. However, the option will always be Some as long as
    /// this quadtree node or at least one of its descendants is not empty.
    fn remove_tile(&mut self, x: f32, y: f32) -> (Option<T>, bool, int) {
        if self.tile.is_some() {
            let ret = replace(&mut(self.tile), None);
            return match (ret, &self.quadrants)  {
                (Some(tile), &[None, None, None, None]) => { 
                    let size = -(tile.get_mem() as int);
                    (Some(tile), true, size)
                }
                (Some(tile), _) => {
                    let size = -(tile.get_mem() as int);
                    (Some(tile), false, size)
                }
                _ => fail!("Quadtree: tile query failure in remove_tile"),
            }
        }
        
        // This is a hacky heuristic to find a tile that is "far away". There are better methods.
        let quad = self.get_quadrant(x, y);
        let queue = match quad {
            TL => [BR, BL, TR, TL], 
            TR => [BL, BR, TL, TR],
            BL => [TR, TL, BR, BL], 
            BR => [TL, TR, BL, BR],
        };

        let mut del_quad: Option<Quadrant> = None;
        let mut ret = (None, false, 0);
        
        for quad in queue.iter() {
            match self.quadrants[*quad as int] {
                Some(ref mut child) => {
                    let (tile, flag, delta) = child.remove_tile(x, y);
                    match tile {
                        Some(_) => {
                            self.tile_mem = (self.tile_mem as int + delta) as uint;
                            if flag {
                                del_quad = Some(*quad);
                            } else {
                                return (tile, flag, delta);
                            }

                            ret = (tile, flag, delta);
                            break;
                        }
                        None => {},
                    }
                }
                None => {},
            }
        }
        
        match del_quad {
            Some(quad) => {
                self.quadrants[quad as int] = None;
                let (tile, _, delta) = ret;
                match (&self.tile, &self.quadrants) {
                    (&None, &[None, None, None, None]) => (tile, true, delta),
                    _ => (tile, false, delta)
                }
            }
            None => ret,
        }
    }
    
    /// Given a window rect in page coordinates, returns a BufferRequest array,
    /// an unused tile array, and the difference in tile memory between the new and old quadtree nodes.
    /// The override bool will be true if a parent node was marked as invalid; child nodes will be
    /// treated as invalid as well.
    /// NOTE: this method will sometimes modify the tree by deleting tiles.
    /// See the QuadTree function description for more details.
    fn get_tile_rects(&mut self, window: Rect<f32>, clip: Size2D<f32>, scale: f32, tile_size: f32, override: bool) ->
        (~[BufferRequest], ~[T], int) {
        
        let w_x = window.origin.x;
        let w_y = window.origin.y;
        let w_width = window.size.width;
        let w_height = window.size.height;
        let s_x = self.origin.x;
        let s_y = self.origin.y;
        let s_size = self.size;
        
        // if window is outside of visible region, nothing to do
        if w_x + w_width < s_x || w_x > s_x + s_size
            || w_y + w_height < s_y || w_y > s_y + s_size 
            || w_x >= clip.width || w_y >= clip.height {
            return (~[], ~[], 0);
        }
        
        // clip window to visible region
        let w_width = (clip.width - w_x).min(&w_width);
        let w_height = (clip.height - w_y).min(&w_height);
        
        if s_size <= tile_size { // We are the child
            return match self.tile {
                _ if self.status == Rendering || self.status == Hidden => (~[], ~[], 0),
                Some(ref tile) if tile.is_valid(scale) && !override
                && self.status != Invalid => {
                    let redisplay = match self.quadrants {
                        [None, None, None, None] => false,
                        _ => true,
                    };
                    let mut delta = 0;
                    let mut unused_tiles = ~[];
                    if redisplay {
                        let old_mem = self.tile_mem;
                        for child in self.quadrants.mut_iter() {
                            match *child {
                                Some(ref mut node) => {
                                    unused_tiles.push_all_move(node.collect_tiles());
                                }
                                None => {} // Nothing to do
                            }
                            *child = None;
                        }
                        self.tile_mem = tile.get_mem();
                        delta = self.tile_mem as int - old_mem as int;

                    }
                    (~[], unused_tiles, delta)
                }
                _ => (~[self.get_tile_rect(s_x, s_y, clip.width, clip.height, scale, tile_size)], ~[], 0),
            }
        }
        
        // Otherwise, we either have children or will have children
        let w_tl_quad = self.get_quadrant(w_x, w_y);
        let w_br_quad = self.get_quadrant(w_x + w_width, w_y + w_height);
        
        // Figure out which quadrants the window is in
        let builder = |push: |Quadrant|| {
            match (w_tl_quad, w_br_quad) {
                (tl, br) if tl as int == br as int =>  {
                    push(tl);
                }
                (TL, br) => {
                    push(TL);
                    push(br);
                    match br {
                        BR => {
                            push(TR);
                            push(BL);
                        }
                        _ => {}
                    }
                }
                (tl, br) => {
                    push(tl);
                    push(br);
                }
            }
        };
        
        let quads_to_check = vec::build(Some(4), builder);
        
        let mut request = ~[];
        let mut unused = ~[];
        let mut delta = 0;
        
        for quad in quads_to_check.iter() {
            // Recurse into child
            let new_window = match *quad {
                TL => Rect(window.origin,
                           Size2D(w_width.min(&(s_x + s_size / 2.0 - w_x)),
                                  w_height.min(&(s_y + s_size / 2.0 - w_y)))),
                TR => Rect(Point2D(w_x.max(&(s_x + s_size / 2.0)),
                                   w_y),
                           Size2D(w_width.min(&(w_x + w_width - (s_x + s_size / 2.0))),
                                  w_height.min(&(s_y + s_size / 2.0 - w_y)))),
                BL => Rect(Point2D(w_x,
                                   w_y.max(&(s_y + s_size / 2.0))),
                           Size2D(w_width.min(&(s_x + s_size / 2.0 - w_x)),
                                  w_height.min(&(w_y + w_height - (s_y + s_size / 2.0))))),
                BR => Rect(Point2D(w_x.max(&(s_x + s_size / 2.0)),
                                   w_y.max(&(s_y + s_size / 2.0))),
                           Size2D(w_width.min(&(w_x + w_width - (s_x + s_size / 2.0))),
                                  w_height.min(&(w_y + w_height - (s_y + s_size / 2.0))))), 
                
            };

            let override = override || self.status == Invalid;
            self.status = Normal;
            let (c_request, c_unused, c_delta) = match self.quadrants[*quad as int] {
                Some(ref mut child) => child.get_tile_rects(new_window, clip, scale, tile_size, override),
                None => {
                    // Create new child
                    let new_size = self.size / 2.0;
                    let new_x = match *quad {
                        TL | BL => self.origin.x,
                        TR | BR => self.origin.x + new_size,
                    };
                    let new_y = match *quad {
                        TL | TR => self.origin.y,
                        BL | BR => self.origin.y + new_size,
                    };
                    let mut child = ~QuadtreeNode::new_child(new_x, new_y, new_size);
                    let ret = child.get_tile_rects(new_window, clip, scale, tile_size, override);
                    self.quadrants[*quad as int] = Some(child);
                    ret
                }
            };
            
            delta = delta + c_delta;
            request = request + c_request;
            unused.push_all_move(c_unused);
        } 
        self.tile_mem = (self.tile_mem as int + delta) as uint;
        (request, unused, delta)
    }

    /// Remove all tiles from the tree. Use this to collect all tiles before deleting a branch.
    fn collect_tiles(&mut self) -> ~[T] {
        let mut ret = match replace(&mut self.tile, None) {
            Some(tile) => ~[tile],
            None => ~[],
        };
        for child in self.quadrants.mut_iter() {
            match *child {
                Some(ref mut node) => {
                    ret.push_all_move(node.collect_tiles());
                }
                None => {} // Nothing to do
            }
        }
        ret
    }

    /// Set the status of nodes contained within the rect. See the quadtree method for
    /// more info.
    fn set_status(&mut self, rect: Rect<f32>, status: NodeStatus, borders: bool) {
        let self_rect = Rect(self.origin, Size2D(self.size, self.size));
        let intersect = rect.intersection(&self_rect);
        let intersect = match intersect {
            None => return, // We do not intersect the rect, nothing to do
            Some(rect) => rect,
        };

        if self_rect == intersect { // We are completely contained in the rect
            if !(self.status == Hidden && status == Invalid) { // Hidden trumps Invalid
                self.status = status;
            }
            return; // No need to recurse
        }

        match self.quadrants {
            [None, None, None, None] => { // We are a leaf
                if borders && !(self.status == Hidden && status == Invalid) {
                    self.status = status;
                }
            }
            _ => { // We are internal
                for quad in self.quadrants.mut_iter() {
                    match *quad {
                        None => {} // Nothing to do
                        Some(ref mut child) => {
                            child.set_status(intersect, status, borders);
                        }
                    }
                }
            }
        }
    }
}

#[test]
pub fn test_resize() {
    struct T {
        a: int,
    }
    
    impl Tile for T {
        fn get_mem(&self) -> uint {
            1
        }
        
        fn is_valid(&self, _: f32) -> bool {
            true
        }
        fn get_size_2d(&self) -> Size2D<uint> {
            Size2D(0u, 0u)
        }
        fn mark_wont_leak(&mut self) {}
        fn destroy(self, _: &NativePaintingGraphicsContext) {}
    }
    
    let mut q = Quadtree::new(Size2D(6u, 6), 1, None);
    q.add_tile_pixel(0, 0, 1f32, T{a: 0});
    q.add_tile_pixel(5, 5, 1f32, T{a: 1});
    q.bad_resize(8, 1);
    assert!(q.root.size == 8.0);
    q.bad_resize(18, 1);
    assert!(q.root.size == 32.0);
    q.bad_resize(8, 1);
    assert!(q.root.size == 8.0);
    q.bad_resize(3, 1);
    assert!(q.root.size == 4.0);
    assert!(q.get_all_tiles().len() == 1);
}

#[test]
pub fn test() {
    struct T {
        a: int,
    }
    
    impl Tile for T {
        fn get_mem(&self) -> uint {
            1
        }
        
        fn is_valid(&self, _: f32) -> bool {
            true
        }
        fn get_size_2d(&self) -> Size2D<uint> {
            Size2D(0u, 0u)
        }
        fn mark_wont_leak(&mut self) {}
        fn destroy(self, _: &NativePaintingGraphicsContext) {}
    }
    
    let mut q = Quadtree::new(Size2D(8u, 8), 2, Some(4));
    q.add_tile_pixel(0, 0, 1f32, T{a: 0});  
    q.add_tile_pixel(0, 0, 2f32, T{a: 1});
    q.add_tile_pixel(0, 0, 2f32, T{a: 2});
    q.add_tile_pixel(2, 0, 2f32, T{a: 3});
    assert!(q.root.tile_mem == 3);
    assert!(q.get_all_tiles().len() == 3);
    q.add_tile_pixel(0, 2, 2f32, T{a: 4});
    q.add_tile_pixel(2, 2, 2f32, T{a: 5});
    assert!(q.root.tile_mem == 4);

    let (request, _) = q.get_tile_rects_pixel(Rect(Point2D(0, 0), Size2D(2, 2)), 2f32);
    assert!(request.is_empty());
    let (request, _) = q.get_tile_rects_pixel(Rect(Point2D(0, 0), Size2D(2, 2)), 1.9);
    assert!(request.is_empty());
    let (request, _) = q.get_tile_rects_pixel(Rect(Point2D(0, 0), Size2D(2, 2)), 1f32);
    assert!(request.len() == 4);

    q.add_tile_pixel(0, 0, 0.5, T{a: 6});
    q.add_tile_pixel(0, 0, 1f32, T{a: 7});
    let (_, unused) = q.get_tile_rects_pixel(Rect(Point2D(0, 0), Size2D(2, 2)), 0.5);
    assert!(!unused.is_empty());
    assert!(q.root.tile_mem == 1);
}
