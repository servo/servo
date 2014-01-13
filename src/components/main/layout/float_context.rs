/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use geom::point::Point2D;
use geom::rect::Rect;
use geom::size::Size2D;
use servo_util::geometry::{Au, max, min};
use std::i32::max_value;
use std::util::replace;
use std::vec;
use style::computed_values::float;

#[deriving(Clone)]
pub enum FloatType {
    FloatLeft,
    FloatRight
}

impl FloatType {
    pub fn from_property(property: float::T) -> FloatType {
        match property {
            float::none => fail!("can't create a float type from an unfloated property"),
            float::left => FloatLeft,
            float::right => FloatRight,
        }
    }
}

pub enum ClearType {
    ClearLeft,
    ClearRight,
    ClearBoth
}

struct FloatContextBase {
    /// This is an option of a vector to avoid allocation in the fast path (no floats).
    float_data: Option<~[Option<FloatData>]>,
    floats_used: uint,
    max_y: Au,
    offset: Point2D<Au>,
}

#[deriving(Clone)]
struct FloatData {
    bounds: Rect<Au>,
    f_type: FloatType
}

/// All information necessary to place a float
pub struct PlacementInfo {
    width: Au,      // The dimensions of the float
    height: Au,
    ceiling: Au,    // The minimum top of the float, as determined by earlier elements
    max_width: Au,  // The maximum right of the float, generally determined by the contining block
    f_type: FloatType // left or right
}

/// Wrappers around float methods. To avoid allocating data we'll never use,
/// destroy the context on modification.
pub enum FloatContext {
    Invalid,
    Valid(FloatContextBase)
}

impl FloatContext {
    pub fn new(num_floats: uint) -> FloatContext {
        Valid(FloatContextBase::new(num_floats))
    }

    #[inline(always)]
    pub fn clone(&mut self) -> FloatContext {
        match *self {
            Invalid => fail!("Can't clone an invalid float context"),
            Valid(_) => replace(self, Invalid)
        }
    }

    #[inline(always)]
    fn with_mut_base<R>(&mut self, callback: |&mut FloatContextBase| -> R) -> R {
        match *self {
            Invalid => fail!("Float context no longer available"),
            Valid(ref mut base) => callback(&mut *base)
        }
    }

    #[inline(always)]
    pub fn with_base<R>(&self, callback: |&FloatContextBase| -> R) -> R {
        match *self {
            Invalid => fail!("Float context no longer available"),
            Valid(ref base) => callback(&*base)
        }
    }

    #[inline(always)]
    pub fn translate(&mut self, trans: Point2D<Au>) -> FloatContext {
        self.with_mut_base(|base| {
            base.translate(trans);
        });
        replace(self, Invalid)
    }

    #[inline(always)]
    pub fn available_rect(&mut self, top: Au, height: Au, max_x: Au) -> Option<Rect<Au>> {
        self.with_base(|base| {
            base.available_rect(top, height, max_x)
        })
    }

    #[inline(always)]
    pub fn add_float(&mut self, info: &PlacementInfo) -> FloatContext{
        self.with_mut_base(|base| {
            base.add_float(info);
        });
        replace(self, Invalid)
    }

    #[inline(always)]
    pub fn place_between_floats(&self, info: &PlacementInfo) -> Rect<Au> {
        self.with_base(|base| {
            base.place_between_floats(info)
        })
    }

    #[inline(always)]
    pub fn last_float_pos(&mut self) -> Point2D<Au> {
        self.with_base(|base| {
            base.last_float_pos()
        })
    }

    #[inline(always)]
    pub fn clearance(&self, clear: ClearType) -> Au {
        self.with_base(|base| {
            base.clearance(clear)
        })
    }
}

impl FloatContextBase {
    fn new(num_floats: uint) -> FloatContextBase {
        debug!("Creating float context of size {}", num_floats);
        FloatContextBase {
            float_data: if num_floats == 0 {
                None
            } else {
                Some(vec::from_elem(num_floats, None))
            },
            floats_used: 0,
            max_y: Au(0),
            offset: Point2D(Au(0), Au(0))
        }
    }

    fn translate(&mut self, trans: Point2D<Au>) {
        self.offset = self.offset + trans;
    }

    fn last_float_pos(&self) -> Point2D<Au> {
        assert!(self.floats_used > 0, "Error: tried to access FloatContext with no floats in it");

        match self.float_data.get_ref()[self.floats_used - 1] {
            None => fail!("FloatContext error: floats should never be None here"),
            Some(float) => {
                debug!("Returning float position: {}", float.bounds.origin + self.offset);
                float.bounds.origin + self.offset
            }
        }
    }

    /// Returns a rectangle that encloses the region from top to top + height,
    /// with width small enough that it doesn't collide with any floats. max_x
    /// is the x-coordinate beyond which floats have no effect (generally 
    /// this is the containing block width).
    fn available_rect(&self, top: Au, height: Au, max_x: Au) -> Option<Rect<Au>> {
        fn range_intersect(top_1: Au, bottom_1: Au, top_2: Au, bottom_2: Au) -> (Au, Au) {
            (max(top_1, top_2), min(bottom_1, bottom_2))
        }

        let top = top - self.offset.y;

        debug!("available_rect: trying to find space at {}", top);

        // Relevant dimensions for the right-most left float
        let mut max_left = Au(0) - self.offset.x;
        let mut l_top = None;
        let mut l_bottom = None;
        // Relevant dimensions for the left-most right float
        let mut min_right = max_x - self.offset.x;
        let mut r_top = None;
        let mut r_bottom = None;

        // Find the float collisions for the given vertical range.
        for floats in self.float_data.iter() {
            for float in floats.iter() {
                debug!("available_rect: Checking for collision against float");
                match *float {
                    None => (),
                    Some(data) => {
                        let float_pos = data.bounds.origin;
                        let float_size = data.bounds.size;
                        debug!("float_pos: {}, float_size: {}", float_pos, float_size);
                        match data.f_type {
                            FloatLeft => {
                                if(float_pos.x + float_size.width > max_left && 
                                   float_pos.y + float_size.height > top && float_pos.y < top + height) {
                                    max_left = float_pos.x + float_size.width;
                                
                                    l_top = Some(float_pos.y);
                                    l_bottom = Some(float_pos.y + float_size.height);

                                    debug!("available_rect: collision with left float: new max_left is {}",
                                            max_left);
                                }
                            }
                            FloatRight => {
                                if(float_pos.x < min_right && 
                                   float_pos.y + float_size.height > top && float_pos.y < top + height) {
                                    min_right = float_pos.x;

                                    r_top = Some(float_pos.y);
                                    r_bottom = Some(float_pos.y + float_size.height);
                                    debug!("available_rect: collision with right float: new min_right is {}",
                                            min_right);
                                }
                            }
                        }
                    }
                }
            };
        }

        // Extend the vertical range of the rectangle to the closest floats.
        // If there are floats on both sides, take the intersection of the
        // two areas. Also make sure we never return a top smaller than the
        // given upper bound.
        let (top, bottom) = match (r_top, r_bottom, l_top, l_bottom) {
            (Some(r_top), Some(r_bottom), Some(l_top), Some(l_bottom)) => 
                range_intersect(max(top, r_top), r_bottom, max(top, l_top), l_bottom),

            (None, None, Some(l_top), Some(l_bottom)) => (max(top, l_top), l_bottom),
            (Some(r_top), Some(r_bottom), None, None) => (max(top, r_top), r_bottom),
            (None, None, None, None) => return None,
            _ => fail!("Reached unreachable state when computing float area")
        };

        // This assertion is too strong and fails in some cases. It is OK to
        // return negative widths since we check against that right away, but
        // we should still undersrtand why they occur and add a stronger
        // assertion here.
        //assert!(max_left < min_right); 
        
        assert!(top <= bottom, "Float position error");

        Some(Rect{
            origin: Point2D(max_left, top) + self.offset,
            size: Size2D(min_right - max_left, bottom - top)
        })
    }

    fn add_float(&mut self, info: &PlacementInfo) {
        assert!(self.float_data.is_some());
        debug!("Floats_used: {}, Floats available: {}",
               self.floats_used,
               self.float_data.get_ref().len());
        assert!(self.floats_used < self.float_data.get_ref().len() && 
                self.float_data.get_ref()[self.floats_used].is_none());

        let new_info = PlacementInfo {
            width: info.width,
            height: info.height,
            ceiling: max(info.ceiling, self.max_y + self.offset.y),
            max_width: info.max_width,
            f_type: info.f_type
        };

        debug!("add_float: added float with info {:?}", new_info);

        let new_float = FloatData {    
            bounds: Rect {
                origin: self.place_between_floats(&new_info).origin - self.offset,
                size: Size2D(info.width, info.height)
            },
            f_type: info.f_type
        };
        self.float_data.get_mut_ref()[self.floats_used] = Some(new_float);
        self.max_y = max(self.max_y, new_float.bounds.origin.y);
        self.floats_used += 1;
    }

    /// Given the top 3 sides of the rectange, finds the largest height that
    /// will result in the rectange not colliding with any floats. Returns
    /// None if that height is infinite.
    fn max_height_for_bounds(&self, left: Au, top: Au, width: Au) -> Option<Au> {
        let top = top - self.offset.y;
        let left = left - self.offset.x;
        let mut max_height = None;

        for floats in self.float_data.iter() {
            for float in floats.iter() {
                match *float {
                    None => (),
                    Some(f_data) => {
                        if f_data.bounds.origin.y + f_data.bounds.size.height > top &&
                           f_data.bounds.origin.x + f_data.bounds.size.width > left &&
                           f_data.bounds.origin.x < left + width {
                               let new_y = f_data.bounds.origin.y;
                               max_height = Some(min(max_height.unwrap_or(new_y), new_y));
                           }
                    }
                }
            }
        }

        max_height.map(|h| h + self.offset.y)
    }

    /// Given necessary info, finds the closest place a box can be positioned
    /// without colliding with any floats.
    fn place_between_floats(&self, info: &PlacementInfo) -> Rect<Au>{
        debug!("place_float: Placing float with width {} and height {}", info.width, info.height);
        // Can't go any higher than previous floats or
        // previous elements in the document.
        let mut float_y = info.ceiling;
        loop {
            let maybe_location = self.available_rect(float_y, info.height, info.max_width);
            debug!("place_float: Got available rect: {:?} for y-pos: {}", maybe_location, float_y);
            match maybe_location {

                // If there are no floats blocking us, return the current location
                // TODO(eatkinson): integrate with overflow
                None => return match info.f_type { 
                    FloatLeft => Rect(Point2D(Au(0), float_y), 
                                      Size2D(info.max_width, Au(max_value))),

                    FloatRight => Rect(Point2D(info.max_width - info.width, float_y), 
                                       Size2D(info.max_width, Au(max_value)))
                },

                Some(rect) => {
                    assert!(rect.origin.y + rect.size.height != float_y, 
                            "Non-terminating float placement");
                    
                    // Place here if there is enough room
                    if (rect.size.width >= info.width) {
                        let height = self.max_height_for_bounds(rect.origin.x, 
                                                                rect.origin.y, 
                                                                rect.size.width);
                        let height = height.unwrap_or(Au(max_value));
                        return match info.f_type {
                            FloatLeft => Rect(Point2D(rect.origin.x, float_y),
                                              Size2D(rect.size.width, height)),
                            FloatRight => {
                                Rect(Point2D(rect.origin.x + rect.size.width - info.width, float_y),
                                     Size2D(rect.size.width, height))
                            }
                        };
                    }

                    // Try to place at the next-lowest location.
                    // Need to be careful of fencepost errors.
                    float_y = rect.origin.y + rect.size.height;
                }
            }
        }
    }

    fn clearance(&self, clear: ClearType) -> Au {
        let mut clearance = Au(0);
        for floats in self.float_data.iter() {
            for float in floats.iter() {
                match *float {
                    None => (),
                    Some(f_data) => {
                        match (clear, f_data.f_type) {
                            (ClearLeft, FloatLeft) |
                            (ClearRight, FloatRight) |
                            (ClearBoth, _) => {
                                clearance = max(
                                    clearance,
                                    self.offset.y + f_data.bounds.origin.y + f_data.bounds.size.height);
                            }
                            _ => ()
                        }
                    }
                }
            }
        }
        clearance
    }
}

