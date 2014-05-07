/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

//! Text layout.

use layout::box_::{Box, ScannedTextBox, ScannedTextBoxInfo, UnscannedTextBox};
use layout::flow::Flow;
use layout::inline::InlineBoxes;

use gfx::font::{FontMetrics, FontStyle};
use gfx::font_context::FontContext;
use gfx::text::text_run::TextRun;
use gfx::text::util::{CompressWhitespaceNewline, transform_text, CompressNone};
use servo_util::geometry::Au;
use servo_util::range::Range;
use std::mem;
use style::ComputedValues;
use style::computed_values::{font_family, line_height, white_space};
use sync::Arc;

struct NewLinePositions {
    new_line_pos: Vec<uint>,
}

// A helper function.
fn can_coalesce_text_nodes(boxes: &[Box], left_i: uint, right_i: uint) -> bool {
    assert!(left_i != right_i);
    boxes[left_i].can_merge_with_box(&boxes[right_i])
}

/// A stack-allocated object for scanning an inline flow into `TextRun`-containing `TextBox`es.
pub struct TextRunScanner {
    pub clump: Range,
}

impl TextRunScanner {
    pub fn new() -> TextRunScanner {
        TextRunScanner {
            clump: Range::empty(),
        }
    }

    pub fn scan_for_runs(&mut self, font_context: &mut FontContext, flow: &mut Flow) {
        {
            let inline = flow.as_immutable_inline();
            debug!("TextRunScanner: scanning {:u} boxes for text runs...", inline.boxes.len());
        }

        let InlineBoxes {
            boxes: old_boxes,
            map: mut map
        } = mem::replace(&mut flow.as_inline().boxes, InlineBoxes::new());

        let mut last_whitespace = true;
        let mut new_boxes = Vec::new();
        for box_i in range(0, old_boxes.len()) {
            debug!("TextRunScanner: considering box: {:u}", box_i);
            if box_i > 0 && !can_coalesce_text_nodes(old_boxes.as_slice(), box_i - 1, box_i) {
                last_whitespace = self.flush_clump_to_list(font_context,
                                                           old_boxes.as_slice(),
                                                           &mut new_boxes,
                                                           last_whitespace);
            }

            self.clump.extend_by(1);
        }

        // Handle remaining clumps.
        if self.clump.length() > 0 {
            drop(self.flush_clump_to_list(font_context,
                                          old_boxes.as_slice(),
                                          &mut new_boxes,
                                          last_whitespace))
        }

        debug!("TextRunScanner: swapping out boxes.");

        // Swap out the old and new box list of the flow.
        map.fixup(old_boxes.as_slice(), new_boxes.as_slice());
        flow.as_inline().boxes = InlineBoxes {
            boxes: new_boxes,
            map: map,
        }
    }

    /// A "clump" is a range of inline flow leaves that can be merged together into a single box.
    /// Adjacent text with the same style can be merged, and nothing else can.
    ///
    /// The flow keeps track of the boxes contained by all non-leaf DOM nodes. This is necessary
    /// for correct painting order. Since we compress several leaf boxes here, the mapping must be
    /// adjusted.
    ///
    /// FIXME(#2267, pcwalton): Stop cloning boxes. Instead we will need to replace each `in_box`
    /// with some smaller stub.
    pub fn flush_clump_to_list(&mut self,
                               font_context: &mut FontContext,
                               in_boxes: &[Box],
                               out_boxes: &mut Vec<Box>,
                               last_whitespace: bool)
                               -> bool {
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
                fail!("WAT: can't coalesce non-text nodes in flush_clump_to_list()!")
            }
            (true, false) => {
                // FIXME(pcwalton): Stop cloning boxes, as above.
                debug!("TextRunScanner: pushing single non-text box in range: {}", self.clump);
                let new_box = in_boxes[self.clump.begin()].clone();
                out_boxes.push(new_box)
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
                let compression = match old_box.white_space() {
                    white_space::normal => CompressWhitespaceNewline,
                    white_space::pre => CompressNone,
                };

                let mut new_line_pos = vec!();

                let (transformed_text, whitespace) = transform_text(*text,
                                                                    compression,
                                                                    last_whitespace,
                                                                    &mut new_line_pos);

                new_whitespace = whitespace;

                if transformed_text.len() > 0 {
                    // TODO(#177): Text run creation must account for the renderability of text by
                    // font group fonts. This is probably achieved by creating the font group above
                    // and then letting `FontGroup` decide which `Font` to stick into the text run.
                    let fontgroup = font_context.get_resolved_font_for_style(&font_style);
                    let run = ~fontgroup.borrow().create_textrun(transformed_text.clone(),
                                                                 decoration);

                    debug!("TextRunScanner: pushing single text box in range: {} ({})",
                           self.clump,
                           *text);
                    let range = Range::new(0, run.char_len());
                    let new_metrics = run.metrics_for_range(&range);
                    let new_text_box_info = ScannedTextBoxInfo::new(Arc::new(run), range);
                    let mut new_box = old_box.transform(new_metrics.bounding_box.size,
                                                    ScannedTextBox(new_text_box_info));
                    new_box.new_line_pos = new_line_pos;
                    out_boxes.push(new_box)
                }
            },
            (false, true) => {
                // TODO(#177): Text run creation must account for the renderability of text by
                // font group fonts. This is probably achieved by creating the font group above
                // and then letting `FontGroup` decide which `Font` to stick into the text run.
                let in_box = &in_boxes[self.clump.begin()];
                let font_style = in_box.font_style();
                let fontgroup = font_context.get_resolved_font_for_style(&font_style);
                let decoration = in_box.text_decoration();

                // TODO(#115): Use the actual CSS `white-space` property of the relevant style.
                let compression = match in_box.white_space() {
                    white_space::normal => CompressWhitespaceNewline,
                    white_space::pre => CompressNone,
                };

                let mut new_line_positions: Vec<NewLinePositions> = vec!();

                // First, transform/compress text of all the nodes.
                let mut last_whitespace_in_clump = new_whitespace;
                let transformed_strs: Vec<~str> = Vec::from_fn(self.clump.length(), |i| {
                    // TODO(#113): We should be passing the compression context between calls to
                    // `transform_text`, so that boxes starting and/or ending with whitespace can
                    // be compressed correctly with respect to the text run.
                    let idx = i + self.clump.begin();
                    let in_box = match in_boxes[idx].specific {
                        UnscannedTextBox(ref text_box_info) => &text_box_info.text,
                        _ => fail!("Expected an unscanned text box!"),
                    };

                    let mut new_line_pos = vec!();

                    let (new_str, new_whitespace) = transform_text(*in_box,
                                                                   compression,
                                                                   last_whitespace_in_clump,
                                                                   &mut new_line_pos);
                    new_line_positions.push(NewLinePositions { new_line_pos: new_line_pos });

                    last_whitespace_in_clump = new_whitespace;
                    new_str
                });
                new_whitespace = last_whitespace_in_clump;

                // Next, concatenate all of the transformed strings together, saving the new
                // character indices.
                let mut run_str: ~str = "".to_owned();
                let mut new_ranges: Vec<Range> = vec!();
                let mut char_total = 0;
                for i in range(0, transformed_strs.len()) {
                    let added_chars = transformed_strs.get(i).char_len();
                    new_ranges.push(Range::new(char_total, added_chars));
                    run_str.push_str(*transformed_strs.get(i));
                    char_total += added_chars;
                }

                // Now create the run.
                // TextRuns contain a cycle which is usually resolved by the teardown
                // sequence. If no clump takes ownership, however, it will leak.
                let clump = self.clump;
                let run = if clump.length() != 0 && run_str.len() > 0 {
                    Some(Arc::new(~TextRun::new(&mut *fontgroup.borrow().fonts.get(0).borrow_mut(),
                                                run_str.clone(), decoration)))
                } else {
                    None
                };

                // Make new boxes with the run and adjusted text indices.
                debug!("TextRunScanner: pushing box(es) in range: {}", self.clump);
                for i in clump.eachi() {
                    let logical_offset = i - self.clump.begin();
                    let range = new_ranges.get(logical_offset);
                    if range.length() == 0 {
                        debug!("Elided an `UnscannedTextbox` because it was zero-length after \
                                compression; {}", in_boxes[i]);
                        continue
                    }

                    let new_text_box_info = ScannedTextBoxInfo::new(run.get_ref().clone(), *range);
                    let new_metrics = new_text_box_info.run.metrics_for_range(range);
                    let mut new_box = in_boxes[i].transform(new_metrics.bounding_box.size,
                                                            ScannedTextBox(new_text_box_info));
                    new_box.new_line_pos = new_line_positions.get(logical_offset).new_line_pos.clone();
                    out_boxes.push(new_box)
                }
            }
        } // End of match.

        let end = self.clump.end(); // FIXME: borrow checker workaround
        self.clump.reset(end, 0);

        new_whitespace
    } // End of `flush_clump_to_list`.
}

/// Returns the metrics of the font represented by the given `FontStyle`, respectively.
///
/// `#[inline]` because often the caller only needs a few fields from the font metrics.
#[inline]
pub fn font_metrics_for_style(font_context: &mut FontContext, font_style: &FontStyle)
                              -> FontMetrics {
    let fontgroup = font_context.get_resolved_font_for_style(font_style);
    fontgroup.borrow().fonts.get(0).borrow().metrics.clone()
}

/// Converts a computed style to a font style used for rendering.
///
/// FIXME(pcwalton): This should not be necessary; just make the font part of the style sharable
/// with the display list somehow. (Perhaps we should use an ARC.)
pub fn computed_style_to_font_style(style: &ComputedValues) -> FontStyle {
    debug!("(font style) start");

    // FIXME: Too much allocation here.
    let mut font_families = style.Font.get().font_family.iter().map(|family| {
        match *family {
            font_family::FamilyName(ref name) => (*name).clone(),
        }
    });
    debug!("(font style) font families: `{:?}`", font_families);

    let font_size = style.Font.get().font_size.to_f64().unwrap() / 60.0;
    debug!("(font style) font size: `{:f}px`", font_size);

    FontStyle {
        pt_size: font_size,
        weight: style.Font.get().font_weight,
        style: style.Font.get().font_style,
        families: font_families.collect(),
    }
}

/// Returns the line height needed by the given computed style and font size.
///
/// FIXME(pcwalton): I believe this should not take a separate `font-size` parameter.
pub fn line_height_from_style(style: &ComputedValues, font_size: Au) -> Au {
    let from_inline = match style.InheritedBox.get().line_height {
        line_height::Normal => font_size.scale_by(1.14),
        line_height::Number(l) => font_size.scale_by(l),
        line_height::Length(l) => l
    };
    let minimum = style.InheritedBox.get()._servo_minimum_line_height;
    Au::max(from_inline, minimum)
}

