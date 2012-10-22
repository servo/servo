use au = gfx::geometry;
use core::dlist::DList;
use core::dvec::DVec;
use css::values::{BoxAuto, BoxLength, Px};
use dl = gfx::display_list;
use dom::node::Node;
use geom::point::Point2D;
use geom::rect::Rect;
use geom::size::Size2D;
use gfx::geometry::Au;
use layout::box::*;
use layout::context::LayoutContext;
use layout::flow::{FlowContext, InlineFlow};
use layout::text::TextBoxData;
use num::Num;
use servo_text::text_run::TextRun;
use servo_text::util::*;
use std::arc;
use util::range::{MutableRange, Range};
use util::tree;

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

struct NodeRange {
    node: Node,
    mut range: Range,
}

impl NodeRange {
    static pure fn new(node: Node, range: Range) -> NodeRange {
        NodeRange { node: node, range: range }
    }
}

// stack-allocated object for scanning an inline flow into
// TextRun-containing TextBoxes.
struct TextRunScanner {
    clump: MutableRange,
    flow: @FlowContext,
}

fn TextRunScanner(flow: @FlowContext) -> TextRunScanner {
    TextRunScanner {
        clump: util::range::empty_mut(),
        flow: flow,
    }
}

impl TextRunScanner {
    fn scan_for_runs(ctx: &LayoutContext) {
        assert self.flow.inline().boxes.len() > 0;

        do self.flow.inline().boxes.swap |in_boxes| {
            debug!("TextRunScanner: scanning %u boxes for text runs...", in_boxes.len());
            let out_boxes = DVec();

            for uint::range(0, in_boxes.len()) |box_i| {
                debug!("TextRunScanner: considering box: %?", in_boxes[box_i].debug_str());
                if box_i > 0 && !can_coalesce_text_nodes(in_boxes, box_i-1, box_i) {
                    self.flush_clump_to_list(ctx, in_boxes, &out_boxes);
                }
                self.clump.extend_by(1);
            }
            // handle remaining clumps
            if self.clump.length() > 0 {
                self.flush_clump_to_list(ctx, in_boxes, &out_boxes);
            }

            debug!("TextRunScanner: swapping out boxes.");
            // swap out old and new box list of flow, by supplying
            // temp boxes as return value to boxes.swap |...|
            dvec::unwrap(move out_boxes)
        }

        // helper functions
        pure fn can_coalesce_text_nodes(boxes: &[@RenderBox], left_i: uint, right_i: uint) -> bool {
            assert left_i >= 0 && left_i < boxes.len();
            assert right_i > 0 && right_i < boxes.len();
            assert left_i != right_i;

            let (left, right) = (boxes[left_i], boxes[right_i]);
            match (left, right) {
                // TODO(Issue #117): check whether text styles, fonts are the same.
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
    fn flush_clump_to_list(ctx: &LayoutContext, 
                           in_boxes: &[@RenderBox], out_boxes: &DVec<@RenderBox>) {
        assert self.clump.length() > 0;

        debug!("TextRunScanner: flushing boxes in range=%?", self.clump);
        let is_singleton = self.clump.length() == 1;
        let is_text_clump = match in_boxes[self.clump.begin()] {
            @UnscannedTextBox(*) => true,
            _ => false
        };

        match (is_singleton, is_text_clump) {
            (false, false) => fail ~"WAT: can't coalesce non-text nodes in flush_clump_to_list()!",
            (true, false) => { 
                debug!("TextRunScanner: pushing single non-text box in range: %?", self.clump);
                out_boxes.push(in_boxes[self.clump.begin()]);
            },
            (true, true)  => { 
                let text = in_boxes[self.clump.begin()].raw_text();
                // TODO(Issue #115): use actual CSS 'white-space' property of relevant style.
                let compression = CompressWhitespaceNewline;
                let transformed_text = transform_text(text, compression);
                // TODO(Issue #116): use actual font for corresponding DOM node to create text run.
                let run = @TextRun(ctx.font_cache.get_test_font(), move transformed_text);
                debug!("TextRunScanner: pushing single text box in range: %?", self.clump);
                let new_box = layout::text::adapt_textbox_with_range(in_boxes[self.clump.begin()].d(), run,
                                                                     Range(0, run.text.len()));
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

                // next, concatenate all of the transformed strings together, saving the new text indices

                // TODO(Issue #118): use a rope, simply give ownership of  nonzero strs to rope
                let mut run_str : ~str = ~"";
                let new_ranges : DVec<Range> = DVec();
                for uint::range(0, transformed_strs.len()) |i| {
                    // XXX: if transformed_strs[i].len() == 0 { loop }
                    new_ranges.push(Range(run_str.len(), transformed_strs[i].len()));
                    str::push_str(&mut run_str, transformed_strs[i]);
                }

                // TODO: adjust containing ranges to account for any elided boxes (see XXX above, below)
                
                // create the run, then make new boxes with the run and adjusted text indices

                // TODO(Issue #116): use actual font for corresponding DOM node to create text run.
                let run = @TextRun(ctx.font_cache.get_test_font(), move run_str);
                debug!("TextRunScanner: pushing box(es) in range: %?", self.clump);
                for self.clump.eachi |i| {
                    let range = new_ranges[i - self.clump.begin()];
                    if range.length() == 0 { loop } // XXX
                    let new_box = layout::text::adapt_textbox_with_range(in_boxes[i].d(), run, range);
                    out_boxes.push(new_box);
                }
            }
        } /* /match */
    
        self.clump.reset(self.clump.end(), 0);
    } /* /fn flush_clump_to_list */
}

struct LineboxScanner {
    flow: @FlowContext,
    new_boxes: DVec<@RenderBox>,
    work_list: DList<@RenderBox>,
    pending_line: {range: MutableRange, mut width: Au},
    line_spans: DVec<Range>,

    drop {
        debug!("---DROPPING LINEBOXSCANNER---");
    }
}

fn LineboxScanner(inline: @FlowContext) -> LineboxScanner {
    assert inline.starts_inline_flow();

    LineboxScanner {
        flow: inline,
        new_boxes: DVec(),
        work_list: DList(),
        pending_line: {range: util::range::empty_mut(), mut width: Au(0)},
        line_spans: DVec()
    }
}

impl LineboxScanner {
    priv fn reset_scanner() {
        debug!("Resetting line box scanner's state for flow f%d.", self.flow.d().id);
        self.line_spans.set(~[]);
        self.new_boxes.set(~[]);
        self.reset_linebox();
    }

    priv fn reset_linebox() {
        self.pending_line.range.reset(0,0);
        self.pending_line.width = Au(0);
    }

    pub fn scan_for_lines(ctx: &LayoutContext) {
        self.reset_scanner();
        
        let boxes = &self.flow.inline().boxes;
        let mut i = 0u;

        loop {
            // acquire the next box to lay out from work list or box list
            let cur_box = match self.work_list.pop() {
                Some(box) => { 
                    debug!("LineboxScanner: Working with box from work list: b%d", box.d().id);
                    box
                },
                None => { 
                    if i == boxes.len() { break; }
                    let box = boxes[i]; i += 1;
                    debug!("LineboxScanner: Working with box from box list: b%d", box.d().id);
                    box
                }
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

        //self.repair_elem_ranges();
        self.swap_out_results();
    }

    priv fn repair_elem_ranges() {
        debug!("--- Elem ranges before repair: ---");
        for self.flow.inline().elems.eachi |i: uint, nr: &NodeRange| {
            debug!("%u: %?", i, nr.range);
        }
        debug!("----------------------------------");

        let mut old_i = 0;
        let old_boxes = &self.flow.inline().boxes;

        // index into self.new_boxes
        let mut new_j = 0;

        struct WorkItem {
            begin_idx: uint,
            elem_idx: uint,
        };
        let repair_stack : DVec<WorkItem> = DVec();

        do self.flow.inline().elems.borrow |elems: &[NodeRange]| {
            // index into elems
            let mut elems_k = 0;
            
            while old_i < old_boxes.len() {
                debug!("Considering old box %u", old_i);
                // possibly push several items
                while elems_k < elems.len() && old_i == elems[elems_k].range.begin() {
                    let item = WorkItem {begin_idx: new_j, elem_idx: elems_k};
                    debug!("Push work item for elem %u: %?", elems_k, item);
                    repair_stack.push(item);
                    elems_k += 1;
                }
            
                // slide forward through any dups
                while new_j < self.new_boxes.len() && old_boxes[old_i].d().node == self.new_boxes[new_j].d().node {
                    debug!("Slide through new box %u", new_j);
                    new_j += 1;
                }

                // possibly pop several items
                while repair_stack.len() > 0 && old_i == elems[repair_stack.last().elem_idx].range.end() {
                    let item = repair_stack.pop();
                    debug!("Set range for %u to %?", item.elem_idx, Range(item.begin_idx, new_j));
                    elems[item.elem_idx].range = Range(item.begin_idx, new_j);
                }
                old_i += 1;
            }
            
            // possibly pop several items
            while repair_stack.len() > 0 && old_i == elems[repair_stack.last().elem_idx].range.end() {
                let item = repair_stack.pop();
                debug!("Set range for %u to %?", item.elem_idx, Range(item.begin_idx, new_j));
                elems[item.elem_idx].range = Range(item.begin_idx, new_j);
            }
        }
    }

    priv fn swap_out_results() {
        debug!("LineboxScanner: Propagating scanned lines[n=%u] to inline flow f%d", 
               self.line_spans.len(), self.flow.d().id);

        do self.new_boxes.swap |boxes| {
            self.flow.inline().boxes.set(move boxes);
            ~[]
        };
        do self.line_spans.swap |boxes| {
            self.flow.inline().lines.set(move boxes);
            ~[]
        };

        debug!("--- Elem ranges after repair: ---");
//        for self.flow.inline().elems.eachi |i: uint, nr: &NodeRange| {
//            debug!("%u: %?", i, nr.range);
//        }
        debug!("----------------------------------");
    }

    priv fn flush_current_line() {
        debug!("LineboxScanner: Flushing line %u: %?",
               self.line_spans.len(), self.pending_line);
        // set box horizontal offsets
        let line_range = self.pending_line.range.as_immutable();
        let mut offset_x = Au(0);
        // TODO: interpretation of CSS 'text-direction' and 'text-align' 
        // will change from which side we start laying out the line.
        debug!("LineboxScanner: Setting horizontal offsets for boxes in line %u range: %?",
               self.line_spans.len(), line_range);
        for line_range.eachi |i| {
            let box_data = &self.new_boxes[i].d();
            box_data.position.origin.x = offset_x;
            offset_x += box_data.position.size.width;
        }

        // clear line and add line mapping
        debug!("LineboxScanner: Saving information for flushed line %u.", self.line_spans.len());
        self.line_spans.push(move line_range);
        self.reset_linebox();
    }

    // return value: whether any box was appended.
    priv fn try_append_to_line(ctx: &LayoutContext, in_box: @RenderBox) -> bool {
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
            // TODO: signal that horizontal overflow happened?
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
                        self.work_list.push_head(right_box);
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
                    // TODO: signal that horizontal overflow happened?
                    match (left, right) {
                        (Some(left_box), Some(right_box)) => {
                            self.push_box_to_line(left_box);
                            self.work_list.push_head(right_box);
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
                    self.work_list.push_head(in_box);
                    return false;
                }
            }
        }
    }

    // unconditional push
    priv fn push_box_to_line(box: @RenderBox) {
        debug!("LineboxScanner: Pushing box b%d to line %u", box.d().id, self.line_spans.len());

        if self.pending_line.range.length() == 0 {
            assert self.new_boxes.len() <= (core::u16::max_value as uint);
            self.pending_line.range.reset(self.new_boxes.len(), 0);
        }
        self.pending_line.range.extend_by(1);
        self.pending_line.width += box.d().position.size.width;
        self.new_boxes.push(box);
    }
}

struct InlineFlowData {
    // A vec of all inline render boxes. Several boxes may
    // correspond to one Node/Element.
    boxes: DVec<@RenderBox>,
    // vec of ranges into boxes that represents line positions.
    // these ranges are disjoint, and are the result of inline layout.
    lines: DVec<Range>,
    // vec of ranges into boxes that represent elements. These ranges
    // must be well-nested, and are only related to the content of
    // boxes (not lines). Ranges are only kept for non-leaf elements.
    elems: DVec<NodeRange>
}

fn InlineFlowData() -> InlineFlowData {
    InlineFlowData {
        boxes: DVec(),
        lines: DVec(),
        elems: DVec(),
    }
}

trait InlineLayout {
    pure fn starts_inline_flow() -> bool;

    fn bubble_widths_inline(@self, ctx: &LayoutContext);
    fn assign_widths_inline(@self, ctx: &LayoutContext);
    fn assign_height_inline(@self, ctx: &LayoutContext);
    fn build_display_list_inline(@self, a: &dl::DisplayListBuilder, b: &Rect<Au>, c: &Point2D<Au>, d: &dl::DisplayList);
}

impl FlowContext : InlineLayout {
    pure fn starts_inline_flow() -> bool { match self { InlineFlow(*) => true, _ => false } }

    fn bubble_widths_inline(@self, ctx: &LayoutContext) {
        assert self.starts_inline_flow();

        let scanner = TextRunScanner(self);
        scanner.scan_for_runs(ctx);

        let mut min_width = Au(0);
        let mut pref_width = Au(0);

        for self.inline().boxes.each |box| {
            debug!("FlowContext[%d]: measuring %s", self.d().id, box.debug_str());
            min_width = au::max(min_width, box.get_min_width(ctx));
            pref_width = au::max(pref_width, box.get_pref_width(ctx));
        }

        self.d().min_width = min_width;
        self.d().pref_width = pref_width;
    }

    /* Recursively (top-down) determines the actual width of child
    contexts and boxes. When called on this context, the context has
    had its width set by the parent context. */
    fn assign_widths_inline(@self, ctx: &LayoutContext) {
        assert self.starts_inline_flow();

        // initialize (content) box widths, if they haven't been
        // already. This could be combined with LineboxScanner's walk
        // over the box list, and/or put into RenderBox.
        for self.inline().boxes.each |box| {
            box.d().position.size.width = match *box {
                @ImageBox(_,img) => au::from_px(img.get_size().get_default(Size2D(0,0)).width),
                @TextBox(*) => { /* text boxes are initialized with dimensions */
                                   box.d().position.size.width
                },
                @GenericBox(*) => au::from_px(45), /* TODO: should use CSS 'width'? */
                _ => fail fmt!("Tried to assign width to unknown Box variant: %?", box)
            };
        } // for boxes.each |box|

        let scanner = LineboxScanner(self);
        scanner.scan_for_lines(ctx);
   
        /* There are no child contexts, so stop here. */

        // TODO: once there are 'inline-block' elements, this won't be
        // true.  In that case, set the InlineBlockBox's width to the
        // shrink-to-fit width, perform inline flow, and set the block
        // flow context's width as the assigned width of the
        // 'inline-block' box that created this flow before recursing.
    }

    fn assign_height_inline(@self, _ctx: &LayoutContext) {
        // TODO: get from CSS 'line-height' property
        let line_height = au::from_px(20);
        let mut cur_y = Au(0);

        for self.inline().lines.eachi |i, line_span| {
            debug!("assign_height_inline: processing line %u with box span: %?", i, line_span);
            // coords relative to left baseline
            let mut linebox_bounding_box = au::zero_rect();
            let boxes = &self.inline().boxes;
            for line_span.eachi |box_i| {
                let cur_box = boxes[box_i];

                // compute box height.
                cur_box.d().position.size.height = match cur_box {
                    @ImageBox(_,img) => au::from_px(img.size().height),
                    @TextBox(*) => { /* text boxes are initialized with dimensions */
                        cur_box.d().position.size.height
                    },
                    @GenericBox(*) => au::from_px(30), /* TODO: should use CSS 'height'? */
                    _ => fail fmt!("Tried to assign height to unknown Box variant: %s", cur_box.debug_str())
                };

                // compute bounding rect, with left baseline as origin.
                // so, linebox height is a matter of lining up ideal baselines,
                // and then using the union of all these rects.
                let bounding_box = match cur_box {
                    // adjust to baseline coords
                    // TODO: account for padding, margin, border in bounding box?
                    @ImageBox(*) | @GenericBox(*) => {
                        let box_bounds = cur_box.d().position;
                        box_bounds.translate(&Point2D(Au(0), -cur_box.d().position.size.height))
                    },
                    // adjust bounding box metric to box's horizontal offset
                    // TODO: can we trust the leading provided by font metrics?
                    @TextBox(_, data) => { 
                        let text_bounds = data.run.metrics_for_range(data.range).bounding_box;
                        text_bounds.translate(&Point2D(cur_box.d().position.origin.x, Au(0)))
                    },
                    _ => fail fmt!("Tried to compute bounding box of unknown Box variant: %s", cur_box.debug_str())
                };
                cur_box.d().position.origin.y = cur_y;
                debug!("assign_height_inline: bounding box for box b%d = %?", cur_box.d().id, bounding_box);
                linebox_bounding_box = linebox_bounding_box.union(&bounding_box);
                debug!("assign_height_inline: linebox bounding box = %?", linebox_bounding_box);
            }
            let linebox_height = linebox_bounding_box.size.height;
            cur_y += au::max(line_height, linebox_height);
        } // /lines.each |line_span|

        self.d().position.size.height = cur_y;
    }

    fn build_display_list_inline(@self, builder: &dl::DisplayListBuilder, dirty: &Rect<Au>, 
                                 offset: &Point2D<Au>, list: &dl::DisplayList) {

        assert self.starts_inline_flow();

        // TODO: if the CSS box introducing this inline context is *not* anonymous,
        // we need to draw it too, in a way similar to BlockFlowContext

        // TODO: once we form line boxes and have their cached bounds, we can be 
        // smarter and not recurse on a line if nothing in it can intersect dirty
        debug!("FlowContext[%d]: building display list for %u inline boxes",
               self.d().id, self.inline().boxes.len());
        for self.inline().boxes.each |box| {
            box.build_display_list(builder, dirty, offset, list)
        }

        // TODO: should inline-block elements have flows as children
        // of the inline flow, or should the flow be nested inside the
        // box somehow? Maybe it's best to unify flows and boxes into
        // the same enum, so inline-block flows are normal
        // (indivisible) children in the inline flow child list.
    }

} // @FlowContext : InlineLayout
