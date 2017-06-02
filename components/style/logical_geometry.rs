/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

//! Geometry in flow-relative space.

use euclid::{Point2D, Rect, Size2D, SideOffsets2D};
use euclid::num::Zero;
use std::cmp::{max, min};
use std::fmt::{self, Debug, Error, Formatter};
use std::ops::{Add, Sub};
use unicode_bidi as bidi;

pub enum BlockFlowDirection {
    TopToBottom,
    RightToLeft,
    LeftToRight
}

pub enum InlineBaseDirection {
    LeftToRight,
    RightToLeft
}

// TODO: improve the readability of the WritingMode serialization, refer to the Debug:fmt()
bitflags!(
    #[cfg_attr(feature = "servo", derive(HeapSizeOf, Serialize))]
    pub flags WritingMode: u8 {
        const FLAG_RTL = 1 << 0,
        const FLAG_VERTICAL = 1 << 1,
        const FLAG_VERTICAL_LR = 1 << 2,
        /// For vertical writing modes only.  When set, line-over/line-under
        /// sides are inverted from block-start/block-end.  This flag is
        /// set when sideways-lr is used.
        const FLAG_LINE_INVERTED = 1 << 3,
        const FLAG_SIDEWAYS = 1 << 4,
        const FLAG_UPRIGHT = 1 << 5,
    }
);

impl WritingMode {
    #[inline]
    pub fn is_vertical(&self) -> bool {
        self.intersects(FLAG_VERTICAL)
    }

    /// Assuming .is_vertical(), does the block direction go left to right?
    #[inline]
    pub fn is_vertical_lr(&self) -> bool {
        self.intersects(FLAG_VERTICAL_LR)
    }

    /// Assuming .is_vertical(), does the inline direction go top to bottom?
    #[inline]
    pub fn is_inline_tb(&self) -> bool {
        // https://drafts.csswg.org/css-writing-modes-3/#logical-to-physical
        self.intersects(FLAG_RTL) == self.intersects(FLAG_LINE_INVERTED)
    }

    #[inline]
    pub fn is_bidi_ltr(&self) -> bool {
        !self.intersects(FLAG_RTL)
    }

    #[inline]
    pub fn is_sideways(&self) -> bool {
        self.intersects(FLAG_SIDEWAYS)
    }

    #[inline]
    pub fn is_upright(&self) -> bool {
        self.intersects(FLAG_UPRIGHT)
    }

    #[inline]
    pub fn inline_start_physical_side(&self) -> PhysicalSide {
        match (self.is_vertical(), self.is_inline_tb(), self.is_bidi_ltr()) {
            (false, _, true) => PhysicalSide::Left,
            (false, _, false) => PhysicalSide::Right,
            (true, true, _) => PhysicalSide::Top,
            (true, false, _) => PhysicalSide::Bottom,
        }
    }

    #[inline]
    pub fn inline_end_physical_side(&self) -> PhysicalSide {
        match (self.is_vertical(), self.is_inline_tb(), self.is_bidi_ltr()) {
            (false, _, true) => PhysicalSide::Right,
            (false, _, false) => PhysicalSide::Left,
            (true, true, _) => PhysicalSide::Bottom,
            (true, false, _) => PhysicalSide::Top,
        }
    }

    #[inline]
    pub fn block_start_physical_side(&self) -> PhysicalSide {
        match (self.is_vertical(), self.is_vertical_lr()) {
            (false, _) => PhysicalSide::Top,
            (true, true) => PhysicalSide::Left,
            (true, false) => PhysicalSide::Right,
        }
    }

    #[inline]
    pub fn block_end_physical_side(&self) -> PhysicalSide {
        match (self.is_vertical(), self.is_vertical_lr()) {
            (false, _) => PhysicalSide::Bottom,
            (true, true) => PhysicalSide::Right,
            (true, false) => PhysicalSide::Left,
        }
    }

    #[inline]
    pub fn block_flow_direction(&self) -> BlockFlowDirection {
        match (self.is_vertical(), self.is_vertical_lr()) {
            (false, _) => BlockFlowDirection::TopToBottom,
            (true, true) => BlockFlowDirection::LeftToRight,
            (true, false) => BlockFlowDirection::RightToLeft,
        }
    }

    #[inline]
    pub fn inline_base_direction(&self) -> InlineBaseDirection {
        if self.intersects(FLAG_RTL) {
            InlineBaseDirection::RightToLeft
        } else {
            InlineBaseDirection::LeftToRight
        }
    }

    #[inline]
    /// The default bidirectional embedding level for this writing mode.
    ///
    /// Returns bidi level 0 if the mode is LTR, or 1 otherwise.
    pub fn to_bidi_level(&self) -> bidi::Level {
        if self.is_bidi_ltr() {
            bidi::Level::ltr()
        } else {
            bidi::Level::rtl()
        }
    }
}

impl fmt::Display for WritingMode {
    fn fmt(&self, formatter: &mut Formatter) -> Result<(), Error> {
        if self.is_vertical() {
            try!(write!(formatter, "V"));
            if self.is_vertical_lr() {
                try!(write!(formatter, " LR"));
            } else {
                try!(write!(formatter, " RL"));
            }
            if self.intersects(FLAG_SIDEWAYS) {
                try!(write!(formatter, " Sideways"));
            }
            if self.intersects(FLAG_LINE_INVERTED) {
                try!(write!(formatter, " Inverted"));
            }
        } else {
            try!(write!(formatter, "H"));
        }
        if self.is_bidi_ltr() {
            write!(formatter, " LTR")
        } else {
            write!(formatter, " RTL")
        }
    }
}


/// Wherever logical geometry is used, the writing mode is known based on context:
/// every method takes a `mode` parameter.
/// However, this context is easy to get wrong.
/// In debug builds only, logical geometry objects store their writing mode
/// (in addition to taking it as a parameter to methods) and check it.
/// In non-debug builds, make this storage zero-size and the checks no-ops.
#[cfg(not(debug_assertions))]
#[derive(PartialEq, Eq, Clone, Copy)]
#[cfg_attr(feature = "servo", derive(Serialize))]
struct DebugWritingMode;

#[cfg(debug_assertions)]
#[derive(PartialEq, Eq, Clone, Copy)]
#[cfg_attr(feature = "servo", derive(Serialize))]
struct DebugWritingMode {
    mode: WritingMode
}

#[cfg(not(debug_assertions))]
impl DebugWritingMode {
    #[inline]
    fn check(&self, _other: WritingMode) {}

    #[inline]
    fn check_debug(&self, _other: DebugWritingMode) {}

    #[inline]
    fn new(_mode: WritingMode) -> DebugWritingMode {
        DebugWritingMode
    }
}

#[cfg(debug_assertions)]
impl DebugWritingMode {
    #[inline]
    fn check(&self, other: WritingMode) {
        assert!(self.mode == other)
    }

    #[inline]
    fn check_debug(&self, other: DebugWritingMode) {
        assert!(self.mode == other.mode)
    }

    #[inline]
    fn new(mode: WritingMode) -> DebugWritingMode {
        DebugWritingMode { mode: mode }
    }
}

impl Debug for DebugWritingMode {
    #[cfg(not(debug_assertions))]
    fn fmt(&self, formatter: &mut Formatter) -> Result<(), Error> {
        write!(formatter, "?")
    }

    #[cfg(debug_assertions)]
    fn fmt(&self, formatter: &mut Formatter) -> Result<(), Error> {
        write!(formatter, "{}", self.mode)
    }
}


// Used to specify the logical direction.
#[derive(Debug, Clone, Copy, PartialEq)]
#[cfg_attr(feature = "servo", derive(Serialize))]
pub enum Direction {
    Inline,
    Block
}

/// A 2D size in flow-relative dimensions
#[derive(PartialEq, Eq, Clone, Copy)]
#[cfg_attr(feature = "servo", derive(Serialize))]
pub struct LogicalSize<T> {
    pub inline: T,  // inline-size, a.k.a. logical width, a.k.a. measure
    pub block: T,  // block-size, a.k.a. logical height, a.k.a. extent
    debug_writing_mode: DebugWritingMode,
}

impl<T: Debug> Debug for LogicalSize<T> {
    fn fmt(&self, formatter: &mut Formatter) -> Result<(), Error> {
        write!(formatter, "LogicalSize({:?}, i{:?}×b{:?})",
               self.debug_writing_mode, self.inline, self.block)
    }
}

// Can not implement the Zero trait: its zero() method does not have the `mode` parameter.
impl<T: Zero> LogicalSize<T> {
    #[inline]
    pub fn zero(mode: WritingMode) -> LogicalSize<T> {
        LogicalSize {
            inline: Zero::zero(),
            block: Zero::zero(),
            debug_writing_mode: DebugWritingMode::new(mode),
        }
    }
}

impl<T: Copy> LogicalSize<T> {
    #[inline]
    pub fn new(mode: WritingMode, inline: T, block: T) -> LogicalSize<T> {
        LogicalSize {
            inline: inline,
            block: block,
            debug_writing_mode: DebugWritingMode::new(mode),
        }
    }

    #[inline]
    pub fn from_physical(mode: WritingMode, size: Size2D<T>) -> LogicalSize<T> {
        if mode.is_vertical() {
            LogicalSize::new(mode, size.height, size.width)
        } else {
            LogicalSize::new(mode, size.width, size.height)
        }
    }

    #[inline]
    pub fn width(&self, mode: WritingMode) -> T {
        self.debug_writing_mode.check(mode);
        if mode.is_vertical() {
            self.block
        } else {
            self.inline
        }
    }

    #[inline]
    pub fn set_width(&mut self, mode: WritingMode, width: T) {
        self.debug_writing_mode.check(mode);
        if mode.is_vertical() {
            self.block = width
        } else {
            self.inline = width
        }
    }

    #[inline]
    pub fn height(&self, mode: WritingMode) -> T {
        self.debug_writing_mode.check(mode);
        if mode.is_vertical() {
            self.inline
        } else {
            self.block
        }
    }

    #[inline]
    pub fn set_height(&mut self, mode: WritingMode, height: T) {
        self.debug_writing_mode.check(mode);
        if mode.is_vertical() {
            self.inline = height
        } else {
            self.block = height
        }
    }

    #[inline]
    pub fn to_physical(&self, mode: WritingMode) -> Size2D<T> {
        self.debug_writing_mode.check(mode);
        if mode.is_vertical() {
            Size2D::new(self.block, self.inline)
        } else {
            Size2D::new(self.inline, self.block)
        }
    }

    #[inline]
    pub fn convert(&self, mode_from: WritingMode, mode_to: WritingMode) -> LogicalSize<T> {
        if mode_from == mode_to {
            self.debug_writing_mode.check(mode_from);
            *self
        } else {
            LogicalSize::from_physical(mode_to, self.to_physical(mode_from))
        }
    }
}

impl<T: Add<T, Output=T>> Add for LogicalSize<T> {
    type Output = LogicalSize<T>;

    #[inline]
    fn add(self, other: LogicalSize<T>) -> LogicalSize<T> {
        self.debug_writing_mode.check_debug(other.debug_writing_mode);
        LogicalSize {
            debug_writing_mode: self.debug_writing_mode,
            inline: self.inline + other.inline,
            block: self.block + other.block,
        }
    }
}

impl<T: Sub<T, Output=T>> Sub for LogicalSize<T> {
    type Output = LogicalSize<T>;

    #[inline]
    fn sub(self, other: LogicalSize<T>) -> LogicalSize<T> {
        self.debug_writing_mode.check_debug(other.debug_writing_mode);
        LogicalSize {
            debug_writing_mode: self.debug_writing_mode,
            inline: self.inline - other.inline,
            block: self.block - other.block,
        }
    }
}


/// A 2D point in flow-relative dimensions
#[derive(PartialEq, Eq, Clone, Copy)]
#[cfg_attr(feature = "servo", derive(Serialize))]
pub struct LogicalPoint<T> {
    /// inline-axis coordinate
    pub i: T,
    /// block-axis coordinate
    pub b: T,
    debug_writing_mode: DebugWritingMode,
}

impl<T: Debug> Debug for LogicalPoint<T> {
    fn fmt(&self, formatter: &mut Formatter) -> Result<(), Error> {
        write!(formatter, "LogicalPoint({:?} (i{:?}, b{:?}))",
               self.debug_writing_mode, self.i, self.b)
    }
}

// Can not implement the Zero trait: its zero() method does not have the `mode` parameter.
impl<T: Zero> LogicalPoint<T> {
    #[inline]
    pub fn zero(mode: WritingMode) -> LogicalPoint<T> {
        LogicalPoint {
            i: Zero::zero(),
            b: Zero::zero(),
            debug_writing_mode: DebugWritingMode::new(mode),
        }
    }
}

impl<T: Copy> LogicalPoint<T> {
    #[inline]
    pub fn new(mode: WritingMode, i: T, b: T) -> LogicalPoint<T> {
        LogicalPoint {
            i: i,
            b: b,
            debug_writing_mode: DebugWritingMode::new(mode),
        }
    }
}

impl<T: Copy + Sub<T, Output=T>> LogicalPoint<T> {
    #[inline]
    pub fn from_physical(mode: WritingMode, point: Point2D<T>, container_size: Size2D<T>)
                         -> LogicalPoint<T> {
        if mode.is_vertical() {
            LogicalPoint {
                i: if mode.is_inline_tb() { point.y } else { container_size.height - point.y },
                b: if mode.is_vertical_lr() { point.x } else { container_size.width - point.x },
                debug_writing_mode: DebugWritingMode::new(mode),
            }
        } else {
            LogicalPoint {
                i: if mode.is_bidi_ltr() { point.x } else { container_size.width - point.x },
                b: point.y,
                debug_writing_mode: DebugWritingMode::new(mode),
            }
        }
    }

    #[inline]
    pub fn x(&self, mode: WritingMode, container_size: Size2D<T>) -> T {
        self.debug_writing_mode.check(mode);
        if mode.is_vertical() {
            if mode.is_vertical_lr() { self.b } else { container_size.width - self.b }
        } else {
            if mode.is_bidi_ltr() { self.i } else { container_size.width - self.i }
        }
    }

    #[inline]
    pub fn set_x(&mut self, mode: WritingMode, x: T, container_size: Size2D<T>) {
        self.debug_writing_mode.check(mode);
        if mode.is_vertical() {
            self.b = if mode.is_vertical_lr() { x } else { container_size.width - x }
        } else {
            self.i = if mode.is_bidi_ltr() { x } else { container_size.width - x }
        }
    }

    #[inline]
    pub fn y(&self, mode: WritingMode, container_size: Size2D<T>) -> T {
        self.debug_writing_mode.check(mode);
        if mode.is_vertical() {
            if mode.is_inline_tb() { self.i } else { container_size.height - self.i }
        } else {
            self.b
        }
    }

    #[inline]
    pub fn set_y(&mut self, mode: WritingMode, y: T, container_size: Size2D<T>) {
        self.debug_writing_mode.check(mode);
        if mode.is_vertical() {
            self.i = if mode.is_inline_tb() { y } else { container_size.height - y }
        } else {
            self.b = y
        }
    }

    #[inline]
    pub fn to_physical(&self, mode: WritingMode, container_size: Size2D<T>) -> Point2D<T> {
        self.debug_writing_mode.check(mode);
        if mode.is_vertical() {
            Point2D::new(
                if mode.is_vertical_lr() { self.b } else { container_size.width - self.b },
                if mode.is_inline_tb() { self.i } else { container_size.height - self.i })
        } else {
            Point2D::new(
                if mode.is_bidi_ltr() { self.i } else { container_size.width - self.i },
                self.b)
        }
    }

    #[inline]
    pub fn convert(&self, mode_from: WritingMode, mode_to: WritingMode, container_size: Size2D<T>)
                   -> LogicalPoint<T> {
        if mode_from == mode_to {
            self.debug_writing_mode.check(mode_from);
            *self
        } else {
            LogicalPoint::from_physical(
                mode_to, self.to_physical(mode_from, container_size), container_size)
        }
    }
}

impl<T: Copy + Add<T, Output=T>> LogicalPoint<T> {
    /// This doesn’t really makes sense,
    /// but happens when dealing with multiple origins.
    #[inline]
    pub fn add_point(&self, other: &LogicalPoint<T>) -> LogicalPoint<T> {
        self.debug_writing_mode.check_debug(other.debug_writing_mode);
        LogicalPoint {
            debug_writing_mode: self.debug_writing_mode,
            i: self.i + other.i,
            b: self.b + other.b,
        }
    }
}

impl<T: Copy + Add<T, Output=T>> Add<LogicalSize<T>> for LogicalPoint<T> {
    type Output = LogicalPoint<T>;

    #[inline]
    fn add(self, other: LogicalSize<T>) -> LogicalPoint<T> {
        self.debug_writing_mode.check_debug(other.debug_writing_mode);
        LogicalPoint {
            debug_writing_mode: self.debug_writing_mode,
            i: self.i + other.inline,
            b: self.b + other.block,
        }
    }
}

impl<T: Copy + Sub<T, Output=T>> Sub<LogicalSize<T>> for LogicalPoint<T> {
    type Output = LogicalPoint<T>;

    #[inline]
    fn sub(self, other: LogicalSize<T>) -> LogicalPoint<T> {
        self.debug_writing_mode.check_debug(other.debug_writing_mode);
        LogicalPoint {
            debug_writing_mode: self.debug_writing_mode,
            i: self.i - other.inline,
            b: self.b - other.block,
        }
    }
}


/// A "margin" in flow-relative dimensions
/// Represents the four sides of the margins, borders, or padding of a CSS box,
/// or a combination of those.
/// A positive "margin" can be added to a rectangle to obtain a bigger rectangle.
#[derive(PartialEq, Eq, Clone, Copy)]
#[cfg_attr(feature = "servo", derive(Serialize))]
pub struct LogicalMargin<T> {
    pub block_start: T,
    pub inline_end: T,
    pub block_end: T,
    pub inline_start: T,
    debug_writing_mode: DebugWritingMode,
}

impl<T: Debug> Debug for LogicalMargin<T> {
    fn fmt(&self, formatter: &mut Formatter) -> Result<(), Error> {
        let writing_mode_string = if cfg!(debug_assertions) {
            format!("{:?}, ", self.debug_writing_mode)
        } else {
            "".to_owned()
        };

        write!(formatter, "LogicalMargin({}i:{:?}..{:?} b:{:?}..{:?})",
            writing_mode_string,
            self.inline_start,
            self.inline_end,
            self.block_start,
            self.block_end)
    }
}

impl<T: Zero> LogicalMargin<T> {
    #[inline]
    pub fn zero(mode: WritingMode) -> LogicalMargin<T> {
        LogicalMargin {
            block_start: Zero::zero(),
            inline_end: Zero::zero(),
            block_end: Zero::zero(),
            inline_start: Zero::zero(),
            debug_writing_mode: DebugWritingMode::new(mode),
        }
    }
}

impl<T: Copy> LogicalMargin<T> {
    #[inline]
    pub fn new(mode: WritingMode, block_start: T, inline_end: T, block_end: T, inline_start: T)
               -> LogicalMargin<T> {
        LogicalMargin {
            block_start: block_start,
            inline_end: inline_end,
            block_end: block_end,
            inline_start: inline_start,
            debug_writing_mode: DebugWritingMode::new(mode),
        }
    }

    #[inline]
    pub fn new_all_same(mode: WritingMode, value: T) -> LogicalMargin<T> {
        LogicalMargin::new(mode, value, value, value, value)
    }

    #[inline]
    pub fn from_physical(mode: WritingMode, offsets: SideOffsets2D<T>) -> LogicalMargin<T> {
        let block_start;
        let inline_end;
        let block_end;
        let inline_start;
        if mode.is_vertical() {
            if mode.is_vertical_lr() {
                block_start = offsets.left;
                block_end = offsets.right;
            } else {
                block_start = offsets.right;
                block_end = offsets.left;
            }
            if mode.is_inline_tb() {
                inline_start = offsets.top;
                inline_end = offsets.bottom;
            } else {
                inline_start = offsets.bottom;
                inline_end = offsets.top;
            }
        } else {
            block_start = offsets.top;
            block_end = offsets.bottom;
            if mode.is_bidi_ltr() {
                inline_start = offsets.left;
                inline_end = offsets.right;
            } else {
                inline_start = offsets.right;
                inline_end = offsets.left;
            }
        }
        LogicalMargin::new(mode, block_start, inline_end, block_end, inline_start)
    }

    #[inline]
    pub fn top(&self, mode: WritingMode) -> T {
        self.debug_writing_mode.check(mode);
        if mode.is_vertical() {
            if mode.is_inline_tb() { self.inline_start } else { self.inline_end }
        } else {
            self.block_start
        }
    }

    #[inline]
    pub fn set_top(&mut self, mode: WritingMode, top: T) {
        self.debug_writing_mode.check(mode);
        if mode.is_vertical() {
            if mode.is_inline_tb() { self.inline_start = top } else { self.inline_end = top }
        } else {
            self.block_start = top
        }
    }

    #[inline]
    pub fn right(&self, mode: WritingMode) -> T {
        self.debug_writing_mode.check(mode);
        if mode.is_vertical() {
            if mode.is_vertical_lr() { self.block_end } else { self.block_start }
        } else {
            if mode.is_bidi_ltr() { self.inline_end } else { self.inline_start }
        }
    }

    #[inline]
    pub fn set_right(&mut self, mode: WritingMode, right: T) {
        self.debug_writing_mode.check(mode);
        if mode.is_vertical() {
            if mode.is_vertical_lr() { self.block_end = right } else { self.block_start = right }
        } else {
            if mode.is_bidi_ltr() { self.inline_end = right } else { self.inline_start = right }
        }
    }

    #[inline]
    pub fn bottom(&self, mode: WritingMode) -> T {
        self.debug_writing_mode.check(mode);
        if mode.is_vertical() {
            if mode.is_inline_tb() { self.inline_end } else { self.inline_start }
        } else {
            self.block_end
        }
    }

    #[inline]
    pub fn set_bottom(&mut self, mode: WritingMode, bottom: T) {
        self.debug_writing_mode.check(mode);
        if mode.is_vertical() {
            if mode.is_inline_tb() { self.inline_end = bottom } else { self.inline_start = bottom }
        } else {
            self.block_end = bottom
        }
    }

    #[inline]
    pub fn left(&self, mode: WritingMode) -> T {
        self.debug_writing_mode.check(mode);
        if mode.is_vertical() {
            if mode.is_vertical_lr() { self.block_start } else { self.block_end }
        } else {
            if mode.is_bidi_ltr() { self.inline_start } else { self.inline_end }
        }
    }

    #[inline]
    pub fn set_left(&mut self, mode: WritingMode, left: T) {
        self.debug_writing_mode.check(mode);
        if mode.is_vertical() {
            if mode.is_vertical_lr() { self.block_start = left } else { self.block_end = left }
        } else {
            if mode.is_bidi_ltr() { self.inline_start = left } else { self.inline_end = left }
        }
    }

    #[inline]
    pub fn to_physical(&self, mode: WritingMode) -> SideOffsets2D<T> {
        self.debug_writing_mode.check(mode);
        let top;
        let right;
        let bottom;
        let left;
        if mode.is_vertical() {
            if mode.is_vertical_lr() {
                left = self.block_start;
                right = self.block_end;
            } else {
                right = self.block_start;
                left = self.block_end;
            }
            if mode.is_inline_tb() {
                top = self.inline_start;
                bottom = self.inline_end;
            } else {
                bottom = self.inline_start;
                top = self.inline_end;
            }
        } else {
            top = self.block_start;
            bottom = self.block_end;
            if mode.is_bidi_ltr() {
                left = self.inline_start;
                right = self.inline_end;
            } else {
                right = self.inline_start;
                left = self.inline_end;
            }
        }
        SideOffsets2D::new(top, right, bottom, left)
    }

    #[inline]
    pub fn convert(&self, mode_from: WritingMode, mode_to: WritingMode) -> LogicalMargin<T> {
        if mode_from == mode_to {
            self.debug_writing_mode.check(mode_from);
            *self
        } else {
            LogicalMargin::from_physical(mode_to, self.to_physical(mode_from))
        }
    }
}

impl<T: PartialEq + Zero> LogicalMargin<T> {
    #[inline]
    pub fn is_zero(&self) -> bool {
        self.block_start == Zero::zero() && self.inline_end == Zero::zero() &&
        self.block_end == Zero::zero() && self.inline_start == Zero::zero()
    }
}

impl<T: Copy + Add<T, Output=T>> LogicalMargin<T> {
    #[inline]
    pub fn inline_start_end(&self) -> T {
        self.inline_start + self.inline_end
    }

    #[inline]
    pub fn block_start_end(&self) -> T {
        self.block_start + self.block_end
    }

    #[inline]
    pub fn start_end(&self, direction: Direction) -> T {
        match direction {
            Direction::Inline =>
                self.inline_start + self.inline_end,
            Direction::Block =>
                self.block_start + self.block_end
        }
    }

    #[inline]
    pub fn top_bottom(&self, mode: WritingMode) -> T {
        self.debug_writing_mode.check(mode);
        if mode.is_vertical() {
            self.inline_start_end()
        } else {
            self.block_start_end()
        }
    }

    #[inline]
    pub fn left_right(&self, mode: WritingMode) -> T {
        self.debug_writing_mode.check(mode);
        if mode.is_vertical() {
            self.block_start_end()
        } else {
            self.inline_start_end()
        }
    }
}

impl<T: Add<T, Output=T>> Add for LogicalMargin<T> {
    type Output = LogicalMargin<T>;

    #[inline]
    fn add(self, other: LogicalMargin<T>) -> LogicalMargin<T> {
        self.debug_writing_mode.check_debug(other.debug_writing_mode);
        LogicalMargin {
            debug_writing_mode: self.debug_writing_mode,
            block_start: self.block_start + other.block_start,
            inline_end: self.inline_end + other.inline_end,
            block_end: self.block_end + other.block_end,
            inline_start: self.inline_start + other.inline_start,
        }
    }
}

impl<T: Sub<T, Output=T>> Sub for LogicalMargin<T> {
    type Output = LogicalMargin<T>;

    #[inline]
    fn sub(self, other: LogicalMargin<T>) -> LogicalMargin<T> {
        self.debug_writing_mode.check_debug(other.debug_writing_mode);
        LogicalMargin {
            debug_writing_mode: self.debug_writing_mode,
            block_start: self.block_start - other.block_start,
            inline_end: self.inline_end - other.inline_end,
            block_end: self.block_end - other.block_end,
            inline_start: self.inline_start - other.inline_start,
        }
    }
}


/// A rectangle in flow-relative dimensions
#[derive(PartialEq, Eq, Clone, Copy)]
#[cfg_attr(feature = "servo", derive(Serialize))]
pub struct LogicalRect<T> {
    pub start: LogicalPoint<T>,
    pub size: LogicalSize<T>,
    debug_writing_mode: DebugWritingMode,
}

impl<T: Debug> Debug for LogicalRect<T> {
    fn fmt(&self, formatter: &mut Formatter) -> Result<(), Error> {
        let writing_mode_string = if cfg!(debug_assertions) {
            format!("{:?}, ", self.debug_writing_mode)
        } else {
            "".to_owned()
        };

        write!(formatter, "LogicalRect({}i{:?}×b{:?}, @ (i{:?},b{:?}))",
               writing_mode_string,
               self.size.inline,
               self.size.block,
               self.start.i,
               self.start.b)
    }
}

impl<T: Zero> LogicalRect<T> {
    #[inline]
    pub fn zero(mode: WritingMode) -> LogicalRect<T> {
        LogicalRect {
            start: LogicalPoint::zero(mode),
            size: LogicalSize::zero(mode),
            debug_writing_mode: DebugWritingMode::new(mode),
        }
    }
}

impl<T: Copy> LogicalRect<T> {
    #[inline]
    pub fn new(mode: WritingMode, inline_start: T, block_start: T, inline: T, block: T)
               -> LogicalRect<T> {
        LogicalRect {
            start: LogicalPoint::new(mode, inline_start, block_start),
            size: LogicalSize::new(mode, inline, block),
            debug_writing_mode: DebugWritingMode::new(mode),
        }
    }

    #[inline]
    pub fn from_point_size(mode: WritingMode, start: LogicalPoint<T>, size: LogicalSize<T>)
                           -> LogicalRect<T> {
        start.debug_writing_mode.check(mode);
        size.debug_writing_mode.check(mode);
        LogicalRect {
            start: start,
            size: size,
            debug_writing_mode: DebugWritingMode::new(mode),
        }
    }
}

impl<T: Copy + Add<T, Output=T> + Sub<T, Output=T>> LogicalRect<T> {
    #[inline]
    pub fn from_physical(mode: WritingMode, rect: Rect<T>, container_size: Size2D<T>)
                         -> LogicalRect<T> {
        let inline_start;
        let block_start;
        let inline;
        let block;
        if mode.is_vertical() {
            inline = rect.size.height;
            block = rect.size.width;
            if mode.is_vertical_lr() {
                block_start = rect.origin.x;
            } else {
                block_start = container_size.width - (rect.origin.x + rect.size.width);
            }
            if mode.is_inline_tb() {
                inline_start = rect.origin.y;
            } else {
                inline_start = container_size.height - (rect.origin.y + rect.size.height);
            }
        } else {
            inline = rect.size.width;
            block = rect.size.height;
            block_start = rect.origin.y;
            if mode.is_bidi_ltr() {
                inline_start = rect.origin.x;
            } else {
                inline_start = container_size.width - (rect.origin.x + rect.size.width);
            }
        }
        LogicalRect {
            start: LogicalPoint::new(mode, inline_start, block_start),
            size: LogicalSize::new(mode, inline, block),
            debug_writing_mode: DebugWritingMode::new(mode),
        }
    }

    #[inline]
    pub fn inline_end(&self) -> T {
        self.start.i + self.size.inline
    }

    #[inline]
    pub fn block_end(&self) -> T {
        self.start.b + self.size.block
    }

    #[inline]
    pub fn to_physical(&self, mode: WritingMode, container_size: Size2D<T>) -> Rect<T> {
        self.debug_writing_mode.check(mode);
        let x;
        let y;
        let width;
        let height;
        if mode.is_vertical() {
            width = self.size.block;
            height = self.size.inline;
            if mode.is_vertical_lr() {
                x = self.start.b;
            } else {
                x = container_size.width - self.block_end();
            }
            if mode.is_inline_tb() {
                y = self.start.i;
            } else {
                y = container_size.height - self.inline_end();
            }
        } else {
            width = self.size.inline;
            height = self.size.block;
            y = self.start.b;
            if mode.is_bidi_ltr() {
                x = self.start.i;
            } else {
                x = container_size.width - self.inline_end();
            }
        }
        Rect {
            origin: Point2D::new(x, y),
            size: Size2D::new(width, height),
        }
    }

    #[inline]
    pub fn convert(&self, mode_from: WritingMode, mode_to: WritingMode, container_size: Size2D<T>)
                   -> LogicalRect<T> {
        if mode_from == mode_to {
            self.debug_writing_mode.check(mode_from);
            *self
        } else {
            LogicalRect::from_physical(
                mode_to, self.to_physical(mode_from, container_size), container_size)
        }
    }

    pub fn translate_by_size(&self, offset: LogicalSize<T>) -> LogicalRect<T> {
        LogicalRect {
            start: self.start + offset,
            ..*self
        }
    }

    pub fn translate(&self, offset: &LogicalPoint<T>) -> LogicalRect<T> {
        LogicalRect {
            start: self.start + LogicalSize {
                inline: offset.i,
                block: offset.b,
                debug_writing_mode: offset.debug_writing_mode,
            },
            size: self.size,
            debug_writing_mode: self.debug_writing_mode,
        }
    }
}

impl<T: Copy + Ord + Add<T, Output=T> + Sub<T, Output=T>> LogicalRect<T> {
    #[inline]
    pub fn union(&self, other: &LogicalRect<T>) -> LogicalRect<T> {
        self.debug_writing_mode.check_debug(other.debug_writing_mode);

        let inline_start = min(self.start.i, other.start.i);
        let block_start = min(self.start.b, other.start.b);
        LogicalRect {
            start: LogicalPoint {
                i: inline_start,
                b: block_start,
                debug_writing_mode: self.debug_writing_mode,
            },
            size: LogicalSize {
                inline: max(self.inline_end(), other.inline_end()) - inline_start,
                block: max(self.block_end(), other.block_end()) - block_start,
                debug_writing_mode: self.debug_writing_mode,
            },
            debug_writing_mode: self.debug_writing_mode,
        }
    }
}

impl<T: Copy + Add<T, Output=T> + Sub<T, Output=T>> Add<LogicalMargin<T>> for LogicalRect<T> {
    type Output = LogicalRect<T>;

    #[inline]
    fn add(self, other: LogicalMargin<T>) -> LogicalRect<T> {
        self.debug_writing_mode.check_debug(other.debug_writing_mode);
        LogicalRect {
            start: LogicalPoint {
                // Growing a rectangle on the start side means pushing its
                // start point on the negative direction.
                i: self.start.i - other.inline_start,
                b: self.start.b - other.block_start,
                debug_writing_mode: self.debug_writing_mode,
            },
            size: LogicalSize {
                inline: self.size.inline + other.inline_start_end(),
                block: self.size.block + other.block_start_end(),
                debug_writing_mode: self.debug_writing_mode,
            },
            debug_writing_mode: self.debug_writing_mode,
        }
    }
}


impl<T: Copy + Add<T, Output=T> + Sub<T, Output=T>> Sub<LogicalMargin<T>> for LogicalRect<T> {
    type Output = LogicalRect<T>;

    #[inline]
    fn sub(self, other: LogicalMargin<T>) -> LogicalRect<T> {
        self.debug_writing_mode.check_debug(other.debug_writing_mode);
        LogicalRect {
            start: LogicalPoint {
                // Shrinking a rectangle on the start side means pushing its
                // start point on the positive direction.
                i: self.start.i + other.inline_start,
                b: self.start.b + other.block_start,
                debug_writing_mode: self.debug_writing_mode,
            },
            size: LogicalSize {
                inline: self.size.inline - other.inline_start_end(),
                block: self.size.block - other.block_start_end(),
                debug_writing_mode: self.debug_writing_mode,
            },
            debug_writing_mode: self.debug_writing_mode,
        }
    }
}

#[derive(Copy, Clone, PartialEq, Debug)]
pub enum PhysicalSide {
    Top,
    Right,
    Bottom,
    Left,
}
