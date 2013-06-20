/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

// Implements a Quadtree data structure to keep track of which tiles have
// been rasterized and which have not.

use geom::point::Point2D;
use geom::size::Size2D;
use geom::rect::Rect;

priv enum Quadtype {
    Empty,
    Base,
    Branch,
}

priv enum Quadrant {
    TL = 0,
    TR = 1,
    BL = 2,
    BR = 3,
}


pub struct Quadtree {
    quadtype: Quadtype,
    rect: Rect<uint>,
    quadrants: [Option<~Quadtree>, ..4],
}


impl Quadtree {
    pub fn new(x: uint, y: uint, width: uint, height: uint) -> Quadtree {
        Quadtree {
            quadtype: Empty,
            rect: Rect {
                origin: Point2D(x, y),
                size: Size2D(width, height),
            },

            quadrants: [None, None, None, None],
        }
    }
    
    /// Determine which child contains a given point
    priv fn get_quadrant(&self, x: uint, y: uint) -> Quadrant {
        let self_width = self.rect.size.width;
        let self_height = self.rect.size.height;
        let self_x = self.rect.origin.x;
        let self_y = self.rect.origin.y;
        match (self_width, self_height) {
            (1, _) => {
                if y < self_y + self_height / 2 { 
                    TL
                } else { 
                    BR
                }
            }
            (_, 1) => {
                if x < self_x + self_width / 2 {
                    TL
                } else {
                    BR
                }
            }
            _ => {
                if x < self_x + self_width / 2 {
                    if y < self_y + self_height / 2 { 
                        TL
                    } else { 
                        BL
                    }
                } else if y < self_y + self_height / 2 { 
                    TR
                } else { 
                    BR
                }
            }
        }
    }
    
    /// Change a point from Empty to Base
    pub fn add_region(&mut self, x: uint, y: uint) {
        let self_x = self.rect.origin.x;
        let self_y = self.rect.origin.y;
        let self_width = self.rect.size.width;
        let self_height = self.rect.size.height;

        debug!("Quadtree: adding: (%?, %?) w:%?, h:%?", self_x, self_y, self_width, self_height);

        if x >= self_x + self_width || x < self_x
            || y >= self_y + self_height || y < self_y {
            return; // Out of bounds
        }
        match self.quadtype {
            Base => return,
            Empty => {
                if self_width == 1 && self_height == 1 {
                    self.quadtype = Base;
                    return;
                }
                self.quadtype = Branch;

                // Initialize children
                self.quadrants[TL as int] = Some(~Quadtree::new(self_x,
                                                                self_y,
                                                                (self_width / 2).max(&1),
                                                                (self_height / 2).max(&1)));
                if self_width > 1 && self_height > 1 {
                    self.quadrants[TR as int] = Some(~Quadtree::new(self_x + self_width / 2,
                                                                    self_y,
                                                                    self_width - self_width / 2,
                                                                    self_height / 2));
                    self.quadrants[BL as int] = Some(~Quadtree::new(self_x,
                                                                    self_y + self_height / 2,
                                                                    self_width / 2,
                                                                    self_height - self_height / 2));
                }
                self.quadrants[BR as int] = Some(~Quadtree::new(self_x + self_width / 2,
                                                                self_y + self_height / 2,
                                                                self_width - self_width / 2,
                                                                self_height - self_height / 2));
            }
            Branch => {} // Fall through
        }

        // If we've made it this far, we know we are a branch and therefore have children
        let index = self.get_quadrant(x, y) as int;
        
        match self.quadrants[index] {
            None => fail!("Quadtree: child query failure"),
            Some(ref mut region) => {
                // Recurse if necessary
                match region.quadtype {
                    Empty | Branch => {
                        region.add_region(x, y);
                    }
                    Base => {} // nothing to do
                }
            }
        }
        
        // FIXME: ideally we could make the assignments in the match,
        // but borrowed pointers prevent that. So here's a flag instead.
        let mut base_flag = 0;
        
        // If all children are Bases, convert self to Base
        match (&self.quadrants, self_width, self_height) {
            (&[Some(ref tl_q), _, _, Some(ref br_q)], 1, _) |
            (&[Some(ref tl_q), _, _, Some(ref br_q)], _, 1) => {
                match(tl_q.quadtype, br_q.quadtype) {
                    (Base, Base) => {
                        base_flag = 1;
                    }
                    _ => {} // nothing to do
                }
            }
            (&[Some(ref tl_q), Some(ref tr_q), Some(ref bl_q), Some(ref br_q)], _, _) => {
                    match (tl_q.quadtype, tr_q.quadtype, bl_q.quadtype, br_q.quadtype) {
                        (Base, Base, Base, Base) => {
                            base_flag = 2;
                        }
                        _ => {} // nothing to do
                    }
            }
            _ => {} // nothing to do
        }
        
        match base_flag {
            0 => {}
            1 => {
                self.quadtype = Base;
                self.quadrants[TL as int] = None;
                self.quadrants[BR as int] = None;
            }
            2 => {
                self.quadtype = Base;
                self.quadrants[TL as int] = None;
                self.quadrants[TR as int] = None;
                self.quadrants[BL as int] = None;
                self.quadrants[BR as int] = None;
            }
            _ => fail!("Quadtree: Unknown flag type"),
        }
    }
    
    /// Check if a point is a Base or Empty.
    pub fn check_region(&self, x: uint, y: uint) -> bool {
        let self_x = self.rect.origin.x;
        let self_y = self.rect.origin.y;
        let self_width = self.rect.size.width;
        let self_height = self.rect.size.height;

        if x >= self_x + self_width || x < self_x
            || y >= self_y + self_height || y < self_y {
            return false; // out of bounds
        }

        match self.quadtype {
            Empty => false,
            Base => true,
            Branch => {
                let index = self.get_quadrant(x,y) as int;
                match self.quadrants[index] {
                    None => fail!("Quadtree: child query failed"),
                    Some(ref region) => region.check_region(x, y)
                }
            }
        }
    }
    
}


#[test]
fn test_add_region() {
    let mut t = Quadtree::new(50, 50, 3, 4);
    assert!(!t.check_region(50, 50));
    t.add_region(50, 50);
    assert!(t.check_region(50, 50));
    assert!(!t.check_region(51, 50));
    assert!(!t.check_region(50, 51));
    t.add_region(53, 50);
    assert!(!t.check_region(53, 50));

}