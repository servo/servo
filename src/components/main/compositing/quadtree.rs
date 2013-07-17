/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

// Implements a Quadtree data structure to keep track of which tiles have
// been rasterized and which have not.

use geom::point::Point2D;
use geom::size::Size2D;
use geom::rect::Rect;
use std::uint::{div_ceil, next_power_of_two};
use std::vec::build_sized;
use std::util::replace;
use gfx::render_task::BufferRequest;
use servo_msg::compositor_msg::Tile;

/// Parent to all quadtree nodes. Stores variables needed at all levels. All method calls
/// at this level are in pixel coordinates.
pub struct Quadtree<T> {
    root: QuadtreeNode<T>,
    max_tile_size: uint,
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
    /// If this node is marked for rendering
    render_flag: bool,
    /// Combined size of self.tile and tiles of all descendants
    tile_mem: uint,
}

priv enum Quadrant {
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
    pub fn new(width: uint, height: uint, tile_size: uint, max_mem: Option<uint>) -> Quadtree<T> {
        // Spaces must be squares and powers of 2, so expand the space until it is
        let longer = width.max(&height);
        let num_tiles = div_ceil(longer, tile_size);
        let power_of_two = next_power_of_two(num_tiles);
        let size = power_of_two * tile_size;
        
        Quadtree {
            root: QuadtreeNode {
                tile: None,
                origin: Point2D(0f32, 0f32),
                size: size as f32,
                quadrants: [None, None, None, None],
                render_flag: false,
                tile_mem: 0,
            },
            max_tile_size: tile_size,
            max_mem: max_mem,
        }
    }

    /// Return the maximum allowed tile size
    pub fn get_tile_size(&self) -> uint {
        self.max_tile_size
    }
    /// Get a tile at a given pixel position and scale.
    pub fn get_tile<'r>(&'r self, x: uint, y: uint, scale: f32) -> &'r Option<T> {
        self.root.get_tile(x as f32 / scale, y as f32 / scale)
    }
    /// Add a tile associtated with a given pixel position and scale.
    /// If the tile pushes the total memory over its maximum, tiles will be removed
    /// until total memory is below the maximum again.
    pub fn add_tile(&mut self, x: uint, y: uint, scale: f32, tile: T) {
        self.root.add_tile(x as f32 / scale, y as f32 / scale, tile, self.max_tile_size as f32 / scale);
        match self.max_mem {
            Some(max) => {
                while self.root.tile_mem > max {
                    let r = self.root.remove_tile(x as f32 / scale, y as f32 / scale);
                    match r {
                        (Some(_), _, _) => {}
                        _ => fail!("Quadtree: No valid tiles to remove"),
                    }
                }
            }
            None => {}
        }
    }
    /// Get the tile rect in screen and page coordinates for a given pixel position
    pub fn get_tile_rect(&mut self, x: uint, y: uint, scale: f32) -> BufferRequest {
        self.root.get_tile_rect(x as f32 / scale, y as f32 / scale, scale, self.max_tile_size as f32 / scale)
    }
    /// Get all the tiles in the tree
    pub fn get_all_tiles<'r>(&'r self) -> ~[&'r T] {
        self.root.get_all_tiles()
    }
    /// Ask a tile to be deleted from the quadtree. This tries to delete a tile that is far from the
    /// given point in pixel coordinates.
    pub fn remove_tile(&mut self, x: uint, y: uint, scale: f32) -> T {
        let r = self.root.remove_tile(x as f32 / scale, y as f32 / scale);
        match r {
            (Some(tile), _, _) => tile,
            _ => fail!("Quadtree: No valid tiles to remove"),
        }
    }
    /// Given a window rect in pixel coordinates, this function returns a list of BufferRequests for tiles that
    /// need to be rendered. It also returns a boolean if the window needs to be redisplayed, i.e. if
    /// no tiles need to be rendered, but the display tree needs to be rebuilt. This can occur when the
    /// user zooms out and cached tiles need to be displayed on top of higher resolution tiles.
    /// When this happens, higher resolution tiles will be removed from the quadtree.
    pub fn get_tile_rects(&mut self, window: Rect<int>, scale: f32) -> (~[BufferRequest], bool) {
        let (ret, redisplay, _) = self.root.get_tile_rects(
            Rect(Point2D(window.origin.x as f32 / scale, window.origin.y as f32 / scale),
                 Size2D(window.size.width as f32 / scale, window.size.height as f32 / scale)),
            scale, self.max_tile_size as f32 / scale);
        (ret, redisplay)
    }

    /// Generate html to visualize the tree. For debugging purposes only.
    pub fn get_html(&self) -> ~str {
        static HEADER: &'static str = "<!DOCTYPE html><html>";
        fmt!("%s<body>%s</body></html>", HEADER, self.root.get_html())
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
            render_flag: false,
            tile_mem: 0,
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

    /// Get the lowest-level (highest resolution) tile associated with a given position in page coords.
    fn get_tile<'r> (&'r self, x: f32, y: f32) -> &'r Option<T> {
        if x >= self.origin.x + self.size || x < self.origin.x
            || y >= self.origin.y + self.size || y < self.origin.y {
            fail!("Quadtree: Tried to get a tile outside of range");
        }

        let index = self.get_quadrant(x, y) as int;
        match self.quadrants[index] {
            None => &'r self.tile,
            Some(ref child) => child.get_tile(x, y),
        }
    }

    /// Get all tiles in the tree, parents last.
    fn get_all_tiles<'r>(&'r self) -> ~[&'r T] {
        let mut ret = ~[];

        for self.quadrants.iter().advance |quad| {
            match *quad {
                Some(ref child) => ret = ret + child.get_all_tiles(),
                None => {}
            }
        }
        
        match self.tile {
            Some(ref tile) => ret = ret + ~[tile],
            None => {}
        }


        return ret;
    }

    /// Add a tile associated with a given position in page coords. If the tile size exceeds the maximum,
    /// the node will be split and the method will recurse until the tile size is within limits.
    /// Returns an the difference in tile memory between the new quadtree node and the old quadtree node.
    fn add_tile(&mut self, x: f32, y: f32, tile: T, tile_size: f32) -> int {
        debug!("Quadtree: Adding: (%?, %?) size:%?px", self.origin.x, self.origin.y, self.size);

        if x >= self.origin.x + self.size || x < self.origin.x
            || y >= self.origin.y + self.size || y < self.origin.y {
            fail!("Quadtree: Tried to add tile to invalid region");
        }
        
        if self.size <= tile_size { // We are the child
            let old_size = self.tile_mem;
            self.tile_mem = tile.get_mem();
            self.tile = Some(tile);
            // FIXME: This should be inline, but currently won't compile
            let quads = [TL, TR, BL, BR];
            for quads.iter().advance |quad| {
                self.quadrants[*quad as int] = None;
            }
            self.render_flag = false;
            self.tile_mem as int - old_size as int
        } else { // Send tile to children            
            let quad = self.get_quadrant(x, y);
            match self.quadrants[quad as int] {
                Some(ref mut child) => {
                    let delta = child.add_tile(x, y, tile, tile_size);
                    self.tile_mem = (self.tile_mem as int + delta) as uint;
                    delta
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
                    let delta = c.add_tile(x, y, tile, tile_size);
                    self.tile_mem = (self.tile_mem as int + delta) as uint;    
                    self.quadrants[quad as int] = Some(c);
                    delta
                }
            }
        }
    }

    /// Get a tile rect in screen and page coords for a given position in page coords
    fn get_tile_rect(&mut self, x: f32, y: f32, scale: f32, tile_size: f32) -> BufferRequest {
        if x >= self.origin.x + self.size || x < self.origin.x
            || y >= self.origin.y + self.size || y < self.origin.y {
            fail!("Quadtree: Tried to query a tile rect outside of range");
        }
        
        if self.size <= tile_size {
            let self_x = (self.origin.x * scale).ceil() as uint;
            let self_y = (self.origin.y * scale).ceil() as uint;
            let self_size = (self.size * scale).ceil() as uint;
            self.render_flag = true;
            return BufferRequest(Rect(Point2D(self_x, self_y), Size2D(self_size, self_size)),
                                 Rect(Point2D(self.origin.x, self.origin.y), Size2D(self.size, self.size)));
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
                let result = c.get_tile_rect(x, y, scale, tile_size);
                self.quadrants[quad as int] = Some(c);
                result
            }
            Some(ref mut child) => child.get_tile_rect(x, y, scale, tile_size),
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
        
        for queue.iter().advance |quad| {
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
    /// a redisplay boolean, and the difference in tile memory between the new and old quadtree nodes.
    /// NOTE: this method will sometimes modify the tree by deleting tiles.
    /// See the QuadTree function description for more details.
    fn get_tile_rects(&mut self, window: Rect<f32>, scale: f32, tile_size: f32) ->
        (~[BufferRequest], bool, int) {
        
        let w_x = window.origin.x;
        let w_y = window.origin.y;
        let w_width = window.size.width;
        let w_height = window.size.height;
        let s_x = self.origin.x;
        let s_y = self.origin.y;
        let s_size = self.size;
        
        if w_x < s_x || w_x + w_width > s_x + s_size
            || w_y < s_y || w_y + w_height > s_y + s_size {
            println(fmt!("window: %?, %?, %?, %?; self: %?, %?, %?",
                         w_x, w_y, w_width, w_height, s_x, s_y, s_size));
            fail!("Quadtree: tried to query an invalid tile rect");
        }
        
        if s_size <= tile_size { // We are the child
            return match self.tile {
                _ if self.render_flag => (~[], false, 0),
                Some(ref tile) if tile.is_valid(scale) => {
                    let redisplay = match self.quadrants {
                        [None, None, None, None] => false,
                        _ => true,
                    };
                    let mut delta = 0;
                    if redisplay {
                        let old_mem = self.tile_mem;
                        // FIXME: This should be inline, but currently won't compile
                        let quads = [TL, TR, BL, BR];
                        for quads.iter().advance |quad| {
                            self.quadrants[*quad as int] = None;
                        }
                        self.tile_mem = tile.get_mem();
                        delta = self.tile_mem as int - old_mem as int;

                    }
                    (~[], redisplay, delta)
                }
                _ => (~[self.get_tile_rect(s_x, s_y, scale, tile_size)], false, 0),
            }
        }
        
        // Otherwise, we either have children or will have children
        let w_tl_quad = self.get_quadrant(w_x, w_y);
        let w_br_quad = self.get_quadrant(w_x + w_width, w_y + w_height);
        
        // Figure out which quadrants the window is in
        let builder = |push: &fn(Quadrant)| {
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
        
        let quads_to_check = build_sized(4, builder);
        
        let mut ret = ~[];
        let mut redisplay = false;
        let mut delta = 0;
        
        for quads_to_check.iter().advance |quad| {
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

            let (c_ret, c_redisplay, c_delta) = match self.quadrants[*quad as int] {
                Some(ref mut child) => child.get_tile_rects(new_window, scale, tile_size),
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
                    let (a, b, c) = child.get_tile_rects(new_window, scale, tile_size);
                    self.quadrants[*quad as int] = Some(child);
                    (a, b, c)
                }
            };
            
            delta = delta + c_delta;
            ret = ret + c_ret;
            redisplay = redisplay || c_redisplay;
        } 
        self.tile_mem = (self.tile_mem as int + delta) as uint;
        (ret, redisplay, delta)
    }


    /// Generate html to visualize the tree.
    /// This is really inefficient, but it's for testing only.
    fn get_html(&self) -> ~str {
        let mut ret = ~"";
        match self.tile {
            Some(ref tile) => {
                ret = fmt!("%s%?", ret, tile);
            }
            None => {
                ret = fmt!("%sNO TILE", ret);
            }
        }
        match self.quadrants {
            [None, None, None, None] => {}
            _ => {
                ret = fmt!("%s<table border=1><tr>", ret);
                // FIXME: This should be inline, but currently won't compile
                let quads = [TL, TR, BL, BR];
                for quads.iter().advance |quad| {
                    match self.quadrants[*quad as int] {
                        Some(ref child) => {
                            ret = fmt!("%s<td>%s</td>", ret, child.get_html());
                        }
                        None => {
                            ret = fmt!("%s<td>EMPTY CHILD</td>", ret);
                        }
                    }
                    match *quad {
                        TR => ret = fmt!("%s</tr><tr>", ret),
                        _ => {}
                    }
                }
                ret = fmt!("%s</table>\n", ret);
            }
        }
        return ret;
    }

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
    }
    
    let mut q = Quadtree::new(8, 8, 2, Some(4));
    q.add_tile(0, 0, 1f32, T{a: 0});  
    q.add_tile(0, 0, 2f32, T{a: 1});
    q.add_tile(0, 0, 2f32, T{a: 2});
    q.add_tile(2, 0, 2f32, T{a: 3});
    assert!(q.root.tile_mem == 3);
    assert!(q.get_all_tiles().len() == 3);
    q.add_tile(0, 2, 2f32, T{a: 4});
    q.add_tile(2, 2, 2f32, T{a: 5});
    assert!(q.root.tile_mem == 4);

    let (request, _) = q.get_tile_rects(Rect(Point2D(0, 0), Size2D(2, 2)), 2f32);
    assert!(request.is_empty());
    let (request, _) = q.get_tile_rects(Rect(Point2D(0, 0), Size2D(2, 2)), 1.9);
    assert!(request.is_empty());
    let (request, _) = q.get_tile_rects(Rect(Point2D(0, 0), Size2D(2, 2)), 1f32);
    assert!(request.len() == 4);

    q.add_tile(0, 0, 0.5, T{a: 6});
    q.add_tile(0, 0, 1f32, T{a: 7});
    let (_, redisplay) = q.get_tile_rects(Rect(Point2D(0, 0), Size2D(2, 2)), 0.5);
    assert!(redisplay);
    assert!(q.root.tile_mem == 1);
}
