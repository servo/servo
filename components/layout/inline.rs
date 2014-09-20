/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

#![deny(unsafe_block)]

use css::node_style::StyledNode;
use context::LayoutContext;
use floats::{FloatLeft, Floats, PlacementInfo};
use flow::{BaseFlow, FlowClass, Flow, InlineFlowClass};
use flow;
use layout_debug;
use fragment::{Fragment, InlineBlockFragment, ScannedTextFragment, ScannedTextFragmentInfo, SplitInfo};
use model::IntrinsicISizes;
use text;
use wrapper::ThreadSafeLayoutNode;

use collections::{Deque, RingBuf};
use geom::Rect;
use gfx::display_list::ContentLevel;
use gfx::font::FontMetrics;
use gfx::font_context::FontContext;
use geom::Size2D;
use gfx::text::glyph::CharIndex;
use servo_util::geometry::Au;
use servo_util::logical_geometry::{LogicalRect, LogicalSize};
use servo_util::range;
use servo_util::range::{EachIndex, Range, RangeIndex, IntRangeIndex};
use std::cmp::max;
use std::fmt;
use std::mem;
use std::num;
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
/// with a float or a horizontal wall of the containing block. The block-start
/// inline-start corner of the green zone is the same as that of the line, but
/// the green zone can be taller and wider than the line itself.
#[deriving(Encodable)]
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
    /// ~~~text
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
    /// | [0.0, 1.4) | [1.5, 2.0)  | [2.0, 3.4)  | [3.4, 4.0) |
    /// |------------|-------------|-------------|------------|
    /// | 'I like'   | 'truffles,' | '<img> yes' | 'I do.'    |
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
    /// ~~~text
    /// +-----------------------------------------------------------+
    /// |               ^                                           |
    /// |               |                                           |
    /// |            origin.y                                       |
    /// |               |                                           |
    /// |               v                                           |
    /// |< - origin.x ->+ - - - - - - - - +---------+----           |
    /// |               |                 |         |   ^           |
    /// |               |                 |  <img>  |  size.block-size  |
    /// |               I like truffles,  |         |   v           |
    /// |               + - - - - - - - - +---------+----           |
    /// |               |                           |               |
    /// |               |<------ size.inline-size ------->|               |
    /// |                                                           |
    /// |                                                           |
    /// +-----------------------------------------------------------+
    /// ~~~
    pub bounds: LogicalRect<Au>,
    /// The green zone is the greatest extent from wich a line can extend to
    /// before it collides with a float.
    ///
    /// ~~~text
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
    pub green_zone: LogicalSize<Au>
}

int_range_index! {
    #[deriving(Encodable)]
    #[doc = "The index of a fragment in a flattened vector of DOM elements."]
    struct FragmentIndex(int)
}

/// A line index consists of two indices: a fragment index that refers to the
/// index of a DOM fragment within a flattened inline element; and a glyph index
/// where the 0th glyph refers to the first glyph of that fragment.
#[deriving(Clone, Encodable, PartialEq, PartialOrd, Eq, Ord, Zero)]
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
    /// |  0   |        1         |    2    |       3      |
    /// |------|------------------|---------|--------------|
    /// | 'I ' | 'like truffles,' | `<img>` | ' yes I do.' |
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
    /// | 0 | 1 | 0 | 1 | 2 | 3 | 4 | 5 | 6 | 7 | 8 | 9 | 10 | 11 | 12 | 13 | 14 |
    /// |---|---|---|---|---|---|---|---|---|---|---|---|----|----|----|----|----|
    /// | I |   | l | i | k | e |   | t | r | u | f | f | l  | e  | s  | ,  |    |
    ///
    /// |    0    | 0 | 1 | 2 | 3 | 4 | 5 | 6 | 7 | 8 | 9 |
    /// |---------|---|---|---|---|---|---|---|---|---|---|
    /// | `<img>` |   | y | e | s |   | I |   | d | o | . |
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
    pub cur_b: Au,  // Current position on the block direction
}

impl LineBreaker {
    pub fn new(float_ctx: Floats) -> LineBreaker {
        LineBreaker {
            new_fragments: Vec::new(),
            work_list: RingBuf::new(),
            pending_line: Line {
                range: Range::empty(),
                bounds: LogicalRect::zero(float_ctx.writing_mode),
                green_zone: LogicalSize::zero(float_ctx.writing_mode)
            },
            floats: float_ctx,
            lines: Vec::new(),
            cur_b: Au::new(0)
        }
    }

    pub fn floats(&mut self) -> Floats {
        self.floats.clone()
    }

    fn reset_scanner(&mut self) {
        debug!("Resetting LineBreaker's state for flow.");
        self.lines = Vec::new();
        self.new_fragments = Vec::new();
        self.cur_b = Au(0);
        self.reset_line();
    }

    fn reset_line(&mut self) {
        self.pending_line.range.reset(num::zero(), num::zero());
        self.pending_line.bounds = LogicalRect::new(
            self.floats.writing_mode, Au::new(0), self.cur_b, Au::new(0), Au::new(0));
        self.pending_line.green_zone = LogicalSize::zero(self.floats.writing_mode)
    }

    pub fn scan_for_lines(&mut self, flow: &mut InlineFlow, layout_context: &LayoutContext) {
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
                    white_space::normal => self.try_append_to_line(cur_fragment, flow, layout_context),
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
                debug!("LineBreaker: Partially full line {:u} inline_start at end of scanning.",
                        self.lines.len());
                self.flush_current_line();
            }
        }

        old_fragments.fragments = mem::replace(&mut self.new_fragments, vec![]);
        flow.fragments = old_fragments;
        flow.lines = mem::replace(&mut self.lines, Vec::new());
    }

    fn flush_current_line(&mut self) {
        debug!("LineBreaker: Flushing line {:u}: {:?}",
               self.lines.len(), self.pending_line);

        // clear line and add line mapping
        debug!("LineBreaker: Saving information for flushed line {:u}.", self.lines.len());
        self.lines.push(self.pending_line);
        self.cur_b = self.pending_line.bounds.start.b + self.pending_line.bounds.size.block;
        self.reset_line();
    }

    // FIXME(eatkinson): this assumes that the tallest fragment in the line determines the line block-size
    // This might not be the case with some weird text fonts.
    fn new_block_size_for_line(&self, new_fragment: &Fragment, layout_context: &LayoutContext) -> Au {
        let fragment_block_size = new_fragment.content_block_size(layout_context);
        if fragment_block_size > self.pending_line.bounds.size.block {
            fragment_block_size
        } else {
            self.pending_line.bounds.size.block
        }
    }

    /// Computes the position of a line that has only the provided fragment. Returns the bounding
    /// rect of the line's green zone (whose origin coincides with the line's origin) and the actual
    /// inline-size of the first fragment after splitting.
    fn initial_line_placement(&self, first_fragment: &Fragment, ceiling: Au, flow: &InlineFlow)
                              -> (LogicalRect<Au>, Au) {
        debug!("LineBreaker: Trying to place first fragment of line {}", self.lines.len());

        let first_fragment_size = first_fragment.border_box.size;
        let splittable = first_fragment.can_split();
        debug!("LineBreaker: fragment size: {}, splittable: {}", first_fragment_size, splittable);

        // Initally, pretend a splittable fragment has 0 inline-size.
        // We will move it later if it has nonzero inline-size
        // and that causes problems.
        let placement_inline_size = if splittable {
            Au::new(0)
        } else {
            first_fragment_size.inline
        };

        let info = PlacementInfo {
            size: LogicalSize::new(
                self.floats.writing_mode, placement_inline_size, first_fragment_size.block),
            ceiling: ceiling,
            max_inline_size: flow.base.position.size.inline,
            kind: FloatLeft,
        };

        let line_bounds = self.floats.place_between_floats(&info);

        debug!("LineBreaker: found position for line: {} using placement_info: {:?}",
               line_bounds,
               info);

        // Simple case: if the fragment fits, then we can stop here
        if line_bounds.size.inline > first_fragment_size.inline {
            debug!("LineBreaker: case=fragment fits");
            return (line_bounds, first_fragment_size.inline);
        }

        // If not, but we can't split the fragment, then we'll place
        // the line here and it will overflow.
        if !splittable {
            debug!("LineBreaker: case=line doesn't fit, but is unsplittable");
            return (line_bounds, first_fragment_size.inline);
        }

        debug!("LineBreaker: used to call split_to_inline_size here");
        return (line_bounds, first_fragment_size.inline);
    }

    /// Performs float collision avoidance. This is called when adding a fragment is going to increase
    /// the block-size, and because of that we will collide with some floats.
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
                    new_block_size: Au,
                    line_is_empty: bool)
                    -> bool {
        debug!("LineBreaker: entering float collision avoider!");

        // First predict where the next line is going to be.
        let this_line_y = self.pending_line.bounds.start.b;
        let (next_line, first_fragment_inline_size) = self.initial_line_placement(&in_fragment, this_line_y, flow);
        let next_green_zone = next_line.size;

        let new_inline_size = self.pending_line.bounds.size.inline + first_fragment_inline_size;

        // Now, see if everything can fit at the new location.
        if next_green_zone.inline >= new_inline_size && next_green_zone.block >= new_block_size {
            debug!("LineBreaker: case=adding fragment collides vertically with floats: moving line");

            self.pending_line.bounds.start = next_line.start;
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

            let (inline_start, inline_end, run) = in_fragment.find_split_info_by_new_line()
                .expect("LineBreaker: This split case makes no sense!");
            let writing_mode = self.floats.writing_mode;

            // TODO(bjz): Remove fragment splitting
            let split_fragment = |split: SplitInfo| {
                let info = ScannedTextFragmentInfo::new(run.clone(), split.range);
                let specific = ScannedTextFragment(info);
                let size = LogicalSize::new(
                    writing_mode, split.inline_size, in_fragment.border_box.size.block);
                in_fragment.transform(size, specific)
            };

            debug!("LineBreaker: Pushing the fragment to the inline_start of the new-line character \
                   to the line.");
            let mut inline_start = split_fragment(inline_start);
            inline_start.new_line_pos = vec![];
            self.push_fragment_to_line(inline_start);

            for inline_end in inline_end.move_iter() {
                debug!("LineBreaker: Deferring the fragment to the inline_end of the new-line \
                       character to the line.");
                let mut inline_end = split_fragment(inline_end);
                inline_end.new_line_pos.remove(0);
                self.work_list.push_front(inline_end);
            }
            false
        }
    }

    /// Tries to append the given fragment to the line, splitting it if necessary. Returns false only if
    /// we should break the line.
    fn try_append_to_line(&mut self, in_fragment: Fragment, flow: &InlineFlow, layout_context: &LayoutContext) -> bool {
        let line_is_empty = self.pending_line.range.length() == num::zero();
        if line_is_empty {
            let (line_bounds, _) = self.initial_line_placement(&in_fragment, self.cur_b, flow);
            self.pending_line.bounds.start = line_bounds.start;
            self.pending_line.green_zone = line_bounds.size;
        }

        debug!("LineBreaker: Trying to append fragment to line {:u} (fragment size: {}, green zone: \
                {}): {}",
               self.lines.len(),
               in_fragment.border_box.size,
               self.pending_line.green_zone,
               in_fragment);

        let green_zone = self.pending_line.green_zone;

        // NB: At this point, if `green_zone.inline-size < self.pending_line.bounds.size.inline-size` or
        // `green_zone.block-size < self.pending_line.bounds.size.block-size`, then we committed a line
        // that overlaps with floats.

        let new_block_size = self.new_block_size_for_line(&in_fragment, layout_context);
        if new_block_size > green_zone.block {
            // Uh-oh. Float collision imminent. Enter the float collision avoider
            return self.avoid_floats(in_fragment, flow, new_block_size, line_is_empty)
        }

        // If we're not going to overflow the green zone vertically, we might still do so
        // horizontally. We'll try to place the whole fragment on this line and break somewhere if it
        // doesn't fit.

        let new_inline_size = self.pending_line.bounds.size.inline + in_fragment.border_box.size.inline;
        if new_inline_size <= green_zone.inline {
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

        let available_inline_size = green_zone.inline - self.pending_line.bounds.size.inline;
        let split = in_fragment.find_split_info_for_inline_size(CharIndex(0), available_inline_size, line_is_empty);
        match split.map(|(inline_start, inline_end, run)| {
            // TODO(bjz): Remove fragment splitting
            let split_fragment = |split: SplitInfo| {
                let info = ScannedTextFragmentInfo::new(run.clone(), split.range);
                let specific = ScannedTextFragment(info);
                let size = LogicalSize::new(
                    self.floats.writing_mode, split.inline_size, in_fragment.border_box.size.block);
                in_fragment.transform(size, specific)
            };

            (inline_start.map(|x| { debug!("LineBreaker: Left split {}", x); split_fragment(x) }),
             inline_end.map(|x| { debug!("LineBreaker: Right split {}", x); split_fragment(x) }))
        }) {
            None => {
                debug!("LineBreaker: Tried to split unsplittable render fragment! Deferring to next \
                       line. {}", in_fragment);
                self.work_list.push_front(in_fragment);
                false
            },
            Some((Some(inline_start_fragment), Some(inline_end_fragment))) => {
                debug!("LineBreaker: Line break found! Pushing inline_start fragment to line and deferring \
                       inline_end fragment to next line.");
                self.push_fragment_to_line(inline_start_fragment);
                self.work_list.push_front(inline_end_fragment);
                true
            },
            Some((Some(inline_start_fragment), None)) => {
                debug!("LineBreaker: Pushing inline_start fragment to line.");
                self.push_fragment_to_line(inline_start_fragment);
                true
            },
            Some((None, Some(inline_end_fragment))) => {
                debug!("LineBreaker: Pushing inline_end fragment to line.");
                self.push_fragment_to_line(inline_end_fragment);
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
        self.pending_line.bounds.size.inline = self.pending_line.bounds.size.inline +
            fragment.border_box.size.inline;
        self.pending_line.bounds.size.block = Au::max(self.pending_line.bounds.size.block,
                                                       fragment.border_box.size.block);
        self.new_fragments.push(fragment);
    }
}

/// Represents a list of inline fragments, including element ranges.
#[deriving(Encodable)]
pub struct InlineFragments {
    /// The fragments themselves.
    pub fragments: Vec<Fragment>,
}

impl InlineFragments {
    /// Creates an empty set of inline fragments.
    pub fn new() -> InlineFragments {
        InlineFragments {
            fragments: vec![],
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
    pub fn push(&mut self, fragment: &mut Fragment, style: Arc<ComputedValues>) {
        fragment.add_inline_context_style(style);
        self.fragments.push(fragment.clone());
    }

    /// Merges another set of inline fragments with this one.
    pub fn push_all(&mut self, fragments: InlineFragments) {
        self.fragments.push_all_move(fragments.fragments);
    }

    /// A convenience function to return the fragment at a given index.
    pub fn get<'a>(&'a self, index: uint) -> &'a Fragment {
        &self.fragments[index]
    }

    /// A convenience function to return a mutable reference to the fragment at a given index.
    pub fn get_mut<'a>(&'a mut self, index: uint) -> &'a mut Fragment {
        self.fragments.get_mut(index)
    }

    /// Strips ignorable whitespace from the start of a list of fragments.
    pub fn strip_ignorable_whitespace_from_start(&mut self) {
        if self.is_empty() { return }; // Fast path

        // FIXME (rust#16151): This can be reverted back to using skip_while once
        // the upstream bug is fixed.
        let mut fragments = mem::replace(&mut self.fragments, vec![]).move_iter();
        let mut new_fragments = Vec::new();
        let mut skipping = true;
        for fragment in fragments {
            if skipping && fragment.is_whitespace_only() {
                debug!("stripping ignorable whitespace from start");
                continue
            }

            skipping = false;
            new_fragments.push(fragment);
        }

        self.fragments = new_fragments;
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


        self.fragments = new_fragments;
    }
}

/// Flows for inline layout.
#[deriving(Encodable)]
pub struct InlineFlow {
    /// Data common to all flows.
    pub base: BaseFlow,

    /// A vector of all inline fragments. Several fragments may correspond to one node/element.
    pub fragments: InlineFragments,

    /// A vector of ranges into fragments that represents line positions. These ranges are disjoint and
    /// are the result of inline layout. This also includes some metadata used for positioning
    /// lines.
    pub lines: Vec<Line>,

    /// The minimum block-size above the baseline for each line, as specified by the line block-size and
    /// font style.
    pub minimum_block_size_above_baseline: Au,

    /// The minimum depth below the baseline for each line, as specified by the line block-size and
    /// font style.
    pub minimum_depth_below_baseline: Au,
}

impl InlineFlow {
    pub fn from_fragments(node: ThreadSafeLayoutNode, fragments: InlineFragments) -> InlineFlow {
        InlineFlow {
            base: BaseFlow::new(node),
            fragments: fragments,
            lines: Vec::new(),
            minimum_block_size_above_baseline: Au(0),
            minimum_depth_below_baseline: Au(0),
        }
    }

    pub fn build_display_list_inline(&mut self, layout_context: &LayoutContext) {
        let size = self.base.position.size.to_physical(self.base.writing_mode);
        if !Rect(self.base.abs_position, size).intersects(&layout_context.shared.dirty) {
            return
        }

        // TODO(#228): Once we form lines and have their cached bounds, we can be smarter and
        // not recurse on a line if nothing in it can intersect the dirty region.
        debug!("Flow: building display list for {:u} inline fragments", self.fragments.len());

        for fragment in self.fragments.fragments.mut_iter() {
            let rel_offset = fragment.relative_position(&self.base
                                                             .absolute_position_info
                                                             .relative_containing_block_size);
            let mut accumulator = fragment.build_display_list(&mut self.base.display_list,
                                             layout_context,
                                             self.base.abs_position.add_size(
                                                &rel_offset.to_physical(self.base.writing_mode)),
                                             ContentLevel);
            match fragment.specific {
                InlineBlockFragment(ref mut block_flow) => {
                    let block_flow = block_flow.flow_ref.get_mut();
                    accumulator.push_child(&mut self.base.display_list, block_flow);
                }
                _ => {}
            }
        }

        // TODO(#225): Should `inline-block` elements have flows as children of the inline flow or
        // should the flow be nested inside the fragment somehow?

        // For now, don't traverse the subtree rooted here.
    }

    /// Returns the distance from the baseline for the logical block-start inline-start corner of this fragment,
    /// taking into account the value of the CSS `vertical-align` property. Negative values mean
    /// "toward the logical block-start" and positive values mean "toward the logical block-end".
    ///
    /// The extra boolean is set if and only if `biggest_block-start` and/or `biggest_block-end` were updated.
    /// That is, if the box has a `block-start` or `block-end` value, true is returned.
    fn distance_from_baseline(fragment: &Fragment,
                              ascent: Au,
                              parent_text_block_start: Au,
                              parent_text_block_end: Au,
                              block_size_above_baseline: &mut Au,
                              depth_below_baseline: &mut Au,
                              largest_block_size_for_top_fragments: &mut Au,
                              largest_block_size_for_bottom_fragments: &mut Au,
                              layout_context: &LayoutContext)
                              -> (Au, bool) {
        match fragment.vertical_align() {
            vertical_align::baseline => (-ascent, false),
            vertical_align::middle => {
                // TODO: x-block-size value should be used from font info.
                let xblock_size = Au(0);
                let fragment_block_size = fragment.content_block_size(layout_context);
                let offset_block_start = -(xblock_size + fragment_block_size).scale_by(0.5);
                *block_size_above_baseline = offset_block_start.scale_by(-1.0);
                *depth_below_baseline = fragment_block_size - *block_size_above_baseline;
                (offset_block_start, false)
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
                let fragment_block_size = *block_size_above_baseline + *depth_below_baseline;
                let prev_depth_below_baseline = *depth_below_baseline;
                *block_size_above_baseline = parent_text_block_start;
                *depth_below_baseline = fragment_block_size - *block_size_above_baseline;
                (*depth_below_baseline - prev_depth_below_baseline - ascent, false)
            },
            vertical_align::text_bottom => {
                let fragment_block_size = *block_size_above_baseline + *depth_below_baseline;
                let prev_depth_below_baseline = *depth_below_baseline;
                *depth_below_baseline = parent_text_block_end;
                *block_size_above_baseline = fragment_block_size - *depth_below_baseline;
                (*depth_below_baseline - prev_depth_below_baseline - ascent, false)
            },
            vertical_align::top => {
                *largest_block_size_for_top_fragments =
                    Au::max(*largest_block_size_for_top_fragments,
                            *block_size_above_baseline + *depth_below_baseline);
                let offset_top = *block_size_above_baseline - ascent;
                (offset_top, true)
            },
            vertical_align::bottom => {
                *largest_block_size_for_bottom_fragments =
                    Au::max(*largest_block_size_for_bottom_fragments,
                            *block_size_above_baseline + *depth_below_baseline);
                let offset_bottom = -(*depth_below_baseline + ascent);
                (offset_bottom, true)
            },
            vertical_align::Length(length) => (-(length + ascent), false),
            vertical_align::Percentage(p) => {
                let line_height = fragment.calculate_line_height(layout_context);
                let percent_offset = line_height.scale_by(p);
                (-(percent_offset + ascent), false)
            }
        }
    }

    /// Sets fragment X positions based on alignment for one line.
    fn set_horizontal_fragment_positions(fragments: &mut InlineFragments,
                                         line: &Line,
                                         line_align: text_align::T) {
        // Figure out how much inline-size we have.
        let slack_inline_size = Au::max(Au(0), line.green_zone.inline - line.bounds.size.inline);

        // Set the fragment x positions based on that alignment.
        let mut offset_x = line.bounds.start.i;
        offset_x = offset_x + match line_align {
            // So sorry, but justified text is more complicated than shuffling line
            // coordinates.
            //
            // TODO(burg, issue #213): Implement `text-align: justify`.
            text_align::left | text_align::justify => Au(0),
            text_align::center => slack_inline_size.scale_by(0.5),
            text_align::right => slack_inline_size,
        };

        for i in each_fragment_index(&line.range) {
            let fragment = fragments.get_mut(i.to_uint());
            let size = fragment.border_box.size;
            fragment.border_box = LogicalRect::new(
                fragment.style.writing_mode, offset_x, fragment.border_box.start.b,
                size.inline, size.block);
            offset_x = offset_x + size.inline;
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
        let line_height = text::line_height_from_style(style, &font_metrics);
        let inline_metrics = InlineMetrics::from_font_metrics(&font_metrics, line_height);
        (inline_metrics.block_size_above_baseline, inline_metrics.depth_below_baseline)
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

    fn bubble_inline_sizes(&mut self, _: &LayoutContext) {
        let _scope = layout_debug_scope!("inline::bubble_inline_sizes {:s}", self.base.debug_id());

        let writing_mode = self.base.writing_mode;
        for kid in self.base.child_iter() {
            flow::mut_base(kid).floats = Floats::new(writing_mode);
        }

        let mut intrinsic_inline_sizes = IntrinsicISizes::new();
        for fragment in self.fragments.fragments.mut_iter() {
            debug!("Flow: measuring {}", *fragment);

            let fragment_intrinsic_inline_sizes =
                fragment.intrinsic_inline_sizes();
            intrinsic_inline_sizes.minimum_inline_size = max(
                intrinsic_inline_sizes.minimum_inline_size,
                fragment_intrinsic_inline_sizes.minimum_inline_size);
            intrinsic_inline_sizes.preferred_inline_size =
                intrinsic_inline_sizes.preferred_inline_size +
                fragment_intrinsic_inline_sizes.preferred_inline_size;
        }

        self.base.intrinsic_inline_sizes = intrinsic_inline_sizes;
    }

    /// Recursively (top-down) determines the actual inline-size of child contexts and fragments. When called
    /// on this context, the context has had its inline-size set by the parent context.
    fn assign_inline_sizes(&mut self, _: &LayoutContext) {
        let _scope = layout_debug_scope!("inline::assign_inline_sizes {:s}", self.base.debug_id());

        // Initialize content fragment inline-sizes if they haven't been initialized already.
        //
        // TODO: Combine this with `LineBreaker`'s walk in the fragment list, or put this into `Fragment`.

        debug!("InlineFlow::assign_inline_sizes: floats in: {:?}", self.base.floats);

        {
            let inline_size = self.base.position.size.inline;
            let this = &mut *self;
            for fragment in this.fragments.fragments.mut_iter() {
                fragment.assign_replaced_inline_size_if_necessary(inline_size);
            }
        }

        // If there are any inline-block kids, propagate explicit block sizes down to them.
        let block_container_explicit_block_size = self.base.block_container_explicit_block_size;
        for kid in self.base.child_iter() {
            flow::mut_base(kid).block_container_explicit_block_size =
                block_container_explicit_block_size;
        }
    }

    /// Calculate and set the block-size of this flow. See CSS 2.1 § 10.6.1.
    fn assign_block_size(&mut self, ctx: &LayoutContext) {
        let _scope = layout_debug_scope!("inline::assign_block_size {:s}", self.base.debug_id());

        // Divide the fragments into lines.
        //
        // TODO(#226): Get the CSS `line-block-size` property from the containing block's style to
        // determine minimum line block-size.
        //
        // TODO(#226): Get the CSS `line-block-size` property from each non-replaced inline element to
        // determine its block-size for computing line block-size.
        //
        // TODO(pcwalton): Cache the line scanner?
        debug!("assign_block_size_inline: floats in: {:?}", self.base.floats);

        // assign block-size for inline fragments
        for fragment in self.fragments.fragments.mut_iter() {
            fragment.assign_replaced_block_size_if_necessary();
        }

        let scanner_floats = self.base.floats.clone();
        let mut scanner = LineBreaker::new(scanner_floats);
        scanner.scan_for_lines(self, ctx);

        // All lines use text alignment of the flow.
        let text_align = self.base.flags.text_align();

        // Now, go through each line and lay out the fragments inside.
        let mut line_distance_from_flow_block_start = Au(0);
        for line in self.lines.mut_iter() {
            // Lay out fragments horizontally.
            InlineFlow::set_horizontal_fragment_positions(&mut self.fragments, line, text_align);

            // Set the block-start y position of the current line.
            // `line_height_offset` is updated at the end of the previous loop.
            line.bounds.start.b = line_distance_from_flow_block_start;

            // Calculate the distance from the baseline to the block-start and block-end of the line.
            let mut largest_block_size_above_baseline = self.minimum_block_size_above_baseline;
            let mut largest_depth_below_baseline = self.minimum_depth_below_baseline;

            // Calculate the largest block-size among fragments with 'top' and 'bottom' values
            // respectively.
            let (mut largest_block_size_for_top_fragments, mut largest_block_size_for_bottom_fragments) =
                (Au(0), Au(0));

            for fragment_i in each_fragment_index(&line.range) {
                let fragment = self.fragments.fragments.get_mut(fragment_i.to_uint());

                let InlineMetrics {
                    block_size_above_baseline: mut block_size_above_baseline,
                    depth_below_baseline: mut depth_below_baseline,
                    ascent
                } = fragment.inline_metrics(ctx);

                // To calculate text-top and text-bottom value when `vertical-align` is involved,
                // we should find the top and bottom of the content area of the parent fragment.
                // "Content area" is defined in CSS 2.1 § 10.6.1.
                //
                // TODO: We should extract em-box info from the font size of the parent and
                // calculate the distances from the baseline to the block-start and the block-end of the
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

                // Calculate the final block-size above the baseline for this fragment.
                //
                // The no-update flag decides whether `largest_block-size_for_top_fragments` and
                // `largest_block-size_for_bottom_fragments` are to be updated or not. This will be set
                // if and only if the fragment has `vertical-align` set to `top` or `bottom`.
                let (distance_from_baseline, no_update_flag) =
                    InlineFlow::distance_from_baseline(
                        fragment,
                        ascent,
                        parent_text_top,
                        parent_text_bottom,
                        &mut block_size_above_baseline,
                        &mut depth_below_baseline,
                        &mut largest_block_size_for_top_fragments,
                        &mut largest_block_size_for_bottom_fragments,
                        ctx);

                // Unless the current fragment has `vertical-align` set to `top` or `bottom`,
                // `largest_block-size_above_baseline` and `largest_depth_below_baseline` are updated.
                if !no_update_flag {
                    largest_block_size_above_baseline = Au::max(block_size_above_baseline,
                                                            largest_block_size_above_baseline);
                    largest_depth_below_baseline = Au::max(depth_below_baseline,
                                                           largest_depth_below_baseline);
                }

                // Temporarily use `fragment.border_box.start.b` to mean "the distance from the
                // baseline". We will assign the real value later.
                fragment.border_box.start.b = distance_from_baseline
            }

            // Calculate the distance from the baseline to the top of the largest fragment with a
            // value for `bottom`. Then, if necessary, update `largest_block-size_above_baseline`.
            largest_block_size_above_baseline =
                Au::max(largest_block_size_above_baseline,
                        largest_block_size_for_bottom_fragments - largest_depth_below_baseline);

            // Calculate the distance from baseline to the bottom of the largest fragment with a value
            // for `top`. Then, if necessary, update `largest_depth_below_baseline`.
            largest_depth_below_baseline =
                Au::max(largest_depth_below_baseline,
                        largest_block_size_for_top_fragments - largest_block_size_above_baseline);

            // Now, the distance from the logical block-start of the line to the baseline can be
            // computed as `largest_block-size_above_baseline`.
            let baseline_distance_from_block_start = largest_block_size_above_baseline;

            // Compute the final positions in the block direction of each fragment. Recall that
            // `fragment.border_box.start.b` was set to the distance from the baseline above.
            for fragment_i in each_fragment_index(&line.range) {
                let fragment = self.fragments.get_mut(fragment_i.to_uint());
                match fragment.vertical_align() {
                    vertical_align::top => {
                        fragment.border_box.start.b = fragment.border_box.start.b +
                            line_distance_from_flow_block_start
                    }
                    vertical_align::bottom => {
                        fragment.border_box.start.b = fragment.border_box.start.b +
                            line_distance_from_flow_block_start + baseline_distance_from_block_start +
                            largest_depth_below_baseline
                    }
                    _ => {
                        fragment.border_box.start.b = fragment.border_box.start.b +
                            line_distance_from_flow_block_start + baseline_distance_from_block_start
                    }
                }
            }

            // This is used to set the block-start y position of the next line in the next loop.
            line.bounds.size.block = largest_block_size_above_baseline + largest_depth_below_baseline;
            line_distance_from_flow_block_start = line_distance_from_flow_block_start + line.bounds.size.block;
        } // End of `lines.each` loop.

        self.base.position.size.block = match self.lines.as_slice().last() {
            Some(ref last_line) => last_line.bounds.start.b + last_line.bounds.size.block,
            None => Au::new(0)
        };

        self.base.floats = scanner.floats();
        self.base.floats.translate(LogicalSize::new(
            self.base.writing_mode, Au::new(0), -self.base.position.size.block));
    }

    fn compute_absolute_position(&mut self) {
        for f in self.fragments.fragments.mut_iter() {
            match f.specific {
                InlineBlockFragment(ref mut info) => {
                    let block_flow = info.flow_ref.get_mut().as_block();

                    // FIXME(#2795): Get the real container size
                    let container_size = Size2D::zero();
                    block_flow.base.abs_position = self.base.abs_position +
                                                    f.border_box.start.to_physical(self.base.writing_mode, container_size);
                }
                _ => {}
            }
        }
    }
}

impl fmt::Show for InlineFlow {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        try!(write!(f, "InlineFlow"));
        for (i, fragment) in self.fragments.fragments.iter().enumerate() {
            if i == 0 {
                try!(write!(f, ": {}", fragment))
            } else {
                try!(write!(f, ", {}", fragment))
            }
        }
        Ok(())
    }
}

#[deriving(Clone)]
pub struct InlineFragmentContext {
    pub styles: Vec<Arc<ComputedValues>>,
}

impl InlineFragmentContext {
    pub fn new() -> InlineFragmentContext {
        InlineFragmentContext {
            styles: vec!()
        }
    }
}

/// BSize above the baseline, depth below the baseline, and ascent for a fragment. See CSS 2.1 §
/// 10.8.1.
pub struct InlineMetrics {
    pub block_size_above_baseline: Au,
    pub depth_below_baseline: Au,
    pub ascent: Au,
}

impl InlineMetrics {
    /// Calculates inline metrics from font metrics and line block-size per CSS 2.1 § 10.8.1.
    #[inline]
    pub fn from_font_metrics(font_metrics: &FontMetrics, line_height: Au) -> InlineMetrics {
        let leading = line_height - (font_metrics.ascent + font_metrics.descent);
        InlineMetrics {
            block_size_above_baseline: font_metrics.ascent + leading.scale_by(0.5),
            depth_below_baseline: font_metrics.descent + leading.scale_by(0.5),
            ascent: font_metrics.ascent,
        }
    }

    /// Calculates inline metrics from font metrics and line block-size per CSS 2.1 § 10.8.1.
    #[inline]
    pub fn from_block_height(font_metrics: &FontMetrics, block_height: Au) -> InlineMetrics {
        let leading = block_height - (font_metrics.ascent + font_metrics.descent);
        InlineMetrics {
            block_size_above_baseline: font_metrics.ascent + leading.scale_by(0.5),
            depth_below_baseline: font_metrics.descent + leading.scale_by(0.5),
            ascent: font_metrics.ascent + leading.scale_by(0.5),
        }
    }
}

