/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use geom::point::Point2D;
use geom::size::Size2D;
use geom::rect::Rect;
use gfx::geometry::{Au, max, min};
use std::util::replace;
use std::vec;

pub enum FloatType{
    FloatLeft,
    FloatRight
}

struct FloatContextBase{
    float_data: ~[Option<FloatData>],
    floats_used: uint,
    max_y : Au,
    offset: Point2D<Au>
}

struct FloatData{
    bounds: Rect<Au>,
    f_type: FloatType
}

/// All information necessary to place a float
pub struct PlacementInfo{
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
    fn with_mut_base<R>(&mut self, callback: &fn(&mut FloatContextBase) -> R) -> R {
        match *self {
            Invalid => fail!("Float context no longer available"),
            Valid(ref mut base) => callback(base)
        }
    }

    #[inline(always)]
    pub fn with_base<R>(&self, callback: &fn(&FloatContextBase) -> R) -> R {
        match *self {
            Invalid => fail!("Float context no longer available"),
            Valid(ref base) => callback(base)
        }
    }

    #[inline(always)]
    pub fn translate(&mut self, trans: Point2D<Au>) -> FloatContext {
        do self.with_mut_base |base| {
            base.translate(trans);
        }
        replace(self, Invalid)
    }

    #[inline(always)]
    pub fn available_rect(&mut self, top: Au, height: Au, max_x: Au) -> Option<Rect<Au>> {
        do self.with_base |base| {
            base.available_rect(top, height, max_x)
        }
    }

    #[inline(always)]
    pub fn add_float(&mut self, info: &PlacementInfo) -> FloatContext{
        do self.with_mut_base |base| {
            base.add_float(info);
        }
        replace(self, Invalid)
    }

    #[inline(always)]
    pub fn last_float_pos(&mut self) -> Point2D<Au> {
        do self.with_base |base| {
            base.last_float_pos()
        }
    }
}

impl FloatContextBase{
    fn new(num_floats: uint) -> FloatContextBase {
        debug!("Creating float context of size %?", num_floats);
        let new_data = vec::from_elem(num_floats, None);
        FloatContextBase {
            float_data: new_data,
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

        match self.float_data[self.floats_used - 1] {
            None => fail!("FloatContext error: floats should never be None here"),
            Some(float) => {
                debug!("Returning float position: %?", float.bounds.origin + self.offset);
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

        debug!("available_rect: trying to find space at %?", top);

        let top = top - self.offset.y;

        // Relevant dimensions for the right-most left float
        let mut max_left = Au(0) - self.offset.x;
        let mut l_top = None;
        let mut l_bottom = None;
        // Relevant dimensions for the left-most right float
        let mut min_right = max_x - self.offset.x;
        let mut r_top = None;
        let mut r_bottom = None;

        // Find the float collisions for the given vertical range.
        for self.float_data.iter().advance |float| {
            match *float{
                None => (),
                Some(data) => {
                    let float_pos = data.bounds.origin;
                    let float_size = data.bounds.size;
                    match data.f_type {
                        FloatLeft => {
                            if(float_pos.x + float_size.width > max_left && 
                               float_pos.y + float_size.height > top && float_pos.y < top + height) {
                                max_left = float_pos.x + float_size.width;
                            
                                l_top = Some(float_pos.y);
                                l_bottom = Some(float_pos.y + float_size.height);
                            }
                        }
                        FloatRight => {
                            if(float_pos.x < min_right && 
                               float_pos.y + float_size.height > top && float_pos.y < top + height) {
                                min_right = float_pos.x;

                                r_top = Some(float_pos.y);
                                r_bottom = Some(float_pos.y + float_size.height);
                            }
                        }
                    }
                }
            };
        }

        // Extend the vertical range of the rectangle to the closest floats.
        // If there are floats on both sides, take the intersection of the
        // two areas.
        let (top, bottom) = match (r_top, r_bottom, l_top, l_bottom) {
            (Some(r_top), Some(r_bottom), Some(l_top), Some(l_bottom)) => 
                range_intersect(r_top, r_bottom, l_top, l_bottom),

            (None, None, Some(l_top), Some(l_bottom)) => (l_top, l_bottom),
            (Some(r_top), Some(r_bottom), None, None) => (r_top, r_bottom),
            (None, None, None, None) => return None,
            _ => fail!("Reached unreachable state when computing float area")
        };

        // When the window is smaller than the float, we will return a rect
        // with negative width.
        assert!(max_left < min_right 
                || max_left > max_x - self.offset.x
                || min_right < Au(0) - self.offset.x
                ,"Float position error");

        //TODO(eatkinson): do we need to do something similar for heights?
        assert!(top < bottom, "Float position error");

        Some(Rect{
            origin: Point2D(max_left, top) + self.offset,
            size: Size2D(min_right - max_left, bottom - top)
        })
    }

    fn add_float(&mut self, info: &PlacementInfo) {
        debug!("Floats_used: %?, Floats available: %?", self.floats_used, self.float_data.len());
        assert!(self.floats_used < self.float_data.len() && 
                self.float_data[self.floats_used].is_none());

        let new_float = FloatData {    
            bounds: Rect {
                origin: self.place_float(info) - self.offset,
                size: Size2D(info.width, info.height)
            },
            f_type: info.f_type
        };
        self.float_data[self.floats_used] = Some(new_float);
        self.floats_used += 1;
    }

    /// Given necessary info, finds the position of the float in
    /// LOCAL COORDINATES. i.e. must be translated before placed
    /// in the float list
    fn place_float(&self, info: &PlacementInfo) -> Point2D<Au>{
        debug!("place_float: Placing float with width %? and height %?", info.width, info.height);
        // Can't go any higher than previous floats or
        // previous elements in the document.
        let mut float_y = max(info.ceiling, self.max_y + self.offset.y);
        loop {
            let maybe_location = self.available_rect(float_y, info.height, info.max_width);
            debug!("place_float: Got available rect: %? for y-pos: %?", maybe_location, float_y);
            match maybe_location {
                // If there are no floats blocking us, return the current location
                // TODO(eatknson): integrate with overflow
                None => return Point2D(Au(0), float_y),
                Some(rect) => {
                    assert!(rect.origin.y + rect.size.height != float_y, 
                            "Non-terminating float placement");
                    
                    // Place here if there is enough room
                    if (rect.size.width >= info.width) {
                        return Point2D(rect.origin.x, float_y);
                    }

                    // Try to place at the next-lowest location.
                    // Need to be careful of fencepost errors.
                    float_y = rect.origin.y + rect.size.height;
                }
            }
        }
    }
}

