/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use std::cell::Cell;
use layout::box::{CannotSplit, GenericRenderBoxClass, ImageRenderBoxClass, RenderBox};
use layout::box::{SplitDidFit, SplitDidNotFit, TextRenderBoxClass};
use layout::context::LayoutContext;
use layout::display_list_builder::{DisplayListBuilder, ExtraDisplayListData};
use layout::flow::{FlowContext, FlowData, InlineFlow};
use layout::float_context::FloatContext;
use layout::util::{ElementMapping};

use std::u16;
use std::util;
use geom::{Point2D, Rect, Size2D};
use gfx::display_list::DisplayList;
use gfx::geometry::Au;
use newcss::values::{CSSTextAlignLeft, CSSTextAlignCenter, CSSTextAlignRight, CSSTextAlignJustify};
use newcss::units::{Em, Px, Pt};
use newcss::values::{CSSLineHeightNormal, CSSLineHeightNumber, CSSLineHeightLength, CSSLineHeightPercentage};
use servo_util::range::Range;
use servo_util::tree::{TreeNodeRef, TreeUtils};
use extra::deque::Deque;

/*
Lineboxes are represented as offsets into the child list, rather than
as an object that "owns" boxes. Choosing a different set of line
breaks requires a new list of offsets, and possibly some splitting and
merging of TextBoxes.

A similar list will keep track of the mapping between CSS boxes and
the corresponding render boxes in the inline flow.

After line breaks are determined, render boxes in the inline flow may
overlap visually. For example, in the case of nested inline CSS boxes,
outer inlines must be at least as large as the inner inlines, for
purposes of drawing noninherited things like backgrounds, borders,
outlines.

N.B. roc has an alternative design where the list instead consists of
things like "start outer box, text, start inner box, text, end inner
box, text, end outer box, text". This seems a little complicated to
serve as the starting point, but the current design doesn't make it
hard to try out that alternative.
*/

struct PendingLine {
    range: Range,
    bounds: Rect<Au>
}

struct LineboxScanner {
    flow: FlowContext,
    new_boxes: ~[RenderBox],
    work_list: @mut Deque<RenderBox>,
    pending_line: PendingLine,
    line_spans: ~[Range],
}

impl LineboxScanner {
    pub fn new(inline: FlowContext) -> LineboxScanner {
        assert!(inline.starts_inline_flow());

        LineboxScanner {
            flow: inline,
            new_boxes: ~[],
            work_list: @mut Deque::new(),
            pending_line: PendingLine {range: Range::empty(), bounds: Rect(Point2D(Au(0), Au(0)), Size2D(Au(0), Au(0)))},
            line_spans: ~[],
        }
    }

    fn reset_scanner(&mut self) {
        debug!("Resetting line box scanner's state for flow f%d.", self.flow.id());
        self.line_spans = ~[];
        self.new_boxes = ~[];
        self.reset_linebox();
    }

    fn reset_linebox(&mut self) {
        self.pending_line.range.reset(0,0);
        self.pending_line.bounds = Rect(Point2D(Au(0), Au(0)), Size2D(Au(0), Au(0)));
    }

    pub fn scan_for_lines(&mut self, ctx: &LayoutContext) {
        self.reset_scanner();

        { // FIXME: manually control borrow length
            let inline: &InlineFlowData = self.flow.inline();
            let mut i = 0u;

            loop {
                // acquire the next box to lay out from work list or box list
                let cur_box = if self.work_list.is_empty() {
                    if i == inline.boxes.len() {
                        break
                    }
                    let box = inline.boxes[i]; i += 1;
                    debug!("LineboxScanner: Working with box from box list: b%d", box.id());
                    box
                } else {
                    let box = self.work_list.pop_front();
                    debug!("LineboxScanner: Working with box from work list: b%d", box.id());
                    box
                };

                let box_was_appended = self.try_append_to_line(ctx, cur_box);
                if !box_was_appended {
                    debug!("LineboxScanner: Box wasn't appended, because line %u was full.",
                           self.line_spans.len());
                    self.flush_current_line();
                } else {
                    debug!("LineboxScanner: appended a box to line %u", self.line_spans.len());
                }
            }

            if self.pending_line.range.length() > 0 {
                debug!("LineboxScanner: Partially full linebox %u left at end of scanning.",
                       self.line_spans.len());
                self.flush_current_line();
            }
        }

        { // FIXME: scope the borrow
            let inline: &mut InlineFlowData = self.flow.inline();
            inline.elems.repair_for_box_changes(inline.boxes, self.new_boxes);
        }
        self.swap_out_results();
    }

    fn swap_out_results(&mut self) {
        debug!("LineboxScanner: Propagating scanned lines[n=%u] to inline flow f%d",
               self.line_spans.len(),
               self.flow.id());

        let inline: &mut InlineFlowData = self.flow.inline();
        util::swap(&mut inline.boxes, &mut self.new_boxes);
        util::swap(&mut inline.lines, &mut self.line_spans);
    }

    fn flush_current_line(&mut self) {
        debug!("LineboxScanner: Flushing line %u: %?",
               self.line_spans.len(), self.pending_line);
        // set box horizontal offsets
        let line_range = self.pending_line.range;
        let mut offset_x = Au(0);
        // TODO(Issue #199): interpretation of CSS 'direction' will change how boxes are positioned.
        debug!("LineboxScanner: Setting horizontal offsets for boxes in line %u range: %?",
               self.line_spans.len(), line_range);

        // Get the text alignment.
        // TODO(Issue #222): use 'text-align' property from InlineFlow's
        // block container, not from the style of the first box child.
        let linebox_align;
        if self.pending_line.range.begin() < self.new_boxes.len() {
            let first_box = self.new_boxes[self.pending_line.range.begin()];
            linebox_align = first_box.text_align();
        } else {
            // Nothing to lay out, so assume left alignment.
            linebox_align = CSSTextAlignLeft;
        }

        let slack_width = self.flow.position().size.width - self.pending_line.bounds.size.width;
        match linebox_align {
            // So sorry, but justified text is more complicated than shuffling linebox coordinates.
            // TODO(Issue #213): implement `text-align: justify`
            CSSTextAlignLeft | CSSTextAlignJustify => {
                for line_range.eachi |i| {
                    do self.new_boxes[i].with_mut_base |base| {
                        base.position.origin.x = offset_x;
                        offset_x = offset_x + base.position.size.width;
                    };
                }
            },
            CSSTextAlignCenter => {
                offset_x = slack_width.scale_by(0.5f);
                for line_range.eachi |i| {
                    do self.new_boxes[i].with_mut_base |base| {
                        base.position.origin.x = offset_x;
                        offset_x = offset_x + base.position.size.width;
                    };
                }
            },
            CSSTextAlignRight => {
                offset_x = slack_width;
                for line_range.eachi |i| {
                    do self.new_boxes[i].with_mut_base |base| {
                        base.position.origin.x = offset_x;
                        offset_x = offset_x + base.position.size.width;
                    };
                }
            },
        }

        // clear line and add line mapping
        debug!("LineboxScanner: Saving information for flushed line %u.", self.line_spans.len());
        self.line_spans.push(line_range);
        self.reset_linebox();
    }

    // return value: whether any box was appended.
    fn try_append_to_line(&mut self, ctx: &LayoutContext, in_box: RenderBox) -> bool {
        let remaining_width = self.flow.position().size.width - self.pending_line.bounds.size.width;
        let in_box_width = in_box.position().size.width;
        let line_is_empty: bool = self.pending_line.range.length() == 0;

        debug!("LineboxScanner: Trying to append box to line %u (box width: %?, remaining width: \
                %?): %s",
               self.line_spans.len(),
               in_box_width,
               remaining_width,
               in_box.debug_str());

        if in_box_width <= remaining_width {
            debug!("LineboxScanner: case=box fits without splitting");
            self.push_box_to_line(in_box);
            return true;
        }

        if !in_box.can_split() {
            // force it onto the line anyway, if its otherwise empty
            // TODO(Issue #224): signal that horizontal overflow happened?
            if line_is_empty {
                debug!("LineboxScanner: case=box can't split and line %u is empty, so \
                        overflowing.",
                       self.line_spans.len());
                self.push_box_to_line(in_box);
                return true;
            } else {
                debug!("LineboxScanner: Case=box can't split, not appending.");
                return false;
            }
        }

        // not enough width; try splitting?
        match in_box.split_to_width(ctx, remaining_width, line_is_empty) {
            CannotSplit(_) => {
                error!("LineboxScanner: Tried to split unsplittable render box! %s",
                       in_box.debug_str());
                return false;
            },
            SplitDidFit(left, right) => {
                debug!("LineboxScanner: case=split box did fit; deferring remainder box.");
                match (left, right) {
                    (Some(left_box), Some(right_box)) => {
                        self.push_box_to_line(left_box);
                        self.work_list.add_front(right_box);
                    },
                    (Some(left_box), None) => self.push_box_to_line(left_box),
                    (None, Some(right_box)) => self.push_box_to_line(right_box),
                    (None, None) => error!("LineboxScanner: This split case makes no sense!"),
                }
                return true;
            },
            SplitDidNotFit(left, right) => {
                if line_is_empty {
                    debug!("LineboxScanner: case=split box didn't fit and line %u is empty, so overflowing and deferring remainder box.",
                          self.line_spans.len());
                    // TODO(Issue #224): signal that horizontal overflow happened?
                    match (left, right) {
                        (Some(left_box), Some(right_box)) => {
                            self.push_box_to_line(left_box);
                            self.work_list.add_front(right_box);
                        },
                        (Some(left_box), None) => {
                            self.push_box_to_line(left_box);
                        }
                        (None, Some(right_box)) => {
                            self.push_box_to_line(right_box);
                        },
                        (None, None) => {
                            error!("LineboxScanner: This split case makes no sense!");
                        }
                    }
                    return true;
                } else {
                    debug!("LineboxScanner: case=split box didn't fit, not appending and deferring original box.");
                    self.work_list.add_front(in_box);
                    return false;
                }
            }
        }
    }

    // unconditional push
    fn push_box_to_line(&mut self, box: RenderBox) {
        debug!("LineboxScanner: Pushing box b%d to line %u", box.id(), self.line_spans.len());

        if self.pending_line.range.length() == 0 {
            assert!(self.new_boxes.len() <= (u16::max_value as uint));
            self.pending_line.range.reset(self.new_boxes.len(), 0);
        }
        self.pending_line.range.extend_by(1);
        self.pending_line.bounds.size.width = self.pending_line.bounds.size.width + box.position().size.width;
        self.new_boxes.push(box);
    }
}

pub struct InlineFlowData {
    /// Data common to all flows.
    common: FlowData,

    // A vec of all inline render boxes. Several boxes may
    // correspond to one Node/Element.
    boxes: ~[RenderBox],
    // vec of ranges into boxes that represents line positions.
    // these ranges are disjoint, and are the result of inline layout.
    lines: ~[Range],
    // vec of ranges into boxes that represent elements. These ranges
    // must be well-nested, and are only related to the content of
    // boxes (not lines). Ranges are only kept for non-leaf elements.
    elems: ElementMapping
}

impl InlineFlowData {
    pub fn new(common: FlowData) -> InlineFlowData {
        InlineFlowData {
            common: common,
            boxes: ~[],
            lines: ~[],
            elems: ElementMapping::new(),
        }
    }

    pub fn teardown(&mut self) {
        self.common.teardown();
        for self.boxes.iter().advance |box| {
            box.teardown();
        }
        self.boxes = ~[];
    }
}

pub trait InlineLayout {
    fn starts_inline_flow(&self) -> bool;
}

impl InlineLayout for FlowContext {
    fn starts_inline_flow(&self) -> bool {
        match *self {
            InlineFlow(*) => true,
            _ => false
        }
    }
}

impl InlineFlowData {
    pub fn bubble_widths_inline(@mut self, ctx: &mut LayoutContext) {
        let mut num_floats = 0;

        for InlineFlow(self).each_child |kid| {
            do kid.with_mut_base |base| {
                num_floats += base.num_floats;
                base.floats_in = FloatContext::new(base.num_floats);
            }
        }

        {
            let this = &mut *self;

            let mut min_width = Au(0);
            let mut pref_width = Au(0);

            for this.boxes.iter().advance |box| {
                debug!("FlowContext[%d]: measuring %s", self.common.id, box.debug_str());
                min_width = Au::max(min_width, box.get_min_width(ctx));
                pref_width = Au::max(pref_width, box.get_pref_width(ctx));
            }

            this.common.min_width = min_width;
            this.common.pref_width = pref_width;
            this.common.num_floats = num_floats;
        }
    }

    /// Recursively (top-down) determines the actual width of child contexts and boxes. When called
    /// on this context, the context has had its width set by the parent context.
    pub fn assign_widths_inline(@mut self, _: &mut LayoutContext) {
        // Initialize content box widths if they haven't been initialized already.
        //
        // TODO: Combine this with `LineboxScanner`'s walk in the box list, or put this into
        // `RenderBox`.
        {
            let this = &mut *self;
            for this.boxes.iter().advance |&box| {
                match box {
                    ImageRenderBoxClass(image_box) => {
                        let size = image_box.image.get_size();
                        let width = Au::from_px(size.get_or_default(Size2D(0, 0)).width);
                        image_box.base.position.size.width = width;
                    }
                    TextRenderBoxClass(_) => {
                        // Text boxes are preinitialized.
                    }
                    GenericRenderBoxClass(generic_box) => {
                        // TODO(#225): There will be different cases here for `inline-block` and
                        // other replaced content.
                        // FIXME(pcwalton): This seems clownshoes; can we remove?
                        generic_box.position.size.width = Au::from_px(45);
                    }
                    // FIXME(pcwalton): This isn't very type safe!
                    _ => fail!(fmt!("Tried to assign width to unknown Box variant: %?", box)),
                }
            } // End of for loop.
        }

        for InlineFlow(self).each_child |kid| {
            do kid.with_mut_base |base| {
                base.position.size.width = self.common.position.size.width;
            }
        }
        // There are no child contexts, so stop here.

        // TODO(Issue #225): once there are 'inline-block' elements, this won't be
        // true.  In that case, set the InlineBlockBox's width to the
        // shrink-to-fit width, perform inline flow, and set the block
        // flow context's width as the assigned width of the
        // 'inline-block' box that created this flow before recursing.
    }

    pub fn assign_height_inline(@mut self, ctx: &mut LayoutContext) {

        for InlineFlow(self).each_child |kid| {
            kid.assign_height(ctx);
        }


        // TODO(eatkinson): line boxes need to shrink if there are floats
        let mut scanner = LineboxScanner::new(InlineFlow(self));
        scanner.scan_for_lines(ctx);
        self.common.floats_out = self.common.floats_in.clone();

        // TODO(#226): Get the CSS `line-height` property from the containing block's style to
        // determine minimum linebox height.
        //
        // TODO(#226): Get the CSS `line-height` property from each non-replaced inline element to
        // determine its height for computing linebox height.

        let mut cur_y = Au(0);

        for self.lines.iter().enumerate().advance |(i, line_span)| {
            debug!("assign_height_inline: processing line %u with box span: %?", i, line_span);

            // These coordinates are relative to the left baseline.
            let mut linebox_bounding_box = Au::zero_rect();
            let mut linebox_height = Au(0);
            let mut baseline_offset = Au(0);

            for line_span.eachi |box_i| {
                let cur_box = self.boxes[box_i];

                // Compute the height and bounding box of each box.
                let bounding_box = match cur_box {
                    ImageRenderBoxClass(image_box) => {
                        let size = image_box.image.get_size();
                        let height = Au::from_px(size.get_or_default(Size2D(0, 0)).height);
                        image_box.base.position.size.height = height;

                        if height > linebox_height {
                            linebox_height = height;
                        }

                        image_box.base.position.translate(&Point2D(Au(0), -height))
                    }
                    TextRenderBoxClass(text_box) => {

                        let range = &text_box.range;
                        let run = &text_box.run;
                        
                        // Compute the height based on the line-height and font size
                        let text_bounds = run.metrics_for_range(range).bounding_box;
                        let em_size = text_bounds.size.height;
                        let line_height = match cur_box.line_height() {
                            CSSLineHeightNormal => em_size.scale_by(1.14f),
                            CSSLineHeightNumber(l) => em_size.scale_by(l),
                            CSSLineHeightLength(Em(l)) => em_size.scale_by(l),
                            CSSLineHeightLength(Px(l)) => Au::from_frac_px(l),
                            CSSLineHeightLength(Pt(l)) => Au::from_pt(l),
                            CSSLineHeightPercentage(p) => em_size.scale_by(p / 100.0f)
                        };

                        // If this is the current tallest box then use it for baseline
                        // calculations.
                        // TODO: this will need to take into account type of line-height
                        // and the vertical-align value.
                        if line_height > linebox_height {
                            linebox_height = line_height;
                            // Offset from the top of the linebox is 1/2 of the leading + ascent
                            baseline_offset = text_box.run.font.metrics.ascent +
                                    (linebox_height - em_size).scale_by(0.5f);
                        }
                        text_bounds.translate(&Point2D(text_box.base.position.origin.x, Au(0)))
                    }
                    GenericRenderBoxClass(generic_box) => {
                        // TODO(Issue #225): There will be different cases here for `inline-block`
                        // and other replaced content.
                        // FIXME(pcwalton): This seems clownshoes; can we remove?
                        generic_box.position.size.height = Au::from_px(30);
                        if generic_box.position.size.height > linebox_height {
                            linebox_height = generic_box.position.size.height;
                        }
                        generic_box.position
                    }
                    // FIXME(pcwalton): This isn't very type safe!
                    _ => {
                        fail!(fmt!("Tried to assign height to unknown Box variant: %s",
                                   cur_box.debug_str()))
                    }
                };

                debug!("assign_height_inline: bounding box for box b%d = %?",
                       cur_box.id(),
                       bounding_box);

                linebox_bounding_box = linebox_bounding_box.union(&bounding_box);

                debug!("assign_height_inline: linebox bounding box = %?", linebox_bounding_box);
            }

            // Now go back and adjust the Y coordinates to match the baseline we determined.
            for line_span.eachi |box_i| {
                let cur_box = self.boxes[box_i];

                // TODO(#226): This is completely wrong. We need to use the element's `line-height`
                // when calculating line box height. Then we should go back over and set Y offsets
                // according to the `vertical-align` property of the containing block.
                let offset = match cur_box {
                    TextRenderBoxClass(text_box) => {
                        baseline_offset - text_box.run.font.metrics.ascent
                    },
                    _ => Au(0),
                };

                do cur_box.with_mut_base |base| {
                    base.position.origin.y = offset + cur_y;
                }
            }

            cur_y = cur_y + linebox_height;
        } // End of `lines.each` loop.

        self.common.position.size.height = cur_y;
    }

    pub fn build_display_list_inline<E:ExtraDisplayListData>(&self,
                                                             builder: &DisplayListBuilder,
                                                             dirty: &Rect<Au>,
                                                             offset: &Point2D<Au>,
                                                             list: &Cell<DisplayList<E>>) {
        // TODO(#228): Once we form line boxes and have their cached bounds, we can be smarter and
        // not recurse on a line if nothing in it can intersect the dirty region.
        debug!("FlowContext[%d]: building display list for %u inline boxes",
               self.common.id,
               self.boxes.len());

        for self.boxes.iter().advance |box| {
            box.build_display_list(builder, dirty, offset, list)
        }

        // TODO(#225): Should `inline-block` elements have flows as children of the inline flow or
        // should the flow be nested inside the box somehow?
    }
}

