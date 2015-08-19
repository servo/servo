/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use std::cmp::{max, min};
use std::fmt;
use std::i32;
use style::computed_values::float;
use util::geometry::Au;
use util::logical_geometry::WritingMode;
use util::logical_geometry::{LogicalRect, LogicalSize};
use util::persistent_list::PersistentList;

/// The kind of float: left or right.
#[derive(Clone, RustcEncodable, Debug, Copy)]
pub enum FloatKind {
    Left,
    Right
}

impl FloatKind {
    pub fn from_property(property: float::T) -> Option<FloatKind> {
        match property {
            float::T::none => None,
            float::T::left => Some(FloatKind::Left),
            float::T::right => Some(FloatKind::Right),
        }
    }
}

/// The kind of clearance: left, right, or both.
#[derive(Copy, Clone)]
pub enum ClearType {
    Left,
    Right,
    Both,
}

/// Information about a single float.
#[derive(Clone, Copy)]
struct Float {
    /// The boundaries of this float.
    bounds: LogicalRect<Au>,
    /// The kind of float: left or right.
    kind: FloatKind,
}

impl fmt::Debug for Float {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "bounds={:?} kind={:?}", self.bounds, self.kind)
    }
}

/// Information about the floats next to a flow.
#[derive(Clone)]
struct FloatList {
    /// Information about each of the floats here.
    floats: PersistentList<Float>,
    /// Cached copy of the maximum block-start offset of the float.
    max_block_start: Au,
}

impl FloatList {
    fn new() -> FloatList {
        FloatList {
            floats: PersistentList::new(),
            max_block_start: Au(0),
        }
    }

    /// Returns true if the list is allocated and false otherwise. If false, there are guaranteed
    /// not to be any floats.
    fn is_present(&self) -> bool {
        self.floats.len() > 0
    }
}

impl fmt::Debug for FloatList {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "max_block_start={:?} floats={}", self.max_block_start, self.floats.len())
    }
}

/// All the information necessary to place a float.
pub struct PlacementInfo {
    /// The dimensions of the float.
    pub size: LogicalSize<Au>,
    /// The minimum block-start of the float, as determined by earlier elements.
    pub ceiling: Au,
    /// The maximum inline-end position of the float, generally determined by the containing block.
    pub max_inline_size: Au,
    /// The kind of float.
    pub kind: FloatKind
}

impl fmt::Debug for PlacementInfo {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f,
               "size={:?} ceiling={:?} max_inline_size={:?} kind={:?}",
               self.size,
               self.ceiling,
               self.max_inline_size,
               self.kind)
    }
}

fn range_intersect(block_start_1: Au, block_end_1: Au, block_start_2: Au, block_end_2: Au) -> (Au, Au) {
    (max(block_start_1, block_start_2), min(block_end_1, block_end_2))
}

/// Encapsulates information about floats. This is optimized to avoid allocation if there are
/// no floats, and to avoid copying when translating the list of floats downward.
#[derive(Clone)]
pub struct Floats {
    /// The list of floats.
    list: FloatList,
    /// The offset of the flow relative to the first float.
    offset: LogicalSize<Au>,
    /// The writing mode of these floats.
    pub writing_mode: WritingMode,
}

impl fmt::Debug for Floats {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        if !self.list.is_present() {
            write!(f, "[empty]")
        } else {
            write!(f, "offset={:?} floats={:?}", self.offset, self.list)
        }
    }
}

impl Floats {
    /// Creates a new `Floats` object.
    pub fn new(writing_mode: WritingMode) -> Floats {
        Floats {
            list: FloatList::new(),
            offset: LogicalSize::zero(writing_mode),
            writing_mode: writing_mode,
        }
    }

    /// Adjusts the recorded offset of the flow relative to the first float.
    pub fn translate(&mut self, delta: LogicalSize<Au>) {
        self.offset = self.offset + delta
    }

    /// Returns the position of the last float in flow coordinates.
    pub fn last_float_pos(&self) -> Option<LogicalRect<Au>> {
        match self.list.floats.front() {
            None => None,
            Some(float) => Some(float.bounds.translate_by_size(self.offset)),
        }
    }

    pub fn len(&self) -> usize {
        self.list.floats.len()
    }

    /// Returns a rectangle that encloses the region from block-start to block-start + block-size,
    /// with inline-size small enough that it doesn't collide with any floats. max_x is the
    /// inline-size beyond which floats have no effect. (Generally this is the containing block
    /// inline-size.)
    pub fn available_rect(&self, block_start: Au, block_size: Au, max_x: Au)
                          -> Option<LogicalRect<Au>> {
        let list = &self.list;
        let block_start = block_start - self.offset.block;

        debug!("available_rect: trying to find space at {:?}", block_start);

        // Relevant dimensions for the inline-end-most inline-start float
        let mut max_inline_start = Au(0) - self.offset.inline;
        let mut l_block_start = None;
        let mut l_block_end = None;
        // Relevant dimensions for the inline-start-most inline-end float
        let mut min_inline_end = max_x - self.offset.inline;
        let mut r_block_start = None;
        let mut r_block_end = None;

        // Find the float collisions for the given range in the block direction.
        for float in list.floats.iter() {
            debug!("available_rect: Checking for collision against float");
            let float_pos = float.bounds.start;
            let float_size = float.bounds.size;

            debug!("float_pos: {:?}, float_size: {:?}", float_pos, float_size);
            match float.kind {
                FloatKind::Left if float_pos.i + float_size.inline > max_inline_start &&
                        float_pos.b + float_size.block > block_start &&
                        float_pos.b < block_start + block_size => {
                    max_inline_start = float_pos.i + float_size.inline;

                    l_block_start = Some(float_pos.b);
                    l_block_end = Some(float_pos.b + float_size.block);

                    debug!("available_rect: collision with inline_start float: new \
                            max_inline_start is {:?}",
                           max_inline_start);
                }
                FloatKind::Right if float_pos.i < min_inline_end &&
                       float_pos.b + float_size.block > block_start &&
                       float_pos.b < block_start + block_size => {
                    min_inline_end = float_pos.i;

                    r_block_start = Some(float_pos.b);
                    r_block_end = Some(float_pos.b + float_size.block);
                    debug!("available_rect: collision with inline_end float: new min_inline_end \
                            is {:?}",
                            min_inline_end);
                }
                FloatKind::Left | FloatKind::Right => {}
            }
        }

        // Extend the vertical range of the rectangle to the closest floats.
        // If there are floats on both sides, take the intersection of the
        // two areas. Also make sure we never return a block-start smaller than the
        // given upper bound.
        let (block_start, block_end) = match (r_block_start,
                                              r_block_end,
                                              l_block_start,
                                              l_block_end) {
            (Some(r_block_start), Some(r_block_end), Some(l_block_start), Some(l_block_end)) => {
                range_intersect(max(block_start, r_block_start),
                                r_block_end,
                                max(block_start, l_block_start),
                                l_block_end)
            }
            (None, None, Some(l_block_start), Some(l_block_end)) => {
                (max(block_start, l_block_start), l_block_end)
            }
            (Some(r_block_start), Some(r_block_end), None, None) => {
                (max(block_start, r_block_start), r_block_end)
            }
            (None, None, None, None) => return None,
            _ => panic!("Reached unreachable state when computing float area")
        };

        // FIXME(eatkinson): This assertion is too strong and fails in some cases. It is OK to
        // return negative inline-sizes since we check against that inline-end away, but we should
        // still understand why they occur and add a stronger assertion here.
        // assert!(max_inline-start < min_inline-end);

        assert!(block_start <= block_end, "Float position error");

        Some(LogicalRect::new(self.writing_mode,
                              max_inline_start + self.offset.inline,
                              block_start + self.offset.block,
                              min_inline_end - max_inline_start,
                              block_end - block_start))
    }

    /// Adds a new float to the list.
    pub fn add_float(&mut self, info: &PlacementInfo) {
        let new_info;
        {
            new_info = PlacementInfo {
                size: info.size,
                ceiling: max(info.ceiling, self.list.max_block_start + self.offset.block),
                max_inline_size: info.max_inline_size,
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

        self.list.floats = self.list.floats.prepend_elem(new_float);
        self.list.max_block_start = max(self.list.max_block_start, new_float.bounds.start.b);
    }

    /// Given the three sides of the bounding rectangle in the block-start direction, finds the
    /// largest block-size that will result in the rectangle not colliding with any floats. Returns
    /// `None` if that block-size is infinite.
    fn max_block_size_for_bounds(&self, inline_start: Au, block_start: Au, inline_size: Au)
                                 -> Option<Au> {
        let list = &self.list;

        let block_start = block_start - self.offset.block;
        let inline_start = inline_start - self.offset.inline;
        let mut max_block_size = None;

        for float in list.floats.iter() {
            if float.bounds.start.b + float.bounds.size.block > block_start &&
                   float.bounds.start.i + float.bounds.size.inline > inline_start &&
                   float.bounds.start.i < inline_start + inline_size {
               let new_y = float.bounds.start.b;
               max_block_size = Some(min(max_block_size.unwrap_or(new_y), new_y));
            }
        }

        max_block_size.map(|h| h + self.offset.block)
    }

    /// Given placement information, finds the closest place a fragment can be positioned without
    /// colliding with any floats.
    pub fn place_between_floats(&self, info: &PlacementInfo) -> LogicalRect<Au> {
        debug!("place_between_floats: Placing object with {:?}", info.size);

        // If no floats, use this fast path.
        if !self.list.is_present() {
            match info.kind {
                FloatKind::Left => {
                    return LogicalRect::new(
                        self.writing_mode,
                        Au(0),
                        info.ceiling,
                        info.max_inline_size,
                        Au(i32::MAX))
                }
                FloatKind::Right => {
                    return LogicalRect::new(
                        self.writing_mode,
                        info.max_inline_size - info.size.inline,
                        info.ceiling,
                        info.max_inline_size,
                        Au(i32::MAX))
                }
            }
        }

        // Can't go any higher than previous floats or previous elements in the document.
        let mut float_b = info.ceiling;
        loop {
            let maybe_location = self.available_rect(float_b,
                                                     info.size.block,
                                                     info.max_inline_size);
            debug!("place_float: got available rect: {:?} for block-pos: {:?}",
                   maybe_location,
                   float_b);
            match maybe_location {
                // If there are no floats blocking us, return the current location
                // TODO(eatkinson): integrate with overflow
                None => {
                    return match info.kind {
                        FloatKind::Left => {
                            LogicalRect::new(
                                self.writing_mode,
                                Au(0),
                                float_b,
                                info.max_inline_size,
                                Au(i32::MAX))
                        }
                        FloatKind::Right => {
                            LogicalRect::new(
                                self.writing_mode,
                                info.max_inline_size - info.size.inline,
                                float_b,
                                info.max_inline_size,
                                Au(i32::MAX))
                        }
                    }
                }
                Some(rect) => {
                    assert!(rect.start.b + rect.size.block != float_b,
                            "Non-terminating float placement");

                    // Place here if there is enough room
                    if rect.size.inline >= info.size.inline {
                        let block_size = self.max_block_size_for_bounds(rect.start.i,
                                                                        rect.start.b,
                                                                        rect.size.inline);
                        let block_size = block_size.unwrap_or(Au(i32::MAX));
                        return match info.kind {
                            FloatKind::Left => {
                                LogicalRect::new(
                                    self.writing_mode,
                                    rect.start.i,
                                    float_b,
                                    rect.size.inline,
                                    block_size)
                            }
                            FloatKind::Right => {
                                LogicalRect::new(
                                    self.writing_mode,
                                    rect.start.i + rect.size.inline - info.size.inline,
                                    float_b,
                                    rect.size.inline,
                                    block_size)
                            }
                        }
                    }

                    // Try to place at the next-lowest location.
                    // Need to be careful of fencepost errors.
                    float_b = rect.start.b + rect.size.block;
                }
            }
        }
    }

    pub fn clearance(&self, clear: ClearType) -> Au {
        let list = &self.list;
        let mut clearance = Au(0);
        for float in list.floats.iter() {
            match (clear, float.kind) {
                (ClearType::Left, FloatKind::Left) |
                (ClearType::Right, FloatKind::Right) |
                (ClearType::Both, _) => {
                    let b = self.offset.block + float.bounds.start.b + float.bounds.size.block;
                    clearance = max(clearance, b);
                }
                _ => {}
            }
        }
        clearance
    }
}
