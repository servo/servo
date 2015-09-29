/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

//! Text layout.

#![deny(unsafe_code)]

use fragment::{Fragment, ScannedTextFragmentInfo, SpecificFragmentInfo, UnscannedTextFragmentInfo};
use gfx::font::{DISABLE_KERNING_SHAPING_FLAG, FontMetrics, IGNORE_LIGATURES_SHAPING_FLAG};
use gfx::font::{RTL_FLAG, RunMetrics, ShapingFlags, ShapingOptions};
use gfx::font_context::FontContext;
use gfx::text::glyph::CharIndex;
use gfx::text::text_run::TextRun;
use gfx::text::util::{self, CompressionMode};
use inline::InlineFragments;
use std::borrow::ToOwned;
use std::collections::LinkedList;
use std::mem;
use std::sync::Arc;
use style::computed_values::{line_height, text_orientation, text_rendering, text_transform};
use style::computed_values::{white_space};
use style::properties::ComputedValues;
use style::properties::style_structs::Font as FontStyle;
use unicode_bidi::{is_rtl, process_text};
use unicode_script::{get_script, Script};
use util::geometry::Au;
use util::linked_list::split_off_head;
use util::logical_geometry::{LogicalSize, WritingMode};
use util::range::{Range, RangeIndex};

/// Returns the concatenated text of a list of unscanned text fragments.
fn text(fragments: &LinkedList<Fragment>) -> String {
    // FIXME: Some of this work is later duplicated in split_first_fragment_at_newline_if_necessary
    // and transform_text.  This code should be refactored so that the all the scanning for
    // newlines is done in a single pass.

    let mut text = String::new();

    for fragment in fragments {
        match fragment.specific {
            SpecificFragmentInfo::UnscannedText(ref info) => {
                match fragment.white_space() {
                    white_space::T::normal | white_space::T::nowrap => {
                        text.push_str(&info.text.replace("\n", " "));
                    }
                    white_space::T::pre => {
                        text.push_str(&info.text);
                    }
                }
            }
            _ => {}
        }
    }
    text
}


/// A stack-allocated object for scanning an inline flow into `TextRun`-containing `TextFragment`s.
pub struct TextRunScanner {
    pub clump: LinkedList<Fragment>,
}

impl TextRunScanner {
    pub fn new() -> TextRunScanner {
        TextRunScanner {
            clump: LinkedList::new(),
        }
    }

    pub fn scan_for_runs(&mut self,
                         font_context: &mut FontContext,
                         mut fragments: LinkedList<Fragment>)
                         -> InlineFragments {
        debug!("TextRunScanner: scanning {} fragments for text runs...", fragments.len());
        debug_assert!(!fragments.is_empty());

        // Calculate bidi embedding levels, so we can split bidirectional fragments for reordering.
        let text = text(&fragments);
        let para_level = fragments.front().unwrap().style.writing_mode.to_bidi_level();
        let bidi_info = process_text(&text, Some(para_level));

        // Optimization: If all the text is LTR, don't bother splitting on bidi levels.
        let bidi_levels = if bidi_info.levels.iter().cloned().any(is_rtl) {
            Some(&bidi_info.levels[..])
        } else {
            None
        };

        // FIXME(pcwalton): We want to be sure not to allocate multiple times, since this is a
        // performance-critical spot, but this may overestimate and allocate too much memory.
        let mut new_fragments = Vec::with_capacity(fragments.len());
        let mut last_whitespace = false;
        let mut paragraph_bytes_processed = 0;

        while !fragments.is_empty() {
            // Create a clump.
            split_first_fragment_at_newline_if_necessary(&mut fragments);
            self.clump.append(&mut split_off_head(&mut fragments));
            while !fragments.is_empty() && self.clump
                                               .back()
                                               .unwrap()
                                               .can_merge_with_fragment(fragments.front()
                                                                                 .unwrap()) {
                split_first_fragment_at_newline_if_necessary(&mut fragments);
                self.clump.append(&mut split_off_head(&mut fragments));
            }

            // Flush that clump to the list of fragments we're building up.
            last_whitespace = self.flush_clump_to_list(font_context,
                                                       &mut new_fragments,
                                                       &mut paragraph_bytes_processed,
                                                       bidi_levels,
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
                           paragraph_bytes_processed: &mut usize,
                           bidi_levels: Option<&[u8]>,
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
                return false
            }
        }

        // Concatenate all of the transformed strings together, saving the new character indices.
        let mut mappings: Vec<RunMapping> = Vec::new();
        let runs = {
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
                fontgroup = font_context.layout_font_group_for_style(font_style);
                compression = match in_fragment.white_space() {
                    white_space::T::normal | white_space::T::nowrap => {
                        CompressionMode::CompressWhitespaceNewline
                    }
                    white_space::T::pre => CompressionMode::CompressNone,
                };
                text_transform = inherited_text_style.text_transform;
                letter_spacing = inherited_text_style.letter_spacing.0;
                word_spacing = inherited_text_style.word_spacing.0.unwrap_or(Au(0));
                text_rendering = inherited_text_style.text_rendering;
            }

            // First, transform/compress text of all the nodes.
            let (mut run_info_list, mut run_info) = (Vec::new(), RunInfo::new());
            for (fragment_index, in_fragment) in self.clump.iter().enumerate() {
                let mut mapping = RunMapping::new(&run_info_list[..], &run_info, fragment_index);
                let text;
                let insertion_point;
                match in_fragment.specific {
                    SpecificFragmentInfo::UnscannedText(ref text_fragment_info) => {
                        text = &text_fragment_info.text;
                        insertion_point = text_fragment_info.insertion_point;
                    }
                    _ => panic!("Expected an unscanned text fragment!"),
                };

                let (mut start_position, mut end_position) = (0, 0);
                for character in text.chars() {
                    // Search for the first font in this font group that contains a glyph for this
                    // character.
                    let mut font_index = 0;
                    while font_index < fontgroup.fonts.len() - 1 {
                        if fontgroup.fonts.get(font_index).unwrap().borrow()
                                          .glyph_index(character)
                                          .is_some() {
                            break
                        }
                        font_index += 1;
                    }

                    let bidi_level = match bidi_levels {
                        Some(levels) => levels[*paragraph_bytes_processed],
                        None => 0
                    };

                    // Break the run if the new character has a different explicit script than the
                    // previous characters.
                    //
                    // TODO: Special handling of paired punctuation characters.
                    // http://www.unicode.org/reports/tr24/#Common
                    let script = get_script(character);
                    let compatible_script = is_compatible(script, run_info.script);
                    if compatible_script && !is_specific(run_info.script) && is_specific(script) {
                        run_info.script = script;
                    }

                    // Now, if necessary, flush the mapping we were building up.
                    if run_info.font_index != font_index ||
                       run_info.bidi_level != bidi_level ||
                       !compatible_script
                    {
                        if end_position > start_position {
                            mapping.flush(&mut mappings,
                                          &mut run_info,
                                          &**text,
                                          insertion_point,
                                          compression,
                                          text_transform,
                                          &mut last_whitespace,
                                          &mut start_position,
                                          end_position);
                        }
                        if run_info.text.len() > 0 {
                            run_info_list.push(run_info);
                            run_info = RunInfo::new();
                            mapping = RunMapping::new(&run_info_list[..],
                                                      &run_info,
                                                      fragment_index);
                        }
                        run_info.font_index = font_index;
                        run_info.bidi_level = bidi_level;
                        run_info.script = script;
                    }

                    // Consume this character.
                    end_position += character.len_utf8();
                    *paragraph_bytes_processed += character.len_utf8();
                }

                // If the mapping is zero-length, don't flush it.
                if start_position == end_position {
                    continue
                }

                // Flush the last mapping we created for this fragment to the list.
                mapping.flush(&mut mappings,
                              &mut run_info,
                              &**text,
                              insertion_point,
                              compression,
                              text_transform,
                              &mut last_whitespace,
                              &mut start_position,
                              end_position);
            }

            // Push the final run info.
            run_info_list.push(run_info);

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
                script: Script::Common,
                flags: flags,
            };

            // FIXME(https://github.com/rust-lang/rust/issues/23338)
            run_info_list.into_iter().map(|run_info| {
                let mut options = options;
                options.script = run_info.script;
                if is_rtl(run_info.bidi_level) {
                    options.flags.insert(RTL_FLAG);
                }
                let mut font = fontgroup.fonts.get(run_info.font_index).unwrap().borrow_mut();
                ScannedTextRun {
                    run: Arc::new(TextRun::new(&mut *font,
                                               run_info.text,
                                               &options,
                                               run_info.bidi_level)),
                    insertion_point: run_info.insertion_point,
                }
            }).collect::<Vec<_>>()
        };

        // Make new fragments with the runs and adjusted text indices.
        debug!("TextRunScanner: pushing {} fragment(s)", self.clump.len());
        let mut mappings = mappings.into_iter().peekable();
        for (logical_offset, old_fragment) in
                mem::replace(&mut self.clump, LinkedList::new()).into_iter().enumerate() {
             loop {
                match mappings.peek() {
                    Some(mapping) if mapping.old_fragment_index == logical_offset => {}
                    Some(_) | None => {
                        if let Some(ref mut last_fragment) = out_fragments.last_mut() {
                            last_fragment.meld_with_next_inline_fragment(&old_fragment);
                        }
                        break;
                    }
                };

                let mut mapping = mappings.next().unwrap();
                let scanned_run = runs[mapping.text_run_index].clone();

                let requires_line_break_afterward_if_wrapping_on_newlines =
                    scanned_run.run.text.char_at_reverse(mapping.byte_range.end()) == '\n';
                if requires_line_break_afterward_if_wrapping_on_newlines {
                    mapping.char_range.extend_by(CharIndex(-1));
                }

                let text_size = old_fragment.border_box.size;
                let mut new_text_fragment_info = box ScannedTextFragmentInfo::new(
                    scanned_run.run,
                    mapping.char_range,
                    text_size,
                    &scanned_run.insertion_point,
                    requires_line_break_afterward_if_wrapping_on_newlines);

                let new_metrics = new_text_fragment_info.run.metrics_for_range(&mapping.char_range);
                let writing_mode = old_fragment.style.writing_mode;
                let bounding_box_size = bounding_box_for_run_metrics(&new_metrics, writing_mode);
                new_text_fragment_info.content_size = bounding_box_size;

                let new_fragment = old_fragment.transform(
                    bounding_box_size,
                    SpecificFragmentInfo::ScannedText(new_text_fragment_info));
                out_fragments.push(new_fragment)
            }
        }

        last_whitespace
    }
}

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
    let fontgroup = font_context.layout_font_group_for_style(font_style);
    // FIXME(https://github.com/rust-lang/rust/issues/23338)
    let font = fontgroup.fonts[0].borrow();
    font.metrics.clone()
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

fn split_first_fragment_at_newline_if_necessary(fragments: &mut LinkedList<Fragment>) {
    if fragments.len() < 1 {
        return
    }

    let new_fragment = {
        let mut first_fragment = fragments.front_mut().unwrap();
        let string_before;
        let insertion_point_before;
        {
            let unscanned_text_fragment_info = match first_fragment.specific {
                SpecificFragmentInfo::UnscannedText(ref mut unscanned_text_fragment_info) => {
                    unscanned_text_fragment_info
                }
                _ => return,
            };

            if first_fragment.style.get_inheritedtext().white_space != white_space::T::pre {
                return
            }

            let position = match unscanned_text_fragment_info.text.find('\n') {
                Some(position) if position < unscanned_text_fragment_info.text.len() - 1 => {
                    position
                }
                Some(_) | None => return,
            };

            string_before =
                unscanned_text_fragment_info.text[..(position + 1)].to_owned();
            insertion_point_before = unscanned_text_fragment_info.insertion_point;
            unscanned_text_fragment_info.text =
                unscanned_text_fragment_info.text[(position + 1)..].to_owned().into_boxed_str();
        }
        first_fragment.transform(first_fragment.border_box.size,
                                 SpecificFragmentInfo::UnscannedText(
                                     UnscannedTextFragmentInfo::new(string_before,
                                                                    insertion_point_before)))
    };

    fragments.push_front(new_fragment);
}

/// Information about a text run that we're about to create. This is used in `scan_for_runs`.
struct RunInfo {
    /// The text that will go in this text run.
    text: String,
    /// The insertion point in this text run, if applicable.
    insertion_point: Option<CharIndex>,
    /// The index of the applicable font in the font group.
    font_index: usize,
    /// A cached copy of the number of Unicode characters in the text run.
    character_length: usize,
    /// The bidirection embedding level of this text run.
    bidi_level: u8,
    /// The Unicode script property of this text run.
    script: Script,
}

impl RunInfo {
    fn new() -> RunInfo {
        RunInfo {
            text: String::new(),
            insertion_point: None,
            font_index: 0,
            character_length: 0,
            bidi_level: 0,
            script: Script::Common,
        }
    }
}

/// A mapping from a portion of an unscanned text fragment to the text run we're going to create
/// for it.
#[derive(Copy, Clone, Debug)]
struct RunMapping {
    /// The range of characters within the text fragment.
    char_range: Range<CharIndex>,
    /// The range of byte indices within the text fragment.
    byte_range: Range<usize>,
    /// The index of the unscanned text fragment that this mapping corresponds to.
    old_fragment_index: usize,
    /// The index of the text run we're going to create.
    text_run_index: usize,
}

impl RunMapping {
    /// Given the current set of text runs, creates a run mapping for the next fragment.
    /// `run_info_list` describes the set of runs we've seen already, and `current_run_info`
    /// describes the run we just finished processing.
    fn new(run_info_list: &[RunInfo], current_run_info: &RunInfo, fragment_index: usize)
           -> RunMapping {
        RunMapping {
            char_range: Range::new(CharIndex(current_run_info.character_length as isize),
                                   CharIndex(0)),
            byte_range: Range::new(0, 0),
            old_fragment_index: fragment_index,
            text_run_index: run_info_list.len(),
        }
    }

    /// Flushes this run mapping to the list. `run_info` describes the text run that we're
    /// currently working on. `text` refers to the text of this fragment.
    fn flush(mut self,
             mappings: &mut Vec<RunMapping>,
             run_info: &mut RunInfo,
             text: &str,
             insertion_point: Option<CharIndex>,
             compression: CompressionMode,
             text_transform: text_transform::T,
             last_whitespace: &mut bool,
             start_position: &mut usize,
             end_position: usize) {
        let old_byte_length = run_info.text.len();
        *last_whitespace = util::transform_text(&text[(*start_position)..end_position],
                                                compression,
                                                *last_whitespace,
                                                &mut run_info.text);

        // Account for `text-transform`. (Confusingly, this is not handled in "text
        // transformation" above, but we follow Gecko in the naming.)
        let is_first_run = *start_position == 0;
        let character_count = apply_style_transform_if_necessary(&mut run_info.text,
                                                                 old_byte_length,
                                                                 text_transform,
                                                                 *last_whitespace,
                                                                 is_first_run);

        // Record the position of the insertion point if necessary.
        if let Some(insertion_point) = insertion_point {
            run_info.insertion_point =
                Some(CharIndex(run_info.character_length as isize + insertion_point.0))
        }

        run_info.character_length = run_info.character_length + character_count;
        *start_position = end_position;

        // Don't flush empty mappings.
        if character_count == 0 {
            return
        }

        let new_byte_length = run_info.text.len();
        self.byte_range = Range::new(old_byte_length, new_byte_length - old_byte_length);
        self.char_range.extend_by(CharIndex(character_count as isize));
        mappings.push(self)
    }
}


/// Accounts for `text-transform`.
///
/// FIXME(#4311, pcwalton): Title-case mapping can change length of the string;
/// case mapping should be language-specific; `full-width`;
/// use graphemes instead of characters.
fn apply_style_transform_if_necessary(string: &mut String,
                                      first_character_position: usize,
                                      text_transform: text_transform::T,
                                      last_whitespace: bool,
                                      is_first_run: bool)
                                      -> usize {
    match text_transform {
        text_transform::T::none => string[first_character_position..].chars().count(),
        text_transform::T::uppercase => {
            let original = string[first_character_position..].to_owned();
            string.truncate(first_character_position);
            let mut count = 0;
            for ch in original.chars().flat_map(|ch| ch.to_uppercase()) {
                string.push(ch);
                count += 1;
            }
            count
        }
        text_transform::T::lowercase => {
            let original = string[first_character_position..].to_owned();
            string.truncate(first_character_position);
            let mut count = 0;
            for ch in original.chars().flat_map(|ch| ch.to_lowercase()) {
                string.push(ch);
                count += 1;
            }
            count
        }
        text_transform::T::capitalize => {
            let original = string[first_character_position..].to_owned();
            string.truncate(first_character_position);

            let mut capitalize_next_letter = is_first_run || last_whitespace;
            let mut count = 0;
            for character in original.chars() {
                count += 1;

                // FIXME(#4311, pcwalton): Should be the CSS/Unicode notion of a *typographic
                // letter unit*, not an *alphabetic* character:
                //
                //    http://dev.w3.org/csswg/css-text/#typographic-letter-unit
                if capitalize_next_letter && character.is_alphabetic() {
                    string.push(character.to_uppercase().next().unwrap());
                    capitalize_next_letter = false;
                    continue
                }

                string.push(character);

                // FIXME(#4311, pcwalton): Try UAX29 instead of just whitespace.
                if character.is_whitespace() {
                    capitalize_next_letter = true
                }
            }

            count
        }
    }
}

#[derive(Clone)]
struct ScannedTextRun {
    run: Arc<TextRun>,
    insertion_point: Option<CharIndex>,
}

/// Can a character with script `b` continue a text run with script `a`?
fn is_compatible(a: Script, b: Script) -> bool {
    a == b || !is_specific(a) || !is_specific(b)
}

/// Returns true if the script is not invalid or inherited.
fn is_specific(script: Script) -> bool {
    script != Script::Common && script != Script::Inherited
}
