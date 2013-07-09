/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

//! Text layout.

use std::uint;
use std::vec;

use gfx::text::text_run::TextRun;
use gfx::text::util::{CompressWhitespaceNewline, transform_text};
use layout::box::{RenderBox, RenderBoxBase, TextRenderBox};
use layout::box::{TextRenderBoxClass, UnscannedTextRenderBoxClass};
use layout::context::LayoutContext;
use layout::flow::FlowContext;
use layout::util::{NodeRange};
use newcss::values::{CSSTextDecoration, CSSTextDecorationUnderline};
use servo_util::range::Range;


/// Creates a TextRenderBox from a range and a text run.
pub fn adapt_textbox_with_range(mut base: RenderBoxBase, run: @TextRun, range: Range)
                                -> TextRenderBox {
    debug!("Creating textbox with span: (strlen=%u, off=%u, len=%u) of textrun (%s) (len=%u)",
           run.char_len(),
           range.begin(),
           range.length(),
           run.text,
           run.char_len());

    assert!(range.begin() < run.char_len());
    assert!(range.end() <= run.char_len());
    assert!(range.length() > 0);

    let metrics = run.metrics_for_range(&range);
    base.position.size = metrics.bounding_box.size;

    TextRenderBox {
        base: base,
        run: run,
        range: range,
    }
}

pub trait UnscannedMethods {
    /// Copies out the text from an unscanned text box. Fails if this is not an unscanned text box.
    fn raw_text(&self) -> ~str;
}

impl UnscannedMethods for RenderBox {
    fn raw_text(&self) -> ~str {
        match *self {
            UnscannedTextRenderBoxClass(text_box) => copy text_box.text,
            _ => fail!(~"unsupported operation: box.raw_text() on non-unscanned text box."),
        }
    }
}

/// A stack-allocated object for scanning an inline flow into `TextRun`-containing `TextBox`es.
struct TextRunScanner {
    clump: Range,
}

impl TextRunScanner {
    pub fn new() -> TextRunScanner {
        TextRunScanner {
            clump: Range::empty(),
        }
    }

    pub fn scan_for_runs(&mut self, ctx: &LayoutContext, flow: FlowContext) {
        let inline = flow.inline();
        assert!(inline.boxes.len() > 0);
        debug!("TextRunScanner: scanning %u boxes for text runs...", inline.boxes.len());

        let mut last_whitespace = true;
        let mut out_boxes = ~[];
        for uint::range(0, flow.inline().boxes.len()) |box_i| {
            debug!("TextRunScanner: considering box: %?", flow.inline().boxes[box_i].debug_str());
            if box_i > 0 && !can_coalesce_text_nodes(flow.inline().boxes, box_i-1, box_i) {
                last_whitespace = self.flush_clump_to_list(ctx, flow, last_whitespace, &mut out_boxes);
            }
            self.clump.extend_by(1);
        }
        // handle remaining clumps
        if self.clump.length() > 0 {
            self.flush_clump_to_list(ctx, flow, last_whitespace, &mut out_boxes);
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
    pub fn flush_clump_to_list(&mut self,
                               ctx: &LayoutContext,
                               flow: FlowContext,
                               last_whitespace: bool,
                               out_boxes: &mut ~[RenderBox]) -> bool {
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

        let mut new_whitespace = last_whitespace;

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

                let (transformed_text, whitespace) = transform_text(text, compression, last_whitespace);
                new_whitespace = whitespace;

                if transformed_text.len() > 0 {
                    // TODO(#177): Text run creation must account for the renderability of text by
                    // font group fonts. This is probably achieved by creating the font group above
                    // and then letting `FontGroup` decide which `Font` to stick into the text run.
                    let fontgroup = ctx.font_ctx.get_resolved_font_for_style(&font_style);
                    let run = @fontgroup.create_textrun(transformed_text, underline);

                    debug!("TextRunScanner: pushing single text box in range: %? (%?)", self.clump, text);
                    let new_box = do old_box.with_base |old_box_base| {
                        let range = Range::new(0, run.char_len());
                        @mut adapt_textbox_with_range(*old_box_base, run, range)
                    };

                    out_boxes.push(TextRenderBoxClass(new_box));
                }
            },
            (false, true) => {
                // TODO(#115): Use the actual CSS `white-space` property of the relevant style.
                let compression = CompressWhitespaceNewline;

                // First, transform/compress text of all the nodes.
                let mut last_whitespace_in_clump = new_whitespace;
                let transformed_strs: ~[~str] = do vec::from_fn(self.clump.length()) |i| {
                    // TODO(#113): We should be passing the compression context between calls to
                    // `transform_text`, so that boxes starting and/or ending with whitespace can
                    // be compressed correctly with respect to the text run.
                    let idx = i + self.clump.begin();
                    let (new_str, new_whitespace) = transform_text(in_boxes[idx].raw_text(),
                                                                   compression,
                                                                   last_whitespace_in_clump);
                    last_whitespace_in_clump = new_whitespace;
                    new_str
                };
                new_whitespace = last_whitespace_in_clump;

                // Next, concatenate all of the transformed strings together, saving the new
                // character indices.
                let mut run_str: ~str = ~"";
                let mut new_ranges: ~[Range] = ~[];
                let mut char_total = 0;
                for uint::range(0, transformed_strs.len()) |i| {
                    let added_chars = transformed_strs[i].char_len();
                    new_ranges.push(Range::new(char_total, added_chars));
                    run_str.push_str(transformed_strs[i]);
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
                let run = if clump.length() != 0 && run_str.len() > 0 {
                    Some(@TextRun::new(fontgroup.fonts[0], run_str, underline))
                } else {
                    None
                };

                // Make new boxes with the run and adjusted text indices.
                debug!("TextRunScanner: pushing box(es) in range: %?", self.clump);
                for clump.eachi |i| {
                    let range = new_ranges[i - self.clump.begin()];
                    if range.length() == 0 {
                        debug!("Elided an `UnscannedTextbox` because it was zero-length after \
                                compression; %s",
                               in_boxes[i].debug_str());
                        loop
                    }

                    do in_boxes[i].with_base |base| {
                        let new_box = @mut adapt_textbox_with_range(*base, run.get(), range);
                        out_boxes.push(TextRenderBoxClass(new_box));
                    }
                }
            }
        } // End of match.

        debug!("--- In boxes: ---");
        for in_boxes.iter().enumerate().advance |(i, box)| {
            debug!("%u --> %s", i, box.debug_str());
        }
        debug!("------------------");

        debug!("--- Out boxes: ---");
        for out_boxes.iter().enumerate().advance |(i, box)| {
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

        new_whitespace
    } // End of `flush_clump_to_list`.
}
