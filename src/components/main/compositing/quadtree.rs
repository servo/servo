/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

// Implements a Quadtree data structure to keep track of which tiles have
// been rasterized and which have not.

use geom::point::Point2D;

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
        let num_tiles = uint::div_ceil(longer, tile_size);
        let power_of_two = uint::next_power_of_two(num_tiles);
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
    
    /// Get a tile at a given pixel position and scale.
    pub fn get_tile<'r>(&'r self, x: uint, y: uint, scale: f32) -> &'r Option<T> {
        self.root.get_tile(x as f32 / scale, y as f32 / scale)
    }
    /// Add a tile associtated with a given pixel position and scale.
    pub fn add_tile(&mut self, x: uint, y: uint, scale: f32, tile: T) {
        self.root.add_tile(x as f32 / scale, y as f32 / scale, tile, self.max_tile_size as f32 / scale);
    }


}

impl<T> QuadtreeNode<T> {
    // Private method to create new children
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
            for vec::each([TL, TR, BL, BR]) |quad| {
                self.quadrants[*quad as int] = None;
            }
        } else { //send tile to children            
            let quad = self.get_quadrant(x, y);
            match self.quadrants[quad as int] {
                Some(ref mut child) => child.add_tile(x, y, tile, tile_size),
                None => { //make new child      
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

                    // If we have 4 children, we probably shouldn't be hanging onto a tile
                    // Though this isn't always true if we have grandchildren
                    match self.quadrants {
                        [Some(_), Some(_), Some(_), Some(_)] => {
                            self.tile = None;
                        }
                        _ => {}
                    }

                }
            }
        }
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