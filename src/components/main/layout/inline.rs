/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use css::node_style::StyledNode;
use layout::box_::{Box, CannotSplit, SplitDidFit, SplitDidNotFit};
use layout::context::LayoutContext;
use layout::floats::{FloatLeft, Floats, PlacementInfo};
use layout::flow::{BaseFlow, FlowClass, Flow, InlineFlowClass};
use layout::flow;
use layout::model::IntrinsicWidths;
use layout::model;
use layout::text;
use layout::wrapper::ThreadSafeLayoutNode;

use collections::{Deque, RingBuf};
use geom::{Point2D, Rect, SideOffsets2D, Size2D};
use gfx::display_list::ContentLevel;
use gfx::font::FontMetrics;
use gfx::font_context::FontContext;
use servo_util::geometry::Au;
use servo_util::geometry;
use servo_util::range;
use servo_util::range::{EachIndex, Range, RangeIndex, IntRangeIndex};
use std::iter::Enumerate;
use std::fmt;
use std::mem;
use std::num;
use std::slice::{Items, MutItems};
use std::u16;
use style::computed_values::{text_align, vertical_align, white_space};
use style::ComputedValues;
use sync::Arc;

/// Lineboxes are represented as offsets into the child list, rather than
/// as an object that "owns" boxes. Choosing a different set of line
/// breaks requires a new list of offsets, and possibly some splitting and
/// merging of TextBoxes.
///
/// A similar list will keep track of the mapping between CSS boxes and
/// the corresponding boxes in the inline flow.
///
/// After line breaks are determined, render boxes in the inline flow may
/// overlap visually. For example, in the case of nested inline CSS boxes,
/// outer inlines must be at least as large as the inner inlines, for
/// purposes of drawing noninherited things like backgrounds, borders,
/// outlines.
///
/// N.B. roc has an alternative design where the list instead consists of
/// things like "start outer box, text, start inner box, text, end inner
/// box, text, end outer box, text". This seems a little complicated to
/// serve as the starting point, but the current design doesn't make it
/// hard to try out that alternative.
///
/// Line boxes also contain some metadata used during line breaking. The
/// green zone is the area that the line can expand to before it collides
/// with a float or a horizontal wall of the containing block. The top
/// left corner of the green zone is the same as that of the line, but
/// the green zone can be taller and wider than the line itself.
pub struct LineBox {
    /// Consider the following HTML and rendered element with linebreaks:
    ///
    /// ~~~html
    /// <span>I <span>like truffles,</span> yes I do.</span>
    /// ~~~
    ///
    /// ~~~
    /// +-----------+
    /// | I like    |
    /// | truffles, |
    /// | yes I do. |
    /// +-----------+
    /// ~~~
    ///
    /// The ranges that describe these lines would be:
    ///
    /// ~~~
    /// | [0.0, 1.4) | [1.5, 2.0)  | [2.1, 3.0)  |
    /// |------------|-------------|-------------|
    /// | 'I like'   | 'truffles,' | 'yes I do.' |
    /// ~~~
    pub range: Range<LineIndices>,
    pub bounds: Rect<Au>,
    pub green_zone: Size2D<Au>
}

int_range_index! {
    #[doc = "The index of a box fragment into the flattened vector of DOM"]
    #[doc = "elements."]
    #[doc = ""]
    #[doc = "For example, given the HTML below:"]
    #[doc = ""]
    #[doc = "~~~"]
    #[doc = "<span>I <span>like      truffles,</span> yes I do.</span>"]
    #[doc = "~~~"]
    #[doc = ""]
    #[doc = "The fragments would be indexed as follows:"]
    #[doc = ""]
    #[doc = "~~~"]
    #[doc = "|  0   |        1         |       2      |"]
    #[doc = "|------|------------------|--------------|"]
    #[doc = "| 'I ' | 'like truffles,' | ' yes I do.' |"]
    #[doc = "~~~"]
    struct FragmentIndex(int)
}

int_range_index! {
    #[doc = "The index of a glyph in a single DOM fragment. Ligatures and"]
    #[doc = "continuous runs of whitespace are treated as single glyphs."]
    #[doc = "Non-breakable DOM fragments such as images are treated as"]
    #[doc = "having a range length of `1`."]
    #[doc = ""]
    #[doc = "For example, given the HTML below:"]
    #[doc = ""]
    #[doc = "~~~"]
    #[doc = "<span>like      truffles,</span>"]
    #[doc = "~~~"]
    #[doc = ""]
    #[doc = "The glyphs would be indexed as follows:"]
    #[doc = ""]
    #[doc = "~~~"]
    #[doc = "| 0 | 1 | 2 | 3 | 4 | 5 | 6 | 7 |  8  | 9 | 10 | 11 |"]
    #[doc = "|---|---|---|---|---|---|---|---|-----|---|----|----|"]
    #[doc = "| l | i | k | e |   | t | r | u | ffl | e | s  | ,  |"]
    #[doc = "~~~"]
    struct GlyphIndex(int)
}

/// A line index consists of two indices: a fragment index that refers to the
/// index of a DOM fragment within a flattened inline element; and a glyph index
/// where the 0th glyph refers to the first glyph of that fragment.
#[deriving(Clone, Eq, Ord, TotalEq, TotalOrd, Zero)]
pub struct LineIndices {
    pub fragment_index: FragmentIndex,
    pub glyph_index: GlyphIndex,
}

impl RangeIndex for LineIndices {}

impl Add<LineIndices, LineIndices> for LineIndices {
    fn add(&self, other: &LineIndices) -> LineIndices {
        // TODO: use debug_assert! after rustc upgrade
        if cfg!(not(ndebug)) {
            assert!(other.fragment_index == num::zero() || other.glyph_index == num::zero(),
                    "Attempted to add {} to {}. Both the fragment_index and \
                     glyph_index of the RHS are non-zero. This probably \
                     was a mistake!", self, other);
        }
        LineIndices {
            fragment_index: self.fragment_index + other.fragment_index,
            glyph_index: self.glyph_index + other.glyph_index,
        }
    }
}

impl Sub<LineIndices, LineIndices> for LineIndices {
    fn sub(&self, other: &LineIndices) -> LineIndices {
        // TODO: use debug_assert! after rustc upgrade
        if cfg!(not(ndebug)) {
            assert!(other.fragment_index == num::zero() || other.glyph_index == num::zero(),
                    "Attempted to subtract {} from {}. Both the \
                     fragment_index and glyph_index of the RHS are non-zero. \
                     This probably was a mistake!", self, other);
        }
        LineIndices {
            fragment_index: self.fragment_index - other.fragment_index,
            glyph_index: self.glyph_index - other.glyph_index,
        }
    }
}

impl Neg<LineIndices> for LineIndices {
    fn neg(&self) -> LineIndices {
        // TODO: use debug_assert! after rustc upgrade
        if cfg!(not(ndebug)) {
            assert!(self.fragment_index == num::zero() || self.glyph_index == num::zero(),
                    "Attempted to negate {}. Both the fragment_index and \
                     glyph_index are non-zero. This probably was a mistake!",
                     self);
        }
        LineIndices {
            fragment_index: -self.fragment_index,
            glyph_index: -self.glyph_index,
        }
    }
}

impl fmt::Show for LineIndices {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f.buf, "{}.{}", self.fragment_index, self.glyph_index)
    }
}

pub fn each_fragment_index(range: &Range<LineIndices>) -> EachIndex<int, FragmentIndex> {
    range::each_index(range.begin().fragment_index, range.length().fragment_index)
}

pub fn each_glyph_index(range: &Range<LineIndices>) -> EachIndex<int, GlyphIndex> {
    range::each_index(range.begin().glyph_index, range.length().glyph_index)
}

struct LineboxScanner {
    pub floats: Floats,
    pub new_boxes: Vec<Box>,
    pub work_list: RingBuf<Box>,
    pub pending_line: LineBox,
    pub lines: Vec<LineBox>,
    pub cur_y: Au,
}

impl LineboxScanner {
    pub fn new(float_ctx: Floats) -> LineboxScanner {
        LineboxScanner {
            floats: float_ctx,
            new_boxes: Vec::new(),
            work_list: RingBuf::new(),
            pending_line: LineBox {
                range: Range::empty(),
                bounds: Rect(Point2D(Au::new(0), Au::new(0)), Size2D(Au::new(0), Au::new(0))),
                green_zone: Size2D(Au::new(0), Au::new(0))
            },
            lines: Vec::new(),
            cur_y: Au::new(0)
        }
    }

    pub fn floats(&mut self) -> Floats {
        self.floats.clone()
    }

    fn reset_scanner(&mut self) {
        debug!("Resetting line box scanner's state for flow.");
        self.lines = Vec::new();
        self.new_boxes = Vec::new();
        self.cur_y = Au(0);
        self.reset_linebox();
    }

    fn reset_linebox(&mut self) {
        self.pending_line.range.reset(num::zero(), num::zero());
        self.pending_line.bounds = Rect(Point2D(Au::new(0), self.cur_y), Size2D(Au::new(0), Au::new(0)));
        self.pending_line.green_zone = Size2D(Au::new(0), Au::new(0))
    }

    pub fn scan_for_lines(&mut self, flow: &mut InlineFlow) {
        self.reset_scanner();

        // Swap out temporarily.
        let InlineBoxes {
            boxes: old_boxes,
            map: mut map
        } = mem::replace(&mut flow.boxes, InlineBoxes::new());

        let mut old_box_iter = old_boxes.iter();
        loop {
            // acquire the next box to lay out from work list or box list
            let cur_box = if self.work_list.is_empty() {
                match old_box_iter.next() {
                    None => break,
                    Some(fragment) => {
                        debug!("LineboxScanner: Working with fragment from flow: b{}",
                               fragment.debug_id());
                        (*fragment).clone()
                    }
                }
            } else {
                let fragment = self.work_list.pop_front().unwrap();
                debug!("LineboxScanner: Working with box from work list: b{}",
                       fragment.debug_id());
                fragment
            };

            let box_was_appended = match cur_box.white_space() {
                white_space::normal => self.try_append_to_line(cur_box, flow),
                white_space::pre => self.try_append_to_line_by_new_line(cur_box),
            };

            if !box_was_appended {
                debug!("LineboxScanner: Box wasn't appended, because line {:u} was full.",
                        self.lines.len());
                self.flush_current_line();
            } else {
                debug!("LineboxScanner: appended a box to line {:u}", self.lines.len());
            }
        }

        if self.pending_line.range.length() > num::zero() {
            debug!("LineboxScanner: Partially full linebox {:u} left at end of scanning.",
                    self.lines.len());
            self.flush_current_line();
        }

        map.fixup(old_boxes.as_slice(), self.new_boxes.as_slice());
        flow.boxes = InlineBoxes {
            boxes: mem::replace(&mut self.new_boxes, Vec::new()),
            map: map,
        };

        flow.lines = mem::replace(&mut self.lines, Vec::new());
    }

    fn flush_current_line(&mut self) {
        debug!("LineboxScanner: Flushing line {:u}: {:?}",
               self.lines.len(), self.pending_line);

        // clear line and add line mapping
        debug!("LineboxScanner: Saving information for flushed line {:u}.", self.lines.len());
        self.lines.push(self.pending_line);
        self.cur_y = self.pending_line.bounds.origin.y + self.pending_line.bounds.size.height;
        self.reset_linebox();
    }

    // FIXME(eatkinson): this assumes that the tallest box in the line determines the line height
    // This might not be the case with some weird text fonts.
    fn new_height_for_line(&self, new_box: &Box) -> Au {
        let box_height = new_box.content_height();
        if box_height > self.pending_line.bounds.size.height {
            box_height
        } else {
            self.pending_line.bounds.size.height
        }
    }

    /// Computes the position of a line that has only the provided box. Returns the bounding rect
    /// of the line's green zone (whose origin coincides with the line's origin) and the actual
    /// width of the first box after splitting.
    fn initial_line_placement(&self, first_box: &Box, ceiling: Au, flow: &mut InlineFlow)
                              -> (Rect<Au>, Au) {
        debug!("LineboxScanner: Trying to place first box of line {}", self.lines.len());

        let first_box_size = first_box.border_box.size;
        let splittable = first_box.can_split();
        debug!("LineboxScanner: box size: {}, splittable: {}", first_box_size, splittable);
        let line_is_empty: bool = self.pending_line.range.length() == num::zero();

        // Initally, pretend a splittable box has 0 width.
        // We will move it later if it has nonzero width
        // and that causes problems.
        let placement_width = if splittable {
            Au::new(0)
        } else {
            first_box_size.width
        };

        let mut info = PlacementInfo {
            size: Size2D(placement_width, first_box_size.height),
            ceiling: ceiling,
            max_width: flow.base.position.size.width,
            kind: FloatLeft,
        };

        let line_bounds = self.floats.place_between_floats(&info);

        debug!("LineboxScanner: found position for line: {} using placement_info: {:?}",
               line_bounds,
               info);

        // Simple case: if the box fits, then we can stop here
        if line_bounds.size.width > first_box_size.width {
            debug!("LineboxScanner: case=box fits");
            return (line_bounds, first_box_size.width);
        }

        // If not, but we can't split the box, then we'll place
        // the line here and it will overflow.
        if !splittable {
            debug!("LineboxScanner: case=line doesn't fit, but is unsplittable");
            return (line_bounds, first_box_size.width);
        }

        // Otherwise, try and split the box
        // FIXME(eatkinson): calling split_to_width here seems excessive and expensive.
        // We should find a better abstraction or merge it with the call in
        // try_append_to_line.
        match first_box.split_to_width(line_bounds.size.width, line_is_empty) {
            CannotSplit => {
                error!("LineboxScanner: Tried to split unsplittable render box! {}",
                        first_box);
                return (line_bounds, first_box_size.width);
            }
            SplitDidFit(left, right) => {

                debug!("LineboxScanner: case=box split and fit");
                let actual_box_width = match (left, right) {
                    (Some(l_box), Some(_))  => l_box.border_box.size.width,
                    (Some(l_box), None)     => l_box.border_box.size.width,
                    (None, Some(r_box))     => r_box.border_box.size.width,
                    (None, None)            => fail!("This case makes no sense.")
                };
                return (line_bounds, actual_box_width);
            }
            SplitDidNotFit(left, right) => {
                // The split didn't fit, but we might be able to
                // push it down past floats.


                debug!("LineboxScanner: case=box split and fit didn't fit; trying to push it down");
                let actual_box_width = match (left, right) {
                    (Some(l_box), Some(_))  => l_box.border_box.size.width,
                    (Some(l_box), None)     => l_box.border_box.size.width,
                    (None, Some(r_box))     => r_box.border_box.size.width,
                    (None, None)            => fail!("This case makes no sense.")
                };

                info.size.width = actual_box_width;
                let new_bounds = self.floats.place_between_floats(&info);

                debug!("LineboxScanner: case=new line position: {}", new_bounds);
                return (new_bounds, actual_box_width);
            }
        }

    }

    /// Performs float collision avoidance. This is called when adding a box is going to increase
    /// the height, and because of that we will collide with some floats.
    ///
    /// We have two options here:
    /// 1) Move the entire line so that it doesn't collide any more.
    /// 2) Break the line and put the new box on the next line.
    ///
    /// The problem with option 1 is that we might move the line and then wind up breaking anyway,
    /// which violates the standard.
    /// But option 2 is going to look weird sometimes.
    ///
    /// So we'll try to move the line whenever we can, but break if we have to.
    ///
    /// Returns false if and only if we should break the line.
    fn avoid_floats(&mut self,
                    in_box: Box,
                    flow: &mut InlineFlow,
                    new_height: Au,
                    line_is_empty: bool)
                    -> bool {
        debug!("LineboxScanner: entering float collision avoider!");

        // First predict where the next line is going to be.
        let this_line_y = self.pending_line.bounds.origin.y;
        let (next_line, first_box_width) = self.initial_line_placement(&in_box, this_line_y, flow);
        let next_green_zone = next_line.size;

        let new_width = self.pending_line.bounds.size.width + first_box_width;

        // Now, see if everything can fit at the new location.
        if next_green_zone.width >= new_width && next_green_zone.height >= new_height {
            debug!("LineboxScanner: case=adding box collides vertically with floats: moving line");

            self.pending_line.bounds.origin = next_line.origin;
            self.pending_line.green_zone = next_green_zone;

            assert!(!line_is_empty, "Non-terminating line breaking");
            self.work_list.push_front(in_box);
            return true
        }

        debug!("LineboxScanner: case=adding box collides vertically with floats: breaking line");
        self.work_list.push_front(in_box);
        false
    }

    fn try_append_to_line_by_new_line(&mut self, in_box: Box) -> bool {
        if in_box.new_line_pos.len() == 0 {
            // In case of box does not include new-line character
            self.push_box_to_line(in_box);
            true
        } else {
            // In case of box includes new-line character
            match in_box.split_by_new_line() {
                SplitDidFit(left, right) => {
                    match (left, right) {
                        (Some(left_box), Some(right_box)) => {
                            self.push_box_to_line(left_box);
                            self.work_list.push_front(right_box);
                        }
                        (Some(left_box), None) => {
                            self.push_box_to_line(left_box);
                        }
                        _ => error!("LineboxScanner: This split case makes no sense!"),
                    }
                }
                _ => {}
            }
            false
        }
    }

    /// Tries to append the given box to the line, splitting it if necessary. Returns false only if
    /// we should break the line.
    fn try_append_to_line(&mut self, in_box: Box, flow: &mut InlineFlow) -> bool {
        let line_is_empty = self.pending_line.range.length() == num::zero();
        if line_is_empty {
            let (line_bounds, _) = self.initial_line_placement(&in_box, self.cur_y, flow);
            self.pending_line.bounds.origin = line_bounds.origin;
            self.pending_line.green_zone = line_bounds.size;
        }

        debug!("LineboxScanner: Trying to append box to line {:u} (box size: {}, green zone: \
                {}): {}",
               self.lines.len(),
               in_box.border_box.size,
               self.pending_line.green_zone,
               in_box);

        let green_zone = self.pending_line.green_zone;

        // NB: At this point, if `green_zone.width < self.pending_line.bounds.size.width` or
        // `green_zone.height < self.pending_line.bounds.size.height`, then we committed a line
        // that overlaps with floats.

        let new_height = self.new_height_for_line(&in_box);
        if new_height > green_zone.height {
            // Uh-oh. Float collision imminent. Enter the float collision avoider
            return self.avoid_floats(in_box, flow, new_height, line_is_empty)
        }

        // If we're not going to overflow the green zone vertically, we might still do so
        // horizontally. We'll try to place the whole box on this line and break somewhere if it
        // doesn't fit.

        let new_width = self.pending_line.bounds.size.width + in_box.border_box.size.width;
        if new_width <= green_zone.width {
            debug!("LineboxScanner: case=box fits without splitting");
            self.push_box_to_line(in_box);
            return true
        }

        if !in_box.can_split() {
            // TODO(eatkinson, issue #224): Signal that horizontal overflow happened?
            if line_is_empty {
                debug!("LineboxScanner: case=box can't split and line {:u} is empty, so \
                        overflowing.",
                        self.lines.len());
                self.push_box_to_line(in_box);
                return true
            }
        }

        let available_width = green_zone.width - self.pending_line.bounds.size.width;
        let split = in_box.split_to_width(available_width, line_is_empty);
        let (left, right) = match (split, line_is_empty) {
            (CannotSplit, _) => {
                debug!("LineboxScanner: Tried to split unsplittable render box! {}",
                        in_box);
                self.work_list.push_front(in_box);
                return false
            }
            (SplitDidNotFit(_, _), false) => {
                debug!("LineboxScanner: case=split box didn't fit, not appending and deferring \
                        original box.");
                self.work_list.push_front(in_box);
                return false
            }
            (SplitDidFit(left, right), _) => {
                debug!("LineboxScanner: case=split box did fit; deferring remainder box.");
                (left, right)
                // Fall through to push boxes to the line.
            }
            (SplitDidNotFit(left, right), true) => {
                // TODO(eatkinson, issue #224): Signal that horizontal overflow happened?
                debug!("LineboxScanner: case=split box didn't fit and line {:u} is empty, so \
                        overflowing and deferring remainder box.",
                        self.lines.len());
                (left, right)
                // Fall though to push boxes to the line.
            }
        };

        match (left, right) {
            (Some(left_box), Some(right_box)) => {
                self.push_box_to_line(left_box);
                self.work_list.push_front(right_box);
            }
            (Some(left_box), None) => self.push_box_to_line(left_box),
            (None, Some(right_box)) => self.push_box_to_line(right_box),
            (None, None) => error!("LineboxScanner: This split case makes no sense!"),
        }

        true
    }

    // An unconditional push
    fn push_box_to_line(&mut self, box_: Box) {
        debug!("LineboxScanner: Pushing box {} to line {:u}", box_.debug_id(), self.lines.len());

        if self.pending_line.range.length() == num::zero() {
            assert!(self.new_boxes.len() <= (u16::MAX as uint));
            self.pending_line.range.reset(
                LineIndices {
                    fragment_index: FragmentIndex(self.new_boxes.len() as int),
                    glyph_index: GlyphIndex(0) /* unused for now */,
                },
                num::zero()
            );
        }
        self.pending_line.range.extend_by(LineIndices {
            fragment_index: FragmentIndex(1),
            glyph_index: GlyphIndex(0) /* unused for now */ ,
        });
        self.pending_line.bounds.size.width = self.pending_line.bounds.size.width +
            box_.border_box.size.width;
        self.pending_line.bounds.size.height = Au::max(self.pending_line.bounds.size.height,
                                                       box_.border_box.size.height);
        self.new_boxes.push(box_);
    }
}

/// Iterator over boxes.
pub struct BoxIterator<'a> {
    iter: Enumerate<Items<'a,Box>>,
    map: &'a FragmentMap,
}

impl<'a> Iterator<(&'a Box, InlineFragmentContext<'a>)> for BoxIterator<'a> {
    #[inline]
    fn next(&mut self) -> Option<(&'a Box, InlineFragmentContext<'a>)> {
        match self.iter.next() {
            None => None,
            Some((i, fragment)) => Some((
                fragment,
                InlineFragmentContext::new(self.map, FragmentIndex(i as int)),
            )),
        }
    }
}

/// Mutable iterator over boxes.
pub struct MutBoxIterator<'a> {
    iter: Enumerate<MutItems<'a,Box>>,
    map: &'a FragmentMap,
}

impl<'a> Iterator<(&'a mut Box, InlineFragmentContext<'a>)> for MutBoxIterator<'a> {
    #[inline]
    fn next(&mut self) -> Option<(&'a mut Box, InlineFragmentContext<'a>)> {
        match self.iter.next() {
            None => None,
            Some((i, fragment)) => Some((
                fragment,
                InlineFragmentContext::new(self.map, FragmentIndex(i as int)),
            )),
        }
    }
}

/// Represents a list of inline boxes, including element ranges.
pub struct InlineBoxes {
    /// The boxes themselves.
    pub boxes: Vec<Box>,
    /// Tracks the elements that made up the boxes above.
    pub map: FragmentMap,
}

impl InlineBoxes {
    /// Creates an empty set of inline boxes.
    pub fn new() -> InlineBoxes {
        InlineBoxes {
            boxes: Vec::new(),
            map: FragmentMap::new(),
        }
    }

    /// Returns the number of inline boxes.
    pub fn len(&self) -> uint {
        self.boxes.len()
    }

    /// Returns true if this list contains no boxes and false if it contains at least one box.
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// Pushes a new inline box.
    pub fn push(&mut self, fragment: Box, style: Arc<ComputedValues>) {
        self.map.push(style, Range::new(FragmentIndex(self.boxes.len() as int), FragmentIndex(1)));
        self.boxes.push(fragment)
    }

    /// Merges another set of inline boxes with this one.
    pub fn push_all(&mut self, other: InlineBoxes) {
        let InlineBoxes {
            boxes: other_boxes,
            map: other_map
        } = other;
        let adjustment = FragmentIndex(self.boxes.len() as int);
        self.map.push_all(other_map, adjustment);
        self.boxes.push_all_move(other_boxes);
    }

    /// Returns an iterator that iterates over all boxes along with the appropriate context.
    pub fn iter<'a>(&'a self) -> BoxIterator<'a> {
        BoxIterator {
            iter: self.boxes.as_slice().iter().enumerate(),
            map: &self.map,
        }
    }

    /// Returns an iterator that iterates over all boxes along with the appropriate context and
    /// allows those boxes to be mutated.
    pub fn mut_iter<'a>(&'a mut self) -> MutBoxIterator<'a> {
        MutBoxIterator {
            iter: self.boxes.as_mut_slice().mut_iter().enumerate(),
            map: &self.map,
        }
    }

    /// A convenience function to return the box at a given index.
    pub fn get<'a>(&'a self, index: uint) -> &'a Box {
        self.boxes.get(index)
    }

    /// A convenience function to return a mutable reference to the box at a given index.
    pub fn get_mut<'a>(&'a mut self, index: uint) -> &'a mut Box {
        self.boxes.get_mut(index)
    }
}

/// Flows for inline layout.
pub struct InlineFlow {
    /// Data common to all flows.
    pub base: BaseFlow,

    /// A vector of all inline fragments. Several fragments may correspond to one node/element.
    pub boxes: InlineBoxes,

    /// A vector of ranges into boxes that represents line positions. These ranges are disjoint and
    /// are the result of inline layout. This also includes some metadata used for positioning
    /// lines.
    pub lines: Vec<LineBox>,

    /// The minimum height above the baseline for each line, as specified by the line height and
    /// font style.
    pub minimum_height_above_baseline: Au,

    /// The minimum depth below the baseline for each line, as specified by the line height and
    /// font style.
    pub minimum_depth_below_baseline: Au,
}

impl InlineFlow {
    pub fn from_boxes(node: ThreadSafeLayoutNode, boxes: InlineBoxes) -> InlineFlow {
        InlineFlow {
            base: BaseFlow::new(node),
            boxes: boxes,
            lines: Vec::new(),
            minimum_height_above_baseline: Au(0),
            minimum_depth_below_baseline: Au(0),
        }
    }

    pub fn build_display_list_inline(&mut self, layout_context: &LayoutContext) {
        let abs_rect = Rect(self.base.abs_position, self.base.position.size);
        if !abs_rect.intersects(&layout_context.dirty) {
            return
        }

        // TODO(#228): Once we form line boxes and have their cached bounds, we can be smarter and
        // not recurse on a line if nothing in it can intersect the dirty region.
        debug!("Flow: building display list for {:u} inline boxes", self.boxes.len());

        for (fragment, context) in self.boxes.mut_iter() {
            let rel_offset = fragment.relative_position(&self.base
                                                             .absolute_position_info
                                                             .relative_containing_block_size,
                                                        Some(context));
            drop(fragment.build_display_list(&mut self.base.display_list,
                                             layout_context,
                                             self.base.abs_position + rel_offset,
                                             ContentLevel,
                                             Some(context)));
        }

        // TODO(#225): Should `inline-block` elements have flows as children of the inline flow or
        // should the flow be nested inside the box somehow?

        // For now, don't traverse the subtree rooted here.
    }

    /// Returns the distance from the baseline for the logical top left corner of this fragment,
    /// taking into account the value of the CSS `vertical-align` property. Negative values mean
    /// "toward the logical top" and positive values mean "toward the logical bottom".
    ///
    /// The extra boolean is set if and only if `biggest_top` and/or `biggest_bottom` were updated.
    /// That is, if the box has a `top` or `bottom` value, true is returned.
    fn distance_from_baseline(fragment: &Box,
                              ascent: Au,
                              parent_text_top: Au,
                              parent_text_bottom: Au,
                              height_above_baseline: &mut Au,
                              depth_below_baseline: &mut Au,
                              largest_height_for_top_fragments: &mut Au,
                              largest_height_for_bottom_fragments: &mut Au)
                              -> (Au, bool) {
        match fragment.vertical_align() {
            vertical_align::baseline => (-ascent, false),
            vertical_align::middle => {
                // TODO: x-height value should be used from font info.
                let xheight = Au(0);
                (-(xheight + fragment.content_height()).scale_by(0.5), false)
            },
            vertical_align::sub => {
                // TODO: The proper position for subscripts should be used. Lower the baseline to
                // the proper position for subscripts.
                let sub_offset = Au(0);
                (sub_offset - ascent, false)
            },
            vertical_align::super_ => {
                // TODO: The proper position for superscripts should be used. Raise the baseline to
                // the proper position for superscripts.
                let super_offset = Au(0);
                (-super_offset - ascent, false)
            },
            vertical_align::text_top => {
                let box_height = *height_above_baseline + *depth_below_baseline;
                let prev_depth_below_baseline = *depth_below_baseline;
                *height_above_baseline = parent_text_top;
                *depth_below_baseline = box_height - *height_above_baseline;
                (*depth_below_baseline - prev_depth_below_baseline - ascent, false)
            },
            vertical_align::text_bottom => {
                let box_height = *height_above_baseline + *depth_below_baseline;
                let prev_depth_below_baseline = *depth_below_baseline;
                *depth_below_baseline = parent_text_bottom;
                *height_above_baseline = box_height - *depth_below_baseline;
                (*depth_below_baseline - prev_depth_below_baseline - ascent, false)
            },
            vertical_align::top => {
                *largest_height_for_top_fragments =
                    Au::max(*largest_height_for_top_fragments,
                            *height_above_baseline + *depth_below_baseline);
                let offset_top = *height_above_baseline - ascent;
                (offset_top, true)
            },
            vertical_align::bottom => {
                *largest_height_for_bottom_fragments =
                    Au::max(*largest_height_for_bottom_fragments,
                            *height_above_baseline + *depth_below_baseline);
                let offset_bottom = -(*depth_below_baseline + ascent);
                (offset_bottom, true)
            },
            vertical_align::Length(length) => (-(length + ascent), false),
            vertical_align::Percentage(p) => {
                let pt_size = fragment.font_style().pt_size;
                let line_height = fragment.calculate_line_height(Au::from_pt(pt_size));
                let percent_offset = line_height.scale_by(p);
                (-(percent_offset + ascent), false)
            }
        }
    }

    /// Sets box X positions based on alignment for one line.
    fn set_horizontal_box_positions(boxes: &mut InlineBoxes,
                                    line: &LineBox,
                                    linebox_align: text_align::T) {
        // Figure out how much width we have.
        let slack_width = Au::max(Au(0), line.green_zone.width - line.bounds.size.width);

        // Set the box x positions based on that alignment.
        let mut offset_x = line.bounds.origin.x;
        offset_x = offset_x + match linebox_align {
            // So sorry, but justified text is more complicated than shuffling linebox
            // coordinates.
            //
            // TODO(burg, issue #213): Implement `text-align: justify`.
            text_align::left | text_align::justify => Au(0),
            text_align::center => slack_width.scale_by(0.5),
            text_align::right => slack_width,
        };

        for i in each_fragment_index(&line.range) {
            let box_ = boxes.get_mut(i.to_uint());
            let size = box_.border_box.size;
            box_.border_box = Rect(Point2D(offset_x, box_.border_box.origin.y), size);
            offset_x = offset_x + size.width;
        }
    }

    /// Computes the minimum ascent and descent for each line. This is done during flow
    /// construction.
    ///
    /// `style` is the style of the block.
    pub fn compute_minimum_ascent_and_descent(&mut self,
                                              font_context: &mut FontContext,
                                              style: &ComputedValues) {
        let font_style = text::computed_style_to_font_style(style);
        let font_metrics = text::font_metrics_for_style(font_context, &font_style);
        let line_height = text::line_height_from_style(style, style.Font.get().font_size);
        let inline_metrics = InlineMetrics::from_font_metrics(&font_metrics, line_height);
        self.minimum_height_above_baseline = inline_metrics.height_above_baseline;
        self.minimum_depth_below_baseline = inline_metrics.depth_below_baseline;
    }
}

impl Flow for InlineFlow {
    fn class(&self) -> FlowClass {
        InlineFlowClass
    }

    fn as_immutable_inline<'a>(&'a self) -> &'a InlineFlow {
        self
    }

    fn as_inline<'a>(&'a mut self) -> &'a mut InlineFlow {
        self
    }

    fn bubble_widths(&mut self, _: &mut LayoutContext) {
        for kid in self.base.child_iter() {
            flow::mut_base(kid).floats = Floats::new();
        }

        let mut intrinsic_widths = IntrinsicWidths::new();
        for (fragment, context) in self.boxes.mut_iter() {
            debug!("Flow: measuring {}", *fragment);

            let box_intrinsic_widths = fragment.intrinsic_widths(Some(context));
            intrinsic_widths.minimum_width = geometry::max(intrinsic_widths.minimum_width,
                                                           box_intrinsic_widths.minimum_width);
            intrinsic_widths.preferred_width = geometry::max(intrinsic_widths.preferred_width,
                                                             box_intrinsic_widths.preferred_width);
        }

        self.base.intrinsic_widths = intrinsic_widths;
    }

    /// Recursively (top-down) determines the actual width of child contexts and boxes. When called
    /// on this context, the context has had its width set by the parent context.
    fn assign_widths(&mut self, _: &mut LayoutContext) {
        // Initialize content box widths if they haven't been initialized already.
        //
        // TODO: Combine this with `LineboxScanner`'s walk in the box list, or put this into `Box`.

        debug!("InlineFlow::assign_widths: floats in: {:?}", self.base.floats);

        {
            let this = &mut *self;
            for (fragment, context) in this.boxes.mut_iter() {
                fragment.assign_replaced_width_if_necessary(self.base.position.size.width,
                                                            Some(context))
            }
        }

        assert!(self.base.children.len() == 0,
                "InlineFlow: should not have children flows in the current layout implementation.");

        // There are no child contexts, so stop here.

        // TODO(Issue #225): once there are 'inline-block' elements, this won't be
        // true.  In that case, set the InlineBlockBox's width to the
        // shrink-to-fit width, perform inline flow, and set the block
        // flow context's width as the assigned width of the
        // 'inline-block' box that created this flow before recursing.
    }

    /// Calculate and set the height of this flow. See CSS 2.1 § 10.6.1.
    fn assign_height(&mut self, _: &mut LayoutContext) {
        debug!("assign_height_inline: assigning height for flow");

        // Divide the boxes into lines.
        //
        // TODO(#226): Get the CSS `line-height` property from the containing block's style to
        // determine minimum linebox height.
        //
        // TODO(#226): Get the CSS `line-height` property from each non-replaced inline element to
        // determine its height for computing linebox height.
        //
        // TODO(pcwalton): Cache the linebox scanner?
        debug!("assign_height_inline: floats in: {:?}", self.base.floats);

        // assign height for inline boxes
        for (fragment, _) in self.boxes.mut_iter() {
            fragment.assign_replaced_height_if_necessary();
        }

        let scanner_floats = self.base.floats.clone();
        let mut scanner = LineboxScanner::new(scanner_floats);
        scanner.scan_for_lines(self);

        // All lines use text alignment of the flow.
        let text_align = self.base.flags.text_align();

        // Now, go through each line and lay out the boxes inside.
        let mut line_distance_from_flow_top = Au(0);
        for line in self.lines.mut_iter() {
            // Lay out boxes horizontally.
            InlineFlow::set_horizontal_box_positions(&mut self.boxes, line, text_align);

            // Set the top y position of the current line box.
            // `line_height_offset` is updated at the end of the previous loop.
            line.bounds.origin.y = line_distance_from_flow_top;

            // Calculate the distance from the baseline to the top and bottom of the line box.
            let mut largest_height_above_baseline = self.minimum_height_above_baseline;
            let mut largest_depth_below_baseline = self.minimum_depth_below_baseline;

            // Calculate the largest height among boxes with 'top' and 'bottom' values
            // respectively.
            let (mut largest_height_for_top_fragments, mut largest_height_for_bottom_fragments) =
                (Au(0), Au(0));

            for box_i in each_fragment_index(&line.range) {
                let fragment = self.boxes.boxes.get_mut(box_i.to_uint());

                let InlineMetrics {
                    height_above_baseline: mut height_above_baseline,
                    depth_below_baseline: mut depth_below_baseline,
                    ascent
                } = fragment.inline_metrics();

                // To calculate text-top and text-bottom value when `vertical-align` is involved,
                // we should find the top and bottom of the content area of the parent box.
                // "Content area" is defined in CSS 2.1 § 10.6.1.
                //
                // TODO: We should extract em-box info from the font size of the parent and
                // calculate the distances from the baseline to the top and the bottom of the
                // parent's content area.

                // We should calculate the distance from baseline to the top of parent's content
                // area. But for now we assume it's the font size.
                //
                // CSS 2.1 does not state which font to use. Previous versions of the code used
                // the parent's font; this code uses the current font.
                let parent_text_top = fragment.style().Font.get().font_size;

                // We should calculate the distance from baseline to the bottom of the parent's
                // content area. But for now we assume it's zero.
                let parent_text_bottom = Au(0);

                // Calculate the final height above the baseline for this box.
                //
                // The no-update flag decides whether `largest_height_for_top_fragments` and
                // `largest_height_for_bottom_fragments` are to be updated or not. This will be set
                // if and only if the fragment has `vertical-align` set to `top` or `bottom`.
                let (distance_from_baseline, no_update_flag) =
                    InlineFlow::distance_from_baseline(
                        fragment,
                        ascent,
                        parent_text_top,
                        parent_text_bottom,
                        &mut height_above_baseline,
                        &mut depth_below_baseline,
                        &mut largest_height_for_top_fragments,
                        &mut largest_height_for_bottom_fragments);

                // Unless the current fragment has `vertical-align` set to `top` or `bottom`,
                // `largest_height_above_baseline` and `largest_depth_below_baseline` are updated.
                if !no_update_flag {
                    largest_height_above_baseline = Au::max(height_above_baseline,
                                                            largest_height_above_baseline);
                    largest_depth_below_baseline = Au::max(depth_below_baseline,
                                                           largest_depth_below_baseline);
                }

                // Temporarily use `fragment.border_box.origin.y` to mean "the distance from the
                // baseline". We will assign the real value later.
                fragment.border_box.origin.y = distance_from_baseline
            }

            // Calculate the distance from the baseline to the top of the largest box with a
            // value for `bottom`. Then, if necessary, update `largest_height_above_baseline`.
            largest_height_above_baseline =
                Au::max(largest_height_above_baseline,
                        largest_height_for_bottom_fragments - largest_depth_below_baseline);

            // Calculate the distance from baseline to the bottom of the largest box with a value
            // for `top`. Then, if necessary, update `largest_depth_below_baseline`.
            largest_depth_below_baseline =
                Au::max(largest_depth_below_baseline,
                        largest_height_for_top_fragments - largest_height_above_baseline);

            // Now, the distance from the logical top of the line box to the baseline can be
            // computed as `largest_height_above_baseline`.
            let baseline_distance_from_top = largest_height_above_baseline;

            // Compute the final positions in the block direction of each fragment. Recall that
            // `fragment.border_box.origin.y` was set to the distance from the baseline above.
            for box_i in each_fragment_index(&line.range) {
                let fragment = self.boxes.get_mut(box_i.to_uint());
                match fragment.vertical_align() {
                    vertical_align::top => {
                        fragment.border_box.origin.y = fragment.border_box.origin.y +
                            line_distance_from_flow_top
                    }
                    vertical_align::bottom => {
                        fragment.border_box.origin.y = fragment.border_box.origin.y +
                            line_distance_from_flow_top + baseline_distance_from_top +
                            largest_depth_below_baseline
                    }
                    _ => {
                        fragment.border_box.origin.y = fragment.border_box.origin.y +
                            line_distance_from_flow_top + baseline_distance_from_top
                    }
                }
            }

            // This is used to set the top y position of the next line box in the next loop.
            line.bounds.size.height = largest_height_above_baseline + largest_depth_below_baseline;
            line_distance_from_flow_top = line_distance_from_flow_top + line.bounds.size.height;
        } // End of `lines.each` loop.

        self.base.position.size.height =
            if self.lines.len() > 0 {
                self.lines.as_slice().last().get_ref().bounds.origin.y +
                    self.lines.as_slice().last().get_ref().bounds.size.height
            } else {
                Au::new(0)
            };

        self.base.floats = scanner.floats();
        self.base.floats.translate(Point2D(Au::new(0), -self.base.position.size.height));
    }
}

impl fmt::Show for InlineFlow {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        try!(write!(f.buf, "InlineFlow"));
        for (i, (fragment, _)) in self.boxes.iter().enumerate() {
            if i == 0 {
                try!(write!(f.buf, ": {}", fragment))
            } else {
                try!(write!(f.buf, ", {}", fragment))
            }
        }
        Ok(())
    }
}

/// Information that inline flows keep about a single nested element. This is used to recover the
/// DOM structure from the flat box list when it's needed.
pub struct FragmentRange {
    /// The style of the DOM node that this range refers to.
    pub style: Arc<ComputedValues>,
    /// The range, in indices into the fragment list.
    pub range: Range<FragmentIndex>,
}

impl FragmentRange {
    /// Creates a new fragment range from the given values.
    fn new(style: Arc<ComputedValues>, range: Range<FragmentIndex>) -> FragmentRange {
        FragmentRange {
            style: style,
            range: range,
        }
    }

    /// Returns the dimensions of the border in this fragment range.
    pub fn border(&self) -> SideOffsets2D<Au> {
        model::border_from_style(&*self.style)
    }

    /// Returns the dimensions of the padding in this fragment range.
    pub fn padding(&self) -> SideOffsets2D<Au> {
        // FIXME(#2266, pcwalton): Is Au(0) right here for the containing block?
        model::padding_from_style(&*self.style, Au(0))
    }
}

struct FragmentFixupWorkItem {
    style: Arc<ComputedValues>,
    new_start_index: FragmentIndex,
    old_end_index: FragmentIndex,
}

/// The type of an iterator over fragment ranges in the fragment map.
pub struct RangeIterator<'a> {
    iter: Items<'a,FragmentRange>,
    index: FragmentIndex,
    seen_first: bool,
}

impl<'a> Iterator<&'a FragmentRange> for RangeIterator<'a> {
    fn next(&mut self) -> Option<&'a FragmentRange> {
        if self.seen_first {
            match self.iter.next() {
                Some(fragment_range) if fragment_range.range.contains(self.index) => {
                    return Some(fragment_range)
                }
                Some(_) | None => return None
            }
        }

        loop {
            match self.iter.next() {
                None => return None,
                Some(fragment_range) if fragment_range.range.contains(self.index) => {
                    self.seen_first = true;
                    return Some(fragment_range)
                }
                Some(_) => {}
            }
        }
    }
}

/// Information that inline flows keep about nested elements. This is used to recover the DOM
/// structure from the flat box list when it's needed.
pub struct FragmentMap {
    list: Vec<FragmentRange>,
}

impl FragmentMap {
    /// Creates a new fragment map.
    pub fn new() -> FragmentMap {
        FragmentMap {
            list: Vec::new(),
        }
    }

    /// Adds the given node to the fragment map.
    pub fn push(&mut self, style: Arc<ComputedValues>, range: Range<FragmentIndex>) {
        self.list.push(FragmentRange::new(style, range))
    }

    /// Pushes the ranges in another fragment map onto the end of this one, adjusting indices as
    /// necessary.
    fn push_all(&mut self, other: FragmentMap, adjustment: FragmentIndex) {
        let FragmentMap {
            list: other_list
        } = other;

        for other_range in other_list.move_iter() {
            let FragmentRange {
                style: other_style,
                range: mut other_range
            } = other_range;

            other_range.shift_by(adjustment);
            self.push(other_style, other_range)
        }
    }

    /// Returns the range with the given index.
    pub fn get_mut<'a>(&'a mut self, index: FragmentIndex) -> &'a mut FragmentRange {
        &mut self.list.as_mut_slice()[index.to_uint()]
    }

    /// Iterates over all ranges that contain the box with the given index, outermost first.
    #[inline(always)]
    fn ranges_for_index<'a>(&'a self, index: FragmentIndex) -> RangeIterator<'a> {
        RangeIterator {
            iter: self.list.as_slice().iter(),
            index: index,
            seen_first: false,
        }
    }

    /// Rebuilds the list after the fragments have been split or deleted (for example, for line
    /// breaking). This assumes that the overall structure of the DOM has not changed; if the
    /// DOM has changed, then the flow constructor will need to do more complicated surgery than
    /// this function can provide.
    ///
    /// FIXME(#2267, pcwalton): It would be more efficient to not have to clone boxes all the time;
    /// i.e. if `old_boxes` contained less info than the entire range of boxes. See
    /// `layout::construct::strip_ignorable_whitespace_from_start` for an example of some code that
    /// needlessly has to clone boxes.
    pub fn fixup(&mut self, old_fragments: &[Box], new_fragments: &[Box]) {
        // TODO(pcwalton): Post Rust upgrade, use `with_capacity` here.
        let old_list = mem::replace(&mut self.list, Vec::new());
        let mut worklist = Vec::new();        // FIXME(#2269, pcwalton): was smallvec4
        let mut old_list_iter = old_list.move_iter().peekable();
        let mut new_fragments_iter = new_fragments.iter().enumerate().peekable();

        // FIXME(#2270, pcwalton): I don't think this will work if multiple old fragments
        // correspond to the same node.
        for (i, old_fragment) in old_fragments.iter().enumerate() {
            let old_fragment_index = FragmentIndex(i as int);
            // Find the start of the corresponding new fragment.
            let new_fragment_start = match new_fragments_iter.peek() {
                Some(&(index, new_fragment)) if new_fragment.node == old_fragment.node => {
                    // We found the start of the corresponding new fragment.
                    FragmentIndex(index as int)
                }
                Some(_) | None => {
                    // The old fragment got deleted entirely.
                    continue
                }
            };
            drop(new_fragments_iter.next());

            // Eat any additional fragments that the old fragment got split into.
            loop {
                match new_fragments_iter.peek() {
                    Some(&(_, new_fragment)) if new_fragment.node == old_fragment.node => {}
                    Some(_) | None => break,
                }
                drop(new_fragments_iter.next());
            }

            // Find all ranges that started at this old fragment and add them onto the worklist.
            loop {
                match old_list_iter.peek() {
                    None => break,
                    Some(fragment_range) => {
                        if fragment_range.range.begin() > old_fragment_index {
                            // We haven't gotten to the appropriate old fragment yet, so stop.
                            break
                        }
                        // Note that it can be the case that `fragment_range.range.begin() < i`.
                        // This is OK, as it corresponds to the case in which a fragment got
                        // deleted entirely (e.g. ignorable whitespace got nuked). In that case we
                        // want to keep the range, but shorten it.
                    }
                };

                let FragmentRange {
                    style: style,
                    range: old_range,
                } = old_list_iter.next().unwrap();
                worklist.push(FragmentFixupWorkItem {
                    style: style,
                    new_start_index: new_fragment_start,
                    old_end_index: old_range.end(),
                });
            }

            // Pop off any ranges that ended at this fragment.
            loop {
                match worklist.as_slice().last() {
                    None => break,
                    Some(last_work_item) => {
                        if last_work_item.old_end_index > old_fragment_index + FragmentIndex(1) {
                            // Haven't gotten to it yet.
                            break
                        }
                    }
                }

                let new_last_index = match new_fragments_iter.peek() {
                    None => {
                        // At the end.
                        FragmentIndex(new_fragments.len() as int)
                    }
                    Some(&(index, _)) => {
                        FragmentIndex(index as int)
                    },
                };

                let FragmentFixupWorkItem {
                    style,
                    new_start_index,
                    ..
                } = worklist.pop().unwrap();
                let range = Range::new(new_start_index, new_last_index - new_start_index);
                self.list.push(FragmentRange::new(style, range))
            }
        }
    }
}

/// The context that an inline fragment appears in. This allows the fragment map to be passed in
/// conveniently to various fragment functions.
pub struct InlineFragmentContext<'a> {
    map: &'a FragmentMap,
    index: FragmentIndex,
}

impl<'a> InlineFragmentContext<'a> {
    pub fn new<'a>(map: &'a FragmentMap, index: FragmentIndex) -> InlineFragmentContext<'a> {
        InlineFragmentContext {
            map: map,
            index: index,
        }
    }

    pub fn ranges(&self) -> RangeIterator<'a> {
        self.map.ranges_for_index(self.index)
    }
}

/// Height above the baseline, depth below the baseline, and ascent for a fragment. See CSS 2.1 §
/// 10.8.1.
pub struct InlineMetrics {
    pub height_above_baseline: Au,
    pub depth_below_baseline: Au,
    pub ascent: Au,
}

impl InlineMetrics {
    /// Calculates inline metrics from font metrics and line height per CSS 2.1 § 10.8.1.
    #[inline]
    pub fn from_font_metrics(font_metrics: &FontMetrics, line_height: Au) -> InlineMetrics {
        let leading = line_height - (font_metrics.ascent + font_metrics.descent);
        InlineMetrics {
            height_above_baseline: font_metrics.ascent + leading.scale_by(0.5),
            depth_below_baseline: font_metrics.descent + leading.scale_by(0.5),
            ascent: font_metrics.ascent,
        }
    }
}

