use au = gfx::geometry;
use core::dlist::DList;
use core::dvec::DVec;
use css::values::{BoxAuto, BoxLength, Px};
use dl = gfx::display_list;
use dom::node::Node;
use geom::point::Point2D;
use geom::rect::Rect;
use geom::size::Size2D;
use gfx::geometry::au;
use layout::box::{RenderBox, ImageBox, TextBox, GenericBox, UnscannedTextBox};
use layout::context::LayoutContext;
use layout::flow::{FlowContext, InlineFlow};
use layout::text::TextBoxData;
use num::Num;
use servo_text::text_run::TextRun;
use servo_text::util::*;
use std::arc;
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

type BoxRange = {mut start: u16, mut len: u16};
type NodeRange = {node: Node, span: BoxRange};

// stack-allocated object for scanning an inline flow into
// TextRun-containing TextBoxes.
struct TextRunScanner {
    mut in_clump: bool,
    mut clump_start: uint,
    mut clump_end: uint,
    flow: @FlowContext,
}

fn TextRunScanner(flow: @FlowContext) -> TextRunScanner {
    TextRunScanner {
        in_clump: false,
        clump_start: 0,
        clump_end: 0,
        flow: flow,
    }
}

impl TextRunScanner {
    fn scan_for_runs(ctx: &LayoutContext) {
        // if reused, must be reset.
        assert !self.in_clump;
        assert self.flow.inline().boxes.len() > 0;

        do self.flow.inline().boxes.swap |in_boxes| {
            debug!("TextRunScanner: scanning %u boxes for text runs...", in_boxes.len());
            
            let out_boxes = DVec();
            let mut prev_box: @RenderBox = in_boxes[0];

            for uint::range(0, in_boxes.len()) |i| {
                debug!("TextRunScanner: considering box: %?", in_boxes[i].debug_str());

                let can_coalesce_with_prev = i > 0 && boxes_can_be_coalesced(prev_box, in_boxes[i]);

                match (self.in_clump, can_coalesce_with_prev) {
                    // start a new clump
                    (false, _)    => { self.reset_clump_to_index(i); },
                    // extend clump
                    (true, true)  => { self.clump_end = i; },
                    // boundary detected; flush and start new clump
                    (true, false) => {
                        self.flush_clump_to_list(ctx, in_boxes, &out_boxes);
                        self.reset_clump_to_index(i);
                    }
                };
                
                prev_box = in_boxes[i];
            }
            // handle remaining clumps
            if self.in_clump {
                self.flush_clump_to_list(ctx, in_boxes, &out_boxes);
            }

            debug!("TextRunScanner: swapping out boxes.");
            // swap out old and new box list of flow, by supplying
            // temp boxes as return value to boxes.swap |...|
            dvec::unwrap(out_boxes)
        }

        // helper functions
        pure fn boxes_can_be_coalesced(a: @RenderBox, b: @RenderBox) -> bool {
            assert !core::box::ptr_eq(a, b);

            match (a, b) {
                // TODO(Issue #117): check whether text styles, fonts are the same.
                (@UnscannedTextBox(*), @UnscannedTextBox(*)) => a.can_merge_with_box(b),
                (_, _) => false
            }
        }
    }

    fn reset_clump_to_index(i: uint) {
        debug!("TextRunScanner: resetting clump to %u", i);
        
        self.clump_start = i;
        self.clump_end = i;
        self.in_clump = true;
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
        assert self.in_clump;

        debug!("TextRunScanner: flushing boxes when start=%u,end=%u",
               self.clump_start, self.clump_end);

        let is_singleton = (self.clump_start == self.clump_end);
        let is_text_clump = match in_boxes[self.clump_start] {
            @UnscannedTextBox(*) => true,
            _ => false
        };

        match (is_singleton, is_text_clump) {
            (false, false) => fail ~"WAT: can't coalesce non-text boxes in flush_clump_to_list()!",
            (true, false) => { 
                debug!("TextRunScanner: pushing single non-text box when start=%u,end=%u", 
                       self.clump_start, self.clump_end);
                out_boxes.push(in_boxes[self.clump_start]);
            },
            (true, true)  => { 
                let text = in_boxes[self.clump_start].raw_text();
                // TODO(Issue #115): use actual CSS 'white-space' property of relevant style.
                let compression = CompressWhitespaceNewline;
                let transformed_text = transform_text(text, compression);
                // TODO(Issue #116): use actual font for corresponding DOM node to create text run.
                let run = @TextRun(ctx.font_cache.get_test_font(), move transformed_text);
                let box_guts = TextBoxData(run, 0, run.text.len());
                debug!("TextRunScanner: pushing single text box when start=%u,end=%u",
                       self.clump_start, self.clump_end);
                out_boxes.push(@TextBox(copy *in_boxes[self.clump_start].d(), box_guts));
            },
            (false, true) => {
                // TODO(Issue #115): use actual CSS 'white-space' property of relevant style.
                let compression = CompressWhitespaceNewline;
                let clump_box_count = self.clump_end - self.clump_start + 1;

                // first, transform/compress text of all the nodes
                let transformed_strs : ~[~str] = vec::from_fn(clump_box_count, |i| {
                    // TODO(Issue #113): we shoud be passing compression context
                    // between calls to transform_text, so that boxes
                    // starting/ending with whitespace &c can be
                    // compressed correctly w.r.t. the TextRun.
                    let idx = i + self.clump_start;
                    transform_text(in_boxes[idx].raw_text(), compression)
                });

                // then, fix NodeRange mappings to account for elided boxes.
                do self.flow.inline().elems.borrow |ranges: &[NodeRange]| {
                    for ranges.each |range: &NodeRange| {
                        let span = &range.span;
                        let relation = relation_of_clump_and_range(span, self.clump_start, self.clump_end);
                        debug!("TextRunScanner: possibly repairing element range %?", range.span);
                        debug!("TextRunScanner: relation of range and clump(start=%u, end=%u): %?",
                               self.clump_start, self.clump_end, relation);
                        match relation {
                            RangeEntirelyBeforeClump => {},
                            RangeEntirelyAfterClump => { span.start -= clump_box_count as u16; },
                            RangeCoincidesClump | RangeContainedByClump => 
                                                       { span.start  = self.clump_start as u16;
                                                         span.len    = 1 },
                            RangeContainsClump =>      { span.len   -= clump_box_count as u16; },
                            RangeOverlapsClumpStart(overlap) => 
                                                       { span.len   -= (overlap - 1) as u16; },
                            RangeOverlapsClumpEnd(overlap) => 
                                                       { span.start  = self.clump_start as u16;
                                                         span.len   -= (overlap - 1) as u16; }
                        }
                        debug!("TextRunScanner: new element range: ---- %?", range.span);
                    }
                }

                // TODO(Issue #118): use a rope, simply give ownership of  nonzero strs to rope
                let mut run_str : ~str = ~"";
                for uint::range(0, transformed_strs.len()) |i| {
                    str::push_str(&mut run_str, transformed_strs[i]);
                }
                
                // TODO(Issue #116): use actual font for corresponding DOM node to create text run.
                let run = @TextRun(ctx.font_cache.get_test_font(), move run_str);
                let box_guts = TextBoxData(run, 0, run.text.len());
                debug!("TextRunScanner: pushing box(es) when start=%u,end=%u",
                       self.clump_start, self.clump_end);
                out_boxes.push(@TextBox(copy *in_boxes[self.clump_start].d(), box_guts));
            }
        } /* /match */
        self.in_clump = false;
    
        enum ClumpRangeRelation {
            RangeOverlapsClumpStart(/* overlap */ uint),
            RangeOverlapsClumpEnd(/* overlap */ uint),
            RangeContainedByClump,
            RangeContainsClump,
            RangeCoincidesClump,
            RangeEntirelyBeforeClump,
            RangeEntirelyAfterClump
        }
        
        fn relation_of_clump_and_range(range: &BoxRange, clump_start: uint, 
                                       clump_end: uint) -> ClumpRangeRelation {
            let range_start = range.start as uint;
            let range_end = (range.start + range.len) as uint;

            if range_end < clump_start {
                return RangeEntirelyBeforeClump;
            } 
            if range_start > clump_end {
                return RangeEntirelyAfterClump;
            }
            if range_start == clump_start && range_end == clump_end {
                return RangeCoincidesClump;
            }
            if range_start <= clump_start && range_end >= clump_end {
                return RangeContainsClump;
            }
            if range_start >= clump_start && range_end <= clump_end {
                return RangeContainedByClump;
            }
            if range_start < clump_start && range_end < clump_end {
                let overlap = range_end - clump_start;
                return RangeOverlapsClumpStart(overlap);
            }
            if range_start > clump_start && range_end > clump_end {
                let overlap = clump_end - range_start;
                return RangeOverlapsClumpEnd(overlap);
            }
            fail fmt!("relation_of_clump_and_range(): didn't classify range=%?, clump_start=%u, clump_end=%u",
                      range, clump_start, clump_end);
        }
    } /* /fn flush_clump_to_list */
}

struct InlineFlowData {
    // A vec of all inline render boxes. Several boxes may
    // correspond to one Node/Element.
    boxes: DVec<@RenderBox>,
    // vec of ranges into boxes that represents line positions.
    // these ranges are disjoint, and are the result of inline layout.
    lines: DVec<BoxRange>,
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
    fn build_display_list_inline(@self, a: &dl::DisplayListBuilder, b: &Rect<au>, c: &Point2D<au>, d: &dl::DisplayList);
}

impl FlowContext : InlineLayout {
    pure fn starts_inline_flow() -> bool { match self { InlineFlow(*) => true, _ => false } }

    fn bubble_widths_inline(@self, ctx: &LayoutContext) {
        assert self.starts_inline_flow();

        let scanner = TextRunScanner(self);
        scanner.scan_for_runs(ctx);

        let mut min_width = au(0);
        let mut pref_width = au(0);

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

        /* Perform inline flow with the available width. */
        //let avail_width = self.d().position.size.width;

        let line_height = au::from_px(20);
        //let mut cur_x = au(0);
        let mut cur_y = au(0);
        
        // TODO(Issue #118): remove test font uses
        let test_font = ctx.font_cache.get_test_font();
        
        for self.inline().boxes.each |box| {
            /* TODO: actually do inline flow.
            - Create a working linebox, and successively put boxes
            into it, splitting if necessary.
            
            - Set width and height for each positioned element based on 
            where its chunks ended up.

            - Save the dvec of this context's lineboxes. */

            box.d().position.size.width = match *box {
                @ImageBox(_,img) => au::from_px(img.get_size().get_default(Size2D(0,0)).width),
                @TextBox(_,d) => { 
                    // TODO: measure twice, cut once doesn't apply to text. Shouldn't need
                    // to measure text again here (should be inside TextBox.split)
                    let metrics = test_font.measure_text(d.run, d.offset, d.length);
                    metrics.advance_width
                },
                // TODO: this should be set to the extents of its children
                @GenericBox(*) => au(0),
                _ => fail fmt!("Tried to assign width to unknown Box variant: %?", box)
            };

            box.d().position.size.height = match *box {
                @ImageBox(_,img) => au::from_px(img.get_size().get_default(Size2D(0,0)).height),
                // TODO: we should use the bounding box of the actual text, i think?
                @TextBox(*) => test_font.metrics.em_size,
                // TODO: this should be set to the extents of its children
                @GenericBox(*) => au(0),
                _ => fail fmt!("Tried to assign width to unknown Box variant: %?", box)
            };

            box.d().position.origin = Point2D(au(0), cur_y);
            cur_y = cur_y.add(&au::max(line_height, box.d().position.size.height));
        } // for boxes.each |box|

    self.d().position.size.height = cur_y;
    
    /* There are no child contexts, so stop here. */

    // TODO: once there are 'inline-block' elements, this won't be
    // true.  In that case, perform inline flow, and then set the
    // block flow context's width as the width of the
    // 'inline-block' box that created this flow.
    }

    fn assign_height_inline(@self, _ctx: &LayoutContext) {
        // Don't need to set box or ctx heights, since that is done
        // during inline flowing.
    }

    fn build_display_list_inline(@self, builder: &dl::DisplayListBuilder, dirty: &Rect<au>, 
                                 offset: &Point2D<au>, list: &dl::DisplayList) {

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
