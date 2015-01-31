/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

#![deny(unsafe_blocks)]

use css::node_style::StyledNode;
use context::LayoutContext;
use display_list_builder::{FragmentDisplayListBuilding, InlineFlowDisplayListBuilding};
use floats::{FloatKind, Floats, PlacementInfo};
use flow::{BaseFlow, FlowClass, Flow, MutableFlowUtils, ForceNonfloatedFlag};
use flow::{IS_ABSOLUTELY_POSITIONED};
use flow;
use fragment::{CoordinateSystem, Fragment, FragmentBorderBoxIterator, ScannedTextFragmentInfo};
use fragment::{SpecificFragmentInfo};
use fragment::SplitInfo;
use incremental::{REFLOW, REFLOW_OUT_OF_FLOW};
use layout_debug;
use model::IntrinsicISizesContribution;
use text;

use collections::{RingBuf};
use geom::{Point2D, Rect};
use gfx::font::FontMetrics;
use gfx::font_context::FontContext;
use gfx::text::glyph::CharIndex;
use servo_util::arc_ptr_eq;
use servo_util::geometry::{Au, ZERO_RECT};
use servo_util::logical_geometry::{LogicalRect, LogicalSize, WritingMode};
use servo_util::range::{Range, RangeIndex};
use std::cmp::max;
use std::fmt;
use std::mem;
use std::num::ToPrimitive;
use std::ops::{Add, Sub, Mul, Div, Rem, Neg, Shl, Shr, Not, BitOr, BitAnd, BitXor};
use std::u16;
use style::computed_values::{overflow, text_align, text_justify, text_overflow, vertical_align};
use style::computed_values::{white_space};
use style::properties::ComputedValues;
use std::sync::Arc;

// From gfxFontConstants.h in Firefox
static FONT_SUBSCRIPT_OFFSET_RATIO: f64 = 0.20;
static FONT_SUPERSCRIPT_OFFSET_RATIO: f64 = 0.34;

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
#[derive(RustcEncodable, Debug, Copy)]
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
    /// | [0, 2)   | [2, 3)      | [3, 4)      | [4, 5)   |
    /// |----------|-------------|-------------|----------|
    /// | 'I like' | 'truffles,' | '<img> yes' | 'I do.'  |
    pub range: Range<FragmentIndex>,

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
    /// |               |                 |  <img>  |  size.block   |
    /// |               I like truffles,  |         |   v           |
    /// |               + - - - - - - - - +---------+----           |
    /// |               |                           |               |
    /// |               |<------ size.inline ------>|               |
    /// |                                                           |
    /// |                                                           |
    /// +-----------------------------------------------------------+
    /// ~~~
    pub bounds: LogicalRect<Au>,

    /// The green zone is the greatest extent from which a line can extend to
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
    pub green_zone: LogicalSize<Au>,
}

int_range_index! {
    #[derive(RustcEncodable)]
    #[doc = "The index of a fragment in a flattened vector of DOM elements."]
    struct FragmentIndex(int)
}

bitflags! {
    flags InlineReflowFlags: u8 {
        #[doc="The `white-space: nowrap` property from CSS 2.1 § 16.6 is in effect."]
        const NO_WRAP_INLINE_REFLOW_FLAG = 0x01
    }
}

/// Arranges fragments into lines, splitting them up as necessary.
struct LineBreaker {
    /// The floats we need to flow around.
    floats: Floats,
    /// The resulting fragment list for the flow, consisting of possibly-broken fragments.
    new_fragments: Vec<Fragment>,
    /// The next fragment or fragments that we need to work on.
    work_list: RingBuf<Fragment>,
    /// The line we're currently working on.
    pending_line: Line,
    /// The lines we've already committed.
    lines: Vec<Line>,
    /// The current position in the block direction.
    cur_b: Au,
    /// The computed value of the indentation for the first line (`text-indent`, CSS 2.1 § 16.1).
    first_line_indentation: Au,
}

impl LineBreaker {
    /// Creates a new `LineBreaker` with a set of floats and the indentation of the first line.
    fn new(float_context: Floats, first_line_indentation: Au) -> LineBreaker {
        LineBreaker {
            new_fragments: Vec::new(),
            work_list: RingBuf::new(),
            pending_line: Line {
                range: Range::empty(),
                bounds: LogicalRect::zero(float_context.writing_mode),
                green_zone: LogicalSize::zero(float_context.writing_mode)
            },
            floats: float_context,
            lines: Vec::new(),
            cur_b: Au(0),
            first_line_indentation: first_line_indentation,
        }
    }

    /// Resets the `LineBreaker` to the initial state it had after a call to `new`.
    fn reset_scanner(&mut self) {
        self.lines = Vec::new();
        self.new_fragments = Vec::new();
        self.cur_b = Au(0);
        self.reset_line();
    }

    /// Reinitializes the pending line to blank data.
    fn reset_line(&mut self) {
        self.pending_line.range.reset(FragmentIndex(0), FragmentIndex(0));
        self.pending_line.bounds = LogicalRect::new(self.floats.writing_mode,
                                                    Au(0),
                                                    self.cur_b,
                                                    Au(0),
                                                    Au(0));
        self.pending_line.green_zone = LogicalSize::zero(self.floats.writing_mode)
    }

    /// Reflows fragments for the given inline flow.
    fn scan_for_lines(&mut self, flow: &mut InlineFlow, layout_context: &LayoutContext) {
        self.reset_scanner();

        // Create our fragment iterator.
        debug!("LineBreaker: scanning for lines, {} fragments", flow.fragments.len());
        let mut old_fragments = mem::replace(&mut flow.fragments, InlineFragments::new());
        let mut old_fragment_iter = old_fragments.fragments.into_iter();

        // Set up our initial line state with the clean lines from a previous reflow.
        //
        // TODO(pcwalton): This would likely be better as a list of dirty line indices. That way we
        // could resynchronize if we discover during reflow that all subsequent fragments must have
        // the same position as they had in the previous reflow. I don't know how common this case
        // really is in practice, but it's probably worth handling.
        self.lines = mem::replace(&mut flow.lines, Vec::new());
        match self.lines.as_slice().last() {
            None => {}
            Some(last_line) => {
                for _ in range(FragmentIndex(0), last_line.range.end()) {
                    self.new_fragments.push(old_fragment_iter.next().unwrap())
                }
            }
        }

        // Do the reflow.
        self.reflow_fragments(old_fragment_iter, flow, layout_context);

        // Place the fragments back into the flow.
        old_fragments.fragments = mem::replace(&mut self.new_fragments, vec![]);
        flow.fragments = old_fragments;
        flow.lines = mem::replace(&mut self.lines, Vec::new());
    }

    /// Reflows the given fragments, which have been plucked out of the inline flow.
    fn reflow_fragments<'a,I>(&mut self,
                              mut old_fragment_iter: I,
                              flow: &'a InlineFlow,
                              layout_context: &LayoutContext)
                              where I: Iterator<Item=Fragment> {
        loop {
            // Acquire the next fragment to lay out from the work list or fragment list, as
            // appropriate.
            let fragment = match self.next_unbroken_fragment(&mut old_fragment_iter) {
                None => break,
                Some(fragment) => fragment,
            };

            // Set up our reflow flags.
            let flags = match fragment.style().get_inheritedtext().white_space {
                white_space::T::normal => InlineReflowFlags::empty(),
                white_space::T::pre | white_space::T::nowrap => NO_WRAP_INLINE_REFLOW_FLAG,
            };

            // Try to append the fragment, and commit the line (so we can try again with the next
            // line) if we couldn't.
            match fragment.style().get_inheritedtext().white_space {
                white_space::T::normal | white_space::T::nowrap => {
                    if !self.append_fragment_to_line_if_possible(fragment,
                                                                 flow,
                                                                 layout_context,
                                                                 flags) {
                        self.flush_current_line()
                    }
                }
                white_space::T::pre => {
                    // FIXME(pcwalton): Surely we can unify
                    // `append_fragment_to_line_if_possible` and
                    // `try_append_to_line_by_new_line` by adding another bit in the reflow
                    // flags.
                    if !self.try_append_to_line_by_new_line(layout_context, fragment) {
                        self.flush_current_line()
                    }
                }
            }
        }

        if !self.pending_line_is_empty() {
            debug!("LineBreaker: partially full line {} at end of scanning; committing it",
                    self.lines.len());
            self.flush_current_line()
        }

        // Strip trailing whitespace from the last line if necessary.
        if let Some(ref mut last_line) = self.lines.last_mut() {
            if let Some(ref mut last_fragment) = self.new_fragments.last_mut() {
                let previous_inline_size = last_line.bounds.size.inline -
                    last_fragment.border_box.size.inline;
                if last_fragment.strip_trailing_whitespace_if_necessary() {
                    last_line.bounds.size.inline = previous_inline_size +
                        last_fragment.border_box.size.inline;
                }
            }
        }
    }

    /// Acquires a new fragment to lay out from the work list or fragment list as appropriate.
    /// Note that you probably don't want to call this method directly in order to be
    /// incremental-reflow-safe; try `next_unbroken_fragment` instead.
    fn next_fragment<I>(&mut self, old_fragment_iter: &mut I) -> Option<Fragment>
                        where I: Iterator<Item=Fragment> {
        if self.work_list.is_empty() {
            return match old_fragment_iter.next() {
                None => None,
                Some(fragment) => {
                    debug!("LineBreaker: working with fragment from flow: {:?}", fragment);
                    Some(fragment)
                }
            }
        }

        debug!("LineBreaker: working with fragment from work list: {:?}", self.work_list.front());
        self.work_list.pop_front()
    }

    /// Acquires a new fragment to lay out from the work list or fragment list, merging it with any
    /// subsequent fragments as appropriate. In effect, what this method does is to return the next
    /// fragment to lay out, undoing line break operations that any previous reflows may have
    /// performed. You probably want to be using this method instead of `next_fragment`.
    fn next_unbroken_fragment<I>(&mut self, old_fragment_iter: &mut I) -> Option<Fragment>
                                 where I: Iterator<Item=Fragment> {
        let mut result = match self.next_fragment(old_fragment_iter) {
            None => return None,
            Some(fragment) => fragment,
        };

        loop {
            // FIXME(pcwalton): Yuck! I hate this `new_line_pos` stuff. Can we avoid having to do
            // this?
            result.restore_new_line_pos();

            let candidate = match self.next_fragment(old_fragment_iter) {
                None => return Some(result),
                Some(fragment) => fragment,
            };

            let need_to_merge = match (&mut result.specific, &candidate.specific) {
                (&mut SpecificFragmentInfo::ScannedText(ref mut result_info),
                 &SpecificFragmentInfo::ScannedText(ref candidate_info))
                    if arc_ptr_eq(&result_info.run, &candidate_info.run) &&
                        result_info.range.end() + CharIndex(1) == candidate_info.range.begin() => {
                    // We found a previously-broken fragment. Merge it up.
                    result_info.range.extend_by(candidate_info.range.length() + CharIndex(1));
                    true
                }
                _ => false,
            };

            if !need_to_merge {
                self.work_list.push_front(candidate);
                return Some(result)
            }
        }
    }

    /// Commits a line to the list.
    fn flush_current_line(&mut self) {
        debug!("LineBreaker: flushing line {}: {:?}", self.lines.len(), self.pending_line);
        self.lines.push(self.pending_line);
        self.cur_b = self.pending_line.bounds.start.b + self.pending_line.bounds.size.block;
        self.reset_line();
    }

    // FIXME(eatkinson): this assumes that the tallest fragment in the line determines the line
    // block-size. This might not be the case with some weird text fonts.
    fn new_block_size_for_line(&self, new_fragment: &Fragment, layout_context: &LayoutContext)
                               -> Au {
        let fragment_block_size = new_fragment.content_block_size(layout_context);
        if fragment_block_size > self.pending_line.bounds.size.block {
            fragment_block_size
        } else {
            self.pending_line.bounds.size.block
        }
    }

    /// Computes the position of a line that has only the provided fragment. Returns the bounding
    /// rect of the line's green zone (whose origin coincides with the line's origin) and the
    /// actual inline-size of the first fragment after splitting.
    fn initial_line_placement(&self,
                              flow: &InlineFlow,
                              first_fragment: &Fragment,
                              ceiling: Au)
                              -> (LogicalRect<Au>, Au) {
        debug!("LineBreaker: trying to place first fragment of line {}; fragment size: {:?}, \
                splittable: {}",
               self.lines.len(),
               first_fragment.border_box.size,
               first_fragment.can_split());

        // Initially, pretend a splittable fragment has zero inline-size. We will move it later if
        // it has nonzero inline-size and that causes problems.
        let placement_inline_size = if first_fragment.can_split() {
            Au(0)
        } else {
            first_fragment.border_box.size.inline + self.indentation_for_pending_fragment()
        };

        // Try to place the fragment between floats.
        let line_bounds = self.floats.place_between_floats(&PlacementInfo {
            size: LogicalSize::new(self.floats.writing_mode,
                                   placement_inline_size,
                                   first_fragment.border_box.size.block),
            ceiling: ceiling,
            max_inline_size: flow.base.position.size.inline,
            kind: FloatKind::Left,
        });

        // Simple case: if the fragment fits, then we can stop here.
        if line_bounds.size.inline > first_fragment.border_box.size.inline {
            debug!("LineBreaker: fragment fits on line {}", self.lines.len());
            return (line_bounds, first_fragment.border_box.size.inline);
        }

        // If not, but we can't split the fragment, then we'll place the line here and it will
        // overflow.
        if !first_fragment.can_split() {
            debug!("LineBreaker: line doesn't fit, but is unsplittable");
        }

        (line_bounds, first_fragment.border_box.size.inline)
    }

    /// Performs float collision avoidance. This is called when adding a fragment is going to
    /// increase the block-size, and because of that we will collide with some floats.
    ///
    /// We have two options here:
    /// 1) Move the entire line so that it doesn't collide any more.
    /// 2) Break the line and put the new fragment on the next line.
    ///
    /// The problem with option 1 is that we might move the line and then wind up breaking anyway,
    /// which violates the standard. But option 2 is going to look weird sometimes.
    ///
    /// So we'll try to move the line whenever we can, but break if we have to.
    ///
    /// Returns false if and only if we should break the line.
    fn avoid_floats(&mut self,
                    flow: &InlineFlow,
                    in_fragment: Fragment,
                    new_block_size: Au)
                    -> bool {
        debug!("LineBreaker: entering float collision avoider!");

        // First predict where the next line is going to be.
        let (next_line, first_fragment_inline_size) =
            self.initial_line_placement(flow,
                                        &in_fragment,
                                        self.pending_line.bounds.start.b);
        let next_green_zone = next_line.size;

        let new_inline_size = self.pending_line.bounds.size.inline + first_fragment_inline_size;

        // Now, see if everything can fit at the new location.
        if next_green_zone.inline >= new_inline_size && next_green_zone.block >= new_block_size {
            debug!("LineBreaker: case=adding fragment collides vertically with floats: moving \
                    line");

            self.pending_line.bounds.start = next_line.start;
            self.pending_line.green_zone = next_green_zone;

            debug_assert!(!self.pending_line_is_empty(), "Non-terminating line breaking");
            self.work_list.push_front(in_fragment);
            return true
        }

        debug!("LineBreaker: case=adding fragment collides vertically with floats: breaking line");
        self.work_list.push_front(in_fragment);
        false
    }

    /// Tries to append the given fragment to the line for `pre`-formatted text, splitting it if
    /// necessary. Returns true if we successfully pushed the fragment to the line or false if we
    /// couldn't.
    fn try_append_to_line_by_new_line(&mut self,
                                      layout_context: &LayoutContext,
                                      in_fragment: Fragment)
                                      -> bool {
        let should_push = match in_fragment.newline_positions() {
            None => true,
            Some(ref positions) => positions.is_empty(),
        };
        if should_push {
            debug!("LineBreaker: did not find a newline character; pushing the fragment to \
                   the line without splitting");
            self.push_fragment_to_line(layout_context, in_fragment);
            return true
        }

        debug!("LineBreaker: Found a new-line character, so splitting the line.");

        let (inline_start, inline_end, run) =
            in_fragment.find_split_info_by_new_line()
                       .expect("LineBreaker: this split case makes no sense!");
        let writing_mode = self.floats.writing_mode;

        let split_fragment = |&: split: SplitInfo| {
            let size = LogicalSize::new(writing_mode,
                                        split.inline_size,
                                        in_fragment.border_box.size.block);
            let info = box ScannedTextFragmentInfo::new(run.clone(),
                                                        split.range,
                                                        (*in_fragment.newline_positions()
                                                                     .unwrap()).clone(),
                                                        size);
            in_fragment.transform(size, SpecificFragmentInfo::ScannedText(info))
        };

        debug!("LineBreaker: Pushing the fragment to the inline_start of the new-line character \
                to the line.");
        let mut inline_start = split_fragment(inline_start);
        inline_start.save_new_line_pos();
        *inline_start.newline_positions_mut().unwrap() = vec![];
        self.push_fragment_to_line(layout_context, inline_start);

        for inline_end in inline_end.into_iter() {
            debug!("LineBreaker: Deferring the fragment to the inline_end of the new-line \
                   character to the line.");
            let mut inline_end = split_fragment(inline_end);
            inline_end.newline_positions_mut().unwrap().remove(0);
            self.work_list.push_front(inline_end);
        }

        false
    }

    /// Tries to append the given fragment to the line, splitting it if necessary. Returns true if
    /// we successfully pushed the fragment to the line or false if we couldn't.
    fn append_fragment_to_line_if_possible(&mut self,
                                           fragment: Fragment,
                                           flow: &InlineFlow,
                                           layout_context: &LayoutContext,
                                           flags: InlineReflowFlags)
                                           -> bool {
        // Determine initial placement for the fragment if we need to.
        if self.pending_line_is_empty() {
            let (line_bounds, _) = self.initial_line_placement(flow, &fragment, self.cur_b);
            self.pending_line.bounds.start = line_bounds.start;
            self.pending_line.green_zone = line_bounds.size;
        }

        debug!("LineBreaker: trying to append to line {} (fragment size: {:?}, green zone: {:?}): \
               {:?}",
               self.lines.len(),
               fragment.border_box.size,
               self.pending_line.green_zone,
               fragment);

        // NB: At this point, if `green_zone.inline < self.pending_line.bounds.size.inline` or
        // `green_zone.block < self.pending_line.bounds.size.block`, then we committed a line that
        // overlaps with floats.
        let green_zone = self.pending_line.green_zone;
        let new_block_size = self.new_block_size_for_line(&fragment, layout_context);
        if new_block_size > green_zone.block {
            // Uh-oh. Float collision imminent. Enter the float collision avoider!
            return self.avoid_floats(flow, fragment, new_block_size)
        }

        // If we're not going to overflow the green zone vertically, we might still do so
        // horizontally. We'll try to place the whole fragment on this line and break somewhere if
        // it doesn't fit.
        let indentation = self.indentation_for_pending_fragment();
        let new_inline_size = self.pending_line.bounds.size.inline +
            fragment.border_box.size.inline + indentation;
        if new_inline_size <= green_zone.inline {
            debug!("LineBreaker: fragment fits without splitting");
            self.push_fragment_to_line(layout_context, fragment);
            return true
        }

        // If we can't split the fragment or aren't allowed to because of the wrapping mode, then
        // just overflow.
        if (!fragment.can_split() && self.pending_line_is_empty()) ||
                flags.contains(NO_WRAP_INLINE_REFLOW_FLAG) {
            debug!("LineBreaker: fragment can't split and line {} is empty, so overflowing",
                    self.lines.len());
            self.push_fragment_to_line(layout_context, fragment);
            return false
        }

        // Split it up!
        let available_inline_size = green_zone.inline - self.pending_line.bounds.size.inline -
            indentation;
        let inline_start_fragment;
        let inline_end_fragment;
        let split_result = match fragment.calculate_split_position(available_inline_size,
                                                                   self.pending_line_is_empty()) {
            None => {
                debug!("LineBreaker: fragment was unsplittable; deferring to next line");
                self.work_list.push_front(fragment);
                return false
            }
            Some(split_result) => split_result,
        };

        inline_start_fragment = split_result.inline_start.as_ref().map(|x| {
            fragment.transform_with_split_info(x, split_result.text_run.clone())
        });
        inline_end_fragment = split_result.inline_end.as_ref().map(|x| {
            fragment.transform_with_split_info(x, split_result.text_run.clone())
        });

        // Push the first fragment onto the line we're working on and start off the next line with
        // the second fragment. If there's no second fragment, the next line will start off empty.
        match (inline_start_fragment, inline_end_fragment) {
            (Some(inline_start_fragment), Some(inline_end_fragment)) => {
                self.push_fragment_to_line(layout_context, inline_start_fragment);
                self.flush_current_line();
                self.work_list.push_front(inline_end_fragment)
            },
            (Some(fragment), None) => {
                self.push_fragment_to_line(layout_context, fragment);
            }
            (None, Some(_)) => debug_assert!(false, "un-normalized split result"),
            (None, None) => {}
        }

        true
    }

    /// Pushes a fragment to the current line unconditionally, possibly truncating it and placing
    /// an ellipsis based on the value of `text-overflow`.
    fn push_fragment_to_line(&mut self, layout_context: &LayoutContext, fragment: Fragment) {
        let indentation = self.indentation_for_pending_fragment();
        if self.pending_line_is_empty() {
            assert!(self.new_fragments.len() <= (u16::MAX as uint));
            self.pending_line.range.reset(FragmentIndex(self.new_fragments.len() as int),
                                          FragmentIndex(0));
        }

        // Determine if an ellipsis will be necessary to account for `text-overflow`.
        let mut need_ellipsis = false;
        let available_inline_size = self.pending_line.green_zone.inline -
            self.pending_line.bounds.size.inline - indentation;
        match (fragment.style().get_inheritedtext().text_overflow,
               fragment.style().get_box().overflow) {
            (text_overflow::T::clip, _) | (_, overflow::T::visible) => {}
            (text_overflow::T::ellipsis, _) => {
                need_ellipsis = fragment.border_box.size.inline > available_inline_size;
            }
        }

        if !need_ellipsis {
            self.push_fragment_to_line_ignoring_text_overflow(fragment);
            return
        }

        let ellipsis = fragment.transform_into_ellipsis(layout_context);
        if let Some(truncation_info) =
                fragment.truncate_to_inline_size(available_inline_size -
                                                 ellipsis.border_box.size.inline) {
            let fragment = fragment.transform_with_split_info(&truncation_info.split,
                                                              truncation_info.text_run);
            self.push_fragment_to_line_ignoring_text_overflow(fragment);
        }
        self.push_fragment_to_line_ignoring_text_overflow(ellipsis);
    }

    /// Pushes a fragment to the current line unconditionally, without placing an ellipsis in the
    /// case of `text-overflow: ellipsis`.
    fn push_fragment_to_line_ignoring_text_overflow(&mut self, fragment: Fragment) {
        let indentation = self.indentation_for_pending_fragment();

        self.pending_line.range.extend_by(FragmentIndex(1));
        self.pending_line.bounds.size.inline = self.pending_line.bounds.size.inline +
            fragment.border_box.size.inline +
            indentation;
        self.pending_line.bounds.size.block = max(self.pending_line.bounds.size.block,
                                                  fragment.border_box.size.block);
        self.new_fragments.push(fragment);
    }

    /// Returns the indentation that needs to be applied before the fragment we're reflowing.
    fn indentation_for_pending_fragment(&self) -> Au {
        if self.pending_line_is_empty() && self.lines.is_empty() {
            self.first_line_indentation
        } else {
            Au(0)
        }
    }

    /// Returns true if the pending line is empty and false otherwise.
    fn pending_line_is_empty(&self) -> bool {
        self.pending_line.range.length() == FragmentIndex(0)
    }
}

/// Represents a list of inline fragments, including element ranges.
#[derive(RustcEncodable, Clone)]
pub struct InlineFragments {
    /// The fragments themselves.
    pub fragments: Vec<Fragment>,
}

impl fmt::Debug for InlineFragments {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?}", self.fragments)
    }
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

    /// Returns true if this list contains no fragments and false if it contains at least one
    /// fragment.
    pub fn is_empty(&self) -> bool {
        self.fragments.is_empty()
    }

    /// Pushes a new inline fragment.
    pub fn push(&mut self, fragment: &mut Fragment) {
        self.fragments.push(fragment.clone());
    }

    /// Merges another set of inline fragments with this one.
    pub fn push_all(&mut self, fragments: InlineFragments) {
        self.fragments.extend(fragments.fragments.into_iter());
    }

    /// A convenience function to return the fragment at a given index.
    pub fn get<'a>(&'a self, index: uint) -> &'a Fragment {
        &self.fragments[index]
    }

    /// A convenience function to return a mutable reference to the fragment at a given index.
    pub fn get_mut<'a>(&'a mut self, index: uint) -> &'a mut Fragment {
        &mut self.fragments[index]
    }
}

/// Flows for inline layout.
#[derive(RustcEncodable)]
pub struct InlineFlow {
    /// Data common to all flows.
    pub base: BaseFlow,

    /// A vector of all inline fragments. Several fragments may correspond to one node/element.
    pub fragments: InlineFragments,

    /// A vector of ranges into fragments that represents line positions. These ranges are disjoint
    /// and are the result of inline layout. This also includes some metadata used for positioning
    /// lines.
    pub lines: Vec<Line>,

    /// The minimum block-size above the baseline for each line, as specified by the line block-
    /// size and font style.
    pub minimum_block_size_above_baseline: Au,

    /// The minimum depth below the baseline for each line, as specified by the line block-size and
    /// font style.
    pub minimum_depth_below_baseline: Au,

    /// The amount of indentation to use on the first line. This is determined by our block parent
    /// (because percentages are relative to the containing block, and we aren't in a position to
    /// compute things relative to our parent's containing block).
    pub first_line_indentation: Au,
}

impl InlineFlow {
    pub fn from_fragments(fragments: InlineFragments, writing_mode: WritingMode) -> InlineFlow {
        InlineFlow {
            base: BaseFlow::new(None, writing_mode, ForceNonfloatedFlag::ForceNonfloated),
            fragments: fragments,
            lines: Vec::new(),
            minimum_block_size_above_baseline: Au(0),
            minimum_depth_below_baseline: Au(0),
            first_line_indentation: Au(0),
        }
    }

    /// Returns the distance from the baseline for the logical block-start inline-start corner of
    /// this fragment, taking into account the value of the CSS `vertical-align` property.
    /// Negative values mean "toward the logical block-start" and positive values mean "toward the
    /// logical block-end".
    ///
    /// The extra boolean is set if and only if `largest_block_size_for_top_fragments` and/or
    /// `largest_block_size_for_bottom_fragments` were updated. That is, if the box has a `top` or
    /// `bottom` value for `vertical-align, true is returned.
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
            vertical_align::T::baseline => (-ascent, false),
            vertical_align::T::middle => {
                // TODO: x-height value should be used from font info.
                // TODO: The code below passes our current reftests but doesn't work in all
                // situations. Add vertical align reftests and fix this.
                (-ascent, false)
            },
            vertical_align::T::sub => {
                let sub_offset = (parent_text_block_start + parent_text_block_end)
                                    .scale_by(FONT_SUBSCRIPT_OFFSET_RATIO);
                (sub_offset - ascent, false)
            },
            vertical_align::T::super_ => {
                let super_offset = (parent_text_block_start + parent_text_block_end)
                                    .scale_by(FONT_SUPERSCRIPT_OFFSET_RATIO);
                (-super_offset - ascent, false)
            },
            vertical_align::T::text_top => {
                let fragment_block_size = *block_size_above_baseline + *depth_below_baseline;
                let prev_depth_below_baseline = *depth_below_baseline;
                *block_size_above_baseline = parent_text_block_start;
                *depth_below_baseline = fragment_block_size - *block_size_above_baseline;
                (*depth_below_baseline - prev_depth_below_baseline - ascent, false)
            },
            vertical_align::T::text_bottom => {
                let fragment_block_size = *block_size_above_baseline + *depth_below_baseline;
                let prev_depth_below_baseline = *depth_below_baseline;
                *depth_below_baseline = parent_text_block_end;
                *block_size_above_baseline = fragment_block_size - *depth_below_baseline;
                (*depth_below_baseline - prev_depth_below_baseline - ascent, false)
            },
            vertical_align::T::top => {
                *largest_block_size_for_top_fragments =
                    max(*largest_block_size_for_top_fragments,
                        *block_size_above_baseline + *depth_below_baseline);
                let offset_top = *block_size_above_baseline - ascent;
                (offset_top, true)
            },
            vertical_align::T::bottom => {
                *largest_block_size_for_bottom_fragments =
                    max(*largest_block_size_for_bottom_fragments,
                        *block_size_above_baseline + *depth_below_baseline);
                let offset_bottom = -(*depth_below_baseline + ascent);
                (offset_bottom, true)
            },
            vertical_align::T::Length(length) => (-(length + ascent), false),
            vertical_align::T::Percentage(p) => {
                let line_height = fragment.calculate_line_height(layout_context);
                let percent_offset = line_height.scale_by(p);
                (-(percent_offset + ascent), false)
            }
        }
    }

    /// Sets fragment positions in the inline direction based on alignment for one line. This
    /// performs text justification if mandated by the style.
    fn set_inline_fragment_positions(fragments: &mut InlineFragments,
                                     line: &Line,
                                     line_align: text_align::T,
                                     indentation: Au,
                                     is_last_line: bool) {
        // Figure out how much inline-size we have.
        let slack_inline_size = max(Au(0), line.green_zone.inline - line.bounds.size.inline);

        // Compute the value we're going to use for `text-justify`.
        let text_justify = if fragments.fragments.is_empty() {
            return
        } else {
            fragments.fragments[0].style().get_inheritedtext().text_justify
        };

        // Set the fragment inline positions based on that alignment, and justify the text if
        // necessary.
        let mut inline_start_position_for_fragment = line.bounds.start.i + indentation;
        match line_align {
            text_align::T::justify if !is_last_line && text_justify != text_justify::T::none => {
                InlineFlow::justify_inline_fragments(fragments, line, slack_inline_size)
            }
            text_align::T::left | text_align::T::justify => {}
            text_align::T::center => {
                inline_start_position_for_fragment = inline_start_position_for_fragment +
                    slack_inline_size.scale_by(0.5)
            }
            text_align::T::right => {
                inline_start_position_for_fragment = inline_start_position_for_fragment +
                    slack_inline_size
            }
        }

        for fragment_index in range(line.range.begin(), line.range.end()) {
            let fragment = fragments.get_mut(fragment_index.to_uint());
            let size = fragment.border_box.size;
            fragment.border_box = LogicalRect::new(fragment.style.writing_mode,
                                                   inline_start_position_for_fragment,
                                                   fragment.border_box.start.b,
                                                   size.inline,
                                                   size.block);
            fragment.update_late_computed_inline_position_if_necessary();
            inline_start_position_for_fragment = inline_start_position_for_fragment + size.inline;
        }
    }

    /// Justifies the given set of inline fragments, distributing the `slack_inline_size` among all
    /// of them according to the value of `text-justify`.
    fn justify_inline_fragments(fragments: &mut InlineFragments,
                                line: &Line,
                                slack_inline_size: Au) {
        // Fast path.
        if slack_inline_size == Au(0) {
            return
        }

        // First, calculate the number of expansion opportunities (spaces, normally).
        let mut expansion_opportunities = 0i32;
        for fragment_index in line.range.each_index() {
            let fragment = fragments.get(fragment_index.to_uint());
            let scanned_text_fragment_info =
                if let SpecificFragmentInfo::ScannedText(ref info) = fragment.specific {
                    info
                } else {
                    continue
                };
            for slice in scanned_text_fragment_info.run.character_slices_in_range(
                    &scanned_text_fragment_info.range) {
                expansion_opportunities += slice.glyphs.space_count_in_range(&slice.range) as i32
            }
        }

        // Then distribute all the space across the expansion opportunities.
        let space_per_expansion_opportunity = slack_inline_size.to_subpx() /
            (expansion_opportunities as f64);
        for fragment_index in line.range.each_index() {
            let fragment = fragments.get_mut(fragment_index.to_uint());
            let mut scanned_text_fragment_info =
                if let SpecificFragmentInfo::ScannedText(ref mut info) = fragment.specific {
                    info
                } else {
                    continue
                };
            let fragment_range = scanned_text_fragment_info.range;

            // FIXME(pcwalton): This is an awful lot of uniqueness making. I don't see any easy way
            // to get rid of it without regressing the performance of the non-justified case,
            // though.
            let run = scanned_text_fragment_info.run.make_unique();
            {
                let glyph_runs = run.glyphs.make_unique();
                for mut glyph_run in glyph_runs.iter_mut() {
                    let mut range = glyph_run.range.intersect(&fragment_range);
                    if range.is_empty() {
                        continue
                    }
                    range.shift_by(-glyph_run.range.begin());

                    let glyph_store = glyph_run.glyph_store.make_unique();
                    glyph_store.distribute_extra_space_in_range(&range,
                                                                space_per_expansion_opportunity);
                }
            }

            // Recompute the fragment's border box size.
            let new_inline_size = run.advance_for_range(&fragment_range);
            let new_size = LogicalSize::new(fragment.style.writing_mode,
                                            new_inline_size,
                                            fragment.border_box.size.block);
            fragment.border_box = LogicalRect::from_point_size(fragment.style.writing_mode,
                                                               fragment.border_box.start,
                                                               new_size);
        }
    }

    /// Sets final fragment positions in the block direction for one line. Assumes that the
    /// fragment positions were initially set to the distance from the baseline first.
    fn set_block_fragment_positions(fragments: &mut InlineFragments,
                                    line: &Line,
                                    line_distance_from_flow_block_start: Au,
                                    baseline_distance_from_block_start: Au,
                                    largest_depth_below_baseline: Au) {
        for fragment_index in range(line.range.begin(), line.range.end()) {
            let fragment = fragments.get_mut(fragment_index.to_uint());
            match fragment.vertical_align() {
                vertical_align::T::top => {
                    fragment.border_box.start.b = fragment.border_box.start.b +
                        line_distance_from_flow_block_start
                }
                vertical_align::T::bottom => {
                    fragment.border_box.start.b = fragment.border_box.start.b +
                        line_distance_from_flow_block_start + baseline_distance_from_block_start +
                        largest_depth_below_baseline
                }
                _ => {
                    fragment.border_box.start.b = fragment.border_box.start.b +
                        line_distance_from_flow_block_start + baseline_distance_from_block_start
                }
            }
            fragment.update_late_computed_block_position_if_necessary();
        }
    }

    /// Computes the minimum ascent and descent for each line. This is done during flow
    /// construction.
    ///
    /// `style` is the style of the block.
    pub fn compute_minimum_ascent_and_descent(&self,
                                              font_context: &mut FontContext,
                                              style: &ComputedValues)
                                              -> (Au, Au) {
        // As a special case, if this flow contains only hypothetical fragments, then the entire
        // flow is hypothetical and takes up no space. See CSS 2.1 § 10.3.7.
        if self.fragments.fragments.iter().all(|fragment| fragment.is_hypothetical()) {
            return (Au(0), Au(0))
        }

        let font_style = style.get_font_arc();
        let font_metrics = text::font_metrics_for_style(font_context, font_style);
        let line_height = text::line_height_from_style(style, &font_metrics);
        let inline_metrics = InlineMetrics::from_font_metrics(&font_metrics, line_height);

        let mut block_size_above_baseline = inline_metrics.block_size_above_baseline;
        let mut depth_below_baseline = inline_metrics.depth_below_baseline;

        // According to CSS 2.1 § 10.8, `line-height` of any inline element specifies the minimal
        // height of line boxes within the element.
        for frag in self.fragments.fragments.iter() {
            match frag.inline_context {
                Some(ref inline_context) => {
                    for style in inline_context.styles.iter() {
                        let font_style = style.get_font_arc();
                        let font_metrics = text::font_metrics_for_style(font_context, font_style);
                        let line_height = text::line_height_from_style(&**style, &font_metrics);
                        let inline_metrics = InlineMetrics::from_font_metrics(&font_metrics,
                                                                              line_height);
                        block_size_above_baseline = max(block_size_above_baseline,
                                                        inline_metrics.block_size_above_baseline);
                        depth_below_baseline = max(depth_below_baseline,
                                                   inline_metrics.depth_below_baseline);
                    }
                }
                None => {}
            }
        }

        (block_size_above_baseline, depth_below_baseline)
    }

    fn update_restyle_damage(&mut self) {
        let mut damage = self.base.restyle_damage;

        for frag in self.fragments.fragments.iter() {
            damage.insert(frag.restyle_damage());
        }

        self.base.restyle_damage = damage;
    }
}

impl Flow for InlineFlow {
    fn class(&self) -> FlowClass {
        FlowClass::Inline
    }

    fn as_immutable_inline<'a>(&'a self) -> &'a InlineFlow {
        self
    }

    fn as_inline<'a>(&'a mut self) -> &'a mut InlineFlow {
        self
    }

    fn bubble_inline_sizes(&mut self) {
        self.update_restyle_damage();

        let _scope = layout_debug_scope!("inline::bubble_inline_sizes {:x}", self.base.debug_id());

        let writing_mode = self.base.writing_mode;
        for kid in self.base.child_iter() {
            flow::mut_base(kid).floats = Floats::new(writing_mode);
        }

        let mut computation = IntrinsicISizesContribution::new();
        for fragment in self.fragments.fragments.iter_mut() {
            debug!("Flow: measuring {:?}", *fragment);
            computation.union_inline(&fragment.compute_intrinsic_inline_sizes().finish())
        }
        self.base.intrinsic_inline_sizes = computation.finish()
    }

    /// Recursively (top-down) determines the actual inline-size of child contexts and fragments.
    /// When called on this context, the context has had its inline-size set by the parent context.
    fn assign_inline_sizes(&mut self, _: &LayoutContext) {
        let _scope = layout_debug_scope!("inline::assign_inline_sizes {:x}", self.base.debug_id());

        // Initialize content fragment inline-sizes if they haven't been initialized already.
        //
        // TODO: Combine this with `LineBreaker`'s walk in the fragment list, or put this into
        // `Fragment`.

        debug!("InlineFlow::assign_inline_sizes: floats in: {:?}", self.base.floats);

        let inline_size = self.base.block_container_inline_size;
        self.base.position.size.inline = inline_size;

        {
            let this = &mut *self;
            for fragment in this.fragments.fragments.iter_mut() {
                fragment.compute_border_and_padding(inline_size);
                fragment.compute_block_direction_margins(inline_size);
                fragment.compute_inline_direction_margins(inline_size);
                fragment.assign_replaced_inline_size_if_necessary(inline_size);
            }
        }

        // If there are any inline-block kids, propagate explicit block and inline
        // sizes down to them.
        let block_container_explicit_block_size = self.base.block_container_explicit_block_size;
        for kid in self.base.child_iter() {
            let kid_base = flow::mut_base(kid);

            kid_base.block_container_inline_size = inline_size;
            kid_base.block_container_explicit_block_size = block_container_explicit_block_size;
        }
    }

    /// Calculate and set the block-size of this flow. See CSS 2.1 § 10.6.1.
    fn assign_block_size(&mut self, layout_context: &LayoutContext) {
        let _scope = layout_debug_scope!("inline::assign_block_size {:x}", self.base.debug_id());

        // Collect various offsets needed by absolutely positioned inline-block or hypothetical
        // absolute descendants.
        (&mut *self as &mut Flow).collect_static_block_offsets_from_children();

        // Divide the fragments into lines.
        //
        // TODO(pcwalton, #226): Get the CSS `line-height` property from the style of the
        // containing block to determine the minimum line block size.
        //
        // TODO(pcwalton, #226): Get the CSS `line-height` property from each non-replaced inline
        // element to determine its block-size for computing the line's own block-size.
        //
        // TODO(pcwalton): Cache the line scanner?
        debug!("assign_block_size_inline: floats in: {:?}", self.base.floats);

        // Assign the block-size for the inline fragments.
        let containing_block_block_size =
            self.base.block_container_explicit_block_size.unwrap_or(Au(0));
        for fragment in self.fragments.fragments.iter_mut() {
            fragment.assign_replaced_block_size_if_necessary(
                containing_block_block_size);
        }

        // Reset our state, so that we handle incremental reflow correctly.
        //
        // TODO(pcwalton): Do something smarter, like Gecko and WebKit?
        self.lines = Vec::new();

        // Determine how much indentation the first line wants.
        let mut indentation = if self.fragments.is_empty() {
            Au(0)
        } else {
            self.first_line_indentation
        };

        // Perform line breaking.
        let mut scanner = LineBreaker::new(self.base.floats.clone(), indentation);
        scanner.scan_for_lines(self, layout_context);

        // Now, go through each line and lay out the fragments inside.
        let mut line_distance_from_flow_block_start = Au(0);
        let line_count = self.lines.len();
        for line_index in range(0, line_count) {
            let line = &mut self.lines[line_index];

            // Lay out fragments in the inline direction, and justify them if necessary.
            InlineFlow::set_inline_fragment_positions(&mut self.fragments,
                                                      line,
                                                      self.base.flags.text_align(),
                                                      indentation,
                                                      line_index + 1 == line_count);

            // Set the block-start position of the current line.
            // `line_height_offset` is updated at the end of the previous loop.
            line.bounds.start.b = line_distance_from_flow_block_start;

            // Calculate the distance from the baseline to the block-start and block-end of the
            // line.
            let mut largest_block_size_above_baseline = self.minimum_block_size_above_baseline;
            let mut largest_depth_below_baseline = self.minimum_depth_below_baseline;

            // Calculate the largest block-size among fragments with 'top' and 'bottom' values
            // respectively.
            let (mut largest_block_size_for_top_fragments,
                 mut largest_block_size_for_bottom_fragments) = (Au(0), Au(0));

            for fragment_index in range(line.range.begin(), line.range.end()) {
                let fragment = &mut self.fragments.fragments[fragment_index.to_uint()];

                let InlineMetrics {
                    mut block_size_above_baseline,
                    mut depth_below_baseline,
                    ascent
                } = fragment.inline_metrics(layout_context);

                // To calculate text-top and text-bottom value when `vertical-align` is involved,
                // we should find the top and bottom of the content area of the parent fragment.
                // "Content area" is defined in CSS 2.1 § 10.6.1.
                //
                // TODO: We should extract em-box info from the font size of the parent and
                // calculate the distances from the baseline to the block-start and the block-end
                // of the parent's content area.

                // We should calculate the distance from baseline to the top of parent's content
                // area. But for now we assume it's the font size.
                //
                // CSS 2.1 does not state which font to use. This version of the code uses
                // the parent's font.

                // Calculate the final block-size above the baseline for this fragment.
                //
                // The no-update flag decides whether `largest_block_size_for_top_fragments` and
                // `largest_block_size_for_bottom_fragments` are to be updated or not. This will be
                // set if and only if the fragment has `vertical-align` set to `top` or `bottom`.
                let (distance_from_baseline, no_update_flag) =
                    InlineFlow::distance_from_baseline(
                        fragment,
                        ascent,
                        self.minimum_block_size_above_baseline,
                        self.minimum_depth_below_baseline,
                        &mut block_size_above_baseline,
                        &mut depth_below_baseline,
                        &mut largest_block_size_for_top_fragments,
                        &mut largest_block_size_for_bottom_fragments,
                        layout_context);

                // Unless the current fragment has `vertical-align` set to `top` or `bottom`,
                // `largest_block_size_above_baseline` and `largest_depth_below_baseline` are
                // updated.
                if !no_update_flag {
                    largest_block_size_above_baseline = max(block_size_above_baseline,
                                                            largest_block_size_above_baseline);
                    largest_depth_below_baseline = max(depth_below_baseline,
                                                       largest_depth_below_baseline);
                }

                // Temporarily use `fragment.border_box.start.b` to mean "the distance from the
                // baseline". We will assign the real value later.
                fragment.border_box.start.b = distance_from_baseline
            }

            // Calculate the distance from the baseline to the top of the largest fragment with a
            // value for `bottom`. Then, if necessary, update `largest_block-size_above_baseline`.
            largest_block_size_above_baseline =
                max(largest_block_size_above_baseline,
                    largest_block_size_for_bottom_fragments - largest_depth_below_baseline);

            // Calculate the distance from baseline to the bottom of the largest fragment with a
            // value for `top`. Then, if necessary, update `largest_depth_below_baseline`.
            largest_depth_below_baseline =
                max(largest_depth_below_baseline,
                    largest_block_size_for_top_fragments - largest_block_size_above_baseline);

            // Now, the distance from the logical block-start of the line to the baseline can be
            // computed as `largest_block-size_above_baseline`.
            let baseline_distance_from_block_start = largest_block_size_above_baseline;

            // Compute the final positions in the block direction of each fragment. Recall that
            // `fragment.border_box.start.b` was set to the distance from the baseline above.
            InlineFlow::set_block_fragment_positions(&mut self.fragments,
                                                     line,
                                                     line_distance_from_flow_block_start,
                                                     baseline_distance_from_block_start,
                                                     largest_depth_below_baseline);

            // This is used to set the block-start position of the next line in the next loop.
            line.bounds.size.block = largest_block_size_above_baseline +
                largest_depth_below_baseline;
            line_distance_from_flow_block_start = line_distance_from_flow_block_start +
                line.bounds.size.block;

            // We're no longer on the first line, so set indentation to zero.
            indentation = Au(0)
        } // End of `lines.iter_mut()` loop.

        // Assign block sizes for any inline-block descendants.
        for kid in self.base.child_iter() {
            if flow::base(kid).flags.contains(IS_ABSOLUTELY_POSITIONED) ||
                    flow::base(kid).flags.is_float() {
                continue
            }
            kid.assign_block_size_for_inorder_child_if_necessary(layout_context);
        }

        self.base.position.size.block = match self.lines.as_slice().last() {
            Some(ref last_line) => last_line.bounds.start.b + last_line.bounds.size.block,
            None => Au(0),
        };

        self.base.floats = scanner.floats.clone();
        self.base.floats.translate(LogicalSize::new(self.base.writing_mode,
                                                    Au(0),
                                                    -self.base.position.size.block));

        self.base.restyle_damage.remove(REFLOW_OUT_OF_FLOW | REFLOW);
    }

    fn compute_absolute_position(&mut self) {
        for fragment in self.fragments.fragments.iter_mut() {
            let stacking_relative_border_box =
                fragment.stacking_relative_border_box(&self.base.stacking_relative_position,
                                                      &self.base
                                                           .absolute_position_info
                                                           .relative_containing_block_size,
                                                      CoordinateSystem::Self);
            let clip = fragment.clipping_region_for_children(&self.base.clip,
                                                             &stacking_relative_border_box);
            match fragment.specific {
                SpecificFragmentInfo::InlineBlock(ref mut info) => {
                    flow::mut_base(&mut *info.flow_ref).clip = clip;
                    let block_flow = info.flow_ref.as_block();
                    block_flow.base.absolute_position_info = self.base.absolute_position_info;
                    block_flow.base.stacking_relative_position =
                        stacking_relative_border_box.origin;
                }
                SpecificFragmentInfo::InlineAbsoluteHypothetical(ref mut info) => {
                    flow::mut_base(&mut *info.flow_ref).clip = clip;
                    let block_flow = info.flow_ref.as_block();
                    block_flow.base.absolute_position_info = self.base.absolute_position_info;
                    block_flow.base.stacking_relative_position =
                        stacking_relative_border_box.origin

                }
                _ => {}
            }
        }
    }

    fn update_late_computed_inline_position_if_necessary(&mut self, _: Au) {}

    fn update_late_computed_block_position_if_necessary(&mut self, _: Au) {}

    fn build_display_list(&mut self, layout_context: &LayoutContext) {
        self.build_display_list_for_inline(layout_context)
    }

    fn repair_style(&mut self, _: &Arc<ComputedValues>) {}

    fn compute_overflow(&self) -> Rect<Au> {
        let mut overflow = ZERO_RECT;
        for fragment in self.fragments.fragments.iter() {
            overflow = overflow.union(&fragment.compute_overflow())
        }
        overflow
    }

    fn iterate_through_fragment_border_boxes(&self,
                                             iterator: &mut FragmentBorderBoxIterator,
                                             stacking_context_position: &Point2D<Au>) {
        // FIXME(#2795): Get the real container size.
        for fragment in self.fragments.fragments.iter() {
            if !iterator.should_process(fragment) {
                continue
            }

            let stacking_relative_position = &self.base.stacking_relative_position;
            let relative_containing_block_size =
                &self.base.absolute_position_info.relative_containing_block_size;
            iterator.process(fragment,
                             &fragment.stacking_relative_border_box(stacking_relative_position,
                                                                    relative_containing_block_size,
                                                                    CoordinateSystem::Parent)
                                      .translate(stacking_context_position))
        }
    }
}

impl fmt::Debug for InlineFlow {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?} - {:x} - {:?}", self.class(), self.base.debug_id(), self.fragments)
    }
}

#[derive(Clone)]
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

/// Block-size above the baseline, depth below the baseline, and ascent for a fragment. See CSS 2.1
/// § 10.8.1.
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
