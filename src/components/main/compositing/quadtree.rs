/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

// Implements a Quadtree data structure to keep track of which tiles have
// been rasterized and which have not.

use geom::point::Point2D;
use geom::size::Size2D;
use geom::rect::Rect;
use std::uint::{div_ceil, next_power_of_two};

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
    /// The positiong of the node in page coordinates.
    origin: Point2D<f32>,
    /// The width and hight of the node in page coordinates.
    size: f32,
    /// The node's children.
    quadrants: [Option<~QuadtreeNode<T>>, ..4],
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
    pub fn get_tile_rect(&self, x: uint, y: uint, scale: f32) -> (Rect<uint>, Rect<f32>) {
        self.root.get_tile_rect(x as f32 / scale, y as f32 / scale, scale, self.max_tile_size as f32 / scale)
    }
    /// Get all the tiles in the tree
    pub fn get_all_tiles<'r>(&'r self) -> ~[&'r T] {
        self.root.get_all_tiles()
    }
    /// Generate html to visualize the tree
    pub fn get_html(&self) -> ~str {
        let header = "<!DOCTYPE html PUBLIC \"-//W3C//DTD XHTML 1.0 Transitional//EN\" \"http://www.w3.org/TR/xhtml1/DTD/xhtml1-transitional.dtd\"> <html xmlns=\"http://www.w3.org/1999/xhtml\">";
        fmt!("%s<body>%s</body></html>", header, self.root.get_html())
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
    /// FIXME: this could probably be more efficient
    fn get_all_tiles<'r>(&'r self) -> ~[&'r T] {
        let mut ret = ~[];
        
        match self.tile {
            Some (ref tile) => ret = ~[tile],
            None => {}
        }

        for self.quadrants.each |quad| {
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
            for [TL, TR, BL, BR].each |quad| {
                self.quadrants[*quad as int] = None;
            }
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
    fn get_tile_rect(&self, x: f32, y: f32, scale: f32, tile_size: f32) -> (Rect<uint>, Rect<f32>) {    
        if x >= self.origin.x + self.size || x < self.origin.x
            || y >= self.origin.y + self.size || y < self.origin.y {
            fail!("Quadtree: Tried to query a tile rect outside of range");
        }
        
        if self.size <= tile_size {
            let self_x = (self.origin.x * scale).ceil() as uint;
            let self_y = (self.origin.y * scale).ceil() as uint;
            let self_size = (self.size * scale).ceil() as uint;
            return (Rect(Point2D(self_x, self_y), Size2D(self_size, self_size)),
                    Rect(Point2D(self.origin.x, self.origin.y), Size2D(self.size, self.size)));
        }
        
        let index = self.get_quadrant(x,y) as int;
        match self.quadrants[index] {
            None => {
                // Calculate where the new tile should go
                let factor = self.size / tile_size;
                let divisor = next_power_of_two(factor.ceil() as uint);
                let new_size_page = self.size / (divisor as f32);
                let new_size_pixel = (new_size_page * scale).ceil() as uint;

                let new_x_page = self.origin.x + new_size_page * ((x - self.origin.x) / new_size_page).floor();
                let new_y_page = self.origin.y + new_size_page * ((y - self.origin.y) / new_size_page).floor();
                let new_x_pixel = (new_x_page * scale).ceil() as uint;
                let new_y_pixel = (new_y_page * scale).ceil() as uint;
                
                (Rect(Point2D(new_x_pixel, new_y_pixel), Size2D(new_size_pixel, new_size_pixel)),
                 Rect(Point2D(new_x_page, new_y_page), Size2D(new_size_page, new_size_page)))
            }
            Some(ref child) => child.get_tile_rect(x, y, scale, tile_size),
        }
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
                for [TL, TR, BL, BR].each |quad| {
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