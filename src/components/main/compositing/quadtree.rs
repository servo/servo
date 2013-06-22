/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

// Implements a Quadtree data structure to keep track of which tiles have
// been rasterized and which have not.

use geom::point::Point2D;

pub struct Quadtree<T> {
    tile: Option<T>,
    origin: Point2D<f32>,
    size: f32,
    quadrants: [Option<~Quadtree<T>>, ..4],
    scale: @mut f32,
    max_tile_size: uint,
}

priv enum Quadrant {
    TL = 0,
    TR = 1,
    BL = 2,
    BR = 3,
}

impl<T> Quadtree<T> {
    // Public method to create a new Quadtree
    pub fn new(x: uint, y: uint, width: uint, height: uint, tile_size: uint, scale: @mut f32) -> Quadtree<T> {
        if(*scale != 1.0) { 
            println("Warning: Quadtree: Quadtree initialized while zoomed; this action is unsupported.");
            println("Please set zoom to 1.0 before creating the Quadtree.");
        }
        
        // Spaces must be squares and powers of 2, so expand the space until it is
        let longer = width.max(&height);
        let num_tiles = uint::div_ceil(longer, tile_size);
        let power_of_two = uint::next_power_of_two(num_tiles);
        let size = power_of_two * tile_size;
        
        Quadtree {
            tile: None,
            origin: Point2D(x as f32, y as f32),
            size: size as f32,
            quadrants: [None, None, None, None],
            scale: scale,
            max_tile_size: tile_size,
        }
    }

    // Private method to create new children
    fn new_child(&self, x: f32, y: f32, size: f32) -> Quadtree<T> {
        Quadtree {
            tile: None,
            origin: Point2D(x, y),
            size: size,
            quadrants: [None, None, None, None],
            scale: self.scale,
            max_tile_size: self.max_tile_size,
        }
    }
    
    /// Determine which child contains a given point
    fn get_quadrant(&self, x: uint, y: uint) -> Quadrant {
        let self_x = (self.origin.x * *(self.scale)).ceil() as uint;
        let self_y = (self.origin.y * *(self.scale)).ceil() as uint;
        let self_size = (self.size * *(self.scale)).ceil() as uint;

        if x < self_x + self_size / 2 {
            if y < self_y + self_size / 2 { 
                TL
            } else { 
                BL
            }
        } else if y < self_y + self_size / 2 { 
            TR
        } else { 
            BR
        }
    }

    /// Get the lowest-level (highest resolution) tile associated with a certain pixel
    pub fn get_tile<'r> (&'r self, x: uint, y: uint) -> &'r Option<T> {
        let self_x = (self.origin.x * *(self.scale)).ceil() as uint;
        let self_y = (self.origin.y * *(self.scale)).ceil() as uint;
        let self_size = (self.size * *(self.scale)).ceil() as uint;

        if x >= self_x + self_size || x < self_x
            || y >= self_y + self_size || y < self_y {
            fail!("Quadtree: Tried to get a tile outside of range");
        }

        let index = self.get_quadrant(x,y) as int;
        match self.quadrants[index] {
            None => &'r self.tile,
            Some(ref child) => child.get_tile(x, y),
        }
    }    

    
    /// Add a tile
    pub fn add_tile(&mut self, x: uint, y: uint, tile: T) {
        let self_x = (self.origin.x * *(self.scale)).ceil() as uint;
        let self_y = (self.origin.y * *(self.scale)).ceil() as uint;
        let self_size = (self.size * *(self.scale)).ceil() as uint;

        debug!("Quadtree: Adding: (%?, %?) size:%?px", self_x, self_y, self_size);

        if x >= self_x + self_size || x < self_x
            || y >= self_y + self_size || y < self_y {
            fail!("Quadtree: Tried to add tile to invalid region");
        }
        
        if self_size <= self.max_tile_size { // We are the child            
            self.tile = Some(tile);
            for vec::each([TL, TR, BL, BR]) |quad| {
                self.quadrants[*quad as int] = None;
            }
        } else { //send tile to children            
            let quad = self.get_quadrant(x, y);
            match self.quadrants[quad as int] {
                Some(ref mut child) => child.add_tile(x, y, tile),
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
                    let mut c = ~self.new_child(new_x, new_y, new_size);
                    c.add_tile(x, y, tile);
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
    let scale = @mut 1.0;
    let mut t = Quadtree::new(50, 30, 20, 20, 10, scale);
    assert!(t.get_tile(50, 30).is_none());
    t.add_tile(50, 30, 1);
    assert!(t.get_tile(50, 30).get() == 1);
    assert!(t.get_tile(59, 39).get() == 1);
    assert!(t.get_tile(60, 40).is_none());
    *scale = 2.0;
    assert!(t.get_tile(110, 70).get() == 1);
    t.add_tile(100, 60, 2);
    assert!(t.get_tile(109, 69).get() == 2);
    assert!(t.get_tile(110, 70).get() == 1);
}