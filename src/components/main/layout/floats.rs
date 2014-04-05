/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use geom::point::Point2D;
use geom::rect::Rect;
use geom::size::Size2D;
use servo_util::cowarc::CowArc;
use servo_util::geometry::{Au, max, min};
use std::i32;
use style::computed_values::float;

/// The kind of float: left or right.
#[deriving(Clone)]
pub enum FloatKind {
    FloatLeft,
    FloatRight
}

impl FloatKind {
    pub fn from_property(property: float::T) -> FloatKind {
        match property {
            float::none => fail!("can't create a float type from an unfloated property"),
            float::left => FloatLeft,
            float::right => FloatRight,
        }
    }
}

/// The kind of clearance: left, right, or both.
pub enum ClearType {
    ClearLeft,
    ClearRight,
    ClearBoth,
}

/// Information about a single float.
#[deriving(Clone)]
struct Float {
    /// The boundaries of this float.
    bounds: Rect<Au>,
    /// The kind of float: left or right.
    kind: FloatKind,
}

/// Information about the floats next to a flow.
///
/// FIXME(pcwalton): When we have fast `MutexArc`s, try removing `#[deriving(Clone)]` and wrap in a
/// mutex.
#[deriving(Clone)]
struct FloatList {
    /// Information about each of the floats here.
    floats: ~[Float],
    /// Cached copy of the maximum top offset of the float.
    max_top: Au,
}

impl FloatList {
    fn new() -> FloatList {
        FloatList {
            floats: ~[],
            max_top: Au(0),
        }
    }
}

/// Wraps a `FloatList` to avoid allocation in the common case of no floats.
///
/// FIXME(pcwalton): When we have fast `MutexArc`s, try removing `CowArc` and use a mutex instead.
#[deriving(Clone)]
struct FloatListRef {
    list: Option<CowArc<FloatList>>,
}

impl FloatListRef {
    fn new() -> FloatListRef {
        FloatListRef {
            list: None,
        }
    }

    /// Returns true if the list is allocated and false otherwise. If false, there are guaranteed
    /// not to be any floats.
    fn is_present(&self) -> bool {
        self.list.is_some()
    }

    #[inline]
    fn get<'a>(&'a self) -> Option<&'a FloatList> {
        match self.list {
            None => None,
            Some(ref list) => Some(list.get()),
        }
    }

    #[inline]
    fn get_mut<'a>(&'a mut self) -> &'a mut FloatList {
        if self.list.is_none() {
            self.list = Some(CowArc::new(FloatList::new()))
        }
        self.list.as_mut().unwrap().get_mut()
    }
}

/// All the information necessary to place a float.
pub struct PlacementInfo {
    /// The dimensions of the float.
    pub size: Size2D<Au>,
    /// The minimum top of the float, as determined by earlier elements.
    pub ceiling: Au,
    /// The maximum right position of the float, generally determined by the containing block.
    pub max_width: Au,
    /// The kind of float.
    pub kind: FloatKind
}

fn range_intersect(top_1: Au, bottom_1: Au, top_2: Au, bottom_2: Au) -> (Au, Au) {
    (max(top_1, top_2), min(bottom_1, bottom_2))
}

/// Encapsulates information about floats. This is optimized to avoid allocation if there are
/// no floats, and to avoid copying when translating the list of floats downward.
#[deriving(Clone)]
pub struct Floats {
    /// The list of floats.
    list: FloatListRef,
    /// The offset of the flow relative to the first float.
    offset: Point2D<Au>,
}

impl Floats {
    /// Creates a new `Floats` object.
    pub fn new() -> Floats {
        Floats {
            list: FloatListRef::new(),
            offset: Point2D(Au(0), Au(0)),
        }
    }

    /// Adjusts the recorded offset of the flow relative to the first float.
    pub fn translate(&mut self, delta: Point2D<Au>) {
        self.offset = self.offset + delta
    }

    /// Returns the position of the last float in flow coordinates.
    pub fn last_float_pos(&self) -> Option<Point2D<Au>> {
        match self.list.get() {
            None => None,
            Some(list) => {
                match list.floats.last() {
                    None => None,
                    Some(float) => Some(float.bounds.origin + self.offset),
                }
            }
        }
    }

    /// Returns a rectangle that encloses the region from top to top + height, with width small
    /// enough that it doesn't collide with any floats. max_x is the x-coordinate beyond which
    /// floats have no effect. (Generally this is the containing block width.)
    pub fn available_rect(&self, top: Au, height: Au, max_x: Au) -> Option<Rect<Au>> {
        let list = match self.list.get() {
            None => return None,
            Some(list) => list,
        };

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
        for float in list.floats.iter() {
            debug!("available_rect: Checking for collision against float");
            let float_pos = float.bounds.origin;
            let float_size = float.bounds.size;

            debug!("float_pos: {}, float_size: {}", float_pos, float_size);
            match float.kind {
                FloatLeft if float_pos.x + float_size.width > max_left &&
                        float_pos.y + float_size.height > top && float_pos.y < top + height => {
                    max_left = float_pos.x + float_size.width;

                    l_top = Some(float_pos.y);
                    l_bottom = Some(float_pos.y + float_size.height);

                    debug!("available_rect: collision with left float: new max_left is {}",
                            max_left);
                }
                FloatRight if float_pos.x < min_right &&
                       float_pos.y + float_size.height > top && float_pos.y < top + height => {
                    min_right = float_pos.x;

                    r_top = Some(float_pos.y);
                    r_bottom = Some(float_pos.y + float_size.height);
                    debug!("available_rect: collision with right float: new min_right is {}",
                            min_right);
                }
                FloatLeft | FloatRight => {}
            }
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

        // FIXME(eatkinson): This assertion is too strong and fails in some cases. It is OK to
        // return negative widths since we check against that right away, but we should still
        // undersrtand why they occur and add a stronger assertion here.
        // assert!(max_left < min_right);

        assert!(top <= bottom, "Float position error");

        Some(Rect {
            origin: Point2D(max_left, top) + self.offset,
            size: Size2D(min_right - max_left, bottom - top)
        })
    }

    /// Adds a new float to the list.
    pub fn add_float(&mut self, info: &PlacementInfo) {
        let new_info;
        {
            let list = self.list.get_mut();
            new_info = PlacementInfo {
                size: info.size,
                ceiling: max(info.ceiling, list.max_top + self.offset.y),
                max_width: info.max_width,
                kind: info.kind
            }
        }

        debug!("add_float: added float with info {:?}", new_info);

        let new_float = Float {
            bounds: Rect {
                origin: self.place_between_floats(&new_info).origin - self.offset,
                size: info.size,
            },
            kind: info.kind
        };

        let list = self.list.get_mut();
        list.floats.push(new_float);
        list.max_top = max(list.max_top, new_float.bounds.origin.y);
    }

    /// Given the top 3 sides of the rectangle, finds the largest height that will result in the
    /// rectangle not colliding with any floats. Returns None if that height is infinite.
    fn max_height_for_bounds(&self, left: Au, top: Au, width: Au) -> Option<Au> {
        let list = match self.list.get() {
            None => return None,
            Some(list) => list,
        };

        let top = top - self.offset.y;
        let left = left - self.offset.x;
        let mut max_height = None;

        for float in list.floats.iter() {
            if float.bounds.origin.y + float.bounds.size.height > top &&
                   float.bounds.origin.x + float.bounds.size.width > left &&
                   float.bounds.origin.x < left + width {
               let new_y = float.bounds.origin.y;
               max_height = Some(min(max_height.unwrap_or(new_y), new_y));
            }
        }

        max_height.map(|h| h + self.offset.y)
    }

    /// Given placement information, finds the closest place a box can be positioned without
    /// colliding with any floats.
    pub fn place_between_floats(&self, info: &PlacementInfo) -> Rect<Au> {
        debug!("place_between_floats: Placing object with width {} and height {}",
               info.size.width,
               info.size.height);

        // If no floats, use this fast path.
        if !self.list.is_present() {
            match info.kind {
                FloatLeft => {
                    return Rect(Point2D(Au(0), info.ceiling),
                                Size2D(info.max_width, Au(i32::MAX)))
                }
                FloatRight => {
                    return Rect(Point2D(info.max_width - info.size.width, info.ceiling),
                                Size2D(info.max_width, Au(i32::MAX)))
                }
            }
        }

        // Can't go any higher than previous floats or previous elements in the document.
        let mut float_y = info.ceiling;
        loop {
            let maybe_location = self.available_rect(float_y, info.size.height, info.max_width);
            debug!("place_float: Got available rect: {:?} for y-pos: {}", maybe_location, float_y);
            match maybe_location {
                // If there are no floats blocking us, return the current location
                // TODO(eatkinson): integrate with overflow
                None => {
                    return match info.kind {
                        FloatLeft => {
                            Rect(Point2D(Au(0), float_y),
                                 Size2D(info.max_width, Au(i32::MAX)))
                        }
                        FloatRight => {
                            Rect(Point2D(info.max_width - info.size.width, float_y),
                                         Size2D(info.max_width, Au(i32::MAX)))
                        }
                    }
                }
                Some(rect) => {
                    assert!(rect.origin.y + rect.size.height != float_y,
                            "Non-terminating float placement");

                    // Place here if there is enough room
                    if rect.size.width >= info.size.width {
                        let height = self.max_height_for_bounds(rect.origin.x,
                                                                rect.origin.y,
                                                                rect.size.width);
                        let height = height.unwrap_or(Au(i32::MAX));
                        return match info.kind {
                            FloatLeft => {
                                Rect(Point2D(rect.origin.x, float_y),
                                             Size2D(rect.size.width, height))
                            }
                            FloatRight => {
                                Rect(Point2D(rect.origin.x + rect.size.width - info.size.width,
                                             float_y),
                                     Size2D(rect.size.width, height))
                            }
                        }
                    }

                    // Try to place at the next-lowest location.
                    // Need to be careful of fencepost errors.
                    float_y = rect.origin.y + rect.size.height;
                }
            }
        }
    }

    pub fn clearance(&self, clear: ClearType) -> Au {
        let list = match self.list.get() {
            None => return Au(0),
            Some(list) => list,
        };

        let mut clearance = Au(0);
        for float in list.floats.iter() {
            match (clear, float.kind) {
                (ClearLeft, FloatLeft) |
                (ClearRight, FloatRight) |
                (ClearBoth, _) => {
                    let y = self.offset.y + float.bounds.origin.y + float.bounds.size.height;
                    clearance = max(clearance, y);
                }
                _ => {}
            }
        }
        clearance
    }
}

