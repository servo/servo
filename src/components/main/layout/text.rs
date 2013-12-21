/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

//! Text layout.

use layout::box_::{Box, ScannedTextBox, ScannedTextBoxInfo, UnscannedTextBox};
use layout::context::LayoutContext;
use layout::flow::Flow;

use extra::arc::Arc;
use gfx::text::text_run::TextRun;
use gfx::text::util::{CompressWhitespaceNewline, transform_text};
use servo_util::range::Range;
use std::vec;

/// A stack-allocated object for scanning an inline flow into `TextRun`-containing `TextBox`es.
pub struct TextRunScanner {
    clump: Range,
}

impl TextRunScanner {
    pub fn new() -> TextRunScanner {
        TextRunScanner {
            clump: Range::empty(),
        }
    }

    pub fn scan_for_runs(&mut self, ctx: &mut LayoutContext, flow: &mut Flow) {
        {
            let inline = flow.as_immutable_inline();
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
        fn can_coalesce_text_nodes(boxes: &[Box], left_i: uint, right_i: uint) -> bool {
            assert!(left_i < boxes.len());
            assert!(right_i > 0 && right_i < boxes.len());
            assert!(left_i != right_i);
            boxes[left_i].can_merge_with_box(&boxes[right_i])
        }
    }

    /// A "clump" is a range of inline flow leaves that can be merged together into a single box.
    /// Adjacent text with the same style can be merged, and nothing else can.
    ///
    /// The flow keeps track of the boxes contained by all non-leaf DOM nodes. This is necessary
    /// for correct painting order. Since we compress several leaf boxes here, the mapping must be
    /// adjusted.
    ///
    /// FIXME(pcwalton): Stop cloning boxes. Instead we will need to consume the `in_box`es as we
    /// iterate over them.
    pub fn flush_clump_to_list(&mut self,
                               ctx: &mut LayoutContext,
                               flow: &mut Flow,
                               last_whitespace: bool,
                               out_boxes: &mut ~[Box])
                               -> bool {
        let inline = flow.as_inline();
        let in_boxes = &mut inline.boxes;

        assert!(self.clump.length() > 0);

        debug!("TextRunScanner: flushing boxes in range={}", self.clump);
        let is_singleton = self.clump.length() == 1;

        let is_text_clump = match in_boxes[self.clump.begin()].specific {
            UnscannedTextBox(_) => true,
            _ => false,
        };

        let mut new_whitespace = last_whitespace;
        match (is_singleton, is_text_clump) {
            (false, false) => {
                fail!(~"WAT: can't coalesce non-text nodes in flush_clump_to_list()!")
            }
            (true, false) => {
                // FIXME(pcwalton): Stop cloning boxes, as above.
                debug!("TextRunScanner: pushing single non-text box in range: {}", self.clump);
                out_boxes.push(in_boxes[self.clump.begin()].clone());
            },
            (true, true)  => {
                let old_box = &in_boxes[self.clump.begin()];
                let text = match old_box.specific {
                    UnscannedTextBox(ref text_box_info) => &text_box_info.text,
                    _ => fail!("Expected an unscanned text box!"),
                };

                let font_style = old_box.font_style();
                let decoration = old_box.text_decoration();

                // TODO(#115): Use the actual CSS `white-space` property of the relevant style.
                let compression = CompressWhitespaceNewline;

                let (transformed_text, whitespace) = transform_text(*text,
                                                                    compression,
                                                                    last_whitespace);
                new_whitespace = whitespace;

                if transformed_text.len() > 0 {
                    // TODO(#177): Text run creation must account for the renderability of text by
                    // font group fonts. This is probably achieved by creating the font group above
                    // and then letting `FontGroup` decide which `Font` to stick into the text run.
                    let fontgroup = ctx.font_ctx.get_resolved_font_for_style(&font_style);
                    let run = ~fontgroup.borrow().with(|fg| fg.create_textrun(transformed_text.clone(), decoration));

                    debug!("TextRunScanner: pushing single text box in range: {} ({})",
                           self.clump,
                           *text);
                    let range = Range::new(0, run.char_len());
                    let new_metrics = run.metrics_for_range(&range);
                    let new_text_box_info = ScannedTextBoxInfo::new(Arc::new(run), range);
                    let new_box = old_box.transform(new_metrics.bounding_box.size,
                                                    ScannedTextBox(new_text_box_info));
                    out_boxes.push(new_box)
                }
            },
            (false, true) => {
                // TODO(#115): Use the actual CSS `white-space` property of the relevant style.
                let compression = CompressWhitespaceNewline;

                // First, transform/compress text of all the nodes.
                let mut last_whitespace_in_clump = new_whitespace;
                let transformed_strs: ~[~str] = vec::from_fn(self.clump.length(), |i| {
                    // TODO(#113): We should be passing the compression context between calls to
                    // `transform_text`, so that boxes starting and/or ending with whitespace can
                    // be compressed correctly with respect to the text run.
                    let idx = i + self.clump.begin();
                    let in_box = match in_boxes[idx].specific {
                        UnscannedTextBox(ref text_box_info) => &text_box_info.text,
                        _ => fail!("Expected an unscanned text box!"),
                    };

                    let (new_str, new_whitespace) = transform_text(*in_box,
                                                                   compression,
                                                                   last_whitespace_in_clump);
                    last_whitespace_in_clump = new_whitespace;
                    new_str
                });
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
                let in_box = &in_boxes[self.clump.begin()];
                let font_style = in_box.font_style();
                let fontgroup = ctx.font_ctx.get_resolved_font_for_style(&font_style);
                let decoration = in_box.text_decoration();

                // TextRuns contain a cycle which is usually resolved by the teardown
                // sequence. If no clump takes ownership, however, it will leak.
                let clump = self.clump;
                let run = if clump.length() != 0 && run_str.len() > 0 {
                    fontgroup.borrow().with(|fg| {
                        fg.fonts[0].borrow().with_mut(|font| {
                            Some(Arc::new(~TextRun::new(font, run_str.clone(), decoration)))
                        })
                    })
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

                    let new_text_box_info = ScannedTextBoxInfo::new(run.get_ref().clone(), range);
                    let new_metrics = new_text_box_info.run.get().metrics_for_range(&range);
                    let new_box = in_boxes[i].transform(new_metrics.bounding_box.size,
                                                        ScannedTextBox(new_text_box_info));
                    out_boxes.push(new_box)
                }
            }
        } // End of match.

        debug!("--- In boxes: ---");
        for (i, box_) in in_boxes.iter().enumerate() {
            debug!("{:u} --> {:s}", i, box_.debug_str());
        }
        debug!("------------------");

        debug!("--- Out boxes: ---");
        for (i, box_) in out_boxes.iter().enumerate() {
            debug!("{:u} --> {:s}", i, box_.debug_str());
        }
        debug!("------------------");

        debug!("--- Elem ranges: ---");
        for (i, nr) in inline.elems.eachi() {
            debug!("{:u}: {} --> {:?}", i, nr.range, nr.node.id()); ()
        }
        debug!("--------------------");

        let end = self.clump.end(); // FIXME: borrow checker workaround
        self.clump.reset(end, 0);

        new_whitespace
    } // End of `flush_clump_to_list`.
}
