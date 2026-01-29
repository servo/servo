/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use std::iter::once;
use std::ops::Range;

use malloc_size_of_derive::MallocSizeOf;
use rayon::iter::Either;
use unicode_segmentation::UnicodeSegmentation;

use crate::text::{Utf8CodeUnitLength, Utf16CodeUnitLength};

fn contents_vec(contents: impl Into<String>) -> Vec<String> {
    let mut contents: Vec<_> = contents
        .into()
        .split('\n')
        .map(|line| format!("{line}\n"))
        .collect();
    // The last line should not have a newline.
    if let Some(last_line) = contents.last_mut() {
        last_line.truncate(last_line.len() - 1);
    }
    contents
}

/// Describes a unit of movement for [`Rope::move_by`].
pub enum RopeMovement {
    Character,
    Grapheme,
    Word,
    Line,
    LineStartOrEnd,
    RopeStartOrEnd,
}

/// An implementation of a [rope data structure], composed of lines of
/// owned strings. This is used to implement text controls in Servo.
///
/// [rope data structure]: https://en.wikipedia.org/wiki/Rope_(data_structure)
#[derive(MallocSizeOf)]
pub struct Rope {
    /// The lines of the rope. Each line is an owned string that ends with a newline
    /// (`\n`), apart from the last line which has no trailing newline.
    lines: Vec<String>,
}

impl Rope {
    pub fn new(contents: impl Into<String>) -> Self {
        Self {
            lines: contents_vec(contents),
        }
    }

    pub fn contents(&self) -> String {
        self.lines.join("")
    }

    pub fn last_index(&self) -> RopeIndex {
        let line_index = self.lines.len() - 1;
        RopeIndex::new(line_index, self.line(line_index).len())
    }

    /// Replace the given range of [`RopeIndex`]s with the given string. Returns the
    /// [`RopeIndex`] of the end of the insertion.
    pub fn replace_range(
        &mut self,
        mut range: Range<RopeIndex>,
        string: impl Into<String>,
    ) -> RopeIndex {
        range.start = self.normalize_index(range.start);
        range.end = self.normalize_index(range.end);
        assert!(range.start <= range.end);

        let start_index = range.start;
        self.delete_range(range);

        let mut new_contents = contents_vec(string);
        let Some(first_line_of_new_contents) = new_contents.first() else {
            return start_index;
        };

        if new_contents.len() == 1 {
            self.line_for_index_mut(start_index)
                .insert_str(start_index.code_point, first_line_of_new_contents);
            return RopeIndex::new(
                start_index.line,
                start_index.code_point + first_line_of_new_contents.len(),
            );
        }

        let start_line = self.line_for_index_mut(start_index);
        let last_line = new_contents.last().expect("Should have at least one line");
        let last_index = RopeIndex::new(
            start_index.line + new_contents.len().saturating_sub(1),
            last_line.len(),
        );

        let remaining_string = start_line.split_off(start_index.code_point);
        start_line.push_str(first_line_of_new_contents);
        new_contents
            .last_mut()
            .expect("Should have at least one line")
            .push_str(&remaining_string);

        let splice_index = start_index.line + 1;
        self.lines
            .splice(splice_index..splice_index, new_contents.into_iter().skip(1));
        last_index
    }

    fn delete_range(&mut self, mut range: Range<RopeIndex>) {
        range.start = self.normalize_index(range.start);
        range.end = self.normalize_index(range.end);
        assert!(range.start <= range.end);

        if range.start.line == range.end.line {
            self.line_for_index_mut(range.start)
                .replace_range(range.start.code_point..range.end.code_point, "");
            return;
        }

        // Remove the start line and any before the last line.
        let removed_lines = self.lines.splice(range.start.line..range.end.line, []);
        let first_line = removed_lines
            .into_iter()
            .nth(0)
            .expect("Should have removed at least one line");

        let first_line_prefix = &first_line[0..range.start.code_point];
        let new_end_line = range.start.line;
        self.lines[new_end_line].replace_range(0..range.end.code_point, first_line_prefix);
    }

    /// Create a [`RopeSlice`] for this [`Rope`] from `start` to `end`. If either of
    /// these is `None`, then the slice will extend to the extent of the rope.
    pub fn slice<'a>(&'a self, start: Option<RopeIndex>, end: Option<RopeIndex>) -> RopeSlice<'a> {
        RopeSlice {
            rope: self,
            start: start.unwrap_or_default(),
            end: end.unwrap_or_else(|| self.last_index()),
        }
    }

    pub fn chars<'a>(&'a self) -> RopeChars<'a> {
        self.slice(None, None).chars()
    }

    /// Return `true` if the [`Rope`] is empty or false otherwise. This will also
    /// return `true` if the contents of the [`Rope`] are a single empty line.
    pub fn is_empty(&self) -> bool {
        self.lines.first().is_none_or(String::is_empty)
    }

    /// The total number of code units required to encode the content in utf16.
    pub fn len_utf16(&self) -> Utf16CodeUnitLength {
        Utf16CodeUnitLength(self.chars().map(char::len_utf16).sum())
    }

    fn line(&self, index: usize) -> &str {
        &self.lines[index]
    }

    fn line_for_index(&self, index: RopeIndex) -> &String {
        &self.lines[index.line]
    }

    fn line_for_index_mut(&mut self, index: RopeIndex) -> &mut String {
        &mut self.lines[index.line]
    }

    fn last_index_in_line(&self, line: usize) -> RopeIndex {
        if line >= self.lines.len() - 1 {
            return self.last_index();
        }
        RopeIndex {
            line,
            code_point: self.line(line).len() - 1,
        }
    }

    /// Return a [`RopeIndex`] which points to the start of the subsequent line.
    /// If the given [`RopeIndex`] is already on the final line, this will return
    /// the final index of the entire [`Rope`].
    fn start_of_following_line(&self, index: RopeIndex) -> RopeIndex {
        if index.line >= self.lines.len() - 1 {
            return self.last_index();
        }
        RopeIndex::new(index.line + 1, 0)
    }

    /// Return a [`RopeIndex`] which points to the end of preceding line. If already
    /// at the end of the first line, this will return the start index of the entire
    /// [`Rope`].
    fn end_of_preceding_line(&self, index: RopeIndex) -> RopeIndex {
        if index.line == 0 {
            return Default::default();
        }
        let line_index = index.line.saturating_sub(1);
        RopeIndex::new(line_index, self.line(line_index).len())
    }

    pub fn move_by(&self, origin: RopeIndex, unit: RopeMovement, amount: isize) -> RopeIndex {
        if amount == 0 {
            return origin;
        }

        match unit {
            RopeMovement::Character | RopeMovement::Grapheme | RopeMovement::Word => {
                self.move_by_iterator(origin, unit, amount)
            },
            RopeMovement::Line => self.move_by_lines(origin, amount),
            RopeMovement::LineStartOrEnd => {
                if amount >= 0 {
                    self.last_index_in_line(origin.line)
                } else {
                    RopeIndex::new(origin.line, 0)
                }
            },
            RopeMovement::RopeStartOrEnd => {
                if amount >= 0 {
                    self.last_index()
                } else {
                    Default::default()
                }
            },
        }
    }

    fn move_by_lines(&self, origin: RopeIndex, lines_to_move: isize) -> RopeIndex {
        let new_line_index = (origin.line as isize) + lines_to_move;
        if new_line_index < 0 {
            return Default::default();
        }
        if new_line_index > (self.lines.len() - 1) as isize {
            return self.last_index();
        }

        let new_line_index = new_line_index.unsigned_abs();
        let char_count = self.line(origin.line)[0..origin.code_point].chars().count();
        let new_code_point_index = self
            .line(new_line_index)
            .char_indices()
            .take(char_count)
            .last()
            .map(|(byte_index, character)| byte_index + character.len_utf8())
            .unwrap_or_default();
        RopeIndex::new(new_line_index, new_code_point_index)
            .min(self.last_index_in_line(new_line_index))
    }

    fn move_by_iterator(&self, origin: RopeIndex, unit: RopeMovement, amount: isize) -> RopeIndex {
        assert_ne!(amount, 0);
        let (boundary_value, slice) = if amount > 0 {
            (self.last_index(), self.slice(Some(origin), None))
        } else {
            (RopeIndex::default(), self.slice(None, Some(origin)))
        };

        let iterator = match unit {
            RopeMovement::Character => slice.char_indices(),
            RopeMovement::Grapheme => slice.grapheme_indices(),
            RopeMovement::Word => slice.word_indices(),
            _ => unreachable!("Should not be called for other movement types"),
        };
        let iterator = if amount > 0 {
            Either::Left(iterator)
        } else {
            Either::Right(iterator.rev())
        };

        let mut iterations = amount.unsigned_abs();
        for mut index in iterator {
            iterations = iterations.saturating_sub(1);
            if iterations == 0 {
                // Instead of returning offsets for the absolute end of a line, return the
                // start offset for the next line.
                if index.code_point >= self.line_for_index(index).len() {
                    index = self.start_of_following_line(index);
                }
                return index;
            }
        }

        boundary_value
    }

    /// Given a [`RopeIndex`], clamp it and ensure that it is on a character boundary,
    /// meaning that its indices are all bound by the actual size of the line and the
    /// number of lines in this [`Rope`].
    pub fn normalize_index(&self, rope_index: RopeIndex) -> RopeIndex {
        let last_line = self.lines.len().saturating_sub(1);
        let line_index = rope_index.line.min(last_line);

        // This may appear a bit odd as we are adding an index to the end of the line,
        // but `RopeIndex` isn't just an offset to a UTF-8 code point, but also can
        // serve as the end of an exclusive range so there is one more index at the end
        // that is still valid.
        //
        // Lines other than the last line have a trailing newline. We do not want to allow
        // an index past the trailing newline.
        let line = self.line(line_index);
        let line_length_utf8 = if line_index == last_line {
            line.len()
        } else {
            line.len() - 1
        };

        let mut code_point = rope_index.code_point.min(line_length_utf8);
        while code_point < line.len() && !line.is_char_boundary(code_point) {
            code_point += 1;
        }

        RopeIndex::new(line_index, code_point)
    }

    /// Convert a [`RopeIndex`] into a byte offset from the start of the content.
    pub fn index_to_utf8_offset(&self, rope_index: RopeIndex) -> Utf8CodeUnitLength {
        let rope_index = self.normalize_index(rope_index);
        Utf8CodeUnitLength(
            self.lines
                .iter()
                .take(rope_index.line)
                .map(String::len)
                .sum::<usize>() +
                rope_index.code_point,
        )
    }

    pub fn index_to_utf16_offset(&self, rope_index: RopeIndex) -> Utf16CodeUnitLength {
        let rope_index = self.normalize_index(rope_index);
        let final_line = self.line(rope_index.line);

        // The offset might be past the end of the line due to being an exclusive offset.
        let final_line_offset = Utf16CodeUnitLength(
            final_line[0..rope_index.code_point]
                .chars()
                .map(char::len_utf16)
                .sum(),
        );

        self.lines
            .iter()
            .take(rope_index.line)
            .map(|line| Utf16CodeUnitLength(line.chars().map(char::len_utf16).sum()))
            .sum::<Utf16CodeUnitLength>() +
            final_line_offset
    }

    /// Convert a [`RopeIndex`] into a character offset from the start of the content.
    pub fn index_to_character_offset(&self, rope_index: RopeIndex) -> usize {
        let rope_index = self.normalize_index(rope_index);

        // The offset might be past the end of the line due to being an exclusive offset.
        let final_line = self.line(rope_index.line);
        let final_line_offset = final_line[0..rope_index.code_point].chars().count();
        self.lines
            .iter()
            .take(rope_index.line)
            .map(|line| line.chars().count())
            .sum::<usize>() +
            final_line_offset
    }

    /// Convert a byte offset from the start of the content into a [`RopeIndex`].
    pub fn utf8_offset_to_rope_index(&self, utf8_offset: Utf8CodeUnitLength) -> RopeIndex {
        let mut current_utf8_offset = utf8_offset.0;
        for (line_index, line) in self.lines.iter().enumerate() {
            if current_utf8_offset == 0 || current_utf8_offset < line.len() {
                return RopeIndex::new(line_index, current_utf8_offset);
            }
            current_utf8_offset -= line.len();
        }
        self.last_index()
    }

    pub fn utf16_offset_to_utf8_offset(
        &self,
        utf16_offset: Utf16CodeUnitLength,
    ) -> Utf8CodeUnitLength {
        let mut current_utf16_offset = Utf16CodeUnitLength::zero();
        let mut current_utf8_offset = Utf8CodeUnitLength::zero();

        for character in self.chars() {
            let utf16_length = character.len_utf16();
            if current_utf16_offset + Utf16CodeUnitLength(utf16_length) > utf16_offset {
                return current_utf8_offset;
            }
            current_utf8_offset += Utf8CodeUnitLength(character.len_utf8());
            current_utf16_offset += Utf16CodeUnitLength(utf16_length);
        }
        current_utf8_offset
    }

    /// Find the boundaries of the word most relevant to the given [`RopeIndex`]. Word
    /// returned in order or precedence:
    ///
    /// - If the index intersects the word or is the index directly preceding a word,
    ///   the boundaries of that word are returned.
    /// - The word preceding the cursor.
    /// - If there is no word preceding the cursor, the start of the line to the end
    ///   of the next word.
    pub fn relevant_word_boundaries<'a>(&'a self, index: RopeIndex) -> RopeSlice<'a> {
        let line = self.line_for_index(index);
        let mut result_start = 0;
        let mut result_end = None;
        for (word_start, word) in line.unicode_word_indices() {
            if word_start > index.code_point {
                result_end = result_end.or_else(|| Some(word_start + word.len()));
                break;
            }
            result_start = word_start;
            result_end = Some(word_start + word.len());
        }

        let result_end = result_end.unwrap_or(result_start);
        self.slice(
            Some(RopeIndex::new(index.line, result_start)),
            Some(RopeIndex::new(index.line, result_end)),
        )
    }

    /// Return the boundaries of the line that contains the given [`RopeIndex`].
    pub fn line_boundaries<'a>(&'a self, index: RopeIndex) -> RopeSlice<'a> {
        self.slice(
            Some(RopeIndex::new(index.line, 0)),
            Some(self.last_index_in_line(index.line)),
        )
    }

    fn character_at(&self, index: RopeIndex) -> Option<char> {
        let line = self.line_for_index(index);
        line[index.code_point..].chars().next()
    }

    fn character_before(&self, index: RopeIndex) -> Option<char> {
        let line = self.line_for_index(index);
        line[..index.code_point].chars().next_back()
    }
}

/// An index into a [`Rope`] data structure. Used to efficiently identify a particular
/// position in a [`Rope`]. As [`Rope`] always uses Rust strings interally, code point
/// indices represented in a [`RopeIndex`] are assumed to be UTF-8 code points (one byte
/// each).
///
/// Note that it is possible for a [`RopeIndex`] to point past the end of the last line,
/// as it can be used in exclusive ranges. In lines other than the last line, it should
/// always refer to offsets before the trailing newline.
#[derive(Clone, Copy, Debug, Default, Eq, MallocSizeOf, PartialEq, PartialOrd, Ord)]
pub struct RopeIndex {
    /// The index of the line that this [`RopeIndex`] refers to.
    pub line: usize,
    /// The index of the code point on the [`RopeIndex`]'s line in UTF-8 code
    /// points.
    ///
    /// Note: This is not a `Utf8CodeUnitLength` in order to avoid continually having
    /// to unpack the inner value.
    pub code_point: usize,
}

impl RopeIndex {
    pub fn new(line: usize, code_point: usize) -> Self {
        Self { line, code_point }
    }
}

/// A slice of a [`Rope`]. This can be used to to iterate over a subset of characters of a
/// [`Rope`] or to return the content of the [`RopeSlice`] as a `String`.
pub struct RopeSlice<'a> {
    /// The underlying [`Rope`] of this [`RopeSlice`]
    rope: &'a Rope,
    /// The inclusive `RopeIndex` of the start of this [`RopeSlice`].
    pub start: RopeIndex,
    /// The exclusive end `RopeIndex` of this [`RopeSlice`].
    pub end: RopeIndex,
}

impl From<RopeSlice<'_>> for String {
    fn from(slice: RopeSlice<'_>) -> Self {
        if slice.start.line == slice.end.line {
            slice.rope.line_for_index(slice.start)[slice.start.code_point..slice.end.code_point]
                .into()
        } else {
            once(&slice.rope.line_for_index(slice.start)[slice.start.code_point..])
                .chain(
                    (slice.start.line + 1..slice.end.line)
                        .map(|line_index| slice.rope.line(line_index)),
                )
                .chain(once(
                    &slice.rope.line_for_index(slice.end)[..slice.end.code_point],
                ))
                .collect()
        }
    }
}

impl<'a> RopeSlice<'a> {
    pub fn chars(self) -> RopeChars<'a> {
        RopeChars {
            movement_iterator: RopeMovementIterator {
                slice: self,
                end_of_forward_motion: |_, string| {
                    let (offset, character) = string.char_indices().next()?;
                    Some(offset + character.len_utf8())
                },
                start_of_backward_motion: |_, string: &str| {
                    Some(string.char_indices().next_back()?.0)
                },
            },
        }
    }

    fn char_indices(self) -> RopeMovementIterator<'a> {
        RopeMovementIterator {
            slice: self,
            end_of_forward_motion: |_, string| {
                let (offset, character) = string.char_indices().next()?;
                Some(offset + character.len_utf8())
            },
            start_of_backward_motion: |_, string: &str| Some(string.char_indices().next_back()?.0),
        }
    }

    fn grapheme_indices(self) -> RopeMovementIterator<'a> {
        RopeMovementIterator {
            slice: self,
            end_of_forward_motion: |_, string| {
                let (offset, grapheme) = string.grapheme_indices(true).next()?;
                Some(offset + grapheme.len())
            },
            start_of_backward_motion: |_, string| {
                Some(string.grapheme_indices(true).next_back()?.0)
            },
        }
    }

    fn word_indices(self) -> RopeMovementIterator<'a> {
        RopeMovementIterator {
            slice: self,
            end_of_forward_motion: |_, string| {
                let (offset, word) = string.unicode_word_indices().next()?;
                Some(offset + word.len())
            },
            start_of_backward_motion: |_, string| {
                Some(string.unicode_word_indices().next_back()?.0)
            },
        }
    }
}

/// A generic movement iterator for a [`Rope`]. This can move in both directions. Note
/// than when moving forward and backward, the indices returned for each unit are
/// different. When moving forward, the end of the unit of movement is returned and when
/// moving backward the start of the unit of movement is returned. This matches the
/// expected behavior when interactively moving through editable text.
struct RopeMovementIterator<'a> {
    slice: RopeSlice<'a>,
    end_of_forward_motion: fn(&RopeSlice, &'a str) -> Option<usize>,
    start_of_backward_motion: fn(&RopeSlice, &'a str) -> Option<usize>,
}

impl Iterator for RopeMovementIterator<'_> {
    type Item = RopeIndex;

    fn next(&mut self) -> Option<Self::Item> {
        // If the two indices have crossed over, iteration is done.
        if self.slice.start >= self.slice.end {
            return None;
        }

        assert!(self.slice.start.line < self.slice.rope.lines.len());
        let line = self.slice.rope.line_for_index(self.slice.start);

        if self.slice.start.code_point < line.len() + 1 {
            if let Some(end_offset) =
                (self.end_of_forward_motion)(&self.slice, &line[self.slice.start.code_point..])
            {
                self.slice.start.code_point += end_offset;
                return Some(self.slice.start);
            }
        }

        // Advance the line as we are at the end of the line.
        self.slice.start = self.slice.rope.start_of_following_line(self.slice.start);
        self.next()
    }
}

impl DoubleEndedIterator for RopeMovementIterator<'_> {
    fn next_back(&mut self) -> Option<Self::Item> {
        // If the two indices have crossed over, iteration is done.
        if self.slice.end <= self.slice.start {
            return None;
        }

        let line = self.slice.rope.line_for_index(self.slice.end);
        if self.slice.end.code_point > 0 {
            if let Some(new_start_index) =
                (self.start_of_backward_motion)(&self.slice, &line[..self.slice.end.code_point])
            {
                self.slice.end.code_point = new_start_index;
                return Some(self.slice.end);
            }
        }

        // Decrease the line index as we are at the start of the line.
        self.slice.end = self.slice.rope.end_of_preceding_line(self.slice.end);
        self.next_back()
    }
}

/// A `Chars`-like iterator for [`Rope`].
pub struct RopeChars<'a> {
    movement_iterator: RopeMovementIterator<'a>,
}

impl Iterator for RopeChars<'_> {
    type Item = char;
    fn next(&mut self) -> Option<Self::Item> {
        self.movement_iterator
            .next()
            .and_then(|index| self.movement_iterator.slice.rope.character_before(index))
    }
}

impl DoubleEndedIterator for RopeChars<'_> {
    fn next_back(&mut self) -> Option<Self::Item> {
        self.movement_iterator
            .next_back()
            .and_then(|index| self.movement_iterator.slice.rope.character_at(index))
    }
}

#[test]
fn test_rope_index_conversion_to_utf8_offset() {
    let rope = Rope::new("A\nBB\nCCC\nDDDD");
    assert_eq!(
        rope.index_to_utf8_offset(RopeIndex::new(0, 0)),
        Utf8CodeUnitLength(0),
    );
    assert_eq!(
        rope.index_to_utf8_offset(RopeIndex::new(0, 1)),
        Utf8CodeUnitLength(1),
    );
    assert_eq!(
        rope.index_to_utf8_offset(RopeIndex::new(0, 10)),
        Utf8CodeUnitLength(1),
        "RopeIndex with offset past the end of the line should return final offset in line",
    );
    assert_eq!(
        rope.index_to_utf8_offset(RopeIndex::new(1, 0)),
        Utf8CodeUnitLength(2),
    );
    assert_eq!(
        rope.index_to_utf8_offset(RopeIndex::new(1, 2)),
        Utf8CodeUnitLength(4),
    );

    assert_eq!(
        rope.index_to_utf8_offset(RopeIndex::new(3, 0)),
        Utf8CodeUnitLength(9),
    );
    assert_eq!(
        rope.index_to_utf8_offset(RopeIndex::new(3, 3)),
        Utf8CodeUnitLength(12),
    );
    assert_eq!(
        rope.index_to_utf8_offset(RopeIndex::new(3, 4)),
        Utf8CodeUnitLength(13),
        "There should be no newline at the end of the TextInput",
    );
    assert_eq!(
        rope.index_to_utf8_offset(RopeIndex::new(3, 40)),
        Utf8CodeUnitLength(13),
        "There should be no newline at the end of the TextInput",
    );
}

#[test]
fn test_rope_index_conversion_to_utf16_offset() {
    let rope = Rope::new("A\nBB\nCCC\n家家");
    assert_eq!(
        rope.index_to_utf16_offset(RopeIndex::new(0, 0)),
        Utf16CodeUnitLength(0),
    );
    assert_eq!(
        rope.index_to_utf16_offset(RopeIndex::new(0, 1)),
        Utf16CodeUnitLength(1),
    );
    assert_eq!(
        rope.index_to_utf16_offset(RopeIndex::new(0, 10)),
        Utf16CodeUnitLength(1),
        "RopeIndex with offset past the end of the line should return final offset in line",
    );
    assert_eq!(
        rope.index_to_utf16_offset(RopeIndex::new(3, 0)),
        Utf16CodeUnitLength(9),
    );

    assert_eq!(
        rope.index_to_utf16_offset(RopeIndex::new(3, 3)),
        Utf16CodeUnitLength(10),
        "3 code unit UTF-8 encodede character"
    );
    assert_eq!(
        rope.index_to_utf16_offset(RopeIndex::new(3, 6)),
        Utf16CodeUnitLength(11),
    );
    assert_eq!(
        rope.index_to_utf16_offset(RopeIndex::new(3, 20)),
        Utf16CodeUnitLength(11),
    );
}

#[test]
fn test_utf16_offset_to_utf8_offset() {
    let rope = Rope::new("A\nBB\nCCC\n家家");
    assert_eq!(
        rope.utf16_offset_to_utf8_offset(Utf16CodeUnitLength(0)),
        Utf8CodeUnitLength(0),
    );
    assert_eq!(
        rope.utf16_offset_to_utf8_offset(Utf16CodeUnitLength(1)),
        Utf8CodeUnitLength(1),
    );
    assert_eq!(
        rope.utf16_offset_to_utf8_offset(Utf16CodeUnitLength(2)),
        Utf8CodeUnitLength(2),
        "Offset past the end of the line",
    );
    assert_eq!(
        rope.utf16_offset_to_utf8_offset(Utf16CodeUnitLength(9)),
        Utf8CodeUnitLength(9),
    );

    assert_eq!(
        rope.utf16_offset_to_utf8_offset(Utf16CodeUnitLength(10)),
        Utf8CodeUnitLength(12),
        "3 code unit UTF-8 encodede character"
    );
    assert_eq!(
        rope.utf16_offset_to_utf8_offset(Utf16CodeUnitLength(11)),
        Utf8CodeUnitLength(15),
    );
    assert_eq!(
        rope.utf16_offset_to_utf8_offset(Utf16CodeUnitLength(300)),
        Utf8CodeUnitLength(15),
    );
}

#[test]
fn test_rope_delete_slice() {
    let mut rope = Rope::new("ABC\nDEF\n");
    rope.delete_range(RopeIndex::new(0, 1)..RopeIndex::new(0, 2));
    assert_eq!(rope.contents(), "AC\nDEF\n");

    // Trying to delete beyond the last index of the line should note remove any trailing
    // newlines from the rope.
    let mut rope = Rope::new("ABC\nDEF\n");
    rope.delete_range(RopeIndex::new(0, 3)..RopeIndex::new(0, 4));
    assert_eq!(rope.lines, ["ABC\n", "DEF\n", ""]);

    let mut rope = Rope::new("ABC\nDEF\n");
    rope.delete_range(RopeIndex::new(0, 0)..RopeIndex::new(0, 4));
    assert_eq!(rope.lines, ["\n", "DEF\n", ""]);

    let mut rope = Rope::new("A\nBB\nCCC");
    rope.delete_range(RopeIndex::new(0, 2)..RopeIndex::new(1, 0));
    assert_eq!(rope.lines, ["ABB\n", "CCC"]);
}

#[test]
fn test_rope_replace_slice() {
    let mut rope = Rope::new("AAA\nBBB\nCCC");
    rope.replace_range(RopeIndex::new(0, 1)..RopeIndex::new(0, 2), "x");
    assert_eq!(rope.contents(), "AxA\nBBB\nCCC",);

    let mut rope = Rope::new("A\nBB\nCCC");
    rope.replace_range(RopeIndex::new(0, 2)..RopeIndex::new(1, 0), "D");
    assert_eq!(rope.lines, ["ADBB\n", "CCC"]);

    let mut rope = Rope::new("AAA\nBBB\nCCC\nDDD");
    rope.replace_range(RopeIndex::new(0, 2)..RopeIndex::new(2, 1), "x");
    assert_eq!(rope.lines, ["AAxCC\n", "DDD"]);
}

#[test]
fn test_rope_relevant_word() {
    let rope = Rope::new("AAA    BBB   CCC");
    let boundaries = rope.relevant_word_boundaries(RopeIndex::new(0, 0));
    assert_eq!(boundaries.start, RopeIndex::new(0, 0));
    assert_eq!(boundaries.end, RopeIndex::new(0, 3));

    // Choose previous word if starting on whitespace.
    let boundaries = rope.relevant_word_boundaries(RopeIndex::new(0, 4));
    assert_eq!(boundaries.start, RopeIndex::new(0, 0));
    assert_eq!(boundaries.end, RopeIndex::new(0, 3));

    // Choose next word if starting at word start.
    let boundaries = rope.relevant_word_boundaries(RopeIndex::new(0, 7));
    assert_eq!(boundaries.start, RopeIndex::new(0, 7));
    assert_eq!(boundaries.end, RopeIndex::new(0, 10));

    // Choose word if starting at in middle.
    let boundaries = rope.relevant_word_boundaries(RopeIndex::new(0, 8));
    assert_eq!(boundaries.start, RopeIndex::new(0, 7));
    assert_eq!(boundaries.end, RopeIndex::new(0, 10));

    // Choose start of line to end of first word if in whitespace at start of line.
    let rope = Rope::new("         AAA    BBB   CCC");
    let boundaries = rope.relevant_word_boundaries(RopeIndex::new(0, 3));
    assert_eq!(boundaries.start, RopeIndex::new(0, 0));
    assert_eq!(boundaries.end, RopeIndex::new(0, 12));

    // Works properly if line is empty.
    let rope = Rope::new("");
    let boundaries = rope.relevant_word_boundaries(RopeIndex::new(0, 0));
    assert_eq!(boundaries.start, RopeIndex::new(0, 0));
    assert_eq!(boundaries.end, RopeIndex::new(0, 0));
}

#[test]
fn test_rope_index_intersects_character() {
    let rope = Rope::new("񉡚");
    let rope_index = RopeIndex::new(0, 1);
    assert_eq!(rope.normalize_index(rope_index), RopeIndex::new(0, 4));
    assert_eq!(
        rope.index_to_utf16_offset(rope_index),
        Utf16CodeUnitLength(2)
    );
    assert_eq!(rope.index_to_utf8_offset(rope_index), Utf8CodeUnitLength(4));

    let rope = Rope::new("abc\ndef");
    assert_eq!(
        rope.normalize_index(RopeIndex::new(0, 100)),
        RopeIndex::new(0, 3),
        "Normalizing index past end of line should just clamp to line length."
    );
    assert_eq!(
        rope.normalize_index(RopeIndex::new(1, 100)),
        RopeIndex::new(1, 3),
        "Normalizing index past end of line should just clamp to line length."
    );
}
