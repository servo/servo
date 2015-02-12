/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

//! Text layout.

#![deny(unsafe_blocks)]

use fragment::{Fragment, SpecificFragmentInfo, ScannedTextFragmentInfo};
use inline::InlineFragments;

use gfx::font::{DISABLE_KERNING_SHAPING_FLAG, FontMetrics, IGNORE_LIGATURES_SHAPING_FLAG};
use gfx::font::{RunMetrics, ShapingFlags, ShapingOptions};
use gfx::font_context::FontContext;
use gfx::text::glyph::CharIndex;
use gfx::text::text_run::TextRun;
use gfx::text::util::{self, CompressionMode};
use servo_util::dlist;
use servo_util::geometry::Au;
use servo_util::logical_geometry::{LogicalSize, WritingMode};
use servo_util::range::Range;
use servo_util::smallvec::{SmallVec, SmallVec1};
use std::collections::DList;
use std::mem;
use style::computed_values::{line_height, text_orientation, text_rendering, text_transform};
use style::computed_values::{white_space};
use style::properties::ComputedValues;
use style::properties::style_structs::Font as FontStyle;
use std::sync::Arc;

/// A stack-allocated object for scanning an inline flow into `TextRun`-containing `TextFragment`s.
pub struct TextRunScanner {
    pub clump: DList<Fragment>,
}

impl TextRunScanner {
    pub fn new() -> TextRunScanner {
        TextRunScanner {
            clump: DList::new(),
        }
    }

    pub fn scan_for_runs(&mut self, font_context: &mut FontContext, mut fragments: DList<Fragment>)
                         -> InlineFragments {
        debug!("TextRunScanner: scanning {} fragments for text runs...", fragments.len());

        // FIXME(pcwalton): We want to be sure not to allocate multiple times, since this is a
        // performance-critical spot, but this may overestimate and allocate too much memory.
        let mut new_fragments = Vec::with_capacity(fragments.len());
        let mut last_whitespace = true;
        while !fragments.is_empty() {
            // Create a clump.
            self.clump.append(&mut dlist::split_off_head(&mut fragments));
            while !fragments.is_empty() && self.clump
                                               .back()
                                               .unwrap()
                                               .can_merge_with_fragment(fragments.front()
                                                                                 .unwrap()) {
                self.clump.append(&mut dlist::split_off_head(&mut fragments));
            }

            // Flush that clump to the list of fragments we're building up.
            last_whitespace = self.flush_clump_to_list(font_context,
                                                       &mut new_fragments,
                                                       last_whitespace);
        }

        debug!("TextRunScanner: complete.");
        InlineFragments {
            fragments: new_fragments,
        }
    }

    /// A "clump" is a range of inline flow leaves that can be merged together into a single
    /// fragment. Adjacent text with the same style can be merged, and nothing else can.
    ///
    /// The flow keeps track of the fragments contained by all non-leaf DOM nodes. This is necessary
    /// for correct painting order. Since we compress several leaf fragments here, the mapping must
    /// be adjusted.
    fn flush_clump_to_list(&mut self,
                           font_context: &mut FontContext,
                           out_fragments: &mut Vec<Fragment>,
                           mut last_whitespace: bool)
                           -> bool {
        debug!("TextRunScanner: flushing {} fragments in range", self.clump.len());

        debug_assert!(!self.clump.is_empty());
        match self.clump.front().unwrap().specific {
            SpecificFragmentInfo::UnscannedText(_) => {}
            _ => {
                debug_assert!(self.clump.len() == 1,
                              "WAT: can't coalesce non-text nodes in flush_clump_to_list()!");
                out_fragments.push(self.clump.pop_front().unwrap());
                return last_whitespace
            }
        }

        // TODO(#177): Text run creation must account for the renderability of text by font group
        // fonts. This is probably achieved by creating the font group above and then letting
        // `FontGroup` decide which `Font` to stick into the text run.
        //
        // Concatenate all of the transformed strings together, saving the new character indices.
        let mut new_ranges: SmallVec1<Range<CharIndex>> = SmallVec1::new();
        let mut new_line_positions: SmallVec1<NewLinePositions> = SmallVec1::new();
        let mut char_total = CharIndex(0);
        let run = {
            let fontgroup;
            let compression;
            let text_transform;
            let letter_spacing;
            let word_spacing;
            let text_rendering;
            {
                let in_fragment = self.clump.front().unwrap();
                let font_style = in_fragment.style().get_font_arc();
                let inherited_text_style = in_fragment.style().get_inheritedtext();
                fontgroup = font_context.get_layout_font_group_for_style(font_style);
                compression = match in_fragment.white_space() {
                    white_space::T::normal | white_space::T::nowrap => {
                        CompressionMode::CompressWhitespaceNewline
                    }
                    white_space::T::pre => CompressionMode::CompressNone,
                };
                text_transform = inherited_text_style.text_transform;
                letter_spacing = inherited_text_style.letter_spacing;
                word_spacing = inherited_text_style.word_spacing.unwrap_or(Au(0));
                text_rendering = inherited_text_style.text_rendering;
            }

            // First, transform/compress text of all the nodes.
            let mut run_text = String::new();
            for in_fragment in self.clump.iter() {
                let in_fragment = match in_fragment.specific {
                    SpecificFragmentInfo::UnscannedText(ref text_fragment_info) => {
                        &text_fragment_info.text
                    }
                    _ => panic!("Expected an unscanned text fragment!"),
                };

                let mut new_line_pos = Vec::new();
                let old_length = CharIndex(run_text.chars().count() as int);
                last_whitespace = util::transform_text(in_fragment.as_slice(),
                                                       compression,
                                                       last_whitespace,
                                                       &mut run_text,
                                                       &mut new_line_pos);
                new_line_positions.push(NewLinePositions(new_line_pos));

                let added_chars = CharIndex(run_text.chars().count() as int) - old_length;
                new_ranges.push(Range::new(char_total, added_chars));
                char_total = char_total + added_chars;
            }

            // Account for `text-transform`. (Confusingly, this is not handled in "text
            // transformation" above, but we follow Gecko in the naming.)
            self.apply_style_transform_if_necessary(&mut run_text, text_transform);

            // Now create the run.
            //
            // TextRuns contain a cycle which is usually resolved by the teardown sequence.
            // If no clump takes ownership, however, it will leak.
            if run_text.len() == 0 {
                self.clump = DList::new();
                return last_whitespace
            }

            // Per CSS 2.1 § 16.4, "when the resultant space between two characters is not the same
            // as the default space, user agents should not use ligatures." This ensures that, for
            // example, `finally` with a wide `letter-spacing` renders as `f i n a l l y` and not
            // `ﬁ n a l l y`.
            let mut flags = ShapingFlags::empty();
            match letter_spacing {
                Some(Au(0)) | None => {}
                Some(_) => flags.insert(IGNORE_LIGATURES_SHAPING_FLAG),
            }
            if text_rendering == text_rendering::T::optimizespeed {
                flags.insert(IGNORE_LIGATURES_SHAPING_FLAG);
                flags.insert(DISABLE_KERNING_SHAPING_FLAG)
            }
            let options = ShapingOptions {
                letter_spacing: letter_spacing,
                word_spacing: word_spacing,
                flags: flags,
            };

            Arc::new(box TextRun::new(&mut *fontgroup.fonts.get(0).borrow_mut(),
                                      run_text,
                                      &options))
        };

        // Make new fragments with the run and adjusted text indices.
        debug!("TextRunScanner: pushing {} fragment(s)", self.clump.len());
        for (logical_offset, old_fragment) in
                mem::replace(&mut self.clump, DList::new()).into_iter().enumerate() {
            let range = *new_ranges.get(logical_offset);
            if range.is_empty() {
                debug!("Elided an `SpecificFragmentInfo::UnscannedText` because it was \
                        zero-length after compression");
                continue
            }

            let text_size = old_fragment.border_box.size;
            let &mut NewLinePositions(ref mut new_line_positions) =
                new_line_positions.get_mut(logical_offset);
            let mut new_text_fragment_info =
                box ScannedTextFragmentInfo::new(run.clone(),
                                                 range,
                                                 mem::replace(new_line_positions, Vec::new()),
                                                 text_size);
            let new_metrics = new_text_fragment_info.run.metrics_for_range(&range);
            let bounding_box_size = bounding_box_for_run_metrics(&new_metrics,
                                                                 old_fragment.style.writing_mode);
            new_text_fragment_info.content_size = bounding_box_size;
            let new_fragment =
                old_fragment.transform(bounding_box_size,
                                       SpecificFragmentInfo::ScannedText(new_text_fragment_info));
            out_fragments.push(new_fragment)
        }

        last_whitespace
    }

    /// Accounts for `text-transform`.
    ///
    /// FIXME(#4311, pcwalton): Case mapping can change length of the string; case mapping should
    /// be language-specific; `full-width`; use graphemes instead of characters.
    fn apply_style_transform_if_necessary(&mut self,
                                          string: &mut String,
                                          text_transform: text_transform::T) {
        match text_transform {
            text_transform::T::none => {}
            text_transform::T::uppercase => {
                let length = string.len();
                let original = mem::replace(string, String::with_capacity(length));
                for character in original.chars() {
                    string.push(character.to_uppercase())
                }
            }
            text_transform::T::lowercase => {
                let length = string.len();
                let original = mem::replace(string, String::with_capacity(length));
                for character in original.chars() {
                    string.push(character.to_lowercase())
                }
            }
            text_transform::T::capitalize => {
                let length = string.len();
                let original = mem::replace(string, String::with_capacity(length));
                let mut capitalize_next_letter = true;
                for character in original.chars() {
                    // FIXME(#4311, pcwalton): Should be the CSS/Unicode notion of a *typographic
                    // letter unit*, not an *alphabetic* character:
                    //
                    //    http://dev.w3.org/csswg/css-text/#typographic-letter-unit
                    if capitalize_next_letter && character.is_alphabetic() {
                        string.push(character.to_uppercase());
                        capitalize_next_letter = false;
                        continue
                    }

                    string.push(character);

                    // FIXME(#4311, pcwalton): Try UAX29 instead of just whitespace.
                    if character.is_whitespace() {
                        capitalize_next_letter = true
                    }
                }
            }
        }
    }
}

struct NewLinePositions(Vec<CharIndex>);

#[inline]
fn bounding_box_for_run_metrics(metrics: &RunMetrics, writing_mode: WritingMode)
                                -> LogicalSize<Au> {

    // This does nothing, but it will fail to build
    // when more values are added to the `text-orientation` CSS property.
    // This will be a reminder to update the code below.
    let dummy: Option<text_orientation::T> = None;
    match dummy {
        Some(text_orientation::T::sideways_right) |
        Some(text_orientation::T::sideways_left) |
        Some(text_orientation::T::sideways) |
        None => {}
    }

    // In vertical sideways or horizontal upright text,
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
pub fn font_metrics_for_style(font_context: &mut FontContext, font_style: Arc<FontStyle>)
                              -> FontMetrics {
    let fontgroup = font_context.get_layout_font_group_for_style(font_style);
    fontgroup.fonts.get(0).borrow().metrics.clone()
}

/// Returns the line block-size needed by the given computed style and font size.
pub fn line_height_from_style(style: &ComputedValues, metrics: &FontMetrics) -> Au {
    let font_size = style.get_font().font_size;
    match style.get_inheritedbox().line_height {
        line_height::T::Normal => metrics.line_gap,
        line_height::T::Number(l) => font_size.scale_by(l),
        line_height::T::Length(l) => l
    }
}
