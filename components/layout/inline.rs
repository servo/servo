/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

#![deny(unsafe_code)]

use block::{AbsoluteAssignBSizesTraversal, AbsoluteStoreOverflowTraversal};
use context::LayoutContext;
use display_list_builder::{FragmentDisplayListBuilding, InlineFlowDisplayListBuilding};
use floats::{FloatKind, Floats, PlacementInfo};
use flow::{self, BaseFlow, FlowClass, Flow, ForceNonfloatedFlag, IS_ABSOLUTELY_POSITIONED};
use flow::{MutableFlowUtils, OpaqueFlow};
use fragment::{CoordinateSystem, Fragment, FragmentBorderBoxIterator, SpecificFragmentInfo};
use incremental::{REFLOW, REFLOW_OUT_OF_FLOW, RESOLVE_GENERATED_CONTENT};
use layout_debug;
use model::IntrinsicISizesContribution;
use text;
use wrapper::PseudoElementType;

use euclid::{Point2D, Rect, Size2D};
use gfx::display_list::OpaqueNode;
use gfx::font::FontMetrics;
use gfx::font_context::FontContext;
use std::cmp::max;
use std::collections::VecDeque;
use std::fmt;
use std::mem;
use std::sync::Arc;
use std::isize;
use style::computed_values::{display, overflow_x, position, text_align, text_justify};
use style::computed_values::{text_overflow, vertical_align, white_space};
use style::properties::ComputedValues;
use unicode_bidi;
use util::geometry::{Au, MAX_AU, ZERO_RECT};
use util::logical_geometry::{LogicalRect, LogicalSize, WritingMode};
use util::range::{Range, RangeIndex};
use util;

// From gfxFontConstants.h in Firefox
static FONT_SUBSCRIPT_OFFSET_RATIO: f32 = 0.20;
static FONT_SUPERSCRIPT_OFFSET_RATIO: f32 = 0.34;

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
#[derive(RustcEncodable, Debug, Clone)]
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
    /// | [0, 2)   | [2, 3)      | [3, 5)      | [5, 6)   |
    /// |----------|-------------|-------------|----------|
    /// | 'I like' | 'truffles,' | '<img> yes' | 'I do.'  |
    pub range: Range<FragmentIndex>,

    /// The bidirectional embedding level runs for this line, in visual order.
    ///
    /// Can be set to `None` if the line is 100% left-to-right.
    pub visual_runs: Option<Vec<(Range<FragmentIndex>, u8)>>,

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

    /// The inline metrics for this line.
    pub inline_metrics: InlineMetrics,
}

impl Line {
    fn new(writing_mode: WritingMode,
           minimum_block_size_above_baseline: Au,
           minimum_depth_below_baseline: Au)
           -> Line {
        Line {
            range: Range::empty(),
            visual_runs: None,
            bounds: LogicalRect::zero(writing_mode),
            green_zone: LogicalSize::zero(writing_mode),
            inline_metrics: InlineMetrics::new(minimum_block_size_above_baseline,
                                               minimum_depth_below_baseline,
                                               minimum_block_size_above_baseline),
        }
    }
}

int_range_index! {
    #[derive(RustcEncodable)]
    #[doc = "The index of a fragment in a flattened vector of DOM elements."]
    struct FragmentIndex(isize)
}

bitflags! {
    flags InlineReflowFlags: u8 {
        #[doc="The `white-space: nowrap` property from CSS 2.1 § 16.6 is in effect."]
        const NO_WRAP_INLINE_REFLOW_FLAG = 0x01,
        #[doc="The `white-space: pre` property from CSS 2.1 § 16.6 is in effect."]
        const WRAP_ON_NEWLINE_INLINE_REFLOW_FLAG = 0x02
    }
}

/// Arranges fragments into lines, splitting them up as necessary.
struct LineBreaker {
    /// The floats we need to flow around.
    floats: Floats,
    /// The resulting fragment list for the flow, consisting of possibly-broken fragments.
    new_fragments: Vec<Fragment>,
    /// The next fragment or fragments that we need to work on.
    work_list: VecDeque<Fragment>,
    /// The line we're currently working on.
    pending_line: Line,
    /// The lines we've already committed.
    lines: Vec<Line>,
    /// The index of the last known good line breaking opportunity. The opportunity will either
    /// be inside this fragment (if it is splittable) or immediately prior to it.
    last_known_line_breaking_opportunity: Option<FragmentIndex>,
    /// The current position in the block direction.
    cur_b: Au,
    /// The computed value of the indentation for the first line (`text-indent`, CSS 2.1 § 16.1).
    first_line_indentation: Au,
    /// The minimum block-size above the baseline for each line, as specified by the line height
    /// and font style.
    minimum_block_size_above_baseline: Au,
    /// The minimum depth below the baseline for each line, as specified by the line height and
    /// font style.
    minimum_depth_below_baseline: Au,
}

impl LineBreaker {
    /// Creates a new `LineBreaker` with a set of floats and the indentation of the first line.
    fn new(float_context: Floats,
           first_line_indentation: Au,
           minimum_block_size_above_baseline: Au,
           minimum_depth_below_baseline: Au)
           -> LineBreaker {
        LineBreaker {
            new_fragments: Vec::new(),
            work_list: VecDeque::new(),
            pending_line: Line::new(float_context.writing_mode,
                                    minimum_block_size_above_baseline,
                                    minimum_depth_below_baseline),
            floats: float_context,
            lines: Vec::new(),
            cur_b: Au(0),
            last_known_line_breaking_opportunity: None,
            first_line_indentation: first_line_indentation,
            minimum_block_size_above_baseline: minimum_block_size_above_baseline,
            minimum_depth_below_baseline: minimum_depth_below_baseline,
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
    fn reset_line(&mut self) -> Line {
        self.last_known_line_breaking_opportunity = None;
        mem::replace(&mut self.pending_line, Line::new(self.floats.writing_mode,
                                                       self.minimum_block_size_above_baseline,
                                                       self.minimum_depth_below_baseline))
    }

    /// Reflows fragments for the given inline flow.
    fn scan_for_lines(&mut self, flow: &mut InlineFlow, layout_context: &LayoutContext) {
        self.reset_scanner();

        // Create our fragment iterator.
        debug!("LineBreaker: scanning for lines, {} fragments", flow.fragments.len());
        let mut old_fragments = mem::replace(&mut flow.fragments, InlineFragments::new());
        let old_fragment_iter = old_fragments.fragments.into_iter();

        // TODO(pcwalton): This would likely be better as a list of dirty line indices. That way we
        // could resynchronize if we discover during reflow that all subsequent fragments must have
        // the same position as they had in the previous reflow. I don't know how common this case
        // really is in practice, but it's probably worth handling.
        self.lines = Vec::new();

        // Do the reflow.
        self.reflow_fragments(old_fragment_iter, flow, layout_context);

        // Perform unicode bidirectional layout.
        let para_level = flow.base.writing_mode.to_bidi_level();

        // The text within a fragment is at a single bidi embedding level (because we split
        // fragments on level run boundaries during flow construction), so we can build a level
        // array with just one entry per fragment.
        let levels: Vec<u8> = self.new_fragments.iter().map(|fragment| match fragment.specific {
            SpecificFragmentInfo::ScannedText(ref info) => info.run.bidi_level,
            _ => para_level
        }).collect();

        let mut lines = mem::replace(&mut self.lines, Vec::new());

        // If everything is LTR, don't bother with reordering.
        let has_rtl = levels.iter().cloned().any(unicode_bidi::is_rtl);

        if has_rtl {
            // Compute and store the visual ordering of the fragments within the line.
            for line in &mut lines {
                let range = line.range.begin().to_usize()..line.range.end().to_usize();
                let runs = unicode_bidi::visual_runs(range, &levels);
                line.visual_runs = Some(runs.iter().map(|run| {
                    let start = FragmentIndex(run.start as isize);
                    let len = FragmentIndex(run.len() as isize);
                    (Range::new(start, len), levels[run.start])
                }).collect());
            }
        }

        // Place the fragments back into the flow.
        old_fragments.fragments = mem::replace(&mut self.new_fragments, vec![]);
        flow.fragments = old_fragments;
        flow.lines = lines;
    }

    /// Reflows the given fragments, which have been plucked out of the inline flow.
    fn reflow_fragments<'a, I>(&mut self,
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
                white_space::T::nowrap => NO_WRAP_INLINE_REFLOW_FLAG,
                white_space::T::pre => {
                    WRAP_ON_NEWLINE_INLINE_REFLOW_FLAG | NO_WRAP_INLINE_REFLOW_FLAG
                }
            };

            // Try to append the fragment.
            self.reflow_fragment(fragment, flow, layout_context, flags);
        }

        if !self.pending_line_is_empty() {
            debug!("LineBreaker: partially full line {} at end of scanning; committing it",
                    self.lines.len());
            self.flush_current_line()
        }
    }

    /// Acquires a new fragment to lay out from the work list or fragment list as appropriate.
    /// Note that you probably don't want to call this method directly in order to be incremental-
    /// reflow-safe; try `next_unbroken_fragment` instead.
    fn next_fragment<I>(&mut self, old_fragment_iter: &mut I) -> Option<Fragment>
                        where I: Iterator<Item=Fragment> {
        self.work_list.pop_front().or_else(|| old_fragment_iter.next())
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
            let candidate = match self.next_fragment(old_fragment_iter) {
                None => return Some(result),
                Some(fragment) => fragment,
            };

            let need_to_merge = match (&mut result.specific, &candidate.specific) {
                (&mut SpecificFragmentInfo::ScannedText(ref mut result_info),
                 &SpecificFragmentInfo::ScannedText(ref candidate_info)) => {
                    util::arc_ptr_eq(&result_info.run, &candidate_info.run) &&
                        inline_contexts_are_equal(&result.inline_context,
                                                  &candidate.inline_context)
                }
                _ => false,
            };


            if need_to_merge {
                result.merge_with(candidate);
                continue
            }

            self.work_list.push_front(candidate);
            return Some(result)
        }
    }

    /// Commits a line to the list.
    fn flush_current_line(&mut self) {
        debug!("LineBreaker: flushing line {}: {:?}", self.lines.len(), self.pending_line);
        self.strip_trailing_whitespace_from_pending_line_if_necessary();
        self.lines.push(self.pending_line.clone());
        self.cur_b = self.pending_line.bounds.start.b + self.pending_line.bounds.size.block;
        self.reset_line();
    }

    /// Removes trailing whitespace from the pending line if necessary. This is done right before
    /// flushing it.
    fn strip_trailing_whitespace_from_pending_line_if_necessary(&mut self) {
        if self.pending_line.range.is_empty() {
            return
        }
        let last_fragment_index = self.pending_line.range.end() - FragmentIndex(1);
        let mut fragment = &mut self.new_fragments[last_fragment_index.get() as usize];

        let mut old_fragment_inline_size = None;
        if let SpecificFragmentInfo::ScannedText(_) = fragment.specific {
            old_fragment_inline_size = Some(fragment.border_box.size.inline +
                                            fragment.margin.inline_start_end());
        }

        fragment.strip_trailing_whitespace_if_necessary();

        if let SpecificFragmentInfo::ScannedText(ref mut scanned_text_fragment_info) =
                fragment.specific {
            let scanned_text_fragment_info = &mut **scanned_text_fragment_info;
            let range = &mut scanned_text_fragment_info.range;

            scanned_text_fragment_info.content_size.inline =
                scanned_text_fragment_info.run.metrics_for_range(range).advance_width;
            fragment.border_box.size.inline = scanned_text_fragment_info.content_size.inline +
                fragment.border_padding.inline_start_end();
            self.pending_line.bounds.size.inline = self.pending_line.bounds.size.inline -
                (old_fragment_inline_size.unwrap() -
                 (fragment.border_box.size.inline + fragment.margin.inline_start_end()));
        }
    }

    // FIXME(eatkinson): this assumes that the tallest fragment in the line determines the line
    // block-size. This might not be the case with some weird text fonts.
    fn new_inline_metrics_for_line(&self, new_fragment: &Fragment, layout_context: &LayoutContext)
                                   -> InlineMetrics {
        self.pending_line.inline_metrics.max(&new_fragment.inline_metrics(layout_context))
    }

    fn new_block_size_for_line(&self, new_fragment: &Fragment, layout_context: &LayoutContext)
                               -> Au {
        max(self.pending_line.bounds.size.block,
            self.new_inline_metrics_for_line(new_fragment, layout_context).block_size())
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
            first_fragment.margin_box_inline_size() + self.indentation_for_pending_fragment()
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
        if line_bounds.size.inline > first_fragment.margin_box_inline_size() {
            debug!("LineBreaker: fragment fits on line {}", self.lines.len());
            return (line_bounds, first_fragment.margin_box_inline_size());
        }

        // If not, but we can't split the fragment, then we'll place the line here and it will
        // overflow.
        if !first_fragment.can_split() {
            debug!("LineBreaker: line doesn't fit, but is unsplittable");
        }

        (line_bounds, first_fragment.margin_box_inline_size())
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

    /// Tries to append the given fragment to the line, splitting it if necessary. Commits the
    /// current line if needed.
    fn reflow_fragment(&mut self,
                       mut fragment: Fragment,
                       flow: &InlineFlow,
                       layout_context: &LayoutContext,
                       flags: InlineReflowFlags) {
        // Determine initial placement for the fragment if we need to.
        //
        // Also, determine whether we can legally break the line before, or inside, this fragment.
        let fragment_is_line_break_opportunity = if self.pending_line_is_empty() {
            fragment.strip_leading_whitespace_if_necessary();
            let (line_bounds, _) = self.initial_line_placement(flow, &fragment, self.cur_b);
            self.pending_line.bounds.start = line_bounds.start;
            self.pending_line.green_zone = line_bounds.size;
            false
        } else {
            !flags.contains(NO_WRAP_INLINE_REFLOW_FLAG)
        };

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
            if !self.avoid_floats(flow, fragment, new_block_size) {
                self.flush_current_line();
            }
            return
        }

        // Record the last known good line break opportunity if this is one.
        if fragment_is_line_break_opportunity {
            self.last_known_line_breaking_opportunity = Some(self.pending_line.range.end())
        }

        // If we must flush the line after finishing this fragment due to `white-space: pre`,
        // detect that.
        let line_flush_mode =
            if flags.contains(WRAP_ON_NEWLINE_INLINE_REFLOW_FLAG) &&
                    fragment.requires_line_break_afterward_if_wrapping_on_newlines() {
                LineFlushMode::Flush
            } else {
                LineFlushMode::No
            };

        // If we're not going to overflow the green zone vertically, we might still do so
        // horizontally. We'll try to place the whole fragment on this line and break somewhere if
        // it doesn't fit.
        let indentation = self.indentation_for_pending_fragment();
        let new_inline_size = self.pending_line.bounds.size.inline +
            fragment.margin_box_inline_size() + indentation;
        if new_inline_size <= green_zone.inline {
            debug!("LineBreaker: fragment fits without splitting");
            self.push_fragment_to_line(layout_context, fragment, line_flush_mode);
            return
        }

        // If we can't split the fragment and we're at the start of the line, then just overflow.
        if !fragment.can_split() && self.pending_line_is_empty() {
            debug!("LineBreaker: fragment can't split and line {} is empty, so overflowing",
                    self.lines.len());
            self.push_fragment_to_line(layout_context, fragment, LineFlushMode::No);
            return
        }

        // If the wrapping mode prevents us from splitting, then back up and split at the last
        // known good split point.
        if flags.contains(NO_WRAP_INLINE_REFLOW_FLAG) &&
                !flags.contains(WRAP_ON_NEWLINE_INLINE_REFLOW_FLAG) {
            debug!("LineBreaker: white-space: nowrap in effect; falling back to last known good \
                    split point");
            if !self.split_line_at_last_known_good_position() {
                // No line breaking opportunity exists at all for this line. Overflow.
                self.push_fragment_to_line(layout_context, fragment, LineFlushMode::No)
            } else {
                self.work_list.push_front(fragment)
            }
            return
        }

        // Split it up!
        let available_inline_size = if !flags.contains(NO_WRAP_INLINE_REFLOW_FLAG) {
            green_zone.inline - self.pending_line.bounds.size.inline - indentation
        } else {
            MAX_AU
        };
        let inline_start_fragment;
        let inline_end_fragment;
        let split_result = match fragment.calculate_split_position(available_inline_size,
                                                                   self.pending_line_is_empty()) {
            None => {
                // We failed to split. Defer to the next line if we're allowed to; otherwise,
                // rewind to the last line breaking opportunity.
                if fragment_is_line_break_opportunity {
                    debug!("LineBreaker: fragment was unsplittable; deferring to next line");
                    self.work_list.push_front(fragment);
                    self.flush_current_line();
                } else if self.split_line_at_last_known_good_position() {
                    // We split the line at a known-good prior position. Restart with the current
                    // fragment.
                    self.work_list.push_front(fragment)
                } else {
                    // We failed to split and there is no known-good place on this line to split.
                    // Overflow.
                    self.push_fragment_to_line(layout_context, fragment, LineFlushMode::No)
                }
                return
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
                self.push_fragment_to_line(layout_context,
                                           inline_start_fragment,
                                           LineFlushMode::Flush);
                self.work_list.push_front(inline_end_fragment)
            },
            (Some(fragment), None) => {
                self.push_fragment_to_line(layout_context, fragment, line_flush_mode);
            }
            (None, Some(fragment)) => {
                // Yes, this can happen!
                self.flush_current_line();
                self.work_list.push_front(fragment)
            }
            (None, None) => {}
        }
    }

    /// Pushes a fragment to the current line unconditionally, possibly truncating it and placing
    /// an ellipsis based on the value of `text-overflow`. If `flush_line` is `Flush`, then flushes
    /// the line afterward;
    fn push_fragment_to_line(&mut self,
                             layout_context: &LayoutContext,
                             fragment: Fragment,
                             line_flush_mode: LineFlushMode) {
        let indentation = self.indentation_for_pending_fragment();
        if self.pending_line_is_empty() {
            debug_assert!(self.new_fragments.len() <= (isize::MAX as usize));
            self.pending_line.range.reset(FragmentIndex(self.new_fragments.len() as isize),
                                          FragmentIndex(0));
        }

        // Determine if an ellipsis will be necessary to account for `text-overflow`.
        let mut need_ellipsis = false;
        let available_inline_size = self.pending_line.green_zone.inline -
            self.pending_line.bounds.size.inline - indentation;
        match (fragment.style().get_inheritedtext().text_overflow,
               fragment.style().get_box().overflow_x) {
            (text_overflow::T::clip, _) | (_, overflow_x::T::visible) => {}
            (text_overflow::T::ellipsis, _) => {
                need_ellipsis = fragment.margin_box_inline_size() > available_inline_size;
            }
        }

        if !need_ellipsis {
            self.push_fragment_to_line_ignoring_text_overflow(fragment, layout_context);
        } else {
            let ellipsis = fragment.transform_into_ellipsis(layout_context);
            if let Some(truncation_info) =
                    fragment.truncate_to_inline_size(available_inline_size -
                                                     ellipsis.margin_box_inline_size()) {
                let fragment = fragment.transform_with_split_info(&truncation_info.split,
                                                                  truncation_info.text_run);
                self.push_fragment_to_line_ignoring_text_overflow(fragment, layout_context);
            }
            self.push_fragment_to_line_ignoring_text_overflow(ellipsis, layout_context);
        }

        if line_flush_mode == LineFlushMode::Flush {
            self.flush_current_line()
        }
    }

    /// Pushes a fragment to the current line unconditionally, without placing an ellipsis in the
    /// case of `text-overflow: ellipsis`.
    fn push_fragment_to_line_ignoring_text_overflow(&mut self,
                                                    fragment: Fragment,
                                                    layout_context: &LayoutContext) {
        let indentation = self.indentation_for_pending_fragment();
        self.pending_line.range.extend_by(FragmentIndex(1));
        self.pending_line.bounds.size.inline = self.pending_line.bounds.size.inline +
            fragment.margin_box_inline_size() +
            indentation;
        self.pending_line.inline_metrics =
            self.new_inline_metrics_for_line(&fragment, layout_context);
        self.pending_line.bounds.size.block =
            self.new_block_size_for_line(&fragment, layout_context);
        self.new_fragments.push(fragment);
    }

    fn split_line_at_last_known_good_position(&mut self) -> bool {
        let last_known_line_breaking_opportunity =
            match self.last_known_line_breaking_opportunity {
                None => return false,
                Some(last_known_line_breaking_opportunity) => last_known_line_breaking_opportunity,
            };

        for fragment_index in (last_known_line_breaking_opportunity.get()..
                               self.pending_line.range.end().get()).rev() {
            debug_assert!(fragment_index == (self.new_fragments.len() as isize) - 1);
            self.work_list.push_front(self.new_fragments.pop().unwrap());
        }

        // FIXME(pcwalton): This should actually attempt to split the last fragment if
        // possible to do so, to handle cases like:
        //
        //     (available width)
        //     +-------------+
        //     The alphabet
        //     (<em>abcdefghijklmnopqrstuvwxyz</em>)
        //
        // Here, the last known-good split point is inside the fragment containing
        // "The alphabet (", which has already been committed by the time we get to this
        // point. Unfortunately, the existing splitting API (`calculate_split_position`)
        // has no concept of "split right before the last non-whitespace position". We'll
        // need to add that feature to the API to handle this case correctly.
        self.pending_line.range.extend_to(last_known_line_breaking_opportunity);
        self.flush_current_line();
        true
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
        for fragment in &self.fragments {
            try!(write!(f, "\n  * {:?}", fragment))
        }
        Ok(())
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
    pub fn len(&self) -> usize {
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
    pub fn push_all(&mut self, mut other: InlineFragments) {
        self.fragments.append(&mut other.fragments);
    }

    /// A convenience function to return the fragment at a given index.
    pub fn get<'a>(&'a self, index: usize) -> &'a Fragment {
        &self.fragments[index]
    }

    /// A convenience function to return a mutable reference to the fragment at a given index.
    pub fn get_mut<'a>(&'a mut self, index: usize) -> &'a mut Fragment {
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

    /// The minimum block-size above the baseline for each line, as specified by the line height
    /// and font style.
    pub minimum_block_size_above_baseline: Au,

    /// The minimum depth below the baseline for each line, as specified by the line height and
    /// font style.
    pub minimum_depth_below_baseline: Au,

    /// The amount of indentation to use on the first line. This is determined by our block parent
    /// (because percentages are relative to the containing block, and we aren't in a position to
    /// compute things relative to our parent's containing block).
    pub first_line_indentation: Au,
}

impl InlineFlow {
    pub fn from_fragments(fragments: InlineFragments, writing_mode: WritingMode) -> InlineFlow {
        let mut flow = InlineFlow {
            base: BaseFlow::new(None, writing_mode, ForceNonfloatedFlag::ForceNonfloated),
            fragments: fragments,
            lines: Vec::new(),
            minimum_block_size_above_baseline: Au(0),
            minimum_depth_below_baseline: Au(0),
            first_line_indentation: Au(0),
        };

        for fragment in &flow.fragments.fragments {
            if fragment.is_generated_content() {
                flow.base.restyle_damage.insert(RESOLVE_GENERATED_CONTENT)
            }
        }

        flow
    }

    /// Returns the distance from the baseline for the logical block-start inline-start corner of
    /// this fragment, taking into account the value of the CSS `vertical-align` property.
    /// Negative values mean "toward the logical block-start" and positive values mean "toward the
    /// logical block-end".
    ///
    /// The extra boolean is set if and only if `largest_block_size_for_top_fragments` and/or
    /// `largest_block_size_for_bottom_fragments` were updated. That is, if the box has a `top` or
    /// `bottom` value for `vertical-align`, true is returned.
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
        let (mut offset_from_baseline, mut largest_size_updated) = (Au(0), false);
        for style in fragment.inline_styles() {
            // Ignore `vertical-align` values for table cells.
            let box_style = style.get_box();
            match box_style.display {
                display::T::inline | display::T::block | display::T::inline_block => {}
                _ => continue,
            }

            match box_style.vertical_align {
                vertical_align::T::baseline => {}
                vertical_align::T::middle => {
                    // TODO: x-height value should be used from font info.
                    // TODO: Doing nothing here passes our current reftests but doesn't work in
                    // all situations. Add vertical align reftests and fix this.
                },
                vertical_align::T::sub => {
                    let sub_offset = (parent_text_block_start + parent_text_block_end)
                                        .scale_by(FONT_SUBSCRIPT_OFFSET_RATIO);
                    offset_from_baseline = offset_from_baseline + sub_offset
                },
                vertical_align::T::super_ => {
                    let super_offset = (parent_text_block_start + parent_text_block_end)
                                        .scale_by(FONT_SUPERSCRIPT_OFFSET_RATIO);
                    offset_from_baseline = offset_from_baseline - super_offset
                },
                vertical_align::T::text_top => {
                    let fragment_block_size = *block_size_above_baseline +
                        *depth_below_baseline;
                    let prev_depth_below_baseline = *depth_below_baseline;
                    *block_size_above_baseline = parent_text_block_start;
                    *depth_below_baseline = fragment_block_size - *block_size_above_baseline;
                    offset_from_baseline = offset_from_baseline + *depth_below_baseline -
                        prev_depth_below_baseline
                },
                vertical_align::T::text_bottom => {
                    let fragment_block_size = *block_size_above_baseline +
                        *depth_below_baseline;
                    let prev_depth_below_baseline = *depth_below_baseline;
                    *depth_below_baseline = parent_text_block_end;
                    *block_size_above_baseline = fragment_block_size - *depth_below_baseline;
                    offset_from_baseline = offset_from_baseline + *depth_below_baseline -
                        prev_depth_below_baseline
                },
                vertical_align::T::top => {
                    if !largest_size_updated {
                        largest_size_updated = true;
                        *largest_block_size_for_top_fragments =
                            max(*largest_block_size_for_top_fragments,
                                *block_size_above_baseline + *depth_below_baseline);
                        offset_from_baseline = offset_from_baseline +
                            *block_size_above_baseline
                    }
                },
                vertical_align::T::bottom => {
                    if !largest_size_updated {
                        largest_size_updated = true;
                        *largest_block_size_for_bottom_fragments =
                            max(*largest_block_size_for_bottom_fragments,
                                *block_size_above_baseline + *depth_below_baseline);
                        offset_from_baseline = offset_from_baseline - *depth_below_baseline
                    }
                },
                vertical_align::T::Length(length) => {
                    offset_from_baseline = offset_from_baseline - length
                }
                vertical_align::T::Percentage(p) => {
                    let line_height = fragment.calculate_line_height(layout_context);
                    let percent_offset = line_height.scale_by(p);
                    offset_from_baseline = offset_from_baseline - percent_offset
                }
            }
        }
        (offset_from_baseline - ascent, largest_size_updated)
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
        if fragments.fragments.is_empty() {
            return
        }
        let text_justify = fragments.fragments[0].style().get_inheritedtext().text_justify;

        // Translate `left` and `right` to logical directions.
        let is_ltr = fragments.fragments[0].style().writing_mode.is_bidi_ltr();
        let line_align = match (line_align, is_ltr) {
            (text_align::T::left, true) | (text_align::T::right, false) => text_align::T::start,
            (text_align::T::left, false) | (text_align::T::right, true) => text_align::T::end,
            _ => line_align
        };

        // Set the fragment inline positions based on that alignment, and justify the text if
        // necessary.
        let mut inline_start_position_for_fragment = line.bounds.start.i + indentation;
        match line_align {
            text_align::T::justify if !is_last_line && text_justify != text_justify::T::none => {
                InlineFlow::justify_inline_fragments(fragments, line, slack_inline_size)
            }
            text_align::T::justify | text_align::T::start => {}
            text_align::T::center | text_align::T::servo_center => {
                inline_start_position_for_fragment = inline_start_position_for_fragment +
                    slack_inline_size.scale_by(0.5)
            }
            text_align::T::end => {
                inline_start_position_for_fragment = inline_start_position_for_fragment +
                    slack_inline_size
            }
            text_align::T::left | text_align::T::right => unreachable!()
        }

        // Lay out the fragments in visual order.
        let run_count = match line.visual_runs {
            Some(ref runs) => runs.len(),
            None => 1
        };
        for run_idx in 0..run_count {
            let (range, level) = match line.visual_runs {
                Some(ref runs) if is_ltr => runs[run_idx],
                Some(ref runs) => runs[run_count - run_idx - 1], // reverse order for RTL runs
                None => (line.range, 0)
            };
            // If the bidi embedding direction is opposite the layout direction, lay out this
            // run in reverse order.
            let reverse = unicode_bidi::is_ltr(level) != is_ltr;
            let fragment_indices = if reverse {
                (range.end().get() - 1..range.begin().get() - 1).step_by(-1)
            } else {
                (range.begin().get()..range.end().get()).step_by(1)
            };

            for fragment_index in fragment_indices {
                let fragment = fragments.get_mut(fragment_index as usize);
                inline_start_position_for_fragment = inline_start_position_for_fragment +
                    fragment.margin.inline_start;

                let border_start = if fragment.style.writing_mode.is_bidi_ltr() == is_ltr {
                    inline_start_position_for_fragment
                } else {
                    line.green_zone.inline - inline_start_position_for_fragment
                                           - fragment.margin.inline_end
                                           - fragment.border_box.size.inline
                };
                fragment.border_box = LogicalRect::new(fragment.style.writing_mode,
                                                       border_start,
                                                       fragment.border_box.start.b,
                                                       fragment.border_box.size.inline,
                                                       fragment.border_box.size.block);
                fragment.update_late_computed_inline_position_if_necessary();
                inline_start_position_for_fragment = inline_start_position_for_fragment +
                    fragment.border_box.size.inline + fragment.margin.inline_end;
            }
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
            let fragment = fragments.get(fragment_index.to_usize());
            let scanned_text_fragment_info = match fragment.specific {
                SpecificFragmentInfo::ScannedText(ref info) if !info.range.is_empty() => info,
                _ => continue
            };
            let fragment_range = scanned_text_fragment_info.range;

            for slice in scanned_text_fragment_info.run.character_slices_in_range(&fragment_range) {
                expansion_opportunities += slice.glyphs.space_count_in_range(&slice.range) as i32
            }
        }

        // Then distribute all the space across the expansion opportunities.
        let space_per_expansion_opportunity = slack_inline_size.to_f64_px() /
            (expansion_opportunities as f64);
        for fragment_index in line.range.each_index() {
            let fragment = fragments.get_mut(fragment_index.to_usize());
            let mut scanned_text_fragment_info = match fragment.specific {
                SpecificFragmentInfo::ScannedText(ref mut info) if !info.range.is_empty() => info,
                _ => continue
            };
            let fragment_range = scanned_text_fragment_info.range;

            // FIXME(pcwalton): This is an awful lot of uniqueness making. I don't see any easy way
            // to get rid of it without regressing the performance of the non-justified case,
            // though.
            let run = Arc::make_unique(&mut scanned_text_fragment_info.run);
            {
                let glyph_runs = Arc::make_unique(&mut run.glyphs);
                for mut glyph_run in &mut *glyph_runs {
                    let mut range = glyph_run.range.intersect(&fragment_range);
                    if range.is_empty() {
                        continue
                    }
                    range.shift_by(-glyph_run.range.begin());

                    let glyph_store = Arc::make_unique(&mut glyph_run.glyph_store);
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
        for fragment_index in line.range.each_index() {
            // If any of the inline styles say `top` or `bottom`, adjust the vertical align
            // appropriately.
            //
            // FIXME(#5624, pcwalton): This passes our current reftests but isn't the right thing
            // to do.
            let fragment = fragments.get_mut(fragment_index.to_usize());
            let mut vertical_align = vertical_align::T::baseline;
            for style in fragment.inline_styles() {
                match (style.get_box().display, style.get_box().vertical_align) {
                    (display::T::inline, vertical_align::T::top) |
                    (display::T::block, vertical_align::T::top) |
                    (display::T::inline_block, vertical_align::T::top) => {
                        vertical_align = vertical_align::T::top;
                        break
                    }
                    (display::T::inline, vertical_align::T::bottom) |
                    (display::T::block, vertical_align::T::bottom) |
                    (display::T::inline_block, vertical_align::T::bottom) => {
                        vertical_align = vertical_align::T::bottom;
                        break
                    }
                    _ => {}
                }
            }

            match vertical_align {
                vertical_align::T::top => {
                    fragment.border_box.start.b = fragment.border_box.start.b +
                        line_distance_from_flow_block_start
                }
                vertical_align::T::bottom => {
                    fragment.border_box.start.b = fragment.border_box.start.b +
                        line_distance_from_flow_block_start +
                        baseline_distance_from_block_start +
                        largest_depth_below_baseline;
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
        for frag in &self.fragments.fragments {
            match frag.inline_context {
                Some(ref inline_context) => {
                    for node in &inline_context.nodes {
                        let font_style = node.style.get_font_arc();
                        let font_metrics = text::font_metrics_for_style(font_context, font_style);
                        let line_height = text::line_height_from_style(&*node.style, &font_metrics);
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

        for frag in &self.fragments.fragments {
            damage.insert(frag.restyle_damage());
        }

        self.base.restyle_damage = damage;
    }

    fn containing_block_range_for_flow_surrounding_fragment_at_index(&self,
                                                                     fragment_index: FragmentIndex)
                                                                     -> Range<FragmentIndex> {
        let mut start_index = fragment_index;
        while start_index > FragmentIndex(0) &&
                self.fragments
                    .fragments[(start_index - FragmentIndex(1)).get() as usize]
                    .is_positioned() {
            start_index = start_index - FragmentIndex(1)
        }

        let mut end_index = fragment_index + FragmentIndex(1);
        while end_index < FragmentIndex(self.fragments.fragments.len() as isize) &&
                self.fragments.fragments[end_index.get() as usize].is_positioned() {
            end_index = end_index + FragmentIndex(1)
        }

        Range::new(start_index, end_index - start_index)
    }

    fn containing_block_range_for_flow(&self, opaque_flow: OpaqueFlow) -> Range<FragmentIndex> {
        let index = FragmentIndex(self.fragments.fragments.iter().position(|fragment| {
            match fragment.specific {
                SpecificFragmentInfo::InlineAbsolute(ref inline_absolute) => {
                    OpaqueFlow::from_flow(&*inline_absolute.flow_ref) == opaque_flow
                }
                SpecificFragmentInfo::InlineAbsoluteHypothetical(
                        ref inline_absolute_hypothetical) => {
                    OpaqueFlow::from_flow(&*inline_absolute_hypothetical.flow_ref) == opaque_flow
                }
                _ => false,
            }
        }).expect("containing_block_range_for_flow(): couldn't find inline absolute fragment!")
                                 as isize);
        self.containing_block_range_for_flow_surrounding_fragment_at_index(index)
    }
}

impl Flow for InlineFlow {
    fn class(&self) -> FlowClass {
        FlowClass::Inline
    }

    fn as_inline<'a>(&'a self) -> &'a InlineFlow {
        self
    }

    fn as_mut_inline<'a>(&'a mut self) -> &'a mut InlineFlow {
        self
    }

    fn bubble_inline_sizes(&mut self) {
        self.update_restyle_damage();

        let _scope = layout_debug_scope!("inline::bubble_inline_sizes {:x}", self.base.debug_id());

        let writing_mode = self.base.writing_mode;
        for kid in self.base.child_iter() {
            flow::mut_base(kid).floats = Floats::new(writing_mode);
        }

        let mut intrinsic_sizes_for_flow = IntrinsicISizesContribution::new();
        let mut intrinsic_sizes_for_inline_run = IntrinsicISizesContribution::new();
        let mut intrinsic_sizes_for_nonbroken_run = IntrinsicISizesContribution::new();
        for fragment in &mut self.fragments.fragments {
            let intrinsic_sizes_for_fragment = fragment.compute_intrinsic_inline_sizes().finish();
            match fragment.style.get_inheritedtext().white_space {
                white_space::T::nowrap => {
                    intrinsic_sizes_for_nonbroken_run.union_nonbreaking_inline(
                        &intrinsic_sizes_for_fragment)
                }
                white_space::T::pre => {
                    intrinsic_sizes_for_nonbroken_run.union_nonbreaking_inline(
                        &intrinsic_sizes_for_fragment);

                    // Flush the intrinsic sizes we've been gathering up in order to handle the
                    // line break, if necessary.
                    if fragment.requires_line_break_afterward_if_wrapping_on_newlines() {
                        intrinsic_sizes_for_inline_run.union_inline(
                            &intrinsic_sizes_for_nonbroken_run.finish());
                        intrinsic_sizes_for_nonbroken_run = IntrinsicISizesContribution::new();
                        intrinsic_sizes_for_flow.union_block(
                            &intrinsic_sizes_for_inline_run.finish());
                        intrinsic_sizes_for_inline_run = IntrinsicISizesContribution::new();
                    }
                }
                white_space::T::normal => {
                    // Flush the intrinsic sizes we were gathering up for the nonbroken run, if
                    // necessary.
                    intrinsic_sizes_for_inline_run.union_inline(
                        &intrinsic_sizes_for_nonbroken_run.finish());
                    intrinsic_sizes_for_nonbroken_run = IntrinsicISizesContribution::new();

                    intrinsic_sizes_for_nonbroken_run.union_inline(&intrinsic_sizes_for_fragment)
                }
            }
        }

        // Flush any remaining nonbroken-run and inline-run intrinsic sizes.
        intrinsic_sizes_for_inline_run.union_inline(&intrinsic_sizes_for_nonbroken_run.finish());
        intrinsic_sizes_for_flow.union_block(&intrinsic_sizes_for_inline_run.finish());

        // Finish up the computation.
        self.base.intrinsic_inline_sizes = intrinsic_sizes_for_flow.finish()
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
        let container_mode = self.base.block_container_writing_mode;
        self.base.position.size.inline = inline_size;

        {
            let this = &mut *self;
            for fragment in this.fragments.fragments.iter_mut() {
                let border_collapse = fragment.style.get_inheritedtable().border_collapse;
                fragment.compute_border_and_padding(inline_size, border_collapse);
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
            kid_base.block_container_writing_mode = container_mode;
            kid_base.block_container_explicit_block_size = block_container_explicit_block_size;
        }
    }

    /// Calculate and set the block-size of this flow. See CSS 2.1 § 10.6.1.
    fn assign_block_size(&mut self, layout_context: &LayoutContext) {
        let _scope = layout_debug_scope!("inline::assign_block_size {:x}", self.base.debug_id());

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

        // Assign the block-size and late-computed inline-sizes for the inline fragments.
        let containing_block_block_size =
            self.base.block_container_explicit_block_size;
        for fragment in &mut self.fragments.fragments {
            fragment.update_late_computed_replaced_inline_size_if_necessary();
            fragment.assign_replaced_block_size_if_necessary(containing_block_block_size);
        }

        // Reset our state, so that we handle incremental reflow correctly.
        //
        // TODO(pcwalton): Do something smarter, like Gecko and WebKit?
        self.lines.clear();

        // Determine how much indentation the first line wants.
        let mut indentation = if self.fragments.is_empty() {
            Au(0)
        } else {
            self.first_line_indentation
        };

        // Perform line breaking.
        let mut scanner = LineBreaker::new(self.base.floats.clone(),
                                           indentation,
                                           self.minimum_block_size_above_baseline,
                                           self.minimum_depth_below_baseline);
        scanner.scan_for_lines(self, layout_context);


        // Now, go through each line and lay out the fragments inside.
        let mut line_distance_from_flow_block_start = Au(0);
        let line_count = self.lines.len();
        for line_index in 0..line_count {
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

            for fragment_index in line.range.each_index() {
                let fragment = &mut self.fragments.fragments[fragment_index.to_usize()];

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
        let thread_id = self.base.thread_id;
        for kid in self.base.child_iter() {
            if flow::base(kid).flags.contains(IS_ABSOLUTELY_POSITIONED) ||
                    flow::base(kid).flags.is_float() {
                continue
            }
            kid.assign_block_size_for_inorder_child_if_necessary(layout_context, thread_id);
        }

        if self.contains_positioned_fragments() {
            // Assign block-sizes for all flows in this absolute flow tree.
            // This is preorder because the block-size of an absolute flow may depend on
            // the block-size of its containing block, which may also be an absolute flow.
            (&mut *self as &mut Flow).traverse_preorder_absolute_flows(
                &mut AbsoluteAssignBSizesTraversal(layout_context));
            // Store overflow for all absolute descendants.
            (&mut *self as &mut Flow).traverse_postorder_absolute_flows(
                &mut AbsoluteStoreOverflowTraversal {
                    layout_context: layout_context,
                });
        }

        self.base.position.size.block = match self.lines.last() {
            Some(ref last_line) => last_line.bounds.start.b + last_line.bounds.size.block,
            None => Au(0),
        };

        self.base.floats = scanner.floats.clone();
        let writing_mode = self.base.floats.writing_mode;
        self.base.floats.translate(LogicalSize::new(writing_mode,
                                                    Au(0),
                                                    -self.base.position.size.block));

        self.base.restyle_damage.remove(REFLOW_OUT_OF_FLOW | REFLOW);
    }

    fn compute_absolute_position(&mut self, _: &LayoutContext) {
        // First, gather up the positions of all the containing blocks (if any).
        //
        // FIXME(pcwalton): This will get the absolute containing blocks inside `...` wrong in the
        // case of something like:
        //
        //      <span style="position: relative">
        //          Foo
        //          <span style="display: inline-block">...</span>
        //      </span>
        let mut containing_block_positions = Vec::new();
        let container_size = Size2D::new(self.base.block_container_inline_size, Au(0));
        for (fragment_index, fragment) in self.fragments.fragments.iter().enumerate() {
            match fragment.specific {
                SpecificFragmentInfo::InlineAbsolute(_) => {
                    let containing_block_range =
                        self.containing_block_range_for_flow_surrounding_fragment_at_index(
                            FragmentIndex(fragment_index as isize));
                    let first_fragment_index = containing_block_range.begin().get() as usize;
                    debug_assert!(first_fragment_index < self.fragments.fragments.len());
                    let first_fragment = &self.fragments.fragments[first_fragment_index];
                    let padding_box_origin = (first_fragment.border_box -
                                              first_fragment.style.logical_border_width()).start;
                    containing_block_positions.push(
                        padding_box_origin.to_physical(self.base.writing_mode, container_size));
                }
                SpecificFragmentInfo::InlineBlock(_) if fragment.is_positioned() => {
                    let containing_block_range =
                        self.containing_block_range_for_flow_surrounding_fragment_at_index(
                            FragmentIndex(fragment_index as isize));
                    let first_fragment_index = containing_block_range.begin().get() as usize;
                    debug_assert!(first_fragment_index < self.fragments.fragments.len());
                    let first_fragment = &self.fragments.fragments[first_fragment_index];
                    let padding_box_origin = (first_fragment.border_box -
                                              first_fragment.style.logical_border_width()).start;
                    containing_block_positions.push(
                        padding_box_origin.to_physical(self.base.writing_mode, container_size));
                }
                _ => {}
            }
        }

        // Then compute the positions of all of our fragments.
        let mut containing_block_positions = containing_block_positions.iter();
        for fragment in &mut self.fragments.fragments {
            let stacking_relative_border_box =
                fragment.stacking_relative_border_box(&self.base.stacking_relative_position,
                                                      &self.base
                                                           .absolute_position_info
                                                           .relative_containing_block_size,
                                                      self.base
                                                          .absolute_position_info
                                                          .relative_containing_block_mode,
                                                      CoordinateSystem::Parent);
            let clip = fragment.clipping_region_for_children(&self.base.clip,
                                                             &stacking_relative_border_box,
                                                             false);
            let is_positioned = fragment.is_positioned();
            match fragment.specific {
                SpecificFragmentInfo::InlineBlock(ref mut info) => {
                    flow::mut_base(&mut *info.flow_ref).clip = clip;

                    let block_flow = info.flow_ref.as_mut_block();
                    block_flow.base.absolute_position_info = self.base.absolute_position_info;

                    let stacking_relative_position = self.base.stacking_relative_position;
                    if is_positioned {
                        let padding_box_origin = containing_block_positions.next().unwrap();
                        block_flow.base
                                  .absolute_position_info
                                  .stacking_relative_position_of_absolute_containing_block =
                            stacking_relative_position + *padding_box_origin;
                    }

                    block_flow.base.stacking_relative_position =
                        stacking_relative_border_box.origin;
                    block_flow.base.stacking_relative_position_of_display_port =
                        self.base.stacking_relative_position_of_display_port;
                }
                SpecificFragmentInfo::InlineAbsoluteHypothetical(ref mut info) => {
                    flow::mut_base(&mut *info.flow_ref).clip = clip;

                    let block_flow = info.flow_ref.as_mut_block();
                    block_flow.base.absolute_position_info = self.base.absolute_position_info;

                    block_flow.base.stacking_relative_position =
                        stacking_relative_border_box.origin;
                    block_flow.base.stacking_relative_position_of_display_port =
                        self.base.stacking_relative_position_of_display_port;
                }
                SpecificFragmentInfo::InlineAbsolute(ref mut info) => {
                    flow::mut_base(&mut *info.flow_ref).clip = clip;

                    let block_flow = info.flow_ref.as_mut_block();
                    block_flow.base.absolute_position_info = self.base.absolute_position_info;

                    let stacking_relative_position = self.base.stacking_relative_position;
                    let padding_box_origin = containing_block_positions.next().unwrap();
                    block_flow.base
                              .absolute_position_info
                              .stacking_relative_position_of_absolute_containing_block =
                        stacking_relative_position + *padding_box_origin;

                    block_flow.base.stacking_relative_position =
                        stacking_relative_border_box.origin;
                    block_flow.base.stacking_relative_position_of_display_port =
                        self.base.stacking_relative_position_of_display_port;
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
        for fragment in &self.fragments.fragments {
            overflow = overflow.union(&fragment.compute_overflow())
        }
        overflow
    }

    fn iterate_through_fragment_border_boxes(&self,
                                             iterator: &mut FragmentBorderBoxIterator,
                                             level: i32,
                                             stacking_context_position: &Point2D<Au>) {
        // FIXME(#2795): Get the real container size.
        for fragment in &self.fragments.fragments {
            if !iterator.should_process(fragment) {
                continue
            }

            let stacking_relative_position = &self.base.stacking_relative_position;
            let relative_containing_block_size =
                &self.base.absolute_position_info.relative_containing_block_size;
            let relative_containing_block_mode =
                self.base.absolute_position_info.relative_containing_block_mode;
            iterator.process(fragment,
                             level,
                             &fragment.stacking_relative_border_box(stacking_relative_position,
                                                                    relative_containing_block_size,
                                                                    relative_containing_block_mode,
                                                                    CoordinateSystem::Own)
                                      .translate(stacking_context_position))
        }
    }

    fn mutate_fragments(&mut self, mutator: &mut FnMut(&mut Fragment)) {
        for fragment in &mut self.fragments.fragments {
            (*mutator)(fragment)
        }
    }

    fn contains_positioned_fragments(&self) -> bool {
        self.fragments.fragments.iter().any(|fragment| {
            fragment.style.get_box().position != position::T::static_
        })
    }

    fn contains_relatively_positioned_fragments(&self) -> bool {
        self.fragments.fragments.iter().any(|fragment| {
            fragment.style.get_box().position == position::T::relative
        })
    }

    fn generated_containing_block_size(&self, for_flow: OpaqueFlow) -> LogicalSize<Au> {
        let mut containing_block_size = LogicalSize::new(self.base.writing_mode, Au(0), Au(0));
        for index in self.containing_block_range_for_flow(for_flow).each_index() {
            let fragment = &self.fragments.fragments[index.get() as usize];
            containing_block_size.inline = containing_block_size.inline +
                fragment.border_box.size.inline;
            containing_block_size.block = max(containing_block_size.block,
                                              fragment.border_box.size.block)
        }
        containing_block_size
    }
}

impl fmt::Debug for InlineFlow {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?} - {:x} - {:?}", self.class(), self.base.debug_id(), self.fragments)
    }
}

#[derive(Clone)]
pub struct InlineFragmentNodeInfo {
    pub address: OpaqueNode,
    pub style: Arc<ComputedValues>,
    pub pseudo: PseudoElementType<()>,
}

#[derive(Clone)]
pub struct InlineFragmentContext {
    pub nodes: Vec<InlineFragmentNodeInfo>,
}

impl InlineFragmentContext {
    pub fn new() -> InlineFragmentContext {
        InlineFragmentContext {
            nodes: vec!(),
        }
    }

    #[inline]
    pub fn contains_node(&self, node_address: OpaqueNode) -> bool {
        self.nodes.iter().position(|node| node.address == node_address).is_some()
    }

    fn ptr_eq(&self, other: &InlineFragmentContext) -> bool {
        if self.nodes.len() != other.nodes.len() {
            return false
        }
        for (this_node, other_node) in self.nodes.iter().zip(&other.nodes) {
            if !util::arc_ptr_eq(&this_node.style, &other_node.style) {
                return false
            }
        }
        true
    }
}

fn inline_contexts_are_equal(inline_context_a: &Option<InlineFragmentContext>,
                             inline_context_b: &Option<InlineFragmentContext>)
                             -> bool {
    match (inline_context_a, inline_context_b) {
        (&Some(ref inline_context_a), &Some(ref inline_context_b)) => {
            inline_context_a.ptr_eq(inline_context_b)
        }
        (&None, &None) => true,
        (&Some(_), &None) | (&None, &Some(_)) => false,
    }
}

/// Block-size above the baseline, depth below the baseline, and ascent for a fragment. See CSS 2.1
/// § 10.8.1.
#[derive(Clone, Copy, Debug, RustcEncodable)]
pub struct InlineMetrics {
    pub block_size_above_baseline: Au,
    pub depth_below_baseline: Au,
    pub ascent: Au,
}

impl InlineMetrics {
    /// Creates a new set of inline metrics.
    pub fn new(block_size_above_baseline: Au, depth_below_baseline: Au, ascent: Au)
               -> InlineMetrics {
        InlineMetrics {
            block_size_above_baseline: block_size_above_baseline,
            depth_below_baseline: depth_below_baseline,
            ascent: ascent,
        }
    }

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
    pub fn from_block_height(font_metrics: &FontMetrics,
                             block_height: Au,
                             block_start_margin: Au,
                             block_end_margin: Au)
                             -> InlineMetrics {
        let leading = block_height + block_start_margin + block_end_margin -
            (font_metrics.ascent + font_metrics.descent);
        InlineMetrics {
            block_size_above_baseline: font_metrics.ascent + leading.scale_by(0.5),
            depth_below_baseline: font_metrics.descent + leading.scale_by(0.5),
            ascent: font_metrics.ascent + leading.scale_by(0.5) - block_start_margin,
        }
    }

    pub fn block_size(&self) -> Au {
        self.block_size_above_baseline + self.depth_below_baseline
    }

    pub fn max(&self, other: &InlineMetrics) -> InlineMetrics {
        InlineMetrics {
            block_size_above_baseline: max(self.block_size_above_baseline,
                                           other.block_size_above_baseline),
            depth_below_baseline: max(self.depth_below_baseline, other.depth_below_baseline),
            ascent: max(self.ascent, other.ascent),
        }
    }
}

#[derive(Copy, Clone, PartialEq)]
enum LineFlushMode {
    No,
    Flush,
}
