/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use servo_util::geometry::{Au, max, min};
use servo_util::logical_geometry::WritingMode;
use servo_util::logical_geometry::{LogicalPoint, LogicalRect, LogicalSize};
use std::i32;
use std::fmt;
use style::computed_values::float;
use sync::Arc;

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
    bounds: LogicalRect<Au>,
    /// The kind of float: left or right.
    kind: FloatKind,
}

impl fmt::Show for Float {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "bounds={} kind={:?}", self.bounds, self.kind)
    }
}

/// Information about the floats next to a flow.
///
/// FIXME(pcwalton): When we have fast `MutexArc`s, try removing `#[deriving(Clone)]` and wrap in a
/// mutex.
#[deriving(Clone)]
struct FloatList {
    /// Information about each of the floats here.
    floats: Vec<Float>,
    /// Cached copy of the maximum bstart offset of the float.
    max_bstart: Au,
}

impl FloatList {
    fn new() -> FloatList {
        FloatList {
            floats: vec!(),
            max_bstart: Au(0),
        }
    }
}

impl fmt::Show for FloatList {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "max_bstart={} floats={:?}", self.max_bstart, self.floats)
    }
}

/// Wraps a `FloatList` to avoid allocation in the common case of no floats.
///
/// FIXME(pcwalton): When we have fast `MutexArc`s, try removing `CowArc` and use a mutex instead.
#[deriving(Clone)]
struct FloatListRef {
    list: Option<Arc<FloatList>>,
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
            Some(ref list) => Some(&**list),
        }
    }

    #[allow(experimental)]
    #[inline]
    fn get_mut<'a>(&'a mut self) -> &'a mut FloatList {
        if self.list.is_none() {
            self.list = Some(Arc::new(FloatList::new()))
        }
        self.list.as_mut().unwrap().make_unique()
    }
}

/// All the information necessary to place a float.
pub struct PlacementInfo {
    /// The dimensions of the float.
    pub size: LogicalSize<Au>,
    /// The minimum bstart of the float, as determined by earlier elements.
    pub ceiling: Au,
    /// The maximum iend position of the float, generally determined by the containing block.
    pub max_isize: Au,
    /// The kind of float.
    pub kind: FloatKind
}

impl fmt::Show for PlacementInfo {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "size={} ceiling={} max_isize={} kind={:?}", self.size, self.ceiling, self.max_isize, self.kind)
    }
}

fn range_intersect(bstart_1: Au, bend_1: Au, bstart_2: Au, bend_2: Au) -> (Au, Au) {
    (max(bstart_1, bstart_2), min(bend_1, bend_2))
}

/// Encapsulates information about floats. This is optimized to avoid allocation if there are
/// no floats, and to avoid copying when translating the list of floats downward.
#[deriving(Clone)]
pub struct Floats {
    /// The list of floats.
    list: FloatListRef,
    /// The offset of the flow relative to the first float.
    offset: LogicalSize<Au>,
    pub writing_mode: WritingMode,
}

impl fmt::Show for Floats {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self.list.get() {
            None => {
                write!(f, "[empty]")
            }
            Some(list) => {
                write!(f, "offset={} floats={}", self.offset, list)
            }
        }
    }
}

impl Floats {
    /// Creates a new `Floats` object.
    pub fn new(writing_mode: WritingMode) -> Floats {
        Floats {
            list: FloatListRef::new(),
            offset: LogicalSize::zero(writing_mode),
            writing_mode: writing_mode,
        }
    }

    /// Adjusts the recorded offset of the flow relative to the first float.
    pub fn translate(&mut self, delta: LogicalSize<Au>) {
        self.offset = self.offset + delta
    }

    /// Returns the position of the last float in flow coordinates.
    pub fn last_float_pos(&self) -> Option<LogicalPoint<Au>> {
        match self.list.get() {
            None => None,
            Some(list) => {
                match list.floats.last() {
                    None => None,
                    Some(float) => Some(float.bounds.start + self.offset),
                }
            }
        }
    }

    /// Returns a rectangle that encloses the region from bstart to bstart + bsize, with isize small
    /// enough that it doesn't collide with any floats. max_x is the x-coordinate beyond which
    /// floats have no effect. (Generally this is the containing block isize.)
    pub fn available_rect(&self, bstart: Au, bsize: Au, max_x: Au) -> Option<LogicalRect<Au>> {
        let list = match self.list.get() {
            None => return None,
            Some(list) => list,
        };

        let bstart = bstart - self.offset.bsize;

        debug!("available_rect: trying to find space at {}", bstart);

        // Relevant dimensions for the iend-most istart float
        let mut max_istart = Au(0) - self.offset.isize;
        let mut l_bstart = None;
        let mut l_bend = None;
        // Relevant dimensions for the istart-most iend float
        let mut min_iend = max_x - self.offset.isize;
        let mut r_bstart = None;
        let mut r_bend = None;

        // Find the float collisions for the given vertical range.
        for float in list.floats.iter() {
            debug!("available_rect: Checking for collision against float");
            let float_pos = float.bounds.start;
            let float_size = float.bounds.size;

            debug!("float_pos: {}, float_size: {}", float_pos, float_size);
            match float.kind {
                FloatLeft if float_pos.i + float_size.isize > max_istart &&
                        float_pos.b + float_size.bsize > bstart && float_pos.b < bstart + bsize => {
                    max_istart = float_pos.i + float_size.isize;

                    l_bstart = Some(float_pos.b);
                    l_bend = Some(float_pos.b + float_size.bsize);

                    debug!("available_rect: collision with istart float: new max_istart is {}",
                            max_istart);
                }
                FloatRight if float_pos.i < min_iend &&
                       float_pos.b + float_size.bsize > bstart && float_pos.b < bstart + bsize => {
                    min_iend = float_pos.i;

                    r_bstart = Some(float_pos.b);
                    r_bend = Some(float_pos.b + float_size.bsize);
                    debug!("available_rect: collision with iend float: new min_iend is {}",
                            min_iend);
                }
                FloatLeft | FloatRight => {}
            }
        }

        // Extend the vertical range of the rectangle to the closest floats.
        // If there are floats on both sides, take the intersection of the
        // two areas. Also make sure we never return a bstart smaller than the
        // given upper bound.
        let (bstart, bend) = match (r_bstart, r_bend, l_bstart, l_bend) {
            (Some(r_bstart), Some(r_bend), Some(l_bstart), Some(l_bend)) =>
                range_intersect(max(bstart, r_bstart), r_bend, max(bstart, l_bstart), l_bend),

            (None, None, Some(l_bstart), Some(l_bend)) => (max(bstart, l_bstart), l_bend),
            (Some(r_bstart), Some(r_bend), None, None) => (max(bstart, r_bstart), r_bend),
            (None, None, None, None) => return None,
            _ => fail!("Reached unreachable state when computing float area")
        };

        // FIXME(eatkinson): This assertion is too strong and fails in some cases. It is OK to
        // return negative isizes since we check against that iend away, but we should still
        // undersrtand why they occur and add a stronger assertion here.
        // assert!(max_istart < min_iend);

        assert!(bstart <= bend, "Float position error");

        Some(LogicalRect::new(
            self.writing_mode, max_istart + self.offset.isize, bstart + self.offset.bsize,
            min_iend - max_istart, bend - bstart
        ))
    }

    /// Adds a new float to the list.
    pub fn add_float(&mut self, info: &PlacementInfo) {
        let new_info;
        {
            let list = self.list.get_mut();
            new_info = PlacementInfo {
                size: info.size,
                ceiling: max(info.ceiling, list.max_bstart + self.offset.bsize),
                max_isize: info.max_isize,
                kind: info.kind
            }
        }

        debug!("add_float: added float with info {:?}", new_info);

        let new_float = Float {
            bounds: LogicalRect::from_point_size(
                self.writing_mode,
                self.place_between_floats(&new_info).start - self.offset,
                info.size,
            ),
            kind: info.kind
        };

        let list = self.list.get_mut();
        list.floats.push(new_float);
        list.max_bstart = max(list.max_bstart, new_float.bounds.start.b);
    }

    /// Given the bstart 3 sides of the rectangle, finds the largest bsize that will result in the
    /// rectangle not colliding with any floats. Returns None if that bsize is infinite.
    fn max_bsize_for_bounds(&self, istart: Au, bstart: Au, isize: Au) -> Option<Au> {
        let list = match self.list.get() {
            None => return None,
            Some(list) => list,
        };

        let bstart = bstart - self.offset.bsize;
        let istart = istart - self.offset.isize;
        let mut max_bsize = None;

        for float in list.floats.iter() {
            if float.bounds.start.b + float.bounds.size.bsize > bstart &&
                   float.bounds.start.i + float.bounds.size.isize > istart &&
                   float.bounds.start.i < istart + isize {
               let new_y = float.bounds.start.b;
               max_bsize = Some(min(max_bsize.unwrap_or(new_y), new_y));
            }
        }

        max_bsize.map(|h| h + self.offset.bsize)
    }

    /// Given placement information, finds the closest place a fragment can be positioned without
    /// colliding with any floats.
    pub fn place_between_floats(&self, info: &PlacementInfo) -> LogicalRect<Au> {
        debug!("place_between_floats: Placing object with {}", info.size);

        // If no floats, use this fast path.
        if !self.list.is_present() {
            match info.kind {
                FloatLeft => {
                    return LogicalRect::new(
                        self.writing_mode,
                        Au(0),
                        info.ceiling,
                        info.max_isize,
                        Au(i32::MAX))
                }
                FloatRight => {
                    return LogicalRect::new(
                        self.writing_mode,
                        info.max_isize - info.size.isize,
                        info.ceiling,
                        info.max_isize,
                        Au(i32::MAX))
                }
            }
        }

        // Can't go any higher than previous floats or previous elements in the document.
        let mut float_b = info.ceiling;
        loop {
            let maybe_location = self.available_rect(float_b, info.size.bsize, info.max_isize);
            debug!("place_float: Got available rect: {:?} for y-pos: {}", maybe_location, float_b);
            match maybe_location {
                // If there are no floats blocking us, return the current location
                // TODO(eatkinson): integrate with overflow
                None => {
                    return match info.kind {
                        FloatLeft => {
                            LogicalRect::new(
                                self.writing_mode,
                                Au(0),
                                float_b,
                                info.max_isize,
                                Au(i32::MAX))
                        }
                        FloatRight => {
                            LogicalRect::new(
                                self.writing_mode,
                                info.max_isize - info.size.isize,
                                float_b,
                                info.max_isize,
                                Au(i32::MAX))
                        }
                    }
                }
                Some(rect) => {
                    assert!(rect.start.b + rect.size.bsize != float_b,
                            "Non-terminating float placement");

                    // Place here if there is enough room
                    if rect.size.isize >= info.size.isize {
                        let bsize = self.max_bsize_for_bounds(rect.start.i,
                                                                rect.start.b,
                                                                rect.size.isize);
                        let bsize = bsize.unwrap_or(Au(i32::MAX));
                        return match info.kind {
                            FloatLeft => {
                                LogicalRect::new(
                                    self.writing_mode,
                                    rect.start.i,
                                    float_b,
                                    rect.size.isize,
                                    bsize)
                            }
                            FloatRight => {
                                LogicalRect::new(
                                    self.writing_mode,
                                    rect.start.i + rect.size.isize - info.size.isize,
                                    float_b,
                                    rect.size.isize,
                                    bsize)
                            }
                        }
                    }

                    // Try to place at the next-lowest location.
                    // Need to be careful of fencepost errors.
                    float_b = rect.start.b + rect.size.bsize;
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
                    let b = self.offset.bsize + float.bounds.start.b + float.bounds.size.bsize;
                    clearance = max(clearance, b);
                }
                _ => {}
            }
        }
        clearance
    }
}

