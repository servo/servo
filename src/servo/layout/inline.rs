use core;
use core::cell::Cell;
use dom::node::AbstractNode;
use layout::box::*;
use layout::context::LayoutContext;
use layout::debug::{BoxedDebugMethods, BoxedMutDebugMethods, DebugMethods};
use layout::display_list_builder::DisplayListBuilder;
use layout::flow::{FlowContext, InlineFlow};
use layout::text::{UnscannedMethods, adapt_textbox_with_range};

use geom::{Point2D, Rect, Size2D};
use gfx::display_list::DisplayList;
use gfx::geometry::Au;
use gfx::image::holder;
use gfx::text::text_run::TextRun;
use gfx::text::util::*;
use gfx::util::range::Range;
use newcss::values::{CSSTextAlignCenter, CSSTextAlignJustify, CSSTextAlignLeft, CSSTextAlignRight};
use std::deque::Deque;
use core::util;

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

pub impl ElementMapping {
    fn new() -> ElementMapping {
        ElementMapping { entries: ~[] }
    }

    fn add_mapping(&mut self, node: AbstractNode, range: &Range) {
        self.entries.push(NodeRange::new(node, range))
    }

    fn each(&self, cb: &fn(nr: &NodeRange) -> bool) {
        do self.entries.each |nr| { cb(nr) }
    }

    fn eachi(&self, cb: &fn(i: uint, nr: &NodeRange) -> bool) {
        do self.entries.eachi |i, nr| { cb(i, nr) }
    }

    fn eachi_mut(&self, cb: &fn(i: uint, nr: &NodeRange) -> bool) {
        do self.entries.eachi |i, nr| { cb(i, nr) }
    }

    fn repair_for_box_changes(&mut self, old_boxes: &[@mut RenderBox], new_boxes: &[@mut RenderBox]) {
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
                    let o = old_boxes[old_i];
                    let n = new_boxes[new_j];
                    if o.d().node != n.d().node { break }
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

// stack-allocated object for scanning an inline flow into
// TextRun-containing TextBoxes.
priv struct TextRunScanner {
    clump: Range,
}

priv impl TextRunScanner {
    fn new() -> TextRunScanner {
        TextRunScanner {
            clump: Range::empty(),
        }
    }
}

priv impl TextRunScanner {
    fn scan_for_runs(&mut self, ctx: &mut LayoutContext, flow: @mut FlowContext) {
        let inline = flow.inline();
        assert!(inline.boxes.len() > 0);

        let in_boxes = &mut flow.inline().boxes;
        //do boxes.swap |in_boxes| {
            debug!("TextRunScanner: scanning %u boxes for text runs...", in_boxes.len());
            let mut out_boxes = ~[];

            for uint::range(0, in_boxes.len()) |box_i| {
                debug!("TextRunScanner: considering box: %?", in_boxes[box_i].debug_str());
                if box_i > 0 && !can_coalesce_text_nodes(*in_boxes, box_i-1, box_i) {
                    self.flush_clump_to_list(ctx, flow, *in_boxes, &mut out_boxes);
                }
                self.clump.extend_by(1);
            }
            // handle remaining clumps
            if self.clump.length() > 0 {
                self.flush_clump_to_list(ctx, flow, *in_boxes, &mut out_boxes);
            }

            debug!("TextRunScanner: swapping out boxes.");
            // swap out old and new box list of flow, by supplying
            // temp boxes as return value to boxes.swap |...|
        util::swap(in_boxes, &mut out_boxes);
        //}

        // helper functions
        fn can_coalesce_text_nodes(boxes: &[@mut RenderBox], left_i: uint, right_i: uint) -> bool {
            assert!(left_i < boxes.len());
            assert!(right_i > 0 && right_i < boxes.len());
            assert!(left_i != right_i);

            let (left, right) = (boxes[left_i], boxes[right_i]);
            match (left, right) {
                (@UnscannedTextBox(*), @UnscannedTextBox(*)) => left.can_merge_with_box(right),
                (_, _) => false
            }
        }
    }

    // a 'clump' is a range of inline flow leaves that can be merged
    // together into a single RenderBox. Adjacent text with the same
    // style can be merged, and nothing else can. 
    //
    // the flow keeps track of the RenderBoxes contained by all
    // non-leaf DOM nodes. This is necessary for correct painting
    // order. Since we compress several leaf RenderBoxes here, the
    // mapping must be adjusted.
    // 
    // N.B. in_boxes is passed by reference, since we cannot
    // recursively borrow or swap the flow's dvec of boxes. When all
    // boxes are appended, the caller swaps the flow's box list.
    fn flush_clump_to_list(&mut self,
                           ctx: &mut LayoutContext, 
                           flow: @mut FlowContext,
                           in_boxes: &[@mut RenderBox],
                           out_boxes: &mut ~[@mut RenderBox]) {
        assert!(self.clump.length() > 0);

        debug!("TextRunScanner: flushing boxes in range=%?", self.clump);
        let is_singleton = self.clump.length() == 1;
        let is_text_clump = match in_boxes[self.clump.begin()] {
            @UnscannedTextBox(*) => true,
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
                // TODO(Issue #115): use actual CSS 'white-space' property of relevant style.
                let compression = CompressWhitespaceNewline;
                let transformed_text = transform_text(text, compression);
                // TODO(Issue #177): text run creation must account for text-renderability by fontgroup fonts.
                // this is probably achieved by creating fontgroup above, and then letting FontGroup decide
                // which Font to stick into the TextRun.
                let fontgroup = ctx.font_ctx.get_resolved_font_for_style(&font_style);
                let run = @fontgroup.create_textrun(transformed_text);
                debug!("TextRunScanner: pushing single text box in range: %?", self.clump);
                let new_box = adapt_textbox_with_range(old_box.d(),
                                                       run,
                                                       &Range::new(0, run.char_len()));
                out_boxes.push(new_box);
            },
            (false, true) => {
                // TODO(Issue #115): use actual CSS 'white-space' property of relevant style.
                let compression = CompressWhitespaceNewline;

                // first, transform/compress text of all the nodes
                let transformed_strs : ~[~str] = vec::from_fn(self.clump.length(), |i| {
                    // TODO(Issue #113): we shoud be passing compression context
                    // between calls to transform_text, so that boxes
                    // starting/ending with whitespace &c can be
                    // compressed correctly w.r.t. the TextRun.
                    let idx = i + self.clump.begin();
                    transform_text(in_boxes[idx].raw_text(), compression)
                });

                // next, concatenate all of the transformed strings together, saving the new char indices
                let mut run_str : ~str = ~"";
                let mut new_ranges : ~[Range] = ~[];
                let mut char_total = 0u;
                for uint::range(0, transformed_strs.len()) |i| {
                    let added_chars = str::char_len(transformed_strs[i]);
                    new_ranges.push(Range::new(char_total, added_chars));
                    str::push_str(&mut run_str, transformed_strs[i]);
                    char_total += added_chars;
                }

                // create the run, then make new boxes with the run and adjusted text indices

                // TODO(Issue #177): text run creation must account for text-renderability by fontgroup fonts.
                // this is probably achieved by creating fontgroup above, and then letting FontGroup decide
                // which Font to stick into the TextRun.
                let font_style = in_boxes[self.clump.begin()].font_style();
                let fontgroup = ctx.font_ctx.get_resolved_font_for_style(&font_style);
                let run = @TextRun::new(fontgroup.fonts[0], run_str);
                debug!("TextRunScanner: pushing box(es) in range: %?", self.clump);
                let clump = self.clump;
                for clump.eachi |i| {
                    let range = &new_ranges[i - self.clump.begin()];
                    if range.length() == 0 { 
                        error!("Elided an UnscannedTextbox because it was zero-length after compression; %s",
                              in_boxes[i].debug_str());
                        loop
                    }
                    let new_box = adapt_textbox_with_range(in_boxes[i].d(), run, range);
                    out_boxes.push(new_box);
                }
            }
        } /* /match */
    
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
        let elems: &mut ElementMapping = &mut flow.inline().elems;
        for elems.eachi_mut |i: uint, nr: &NodeRange| {
            debug!("%u: %? --> %s", i, nr.range, nr.node.debug_str()); ()
        }
        debug!("--------------------");

        let end = self.clump.end(); // FIXME: borrow checker workaround
        self.clump.reset(end, 0);
    } /* /fn flush_clump_to_list */
}

struct PendingLine {
    range: Range,
    width: Au
}

struct LineboxScanner {
    flow: @mut FlowContext,
    new_boxes: ~[@mut RenderBox],
    work_list: @mut Deque<@mut RenderBox>,
    pending_line: PendingLine,
    line_spans: ~[Range],
}

fn LineboxScanner(inline: @mut FlowContext) -> LineboxScanner {
    assert!(inline.starts_inline_flow());

    LineboxScanner {
        flow: inline,
        new_boxes: ~[],
        work_list: @mut Deque::new(),
        pending_line: PendingLine {mut range: Range::empty(), mut width: Au(0)},
        line_spans: ~[]
    }
}

impl LineboxScanner {
    priv fn reset_scanner(&mut self) {
        debug!("Resetting line box scanner's state for flow f%d.", self.flow.d().id);
        self.line_spans = ~[];
        self.new_boxes = ~[];
        self.reset_linebox();
    }

    priv fn reset_linebox(&mut self) {
        self.pending_line.range.reset(0,0);
        self.pending_line.width = Au(0);
    }

    pub fn scan_for_lines(&mut self, ctx: &LayoutContext) {
        self.reset_scanner();
        
        let boxes = &mut self.flow.inline().boxes;
        let mut i = 0u;

        loop {
            // acquire the next box to lay out from work list or box list
            let cur_box = if self.work_list.is_empty() {
                if i == boxes.len() { break; }
                let box = boxes[i]; i += 1;
                debug!("LineboxScanner: Working with box from box list: b%d", box.d().id);
                box
            } else {
                let box = self.work_list.pop_front();
                debug!("LineboxScanner: Working with box from work list: b%d", box.d().id);
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

        let boxes = &mut self.flow.inline().boxes;
        let elems = &mut self.flow.inline().elems;
        elems.repair_for_box_changes(*boxes, self.new_boxes);
        self.swap_out_results();
    }

    priv fn swap_out_results(&mut self) {
        debug!("LineboxScanner: Propagating scanned lines[n=%u] to inline flow f%d", 
               self.line_spans.len(), self.flow.d().id);

        //do self.new_boxes.swap |boxes| {
            let inline_boxes = &mut self.flow.inline().boxes;
        util::swap(inline_boxes, &mut self.new_boxes);
            //inline_boxes = boxes;
        //    ~[]
        //};
        //do self.line_spans.swap |boxes| {
            let lines = &mut self.flow.inline().lines;
        util::swap(lines, &mut self.line_spans);
         //   lines = boxes;
        //    ~[]
        //};
    }

    priv fn flush_current_line(&mut self) {
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

        let slack_width = self.flow.d().position.size.width - self.pending_line.width;
        match linebox_align {
            // So sorry, but justified text is more complicated than shuffling linebox coordinates.
            // TODO(Issue #213): implement `text-align: justify`
            CSSTextAlignLeft | CSSTextAlignJustify => {
                for line_range.eachi |i| {
                    let box_data = &self.new_boxes[i].d();
                    box_data.position.origin.x = offset_x;
                    offset_x += box_data.position.size.width;
                }
            },
            CSSTextAlignCenter => {
                offset_x = slack_width.scale_by(0.5f);
                for line_range.eachi |i| {
                    let box_data = &self.new_boxes[i].d();
                    box_data.position.origin.x = offset_x;
                    offset_x += box_data.position.size.width;
                }
            },
            CSSTextAlignRight => {
                offset_x = slack_width;
                for line_range.eachi |i| {
                    let box_data = &self.new_boxes[i].d();
                    box_data.position.origin.x = offset_x;
                    offset_x += box_data.position.size.width;
                }
            },
        }
        // clear line and add line mapping
        debug!("LineboxScanner: Saving information for flushed line %u.", self.line_spans.len());
        self.line_spans.push(line_range);
        self.reset_linebox();
    }

    // return value: whether any box was appended.
    priv fn try_append_to_line(&mut self, ctx: &LayoutContext, in_box: @mut RenderBox) -> bool {
        let remaining_width = self.flow.d().position.size.width - self.pending_line.width;
        let in_box_width = in_box.d().position.size.width;
        let line_is_empty: bool = self.pending_line.range.length() == 0;

        debug!("LineboxScanner: Trying to append box to line %u (box width: %?, remaining width: %?): %s",
               self.line_spans.len(), in_box_width, remaining_width, in_box.debug_str());

        if in_box_width <= remaining_width {
            debug!("LineboxScanner: case=box fits without splitting");
            self.push_box_to_line(in_box);
            return true;
        }

        if !in_box.can_split() {
            // force it onto the line anyway, if its otherwise empty
            // TODO(Issue #224): signal that horizontal overflow happened?
            if line_is_empty {
                debug!("LineboxScanner: case=box can't split and line %u is empty, so overflowing.",
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
                error!("LineboxScanner: Tried to split unsplittable render box! %s", in_box.debug_str());
                return false;
            },
            SplitDidFit(left, right) => {
                debug!("LineboxScanner: case=split box did fit; deferring remainder box.");
                match (left, right) {
                    (Some(left_box), Some(right_box)) => {
                        self.push_box_to_line(left_box);
                        self.work_list.add_front(right_box);
                    },
                    (Some(left_box), None) =>  { self.push_box_to_line(left_box); }
                    (None, Some(right_box)) => { self.push_box_to_line(right_box); }
                    (None, None) => {
                        error!("LineboxScanner: This split case makes no sense!");
                    }
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
    priv fn push_box_to_line(&mut self, box: @mut RenderBox) {
        debug!("LineboxScanner: Pushing box b%d to line %u", box.d().id, self.line_spans.len());

        if self.pending_line.range.length() == 0 {
            assert!(self.new_boxes.len() <= (core::u16::max_value as uint));
            self.pending_line.range.reset(self.new_boxes.len(), 0);
        }
        self.pending_line.range.extend_by(1);
        self.pending_line.width += box.d().position.size.width;
        self.new_boxes.push(box);
    }
}

pub struct InlineFlowData {
    // A vec of all inline render boxes. Several boxes may
    // correspond to one Node/Element.
    boxes: ~[@mut RenderBox],
    // vec of ranges into boxes that represents line positions.
    // these ranges are disjoint, and are the result of inline layout.
    lines: ~[Range],
    // vec of ranges into boxes that represent elements. These ranges
    // must be well-nested, and are only related to the content of
    // boxes (not lines). Ranges are only kept for non-leaf elements.
    elems: ElementMapping
}

pub fn InlineFlowData() -> InlineFlowData {
    InlineFlowData {
        boxes: ~[],
        lines: ~[],
        elems: ElementMapping::new(),
    }
}

pub trait InlineLayout {
    fn starts_inline_flow(&self) -> bool;

    fn bubble_widths_inline(@mut self, ctx: &mut LayoutContext);
    fn assign_widths_inline(@mut self, ctx: &mut LayoutContext);
    fn assign_height_inline(@mut self, ctx: &mut LayoutContext);
    fn build_display_list_inline(@mut self, a: &DisplayListBuilder, b: &Rect<Au>, c: &Point2D<Au>,
                                 d: &Cell<DisplayList>);
}

impl InlineLayout for FlowContext {
    fn starts_inline_flow(&self) -> bool { match *self { InlineFlow(*) => true, _ => false } }

    fn bubble_widths_inline(@mut self, ctx: &mut LayoutContext) {
        assert!(self.starts_inline_flow());

        let mut scanner = TextRunScanner::new();
        scanner.scan_for_runs(ctx, self);

        let mut min_width = Au(0);
        let mut pref_width = Au(0);

        let boxes = &mut self.inline().boxes;
        for boxes.each |box| {
            debug!("FlowContext[%d]: measuring %s", self.d().id, box.debug_str());
            min_width = Au::max(min_width, box.get_min_width(ctx));
            pref_width = Au::max(pref_width, box.get_pref_width(ctx));
        }

        self.d().min_width = min_width;
        self.d().pref_width = pref_width;
    }

    /* Recursively (top-down) determines the actual width of child
    contexts and boxes. When called on this context, the context has
    had its width set by the parent context. */
    fn assign_widths_inline(@mut self, ctx: &mut LayoutContext) {
        assert!(self.starts_inline_flow());

        // initialize (content) box widths, if they haven't been
        // already. This could be combined with LineboxScanner's walk
        // over the box list, and/or put into RenderBox.
        let boxes = &mut self.inline().boxes;
        for boxes.each |&box| {
            let box2 = &mut *box;
            box.d().position.size.width = match *box2 {
                ImageBox(_, ref img) => {
                    let img2: &mut holder::ImageHolder = unsafe { cast::transmute(img) };
                    Au::from_px(img2.get_size().get_or_default(Size2D(0,0)).width)
                }
                TextBox(*) => { /* text boxes are initialized with dimensions */
                                   box.d().position.size.width
                },
                // TODO(Issue #225): different cases for 'inline-block', other replaced content
                GenericBox(*) => Au::from_px(45), 
                _ => fail!(fmt!("Tried to assign width to unknown Box variant: %?", box))
            };
        } // for boxes.each |box|

        let mut scanner = LineboxScanner(self);
        scanner.scan_for_lines(ctx);
   
        /* There are no child contexts, so stop here. */

        // TODO(Issue #225): once there are 'inline-block' elements, this won't be
        // true.  In that case, set the InlineBlockBox's width to the
        // shrink-to-fit width, perform inline flow, and set the block
        // flow context's width as the assigned width of the
        // 'inline-block' box that created this flow before recursing.
    }

    fn assign_height_inline(@mut self, _ctx: &mut LayoutContext) {
        // TODO(Issue #226): get CSS 'line-height' property from
        // containing block's style to determine minimum linebox height.
        // TODO(Issue #226): get CSS 'line-height' property from each non-replaced
        // inline element to determine its height for computing linebox height.
        let line_height = Au::from_px(20);
        let mut cur_y = Au(0);

        let lines = &mut self.inline().lines;
        for lines.eachi |i, line_span| {
            debug!("assign_height_inline: processing line %u with box span: %?", i, line_span);
            // coords relative to left baseline
            let mut linebox_bounding_box = Au::zero_rect();
            let boxes = &mut self.inline().boxes;
            for line_span.eachi |box_i| {
                let cur_box : &mut RenderBox = boxes[box_i]; // FIXME: borrow checker workaround

                // compute box height.
                let d = cur_box.d(); // FIXME: borrow checker workaround
                let cur_box : &mut RenderBox = boxes[box_i]; // FIXME: borrow checker workaround
                d.position.size.height = match *cur_box {
                    ImageBox(_, ref img) => {
                        Au::from_px(img.size().height)
                    }
                    TextBox(*) => { /* text boxes are initialized with dimensions */
                        d.position.size.height
                    },
                    // TODO(Issue #225): different cases for 'inline-block', other replaced content
                    GenericBox(*) => Au::from_px(30),
                    _ => {
                        let cur_box = boxes[box_i]; // FIXME: borrow checker workaround
                        fail!(fmt!("Tried to assign height to unknown Box variant: %s",
                                   cur_box.debug_str()))
                    }
                };

                // compute bounding rect, with left baseline as origin.
                // so, linebox height is a matter of lining up ideal baselines,
                // and then using the union of all these rects.
                let bounding_box = match *cur_box {
                    // adjust to baseline coords
                    // TODO(Issue #227): use left/right margins, border, padding for nonreplaced content,
                    // and also use top/bottom  margins, border, padding for replaced or inline-block content.
                    // TODO(Issue #225): use height, width for 'inline-block', other replaced content
                    ImageBox(*) | GenericBox(*) => {
                        let box_bounds = d.position;
                        box_bounds.translate(&Point2D(Au(0), -d.position.size.height))
                    },
                    // adjust bounding box metric to box's horizontal offset
                    // TODO: we can use font metrics directly instead of re-measuring for the bounding box.
                    TextBox(_, data) => {
                        let text_bounds = data.run.metrics_for_range(&data.range).bounding_box;
                        text_bounds.translate(&Point2D(d.position.origin.x, Au(0)))
                    },
                    _ => {
                        let cur_box = boxes[box_i]; // FIXME: borrow checker workaround
                        fail!(fmt!("Tried to compute bounding box of unknown Box variant: %s",
                                   cur_box.debug_str()))
                    }
                };
                debug!("assign_height_inline: bounding box for box b%d = %?", cur_box.d().id, bounding_box);
                linebox_bounding_box = linebox_bounding_box.union(&bounding_box);
                debug!("assign_height_inline: linebox bounding box = %?", linebox_bounding_box);
            }
            let linebox_height = linebox_bounding_box.size.height;
            let baseline_offset = -linebox_bounding_box.origin.y;
            // now go back and adjust y coordinates to match determined baseline
            for line_span.eachi |box_i| {
                let cur_box = boxes[box_i];
                // TODO(Issue #226): this is completely wrong. Need to use element's 
                // 'line-height' when calculating linebox height. Then, go back over 
                // and set y offsets according to 'vertical-align' property of containing block.
                let halfleading = match cur_box {
                    @TextBox(_, data) => { (data.run.font.metrics.em_size - line_height).scale_by(0.5f) },
                    _ => { Au(0) }
                };
                cur_box.d().position.origin.y = cur_y + halfleading + (baseline_offset - cur_box.d().position.size.height);
            }
            
            cur_y += Au::max(line_height, linebox_height);
        } // /lines.each |line_span|

        self.d().position.size.height = cur_y;
    }

    fn build_display_list_inline(@mut self, builder: &DisplayListBuilder, dirty: &Rect<Au>, 
                                 offset: &Point2D<Au>, list: &Cell<DisplayList>) {

        assert!(self.starts_inline_flow());

        // TODO(Issue #228): once we form line boxes and have their cached bounds, we can be 
        // smarter and not recurse on a line if nothing in it can intersect dirty
        let inline = self.inline(); // FIXME: borrow checker workaround
        debug!("FlowContext[%d]: building display list for %u inline boxes",
               self.d().id, inline.boxes.len());
        let boxes = &mut self.inline().boxes;
        for boxes.each |box| {
            box.build_display_list(builder, dirty, offset, list)
        }

        // TODO(Issue #225): should inline-block elements have flows as children
        // of the inline flow, or should the flow be nested inside the box somehow?
    }

} // @FlowContext : InlineLayout
