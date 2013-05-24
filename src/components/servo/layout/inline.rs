/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use core::cell::Cell;
use core;
use dom::node::AbstractNode;
use layout::box::{CannotSplit, GenericRenderBoxClass, ImageRenderBoxClass, RenderBox};
use layout::box::{SplitDidFit, SplitDidNotFit, TextRenderBoxClass, UnscannedTextRenderBoxClass};
use layout::context::LayoutContext;
use layout::debug::{BoxedDebugMethods, BoxedMutDebugMethods, DebugMethods};
use layout::display_list_builder::DisplayListBuilder;
use layout::flow::{FlowContext, FlowData, InlineFlow};
use layout::text::{UnscannedMethods, adapt_textbox_with_range};

use core::util;
use geom::{Point2D, Rect, Size2D};
use gfx::display_list::DisplayList;
use gfx::geometry::Au;
use gfx::text::text_run::TextRun;
use gfx::text::util::*;
use newcss::values::{CSSTextAlignCenter, CSSTextAlignJustify, CSSTextAlignLeft};
use newcss::values::{CSSTextAlignRight};
use newcss::values::CSSTextDecorationUnderline;
use newcss::values::CSSTextDecoration;
use servo_util::range::Range;
use std::deque::Deque;

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

pub struct NodeRange {
    node: AbstractNode,
    range: Range,
}

pub impl NodeRange {
    fn new(node: AbstractNode, range: &Range) -> NodeRange {
        NodeRange { node: node, range: copy *range }
    }
}

struct ElementMapping {
    priv entries: ~[NodeRange],
}

impl ElementMapping {
    pub fn new() -> ElementMapping {
        ElementMapping { entries: ~[] }
    }

    pub fn add_mapping(&mut self, node: AbstractNode, range: &Range) {
        self.entries.push(NodeRange::new(node, range))
    }

    pub fn each(&self, callback: &fn(nr: &NodeRange) -> bool) -> bool {
        for self.entries.each |nr| {
            if !callback(nr) {
                break
            }
        }
        true
    }

    pub fn eachi(&self, callback: &fn(i: uint, nr: &NodeRange) -> bool) -> bool {
        for self.entries.eachi |i, nr| {
            if !callback(i, nr) {
                break
            }
        }
        true
    }

    pub fn eachi_mut(&self, callback: &fn(i: uint, nr: &NodeRange) -> bool) -> bool {
        for self.entries.eachi |i, nr| {
            if !callback(i, nr) {
                break
            }
        }
        true
    }

    pub fn repair_for_box_changes(&mut self, old_boxes: &[RenderBox], new_boxes: &[RenderBox]) {
        let entries = &mut self.entries;

        debug!("--- Old boxes: ---");
        for old_boxes.eachi |i, box| {
            debug!("%u --> %s", i, box.debug_str());
        }
        debug!("------------------");

        debug!("--- New boxes: ---");
        for new_boxes.eachi |i, box| {
            debug!("%u --> %s", i, box.debug_str());
        }
        debug!("------------------");

        debug!("--- Elem ranges before repair: ---");
        for entries.eachi |i: uint, nr: &NodeRange| {
            debug!("%u: %? --> %s", i, nr.range, nr.node.debug_str());
        }
        debug!("----------------------------------");

        let mut old_i = 0;
        let mut new_j = 0;

        struct WorkItem {
            begin_idx: uint,
            entry_idx: uint,
        };
        let mut repair_stack : ~[WorkItem] = ~[];

            // index into entries
            let mut entries_k = 0;

            while old_i < old_boxes.len() {
                debug!("repair_for_box_changes: Considering old box %u", old_i);
                // possibly push several items
                while entries_k < entries.len() && old_i == entries[entries_k].range.begin() {
                    let item = WorkItem {begin_idx: new_j, entry_idx: entries_k};
                    debug!("repair_for_box_changes: Push work item for elem %u: %?", entries_k, item);
                    repair_stack.push(item);
                    entries_k += 1;
                }
                // XXX: the following loop form causes segfaults; assigning to locals doesn't.
                // while new_j < new_boxes.len() && old_boxes[old_i].d().node != new_boxes[new_j].d().node {
                while new_j < new_boxes.len() {
                    let should_leave = do old_boxes[old_i].with_imm_base |old_box_base| {
                        do new_boxes[new_j].with_imm_base |new_box_base| {
                            old_box_base.node != new_box_base.node
                        }
                    };
                    if should_leave {
                        break
                    }

                    debug!("repair_for_box_changes: Slide through new box %u", new_j);
                    new_j += 1;
                }

                old_i += 1;

                // possibly pop several items
                while repair_stack.len() > 0 && old_i == entries[repair_stack.last().entry_idx].range.end() {
                    let item = repair_stack.pop();
                    debug!("repair_for_box_changes: Set range for %u to %?",
                           item.entry_idx, Range::new(item.begin_idx, new_j - item.begin_idx));
                    entries[item.entry_idx].range = Range::new(item.begin_idx, new_j - item.begin_idx);
                }
            }
        debug!("--- Elem ranges after repair: ---");
        for entries.eachi |i: uint, nr: &NodeRange| {
            debug!("%u: %? --> %s", i, nr.range, nr.node.debug_str());
        }
        debug!("----------------------------------");
    }
}

/// A stack-allocated object for scanning an inline flow into `TextRun`-containing `TextBox`es.
struct TextRunScanner {
    clump: Range,
}

impl TextRunScanner {
    fn new() -> TextRunScanner {
        TextRunScanner {
            clump: Range::empty(),
        }
    }
}

impl TextRunScanner {
    fn scan_for_runs(&mut self, ctx: &mut LayoutContext, flow: FlowContext) {
        let inline = flow.inline();
        assert!(inline.boxes.len() > 0);
        debug!("TextRunScanner: scanning %u boxes for text runs...", inline.boxes.len());

        let mut out_boxes = ~[];
        for uint::range(0, flow.inline().boxes.len()) |box_i| {
            debug!("TextRunScanner: considering box: %?", flow.inline().boxes[box_i].debug_str());
            if box_i > 0 && !can_coalesce_text_nodes(flow.inline().boxes, box_i-1, box_i) {
                self.flush_clump_to_list(ctx, flow, &mut out_boxes);
            }
            self.clump.extend_by(1);
        }
        // handle remaining clumps
        if self.clump.length() > 0 {
            self.flush_clump_to_list(ctx, flow, &mut out_boxes);
        }

        debug!("TextRunScanner: swapping out boxes.");

        // Swap out the old and new box list of the flow.
        flow.inline().boxes = out_boxes;

        // A helper function.
        fn can_coalesce_text_nodes(boxes: &[RenderBox], left_i: uint, right_i: uint) -> bool {
            assert!(left_i < boxes.len());
            assert!(right_i > 0 && right_i < boxes.len());
            assert!(left_i != right_i);

            let (left, right) = (boxes[left_i], boxes[right_i]);
            match (left, right) {
                (UnscannedTextRenderBoxClass(*), UnscannedTextRenderBoxClass(*)) => {
                    left.can_merge_with_box(right)
                }
                (_, _) => false
            }
        }
    }

    /// A "clump" is a range of inline flow leaves that can be merged together into a single
    /// `RenderBox`. Adjacent text with the same style can be merged, and nothing else can. 
    ///
    /// The flow keeps track of the `RenderBox`es contained by all non-leaf DOM nodes. This is
    /// necessary for correct painting order. Since we compress several leaf `RenderBox`es here,
    /// the mapping must be adjusted.
    ///
    /// N.B. `in_boxes` is passed by reference, since the old code used a `DVec`. The caller is
    /// responsible for swapping out the list. It is not clear to me (pcwalton) that this is still
    /// necessary.
    fn flush_clump_to_list(&mut self,
                           ctx: &mut LayoutContext, 
                           flow: FlowContext,
                           out_boxes: &mut ~[RenderBox]) {
        let inline = &mut *flow.inline();
        let in_boxes = &inline.boxes;

        fn has_underline(decoration: CSSTextDecoration) -> bool{
            match decoration {
                CSSTextDecorationUnderline => true,
                _ => false
            }
        }

        assert!(self.clump.length() > 0);

        debug!("TextRunScanner: flushing boxes in range=%?", self.clump);
        let is_singleton = self.clump.length() == 1;
        let is_text_clump = match in_boxes[self.clump.begin()] {
            UnscannedTextRenderBoxClass(*) => true,
            _ => false
        };

        match (is_singleton, is_text_clump) {
            (false, false) => {
                fail!(~"WAT: can't coalesce non-text nodes in flush_clump_to_list()!")
            }
            (true, false) => { 
                debug!("TextRunScanner: pushing single non-text box in range: %?", self.clump);
                out_boxes.push(in_boxes[self.clump.begin()]);
            },
            (true, true)  => {
                let old_box = in_boxes[self.clump.begin()];
                let text = old_box.raw_text();
                let font_style = old_box.font_style();
                let underline = has_underline(old_box.text_decoration());

                // TODO(#115): Use the actual CSS `white-space` property of the relevant style.
                let compression = CompressWhitespaceNewline;

                let transformed_text = transform_text(text, compression);

                // TODO(#177): Text run creation must account for the renderability of text by
                // font group fonts. This is probably achieved by creating the font group above
                // and then letting `FontGroup` decide which `Font` to stick into the text run.
                let fontgroup = ctx.font_ctx.get_resolved_font_for_style(&font_style);
                let run = @fontgroup.create_textrun(transformed_text, underline);

                debug!("TextRunScanner: pushing single text box in range: %?", self.clump);
                let new_box = do old_box.with_imm_base |old_box_base| {
                    let range = Range::new(0, run.char_len());
                    @mut adapt_textbox_with_range(*old_box_base, run, range)
                };

                out_boxes.push(TextRenderBoxClass(new_box));
            },
            (false, true) => {
                // TODO(#115): Use the actual CSS `white-space` property of the relevant style.
                let compression = CompressWhitespaceNewline;

                // First, transform/compress text of all the nodes.
                let transformed_strs: ~[~str] = do vec::from_fn(self.clump.length()) |i| {
                    // TODO(#113): We should be passing the compression context between calls to
                    // `transform_text`, so that boxes starting and/or ending with whitespace can
                    // be compressed correctly with respect to the text run.
                    let idx = i + self.clump.begin();
                    transform_text(in_boxes[idx].raw_text(), compression)
                };

                // Next, concatenate all of the transformed strings together, saving the new
                // character indices.
                let mut run_str: ~str = ~"";
                let mut new_ranges: ~[Range] = ~[];
                let mut char_total = 0;
                for uint::range(0, transformed_strs.len()) |i| {
                    let added_chars = str::char_len(transformed_strs[i]);
                    new_ranges.push(Range::new(char_total, added_chars));
                    str::push_str(&mut run_str, transformed_strs[i]);
                    char_total += added_chars;
                }

                // Now create the run.
                //
                // TODO(#177): Text run creation must account for the renderability of text by
                // font group fonts. This is probably achieved by creating the font group above
                // and then letting `FontGroup` decide which `Font` to stick into the text run.
                let font_style = in_boxes[self.clump.begin()].font_style();
                let fontgroup = ctx.font_ctx.get_resolved_font_for_style(&font_style);
                let underline = has_underline(in_boxes[self.clump.begin()].text_decoration());

                // TextRuns contain a cycle which is usually resolved by the teardown
                // sequence. If no clump takes ownership, however, it will leak.
                let clump = self.clump;
                let run = if clump.length() != 0 {
                    Some(@TextRun::new(fontgroup.fonts[0], run_str, underline))
                } else {
                    None
                };

                // Make new boxes with the run and adjusted text indices.
                debug!("TextRunScanner: pushing box(es) in range: %?", self.clump);
                for clump.eachi |i| {
                    let range = new_ranges[i - self.clump.begin()];
                    if range.length() == 0 { 
                        error!("Elided an `UnscannedTextbox` because it was zero-length after \
                                compression; %s",
                               in_boxes[i].debug_str());
                        loop
                    }

                    do in_boxes[i].with_imm_base |base| {
                        let new_box = @mut adapt_textbox_with_range(*base, run.get(), range);
                        out_boxes.push(TextRenderBoxClass(new_box));
                    }
                }
            }
        } // End of match.
    
        debug!("--- In boxes: ---");
        for in_boxes.eachi |i, box| {
            debug!("%u --> %s", i, box.debug_str());
        }
        debug!("------------------");

        debug!("--- Out boxes: ---");
        for out_boxes.eachi |i, box| {
            debug!("%u --> %s", i, box.debug_str());
        }
        debug!("------------------");

        debug!("--- Elem ranges: ---");
        for inline.elems.eachi_mut |i: uint, nr: &NodeRange| {
            debug!("%u: %? --> %s", i, nr.range, nr.node.debug_str()); ()
        }
        debug!("--------------------");

        let end = self.clump.end(); // FIXME: borrow checker workaround
        self.clump.reset(end, 0);
    } // End of `flush_clump_to_list`.
}

struct PendingLine {
    range: Range,
    width: Au
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
            pending_line: PendingLine {mut range: Range::empty(), mut width: Au(0)},
            line_spans: ~[]
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
        self.pending_line.width = Au(0);
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

        let slack_width = self.flow.position().size.width - self.pending_line.width;
        match linebox_align {
            // So sorry, but justified text is more complicated than shuffling linebox coordinates.
            // TODO(Issue #213): implement `text-align: justify`
            CSSTextAlignLeft | CSSTextAlignJustify => {
                for line_range.eachi |i| {
                    do self.new_boxes[i].with_mut_base |base| {
                        base.position.origin.x = offset_x;
                        offset_x += base.position.size.width;
                    }
                }
            },
            CSSTextAlignCenter => {
                offset_x = slack_width.scale_by(0.5f);
                for line_range.eachi |i| {
                    do self.new_boxes[i].with_mut_base |base| {
                        base.position.origin.x = offset_x;
                        offset_x += base.position.size.width;
                    }
                }
            },
            CSSTextAlignRight => {
                offset_x = slack_width;
                for line_range.eachi |i| {
                    do self.new_boxes[i].with_mut_base |base| {
                        base.position.origin.x = offset_x;
                        offset_x += base.position.size.width;
                    }
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
        let remaining_width = self.flow.position().size.width - self.pending_line.width;
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
            assert!(self.new_boxes.len() <= (core::u16::max_value as uint));
            self.pending_line.range.reset(self.new_boxes.len(), 0);
        }
        self.pending_line.range.extend_by(1);
        self.pending_line.width += box.position().size.width;
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
        for self.boxes.each |box| {
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
        let mut scanner = TextRunScanner::new();
        scanner.scan_for_runs(ctx, InlineFlow(self));

        {
            let this = &mut *self;

            let mut min_width = Au(0);
            let mut pref_width = Au(0);

            for this.boxes.each |box| {
                debug!("FlowContext[%d]: measuring %s", self.common.id, box.debug_str());
                min_width = Au::max(min_width, box.get_min_width(ctx));
                pref_width = Au::max(pref_width, box.get_pref_width(ctx));
            }

            this.common.min_width = min_width;
            this.common.pref_width = pref_width;
        }
    }

    /// Recursively (top-down) determines the actual width of child contexts and boxes. When called
    /// on this context, the context has had its width set by the parent context.
    pub fn assign_widths_inline(@mut self, ctx: &mut LayoutContext) {
        // Initialize content box widths if they haven't been initialized already.
        //
        // TODO: Combine this with `LineboxScanner`'s walk in the box list, or put this into
        // `RenderBox`.
        {
            let this = &mut *self;
            for this.boxes.each |&box| {
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

        let mut scanner = LineboxScanner::new(InlineFlow(self));
        scanner.scan_for_lines(ctx);
   
        // There are no child contexts, so stop here.

        // TODO(Issue #225): once there are 'inline-block' elements, this won't be
        // true.  In that case, set the InlineBlockBox's width to the
        // shrink-to-fit width, perform inline flow, and set the block
        // flow context's width as the assigned width of the
        // 'inline-block' box that created this flow before recursing.
    }

    pub fn assign_height_inline(&mut self, _: &mut LayoutContext) {
        // TODO(#226): Get the CSS `line-height` property from the containing block's style to
        // determine minimum linebox height.
        //
        // TODO(#226): Get the CSS `line-height` property from each non-replaced inline element to
        // determine its height for computing linebox height.

        let line_height = Au::from_px(20);
        let mut cur_y = Au(0);

        for self.lines.eachi |i, line_span| {
            debug!("assign_height_inline: processing line %u with box span: %?", i, line_span);

            // These coordinates are relative to the left baseline.
            let mut linebox_bounding_box = Au::zero_rect();
            let boxes = &mut self.boxes;
            for line_span.eachi |box_i| {
                let cur_box = boxes[box_i]; // FIXME: borrow checker workaround

                // Compute the height of each box.
                match cur_box {
                    ImageRenderBoxClass(image_box) => {
                        let size = image_box.image.get_size();
                        let height = Au::from_px(size.get_or_default(Size2D(0, 0)).height);
                        image_box.base.position.size.height = height;
                    }
                    TextRenderBoxClass(*) => {
                        // Text boxes are preinitialized.
                    }
                    GenericRenderBoxClass(generic_box) => {
                        // TODO(Issue #225): There will be different cases here for `inline-block`
                        // and other replaced content.
                        // FIXME(pcwalton): This seems clownshoes; can we remove?
                        generic_box.position.size.height = Au::from_px(30);
                    }
                    // FIXME(pcwalton): This isn't very type safe!
                    _ => {
                        fail!(fmt!("Tried to assign height to unknown Box variant: %s",
                                   cur_box.debug_str()))
                    }
                }

                // Compute the bounding rect with the left baseline as origin. Determining line box
                // height is a matter of lining up ideal baselines and then taking the union of all
                // these rects.
                let bounding_box = match cur_box {
                    // Adjust to baseline coordinates.
                    //
                    // TODO(#227): Use left/right margins, border, padding for nonreplaced content,
                    // and also use top/bottom margins, border, padding for replaced or
                    // inline-block content.
                    //
                    // TODO(#225): Use height, width for `inline-block` and other replaced content.
                    ImageRenderBoxClass(*) | GenericRenderBoxClass(*) => {
                        let height = cur_box.position().size.height;
                        cur_box.position().translate(&Point2D(Au(0), -height))
                    },

                    // Adjust the bounding box metric to the box's horizontal offset.
                    //
                    // TODO: We can use font metrics directly instead of re-measuring for the
                    // bounding box.
                    TextRenderBoxClass(text_box) => {
                        let range = &text_box.text_data.range;
                        let run = &text_box.text_data.run;
                        let text_bounds = run.metrics_for_range(range).bounding_box;
                        text_bounds.translate(&Point2D(text_box.base.position.origin.x, Au(0)))
                    },

                    _ => {
                        fail!(fmt!("Tried to compute bounding box of unknown Box variant: %s",
                                   cur_box.debug_str()))
                    }
                };

                debug!("assign_height_inline: bounding box for box b%d = %?",
                       cur_box.id(),
                       bounding_box);

                linebox_bounding_box = linebox_bounding_box.union(&bounding_box);

                debug!("assign_height_inline: linebox bounding box = %?", linebox_bounding_box);
            }

            let linebox_height = linebox_bounding_box.size.height;
            let baseline_offset = -linebox_bounding_box.origin.y;

            // Now go back and adjust the Y coordinates to match the baseline we determined.
            for line_span.eachi |box_i| {
                let cur_box = boxes[box_i];

                // TODO(#226): This is completely wrong. We need to use the element's `line-height`
                // when calculating line box height. Then we should go back over and set Y offsets
                // according to the `vertical-align` property of the containing block.
                let halfleading = match cur_box {
                    TextRenderBoxClass(text_box) => {
                        (text_box.text_data.run.font.metrics.em_size - line_height).scale_by(0.5)
                    },
                    _ => Au(0),
                };

                do cur_box.with_mut_base |base| {
                    let height = base.position.size.height;
                    base.position.origin.y = cur_y + halfleading + baseline_offset - height;
                }
            }
            
            cur_y += Au::max(line_height, linebox_height);
        } // End of `lines.each` loop.

        self.common.position.size.height = cur_y;
    }

    pub fn build_display_list_inline(&self,
                                     builder: &DisplayListBuilder,
                                     dirty: &Rect<Au>, 
                                     offset: &Point2D<Au>,
                                     list: &Cell<DisplayList>) {
        // TODO(#228): Once we form line boxes and have their cached bounds, we can be smarter and
        // not recurse on a line if nothing in it can intersect the dirty region.
        debug!("FlowContext[%d]: building display list for %u inline boxes",
               self.common.id,
               self.boxes.len());

        for self.boxes.each |box| {
            box.build_display_list(builder, dirty, offset, list)
        }

        // TODO(#225): Should `inline-block` elements have flows as children of the inline flow or
        // should the flow be nested inside the box somehow?
    }
}

