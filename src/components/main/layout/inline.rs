/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

#![deny(unsafe_block)]

use css::node_style::StyledNode;
use layout::context::LayoutContext;
use layout::floats::{FloatLeft, Floats, PlacementInfo};
use layout::flow::{BaseFlow, FlowClass, Flow, InlineFlowClass};
use layout::flow;
use layout::fragment::{Fragment, ScannedTextFragment, ScannedTextFragmentInfo, SplitInfo};
use layout::model::IntrinsicWidths;
use layout::model;
use layout::text;
use layout::wrapper::ThreadSafeLayoutNode;

use collections::{Deque, RingBuf};
use geom::{Point2D, Rect, SideOffsets2D, Size2D};
use gfx::display_list::ContentLevel;
use gfx::font::FontMetrics;
use gfx::font_context::FontContext;
use gfx::text::glyph::CharIndex;
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

/// `Line`s are represented as offsets into the child list, rather than
/// as an object that "owns" fragments. Choosing a different set of line
/// breaks requires a new list of offsets, and possibly some splitting and
/// merging of TextFragments.
///
/// A similar list will keep track of the mapping between CSS fragments and
/// the corresponding fragments in the inline flow.
///
/// After line breaks are determined, render fragments in the inline flow may
/// overlap visually. For example, in the case of nested inline CSS fragments,
/// outer inlines must be at least as large as the inner inlines, for
/// purposes of drawing noninherited things like backgrounds, borders,
/// outlines.
///
/// N.B. roc has an alternative design where the list instead consists of
/// things like "start outer fragment, text, start inner fragment, text, end inner
/// fragment, text, end outer fragment, text". This seems a little complicated to
/// serve as the starting point, but the current design doesn't make it
/// hard to try out that alternative.
///
/// Line fragments also contain some metadata used during line breaking. The
/// green zone is the area that the line can expand to before it collides
/// with a float or a horizontal wall of the containing block. The top
/// left corner of the green zone is the same as that of the line, but
/// the green zone can be taller and wider than the line itself.
pub struct Line {
    /// A range of line indices that describe line breaks.
    ///
    /// For example, consider the following HTML and rendered element with
    /// linebreaks:
    ///
    /// ~~~html
    /// <span>I <span>like truffles, <img></span> yes I do.</span>
    /// ~~~
    ///
    /// ~~~
    /// +------------+
    /// | I like     |
    /// | truffles,  |
    /// | +----+     |
    /// | |    |     |
    /// | +----+ yes |
    /// | I do.      |
    /// +------------+
    /// ~~~
    ///
    /// The ranges that describe these lines would be:
    ///
    /// ~~~
    /// | [0.0, 1.4) | [1.5, 2.0)  | [2.0, 3.4)  | [3.4, 4.0) |
    /// |------------|-------------|-------------|------------|
    /// | 'I like'   | 'truffles,' | '<img> yes' | 'I do.'    |
    /// ~~~
    pub range: Range<LineIndices>,
    /// The bounds are the exact position and extents of the line with respect
    /// to the parent box.
    ///
    /// For example, for the HTML below...
    ///
    /// ~~~html
    /// <div><span>I <span>like      truffles, <img></span></div>
    /// ~~~
    ///
    /// ...the bounds would be:
    ///
    /// ~~~
    /// +-----------------------------------------------------------+
    /// |               ^                                           |
    /// |               |                                           |
    /// |            origin.y                                       |
    /// |               |                                           |
    /// |               v                                           |
    /// |< - origin.x ->+ - - - - - - - - +---------+----           |
    /// |               |                 |         |   ^           |
    /// |               |                 |  <img>  |  size.height  |
    /// |               I like truffles,  |         |   v           |
    /// |               + - - - - - - - - +---------+----           |
    /// |               |                           |               |
    /// |               |<------ size.width ------->|               |
    /// |                                                           |
    /// |                                                           |
    /// +-----------------------------------------------------------+
    /// ~~~
    pub bounds: Rect<Au>,
    /// The green zone is the greatest extent from wich a line can extend to
    /// before it collides with a float.
    ///
    /// ~~~
    /// +-----------------------+
    /// |:::::::::::::::::      |
    /// |:::::::::::::::::FFFFFF|
    /// |============:::::FFFFFF|
    /// |:::::::::::::::::FFFFFF|
    /// |:::::::::::::::::FFFFFF|
    /// |:::::::::::::::::      |
    /// |    FFFFFFFFF          |
    /// |    FFFFFFFFF          |
    /// |    FFFFFFFFF          |
    /// |                       |
    /// +-----------------------+
    ///
    /// === line
    /// ::: green zone
    /// FFF float
    /// ~~~
    pub green_zone: Size2D<Au>
}

int_range_index! {
    #[doc = "The index of a fragment in a flattened vector of DOM elements."]
    struct FragmentIndex(int)
}

/// A line index consists of two indices: a fragment index that refers to the
/// index of a DOM fragment within a flattened inline element; and a glyph index
/// where the 0th glyph refers to the first glyph of that fragment.
#[deriving(Clone, PartialEq, PartialOrd, Eq, Ord, Zero)]
pub struct LineIndices {
    /// The index of a fragment into the flattened vector of DOM elements.
    ///
    /// For example, given the HTML below:
    ///
    /// ~~~html
    /// <span>I <span>like      truffles, <img></span> yes I do.</span>
    /// ~~~
    ///
    /// The fragments would be indexed as follows:
    ///
    /// ~~~
    /// |  0   |        1         |   2   |       3      |
    /// |------|------------------|-------|--------------|
    /// | 'I ' | 'like truffles,' | <img> | ' yes I do.' |
    /// ~~~
    pub fragment_index: FragmentIndex,
    /// The index of a character in a DOM fragment. Continuous runs of whitespace
    /// are treated as single characters. Non-breakable DOM fragments such as
    /// images are treated as having a range length of `1`.
    ///
    /// For example, given the HTML below:
    ///
    /// ~~~html
    /// <span>I <span>like      truffles, <img></span> yes I do.</span>
    /// ~~~
    ///
    /// The characters would be indexed as follows:
    ///
    /// ~~~
    /// | 0 | 1 | 0 | 1 | 2 | 3 | 4 | 5 | 6 | 7 | 8 | 9 | 10 | 11 | 12 | 13 | 14 |
    /// |---|---|---|---|---|---|---|---|---|---|---|---|----|----|----|----|----|
    /// | I |   | l | i | k | e |   | t | r | u | f | f | l  | e  | s  | ,  |    |
    ///
    /// |   0   | 0 | 1 | 2 | 3 | 4 | 5 | 6 | 7 | 8 | 9 |
    /// |-------|---|---|---|---|---|---|---|---|---|---|
    /// | <img> |   | y | e | s |   | I |   | d | o | . |
    /// ~~~
    pub char_index: CharIndex,
}

impl RangeIndex for LineIndices {}

impl Add<LineIndices, LineIndices> for LineIndices {
    fn add(&self, other: &LineIndices) -> LineIndices {
        // TODO: use debug_assert! after rustc upgrade
        if cfg!(not(ndebug)) {
            assert!(other.fragment_index == num::zero() || other.char_index == num::zero(),
                    "Attempted to add {} to {}. Both the fragment_index and \
                     char_index of the RHS are non-zero. This probably was a \
                     mistake!", self, other);
        }
        LineIndices {
            fragment_index: self.fragment_index + other.fragment_index,
            char_index: self.char_index + other.char_index,
        }
    }
}

impl Sub<LineIndices, LineIndices> for LineIndices {
    fn sub(&self, other: &LineIndices) -> LineIndices {
        // TODO: use debug_assert! after rustc upgrade
        if cfg!(not(ndebug)) {
            assert!(other.fragment_index == num::zero() || other.char_index == num::zero(),
                    "Attempted to subtract {} from {}. Both the fragment_index \
                     and char_index of the RHS are non-zero. This probably was \
                     a mistake!", self, other);
        }
        LineIndices {
            fragment_index: self.fragment_index - other.fragment_index,
            char_index: self.char_index - other.char_index,
        }
    }
}

impl Neg<LineIndices> for LineIndices {
    fn neg(&self) -> LineIndices {
        // TODO: use debug_assert! after rustc upgrade
        if cfg!(not(ndebug)) {
            assert!(self.fragment_index == num::zero() || self.char_index == num::zero(),
                    "Attempted to negate {}. Both the fragment_index and \
                     char_index are non-zero. This probably was a mistake!",
                     self);
        }
        LineIndices {
            fragment_index: -self.fragment_index,
            char_index: -self.char_index,
        }
    }
}

impl fmt::Show for LineIndices {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}.{}", self.fragment_index, self.char_index)
    }
}

pub fn each_fragment_index(range: &Range<LineIndices>) -> EachIndex<int, FragmentIndex> {
    range::each_index(range.begin().fragment_index, range.end().fragment_index)
}

pub fn each_char_index(range: &Range<LineIndices>) -> EachIndex<int, CharIndex> {
    range::each_index(range.begin().char_index, range.end().char_index)
}

struct LineBreaker {
    pub floats: Floats,
    pub new_fragments: Vec<Fragment>,
    pub work_list: RingBuf<Fragment>,
    pub pending_line: Line,
    pub lines: Vec<Line>,
    pub cur_y: Au,
}

impl LineBreaker {
    pub fn new(float_ctx: Floats) -> LineBreaker {
        LineBreaker {
            floats: float_ctx,
            new_fragments: Vec::new(),
            work_list: RingBuf::new(),
            pending_line: Line {
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
        debug!("Resetting LineBreaker's state for flow.");
        self.lines = Vec::new();
        self.new_fragments = Vec::new();
        self.cur_y = Au(0);
        self.reset_line();
    }

    fn reset_line(&mut self) {
        self.pending_line.range.reset(num::zero(), num::zero());
        self.pending_line.bounds = Rect(Point2D(Au::new(0), self.cur_y), Size2D(Au::new(0), Au::new(0)));
        self.pending_line.green_zone = Size2D(Au::new(0), Au::new(0))
    }

    pub fn scan_for_lines(&mut self, flow: &mut InlineFlow) {
        self.reset_scanner();

        let mut old_fragments = mem::replace(&mut flow.fragments, InlineFragments::new());

        { // Enter a new scope so that old_fragment_iter's borrow is released
            let mut old_fragment_iter = old_fragments.fragments.iter();
            loop {
                // acquire the next fragment to lay out from work list or fragment list
                let cur_fragment = if self.work_list.is_empty() {
                    match old_fragment_iter.next() {
                        None => break,
                        Some(fragment) => {
                            debug!("LineBreaker: Working with fragment from flow: b{}",
                                   fragment.debug_id());
                            (*fragment).clone()
                        }
                    }
                } else {
                    let fragment = self.work_list.pop_front().unwrap();
                    debug!("LineBreaker: Working with fragment from work list: b{}",
                           fragment.debug_id());
                    fragment
                };

                let fragment_was_appended = match cur_fragment.white_space() {
                    white_space::normal => self.try_append_to_line(cur_fragment, flow),
                    white_space::pre => self.try_append_to_line_by_new_line(cur_fragment),
                };

                if !fragment_was_appended {
                    debug!("LineBreaker: Fragment wasn't appended, because line {:u} was full.",
                            self.lines.len());
                    self.flush_current_line();
                } else {
                    debug!("LineBreaker: appended a fragment to line {:u}", self.lines.len());
                }
            }

            if self.pending_line.range.length() > num::zero() {
                debug!("LineBreaker: Partially full line {:u} left at end of scanning.",
                        self.lines.len());
                self.flush_current_line();
            }
        }

        old_fragments.fixup(mem::replace(&mut self.new_fragments, vec![]));
        flow.fragments = old_fragments;
        flow.lines = mem::replace(&mut self.lines, Vec::new());
    }

    fn flush_current_line(&mut self) {
        debug!("LineBreaker: Flushing line {:u}: {:?}",
               self.lines.len(), self.pending_line);

        // clear line and add line mapping
        debug!("LineBreaker: Saving information for flushed line {:u}.", self.lines.len());
        self.lines.push(self.pending_line);
        self.cur_y = self.pending_line.bounds.origin.y + self.pending_line.bounds.size.height;
        self.reset_line();
    }

    // FIXME(eatkinson): this assumes that the tallest fragment in the line determines the line height
    // This might not be the case with some weird text fonts.
    fn new_height_for_line(&self, new_fragment: &Fragment) -> Au {
        let fragment_height = new_fragment.content_height();
        if fragment_height > self.pending_line.bounds.size.height {
            fragment_height
        } else {
            self.pending_line.bounds.size.height
        }
    }

    /// Computes the position of a line that has only the provided fragment. Returns the bounding
    /// rect of the line's green zone (whose origin coincides with the line's origin) and the actual
    /// width of the first fragment after splitting.
    fn initial_line_placement(&self, first_fragment: &Fragment, ceiling: Au, flow: &InlineFlow)
                              -> (Rect<Au>, Au) {
        debug!("LineBreaker: Trying to place first fragment of line {}", self.lines.len());

        let first_fragment_size = first_fragment.border_box.size;
        let splittable = first_fragment.can_split();
        debug!("LineBreaker: fragment size: {}, splittable: {}", first_fragment_size, splittable);

        // Initally, pretend a splittable fragment has 0 width.
        // We will move it later if it has nonzero width
        // and that causes problems.
        let placement_width = if splittable {
            Au::new(0)
        } else {
            first_fragment_size.width
        };

        let info = PlacementInfo {
            size: Size2D(placement_width, first_fragment_size.height),
            ceiling: ceiling,
            max_width: flow.base.position.size.width,
            kind: FloatLeft,
        };

        let line_bounds = self.floats.place_between_floats(&info);

        debug!("LineBreaker: found position for line: {} using placement_info: {:?}",
               line_bounds,
               info);

        // Simple case: if the fragment fits, then we can stop here
        if line_bounds.size.width > first_fragment_size.width {
            debug!("LineBreaker: case=fragment fits");
            return (line_bounds, first_fragment_size.width);
        }

        // If not, but we can't split the fragment, then we'll place
        // the line here and it will overflow.
        if !splittable {
            debug!("LineBreaker: case=line doesn't fit, but is unsplittable");
            return (line_bounds, first_fragment_size.width);
        }

        debug!("LineBreaker: used to call split_to_width here");
        return (line_bounds, first_fragment_size.width);
    }

    /// Performs float collision avoidance. This is called when adding a fragment is going to increase
    /// the height, and because of that we will collide with some floats.
    ///
    /// We have two options here:
    /// 1) Move the entire line so that it doesn't collide any more.
    /// 2) Break the line and put the new fragment on the next line.
    ///
    /// The problem with option 1 is that we might move the line and then wind up breaking anyway,
    /// which violates the standard.
    /// But option 2 is going to look weird sometimes.
    ///
    /// So we'll try to move the line whenever we can, but break if we have to.
    ///
    /// Returns false if and only if we should break the line.
    fn avoid_floats(&mut self,
                    in_fragment: Fragment,
                    flow: &InlineFlow,
                    new_height: Au,
                    line_is_empty: bool)
                    -> bool {
        debug!("LineBreaker: entering float collision avoider!");

        // First predict where the next line is going to be.
        let this_line_y = self.pending_line.bounds.origin.y;
        let (next_line, first_fragment_width) = self.initial_line_placement(&in_fragment, this_line_y, flow);
        let next_green_zone = next_line.size;

        let new_width = self.pending_line.bounds.size.width + first_fragment_width;

        // Now, see if everything can fit at the new location.
        if next_green_zone.width >= new_width && next_green_zone.height >= new_height {
            debug!("LineBreaker: case=adding fragment collides vertically with floats: moving line");

            self.pending_line.bounds.origin = next_line.origin;
            self.pending_line.green_zone = next_green_zone;

            assert!(!line_is_empty, "Non-terminating line breaking");
            self.work_list.push_front(in_fragment);
            return true
        }

        debug!("LineBreaker: case=adding fragment collides vertically with floats: breaking line");
        self.work_list.push_front(in_fragment);
        false
    }

    fn try_append_to_line_by_new_line(&mut self, in_fragment: Fragment) -> bool {
        if in_fragment.new_line_pos.len() == 0 {
                debug!("LineBreaker: Did not find a new-line character, so pushing the fragment to \
                       the line without splitting.");
            self.push_fragment_to_line(in_fragment);
            true
        } else {
            debug!("LineBreaker: Found a new-line character, so splitting theline.");

            let (left, right, run) = in_fragment.find_split_info_by_new_line()
                .expect("LineBreaker: This split case makes no sense!");

            // TODO(bjz): Remove fragment splitting
            let split_fragment = |split: SplitInfo| {
                let info = ScannedTextFragmentInfo::new(run.clone(), split.range);
                let specific = ScannedTextFragment(info);
                let size = Size2D(split.width, in_fragment.border_box.size.height);
                in_fragment.transform(size, specific)
            };

            debug!("LineBreaker: Pushing the fragment to the left of the new-line character \
                   to the line.");
            let mut left = split_fragment(left);
            left.new_line_pos = vec![];
            self.push_fragment_to_line(left);

            for right in right.move_iter() {
                debug!("LineBreaker: Deferring the fragment to the right of the new-line \
                       character to the line.");
                let mut right = split_fragment(right);
                right.new_line_pos = in_fragment.new_line_pos.clone();
                self.work_list.push_front(right);
            }
            false
        }
    }

    /// Tries to append the given fragment to the line, splitting it if necessary. Returns false only if
    /// we should break the line.
    fn try_append_to_line(&mut self, in_fragment: Fragment, flow: &InlineFlow) -> bool {
        let line_is_empty = self.pending_line.range.length() == num::zero();
        if line_is_empty {
            let (line_bounds, _) = self.initial_line_placement(&in_fragment, self.cur_y, flow);
            self.pending_line.bounds.origin = line_bounds.origin;
            self.pending_line.green_zone = line_bounds.size;
        }

        debug!("LineBreaker: Trying to append fragment to line {:u} (fragment size: {}, green zone: \
                {}): {}",
               self.lines.len(),
               in_fragment.border_box.size,
               self.pending_line.green_zone,
               in_fragment);

        let green_zone = self.pending_line.green_zone;

        // NB: At this point, if `green_zone.width < self.pending_line.bounds.size.width` or
        // `green_zone.height < self.pending_line.bounds.size.height`, then we committed a line
        // that overlaps with floats.

        let new_height = self.new_height_for_line(&in_fragment);
        if new_height > green_zone.height {
            // Uh-oh. Float collision imminent. Enter the float collision avoider
            return self.avoid_floats(in_fragment, flow, new_height, line_is_empty)
        }

        // If we're not going to overflow the green zone vertically, we might still do so
        // horizontally. We'll try to place the whole fragment on this line and break somewhere if it
        // doesn't fit.

        let new_width = self.pending_line.bounds.size.width + in_fragment.border_box.size.width;
        if new_width <= green_zone.width {
            debug!("LineBreaker: case=fragment fits without splitting");
            self.push_fragment_to_line(in_fragment);
            return true
        }

        if !in_fragment.can_split() {
            // TODO(eatkinson, issue #224): Signal that horizontal overflow happened?
            if line_is_empty {
                debug!("LineBreaker: case=fragment can't split and line {:u} is empty, so \
                        overflowing.",
                        self.lines.len());
                self.push_fragment_to_line(in_fragment);
                return true
            }
        }

        let available_width = green_zone.width - self.pending_line.bounds.size.width;
        let split = in_fragment.find_split_info_for_width(CharIndex(0), available_width, line_is_empty);
        match split.map(|(left, right, run)| {
            // TODO(bjz): Remove fragment splitting
            let split_fragment = |split: SplitInfo| {
                let info = ScannedTextFragmentInfo::new(run.clone(), split.range);
                let specific = ScannedTextFragment(info);
                let size = Size2D(split.width, in_fragment.border_box.size.height);
                in_fragment.transform(size, specific)
            };

            (left.map(|x| { debug!("LineBreaker: Left split {}", x); split_fragment(x) }),
             right.map(|x| { debug!("LineBreaker: Right split {}", x); split_fragment(x) }))
        }) {
            None => {
                debug!("LineBreaker: Tried to split unsplittable render fragment! Deferring to next \
                       line. {}", in_fragment);
                self.work_list.push_front(in_fragment);
                false
            },
            Some((Some(left_fragment), Some(right_fragment))) => {
                debug!("LineBreaker: Line break found! Pushing left fragment to line and deferring \
                       right fragment to next line.");
                self.push_fragment_to_line(left_fragment);
                self.work_list.push_front(right_fragment);
                true
            },
            Some((Some(left_fragment), None)) => {
                debug!("LineBreaker: Pushing left fragment to line.");
                self.push_fragment_to_line(left_fragment);
                true
            },
            Some((None, Some(right_fragment))) => {
                debug!("LineBreaker: Pushing right fragment to line.");
                self.push_fragment_to_line(right_fragment);
                true
            },
            Some((None, None)) => {
                error!("LineBreaker: This split case makes no sense!");
                true
            },
        }
    }

    // An unconditional push
    fn push_fragment_to_line(&mut self, fragment: Fragment) {
        debug!("LineBreaker: Pushing fragment {} to line {:u}", fragment.debug_id(), self.lines.len());

        if self.pending_line.range.length() == num::zero() {
            assert!(self.new_fragments.len() <= (u16::MAX as uint));
            self.pending_line.range.reset(
                LineIndices {
                    fragment_index: FragmentIndex(self.new_fragments.len() as int),
                    char_index: CharIndex(0) /* unused for now */,
                },
                num::zero()
            );
        }
        self.pending_line.range.extend_by(LineIndices {
            fragment_index: FragmentIndex(1),
            char_index: CharIndex(0) /* unused for now */ ,
        });
        self.pending_line.bounds.size.width = self.pending_line.bounds.size.width +
            fragment.border_box.size.width;
        self.pending_line.bounds.size.height = Au::max(self.pending_line.bounds.size.height,
                                                       fragment.border_box.size.height);
        self.new_fragments.push(fragment);
    }
}

/// Iterator over fragments.
pub struct FragmentIterator<'a> {
    iter: Enumerate<Items<'a,Fragment>>,
    ranges: &'a Vec<InlineFragmentRange>,
}

impl<'a> Iterator<(&'a Fragment, InlineFragmentContext<'a>)> for FragmentIterator<'a> {
    #[inline]
    fn next(&mut self) -> Option<(&'a Fragment, InlineFragmentContext<'a>)> {
        self.iter.next().map(|(i, fragment)| {
            (fragment, InlineFragmentContext::new(self.ranges, FragmentIndex(i as int)))
        })
    }
}

/// Mutable iterator over fragments.
pub struct MutFragmentIterator<'a> {
    iter: Enumerate<MutItems<'a,Fragment>>,
    ranges: &'a Vec<InlineFragmentRange>,
}

impl<'a> Iterator<(&'a mut Fragment, InlineFragmentContext<'a>)> for MutFragmentIterator<'a> {
    #[inline]
    fn next(&mut self) -> Option<(&'a mut Fragment, InlineFragmentContext<'a>)> {
        self.iter.next().map(|(i, fragment)| {
            (fragment, InlineFragmentContext::new(self.ranges, FragmentIndex(i as int)))
        })
    }
}

/// Represents a list of inline fragments, including element ranges.
pub struct InlineFragments {
    /// The fragments themselves.
    pub fragments: Vec<Fragment>,
    /// Tracks the elements that made up the fragments above. This is used to
    /// recover the DOM structure from the `fragments` when it's needed.
    pub ranges: Vec<InlineFragmentRange>,
}

impl InlineFragments {
    /// Creates an empty set of inline fragments.
    pub fn new() -> InlineFragments {
        InlineFragments {
            fragments: vec![],
            ranges: vec![],
        }
    }

    /// Returns the number of inline fragments.
    pub fn len(&self) -> uint {
        self.fragments.len()
    }

    /// Returns true if this list contains no fragments and false if it contains at least one fragment.
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// Pushes a new inline fragment.
    pub fn push(&mut self, fragment: Fragment, style: Arc<ComputedValues>) {
        self.ranges.push(InlineFragmentRange::new(
            style, Range::new(FragmentIndex(self.fragments.len() as int), FragmentIndex(1)),
        ));
        self.fragments.push(fragment)
    }

    /// Merges another set of inline fragments with this one.
    pub fn push_all(&mut self, InlineFragments { fragments, ranges }: InlineFragments) {
        let adjustment = FragmentIndex(self.fragments.len() as int);
        self.push_all_ranges(ranges, adjustment);
        self.fragments.push_all_move(fragments);
    }

    /// Returns an iterator that iterates over all fragments along with the appropriate context.
    pub fn iter<'a>(&'a self) -> FragmentIterator<'a> {
        FragmentIterator {
            iter: self.fragments.as_slice().iter().enumerate(),
            ranges: &self.ranges,
        }
    }

    /// Returns an iterator that iterates over all fragments along with the appropriate context and
    /// allows those fragments to be mutated.
    pub fn mut_iter<'a>(&'a mut self) -> MutFragmentIterator<'a> {
        MutFragmentIterator {
            iter: self.fragments.as_mut_slice().mut_iter().enumerate(),
            ranges: &self.ranges,
        }
    }

    /// A convenience function to return the fragment at a given index.
    pub fn get<'a>(&'a self, index: uint) -> &'a Fragment {
        self.fragments.get(index)
    }

    /// A convenience function to return a mutable reference to the fragment at a given index.
    pub fn get_mut<'a>(&'a mut self, index: uint) -> &'a mut Fragment {
        self.fragments.get_mut(index)
    }

    /// Adds the given node to the fragment map.
    pub fn push_range(&mut self, style: Arc<ComputedValues>, range: Range<FragmentIndex>) {
        self.ranges.push(InlineFragmentRange::new(style, range))
    }

    /// Pushes the ranges in a fragment map, adjusting indices as necessary.
    fn push_all_ranges(&mut self, ranges: Vec<InlineFragmentRange>, adjustment: FragmentIndex) {
        for other_range in ranges.move_iter() {
            let InlineFragmentRange {
                style: other_style,
                range: mut other_range
            } = other_range;

            other_range.shift_by(adjustment);
            self.push_range(other_style, other_range)
        }
    }

    /// Returns the range with the given index.
    pub fn get_mut_range<'a>(&'a mut self, index: FragmentIndex) -> &'a mut InlineFragmentRange {
        self.ranges.get_mut(index.to_uint())
    }

    /// Rebuilds the list after the fragments have been split or deleted (for example, for line
    /// breaking). This assumes that the overall structure of the DOM has not changed; if the
    /// DOM has changed, then the flow constructor will need to do more complicated surgery than
    /// this function can provide.
    ///
    /// FIXME(#2267, pcwalton): It would be more efficient to not have to clone fragments all the time;
    /// i.e. if `self.fragments` contained less info than the entire range of fragments. See
    /// `layout::construct::strip_ignorable_whitespace_from_start` for an example of some code that
    /// needlessly has to clone fragments.
    pub fn fixup(&mut self, new_fragments: Vec<Fragment>) {
        // TODO(pcwalton): Post Rust upgrade, use `with_capacity` here.
        let old_list = mem::replace(&mut self.ranges, vec![]);
        let mut worklist = vec![];        // FIXME(#2269, pcwalton): was smallvec4
        let mut old_list_iter = old_list.move_iter().peekable();

        { // Enter a new scope so that new_fragments_iter's borrow is released
            let mut new_fragments_iter = new_fragments.iter().enumerate().peekable();
            // FIXME(#2270, pcwalton): I don't think this will work if multiple old fragments
            // correspond to the same node.
            for (i, old_fragment) in self.fragments.iter().enumerate() {
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

                    let InlineFragmentRange {
                        style: style,
                        range: old_range,
                    } = old_list_iter.next().unwrap();
                    worklist.push(InlineFragmentFixupWorkItem {
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

                    let InlineFragmentFixupWorkItem {
                        style,
                        new_start_index,
                        ..
                    } = worklist.pop().unwrap();
                    let range = Range::new(new_start_index, new_last_index - new_start_index);
                    self.ranges.push(InlineFragmentRange::new(style, range))
                }
            }
        }
        self.fragments = new_fragments;
    }

    /// Strips ignorable whitespace from the start of a list of fragments.
    pub fn strip_ignorable_whitespace_from_start(&mut self) {
        if self.is_empty() { return }; // Fast path

        let new_fragments = mem::replace(&mut self.fragments, vec![])
            .move_iter()
            .skip_while(|fragment| {
                let is_whitespace_only = fragment.is_whitespace_only();
                if is_whitespace_only {
                    debug!("stripping ignorable whitespace from start");
                }
                is_whitespace_only
            }).collect();

        self.fixup(new_fragments);
    }

    /// Strips ignorable whitespace from the end of a list of fragments.
    pub fn strip_ignorable_whitespace_from_end(&mut self) {
        if self.is_empty() {
            return;
        }

        let mut new_fragments = self.fragments.clone();
        while new_fragments.len() > 0 && new_fragments.as_slice().last().get_ref().is_whitespace_only() {
            debug!("stripping ignorable whitespace from end");
            drop(new_fragments.pop());
        }

        self.fixup(new_fragments);
    }
}

/// Flows for inline layout.
pub struct InlineFlow {
    /// Data common to all flows.
    pub base: BaseFlow,

    /// A vector of all inline fragments. Several fragments may correspond to one node/element.
    pub fragments: InlineFragments,

    /// A vector of ranges into fragments that represents line positions. These ranges are disjoint and
    /// are the result of inline layout. This also includes some metadata used for positioning
    /// lines.
    pub lines: Vec<Line>,

    /// The minimum height above the baseline for each line, as specified by the line height and
    /// font style.
    pub minimum_height_above_baseline: Au,

    /// The minimum depth below the baseline for each line, as specified by the line height and
    /// font style.
    pub minimum_depth_below_baseline: Au,
}

impl InlineFlow {
    pub fn from_fragments(node: ThreadSafeLayoutNode, fragments: InlineFragments) -> InlineFlow {
        InlineFlow {
            base: BaseFlow::new(node),
            fragments: fragments,
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

        // TODO(#228): Once we form lines and have their cached bounds, we can be smarter and
        // not recurse on a line if nothing in it can intersect the dirty region.
        debug!("Flow: building display list for {:u} inline fragments", self.fragments.len());

        for (fragment, context) in self.fragments.mut_iter() {
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
        // should the flow be nested inside the fragment somehow?

        // For now, don't traverse the subtree rooted here.
    }

    /// Returns the distance from the baseline for the logical top left corner of this fragment,
    /// taking into account the value of the CSS `vertical-align` property. Negative values mean
    /// "toward the logical top" and positive values mean "toward the logical bottom".
    ///
    /// The extra boolean is set if and only if `biggest_top` and/or `biggest_bottom` were updated.
    /// That is, if the box has a `top` or `bottom` value, true is returned.
    fn distance_from_baseline(fragment: &Fragment,
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
                let fragment_height = fragment.content_height();
                let offset_top = -(xheight + fragment_height).scale_by(0.5);
                *height_above_baseline = offset_top.scale_by(-1.0);
                *depth_below_baseline = fragment_height - *height_above_baseline;
                (offset_top, false)
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
                let fragment_height = *height_above_baseline + *depth_below_baseline;
                let prev_depth_below_baseline = *depth_below_baseline;
                *height_above_baseline = parent_text_top;
                *depth_below_baseline = fragment_height - *height_above_baseline;
                (*depth_below_baseline - prev_depth_below_baseline - ascent, false)
            },
            vertical_align::text_bottom => {
                let fragment_height = *height_above_baseline + *depth_below_baseline;
                let prev_depth_below_baseline = *depth_below_baseline;
                *depth_below_baseline = parent_text_bottom;
                *height_above_baseline = fragment_height - *depth_below_baseline;
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

    /// Sets fragment X positions based on alignment for one line.
    fn set_horizontal_fragment_positions(fragments: &mut InlineFragments,
                                         line: &Line,
                                         line_align: text_align::T) {
        // Figure out how much width we have.
        let slack_width = Au::max(Au(0), line.green_zone.width - line.bounds.size.width);

        // Set the fragment x positions based on that alignment.
        let mut offset_x = line.bounds.origin.x;
        offset_x = offset_x + match line_align {
            // So sorry, but justified text is more complicated than shuffling line
            // coordinates.
            //
            // TODO(burg, issue #213): Implement `text-align: justify`.
            text_align::left | text_align::justify => Au(0),
            text_align::center => slack_width.scale_by(0.5),
            text_align::right => slack_width,
        };

        for i in each_fragment_index(&line.range) {
            let fragment = fragments.get_mut(i.to_uint());
            let size = fragment.border_box.size;
            fragment.border_box = Rect(Point2D(offset_x, fragment.border_box.origin.y), size);
            offset_x = offset_x + size.width;
        }
    }

    /// Computes the minimum ascent and descent for each line. This is done during flow
    /// construction.
    ///
    /// `style` is the style of the block.
    pub fn compute_minimum_ascent_and_descent(&self,
                                              font_context: &mut FontContext,
                                              style: &ComputedValues) -> (Au, Au) {
        let font_style = text::computed_style_to_font_style(style);
        let font_metrics = text::font_metrics_for_style(font_context, &font_style);
        let line_height = text::line_height_from_style(style, style.get_font().font_size);
        let inline_metrics = InlineMetrics::from_font_metrics(&font_metrics, line_height);
        (inline_metrics.height_above_baseline, inline_metrics.depth_below_baseline)
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
        for (fragment, context) in self.fragments.mut_iter() {
            debug!("Flow: measuring {}", *fragment);

            let fragment_intrinsic_widths = fragment.intrinsic_widths(Some(context));
            intrinsic_widths.minimum_width = geometry::max(intrinsic_widths.minimum_width,
                                                           fragment_intrinsic_widths.minimum_width);
            intrinsic_widths.preferred_width = geometry::max(intrinsic_widths.preferred_width,
                                                             fragment_intrinsic_widths.preferred_width);
        }

        self.base.intrinsic_widths = intrinsic_widths;
    }

    /// Recursively (top-down) determines the actual width of child contexts and fragments. When called
    /// on this context, the context has had its width set by the parent context.
    fn assign_widths(&mut self, _: &mut LayoutContext) {
        // Initialize content fragment widths if they haven't been initialized already.
        //
        // TODO: Combine this with `LineBreaker`'s walk in the fragment list, or put this into `Fragment`.

        debug!("InlineFlow::assign_widths: floats in: {:?}", self.base.floats);

        {
            let width = self.base.position.size.width;
            let this = &mut *self;
            for (fragment, context) in this.fragments.mut_iter() {
                fragment.assign_replaced_width_if_necessary(width,
                                                            Some(context))
            }
        }

        assert!(self.base.children.len() == 0,
                "InlineFlow: should not have children flows in the current layout implementation.");

        // There are no child contexts, so stop here.

        // TODO(Issue #225): once there are 'inline-block' elements, this won't be
        // true.  In that case, set the InlineBlockFragment's width to the
        // shrink-to-fit width, perform inline flow, and set the block
        // flow context's width as the assigned width of the
        // 'inline-block' fragment that created this flow before recursing.
    }

    /// Calculate and set the height of this flow. See CSS 2.1 § 10.6.1.
    fn assign_height(&mut self, _: &mut LayoutContext) {
        debug!("assign_height_inline: assigning height for flow");

        // Divide the fragments into lines.
        //
        // TODO(#226): Get the CSS `line-height` property from the containing block's style to
        // determine minimum line height.
        //
        // TODO(#226): Get the CSS `line-height` property from each non-replaced inline element to
        // determine its height for computing line height.
        //
        // TODO(pcwalton): Cache the line scanner?
        debug!("assign_height_inline: floats in: {:?}", self.base.floats);

        // assign height for inline fragments
        for (fragment, _) in self.fragments.mut_iter() {
            fragment.assign_replaced_height_if_necessary();
        }

        let scanner_floats = self.base.floats.clone();
        let mut scanner = LineBreaker::new(scanner_floats);
        scanner.scan_for_lines(self);

        // All lines use text alignment of the flow.
        let text_align = self.base.flags.text_align();

        // Now, go through each line and lay out the fragments inside.
        let mut line_distance_from_flow_top = Au(0);
        for line in self.lines.mut_iter() {
            // Lay out fragments horizontally.
            InlineFlow::set_horizontal_fragment_positions(&mut self.fragments, line, text_align);

            // Set the top y position of the current line.
            // `line_height_offset` is updated at the end of the previous loop.
            line.bounds.origin.y = line_distance_from_flow_top;

            // Calculate the distance from the baseline to the top and bottom of the line.
            let mut largest_height_above_baseline = self.minimum_height_above_baseline;
            let mut largest_depth_below_baseline = self.minimum_depth_below_baseline;

            // Calculate the largest height among fragments with 'top' and 'bottom' values
            // respectively.
            let (mut largest_height_for_top_fragments, mut largest_height_for_bottom_fragments) =
                (Au(0), Au(0));

            for fragment_i in each_fragment_index(&line.range) {
                let fragment = self.fragments.fragments.get_mut(fragment_i.to_uint());

                let InlineMetrics {
                    height_above_baseline: mut height_above_baseline,
                    depth_below_baseline: mut depth_below_baseline,
                    ascent
                } = fragment.inline_metrics();

                // To calculate text-top and text-bottom value when `vertical-align` is involved,
                // we should find the top and bottom of the content area of the parent fragment.
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
                let parent_text_top = fragment.style().get_font().font_size;

                // We should calculate the distance from baseline to the bottom of the parent's
                // content area. But for now we assume it's zero.
                let parent_text_bottom = Au(0);

                // Calculate the final height above the baseline for this fragment.
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

            // Calculate the distance from the baseline to the top of the largest fragment with a
            // value for `bottom`. Then, if necessary, update `largest_height_above_baseline`.
            largest_height_above_baseline =
                Au::max(largest_height_above_baseline,
                        largest_height_for_bottom_fragments - largest_depth_below_baseline);

            // Calculate the distance from baseline to the bottom of the largest fragment with a value
            // for `top`. Then, if necessary, update `largest_depth_below_baseline`.
            largest_depth_below_baseline =
                Au::max(largest_depth_below_baseline,
                        largest_height_for_top_fragments - largest_height_above_baseline);

            // Now, the distance from the logical top of the line to the baseline can be
            // computed as `largest_height_above_baseline`.
            let baseline_distance_from_top = largest_height_above_baseline;

            // Compute the final positions in the block direction of each fragment. Recall that
            // `fragment.border_box.origin.y` was set to the distance from the baseline above.
            for fragment_i in each_fragment_index(&line.range) {
                let fragment = self.fragments.get_mut(fragment_i.to_uint());
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

            // This is used to set the top y position of the next line in the next loop.
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
        try!(write!(f, "InlineFlow"));
        for (i, (fragment, _)) in self.fragments.iter().enumerate() {
            if i == 0 {
                try!(write!(f, ": {}", fragment))
            } else {
                try!(write!(f, ", {}", fragment))
            }
        }
        Ok(())
    }
}

/// Information that inline flows keep about a single nested element. This is used to recover the
/// DOM structure from the flat fragment list when it's needed.
pub struct InlineFragmentRange {
    /// The style of the DOM node that this range refers to.
    pub style: Arc<ComputedValues>,
    /// The range, in indices into the fragment list.
    pub range: Range<FragmentIndex>,
}

impl InlineFragmentRange {
    /// Creates a new fragment range from the given values.
    fn new(style: Arc<ComputedValues>, range: Range<FragmentIndex>) -> InlineFragmentRange {
        InlineFragmentRange {
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

struct InlineFragmentFixupWorkItem {
    style: Arc<ComputedValues>,
    new_start_index: FragmentIndex,
    old_end_index: FragmentIndex,
}

/// The type of an iterator over fragment ranges in the fragment map.
pub struct RangeIterator<'a> {
    iter: Items<'a,InlineFragmentRange>,
    index: FragmentIndex,
    is_first: bool,
}

impl<'a> Iterator<&'a InlineFragmentRange> for RangeIterator<'a> {
    fn next(&mut self) -> Option<&'a InlineFragmentRange> {
        if !self.is_first {
            // Yield the next fragment range if it contains the index
            self.iter.next().and_then(|frag_range| {
                if frag_range.range.contains(self.index) { Some(frag_range) } else { None }
            })
        } else {
            // Find the first fragment range that contains the index if it exists
            let index = self.index;
            let first = self.iter.by_ref().find(|frag_range| {
                frag_range.range.contains(index)
            });
            self.is_first = false; // We have made our first iteration
            first
        }
    }
}

/// The context that an inline fragment appears in. This allows the fragment map to be passed in
/// conveniently to various fragment functions.
pub struct InlineFragmentContext<'a> {
    ranges: &'a Vec<InlineFragmentRange>,
    index: FragmentIndex,
}

impl<'a> InlineFragmentContext<'a> {
    pub fn new<'a>(ranges: &'a Vec<InlineFragmentRange>, index: FragmentIndex) -> InlineFragmentContext<'a> {
        InlineFragmentContext {
            ranges: ranges,
            index: index,
        }
    }

    /// Iterates over all ranges that contain the fragment at context's index, outermost first.
    #[inline(always)]
    pub fn ranges(&self) -> RangeIterator<'a> {
        // TODO: It would be more straightforward to return an existing iterator
        // rather defining our own `RangeIterator`, but this requires unboxed
        // closures in order to satisfy the borrow checker:
        //
        // ~~~rust
        // let index = self.index;
        // self.ranges.iter()
        //     .skip_while(|fr| fr.range.contains(index))
        //     .take_while(|fr| fr.range.contains(index))
        // ~~~
        RangeIterator {
            iter: self.ranges.iter(),
            index: self.index,
            is_first: true,
        }
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

