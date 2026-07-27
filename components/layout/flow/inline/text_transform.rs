/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

//! # Logic for text transform in inline formatting contexts
//!
//! Inline formatting contexts do a variety of text transformations on their text content
//! including white space collapsing, application of the `text-transform` CSS property,
//! and application of the `-webkit-text-security` property. This module contains code to
//! handle this as well as code to map from offsets in the original DOM node to the final
//! IFC text and vice-versa.

use std::cmp::Ordering;
use std::str::Chars;

use icu_segmenter::WordSegmenter;
use itertools::Either;
use malloc_size_of_derive::MallocSizeOf;
use style::computed_values::_webkit_text_security::T as WebKitTextSecurity;
use style::computed_values::white_space_collapse::T as WhiteSpaceCollapse;
use style::properties::ComputedValues;
use style::values::specified::text::{TextTransform, TextTransformCase};

use crate::flow::inline::construct::InlineFormattingContextBuilder;

/// A single iteration in a pipeline of character iterators, that handle things
/// like whitespace collapse and `text-transform` processing for text in an
/// [`InlineFormattingContext`]. Each iteration can consume multiple characters
/// and produce an optional character. Consumption of characters greater than
/// the characters stored in [`CharacterTransformIteration`] indicate that those
/// characters have been collapsed.
pub struct CharacterTransformIteration {
    /// The number of characters consumed when producing the optional character
    /// in this iteration. This can indicate that a certain number of characters
    /// have been collapsed in the input stream.
    pub consumed_character_count: usize,
    /// If this iteration produced a character, it is stored in this field. If no
    /// character is stored here, the iteration collapsed the
    /// [`Self::consumed_character_count`] characters.
    pub character: Option<char>,
}

pub(crate) struct TextTransformationIterator<'a>(
    Box<dyn Iterator<Item = CharacterTransformIteration> + 'a>,
);

impl<'a> TextTransformationIterator<'a> {
    pub(crate) fn new(
        text: &'a str,
        style: &ComputedValues,
        trim_leading_white_space: bool,
        on_word_boundary: bool,
    ) -> Self {
        let white_space_collapse = style.clone_white_space_collapse();
        let mut iterator: Box<dyn Iterator<Item = CharacterTransformIteration>> = Box::new(
            WhitespaceCollapse::new(text.chars(), white_space_collapse, trim_leading_white_space),
        );

        // TODO: Not all text transforms are about case, this logic should stop ignoring
        // TextTransform::FULL_WIDTH and TextTransform::FULL_SIZE_KANA.
        let text_transform = style.clone_text_transform();
        match text_transform.case() {
            TextTransformCase::None => {},
            TextTransformCase::Lowercase => {
                iterator = Box::new(simple_case_transform_iterator(iterator, char::to_lowercase))
            },
            TextTransformCase::Uppercase => {
                iterator = Box::new(simple_case_transform_iterator(iterator, char::to_uppercase))
            },
            TextTransformCase::Capitalize => {
                iterator = Box::new(capitalization_iterator(iterator, on_word_boundary))
            },
            // TODO: implement `math-auto` and enable it in Stylo
        }
        if text_transform.intersects(TextTransform::FULL_WIDTH) {
            // TODO: implement `full-width`
            // iterator = Box::new(full_width_iterator(iterator));
        }
        if text_transform.intersects(TextTransform::FULL_SIZE_KANA) {
            // TODO: implement `full-size-kana`
            // iterator = Box::new(full_size_kana_iterator(iterator));
        }

        let text_security = style.clone__webkit_text_security();
        if text_security != WebKitTextSecurity::None {
            iterator = Box::new(TextSecurityTransform::new(iterator, text_security));
        }
        Self(iterator)
    }
}

impl Iterator for TextTransformationIterator<'_> {
    type Item = CharacterTransformIteration;
    fn next(&mut self) -> Option<Self::Item> {
        self.0.next()
    }
}

impl CharacterTransformIteration {
    fn identity(character: char) -> Self {
        Self {
            consumed_character_count: 1,
            character: Some(character),
        }
    }

    fn with_character(self, character: char) -> Self {
        Self {
            character: Some(character),
            ..self
        }
    }

    /// `character_iter` must be non-empty
    fn from_char_iter(
        mut consumed_character_count: usize,
        character_iter: impl Iterator<Item = char>,
    ) -> impl Iterator<Item = Self> {
        character_iter.map(move |transformed_character| {
            CharacterTransformIteration {
                // Iterations after the first use zero:
                consumed_character_count: std::mem::take(&mut consumed_character_count),
                character: Some(transformed_character),
            }
        })
    }
}

pub struct WhitespaceCollapse<'a> {
    input_iterator: Chars<'a>,
    white_space_collapse: WhiteSpaceCollapse,

    /// Whether or not we are in the process of collapse leading white space. This is true
    /// when the last character handled in our owning [`super::InlineFormattingContext`]
    /// was collapsible white space and we have not seen any non-whitespace characters
    /// during processing of this iterator's input.
    trimming_leading_white_space: bool,

    /// Whether or not the last character produced was newline. There is special behavior
    /// we do after each newline.
    following_newline: bool,

    /// When whitespace collapses before a non-whitespace character, the iterator returns
    /// the collapsed whitespace and in the next iteration the non-whitespace character
    /// must be returned. This value caches it until the next iteration.
    character_pending_to_return: Option<char>,
}

impl<'a> WhitespaceCollapse<'a> {
    pub fn new(
        input_iterator: Chars<'a>,
        white_space_collapse: WhiteSpaceCollapse,
        should_trim_leading_white_space: bool,
    ) -> Self {
        Self {
            input_iterator,
            white_space_collapse,
            following_newline: false,
            trimming_leading_white_space: should_trim_leading_white_space,
            character_pending_to_return: None,
        }
    }

    /// In some cases, white space is replaced by a single character (when not
    /// following a newline and when leading whitespace is not being trimmed). In all
    /// other cases, the white space is simply removed. This method handles that.
    fn character_for_collapsed_whitespace(&self) -> Option<char> {
        if !self.following_newline && !self.trimming_leading_white_space {
            Some(' ')
        } else {
            None
        }
    }

    fn iteration_for_collected_white_space(
        &self,
        collected_whitespace: usize,
    ) -> Option<CharacterTransformIteration> {
        (collected_whitespace != 0).then(|| CharacterTransformIteration {
            consumed_character_count: collected_whitespace,
            character: self.character_for_collapsed_whitespace(),
        })
    }
}

impl Iterator for WhitespaceCollapse<'_> {
    type Item = CharacterTransformIteration;

    fn next(&mut self) -> Option<Self::Item> {
        // Point 4.1.1 first bullet:
        // > If white-space is set to normal, nowrap, or pre-line, whitespace
        // > characters are considered collapsible
        // If whitespace is not considered collapsible, it is preserved entirely, which
        // means that we can simply return the input string exactly.
        if self.white_space_collapse == WhiteSpaceCollapse::Preserve ||
            self.white_space_collapse == WhiteSpaceCollapse::BreakSpaces
        {
            // From <https://drafts.csswg.org/css-text-3/#white-space-processing>:
            // > Carriage returns (U+000D) are treated identically to spaces (U+0020) in all respects.
            //
            // In the non-preserved case these are converted to space below.
            return match self.input_iterator.next() {
                Some('\r') => Some(CharacterTransformIteration::identity(' ')),
                next => next.map(CharacterTransformIteration::identity),
            };
        }

        if let Some(character) = self.character_pending_to_return.take() {
            // Once we produce a non-whitespace character, we are no longer trimming leading whitespace.
            self.trimming_leading_white_space = false;
            self.following_newline = false;
            return Some(CharacterTransformIteration::identity(character));
        }

        // When we enter a collapsible white space region, we may need to wait to produce
        // a single white space character as soon as we encounter a non-white space
        // character. When that happens we queue up the non-white space character for the
        // next iterator call.
        let mut collected_whitespace = 0;

        while let Some(character) = self.input_iterator.next() {
            // Don't push non-newline whitespace immediately. Instead wait to push it until we
            // know that it isn't followed by a newline. See `push_pending_whitespace_if_needed`
            // above.
            if InlineFormattingContextBuilder::is_document_white_space(character) &&
                character != '\n'
            {
                collected_whitespace += 1;
                continue;
            }

            // Point 4.1.1:
            // > 2. Collapsible segment breaks are transformed for rendering according to the
            // >    segment break transformation rules.
            if character == '\n' {
                // From <https://drafts.csswg.org/css-text-3/#line-break-transform>
                // (4.1.3 -- the segment break transformation rules):
                //
                // > When white-space is pre, pre-wrap, or pre-line, segment breaks are not
                // > collapsible and are instead transformed into a preserved line feed"
                //
                // > 1. First, any collapsible segment break immediately following another
                // >    collapsible segment break is removed.
                // > 2. Then any remaining segment break is either transformed into a space (U+0020)
                // >    or removed depending on the context before and after the break.
                let maybe_character = if self.white_space_collapse != WhiteSpaceCollapse::Collapse {
                    Some('\n')
                } else {
                    self.character_for_collapsed_whitespace()
                };

                self.following_newline = true;
                return Some(CharacterTransformIteration {
                    consumed_character_count: collected_whitespace + 1,
                    character: maybe_character,
                });
            }

            // Point 4.1.1:
            // > 2. Any sequence of collapsible spaces and tabs immediately preceding or
            // >    following a segment break is removed.
            // > 3. Every collapsible tab is converted to a collapsible space (U+0020).
            // > 4. Any collapsible space immediately following another collapsible space—even
            // >    one outside the boundary of the inline containing that space, provided both
            // >    spaces are within the same inline formatting context—is collapsed to have zero
            // >    advance width.
            if let Some(iteration) = self.iteration_for_collected_white_space(collected_whitespace)
            {
                self.character_pending_to_return = Some(character);
                return Some(iteration);
            }

            // Once we produce a non-whitespace character, we are no longer trimming leading whitespace.
            self.trimming_leading_white_space = false;
            self.following_newline = false;
            return Some(CharacterTransformIteration::identity(character));
        }

        self.iteration_for_collected_white_space(collected_whitespace)
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        self.input_iterator.size_hint()
    }

    fn count(self) -> usize {
        self.input_iterator.count()
    }
}

fn simple_case_transform_iterator<'a, CharacterIterator>(
    input_iterator: Box<dyn Iterator<Item = CharacterTransformIteration> + 'a>,
    mapping: impl Fn(char) -> CharacterIterator,
) -> impl Iterator<Item = CharacterTransformIteration>
where
    CharacterIterator: Iterator<Item = char>,
{
    input_iterator.flat_map(move |text_step| {
        if let Some(character) = text_step.character {
            Either::Right(CharacterTransformIteration::from_char_iter(
                text_step.consumed_character_count,
                mapping(character),
            ))
        } else {
            Either::Left(std::iter::once(text_step))
        }
    })
}

/// Given a string and whether the start of the string represents a word boundary, create a copy of
/// the string with letters after word boundaries capitalized.
pub(crate) fn capitalization_iterator<'a>(
    input_iterator: Box<dyn Iterator<Item = CharacterTransformIteration> + 'a>,
    allow_word_at_start: bool,
) -> impl Iterator<Item = CharacterTransformIteration> {
    let steps: Vec<_> = input_iterator.collect();
    let string: String = steps.iter().filter_map(|step| step.character).collect();

    let word_segmenter = WordSegmenter::new_auto();
    let mut bounds = word_segmenter.segment_str(&string).peekable();

    let mut output = Vec::with_capacity(steps.len());
    let mut current_byte_index = 0;
    for text_step in steps.into_iter() {
        let Some(character) = text_step.character else {
            output.push(text_step);
            continue;
        };

        let at_word_start = bounds.peek() == Some(&current_byte_index);
        if at_word_start {
            bounds.next();
        }

        if at_word_start && (current_byte_index != 0 || allow_word_at_start) {
            output.extend(CharacterTransformIteration::from_char_iter(
                text_step.consumed_character_count,
                character.to_uppercase(),
            ));
        } else {
            output.push(text_step);
        }

        current_byte_index += character.len_utf8();
    }

    output.into_iter()
}

pub struct TextSecurityTransform<InputIterator> {
    /// The input character iterator.
    iterator: InputIterator,
    /// The `-webkit-text-security` value to use.
    text_security: WebKitTextSecurity,
}

impl<InputIterator> TextSecurityTransform<InputIterator> {
    pub fn new(iterator: InputIterator, text_security: WebKitTextSecurity) -> Self {
        Self {
            iterator,
            text_security,
        }
    }
}

impl<InputIterator> Iterator for TextSecurityTransform<InputIterator>
where
    InputIterator: Iterator<Item = CharacterTransformIteration>,
{
    type Item = CharacterTransformIteration;

    fn next(&mut self) -> Option<Self::Item> {
        // The behavior of `-webkit-text-security` isn't specified, so we have some
        // flexibility in the implementation. We just need to maintain a rough
        // compatibility with other browsers.
        let text_step = self.iterator.next()?;
        let Some(character) = text_step.character else {
            return Some(text_step);
        };

        let mapped_character = match character {
            // This is not ideal, but zero width space is used for some special reasons in
            // `<input>` fields, so these remain untransformed, otherwise they would show up
            // in empty text fields.
            '\u{200B}' => '\u{200B}',
            // Newlines are preserved, so that `<br>` keeps working as expected.
            '\n' => '\n',
            character => match self.text_security {
                WebKitTextSecurity::None => character,
                WebKitTextSecurity::Circle => '○',
                WebKitTextSecurity::Disc => '●',
                WebKitTextSecurity::Square => '■',
            },
        };

        Some(text_step.with_character(mapped_character))
    }
}

#[derive(PartialEq, MallocSizeOf)]
enum OffsetMapEntryType {
    // Represents text that is the same size in the original string as in the final
    // string.
    Identity,
    // Represents text that is shorter in the final string than in the original
    // string.
    Collapse,
    // Represents text that is longer in the final string than in the original
    // string.
    Expand,
}

#[derive(MallocSizeOf)]
struct OffsetMapEntry(OffsetMapEntryType, usize);

#[derive(Default, MallocSizeOf)]
pub struct OffsetMap {
    total_original_size: usize,
    total_final_size: usize,
    entries: Vec<OffsetMapEntry>,
}

impl std::fmt::Debug for OffsetMap {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("OffsetMap")
            .field("total_original_size", &self.total_original_size)
            .field("total_final_size", &self.total_final_size)
            .finish()
    }
}

impl OffsetMap {
    pub fn total_final_size(&self) -> usize {
        self.total_final_size
    }

    fn add_or_ammend_entry(&mut self, entry_type: OffsetMapEntryType, length: usize) {
        if length == 0 {
            return;
        }

        match self.entries.last_mut() {
            Some(OffsetMapEntry(old_entry_type, old_length)) if *old_entry_type == entry_type => {
                *old_length += length
            },
            _ => self.entries.push(OffsetMapEntry(entry_type, length)),
        }
    }

    pub fn collapse(&mut self, length: usize) {
        self.add_or_ammend_entry(OffsetMapEntryType::Collapse, length);
        self.total_original_size += length;
    }

    pub fn expand(&mut self, length: usize) {
        self.add_or_ammend_entry(OffsetMapEntryType::Expand, length);
        self.total_final_size += length;
    }

    pub fn identity(&mut self, length: usize) {
        self.add_or_ammend_entry(OffsetMapEntryType::Identity, length);
        self.total_original_size += length;
        self.total_final_size += length;
    }

    pub fn process_character(&mut self, original_length: usize, new_length: usize) {
        match original_length.cmp(&new_length) {
            Ordering::Less => {
                self.identity(original_length);
                self.expand(new_length - original_length);
            },
            Ordering::Equal => self.identity(original_length),
            Ordering::Greater => {
                self.identity(new_length);
                self.collapse(original_length - new_length);
            },
        }
    }

    pub fn map(&self, target_original_offset: usize) -> usize {
        // Allow mapping one index past the size of the map, as this can be used for
        // selection which can index one character past the end of the last character
        // in the string.
        if target_original_offset > self.total_original_size {
            return self.total_final_size;
        }

        let mut offset_in_original_string = 0;
        let mut offset_in_final_string = 0;
        for entry in self.entries.iter() {
            match entry.0 {
                OffsetMapEntryType::Identity
                    if offset_in_original_string + entry.1 > target_original_offset =>
                {
                    return offset_in_final_string +
                        (target_original_offset - offset_in_original_string);
                },
                OffsetMapEntryType::Identity => {
                    offset_in_final_string += entry.1;
                    offset_in_original_string += entry.1;
                },
                OffsetMapEntryType::Collapse
                    if offset_in_original_string + entry.1 > target_original_offset =>
                {
                    return offset_in_final_string;
                },
                OffsetMapEntryType::Collapse => offset_in_original_string += entry.1,
                OffsetMapEntryType::Expand => offset_in_final_string += entry.1,
            }
        }

        offset_in_final_string
    }
}

#[test]
fn test_offsetmap_basic_expansion() {
    let _original_string = "abcde";
    let final_string = "aabbbccccde";

    let mut offset_map = OffsetMap::default();
    offset_map.process_character(1, 2);
    offset_map.process_character(1, 3);
    offset_map.process_character(1, 4);
    offset_map.process_character(2, 2);

    assert_eq!(offset_map.map(0), 0);
    assert_eq!(offset_map.map(1), 2);
    assert_eq!(offset_map.map(2), 5);
    assert_eq!(offset_map.map(3), 9);
    assert_eq!(offset_map.map(4), 10);

    // Beyond the last index should always map to the index after the last character
    // (for handling selections).
    assert_eq!(offset_map.map(5), 11);
    assert_eq!(offset_map.map(100), 11);

    let map_substring =
        |offset, length| &final_string[offset_map.map(offset)..offset_map.map(offset + length)];
    assert_eq!(map_substring(0, 1), "aa");
    assert_eq!(map_substring(0, 2), "aabbb");
    assert_eq!(map_substring(0, 3), "aabbbcccc");
    assert_eq!(map_substring(0, 4), "aabbbccccd");
    assert_eq!(map_substring(1, 1), "bbb");
}

#[test]
fn test_offsetmap_basic_collapse() {
    let _original_string = "AABBBcDDDDe";
    let final_string = "abcde";

    let mut offset_map = OffsetMap::default();
    offset_map.process_character(2, 1);
    offset_map.process_character(3, 1);
    offset_map.identity(1);
    offset_map.process_character(4, 1);
    offset_map.identity(1);

    assert_eq!(offset_map.map(0), 0);
    // Mapping between characters should map to the index after the final one.
    assert_eq!(offset_map.map(1), 1);
    assert_eq!(offset_map.map(2), 1);
    assert_eq!(offset_map.map(5), 2);
    assert_eq!(offset_map.map(6), 3);
    assert_eq!(offset_map.map(10), 4);

    // Beyond the last index should always map to the index after the last character
    // (for handling selections).
    assert_eq!(offset_map.map(11), 5);
    assert_eq!(offset_map.map(100), 5);

    let map_substring =
        |offset, length| &final_string[offset_map.map(offset)..offset_map.map(offset + length)];
    assert_eq!(map_substring(0, 2), "a");
    assert_eq!(map_substring(0, 3), "ab");
    assert_eq!(map_substring(0, 6), "abc");
    assert_eq!(map_substring(0, 10), "abcd");
}
