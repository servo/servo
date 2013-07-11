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
use gfx::render_task::BufferRequest;

/// Parent to all quadtree nodes. Stores variables needed at all levels. All method calls
/// at this level are in pixel coordinates.
pub struct Quadtree<T> {
    root: QuadtreeNode<T>,
    max_tile_size: uint,
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
}

priv enum Quadrant {
    TL = 0,
    TR = 1,
    BL = 2,
    BR = 3,
}

impl<T> Quadtree<T> {
    /// Public method to create a new Quadtree
    pub fn new(x: uint, y: uint, width: uint, height: uint, tile_size: uint) -> Quadtree<T> {        
        // Spaces must be squares and powers of 2, so expand the space until it is
        let longer = width.max(&height);
        let num_tiles = div_ceil(longer, tile_size);
        let power_of_two = next_power_of_two(num_tiles);
        let size = power_of_two * tile_size;
        
        Quadtree {
            root: QuadtreeNode {
                tile: None,
                origin: Point2D(x as f32, y as f32),
                size: size as f32,
                quadrants: [None, None, None, None],
                render_flag: false,
            },
            max_tile_size: tile_size,
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
    pub fn add_tile(&mut self, x: uint, y: uint, scale: f32, tile: T) {
        self.root.add_tile(x as f32 / scale, y as f32 / scale, tile, self.max_tile_size as f32 / scale);
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
    pub fn remove_tile(&mut self, x: uint, y: uint, scale: f32) {
        self.root.remove_tile(x as f32 / scale, y as f32 / scale);
    }
    /// Given a window rect in page coordinates and a function to check if an existing tile is "valid"
    /// (i.e. is the correct resolution), this function returns a list of BufferRequests for tiles that
    /// need to be rendered. It also returns a boolean if the window needs to be redisplayed, i.e. if
    /// no tiles need to be rendered, but the display tree needs to be rebuilt. This can occur when the
    /// user zooms out and cached tiles need to be displayed on top of higher resolution tiles.
    pub fn get_tile_rects(&mut self, window: Rect<int>, valid: &fn(&T) -> bool, scale: f32) ->
        (~[BufferRequest], bool) {
        
        self.root.get_tile_rects(Rect(Point2D(window.origin.x as f32 / scale, window.origin.y as f32 / scale),
                                      Size2D(window.size.width as f32 / scale, window.size.height as f32 / scale)),
                                 valid, scale, self.max_tile_size as f32 / scale)
    }

    /// Generate html to visualize the tree. For debugging purposes only.
    pub fn get_html(&self) -> ~str {
        static HEADER: &'static str = "<!DOCTYPE html><html>";
        fmt!("%s<body>%s</body></html>", HEADER, self.root.get_html())
    }

}

impl<T> QuadtreeNode<T> {
    /// Private method to create new children
    fn new_child(x: f32, y: f32, size: f32) -> QuadtreeNode<T> {
        QuadtreeNode {
            tile: None,
            origin: Point2D(x, y),
            size: size,
            quadrants: [None, None, None, None],
            render_flag: false,
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

    /// Get all tiles in the tree, parents first.
    fn get_all_tiles<'r>(&'r self) -> ~[&'r T] {
        let mut ret = ~[];
        
        match self.tile {
            Some(ref tile) => ret = ~[tile],
            None => {}
        }

        for self.quadrants.iter().advance |quad| {
            match *quad {
                Some(ref child) => ret = ret + child.get_all_tiles(),
                None => {}
            }
        }

        return ret;
    }

    /// Add a tile associated with a given position in page coords. If the tile size exceeds the maximum,
    /// the node will be split and the method will recurse until the tile size is within limits.
    fn add_tile(&mut self, x: f32, y: f32, tile: T, tile_size: f32) {
        debug!("Quadtree: Adding: (%?, %?) size:%?px", self.origin.x, self.origin.y, self.size);

        if x >= self.origin.x + self.size || x < self.origin.x
            || y >= self.origin.y + self.size || y < self.origin.y {
            fail!("Quadtree: Tried to add tile to invalid region");
        }
        
        if self.size <= tile_size { // We are the child
            self.tile = Some(tile);
            // FIXME: This should be inline, but currently won't compile
            let quads = [TL, TR, BL, BR];
            for quads.iter().advance |quad| {
                self.quadrants[*quad as int] = None;
            }
            self.render_flag = false;
        } else { // Send tile to children            
            let quad = self.get_quadrant(x, y);
            match self.quadrants[quad as int] {
                Some(ref mut child) => child.add_tile(x, y, tile, tile_size),
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
                    c.add_tile(x, y, tile, tile_size);
                    self.quadrants[quad as int] = Some(c);

                    // If my tile is completely occluded, get rid of it.
                    // FIXME: figure out a better way to determine if a tile is completely occluded
                    // e.g. this alg doesn't work if a tile is covered by its grandchildren
                    match self.quadrants {
                        [Some(ref tl_child), Some(ref tr_child), Some(ref bl_child), Some(ref br_child)] => {
                            match (&tl_child.tile, &tr_child.tile, &bl_child.tile, &br_child.tile) {
                                (&Some(_), &Some(_), &Some(_), &Some(_)) => self.tile = None,
                                _ => {}
                            }
                        }
                        _ => {}
                    }
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
                c.render_flag = true;
                let result = c.get_tile_rect(x, y, scale, tile_size);
                self.quadrants[quad as int] = Some(c);
                result
            }
            Some(ref mut child) => child.get_tile_rect(x, y, scale, tile_size),
        }
    }

    /// Removes a tile that is far from the given input point in page coords. Returns true if the child
    /// has no tiles and needs to be deleted.
    fn remove_tile(&mut self, x: f32, y: f32) -> bool {
        match (&self.tile, &self.quadrants) {
            (&Some(_), &[None, None, None, None]) => {
                self.tile = None;
                return true;
            }
            (&Some(_), _) => {
                self.tile = None;
                return false;
            }
            _ => {}
        }
        
        // This is a hacky heuristic to find a tile that is "far away". There are better methods.
        let quad = self.get_quadrant(x, y);
        let my_child = match quad {
            TL => {
                match (&self.quadrants[BR as int], &self.quadrants[BL as int], &self.quadrants[TR as int]) {
                    (&Some(_), _, _) => BR,
                    (&None, &Some(_), _) => BL,
                    (&None, &None, &Some(_)) => TR,
                    _ => TL,
                }
            }
            TR => {
                match (&self.quadrants[BL as int], &self.quadrants[BR as int], &self.quadrants[TL as int]) {
                    (&Some(_), _, _) => BL,
                    (&None, &Some(_), _) => BR,
                    (&None, &None, &Some(_)) => TL,
                    _ => TR,
                }
            }
            BL => {
                match (&self.quadrants[TR as int], &self.quadrants[TL as int], &self.quadrants[BR as int]) {
                    (&Some(_), _, _) => TR,
                    (&None, &Some(_), _) => TL,
                    (&None, &None, &Some(_)) => BR,
                    _ => BL,
                }
            }
            BR => {
                match (&self.quadrants[TL as int], &self.quadrants[TR as int], &self.quadrants[BL as int]) {
                    (&Some(_), _, _) => TL,
                    (&None, &Some(_), _) => TR,
                    (&None, &None, &Some(_)) => BL,
                    _ => BR,
                }
            }
        };

        match self.quadrants[my_child as int] {
            Some(ref mut child) if !child.remove_tile(x, y) => {
                return false;
            }
            Some(_) => {} // fall through
            None => fail!("Quadtree: child query failure"),
        }

        // child.remove_tile() returned true
        self.quadrants[my_child as int] = None;
        match self.quadrants {
            [None, None, None, None] => true,
            _ => false,
        }
    }
    
    /// Given a window rect in page coordinates and a tile validation function, returns a BufferRequest array
    /// and a redisplay boolean. See QuadTree function description for more details.
    fn get_tile_rects(&mut self, window: Rect<f32>, valid: &fn(&T) -> bool, scale: f32, tile_size: f32) ->
        (~[BufferRequest], bool) {
        
        let w_x = window.origin.x;
        let w_y = window.origin.y;
        let w_width = window.size.width;
        let w_height = window.size.height;
        let s_x = self.origin.x;
        let s_y = self.origin.y;
        let s_size = self.size;
        
        if w_x < s_x || w_x + w_width > s_x + s_size
            || w_y < s_y || w_y + w_height > s_y + s_size {
            println(fmt!("window: %?, %?, %?, %?; self: %?, %?, %?", w_x, w_y, w_width, w_height, s_x, s_y, s_size));
            fail!("Quadtree: tried to query an invalid tile rect");
        }
        
        if s_size <= tile_size { // We are the child
            match self.tile {
                Some(ref tile) if valid(tile) => {
                    let redisplay = match self.quadrants {
                        [None, None, None, None] => false,
                        _ => true,
                    };
                    if redisplay {
                        // FIXME: This should be inline, but currently won't compile
                        let quads = [TL, TR, BL, BR];
                        for quads.iter().advance |quad| {
                            self.quadrants[*quad as int] = None;
                        }
                    }
                    return (~[], redisplay);
                }
                None if self.render_flag => {
                    return(~[], false);
                }
                _ => {
                    return (~[self.get_tile_rect(s_x, s_y, scale, tile_size)], false);
                }
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
        
        for quads_to_check.iter().advance |quad| {
            match self.quadrants[*quad as int] {
                Some(ref mut child) => {
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
                    let (c_ret, c_redisplay) = child.get_tile_rects(new_window, |x| valid(x), scale, tile_size); 
                    ret = ret + c_ret;
                    redisplay = redisplay || c_redisplay;
                }
                None => {
                    // Figure out locations of future children
                    let (x_start, y_start, x_end, y_end) = match *quad {
                        TL => (w_x,
                               w_y,
                               (w_x + w_width).min(&(s_x + s_size / 2.0)),
                               (w_y + w_height).min(&(s_y + s_size / 2.0))),
                        TR => (w_x.max(&(s_x + s_size / 2.0)),
                               w_y,
                               (w_x + w_width + tile_size).min(&(s_x + s_size)),
                               (w_y + w_height).min(&(s_y + s_size / 2.0))),
                        BL => (w_x,
                               w_y.max(&(s_y + s_size / 2.0)),
                               (w_x + w_width).min(&(s_x + s_size / 2.0)),
                               (w_y + w_height + tile_size).min(&(s_y + s_size))),
                        BR => (w_x.max(&(s_x + s_size / 2.0)),
                               w_y.max(&(s_y + s_size / 2.0)),
                               (w_x + w_width + tile_size).min(&(s_x + s_size)),
                               (w_y + w_height + tile_size).min(&(s_y + s_size))),
                    };
                    let size = (((x_end - x_start) / tile_size).ceil() *
                                ((y_end - y_start) / tile_size).ceil()) as uint;

                    let builder = |push: &fn(BufferRequest)| {
                        let mut y = y_start;
                        while y < y_end {
                            let mut x = x_start;
                            while x < x_end {
                                push(self.get_tile_rect(x, y, scale, tile_size));
                                x = x + tile_size;
                            }
                            y = y + tile_size;
                        }
                    };
                    ret = ret + build_sized(size, builder);
                }
            }
        }
        
        (ret, redisplay)
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
fn test_add_tile() {
    let mut t = Quadtree::new(50, 30, 20, 20, 10);
    assert!(t.get_tile(50, 30, 1.0).is_none());
    t.add_tile(50, 30, 1.0, 1);
    assert!(t.get_tile(50, 30, 1.0).get() == 1);
    assert!(t.get_tile(59, 39, 1.0).get() == 1);
    assert!(t.get_tile(60, 40, 1.0).is_none());
    assert!(t.get_tile(110, 70, 2.0).get() == 1);
    t.add_tile(100, 60, 2.0, 2);
    assert!(t.get_tile(109, 69, 2.0).get() == 2);
    assert!(t.get_tile(110, 70, 2.0).get() == 1);
}