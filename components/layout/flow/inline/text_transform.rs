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

use icu_segmenter::WordSegmenter;
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
#[derive(Clone, Copy)]
pub enum CharacterTransformIteration {
    /// A character mapped from exactly one character in a DOM text node
    OneToOne(char),
    WhitespaceCharsCollapsedToOneSpace(usize),
    WhitespaceCharsCollapsedToOneNewline(usize),
    WhitespaceCharsCollapsedToNothing(usize),
    ToLowercase(char),
    ToUppercase(char),
    ToTitlecase(char),
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
        let text_security = style.clone__webkit_text_security();
        let chars = text
            .chars()
            .map(move |character| text_security_map_character(text_security, character));
        let white_space_collapse = style.clone_white_space_collapse();
        let iterator =
            WhitespaceCollapse::new(chars, white_space_collapse, trim_leading_white_space);

        // TODO: Not all text transforms are about case, this logic should stop ignoring
        // TextTransform::FULL_WIDTH and TextTransform::FULL_SIZE_KANA.
        let text_transform = style.clone_text_transform();
        let iterator = match text_transform.case() {
            TextTransformCase::None => {
                Box::new(iterator) as Box<dyn Iterator<Item = CharacterTransformIteration>>
            },
            TextTransformCase::Lowercase => Box::new(simple_case_transform_iterator(
                iterator,
                CharacterTransformIteration::ToLowercase,
            )),
            TextTransformCase::Uppercase => Box::new(simple_case_transform_iterator(
                iterator,
                CharacterTransformIteration::ToUppercase,
            )),
            TextTransformCase::Capitalize => Box::new(capitalization_iterator(
                text.len(),
                iterator,
                on_word_boundary,
            )),
            // TODO: implement `math-auto` and enable it in Stylo
        };
        if text_transform.intersects(TextTransform::FULL_WIDTH) {
            // TODO: implement `full-width`
            // iterator = Box::new(full_width_iterator(iterator));
        }
        if text_transform.intersects(TextTransform::FULL_SIZE_KANA) {
            // TODO: implement `full-size-kana`
            // iterator = Box::new(full_size_kana_iterator(iterator));
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
    pub(crate) fn consumed_character_count(&self) -> usize {
        match *self {
            CharacterTransformIteration::OneToOne(_) => 1,
            CharacterTransformIteration::ToLowercase(_) |
            CharacterTransformIteration::ToUppercase(_) |
            CharacterTransformIteration::ToTitlecase(_) => 1,
            CharacterTransformIteration::WhitespaceCharsCollapsedToOneSpace(count) |
            CharacterTransformIteration::WhitespaceCharsCollapsedToOneNewline(count) |
            CharacterTransformIteration::WhitespaceCharsCollapsedToNothing(count) => count,
        }
    }

    pub(crate) fn each_char(&self, mut each: impl FnMut(char)) {
        match *self {
            CharacterTransformIteration::OneToOne(c) => each(c),
            CharacterTransformIteration::WhitespaceCharsCollapsedToOneSpace(_) => each(' '),
            CharacterTransformIteration::WhitespaceCharsCollapsedToOneNewline(_) => each('\n'),
            CharacterTransformIteration::WhitespaceCharsCollapsedToNothing(_) => {},
            CharacterTransformIteration::ToLowercase(c) => c.to_lowercase().for_each(each),
            CharacterTransformIteration::ToUppercase(c) => c.to_uppercase().for_each(each),
            CharacterTransformIteration::ToTitlecase(c) => to_titlecase(c).for_each(each),
        }
    }

    pub fn push_chars_to(&self, string: &mut String) {
        self.each_char(|character| string.push(character));
    }
}

// TODO: replace this function with `character.to_titlecase()` when available:
// https://github.com/rust-lang/rust/issues/153892
// because of:
// https://doc.rust-lang.org/stable/std/primitive.char.html#difference-from-uppercase
fn to_titlecase(character: char) -> impl ExactSizeIterator<Item = char> {
    character.to_uppercase()
}

pub struct WhitespaceCollapse<InputIterator> {
    input_iterator: InputIterator,
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

impl<InputIterator: Iterator<Item = char>> WhitespaceCollapse<InputIterator> {
    pub fn new(
        input_iterator: InputIterator,
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
    fn iteration_for_collapsed_whitespace(
        &self,
        collapsed_whitespace: usize,
    ) -> CharacterTransformIteration {
        if !self.following_newline && !self.trimming_leading_white_space {
            CharacterTransformIteration::WhitespaceCharsCollapsedToOneSpace(collapsed_whitespace)
        } else {
            CharacterTransformIteration::WhitespaceCharsCollapsedToNothing(collapsed_whitespace)
        }
    }

    fn iteration_for_collected_white_space(
        &self,
        collected_whitespace: usize,
    ) -> Option<CharacterTransformIteration> {
        (collected_whitespace != 0)
            .then(|| self.iteration_for_collapsed_whitespace(collected_whitespace))
    }
}

impl<InputIterator: Iterator<Item = char>> Iterator for WhitespaceCollapse<InputIterator> {
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
                Some('\r') => Some(CharacterTransformIteration::OneToOne(' ')),
                next => next.map(CharacterTransformIteration::OneToOne),
            };
        }

        if let Some(character) = self.character_pending_to_return.take() {
            // Once we produce a non-whitespace character, we are no longer trimming leading whitespace.
            self.trimming_leading_white_space = false;
            self.following_newline = false;
            return Some(CharacterTransformIteration::OneToOne(character));
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
                let iteration = if self.white_space_collapse != WhiteSpaceCollapse::Collapse {
                    CharacterTransformIteration::WhitespaceCharsCollapsedToOneNewline(
                        collected_whitespace + 1,
                    )
                } else {
                    self.iteration_for_collapsed_whitespace(collected_whitespace + 1)
                };

                self.following_newline = true;
                return Some(iteration);
            }

            // Non-whitespace character

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
            return Some(CharacterTransformIteration::OneToOne(character));
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

fn simple_case_transform_iterator(
    input_iterator: impl Iterator<Item = CharacterTransformIteration>,
    mapping: impl Fn(char) -> CharacterTransformIteration,
) -> impl Iterator<Item = CharacterTransformIteration> {
    input_iterator.map(move |iteration| {
        if let CharacterTransformIteration::OneToOne(character) = iteration {
            mapping(character)
        } else {
            iteration
        }
    })
}

/// Given a string and whether the start of the string represents a word boundary, create a copy of
/// the string with letters after word boundaries capitalized.
pub(crate) fn capitalization_iterator(
    size_hint: usize,
    input_iterator: impl Iterator<Item = CharacterTransformIteration>,
    allow_word_at_start: bool,
) -> impl Iterator<Item = CharacterTransformIteration> {
    let iterations: Vec<_> = input_iterator.collect();
    let mut string = String::with_capacity(size_hint);
    for iteration in &iterations {
        iteration.push_chars_to(&mut string);
    }

    let word_segmenter = WordSegmenter::new_auto();
    let mut bounds = word_segmenter.segment_str(&string).peekable();

    let mut output = Vec::with_capacity(iterations.len());
    let mut current_byte_index = 0;
    for iteration in iterations.into_iter() {
        let character = match iteration {
            CharacterTransformIteration::OneToOne(c) => c,
            CharacterTransformIteration::WhitespaceCharsCollapsedToOneSpace(_) => ' ',
            CharacterTransformIteration::WhitespaceCharsCollapsedToOneNewline(_) => '\n',
            CharacterTransformIteration::WhitespaceCharsCollapsedToNothing(_) => {
                output.push(iteration);
                continue;
            },
            CharacterTransformIteration::ToLowercase(_) |
            CharacterTransformIteration::ToUppercase(_) |
            CharacterTransformIteration::ToTitlecase(_) => unreachable!(),
        };

        let at_word_start = bounds.peek() == Some(&current_byte_index);
        if at_word_start {
            bounds.next();
        }

        if at_word_start &&
            let CharacterTransformIteration::OneToOne(_) = iteration &&
            (current_byte_index != 0 || allow_word_at_start)
        {
            output.push(CharacterTransformIteration::ToTitlecase(character));
        } else {
            output.push(iteration);
        }

        current_byte_index += character.len_utf8();
    }

    output.into_iter()
}

// The behavior of `-webkit-text-security` isn't specified, so we have some
// flexibility in the implementation. We just need to maintain a rough
// compatibility with other browsers.
fn text_security_map_character(mode: WebKitTextSecurity, character: char) -> char {
    if let WebKitTextSecurity::None = mode {
        character
    } else {
        // TODO: when MSRV is 1.95+
        // std::hint::cold_path();
        match character {
            // This is not ideal, but zero width space is used for some special reasons in
            // `<input>` fields, so these remain untransformed, otherwise they would show up
            // in empty text fields.
            '\u{200B}' => '\u{200B}',
            // Newlines are preserved, so that `<br>` keeps working as expected.
            '\n' => '\n',
            _ => match mode {
                WebKitTextSecurity::None => character, // unreachable
                WebKitTextSecurity::Circle => '○',
                WebKitTextSecurity::Disc => '●',
                WebKitTextSecurity::Square => '■',
            },
        }
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

    fn collapse(&mut self, length: usize) {
        self.add_or_ammend_entry(OffsetMapEntryType::Collapse, length);
        self.total_original_size += length;
    }

    fn expand(&mut self, length: usize) {
        self.add_or_ammend_entry(OffsetMapEntryType::Expand, length);
        self.total_final_size += length;
    }

    fn identity(&mut self, length: usize) {
        self.add_or_ammend_entry(OffsetMapEntryType::Identity, length);
        self.total_original_size += length;
        self.total_final_size += length;
    }

    fn case_map(&mut self, new_length: usize) {
        self.identity(1);
        if new_length > 1 {
            self.expand(new_length - 1);
        }
    }

    pub(crate) fn push_synthetic_control_characters(&mut self, count: usize) {
        self.expand(count);
    }

    pub(crate) fn push_iteration(&mut self, iteration: &CharacterTransformIteration) {
        match *iteration {
            CharacterTransformIteration::OneToOne(_) => self.identity(1),
            CharacterTransformIteration::WhitespaceCharsCollapsedToOneSpace(count) |
            CharacterTransformIteration::WhitespaceCharsCollapsedToOneNewline(count) => {
                self.identity(1);
                if count > 1 {
                    self.collapse(count - 1);
                }
            },
            CharacterTransformIteration::WhitespaceCharsCollapsedToNothing(count) => {
                self.collapse(count)
            },
            CharacterTransformIteration::ToLowercase(c) => self.case_map(c.to_lowercase().len()),
            CharacterTransformIteration::ToUppercase(c) => self.case_map(c.to_uppercase().len()),
            CharacterTransformIteration::ToTitlecase(c) => self.case_map(to_titlecase(c).len()),
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
    offset_map.identity(1);
    offset_map.expand(1);
    offset_map.identity(1);
    offset_map.expand(2);
    offset_map.identity(1);
    offset_map.expand(3);
    offset_map.identity(1);
    offset_map.identity(1);

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
    offset_map.identity(1);
    offset_map.collapse(1);
    offset_map.identity(1);
    offset_map.collapse(2);
    offset_map.identity(1);
    offset_map.identity(1);
    offset_map.collapse(3);
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
