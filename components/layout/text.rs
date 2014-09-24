/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

//! Text layout.

#![deny(unsafe_block)]

use flow::Flow;
use fragment::{Fragment, ScannedTextFragment, ScannedTextFragmentInfo, UnscannedTextFragment};

use gfx::font::{FontMetrics, FontStyle, RunMetrics};
use gfx::font_context::FontContext;
use gfx::text::glyph::CharIndex;
use gfx::text::text_run::TextRun;
use gfx::text::util::{CompressWhitespaceNewline, transform_text, CompressNone};
use servo_util::geometry::Au;
use servo_util::logical_geometry::{LogicalSize, WritingMode};
use servo_util::range::Range;
use style::ComputedValues;
use style::computed_values::{font_family, line_height, text_orientation, white_space};
use sync::Arc;

struct NewLinePositions {
    new_line_pos: Vec<CharIndex>,
}

// A helper function.
fn can_coalesce_text_nodes(fragments: &[Fragment], left_i: uint, right_i: uint) -> bool {
    assert!(left_i != right_i);
    fragments[left_i].can_merge_with_fragment(&fragments[right_i])
}

/// A stack-allocated object for scanning an inline flow into `TextRun`-containing `TextFragment`s.
pub struct TextRunScanner {
    pub clump: Range<CharIndex>,
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
            debug!("TextRunScanner: scanning {:u} fragments for text runs...", inline.fragments.len());
        }

        let fragments = &mut flow.as_inline().fragments;

        let mut last_whitespace = true;
        let mut new_fragments = Vec::new();
        for fragment_i in range(0, fragments.fragments.len()) {
            debug!("TextRunScanner: considering fragment: {:u}", fragment_i);
            if fragment_i > 0 && !can_coalesce_text_nodes(fragments.fragments.as_slice(), fragment_i - 1, fragment_i) {
                last_whitespace = self.flush_clump_to_list(font_context,
                                                           fragments.fragments.as_slice(),
                                                           &mut new_fragments,
                                                           last_whitespace);
            }

            self.clump.extend_by(CharIndex(1));
        }

        // Handle remaining clumps.
        if self.clump.length() > CharIndex(0) {
            drop(self.flush_clump_to_list(font_context,
                                          fragments.fragments.as_slice(),
                                          &mut new_fragments,
                                          last_whitespace))
        }

        debug!("TextRunScanner: swapping out fragments.");

        fragments.fragments = new_fragments;
    }

    /// A "clump" is a range of inline flow leaves that can be merged together into a single
    /// fragment. Adjacent text with the same style can be merged, and nothing else can.
    ///
    /// The flow keeps track of the fragments contained by all non-leaf DOM nodes. This is necessary
    /// for correct painting order. Since we compress several leaf fragments here, the mapping must
    /// be adjusted.
    ///
    /// FIXME(#2267, pcwalton): Stop cloning fragments. Instead we will need to replace each
    /// `in_fragment` with some smaller stub.
    fn flush_clump_to_list(&mut self,
                           font_context: &mut FontContext,
                           in_fragments: &[Fragment],
                           out_fragments: &mut Vec<Fragment>,
                           last_whitespace: bool)
                           -> bool {
        assert!(self.clump.length() > CharIndex(0));

        debug!("TextRunScanner: flushing fragments in range={}", self.clump);
        let is_singleton = self.clump.length() == CharIndex(1);

        let is_text_clump = match in_fragments[self.clump.begin().to_uint()].specific {
            UnscannedTextFragment(_) => true,
            _ => false,
        };

        let mut new_whitespace = last_whitespace;
        match (is_singleton, is_text_clump) {
            (false, false) => {
                fail!("WAT: can't coalesce non-text nodes in flush_clump_to_list()!")
            }
            (true, false) => {
                // FIXME(pcwalton): Stop cloning fragments, as above.
                debug!("TextRunScanner: pushing single non-text fragment in range: {}", self.clump);
                let new_fragment = in_fragments[self.clump.begin().to_uint()].clone();
                out_fragments.push(new_fragment)
            },
            (true, true)  => {
                let old_fragment = &in_fragments[self.clump.begin().to_uint()];
                let text = match old_fragment.specific {
                    UnscannedTextFragment(ref text_fragment_info) => &text_fragment_info.text,
                    _ => fail!("Expected an unscanned text fragment!"),
                };

                let font_style = old_fragment.font_style();

                let compression = match old_fragment.white_space() {
                    white_space::normal | white_space::nowrap => CompressWhitespaceNewline,
                    white_space::pre => CompressNone,
                };

                let mut new_line_pos = vec![];

                let (transformed_text, whitespace) = transform_text(text.as_slice(),
                                                                    compression,
                                                                    last_whitespace,
                                                                    &mut new_line_pos);

                new_whitespace = whitespace;

                if transformed_text.len() > 0 {
                    // TODO(#177): Text run creation must account for the renderability of text by
                    // font group fonts. This is probably achieved by creating the font group above
                    // and then letting `FontGroup` decide which `Font` to stick into the text run.
                    let fontgroup = font_context.get_layout_font_group_for_style(&font_style);
                    let run = box fontgroup.create_textrun(
                        transformed_text.clone());

                    debug!("TextRunScanner: pushing single text fragment in range: {} ({})",
                           self.clump,
                           *text);
                    let range = Range::new(CharIndex(0), run.char_len());
                    let new_metrics = run.metrics_for_range(&range);
                    let bounding_box_size = bounding_box_for_run_metrics(
                        &new_metrics, old_fragment.style.writing_mode);
                    let new_text_fragment_info = ScannedTextFragmentInfo::new(Arc::new(run), range);
                    let mut new_fragment = old_fragment.transform(
                        bounding_box_size, ScannedTextFragment(new_text_fragment_info));
                    new_fragment.new_line_pos = new_line_pos;
                    out_fragments.push(new_fragment)
                }
            },
            (false, true) => {
                // TODO(#177): Text run creation must account for the renderability of text by
                // font group fonts. This is probably achieved by creating the font group above
                // and then letting `FontGroup` decide which `Font` to stick into the text run.
                let in_fragment = &in_fragments[self.clump.begin().to_uint()];
                let font_style = in_fragment.font_style();
                let fontgroup = font_context.get_layout_font_group_for_style(&font_style);

                let compression = match in_fragment.white_space() {
                    white_space::normal | white_space::nowrap => CompressWhitespaceNewline,
                    white_space::pre => CompressNone,
                };

                let mut new_line_positions: Vec<NewLinePositions> = vec![];

                // First, transform/compress text of all the nodes.
                let mut last_whitespace_in_clump = new_whitespace;
                let transformed_strs: Vec<String> = Vec::from_fn(self.clump.length().to_uint(), |i| {
                    let idx = CharIndex(i as int) + self.clump.begin();
                    let in_fragment = match in_fragments[idx.to_uint()].specific {
                        UnscannedTextFragment(ref text_fragment_info) => &text_fragment_info.text,
                        _ => fail!("Expected an unscanned text fragment!"),
                    };

                    let mut new_line_pos = vec![];

                    let (new_str, new_whitespace) = transform_text(in_fragment.as_slice(),
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
                let mut run_str = String::new();
                let mut new_ranges: Vec<Range<CharIndex>> = vec![];
                let mut char_total = CharIndex(0);
                for i in range(0, transformed_strs.len() as int) {
                    let added_chars = CharIndex(transformed_strs[i as uint].as_slice().char_len() as int);
                    new_ranges.push(Range::new(char_total, added_chars));
                    run_str.push_str(transformed_strs[i as uint].as_slice());
                    char_total = char_total + added_chars;
                }

                // Now create the run.
                // TextRuns contain a cycle which is usually resolved by the teardown
                // sequence. If no clump takes ownership, however, it will leak.
                let clump = self.clump;
                let run = if clump.length() != CharIndex(0) && run_str.len() > 0 {
                    Some(Arc::new(box TextRun::new(
                        &mut *fontgroup.fonts[0].borrow_mut(),
                        run_str.to_string())))
                } else {
                    None
                };

                // Make new fragments with the run and adjusted text indices.
                debug!("TextRunScanner: pushing fragment(s) in range: {}", self.clump);
                for i in clump.each_index() {
                    let logical_offset = i - self.clump.begin();
                    let range = new_ranges[logical_offset.to_uint()];
                    if range.length() == CharIndex(0) {
                        debug!("Elided an `UnscannedTextFragment` because it was zero-length after \
                                compression; {}", in_fragments[i.to_uint()]);
                        continue
                    }

                    let new_text_fragment_info = ScannedTextFragmentInfo::new(run.as_ref().unwrap().clone(), range);
                    let old_fragment = &in_fragments[i.to_uint()];
                    let new_metrics = new_text_fragment_info.run.metrics_for_range(&range);
                    let bounding_box_size = bounding_box_for_run_metrics(
                        &new_metrics, old_fragment.style.writing_mode);
                    let mut new_fragment = old_fragment.transform(
                        bounding_box_size, ScannedTextFragment(new_text_fragment_info));
                    new_fragment.new_line_pos = new_line_positions[logical_offset.to_uint()].new_line_pos.clone();
                    out_fragments.push(new_fragment)
                }
            }
        } // End of match.

        let end = self.clump.end(); // FIXME: borrow checker workaround
        self.clump.reset(end, CharIndex(0));

        new_whitespace
    } // End of `flush_clump_to_list`.
}


#[inline]
fn bounding_box_for_run_metrics(metrics: &RunMetrics, writing_mode: WritingMode)
                                -> LogicalSize<Au> {

    // This does nothing, but it will fail to build
    // when more values are added to the `text-orientation` CSS property.
    // This will be a reminder to update the code below.
    let dummy: Option<text_orientation::T> = None;
    match dummy {
        Some(text_orientation::sideways_right) |
        Some(text_orientation::sideways_left) |
        Some(text_orientation::sideways) |
        None => {}
    }

    // In vertical sideways or horizontal upgright text,
    // the "width" of text metrics is always inline
    // This will need to be updated when other text orientations are supported.
    LogicalSize::new(
        writing_mode,
        metrics.bounding_box.size.width,
        metrics.bounding_box.size.height)

}

/// Returns the metrics of the font represented by the given `FontStyle`, respectively.
///
/// `#[inline]` because often the caller only needs a few fields from the font metrics.
#[inline]
pub fn font_metrics_for_style(font_context: &mut FontContext, font_style: &FontStyle)
                              -> FontMetrics {
    let fontgroup = font_context.get_layout_font_group_for_style(font_style);
    fontgroup.fonts[0].borrow().metrics.clone()
}

/// Converts a computed style to a font style used for rendering.
///
/// FIXME(pcwalton): This should not be necessary; just make the font part of the style sharable
/// with the display list somehow. (Perhaps we should use an ARC.)
pub fn computed_style_to_font_style(style: &ComputedValues) -> FontStyle {
    debug!("(font style) start");

    // FIXME: Too much allocation here.
    let mut font_families = style.get_font().font_family.iter().map(|family| {
        match *family {
            font_family::FamilyName(ref name) => (*name).clone(),
        }
    });
    debug!("(font style) font families: `{:?}`", font_families);

    let font_size = style.get_font().font_size.to_f64().unwrap() / 60.0;
    debug!("(font style) font size: `{:f}px`", font_size);

    FontStyle {
        pt_size: font_size,
        weight: style.get_font().font_weight,
        style: style.get_font().font_style,
        variant: style.get_font().font_variant,
        families: font_families.collect(),
    }
}

/// Returns the line block-size needed by the given computed style and font size.
pub fn line_height_from_style(style: &ComputedValues, metrics: &FontMetrics) -> Au {
    let font_size = style.get_font().font_size;
    let from_inline = match style.get_inheritedbox().line_height {
        line_height::Normal => metrics.line_gap,
        line_height::Number(l) => font_size.scale_by(l),
        line_height::Length(l) => l
    };
    let minimum = style.get_inheritedbox()._servo_minimum_line_height;
    Au::max(from_inline, minimum)
}

