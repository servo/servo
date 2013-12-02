/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

//! Text layout.

use std::vec;

use gfx::text::text_run::TextRun;
use gfx::text::util::{CompressWhitespaceNewline, transform_text};
use layout::box::{RenderBox, RenderBoxUtils, TextRenderBox, UnscannedTextRenderBoxClass};
use layout::context::LayoutContext;
use layout::flow::Flow;
use servo_util::range::Range;

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

    pub fn scan_for_runs(&mut self, ctx: &LayoutContext, flow: &mut Flow) {
        {
            let inline = flow.as_immutable_inline();
            // FIXME: this assertion fails on wikipedia, but doesn't seem
            // to cause problems.
            // assert!(inline.boxes.len() > 0);
            debug!("TextRunScanner: scanning {:u} boxes for text runs...", inline.boxes.len());
        }

        let mut last_whitespace = true;
        let mut out_boxes = ~[];
        for box_i in range(0, flow.as_immutable_inline().boxes.len()) {
            debug!("TextRunScanner: considering box: {:u}", box_i);
            if box_i > 0 && !can_coalesce_text_nodes(flow.as_immutable_inline().boxes,
                                                     box_i - 1,
                                                     box_i) {
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
        flow.as_inline().boxes = out_boxes;

        // A helper function.
        fn can_coalesce_text_nodes(boxes: &[@RenderBox], left_i: uint, right_i: uint) -> bool {
            assert!(left_i < boxes.len());
            assert!(right_i > 0 && right_i < boxes.len());
            assert!(left_i != right_i);

            let (left, right) = (boxes[left_i], boxes[right_i]);
            match (left.class(), right.class()) {
                (UnscannedTextRenderBoxClass, UnscannedTextRenderBoxClass) => {
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
                               flow: &mut Flow,
                               last_whitespace: bool,
                               out_boxes: &mut ~[@RenderBox])
                               -> bool {
        let inline = flow.as_inline();
        let in_boxes = &inline.boxes;

        assert!(self.clump.length() > 0);

        debug!("TextRunScanner: flushing boxes in range={}", self.clump);
        let is_singleton = self.clump.length() == 1;
        let possible_text_clump = in_boxes[self.clump.begin()]; // FIXME(pcwalton): Rust bug
        let is_text_clump = possible_text_clump.class() == UnscannedTextRenderBoxClass;

        let mut new_whitespace = last_whitespace;

        match (is_singleton, is_text_clump) {
            (false, false) => {
                fail!(~"WAT: can't coalesce non-text nodes in flush_clump_to_list()!")
            }
            (true, false) => {
                debug!("TextRunScanner: pushing single non-text box in range: {}", self.clump);
                out_boxes.push(in_boxes[self.clump.begin()]);
            },
            (true, true)  => {
                let old_box = in_boxes[self.clump.begin()];
                let text = old_box.as_unscanned_text_render_box().raw_text();
                let font_style = old_box.base().font_style();
                let decoration = old_box.base().text_decoration();

                // TODO(#115): Use the actual CSS `white-space` property of the relevant style.
                let compression = CompressWhitespaceNewline;

                let (transformed_text, whitespace) = transform_text(text, compression, last_whitespace);
                new_whitespace = whitespace;

                if transformed_text.len() > 0 {
                    // TODO(#177): Text run creation must account for the renderability of text by
                    // font group fonts. This is probably achieved by creating the font group above
                    // and then letting `FontGroup` decide which `Font` to stick into the text run.
                    let fontgroup = ctx.font_ctx.get_resolved_font_for_style(&font_style);
                    let run = @fontgroup.create_textrun(transformed_text, decoration);

                    debug!("TextRunScanner: pushing single text box in range: {} ({})", self.clump, text);
                    let range = Range::new(0, run.char_len());
                    let new_box = @TextRenderBox::new((*old_box.base()).clone(), run, range);

                    out_boxes.push(new_box as @RenderBox);
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
                    let in_box = in_boxes[idx].as_unscanned_text_render_box().raw_text();
                    let (new_str, new_whitespace) = transform_text(in_box,
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
                for i in range(0, transformed_strs.len()) {
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
                let in_box = in_boxes[self.clump.begin()];
                let font_style = in_box.base().font_style();
                let fontgroup = ctx.font_ctx.get_resolved_font_for_style(&font_style);
                let decoration = in_box.base().text_decoration();

                // TextRuns contain a cycle which is usually resolved by the teardown
                // sequence. If no clump takes ownership, however, it will leak.
                let clump = self.clump;
                let run = if clump.length() != 0 && run_str.len() > 0 {
                    Some(@TextRun::new(fontgroup.fonts[0], run_str, decoration))
                } else {
                    None
                };

                // Make new boxes with the run and adjusted text indices.
                debug!("TextRunScanner: pushing box(es) in range: {}", self.clump);
                for i in clump.eachi() {
                    let range = new_ranges[i - self.clump.begin()];
                    if range.length() == 0 {
                        debug!("Elided an `UnscannedTextbox` because it was zero-length after \
                                compression; {:s}",
                               in_boxes[i].debug_str());
                        continue
                    }

                    let new_box = @TextRenderBox::new((*in_boxes[i].base()).clone(),
                                                      run.unwrap(),
                                                      range);
                    out_boxes.push(new_box as @RenderBox);
                }
            }
        } // End of match.

        debug!("--- In boxes: ---");
        for (i, box) in in_boxes.iter().enumerate() {
            debug!("{:u} --> {:s}", i, box.debug_str());
        }
        debug!("------------------");

        debug!("--- Out boxes: ---");
        for (i, box) in out_boxes.iter().enumerate() {
            debug!("{:u} --> {:s}", i, box.debug_str());
        }
        debug!("------------------");

        debug!("--- Elem ranges: ---");
        for (i, nr) in inline.elems.eachi() {
            debug!("{:u}: {} --> {:s}", i, nr.range, nr.node.debug_str()); ()
        }
        debug!("--------------------");

        let end = self.clump.end(); // FIXME: borrow checker workaround
        self.clump.reset(end, 0);

        new_whitespace
    } // End of `flush_clump_to_list`.
}
