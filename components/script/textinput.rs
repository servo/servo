/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

//! Common handling of keyboard input and state management for text input controls

use std::borrow::ToOwned;
use std::cmp::min;
use std::default::Default;
use std::ops::{Add, AddAssign, Range};

use keyboard_types::{Key, KeyState, Modifiers, ShortcutMatcher};
use unicode_segmentation::UnicodeSegmentation;
use utf16string::{Utf16Str, Utf16String, utf16};

use crate::clipboard_provider::{ClipboardProvider, EmbedderClipboardProvider};
use crate::dom::bindings::cell::DomRefCell;
use crate::dom::bindings::codegen::Bindings::EventBinding::Event_Binding::EventMethods;
use crate::dom::bindings::inheritance::Castable;
use crate::dom::bindings::str::DOMString;
use crate::dom::compositionevent::CompositionEvent;
use crate::dom::event::Event;
use crate::dom::keyboardevent::KeyboardEvent;
use crate::dom::node::NodeTraits;
use crate::dom::types::ClipboardEvent;
use crate::drag_data_store::{DragDataStore, Kind};
use crate::script_runtime::CanGc;

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

#[derive(Clone, Copy, Debug, JSTraceable, MallocSizeOf, PartialEq, PartialOrd)]
pub struct UTF16CodeUnits(pub usize);

impl UTF16CodeUnits {
    pub fn zero() -> UTF16CodeUnits {
        UTF16CodeUnits(0)
    }

    pub fn one() -> UTF16CodeUnits {
        UTF16CodeUnits(1)
    }

    pub(crate) fn saturating_sub(self, other: UTF16CodeUnits) -> UTF16CodeUnits {
        if self > other {
            UTF16CodeUnits(self.0 - other.0)
        } else {
            UTF16CodeUnits::zero()
        }
    }
}

impl Add for UTF16CodeUnits {
    type Output = UTF16CodeUnits;

    fn add(self, other: UTF16CodeUnits) -> UTF16CodeUnits {
        UTF16CodeUnits(self.0 + other.0)
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

#[derive(Clone, Copy, Debug, Default, JSTraceable, MallocSizeOf, PartialEq, PartialOrd)]
pub struct TextPoint {
    /// 0-based line number
    pub line: usize,
    /// 0-based column number in bytes
    pub index: usize,
}

impl TextPoint {
    /// Returns a TextPoint constrained to be a valid location within lines
    fn constrain_to(&self, lines: &[Utf16String]) -> TextPoint {
        let line = min(self.line, lines.len() - 1);

        TextPoint {
            line,
            index: min(self.index, lines[line].len()),
        }
    }
}

#[derive(Clone, Copy, PartialEq)]
pub(crate) struct SelectionState {
    start: TextPoint,
    end: TextPoint,
    direction: SelectionDirection,
}

/// Encapsulated state for handling keyboard input in a single or multiline text input control.
#[derive(JSTraceable, MallocSizeOf)]
pub struct TextInput<T: ClipboardProvider> {
    /// Current text input content, split across lines without trailing '\n'
    #[ignore_malloc_size_of = "FIXME"]
    #[no_trace]
    lines: Vec<Utf16String>,

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

    /// Was last change made by set_content?
    was_last_change_by_set_content: bool,
}

/// Resulting action to be taken by the owner of a text input that is handling an event.
pub enum KeyReaction {
    TriggerDefaultAction,
    DispatchInput,
    RedrawSelection,
    Nothing,
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
pub(crate) const CMD_OR_CONTROL: Modifiers = Modifiers::META;
#[cfg(not(target_os = "macos"))]
pub(crate) const CMD_OR_CONTROL: Modifiers = Modifiers::CONTROL;

/// The length in bytes of the first n characters in a UTF-8 string.
///
/// If the string has fewer than n characters, returns the length of the whole string.
/// If n is 0, returns 0
fn len_of_first_n_chars(text: &Utf16Str, n: usize) -> usize {
    match text.char_indices().take(n).last() {
        Some((index, ch)) => index + ch.len_utf16(),
        None => 0,
    }
}

/// The length in bytes of the first n code units in a string when encoded in UTF-16.
///
/// If the string is fewer than n code units, returns the length of the whole string.
fn len_of_first_n_code_units(text: &Utf16Str, n: usize) -> usize {
    text.char_indices()
        .nth(n)
        .map(|(position, _)| position)
        .unwrap_or(text.len())
}

impl<T: ClipboardProvider> TextInput<T> {
    /// Instantiate a new text input control
    pub fn new(
        lines: Lines,
        initial: Utf16String,
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
            clipboard_provider,
            max_length,
            min_length,
            selection_direction,
            was_last_change_by_set_content: true,
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

    pub(crate) fn set_max_length(&mut self, length: Option<UTF16CodeUnits>) {
        self.max_length = length;
    }

    pub(crate) fn set_min_length(&mut self, length: Option<UTF16CodeUnits>) {
        self.min_length = length;
    }

    /// Was last edit made by set_content?
    pub(crate) fn was_last_change_by_set_content(&self) -> bool {
        self.was_last_change_by_set_content
    }

    /// Remove a character at the current editing point.
    ///
    /// Returns true if any character was deleted.
    pub fn delete_char(&mut self, dir: Direction) -> bool {
        if self.selection_origin.is_none() || self.selection_origin == Some(self.edit_point) {
            self.adjust_horizontal_by_one(dir, Selection::Selected);
        }
        if self.selection_start() == self.selection_end() {
            false
        } else {
            self.replace_selection(Default::default());
            true
        }
    }

    /// Insert a character at the current editing point
    pub fn insert_char(&mut self, ch: char) {
        self.insert_string(Utf16String::from(ch));
    }

    /// Insert a string at the current editing point
    pub fn insert_string(&mut self, to_insert: Utf16String) {
        if self.selection_origin.is_none() {
            self.selection_origin = Some(self.edit_point);
        }
        self.replace_selection(to_insert);
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
    pub fn selection_start_offset(&self) -> usize {
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
    pub fn selection_end_offset(&self) -> usize {
        self.text_point_to_offset(&self.selection_end())
    }

    /// Whether or not there is an active selection (the selection may be zero-length)
    #[inline]
    pub(crate) fn has_selection(&self) -> bool {
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
    pub(crate) fn sorted_selection_offsets_range(&self) -> Range<usize> {
        self.selection_start_offset()..self.selection_end_offset()
    }

    /// The state of the current selection. Can be used to compare whether selection state has changed.
    pub(crate) fn selection_state(&self) -> SelectionState {
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
            debug_assert!(begin.line < self.lines.len());
            debug_assert!(begin.index <= self.lines[begin.line].len());

            match self.selection_direction {
                SelectionDirection::None | SelectionDirection::Forward => {
                    debug_assert!(begin <= self.edit_point)
                },

                SelectionDirection::Backward => debug_assert!(self.edit_point <= begin),
            }
        }

        debug_assert!(self.edit_point.line < self.lines.len());
        debug_assert!(self.edit_point.index <= self.lines[self.edit_point.line].len());
    }

    pub(crate) fn get_selection_text(&self) -> Option<Utf16String> {
        let text =
            self.fold_selection_slices(Utf16String::new(), |s, slice| s.push_utf16_str(slice));
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
    fn fold_selection_slices<B, F: FnMut(&mut B, &Utf16Str)>(&self, mut acc: B, mut f: F) -> B {
        if self.has_selection() {
            let (start, end) = self.sorted_selection_bounds();
            let start_byte_offset = start.index;
            let end_byte_offset = end.index;

            if start.line == end.line {
                f(
                    &mut acc,
                    &self.lines[start.line][start_byte_offset..end_byte_offset],
                )
            } else {
                f(&mut acc, &self.lines[start.line][start_byte_offset..]);
                for line in &self.lines[start.line + 1..end.line] {
                    f(&mut acc, utf16!("\n"));
                    f(&mut acc, line);
                }
                f(&mut acc, utf16!("\n"));
                f(&mut acc, &self.lines[end.line][..end_byte_offset])
            }
        }

        acc
    }

    pub fn replace_selection(&mut self, insert: Utf16String) {
        if !self.has_selection() {
            return;
        }

        let allowed_to_insert_count = if let Some(max_length) = self.max_length {
            let len_after_selection_replaced =
                self.utf16_len().saturating_sub(self.selection_utf16_len());
            max_length.saturating_sub(len_after_selection_replaced)
        } else {
            UTF16CodeUnits(usize::MAX)
        };

        let last_char_byte_index = len_of_first_n_code_units(&insert, allowed_to_insert_count.0);
        let to_insert = &insert[..last_char_byte_index];

        let (start, end) = self.sorted_selection_bounds();
        let start_byte_offset = start.index;
        let end_byte_offset = end.index;

        let new_lines = {
            let prefix = &self.lines[start.line][..start_byte_offset];
            let suffix = &self.lines[end.line][end_byte_offset..];
            let lines_prefix = &self.lines[..start.line];
            let lines_suffix = &self.lines[end.line + 1..];

            let mut insert_lines = if self.multiline {
                to_insert.split('\n').map(Utf16String::from).collect()
            } else {
                vec![Utf16String::from(to_insert)]
            };

            let mut new_line: Utf16String = prefix.to_owned();

            new_line.push_utf16_str(&insert_lines[0]);
            insert_lines[0] = Utf16String::from(new_line);

            let last_insert_lines_index = insert_lines.len() - 1;
            self.edit_point.index = insert_lines[last_insert_lines_index].len();
            self.edit_point.line = start.line + last_insert_lines_index;

            // FIXME(ajeffrey): efficient append for DOMStrings
            insert_lines[last_insert_lines_index].push_utf16_str(suffix);

            let mut new_lines = vec![];
            new_lines.extend_from_slice(lines_prefix);
            new_lines.extend_from_slice(&insert_lines);
            new_lines.extend_from_slice(lines_suffix);
            new_lines
        };

        self.lines = new_lines;
        self.was_last_change_by_set_content = false;
        self.clear_selection();
        self.assert_ok_selection();
    }

    /// Return the length in bytes of the current line under the editing point.
    pub fn current_line_length(&self) -> usize {
        self.lines[self.edit_point.line].len()
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
            self.edit_point.index = 0;
            if self.selection_origin.is_some() &&
                (self.selection_direction == SelectionDirection::None ||
                    self.selection_direction == SelectionDirection::Forward)
            {
                self.selection_origin = Some(TextPoint { line: 0, index: 0 });
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

        let col = self.lines[self.edit_point.line][..self.edit_point.index]
            .chars()
            .count();
        self.edit_point.line = target_line as usize;
        // NOTE: this adjusts to the nearest complete Unicode codepoint, rather than grapheme cluster
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
        adjust_by_bytes: usize,
        direction: Direction,
        select: Selection,
    ) {
        if self.adjust_selection_for_horizontal_change(direction, select) {
            return;
        }
        self.perform_horizontal_adjustment(adjust_by_bytes, direction, select);
    }

    /// Adjust the editing point position by exactly one grapheme cluster. If the edit point
    /// is at the beginning of the line and the direction is "Backward" or the edit point is at
    /// the end of the line and the direction is "Forward", a vertical adjustment is made
    pub fn adjust_horizontal_by_one(&mut self, direction: Direction, select: Selection) {
        if self.adjust_selection_for_horizontal_change(direction, select) {
            return;
        }
        let adjust_by_bytes = {
            // FIXME: It would be really nice to not have to convert to utf8 here, but the
            // unicode_segmentation crate understandably only concerns itself with utf8.
            let current_line = &self.lines[self.edit_point.line].to_utf8();

            let next_grapheme = match direction {
                Direction::Forward => current_line[self.edit_point.index..].graphemes(true).next(),
                Direction::Backward => current_line[..self.edit_point.index]
                    .graphemes(true)
                    .next_back(),
            };
            match next_grapheme {
                Some(grapheme) => grapheme
                    .chars()
                    .fold(0, |accumulator, c| accumulator + c.len_utf16()),
                None => 1, // Going to the next line is a "one byte" offset
            }
        };
        self.perform_horizontal_adjustment(adjust_by_bytes, direction, select);
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
        } else if self.has_selection() {
            self.edit_point = match adjust {
                Direction::Backward => self.selection_start(),
                Direction::Forward => self.selection_end(),
            };
            self.clear_selection();
            return true;
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
        adjust_by_bytes: usize,
        direction: Direction,
        select: Selection,
    ) {
        match direction {
            Direction::Backward => {
                let remaining_bytes = self.edit_point.index;
                if adjust_by_bytes > remaining_bytes && self.edit_point.line > 0 {
                    // Preserve the current selection origin because `adjust_vertical`
                    // modifies `selection_origin`. Since we are moving backward instead of
                    // highlighting vertically, we need to restore it after adjusting the line.
                    let selection_origin_temp = self.selection_origin;
                    self.adjust_vertical(-1, select);
                    self.edit_point.index = self.current_line_length();
                    // Restore the original selection origin to maintain expected behavior.
                    self.selection_origin = selection_origin_temp;
                    // one shift is consumed by the change of line, hence the -1
                    self.adjust_horizontal(
                        adjust_by_bytes.saturating_sub(remaining_bytes + 1),
                        direction,
                        select,
                    );
                } else {
                    self.edit_point.index = remaining_bytes.saturating_sub(adjust_by_bytes);
                }
            },
            Direction::Forward => {
                let remaining = self
                    .current_line_length()
                    .saturating_sub(self.edit_point.index);
                if adjust_by_bytes > remaining && self.lines.len() > self.edit_point.line + 1 {
                    self.adjust_vertical(1, select);
                    self.edit_point.index = 0;
                    // one shift is consumed by the change of line, hence the -1
                    self.adjust_horizontal(
                        adjust_by_bytes.saturating_sub(remaining + 1),
                        direction,
                        select,
                    );
                } else {
                    self.edit_point.index = min(
                        self.current_line_length(),
                        self.edit_point.index + adjust_by_bytes,
                    );
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
        self.selection_origin = Some(TextPoint { line: 0, index: 0 });
        let last_line = self.lines.len() - 1;
        self.edit_point.line = last_line;
        self.edit_point.index = self.lines[last_line].len();
        self.selection_direction = SelectionDirection::Forward;
        self.assert_ok_selection();
    }

    /// Remove the current selection.
    pub fn clear_selection(&mut self) {
        self.selection_origin = None;
        self.selection_direction = SelectionDirection::None;
    }

    /// Remove the current selection and set the edit point to the end of the content.
    pub(crate) fn clear_selection_to_limit(&mut self, direction: Direction) {
        self.clear_selection();
        self.adjust_horizontal_to_limit(direction, Selection::NotSelected);
    }

    pub fn adjust_horizontal_by_word(&mut self, direction: Direction, select: Selection) {
        if self.adjust_selection_for_horizontal_change(direction, select) {
            return;
        }

        // FIXME: It would be really nice to not have to convert to utf8 here, but the
        // unicode_segmentation crate understandably only concerns itself with utf8.
        let shift_increment = {
            let current_index = self.edit_point.index;
            let current_line = self.edit_point.line;
            let mut newline_adjustment = 0;
            let mut shift_temp: usize = 0;
            match direction {
                Direction::Backward => {
                    let input: String;
                    if current_index == 0 && current_line > 0 {
                        input = self.lines[current_line - 1].to_utf8();
                        newline_adjustment = 1;
                    } else {
                        input = self.lines[current_line][..current_index].to_utf8();
                    }

                    let mut iter = input.split_word_bounds().rev();
                    loop {
                        match iter.next() {
                            None => break,
                            Some(x) => {
                                shift_temp += x.chars().map(|c| c.len_utf16()).sum::<usize>();
                                if x.chars().any(|x| x.is_alphabetic() || x.is_numeric()) {
                                    break;
                                }
                            },
                        }
                    }
                },
                Direction::Forward => {
                    let input: String;
                    let remaining = self.current_line_length().saturating_sub(current_index);
                    if remaining == 0 && self.lines.len() > self.edit_point.line + 1 {
                        input = self.lines[current_line + 1].to_utf8();
                        newline_adjustment = 1;
                    } else {
                        input = self.lines[current_line][current_index..].to_utf8();
                    }

                    let mut iter = input.split_word_bounds();
                    loop {
                        match iter.next() {
                            None => break,
                            Some(x) => {
                                shift_temp += x.len();
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
        let shift_by_bytes: usize = {
            let current_line = &self.lines[self.edit_point.line];
            match direction {
                Direction::Backward => current_line[..self.edit_point.index].len(),
                Direction::Forward => current_line[self.edit_point.index..].len(),
            }
        };
        self.perform_horizontal_adjustment(shift_by_bytes, direction, select);
    }

    pub(crate) fn adjust_horizontal_to_limit(&mut self, direction: Direction, select: Selection) {
        if self.adjust_selection_for_horizontal_change(direction, select) {
            return;
        }
        match direction {
            Direction::Backward => {
                self.edit_point.line = 0;
                self.edit_point.index = 0;
            },
            Direction::Forward => {
                self.edit_point.line = &self.lines.len() - 1;
                self.edit_point.index = (self.lines[&self.lines.len() - 1]).len();
            },
        }
    }

    /// Process a given `KeyboardEvent` and return an action for the caller to execute.
    pub(crate) fn handle_keydown(&mut self, event: &KeyboardEvent) -> KeyReaction {
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
            .shortcut(CMD_OR_CONTROL, 'X', || {
                // FIXME: this is unreachable because ClipboardEvent is fired instead of keydown
                if let Some(text) = self.get_selection_text() {
                    self.clipboard_provider.set_text(text.to_utf8());
                    self.delete_char(Direction::Backward);
                }
                KeyReaction::DispatchInput
            })
            .shortcut(CMD_OR_CONTROL, 'C', || {
                if let Some(text) = self.get_selection_text() {
                    self.clipboard_provider.set_text(text.to_utf8());
                }
                KeyReaction::DispatchInput
            })
            .shortcut(CMD_OR_CONTROL, 'V', || {
                if let Ok(text_content) = self.clipboard_provider.get_text() {
                    self.insert_string(Utf16String::from(text_content));
                }
                KeyReaction::DispatchInput
            })
            .shortcut(Modifiers::empty(), Key::Delete, || {
                if self.delete_char(Direction::Forward) {
                    KeyReaction::DispatchInput
                } else {
                    KeyReaction::Nothing
                }
            })
            .shortcut(Modifiers::empty(), Key::Backspace, || {
                if self.delete_char(Direction::Backward) {
                    KeyReaction::DispatchInput
                } else {
                    KeyReaction::Nothing
                }
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
                self.edit_point.index = 0;
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
                    self.insert_string(c.into());
                    return KeyReaction::DispatchInput;
                }
                if matches!(key, Key::Process) {
                    return KeyReaction::DispatchInput;
                }
                KeyReaction::Nothing
            })
            .unwrap()
    }

    pub(crate) fn handle_compositionend(&mut self, event: &CompositionEvent) -> KeyReaction {
        self.insert_string(Utf16String::from(event.data()));
        KeyReaction::DispatchInput
    }

    pub(crate) fn handle_compositionupdate(&mut self, event: &CompositionEvent) -> KeyReaction {
        let start = self.selection_start_offset();
        self.insert_string(event.data().into());
        self.set_selection_range(
            start as u32,
            (start + event.data().len()) as u32,
            SelectionDirection::Forward,
        );
        KeyReaction::DispatchInput
    }

    /// Whether the content is empty.
    pub(crate) fn is_empty(&self) -> bool {
        self.lines.len() <= 1 && self.lines.first().is_none_or(|line| line.is_empty())
    }

    /// The length of the content in bytes.
    pub(crate) fn len(&self) -> usize {
        self.lines
            .iter()
            .fold(0, |m, l| {
                m + l.len() + 1 // + 1 for the '\n'
            })
            .saturating_sub(1)
    }

    /// The total number of code units required to encode the content in utf16.
    pub(crate) fn utf16_len(&self) -> UTF16CodeUnits {
        self.lines
            .iter()
            .fold(UTF16CodeUnits::zero(), |acc, l| {
                acc + UTF16CodeUnits(l.number_of_code_units() + 1)
                // + 1 for the '\n'
            })
            .saturating_sub(UTF16CodeUnits::one())
    }

    /// The length of the content in Unicode code points.
    pub(crate) fn char_count(&self) -> usize {
        self.lines.iter().fold(0, |m, l| {
            m + l.chars().count() + 1 // + 1 for the '\n'
        }) - 1
    }

    /// Get the current contents of the text input. Multiple lines are joined by \n.
    pub fn get_content(&self) -> Utf16String {
        let mut content = Utf16String::default();
        for (i, line) in self.lines.iter().enumerate() {
            content.push_utf16_str(line);
            if i < self.lines.len() - 1 {
                content.push('\n');
            }
        }
        content
    }

    /// Get a reference to the contents of a single-line text input. Panics if self is a multiline input.
    pub(crate) fn single_line_content(&self) -> &Utf16String {
        assert!(!self.multiline);
        &self.lines[0]
    }

    /// Set the current contents of the text input. If this is control supports multiple lines,
    /// any \n encountered will be stripped and force a new logical line.
    pub fn set_content(&mut self, content: Utf16String) {
        self.lines = if self.multiline {
            // https://html.spec.whatwg.org/multipage/#textarea-line-break-normalisation-transformation
            content
                .replace(utf16!("\r\n"), utf16!("\n"))
                .split('\n')
                .map(Utf16String::from)
                .collect()
        } else {
            vec![content]
        };

        self.was_last_change_by_set_content = true;
        self.edit_point = self.edit_point.constrain_to(&self.lines);

        if let Some(origin) = self.selection_origin {
            self.selection_origin = Some(origin.constrain_to(&self.lines));
        }
        self.assert_ok_selection();
    }

    /// Convert a TextPoint into a byte offset from the start of the content.
    fn text_point_to_offset(&self, text_point: &TextPoint) -> usize {
        self.lines.iter().enumerate().fold(0, |acc, (i, val)| {
            if i < text_point.line {
                acc + val.len() + 1 // +1 for the \n
            } else {
                acc
            }
        }) + text_point.index
    }

    /// Convert a byte offset from the start of the content into a TextPoint.
    fn offset_to_text_point(&self, byte_offset: usize) -> TextPoint {
        let mut index = byte_offset;
        let mut line = 0;
        let last_line_idx = self.lines.len() - 1;
        self.lines.iter().enumerate().fold(0, |acc, (i, val)| {
            if i != last_line_idx {
                let line_end = val.len();
                let new_acc = acc + line_end + 1;
                if byte_offset >= new_acc && index > line_end {
                    index = index.saturating_sub(line_end + 1);
                    line += 1;
                }
                new_acc
            } else {
                acc
            }
        });

        TextPoint { line, index }
    }

    /// Set the selection given start and end indices, in utf16 code units.
    pub fn set_selection_range(&mut self, start: u32, end: u32, direction: SelectionDirection) {
        // Multiply by two to convert from code unit offsets to byte offsets
        let mut start = start as usize * 2;
        let mut end = end as usize * 2;
        let text_end = self.get_content().len();

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
        // FIXME: It would be really nice to not have to convert to utf8 here, but the
        // unicode_segmentation crate understandably only concerns itself with utf8.
        let byte_offset = self.lines[self.edit_point.line]
            .to_utf8()
            .graphemes(true)
            .take(index)
            .map(|grapheme| grapheme.chars().map(|c| c.len_utf16()).sum::<usize>())
            .sum();
        self.edit_point.index = byte_offset;
    }

    fn paste_contents(&mut self, drag_data_store: &DragDataStore) {
        for item in drag_data_store.iter_item_list() {
            if let Kind::Text { data, .. } = item {
                self.insert_string(data.str().into());
            }
        }
    }
}

/// <https://www.w3.org/TR/clipboard-apis/#clipboard-actions> step 3
pub(crate) fn handle_text_clipboard_action(
    owning_node: &impl NodeTraits,
    textinput: &DomRefCell<TextInput<EmbedderClipboardProvider>>,
    event: &ClipboardEvent,
    can_gc: CanGc,
) -> bool {
    let e = event.upcast::<Event>();

    if !e.IsTrusted() {
        return false;
    }

    // Step 3
    match e.Type().str() {
        "copy" => {
            let selection = textinput.borrow().get_selection_text();

            // Step 3.1 Copy the selected contents, if any, to the clipboard
            if let Some(text) = selection {
                textinput
                    .borrow_mut()
                    .clipboard_provider
                    .set_text(text.to_utf8());
            }

            // Step 3.2 Fire a clipboard event named clipboardchange
            owning_node
                .owner_document()
                .fire_clipboardchange_event(can_gc);
        },
        "cut" => {
            let selection = textinput.borrow().get_selection_text();

            // Step 3.1 If there is a selection in an editable context where cutting is enabled, then
            if let Some(text) = selection {
                // Step 3.1.1 Copy the selected contents, if any, to the clipboard
                textinput
                    .borrow_mut()
                    .clipboard_provider
                    .set_text(text.to_utf8());

                // Step 3.1.2 Remove the contents of the selection from the document and collapse the selection.
                textinput.borrow_mut().delete_char(Direction::Backward);

                // Step 3.1.3 Fire a clipboard event named clipboardchange
                owning_node
                    .owner_document()
                    .fire_clipboardchange_event(can_gc);

                // Step 3.1.4 Queue tasks to fire any events that should fire due to the modification.
            } else {
                // Step 3.2 Else, if there is no selection or the context is not editable, then
                return false;
            }
        },
        "paste" => {
            // Step 3.1 If there is a selection or cursor in an editable context where pasting is enabled, then
            if let Some(data) = event.get_clipboard_data() {
                // Step 3.1.1 Insert the most suitable content found on the clipboard, if any, into the context.
                let drag_data_store = data.data_store().expect("This shouldn't fail");
                textinput.borrow_mut().paste_contents(&drag_data_store);

                // Step 3.1.2 Queue tasks to fire any events that should fire due to the modification.
            } else {
                // Step 3.2 Else return false.
                return false;
            }
        },
        _ => (),
    }

    //Step 5
    true
}
