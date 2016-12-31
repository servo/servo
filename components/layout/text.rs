/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

//! Text layout.

#![deny(unsafe_code)]

use app_units::Au;
use fragment::{Fragment, REQUIRES_LINE_BREAK_AFTERWARD_IF_WRAPPING_ON_NEWLINES, ScannedTextFlags};
use fragment::{SELECTED, ScannedTextFragmentInfo, SpecificFragmentInfo, UnscannedTextFragmentInfo};
use gfx::font::{DISABLE_KERNING_SHAPING_FLAG, FontMetrics, IGNORE_LIGATURES_SHAPING_FLAG};
use gfx::font::{KEEP_ALL_FLAG, RTL_FLAG, RunMetrics, ShapingFlags, ShapingOptions};
use gfx::font_context::FontContext;
use gfx::text::glyph::ByteIndex;
use gfx::text::text_run::TextRun;
use gfx::text::util::{self, CompressionMode};
use inline::{FIRST_FRAGMENT_OF_ELEMENT, InlineFragments, LAST_FRAGMENT_OF_ELEMENT};
use linked_list::split_off_head;
use ordered_float::NotNaN;
use range::Range;
use std::borrow::ToOwned;
use std::collections::LinkedList;
use std::mem;
use std::sync::Arc;
use style::computed_values::{line_height, text_orientation, text_rendering, text_transform};
use style::computed_values::{word_break, white_space};
use style::logical_geometry::{LogicalSize, WritingMode};
use style::properties::ServoComputedValues;
use style::properties::style_structs;
use unicode_bidi::{is_rtl, process_text};
use unicode_script::{Script, get_script};

/// Returns the concatenated text of a list of unscanned text fragments.
fn text(fragments: &LinkedList<Fragment>) -> String {
    // FIXME: Some of this work is later duplicated in split_first_fragment_at_newline_if_necessary
    // and transform_text.  This code should be refactored so that the all the scanning for
    // newlines is done in a single pass.

    let mut text = String::new();

    for fragment in fragments {
        if let SpecificFragmentInfo::UnscannedText(ref info) = fragment.specific {
            if fragment.white_space().preserve_newlines() {
                text.push_str(&info.text);
            } else {
                text.push_str(&info.text.replace("\n", " "));
            }
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
            let word_break;
            {
                let in_fragment = self.clump.front().unwrap();
                let font_style = in_fragment.style().clone_font();
                let inherited_text_style = in_fragment.style().get_inheritedtext();
                fontgroup = font_context.layout_font_group_for_style(font_style);
                compression = match in_fragment.white_space() {
                    white_space::T::normal |
                    white_space::T::nowrap => CompressionMode::CompressWhitespaceNewline,
                    white_space::T::pre |
                    white_space::T::pre_wrap => CompressionMode::CompressNone,
                    white_space::T::pre_line => CompressionMode::CompressWhitespace,
                };
                text_transform = inherited_text_style.text_transform;
                letter_spacing = inherited_text_style.letter_spacing.0;
                word_spacing = inherited_text_style.word_spacing.0
                               .map(|lop| lop.to_hash_key())
                               .unwrap_or((Au(0), NotNaN::new(0.0).unwrap()));
                text_rendering = inherited_text_style.text_rendering;
                word_break = inherited_text_style.word_break;
            }

            // First, transform/compress text of all the nodes.
            let (mut run_info_list, mut run_info) = (Vec::new(), RunInfo::new());
            let mut insertion_point = None;

            for (fragment_index, in_fragment) in self.clump.iter().enumerate() {
                debug!("  flushing {:?}", in_fragment);
                let mut mapping = RunMapping::new(&run_info_list[..], fragment_index);
                let text;
                let selection;
                match in_fragment.specific {
                    SpecificFragmentInfo::UnscannedText(ref text_fragment_info) => {
                        text = &text_fragment_info.text;
                        selection = text_fragment_info.selection;
                    }
                    _ => panic!("Expected an unscanned text fragment!"),
                };
                insertion_point = match selection {
                    Some(range) if range.is_empty() => {
                        // `range` is the range within the current fragment. To get the range
                        // within the text run, offset it by the length of the preceding fragments.
                        Some(range.begin() + ByteIndex(run_info.text.len() as isize))
                    }
                    _ => None
                };

                let (mut start_position, mut end_position) = (0, 0);
                for (byte_index, character) in text.char_indices() {
                    // Search for the first font in this font group that contains a glyph for this
                    // character.
                    let font_index = fontgroup.fonts.iter().position(|font| {
                        font.borrow().glyph_index(character).is_some()
                    }).unwrap_or(0);

                    // The following code panics one way or another if this condition isn't met.
                    assert!(fontgroup.fonts.len() > 0);

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

                    let selected = match selection {
                        Some(range) => range.contains(ByteIndex(byte_index as isize)),
                        None => false
                    };

                    // Now, if necessary, flush the mapping we were building up.
                    let flush_run = run_info.font_index != font_index ||
                                    run_info.bidi_level != bidi_level ||
                                    !compatible_script;
                    let flush_mapping = flush_run || mapping.selected != selected;

                    if flush_mapping {
                        mapping.flush(&mut mappings,
                                      &mut run_info,
                                      &**text,
                                      compression,
                                      text_transform,
                                      &mut last_whitespace,
                                      &mut start_position,
                                      end_position);
                        if run_info.text.len() > 0 {
                            if flush_run {
                                run_info.flush(&mut run_info_list, &mut insertion_point);
                                run_info = RunInfo::new();
                            }
                            mapping = RunMapping::new(&run_info_list[..],
                                                      fragment_index);
                        }
                        run_info.font_index = font_index;
                        run_info.bidi_level = bidi_level;
                        run_info.script = script;
                        mapping.selected = selected;
                    }

                    // Consume this character.
                    end_position += character.len_utf8();
                    *paragraph_bytes_processed += character.len_utf8();
                }

                // Flush the last mapping we created for this fragment to the list.
                mapping.flush(&mut mappings,
                              &mut run_info,
                              &**text,
                              compression,
                              text_transform,
                              &mut last_whitespace,
                              &mut start_position,
                              end_position);
            }

            // Push the final run info.
            run_info.flush(&mut run_info_list, &mut insertion_point);

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
            if word_break == word_break::T::keep_all {
                flags.insert(KEEP_ALL_FLAG);
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
        let mut prev_fragments_to_meld = Vec::new();

        for (logical_offset, old_fragment) in
                mem::replace(&mut self.clump, LinkedList::new()).into_iter().enumerate() {
            let mut is_first_mapping_of_this_old_fragment = true;
            loop {
                match mappings.peek() {
                    Some(mapping) if mapping.old_fragment_index == logical_offset => {}
                    Some(_) | None => {
                        if is_first_mapping_of_this_old_fragment {
                            // There were no mappings for this unscanned fragment. Transfer its
                            // flags to the previous/next sibling elements instead.
                            if let Some(ref mut last_fragment) = out_fragments.last_mut() {
                                last_fragment.meld_with_next_inline_fragment(&old_fragment);
                            }
                            prev_fragments_to_meld.push(old_fragment);
                        }
                        break;
                    }
                };
                let mapping = mappings.next().unwrap();
                let scanned_run = runs[mapping.text_run_index].clone();

                let mut byte_range = Range::new(ByteIndex(mapping.byte_range.begin() as isize),
                                                ByteIndex(mapping.byte_range.length() as isize));

                let mut flags = ScannedTextFlags::empty();
                let text_size = old_fragment.border_box.size;

                let requires_line_break_afterward_if_wrapping_on_newlines =
                    scanned_run.run.text[mapping.byte_range.begin()..mapping.byte_range.end()]
                    .ends_with('\n');

                if requires_line_break_afterward_if_wrapping_on_newlines {
                    byte_range.extend_by(ByteIndex(-1)); // Trim the '\n'
                    flags.insert(REQUIRES_LINE_BREAK_AFTERWARD_IF_WRAPPING_ON_NEWLINES);
                }

                if mapping.selected {
                    flags.insert(SELECTED);
                }

                let insertion_point = if mapping.contains_insertion_point(scanned_run.insertion_point) {
                    scanned_run.insertion_point
                } else {
                    None
                };

                let mut new_text_fragment_info = box ScannedTextFragmentInfo::new(
                    scanned_run.run,
                    byte_range,
                    text_size,
                    insertion_point,
                    flags);

                let new_metrics = new_text_fragment_info.run.metrics_for_range(&byte_range);
                let writing_mode = old_fragment.style.writing_mode;
                let bounding_box_size = bounding_box_for_run_metrics(&new_metrics, writing_mode);
                new_text_fragment_info.content_size = bounding_box_size;

                let mut new_fragment = old_fragment.transform(
                    bounding_box_size,
                    SpecificFragmentInfo::ScannedText(new_text_fragment_info));

                let is_last_mapping_of_this_old_fragment = match mappings.peek() {
                    Some(mapping) if mapping.old_fragment_index == logical_offset => false,
                    _ => true
                };

                if let Some(ref mut context) = new_fragment.inline_context {
                    for node in &mut context.nodes {
                        if !is_last_mapping_of_this_old_fragment {
                            node.flags.remove(LAST_FRAGMENT_OF_ELEMENT);
                        }
                        if !is_first_mapping_of_this_old_fragment {
                            node.flags.remove(FIRST_FRAGMENT_OF_ELEMENT);
                        }
                    }
                }

                for prev_fragment in prev_fragments_to_meld.drain(..) {
                    new_fragment.meld_with_prev_inline_fragment(&prev_fragment);
                }

                is_first_mapping_of_this_old_fragment = false;
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

/// Returns the metrics of the font represented by the given `style_structs::Font`, respectively.
///
/// `#[inline]` because often the caller only needs a few fields from the font metrics.
#[inline]
pub fn font_metrics_for_style(font_context: &mut FontContext, font_style: Arc<style_structs::Font>)
                              -> FontMetrics {
    let fontgroup = font_context.layout_font_group_for_style(font_style);
    // FIXME(https://github.com/rust-lang/rust/issues/23338)
    let font = fontgroup.fonts[0].borrow();
    font.metrics.clone()
}

/// Returns the line block-size needed by the given computed style and font size.
pub fn line_height_from_style(style: &ServoComputedValues, metrics: &FontMetrics) -> Au {
    let font_size = style.get_font().font_size;
    match style.get_inheritedtext().line_height {
        line_height::T::Normal => metrics.line_gap,
        line_height::T::Number(l) => font_size.scale_by(l),
        line_height::T::Length(l) => l
    }
}

fn split_first_fragment_at_newline_if_necessary(fragments: &mut LinkedList<Fragment>) {
    if fragments.is_empty() {
        return
    }

    let new_fragment = {
        let mut first_fragment = fragments.front_mut().unwrap();
        let string_before;
        let selection_before;
        {
            if !first_fragment.white_space().preserve_newlines() {
                return;
            }

            let unscanned_text_fragment_info = match first_fragment.specific {
                SpecificFragmentInfo::UnscannedText(ref mut unscanned_text_fragment_info) => {
                    unscanned_text_fragment_info
                }
                _ => return,
            };

            let position = match unscanned_text_fragment_info.text.find('\n') {
                Some(position) if position < unscanned_text_fragment_info.text.len() - 1 => {
                    position
                }
                Some(_) | None => return,
            };

            string_before =
                unscanned_text_fragment_info.text[..(position + 1)].to_owned();
            unscanned_text_fragment_info.text =
                unscanned_text_fragment_info.text[(position + 1)..].to_owned().into_boxed_str();
            let offset = ByteIndex(string_before.len() as isize);
            match unscanned_text_fragment_info.selection {
                Some(ref mut selection) if selection.begin() >= offset => {
                    // Selection is entirely in the second fragment.
                    selection_before = None;
                    selection.shift_by(-offset);
                }
                Some(ref mut selection) if selection.end() > offset => {
                    // Selection is split across two fragments.
                    selection_before = Some(Range::new(selection.begin(), offset));
                    *selection = Range::new(ByteIndex(0), selection.end() - offset);
                }
                _ => {
                    // Selection is entirely in the first fragment.
                    selection_before = unscanned_text_fragment_info.selection;
                    unscanned_text_fragment_info.selection = None;
                }
            };
        }
        first_fragment.transform(first_fragment.border_box.size,
                                 SpecificFragmentInfo::UnscannedText(
                                     box UnscannedTextFragmentInfo::new(string_before,
                                                                        selection_before)))
    };

    fragments.push_front(new_fragment);
}

/// Information about a text run that we're about to create. This is used in `scan_for_runs`.
struct RunInfo {
    /// The text that will go in this text run.
    text: String,
    /// The insertion point in this text run, if applicable.
    insertion_point: Option<ByteIndex>,
    /// The index of the applicable font in the font group.
    font_index: usize,
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
            bidi_level: 0,
            script: Script::Common,
        }
    }

    /// Finish processing this RunInfo and add it to the "done" list.
    ///
    /// * `insertion_point`: The position of the insertion point, in characters relative to the start
    ///   of this text run.
    fn flush(mut self,
             list: &mut Vec<RunInfo>,
             insertion_point: &mut Option<ByteIndex>) {
        if let Some(idx) = *insertion_point {
            let char_len = ByteIndex(self.text.len() as isize);
            if idx <= char_len {
                // The insertion point is in this text run.
                self.insertion_point = insertion_point.take()
            } else {
                // Continue looking for the insertion point in the next text run.
                *insertion_point = Some(idx - char_len)
            }
        }
        list.push(self);
    }
}

/// A mapping from a portion of an unscanned text fragment to the text run we're going to create
/// for it.
#[derive(Copy, Clone, Debug)]
struct RunMapping {
    /// The range of byte indices within the text fragment.
    byte_range: Range<usize>,
    /// The index of the unscanned text fragment that this mapping corresponds to.
    old_fragment_index: usize,
    /// The index of the text run we're going to create.
    text_run_index: usize,
    /// Is the text in this fragment selected?
    selected: bool,
}

impl RunMapping {
    /// Given the current set of text runs, creates a run mapping for the next fragment.
    /// `run_info_list` describes the set of runs we've seen already.
    fn new(run_info_list: &[RunInfo], fragment_index: usize)
           -> RunMapping {
        RunMapping {
            byte_range: Range::new(0, 0),
            old_fragment_index: fragment_index,
            text_run_index: run_info_list.len(),
            selected: false,
        }
    }

    /// Flushes this run mapping to the list. `run_info` describes the text run that we're
    /// currently working on. `text` refers to the text of this fragment.
    fn flush(mut self,
             mappings: &mut Vec<RunMapping>,
             run_info: &mut RunInfo,
             text: &str,
             compression: CompressionMode,
             text_transform: text_transform::T,
             last_whitespace: &mut bool,
             start_position: &mut usize,
             end_position: usize) {
        let was_empty = *start_position == end_position;
        let old_byte_length = run_info.text.len();
        *last_whitespace = util::transform_text(&text[(*start_position)..end_position],
                                                compression,
                                                *last_whitespace,
                                                &mut run_info.text);

        // Account for `text-transform`. (Confusingly, this is not handled in "text
        // transformation" above, but we follow Gecko in the naming.)
        let is_first_run = *start_position == 0;
        apply_style_transform_if_necessary(&mut run_info.text, old_byte_length, text_transform,
                                           *last_whitespace, is_first_run);
        *start_position = end_position;

        let new_byte_length = run_info.text.len();
        let is_empty = new_byte_length == old_byte_length;

        // Don't save mappings that contain only discarded characters.
        // (But keep ones that contained no characters to begin with, since they might have been
        // generated by an empty flow to draw its borders/padding/insertion point.)
        if is_empty && !was_empty {
            return;
        }

        self.byte_range = Range::new(old_byte_length, new_byte_length - old_byte_length);
        mappings.push(self)
    }

    /// Is the insertion point for this text run within this mapping?
    ///
    /// NOTE: We treat the range as inclusive at both ends, since the insertion point can lie
    /// before the first character *or* after the last character, and should be drawn even if the
    /// text is empty.
    fn contains_insertion_point(&self, insertion_point: Option<ByteIndex>) -> bool {
        match insertion_point.map(ByteIndex::to_usize) {
            None => false,
            Some(idx) => self.byte_range.begin() <= idx && idx <= self.byte_range.end()
        }
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
                                      is_first_run: bool) {
    match text_transform {
        text_transform::T::none => {}
        text_transform::T::uppercase => {
            let original = string[first_character_position..].to_owned();
            string.truncate(first_character_position);
            for ch in original.chars().flat_map(|ch| ch.to_uppercase()) {
                string.push(ch);
            }
        }
        text_transform::T::lowercase => {
            let original = string[first_character_position..].to_owned();
            string.truncate(first_character_position);
            for ch in original.chars().flat_map(|ch| ch.to_lowercase()) {
                string.push(ch);
            }
        }
        text_transform::T::capitalize => {
            let original = string[first_character_position..].to_owned();
            string.truncate(first_character_position);

            let mut capitalize_next_letter = is_first_run || last_whitespace;
            for character in original.chars() {
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
        }
    }
}

#[derive(Clone)]
struct ScannedTextRun {
    run: Arc<TextRun>,
    insertion_point: Option<ByteIndex>,
}

/// Can a character with script `b` continue a text run with script `a`?
fn is_compatible(a: Script, b: Script) -> bool {
    a == b || !is_specific(a) || !is_specific(b)
}

/// Returns true if the script is not invalid or inherited.
fn is_specific(script: Script) -> bool {
    script != Script::Common && script != Script::Inherited
}
