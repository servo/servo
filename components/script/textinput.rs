/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

//! Common handling of keyboard input and state management for text input controls

use crate::clipboard_provider::ClipboardProvider;
use crate::dom::bindings::str::DOMString;
use crate::dom::compositionevent::CompositionEvent;
use crate::dom::keyboardevent::KeyboardEvent;
use keyboard_types::{Key, KeyState, Modifiers, ShortcutMatcher};
use std::borrow::ToOwned;
use std::cmp::{max, min};
use std::default::Default;
use std::ops::{Add, AddAssign, Range, Sub, SubAssign};
use std::usize;
use unicode_segmentation::UnicodeSegmentation;

#[derive(Clone, Copy, PartialEq)]
pub enum Selection {
    Selected,
    NotSelected,
}

#[derive(Clone, Copy, Debug, JSTraceable, MallocSizeOf, PartialEq)]
pub enum SelectionDirection {
    Forward,
    Backward,
    None,
}

#[derive(Clone, Copy, Debug, Eq, JSTraceable, MallocSizeOf, Ord, PartialEq, PartialOrd)]
pub struct ByteOffset(pub usize);

impl ByteOffset {
    pub fn zero() -> ByteOffset {
        ByteOffset(0)
    }

    pub fn one() -> ByteOffset {
        ByteOffset(1)
    }

    pub fn unwrap_range(byte_range: Range<ByteOffset>) -> Range<usize> {
        byte_range.start.0..byte_range.end.0
    }
}

impl Add for ByteOffset {
    type Output = ByteOffset;

    fn add(self, other: ByteOffset) -> ByteOffset {
        ByteOffset(self.0 + other.0)
    }
}

impl Sub for ByteOffset {
    type Output = ByteOffset;

    fn sub(self, other: ByteOffset) -> ByteOffset {
        if self > other {
            ByteOffset(self.0 - other.0)
        } else {
            ByteOffset::zero()
        }
    }
}

impl AddAssign for ByteOffset {
    fn add_assign(&mut self, other: ByteOffset) {
        *self = ByteOffset(self.0 + other.0)
    }
}

impl SubAssign for ByteOffset {
    fn sub_assign(&mut self, other: ByteOffset) {
        if *self > other {
            *self = ByteOffset(self.0 + other.0)
        } else {
            *self = ByteOffset::zero()
        }
    }
}

#[derive(Clone, Copy, Debug, JSTraceable, MallocSizeOf, PartialEq, PartialOrd)]
pub struct UTF16CodeUnits(pub usize);

impl UTF16CodeUnits {
    pub fn zero() -> UTF16CodeUnits {
        UTF16CodeUnits(0)
    }

    pub fn one() -> UTF16CodeUnits {
        UTF16CodeUnits(1)
    }
}

impl Add for UTF16CodeUnits {
    type Output = UTF16CodeUnits;

    fn add(self, other: UTF16CodeUnits) -> UTF16CodeUnits {
        UTF16CodeUnits(self.0 + other.0)
    }
}

impl Sub for UTF16CodeUnits {
    type Output = UTF16CodeUnits;

    fn sub(self, other: UTF16CodeUnits) -> UTF16CodeUnits {
        if self > other {
            UTF16CodeUnits(self.0 - other.0)
        } else {
            UTF16CodeUnits::zero()
        }
    }
}

impl AddAssign for UTF16CodeUnits {
    fn add_assign(&mut self, other: UTF16CodeUnits) {
        *self = UTF16CodeUnits(self.0 + other.0)
    }
}

impl From<DOMString> for SelectionDirection {
    fn from(direction: DOMString) -> SelectionDirection {
        match direction.as_ref() {
            "forward" => SelectionDirection::Forward,
            "backward" => SelectionDirection::Backward,
            _ => SelectionDirection::None,
        }
    }
}

impl From<SelectionDirection> for DOMString {
    fn from(direction: SelectionDirection) -> DOMString {
        match direction {
            SelectionDirection::Forward => DOMString::from("forward"),
            SelectionDirection::Backward => DOMString::from("backward"),
            SelectionDirection::None => DOMString::from("none"),
        }
    }
}

#[derive(Clone, Copy, Debug, JSTraceable, MallocSizeOf, PartialEq, PartialOrd)]
pub struct TextPoint {
    /// 0-based line number
    pub line: usize,
    /// 0-based column number in bytes
    pub index: ByteOffset,
}

impl TextPoint {
    /// Returns a TextPoint constrained to be a valid location within lines
    fn constrain_to(&self, lines: &[DOMString]) -> TextPoint {
        let line = min(self.line, lines.len() - 1);

        TextPoint {
            line,
            index: min(self.index, ByteOffset(lines[line].len())),
        }
    }
}

#[derive(Clone, Copy, PartialEq)]
pub struct SelectionState {
    start: TextPoint,
    end: TextPoint,
    direction: SelectionDirection,
}

/// Encapsulated state for handling keyboard input in a single or multiline text input control.
#[derive(JSTraceable, MallocSizeOf)]
pub struct TextInput<T: ClipboardProvider> {
    /// Current text input content, split across lines without trailing '\n'
    lines: Vec<DOMString>,

    /// Current cursor input point
    edit_point: TextPoint,

    /// The current selection goes from the selection_origin until the edit_point. Note that the
    /// selection_origin may be after the edit_point, in the case of a backward selection.
    selection_origin: Option<TextPoint>,
    selection_direction: SelectionDirection,

    /// Is this a multiline input?
    multiline: bool,

    #[ignore_malloc_size_of = "Can't easily measure this generic type"]
    clipboard_provider: T,

    /// The maximum number of UTF-16 code units this text input is allowed to hold.
    ///
    /// <https://html.spec.whatwg.org/multipage/#attr-fe-maxlength>
    max_length: Option<UTF16CodeUnits>,
    min_length: Option<UTF16CodeUnits>,
}

/// Resulting action to be taken by the owner of a text input that is handling an event.
pub enum KeyReaction {
    TriggerDefaultAction,
    DispatchInput,
    RedrawSelection,
    Nothing,
}

impl Default for TextPoint {
    fn default() -> TextPoint {
        TextPoint {
            line: 0,
            index: ByteOffset::zero(),
        }
    }
}

/// Control whether this control should allow multiple lines.
#[derive(Eq, PartialEq)]
pub enum Lines {
    Single,
    Multiple,
}

/// The direction in which to delete a character.
#[derive(Clone, Copy, Eq, PartialEq)]
pub enum Direction {
    Forward,
    Backward,
}

// Some shortcuts use Cmd on Mac and Control on other systems.
#[cfg(target_os = "macos")]
pub const CMD_OR_CONTROL: Modifiers = Modifiers::META;
#[cfg(not(target_os = "macos"))]
pub const CMD_OR_CONTROL: Modifiers = Modifiers::CONTROL;

// FIXME this function does not behave as described (if given string has fewer than n
// characters, it returns 0, not the length of the whole string.
/// The length in bytes of the first n characters in a UTF-8 string.
///
/// If the string has fewer than n characters, returns the length of the whole string.
fn len_of_first_n_chars(text: &str, n: usize) -> ByteOffset {
    match text.char_indices().take(n).last() {
        Some((index, ch)) => ByteOffset(index + ch.len_utf8()),
        None => ByteOffset::zero(),
    }
}

/// The length in bytes of the first n code units in a string when encoded in UTF-16.
///
/// If the string is fewer than n code units, returns the length of the whole string.
fn len_of_first_n_code_units(text: &str, n: UTF16CodeUnits) -> ByteOffset {
    let mut utf8_len = ByteOffset::zero();
    let mut utf16_len = UTF16CodeUnits::zero();
    for c in text.chars() {
        utf16_len += UTF16CodeUnits(c.len_utf16());
        if utf16_len > n {
            break;
        }
        utf8_len += ByteOffset(c.len_utf8());
    }
    utf8_len
}

impl<T: ClipboardProvider> TextInput<T> {
    /// Instantiate a new text input control
    pub fn new(
        lines: Lines,
        initial: DOMString,
        clipboard_provider: T,
        max_length: Option<UTF16CodeUnits>,
        min_length: Option<UTF16CodeUnits>,
        selection_direction: SelectionDirection,
    ) -> TextInput<T> {
        let mut i = TextInput {
            lines: vec![],
            edit_point: Default::default(),
            selection_origin: None,
            multiline: lines == Lines::Multiple,
            clipboard_provider: clipboard_provider,
            max_length: max_length,
            min_length: min_length,
            selection_direction: selection_direction,
        };
        i.set_content(initial);
        i
    }

    pub fn edit_point(&self) -> TextPoint {
        self.edit_point
    }

    pub fn selection_origin(&self) -> Option<TextPoint> {
        self.selection_origin
    }

    /// The selection origin, or the edit point if there is no selection. Note that the selection
    /// origin may be after the edit point, in the case of a backward selection.
    pub fn selection_origin_or_edit_point(&self) -> TextPoint {
        self.selection_origin.unwrap_or(self.edit_point)
    }

    pub fn selection_direction(&self) -> SelectionDirection {
        self.selection_direction
    }

    pub fn set_max_length(&mut self, length: Option<UTF16CodeUnits>) {
        self.max_length = length;
    }

    pub fn set_min_length(&mut self, length: Option<UTF16CodeUnits>) {
        self.min_length = length;
    }

    /// Remove a character at the current editing point
    pub fn delete_char(&mut self, dir: Direction) {
        if self.selection_origin.is_none() || self.selection_origin == Some(self.edit_point) {
            self.adjust_horizontal_by_one(dir, Selection::Selected);
        }
        self.replace_selection(DOMString::new());
    }

    /// Insert a character at the current editing point
    pub fn insert_char(&mut self, ch: char) {
        self.insert_string(ch.to_string());
    }

    /// Insert a string at the current editing point
    pub fn insert_string<S: Into<String>>(&mut self, s: S) {
        if self.selection_origin.is_none() {
            self.selection_origin = Some(self.edit_point);
        }
        self.replace_selection(DOMString::from(s.into()));
    }

    /// The start of the selection (or the edit point, if there is no selection). Always less than
    /// or equal to selection_end(), regardless of the selection direction.
    pub fn selection_start(&self) -> TextPoint {
        match self.selection_direction {
            SelectionDirection::None | SelectionDirection::Forward => {
                self.selection_origin_or_edit_point()
            },
            SelectionDirection::Backward => self.edit_point,
        }
    }

    /// The byte offset of the selection_start()
    pub fn selection_start_offset(&self) -> ByteOffset {
        self.text_point_to_offset(&self.selection_start())
    }

    /// The end of the selection (or the edit point, if there is no selection). Always greater
    /// than or equal to selection_start(), regardless of the selection direction.
    pub fn selection_end(&self) -> TextPoint {
        match self.selection_direction {
            SelectionDirection::None | SelectionDirection::Forward => self.edit_point,
            SelectionDirection::Backward => self.selection_origin_or_edit_point(),
        }
    }

    /// The byte offset of the selection_end()
    pub fn selection_end_offset(&self) -> ByteOffset {
        self.text_point_to_offset(&self.selection_end())
    }

    /// Whether or not there is an active selection (the selection may be zero-length)
    #[inline]
    pub fn has_selection(&self) -> bool {
        self.selection_origin.is_some()
    }

    /// Returns a tuple of (start, end) giving the bounds of the current selection. start is always
    /// less than or equal to end.
    pub fn sorted_selection_bounds(&self) -> (TextPoint, TextPoint) {
        (self.selection_start(), self.selection_end())
    }

    /// Return the selection range as byte offsets from the start of the content.
    ///
    /// If there is no selection, returns an empty range at the edit point.
    pub fn sorted_selection_offsets_range(&self) -> Range<ByteOffset> {
        self.selection_start_offset()..self.selection_end_offset()
    }

    /// The state of the current selection. Can be used to compare whether selection state has changed.
    pub fn selection_state(&self) -> SelectionState {
        SelectionState {
            start: self.selection_start(),
            end: self.selection_end(),
            direction: self.selection_direction,
        }
    }

    // Check that the selection is valid.
    fn assert_ok_selection(&self) {
        debug!(
            "edit_point: {:?}, selection_origin: {:?}, direction: {:?}",
            self.edit_point, self.selection_origin, self.selection_direction
        );
        if let Some(begin) = self.selection_origin {
            let ByteOffset(begin_offset) = begin.index;
            debug_assert!(begin.line < self.lines.len());
            debug_assert!(begin_offset <= self.lines[begin.line].len());

            match self.selection_direction {
                SelectionDirection::None | SelectionDirection::Forward => {
                    debug_assert!(begin <= self.edit_point)
                },

                SelectionDirection::Backward => debug_assert!(self.edit_point <= begin),
            }
        }

        let ByteOffset(edit_offset) = self.edit_point.index;
        debug_assert!(self.edit_point.line < self.lines.len());
        debug_assert!(edit_offset <= self.lines[self.edit_point.line].len());
    }

    pub fn get_selection_text(&self) -> Option<String> {
        let text = self.fold_selection_slices(String::new(), |s, slice| s.push_str(slice));
        if text.is_empty() {
            return None;
        }
        Some(text)
    }

    /// The length of the selected text in UTF-16 code units.
    fn selection_utf16_len(&self) -> UTF16CodeUnits {
        self.fold_selection_slices(UTF16CodeUnits::zero(), |len, slice| {
            *len += UTF16CodeUnits(slice.chars().map(char::len_utf16).sum::<usize>())
        })
    }

    /// Run the callback on a series of slices that, concatenated, make up the selected text.
    ///
    /// The accumulator `acc` can be mutated by the callback, and will be returned at the end.
    fn fold_selection_slices<B, F: FnMut(&mut B, &str)>(&self, mut acc: B, mut f: F) -> B {
        if self.has_selection() {
            let (start, end) = self.sorted_selection_bounds();
            let ByteOffset(start_offset) = start.index;
            let ByteOffset(end_offset) = end.index;

            if start.line == end.line {
                f(&mut acc, &self.lines[start.line][start_offset..end_offset])
            } else {
                f(&mut acc, &self.lines[start.line][start_offset..]);
                for line in &self.lines[start.line + 1..end.line] {
                    f(&mut acc, "\n");
                    f(&mut acc, line);
                }
                f(&mut acc, "\n");
                f(&mut acc, &self.lines[end.line][..end_offset])
            }
        }

        acc
    }

    pub fn replace_selection(&mut self, insert: DOMString) {
        if !self.has_selection() {
            return;
        }

        let allowed_to_insert_count = if let Some(max_length) = self.max_length {
            let len_after_selection_replaced = self.utf16_len() - self.selection_utf16_len();
            if len_after_selection_replaced >= max_length {
                // If, after deleting the selection, the len is still greater than the max
                // length, then don't delete/insert anything
                return;
            }

            max_length - len_after_selection_replaced
        } else {
            UTF16CodeUnits(usize::MAX)
        };

        let ByteOffset(last_char_index) =
            len_of_first_n_code_units(&*insert, allowed_to_insert_count);
        let bytes_to_insert = &insert[..last_char_index];

        let (start, end) = self.sorted_selection_bounds();
        let ByteOffset(start_offset) = start.index;
        let ByteOffset(end_offset) = end.index;

        let new_lines = {
            let prefix = &self.lines[start.line][..start_offset];
            let suffix = &self.lines[end.line][end_offset..];
            let lines_prefix = &self.lines[..start.line];
            let lines_suffix = &self.lines[end.line + 1..];

            let mut insert_lines = if self.multiline {
                bytes_to_insert
                    .split('\n')
                    .map(|s| DOMString::from(s))
                    .collect()
            } else {
                vec![DOMString::from(bytes_to_insert)]
            };

            // FIXME(ajeffrey): effecient append for DOMStrings
            let mut new_line = prefix.to_owned();

            new_line.push_str(&insert_lines[0]);
            insert_lines[0] = DOMString::from(new_line);

            let last_insert_lines_index = insert_lines.len() - 1;
            self.edit_point.index = ByteOffset(insert_lines[last_insert_lines_index].len());
            self.edit_point.line = start.line + last_insert_lines_index;

            // FIXME(ajeffrey): effecient append for DOMStrings
            insert_lines[last_insert_lines_index].push_str(suffix);

            let mut new_lines = vec![];
            new_lines.extend_from_slice(lines_prefix);
            new_lines.extend_from_slice(&insert_lines);
            new_lines.extend_from_slice(lines_suffix);
            new_lines
        };

        self.lines = new_lines;
        self.clear_selection();
        self.assert_ok_selection();
    }

    /// Return the length in bytes of the current line under the editing point.
    pub fn current_line_length(&self) -> ByteOffset {
        ByteOffset(self.lines[self.edit_point.line].len())
    }

    /// Adjust the editing point position by a given number of lines. The resulting column is
    /// as close to the original column position as possible.
    pub fn adjust_vertical(&mut self, adjust: isize, select: Selection) {
        if !self.multiline {
            return;
        }

        if select == Selection::Selected {
            if self.selection_origin.is_none() {
                self.selection_origin = Some(self.edit_point);
            }
        } else {
            self.clear_selection();
        }

        assert!(self.edit_point.line < self.lines.len());

        let target_line: isize = self.edit_point.line as isize + adjust;

        if target_line < 0 {
            self.edit_point.line = 0;
            self.edit_point.index = ByteOffset::zero();
            if self.selection_origin.is_some() &&
                (self.selection_direction == SelectionDirection::None ||
                    self.selection_direction == SelectionDirection::Forward)
            {
                self.selection_origin = Some(TextPoint {
                    line: 0,
                    index: ByteOffset::zero(),
                });
            }
            return;
        } else if target_line as usize >= self.lines.len() {
            self.edit_point.line = self.lines.len() - 1;
            self.edit_point.index = self.current_line_length();
            if self.selection_origin.is_some() &&
                (self.selection_direction == SelectionDirection::Backward)
            {
                self.selection_origin = Some(self.edit_point);
            }
            return;
        }

        let ByteOffset(edit_index) = self.edit_point.index;
        let col = self.lines[self.edit_point.line][..edit_index]
            .chars()
            .count();
        self.edit_point.line = target_line as usize;
        // NOTE: this adjusts to the nearest complete UTF-8 "char", rather than grapheme cluster
        self.edit_point.index = len_of_first_n_chars(&self.lines[self.edit_point.line], col);
        if let Some(origin) = self.selection_origin {
            if ((self.selection_direction == SelectionDirection::None ||
                self.selection_direction == SelectionDirection::Forward) &&
                self.edit_point <= origin) ||
                (self.selection_direction == SelectionDirection::Backward &&
                    origin <= self.edit_point)
            {
                self.selection_origin = Some(self.edit_point);
            }
        }
        self.assert_ok_selection();
    }

    /// Adjust the editing point position by a given number of bytes. If the adjustment
    /// requested is larger than is available in the current line, the editing point is
    /// adjusted vertically and the process repeats with the remaining adjustment requested.
    pub fn adjust_horizontal(
        &mut self,
        adjust: ByteOffset,
        direction: Direction,
        select: Selection,
    ) {
        if self.adjust_selection_for_horizontal_change(direction, select) {
            return;
        }
        self.perform_horizontal_adjustment(adjust, direction, select);
    }

    /// Adjust the editing point position by exactly one grapheme cluster. If the edit point
    /// is at the beginning of the line and the direction is "Backward" or the edit point is at
    /// the end of the line and the direction is "Forward", a vertical adjustment is made
    pub fn adjust_horizontal_by_one(&mut self, direction: Direction, select: Selection) {
        if self.adjust_selection_for_horizontal_change(direction, select) {
            return;
        }
        let adjust = {
            let current_line = &self.lines[self.edit_point.line];
            let ByteOffset(current_offset) = self.edit_point.index;
            let next_ch = match direction {
                Direction::Forward => current_line[current_offset..].graphemes(true).next(),
                Direction::Backward => current_line[..current_offset].graphemes(true).next_back(),
            };
            match next_ch {
                Some(c) => ByteOffset(c.len() as usize),
                None => ByteOffset::one(), // Going to the next line is a "one byte" offset
            }
        };
        self.perform_horizontal_adjustment(adjust, direction, select);
    }

    /// Return whether to cancel the caret move
    fn adjust_selection_for_horizontal_change(
        &mut self,
        adjust: Direction,
        select: Selection,
    ) -> bool {
        if select == Selection::Selected {
            if self.selection_origin.is_none() {
                self.selection_origin = Some(self.edit_point);
            }
        } else {
            if self.has_selection() {
                self.edit_point = match adjust {
                    Direction::Backward => self.selection_start(),
                    Direction::Forward => self.selection_end(),
                };
                self.clear_selection();
                return true;
            }
        }
        false
    }

    /// Update the field selection_direction.
    ///
    /// When the edit_point (or focus) is before the selection_origin (or anchor)
    /// you have a backward selection. Otherwise you have a forward selection.
    fn update_selection_direction(&mut self) {
        debug!(
            "edit_point: {:?}, selection_origin: {:?}",
            self.edit_point, self.selection_origin
        );
        self.selection_direction = if Some(self.edit_point) < self.selection_origin {
            SelectionDirection::Backward
        } else {
            SelectionDirection::Forward
        }
    }

    fn perform_horizontal_adjustment(
        &mut self,
        adjust: ByteOffset,
        direction: Direction,
        select: Selection,
    ) {
        match direction {
            Direction::Backward => {
                let remaining = self.edit_point.index;
                if adjust > remaining && self.edit_point.line > 0 {
                    self.adjust_vertical(-1, select);
                    self.edit_point.index = self.current_line_length();
                    // one shift is consumed by the change of line, hence the -1
                    self.adjust_horizontal(
                        adjust - remaining - ByteOffset::one(),
                        direction,
                        select,
                    );
                } else {
                    self.edit_point.index = max(ByteOffset::zero(), remaining - adjust);
                }
            },
            Direction::Forward => {
                let remaining = self.current_line_length() - self.edit_point.index;
                if adjust > remaining && self.lines.len() > self.edit_point.line + 1 {
                    self.adjust_vertical(1, select);
                    self.edit_point.index = ByteOffset::zero();
                    // one shift is consumed by the change of line, hence the -1
                    self.adjust_horizontal(
                        adjust - remaining - ByteOffset::one(),
                        direction,
                        select,
                    );
                } else {
                    self.edit_point.index =
                        min(self.current_line_length(), self.edit_point.index + adjust);
                }
            },
        };
        self.update_selection_direction();
        self.assert_ok_selection();
    }

    /// Deal with a newline input.
    pub fn handle_return(&mut self) -> KeyReaction {
        if !self.multiline {
            KeyReaction::TriggerDefaultAction
        } else {
            self.insert_char('\n');
            KeyReaction::DispatchInput
        }
    }

    /// Select all text in the input control.
    pub fn select_all(&mut self) {
        self.selection_origin = Some(TextPoint {
            line: 0,
            index: ByteOffset::zero(),
        });
        let last_line = self.lines.len() - 1;
        self.edit_point.line = last_line;
        self.edit_point.index = ByteOffset(self.lines[last_line].len());
        self.selection_direction = SelectionDirection::Forward;
        self.assert_ok_selection();
    }

    /// Remove the current selection.
    pub fn clear_selection(&mut self) {
        self.selection_origin = None;
        self.selection_direction = SelectionDirection::None;
    }

    /// Remove the current selection and set the edit point to the end of the content.
    pub fn clear_selection_to_limit(&mut self, direction: Direction) {
        self.clear_selection();
        self.adjust_horizontal_to_limit(direction, Selection::NotSelected);
    }

    pub fn adjust_horizontal_by_word(&mut self, direction: Direction, select: Selection) {
        if self.adjust_selection_for_horizontal_change(direction, select) {
            return;
        }
        let shift_increment: ByteOffset = {
            let current_index = self.edit_point.index;
            let current_line = self.edit_point.line;
            let mut newline_adjustment = ByteOffset::zero();
            let mut shift_temp = ByteOffset::zero();
            match direction {
                Direction::Backward => {
                    let input: &str;
                    if current_index == ByteOffset::zero() && current_line > 0 {
                        input = &self.lines[current_line - 1];
                        newline_adjustment = ByteOffset::one();
                    } else {
                        let ByteOffset(remaining) = current_index;
                        input = &self.lines[current_line][..remaining];
                    }

                    let mut iter = input.split_word_bounds().rev();
                    loop {
                        match iter.next() {
                            None => break,
                            Some(x) => {
                                shift_temp += ByteOffset(x.len() as usize);
                                if x.chars().any(|x| x.is_alphabetic() || x.is_numeric()) {
                                    break;
                                }
                            },
                        }
                    }
                },
                Direction::Forward => {
                    let input: &str;
                    let remaining = self.current_line_length() - current_index;
                    if remaining == ByteOffset::zero() &&
                        self.lines.len() > self.edit_point.line + 1
                    {
                        input = &self.lines[current_line + 1];
                        newline_adjustment = ByteOffset::one();
                    } else {
                        let ByteOffset(current_offset) = current_index;
                        input = &self.lines[current_line][current_offset..];
                    }

                    let mut iter = input.split_word_bounds();
                    loop {
                        match iter.next() {
                            None => break,
                            Some(x) => {
                                shift_temp += ByteOffset(x.len() as usize);
                                if x.chars().any(|x| x.is_alphabetic() || x.is_numeric()) {
                                    break;
                                }
                            },
                        }
                    }
                },
            };

            shift_temp + newline_adjustment
        };

        self.adjust_horizontal(shift_increment, direction, select);
    }

    pub fn adjust_horizontal_to_line_end(&mut self, direction: Direction, select: Selection) {
        if self.adjust_selection_for_horizontal_change(direction, select) {
            return;
        }
        let shift: usize = {
            let current_line = &self.lines[self.edit_point.line];
            let ByteOffset(current_offset) = self.edit_point.index;
            match direction {
                Direction::Backward => current_line[..current_offset].len() as usize,
                Direction::Forward => current_line[current_offset..].len() as usize,
            }
        };
        self.perform_horizontal_adjustment(ByteOffset(shift), direction, select);
    }

    pub fn adjust_horizontal_to_limit(&mut self, direction: Direction, select: Selection) {
        if self.adjust_selection_for_horizontal_change(direction, select) {
            return;
        }
        match direction {
            Direction::Backward => {
                self.edit_point.line = 0;
                self.edit_point.index = ByteOffset::zero();
            },
            Direction::Forward => {
                self.edit_point.line = &self.lines.len() - 1;
                self.edit_point.index = ByteOffset((&self.lines[&self.lines.len() - 1]).len());
            },
        }
    }

    /// Process a given `KeyboardEvent` and return an action for the caller to execute.
    pub fn handle_keydown(&mut self, event: &KeyboardEvent) -> KeyReaction {
        let key = event.key();
        let mods = event.modifiers();
        self.handle_keydown_aux(key, mods, cfg!(target_os = "macos"))
    }

    // This function exists for easy unit testing.
    // To test Mac OS shortcuts on other systems a flag is passed.
    pub fn handle_keydown_aux(
        &mut self,
        key: Key,
        mut mods: Modifiers,
        macos: bool,
    ) -> KeyReaction {
        let maybe_select = if mods.contains(Modifiers::SHIFT) {
            Selection::Selected
        } else {
            Selection::NotSelected
        };
        mods.remove(Modifiers::SHIFT);
        ShortcutMatcher::new(KeyState::Down, key.clone(), mods)
            .shortcut(Modifiers::CONTROL | Modifiers::ALT, 'B', || {
                self.adjust_horizontal_by_word(Direction::Backward, maybe_select);
                KeyReaction::RedrawSelection
            })
            .shortcut(Modifiers::CONTROL | Modifiers::ALT, 'F', || {
                self.adjust_horizontal_by_word(Direction::Forward, maybe_select);
                KeyReaction::RedrawSelection
            })
            .shortcut(Modifiers::CONTROL | Modifiers::ALT, 'A', || {
                self.adjust_horizontal_to_line_end(Direction::Backward, maybe_select);
                KeyReaction::RedrawSelection
            })
            .shortcut(Modifiers::CONTROL | Modifiers::ALT, 'E', || {
                self.adjust_horizontal_to_line_end(Direction::Forward, maybe_select);
                KeyReaction::RedrawSelection
            })
            .optional_shortcut(macos, Modifiers::CONTROL, 'A', || {
                self.adjust_horizontal_to_line_end(Direction::Backward, maybe_select);
                KeyReaction::RedrawSelection
            })
            .optional_shortcut(macos, Modifiers::CONTROL, 'E', || {
                self.adjust_horizontal_to_line_end(Direction::Forward, maybe_select);
                KeyReaction::RedrawSelection
            })
            .shortcut(CMD_OR_CONTROL, 'A', || {
                self.select_all();
                KeyReaction::RedrawSelection
            })
            .shortcut(CMD_OR_CONTROL, 'C', || {
                if let Some(text) = self.get_selection_text() {
                    self.clipboard_provider.set_clipboard_contents(text);
                }
                KeyReaction::DispatchInput
            })
            .shortcut(CMD_OR_CONTROL, 'V', || {
                let contents = self.clipboard_provider.clipboard_contents();
                self.insert_string(contents);
                KeyReaction::DispatchInput
            })
            .shortcut(Modifiers::empty(), Key::Delete, || {
                self.delete_char(Direction::Forward);
                KeyReaction::DispatchInput
            })
            .shortcut(Modifiers::empty(), Key::Backspace, || {
                self.delete_char(Direction::Backward);
                KeyReaction::DispatchInput
            })
            .optional_shortcut(macos, Modifiers::META, Key::ArrowLeft, || {
                self.adjust_horizontal_to_line_end(Direction::Backward, maybe_select);
                KeyReaction::RedrawSelection
            })
            .optional_shortcut(macos, Modifiers::META, Key::ArrowRight, || {
                self.adjust_horizontal_to_line_end(Direction::Forward, maybe_select);
                KeyReaction::RedrawSelection
            })
            .optional_shortcut(macos, Modifiers::META, Key::ArrowUp, || {
                self.adjust_horizontal_to_limit(Direction::Backward, maybe_select);
                KeyReaction::RedrawSelection
            })
            .optional_shortcut(macos, Modifiers::META, Key::ArrowDown, || {
                self.adjust_horizontal_to_limit(Direction::Forward, maybe_select);
                KeyReaction::RedrawSelection
            })
            .shortcut(Modifiers::ALT, Key::ArrowLeft, || {
                self.adjust_horizontal_by_word(Direction::Backward, maybe_select);
                KeyReaction::RedrawSelection
            })
            .shortcut(Modifiers::ALT, Key::ArrowRight, || {
                self.adjust_horizontal_by_word(Direction::Forward, maybe_select);
                KeyReaction::RedrawSelection
            })
            .shortcut(Modifiers::empty(), Key::ArrowLeft, || {
                self.adjust_horizontal_by_one(Direction::Backward, maybe_select);
                KeyReaction::RedrawSelection
            })
            .shortcut(Modifiers::empty(), Key::ArrowRight, || {
                self.adjust_horizontal_by_one(Direction::Forward, maybe_select);
                KeyReaction::RedrawSelection
            })
            .shortcut(Modifiers::empty(), Key::ArrowUp, || {
                self.adjust_vertical(-1, maybe_select);
                KeyReaction::RedrawSelection
            })
            .shortcut(Modifiers::empty(), Key::ArrowDown, || {
                self.adjust_vertical(1, maybe_select);
                KeyReaction::RedrawSelection
            })
            .shortcut(Modifiers::empty(), Key::Enter, || self.handle_return())
            .optional_shortcut(macos, Modifiers::empty(), Key::Home, || {
                self.edit_point.index = ByteOffset::zero();
                KeyReaction::RedrawSelection
            })
            .optional_shortcut(macos, Modifiers::empty(), Key::End, || {
                self.edit_point.index = self.current_line_length();
                self.assert_ok_selection();
                KeyReaction::RedrawSelection
            })
            .shortcut(Modifiers::empty(), Key::PageUp, || {
                self.adjust_vertical(-28, maybe_select);
                KeyReaction::RedrawSelection
            })
            .shortcut(Modifiers::empty(), Key::PageDown, || {
                self.adjust_vertical(28, maybe_select);
                KeyReaction::RedrawSelection
            })
            .otherwise(|| {
                if let Key::Character(ref c) = key {
                    self.insert_string(c.as_str());
                    return KeyReaction::DispatchInput;
                }
                KeyReaction::Nothing
            })
            .unwrap()
    }

    pub fn handle_compositionend(&mut self, event: &CompositionEvent) -> KeyReaction {
        self.insert_string(event.data());
        KeyReaction::DispatchInput
    }

    /// Whether the content is empty.
    pub fn is_empty(&self) -> bool {
        self.lines.len() <= 1 && self.lines.get(0).map_or(true, |line| line.is_empty())
    }

    /// The length of the content in bytes.
    pub fn len(&self) -> ByteOffset {
        self.lines.iter().fold(ByteOffset::zero(), |m, l| {
            m + ByteOffset(l.len()) + ByteOffset::one() // + 1 for the '\n'
        }) - ByteOffset::one()
    }

    /// The total number of code units required to encode the content in utf16.
    pub fn utf16_len(&self) -> UTF16CodeUnits {
        self.lines.iter().fold(UTF16CodeUnits::zero(), |m, l| {
            m + UTF16CodeUnits(l.chars().map(char::len_utf16).sum::<usize>() + 1) // + 1 for the '\n'
        }) - UTF16CodeUnits::one()
    }

    /// The length of the content in UTF-8 chars.
    pub fn char_count(&self) -> usize {
        self.lines.iter().fold(0, |m, l| {
            m + l.chars().count() + 1 // + 1 for the '\n'
        }) - 1
    }

    /// Get the current contents of the text input. Multiple lines are joined by \n.
    pub fn get_content(&self) -> DOMString {
        let mut content = "".to_owned();
        for (i, line) in self.lines.iter().enumerate() {
            content.push_str(&line);
            if i < self.lines.len() - 1 {
                content.push('\n');
            }
        }
        DOMString::from(content)
    }

    /// Get a reference to the contents of a single-line text input. Panics if self is a multiline input.
    pub fn single_line_content(&self) -> &DOMString {
        assert!(!self.multiline);
        &self.lines[0]
    }

    /// Set the current contents of the text input. If this is control supports multiple lines,
    /// any \n encountered will be stripped and force a new logical line.
    pub fn set_content(&mut self, content: DOMString) {
        self.lines = if self.multiline {
            // https://html.spec.whatwg.org/multipage/#textarea-line-break-normalisation-transformation
            content
                .replace("\r\n", "\n")
                .split(|c| c == '\n' || c == '\r')
                .map(DOMString::from)
                .collect()
        } else {
            vec![content]
        };

        self.edit_point = self.edit_point.constrain_to(&self.lines);

        if let Some(origin) = self.selection_origin {
            self.selection_origin = Some(origin.constrain_to(&self.lines));
        }
        self.assert_ok_selection();
    }

    /// Convert a TextPoint into a byte offset from the start of the content.
    fn text_point_to_offset(&self, text_point: &TextPoint) -> ByteOffset {
        self.lines
            .iter()
            .enumerate()
            .fold(ByteOffset::zero(), |acc, (i, val)| {
                if i < text_point.line {
                    acc + ByteOffset(val.len()) + ByteOffset::one() // +1 for the \n
                } else {
                    acc
                }
            }) +
            text_point.index
    }

    /// Convert a byte offset from the start of the content into a TextPoint.
    fn offset_to_text_point(&self, abs_point: ByteOffset) -> TextPoint {
        let mut index = abs_point;
        let mut line = 0;
        let last_line_idx = self.lines.len() - 1;
        self.lines
            .iter()
            .enumerate()
            .fold(ByteOffset::zero(), |acc, (i, val)| {
                if i != last_line_idx {
                    let line_end = ByteOffset(val.len());
                    let new_acc = acc + line_end + ByteOffset::one();
                    if abs_point >= new_acc && index > line_end {
                        index -= line_end + ByteOffset::one();
                        line += 1;
                    }
                    new_acc
                } else {
                    acc
                }
            });

        TextPoint {
            line: line,
            index: index,
        }
    }

    pub fn set_selection_range(&mut self, start: u32, end: u32, direction: SelectionDirection) {
        let mut start = ByteOffset(start as usize);
        let mut end = ByteOffset(end as usize);
        let text_end = ByteOffset(self.get_content().len());

        if end > text_end {
            end = text_end;
        }
        if start > end {
            start = end;
        }

        self.selection_direction = direction;

        match direction {
            SelectionDirection::None | SelectionDirection::Forward => {
                self.selection_origin = Some(self.offset_to_text_point(start));
                self.edit_point = self.offset_to_text_point(end);
            },
            SelectionDirection::Backward => {
                self.selection_origin = Some(self.offset_to_text_point(end));
                self.edit_point = self.offset_to_text_point(start);
            },
        }
        self.assert_ok_selection();
    }

    /// Set the edit point index position based off of a given grapheme cluster offset
    pub fn set_edit_point_index(&mut self, index: usize) {
        let byte_offset = self.lines[self.edit_point.line]
            .graphemes(true)
            .take(index)
            .fold(ByteOffset::zero(), |acc, x| acc + ByteOffset(x.len()));
        self.edit_point.index = byte_offset;
    }
}
