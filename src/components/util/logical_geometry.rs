/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

/// Geometry in flow-relative space.

use geom::{Size2D, Point2D, SideOffsets2D, Rect};
use std::fmt::{Show, Formatter, FormatError};
use std::num::Zero;


bitflags!(
    flags WritingMode: u8 {
        static FlagRTL = 1 << 0,
        static FlagVertical = 1 << 1,
        static FlagVerticalLR = 1 << 2,
        static FlagSidewaysLeft = 1 << 3
    }
)

impl WritingMode {
    #[inline]
    pub fn is_vertical(&self) -> bool {
        self.intersects(FlagVertical)
    }

    /// Asuming .is_vertical(), does the block direction go left to right?
    #[inline]
    pub fn is_vertical_lr(&self) -> bool {
        self.intersects(FlagVerticalLR)
    }

    /// Asuming .is_vertical(), does the inline direction go top to bottom?
    #[inline]
    pub fn is_inline_tb(&self) -> bool {
        !(self.intersects(FlagSidewaysLeft) ^ self.intersects(FlagRTL))
    }

    #[inline]
    pub fn is_bidi_ltr(&self) -> bool {
        !self.intersects(FlagRTL)
    }
}

impl Show for WritingMode {
    fn fmt(&self, formatter: &mut Formatter) -> Result<(), FormatError> {
        if self.is_vertical() {
            try!(write!(formatter, "V"));
            if self.is_vertical_lr() {
                try!(write!(formatter, " LR"));
            } else {
                try!(write!(formatter, " RL"));
            }
            if self.intersects(FlagSidewaysLeft) {
                try!(write!(formatter, " SidewaysL"));
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
#[cfg(ndebug)]
#[deriving(PartialEq, Eq, Clone)]
struct DebugWritingMode;

#[cfg(not(ndebug))]
#[deriving(PartialEq, Eq, Clone)]
struct DebugWritingMode {
    mode: WritingMode
}

#[cfg(ndebug)]
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

#[cfg(not(ndebug))]
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

impl Show for DebugWritingMode {
    #[cfg(ndebug)]
    fn fmt(&self, formatter: &mut Formatter) -> Result<(), FormatError> {
        write!(formatter, "?")
    }

    #[cfg(not(ndebug))]
    fn fmt(&self, formatter: &mut Formatter) -> Result<(), FormatError> {
        self.mode.fmt(formatter)
    }
}


/// A 2D size in flow-relative dimensions
#[deriving(PartialEq, Eq, Clone)]
pub struct LogicalSize<T> {
    pub isize: T,  // inline-size (a.k.a. logical width)
    pub bsize: T,  // block-size (a.k.a. logical height)
    debug_writing_mode: DebugWritingMode,
}

impl<T: Show> Show for LogicalSize<T> {
    fn fmt(&self, formatter: &mut Formatter) -> Result<(), FormatError> {
        write!(formatter, "LogicalSize[{}, {}, {}]",
               self.debug_writing_mode, self.isize, self.bsize)
    }
}

// Can not implement the Zero trait: its zero() method does not have the `mode` parameter.
impl<T: Zero> LogicalSize<T> {
    #[inline]
    pub fn zero(mode: WritingMode) -> LogicalSize<T> {
        LogicalSize {
            isize: Zero::zero(),
            bsize: Zero::zero(),
            debug_writing_mode: DebugWritingMode::new(mode),
        }
    }

    #[inline]
    pub fn is_zero(&self) -> bool {
        self.isize.is_zero() && self.bsize.is_zero()
    }
}

impl<T: Copy> LogicalSize<T> {
    #[inline]
    pub fn new(mode: WritingMode, isize: T, bsize: T) -> LogicalSize<T> {
        LogicalSize {
            isize: isize,
            bsize: bsize,
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
            self.bsize
        } else {
            self.isize
        }
    }

    #[inline]
    pub fn set_width(&mut self, mode: WritingMode, width: T) {
        self.debug_writing_mode.check(mode);
        if mode.is_vertical() {
            self.bsize = width
        } else {
            self.isize = width
        }
    }

    #[inline]
    pub fn height(&self, mode: WritingMode) -> T {
        self.debug_writing_mode.check(mode);
        if mode.is_vertical() {
            self.isize
        } else {
            self.bsize
        }
    }

    #[inline]
    pub fn set_height(&mut self, mode: WritingMode, height: T) {
        self.debug_writing_mode.check(mode);
        if mode.is_vertical() {
            self.isize = height
        } else {
            self.bsize = height
        }
    }

    #[inline]
    pub fn to_physical(&self, mode: WritingMode) -> Size2D<T> {
        self.debug_writing_mode.check(mode);
        if mode.is_vertical() {
            Size2D { width: self.bsize, height: self.isize }
        } else {
            Size2D { width: self.isize, height: self.bsize }
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

impl<T: Add<T, T>> Add<LogicalSize<T>, LogicalSize<T>> for LogicalSize<T> {
    #[inline]
    fn add(&self, other: &LogicalSize<T>) -> LogicalSize<T> {
        self.debug_writing_mode.check_debug(other.debug_writing_mode);
        LogicalSize {
            debug_writing_mode: self.debug_writing_mode,
            isize: self.isize + other.isize,
            bsize: self.bsize + other.bsize,
        }
    }
}

impl<T: Sub<T, T>> Sub<LogicalSize<T>, LogicalSize<T>> for LogicalSize<T> {
    #[inline]
    fn sub(&self, other: &LogicalSize<T>) -> LogicalSize<T> {
        self.debug_writing_mode.check_debug(other.debug_writing_mode);
        LogicalSize {
            debug_writing_mode: self.debug_writing_mode,
            isize: self.isize - other.isize,
            bsize: self.bsize - other.bsize,
        }
    }
}


/// A 2D point in flow-relative dimensions
#[deriving(PartialEq, Eq, Clone)]
pub struct LogicalPoint<T> {
    pub i: T,  /// inline-axis coordinate
    pub b: T,  /// block-axis coordinate
    debug_writing_mode: DebugWritingMode,
}

impl<T: Show> Show for LogicalPoint<T> {
    fn fmt(&self, formatter: &mut Formatter) -> Result<(), FormatError> {
        write!(formatter, "LogicalPoint[{}, {}, {}]",
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

    #[inline]
    pub fn is_zero(&self) -> bool {
        self.i.is_zero() && self.b.is_zero()
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

impl<T: Copy + Sub<T, T>> LogicalPoint<T> {
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
            Point2D {
                x: if mode.is_vertical_lr() { self.b } else { container_size.width - self.b },
                y: if mode.is_inline_tb() { self.i } else { container_size.height - self.i }
            }
        } else {
            Point2D {
                x: if mode.is_bidi_ltr() { self.i } else { container_size.width - self.i },
                y: self.b
            }
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

impl<T: Add<T,T>> Add<LogicalSize<T>, LogicalPoint<T>> for LogicalPoint<T> {
    #[inline]
    fn add(&self, other: &LogicalSize<T>) -> LogicalPoint<T> {
        self.debug_writing_mode.check_debug(other.debug_writing_mode);
        LogicalPoint {
            debug_writing_mode: self.debug_writing_mode,
            i: self.i + other.isize,
            b: self.b + other.bsize,
        }
    }
}

impl<T: Sub<T,T>> Sub<LogicalSize<T>, LogicalPoint<T>> for LogicalPoint<T> {
    #[inline]
    fn sub(&self, other: &LogicalSize<T>) -> LogicalPoint<T> {
        self.debug_writing_mode.check_debug(other.debug_writing_mode);
        LogicalPoint {
            debug_writing_mode: self.debug_writing_mode,
            i: self.i - other.isize,
            b: self.b - other.bsize,
        }
    }
}


/// A "margin" in flow-relative dimensions
/// Represents the four sides of the margins, borders, or padding of a CSS box,
/// or a combination of those.
/// A positive "margin" can be added to a rectangle to obtain a bigger rectangle.
#[deriving(PartialEq, Eq, Clone)]
pub struct LogicalMargin<T> {
    pub bstart: T,
    pub iend: T,
    pub bend: T,
    pub istart: T,
    debug_writing_mode: DebugWritingMode,
}

impl<T: Show> Show for LogicalMargin<T> {
    fn fmt(&self, formatter: &mut Formatter) -> Result<(), FormatError> {
        write!(formatter, "LogicalMargin[{}, bstart: {}, iend: {}, bend: {}, istart: {}]",
               self.debug_writing_mode, self.bstart, self.iend, self.bend, self.istart)
    }
}

impl<T: Zero> LogicalMargin<T> {
    #[inline]
    pub fn zero(mode: WritingMode) -> LogicalMargin<T> {
        LogicalMargin {
            bstart: Zero::zero(),
            iend: Zero::zero(),
            bend: Zero::zero(),
            istart: Zero::zero(),
            debug_writing_mode: DebugWritingMode::new(mode),
        }
    }

    #[inline]
    pub fn is_zero(&self) -> bool {
        self.bstart.is_zero() &&
        self.iend.is_zero() &&
        self.bend.is_zero() &&
        self.istart.is_zero()
    }
}

impl<T: Copy> LogicalMargin<T> {
    #[inline]
    pub fn new(mode: WritingMode, bstart: T, iend: T, bend: T, istart: T) -> LogicalMargin<T> {
        LogicalMargin {
            bstart: bstart,
            iend: iend,
            bend: bend,
            istart: istart,
            debug_writing_mode: DebugWritingMode::new(mode),
        }
    }

    #[inline]
    pub fn from_physical(mode: WritingMode, offsets: SideOffsets2D<T>) -> LogicalMargin<T> {
        let bstart;
        let iend;
        let bend;
        let istart;
        if mode.is_vertical() {
            if mode.is_vertical_lr() {
                bstart = offsets.left;
                bend = offsets.right;
            } else {
                bstart = offsets.right;
                bend = offsets.left;
            }
            if mode.is_inline_tb() {
                istart = offsets.top;
                iend = offsets.bottom;
            } else {
                istart = offsets.bottom;
                iend = offsets.top;
            }
        } else {
            bstart = offsets.top;
            bend = offsets.bottom;
            if mode.is_bidi_ltr() {
                istart = offsets.left;
                iend = offsets.right;
            } else {
                istart = offsets.right;
                iend = offsets.left;
            }
        }
        LogicalMargin::new(mode, bstart, iend, bend, istart)
    }

    #[inline]
    pub fn top(&self, mode: WritingMode) -> T {
        self.debug_writing_mode.check(mode);
        if mode.is_vertical() {
            if mode.is_inline_tb() { self.istart } else { self.iend }
        } else {
            self.bstart
        }
    }

    #[inline]
    pub fn set_top(&mut self, mode: WritingMode, top: T) {
        self.debug_writing_mode.check(mode);
        if mode.is_vertical() {
            if mode.is_inline_tb() { self.istart = top } else { self.iend = top }
        } else {
            self.bstart = top
        }
    }

    #[inline]
    pub fn right(&self, mode: WritingMode) -> T {
        self.debug_writing_mode.check(mode);
        if mode.is_vertical() {
            if mode.is_vertical_lr() { self.bend } else { self.bstart }
        } else {
            if mode.is_bidi_ltr() { self.iend } else { self.istart }
        }
    }

    #[inline]
    pub fn set_right(&mut self, mode: WritingMode, right: T) {
        self.debug_writing_mode.check(mode);
        if mode.is_vertical() {
            if mode.is_vertical_lr() { self.bend = right } else { self.bstart = right }
        } else {
            if mode.is_bidi_ltr() { self.iend = right } else { self.istart = right }
        }
    }

    #[inline]
    pub fn bottom(&self, mode: WritingMode) -> T {
        self.debug_writing_mode.check(mode);
        if mode.is_vertical() {
            if mode.is_inline_tb() { self.iend } else { self.istart }
        } else {
            self.bend
        }
    }

    #[inline]
    pub fn set_bottom(&mut self, mode: WritingMode, bottom: T) {
        self.debug_writing_mode.check(mode);
        if mode.is_vertical() {
            if mode.is_inline_tb() { self.iend = bottom } else { self.istart = bottom }
        } else {
            self.bend = bottom
        }
    }

    #[inline]
    pub fn left(&self, mode: WritingMode) -> T {
        self.debug_writing_mode.check(mode);
        if mode.is_vertical() {
            if mode.is_vertical_lr() { self.bstart } else { self.bend }
        } else {
            if mode.is_bidi_ltr() { self.istart } else { self.iend }
        }
    }

    #[inline]
    pub fn set_left(&mut self, mode: WritingMode, left: T) {
        self.debug_writing_mode.check(mode);
        if mode.is_vertical() {
            if mode.is_vertical_lr() { self.bstart = left } else { self.bend = left }
        } else {
            if mode.is_bidi_ltr() { self.istart = left } else { self.iend = left }
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
                left = self.bstart;
                right = self.bend;
            } else {
                right = self.bstart;
                left = self.bend;
            }
            if mode.is_inline_tb() {
                top = self.istart;
                bottom = self.iend;
            } else {
                bottom = self.istart;
                top = self.iend;
            }
        } else {
            top = self.bstart;
            bottom = self.bend;
            if mode.is_bidi_ltr() {
                left = self.istart;
                right = self.iend;
            } else {
                right = self.istart;
                left = self.iend;
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

impl<T: Add<T, T>> LogicalMargin<T> {
    #[inline]
    pub fn istart_end(&self) -> T {
        self.istart + self.iend
    }

    #[inline]
    pub fn bstart_end(&self) -> T {
        self.bstart + self.bend
    }

    #[inline]
    pub fn top_bottom(&self, mode: WritingMode) -> T {
        self.debug_writing_mode.check(mode);
        if mode.is_vertical() {
            self.istart_end()
        } else {
            self.bstart_end()
        }
    }

    #[inline]
    pub fn left_right(&self, mode: WritingMode) -> T {
        self.debug_writing_mode.check(mode);
        if mode.is_vertical() {
            self.bstart_end()
        } else {
            self.istart_end()
        }
    }
}

impl<T: Add<T, T>> Add<LogicalMargin<T>, LogicalMargin<T>> for LogicalMargin<T> {
    #[inline]
    fn add(&self, other: &LogicalMargin<T>) -> LogicalMargin<T> {
        self.debug_writing_mode.check_debug(other.debug_writing_mode);
        LogicalMargin {
            debug_writing_mode: self.debug_writing_mode,
            bstart: self.bstart + other.bstart,
            iend: self.iend + other.iend,
            bend: self.bend + other.bend,
            istart: self.istart + other.istart,
        }
    }
}

impl<T: Sub<T, T>> Sub<LogicalMargin<T>, LogicalMargin<T>> for LogicalMargin<T> {
    #[inline]
    fn sub(&self, other: &LogicalMargin<T>) -> LogicalMargin<T> {
        self.debug_writing_mode.check_debug(other.debug_writing_mode);
        LogicalMargin {
            debug_writing_mode: self.debug_writing_mode,
            bstart: self.bstart - other.bstart,
            iend: self.iend - other.iend,
            bend: self.bend - other.bend,
            istart: self.istart - other.istart,
        }
    }
}


/// A rectangle in flow-relative dimensions
#[deriving(PartialEq, Eq, Clone)]
pub struct LogicalRect<T> {
    pub start: LogicalPoint<T>,
    pub size: LogicalSize<T>,
    debug_writing_mode: DebugWritingMode,
}

impl<T: Show> Show for LogicalRect<T> {
    fn fmt(&self, formatter: &mut Formatter) -> Result<(), FormatError> {
        write!(formatter, "LogicalRect[{}, istart: {}, bstart: {}, isize: {}, bsize: {}]",
               self.debug_writing_mode, self.start.i, self.start.b,
               self.size.isize, self.size.bsize)
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

    #[inline]
    pub fn is_zero(&self) -> bool {
        self.start.is_zero() && self.size.is_zero()
    }
}

impl<T: Copy> LogicalRect<T> {
    #[inline]
    pub fn new(mode: WritingMode, istart: T, bstart: T, isize: T, bsize: T) -> LogicalRect<T> {
        LogicalRect {
            start: LogicalPoint::new(mode, istart, bstart),
            size: LogicalSize::new(mode, isize, bsize),
            debug_writing_mode: DebugWritingMode::new(mode),
        }
    }
}

impl<T: Copy + Add<T, T> + Sub<T, T>> LogicalRect<T> {
    #[inline]
    pub fn from_physical(mode: WritingMode, rect: Rect<T>, container_size: Size2D<T>)
                         -> LogicalRect<T> {
        let istart;
        let bstart;
        let isize;
        let bsize;
        if mode.is_vertical() {
            isize = rect.size.height;
            bsize = rect.size.width;
            if mode.is_vertical_lr() {
                bstart = rect.origin.x;
            } else {
                bstart = container_size.width - (rect.origin.x + rect.size.width);
            }
            if mode.is_inline_tb() {
                istart = rect.origin.y;
            } else {
                istart = container_size.height - (rect.origin.y + rect.size.height);
            }
        } else {
            isize = rect.size.width;
            bsize = rect.size.height;
            bstart = rect.origin.y;
            if mode.is_bidi_ltr() {
                istart = rect.origin.x;
            } else {
                istart = container_size.width - (rect.origin.x + rect.size.width);
            }
        }
        LogicalRect {
            start: LogicalPoint::new(mode, istart, bstart),
            size: LogicalSize::new(mode, isize, bsize),
            debug_writing_mode: DebugWritingMode::new(mode),
        }
    }

    #[inline]
    pub fn iend(&self) -> T {
        self.start.i + self.size.isize
    }

    #[inline]
    pub fn bend(&self) -> T {
        self.start.b + self.size.bsize
    }

    #[inline]
    pub fn to_physical(&self, mode: WritingMode, container_size: Size2D<T>) -> Rect<T> {
        self.debug_writing_mode.check(mode);
        let x;
        let y;
        let width;
        let height;
        if mode.is_vertical() {
            width = self.size.bsize;
            height = self.size.isize;
            if mode.is_vertical_lr() {
                x = self.start.b;
            } else {
                x = container_size.width - self.bend();
            }
            if mode.is_inline_tb() {
                y = self.start.i;
            } else {
                y = container_size.height - self.iend();
            }
        } else {
            width = self.size.isize;
            height = self.size.bsize;
            y = self.start.b;
            if mode.is_bidi_ltr() {
                x = self.start.i;
            } else {
                x = container_size.width - self.iend();
            }
        }
        Rect {
            origin: Point2D { x: x, y: y },
            size: Size2D { width: width, height: height },
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
}

impl<T: Add<T, T> + Sub<T, T>> Add<LogicalMargin<T>, LogicalRect<T>> for LogicalRect<T> {
    #[inline]
    fn add(&self, other: &LogicalMargin<T>) -> LogicalRect<T> {
        self.debug_writing_mode.check_debug(other.debug_writing_mode);
        LogicalRect {
            start: LogicalPoint {
                // Growing a rectangle on the start side means pushing its
                // start point on the negative direction.
                i: self.start.i - other.istart,
                b: self.start.b - other.bstart,
                debug_writing_mode: self.debug_writing_mode,
            },
            size: LogicalSize {
                isize: self.size.isize + other.istart_end(),
                bsize: self.size.bsize + other.bstart_end(),
                debug_writing_mode: self.debug_writing_mode,
            },
            debug_writing_mode: self.debug_writing_mode,
        }
    }
}


impl<T: Add<T, T> + Sub<T, T>> Sub<LogicalMargin<T>, LogicalRect<T>> for LogicalRect<T> {
    #[inline]
    fn sub(&self, other: &LogicalMargin<T>) -> LogicalRect<T> {
        self.debug_writing_mode.check_debug(other.debug_writing_mode);
        LogicalRect {
            start: LogicalPoint {
                // Shrinking a rectangle on the start side means pushing its
                // start point on the positive direction.
                i: self.start.i + other.istart,
                b: self.start.b + other.bstart,
                debug_writing_mode: self.debug_writing_mode,
            },
            size: LogicalSize {
                isize: self.size.isize - other.istart_end(),
                bsize: self.size.bsize - other.bstart_end(),
                debug_writing_mode: self.debug_writing_mode,
            },
            debug_writing_mode: self.debug_writing_mode,
        }
    }
}

#[cfg(test)]
fn modes() -> [WritingMode, ..10] {
    [
        WritingMode::empty(),
        FlagVertical,
        FlagVertical | FlagVerticalLR,
        FlagVertical | FlagVerticalLR | FlagSidewaysLeft,
        FlagVertical | FlagSidewaysLeft,
        FlagRTL,
        FlagVertical | FlagRTL,
        FlagVertical | FlagVerticalLR | FlagRTL,
        FlagVertical | FlagVerticalLR | FlagSidewaysLeft | FlagRTL,
        FlagVertical | FlagSidewaysLeft | FlagRTL,
    ]
}

#[test]
fn test_size_round_trip() {
    let physical = Size2D(1, 2);
    for &mode in modes().iter() {
        let logical = LogicalSize::from_physical(mode, physical);
        assert!(logical.to_physical(mode) == physical);
        assert!(logical.width(mode) == 1);
        assert!(logical.height(mode) == 2);
    }
}

#[test]
fn test_point_round_trip() {
    let physical = Point2D(1, 2);
    let container = Size2D(100, 200);
    for &mode in modes().iter() {
        let logical = LogicalPoint::from_physical(mode, physical, container);
        assert!(logical.to_physical(mode, container) == physical);
        assert!(logical.x(mode, container) == 1);
        assert!(logical.y(mode, container) == 2);
    }
}

#[test]
fn test_margin_round_trip() {
    let physical = SideOffsets2D::new(1, 2, 3, 4);
    for &mode in modes().iter() {
        let logical = LogicalMargin::from_physical(mode, physical);
        assert!(logical.to_physical(mode) == physical);
        assert!(logical.top(mode) == 1);
        assert!(logical.right(mode) == 2);
        assert!(logical.bottom(mode) == 3);
        assert!(logical.left(mode) == 4);
    }
}

#[test]
fn test_rect_round_trip() {
    let physical = Rect(Point2D(1, 2), Size2D(3, 4));
    let container = Size2D(100, 200);
    for &mode in modes().iter() {
        let logical = LogicalRect::from_physical(mode, physical, container);
        assert!(logical.to_physical(mode, container) == physical);
    }
}
