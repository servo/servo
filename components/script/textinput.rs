/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

//! Common handling of keyboard input and state management for text input controls

use clipboard_provider::ClipboardProvider;
use dom::keyboardevent::{KeyboardEvent, KeyboardEventHelpers, key_value};
use msg::constellation_msg::{SHIFT, CONTROL, ALT, SUPER};
use msg::constellation_msg::{Key, KeyModifiers};
use util::mem::HeapSizeOf;
use util::str::{DOMString, slice_chars};

use std::borrow::ToOwned;
use std::cmp::{min, max};
use std::default::Default;


#[derive(Copy, Clone, PartialEq)]
pub enum Selection {
    Selected,
    NotSelected
}

#[derive(JSTraceable, Copy, Clone, HeapSizeOf)]
pub struct TextPoint {
    /// 0-based line number
    pub line: usize,
    /// 0-based column number
    pub index: usize,
}

/// Encapsulated state for handling keyboard input in a single or multiline text input control.
#[derive(JSTraceable, HeapSizeOf)]
pub struct TextInput<T: ClipboardProvider> {
    /// Current text input content, split across lines without trailing '\n'
    lines: Vec<DOMString>,
    /// Current cursor input point
    pub edit_point: TextPoint,
    /// Beginning of selection range with edit_point as end that can span multiple lines.
    selection_begin: Option<TextPoint>,
    /// Is this a multiline input?
    multiline: bool,
    #[ignore_heap_size_of = "Can't easily measure this generic type"]
    clipboard_provider: T,
}

/// Resulting action to be taken by the owner of a text input that is handling an event.
pub enum KeyReaction {
    TriggerDefaultAction,
    DispatchInput,
    Nothing,
}

impl Default for TextPoint {
    fn default() -> TextPoint {
        TextPoint {
            line: 0,
            index: 0,
        }
    }
}

/// Control whether this control should allow multiple lines.
#[derive(PartialEq)]
pub enum Lines {
    Single,
    Multiple,
}

/// The direction in which to delete a character.
#[derive(PartialEq)]
pub enum DeleteDir {
    Forward,
    Backward
}


/// Was the keyboard event accompanied by the standard control modifier,
/// i.e. cmd on Mac OS or ctrl on other platforms.
#[cfg(target_os="macos")]
fn is_control_key(mods: KeyModifiers) -> bool {
    mods.contains(SUPER) && !mods.contains(CONTROL | ALT)
}

#[cfg(not(target_os="macos"))]
fn is_control_key(mods: KeyModifiers) -> bool {
    mods.contains(CONTROL) && !mods.contains(SUPER | ALT)
}

fn is_printable_key(key: Key) -> bool {
    match key {
        Key::Space | Key::Apostrophe | Key::Comma | Key::Minus |
        Key::Period | Key::Slash | Key::GraveAccent | Key::Num0 |
        Key::Num1 | Key::Num2 | Key::Num3 | Key::Num4 | Key::Num5 |
        Key::Num6 | Key::Num7 | Key::Num8 | Key::Num9 | Key::Semicolon |
        Key::Equal | Key::A | Key::B | Key::C | Key::D | Key::E | Key::F |
        Key::G | Key::H | Key::I | Key::J | Key::K | Key::L | Key::M | Key::N |
        Key::O | Key::P | Key::Q | Key::R | Key::S | Key::T | Key::U | Key::V |
        Key::W | Key::X | Key::Y | Key::Z | Key::LeftBracket | Key::Backslash |
        Key::RightBracket | Key::Kp0 | Key::Kp1 | Key::Kp2 | Key::Kp3 |
        Key::Kp4 | Key::Kp5 | Key::Kp6 | Key::Kp7 | Key::Kp8 | Key::Kp9 |
        Key::KpDecimal | Key::KpDivide | Key::KpMultiply | Key::KpSubtract |
        Key::KpAdd | Key::KpEqual => true,
        _ => false,
    }
}

impl<T: ClipboardProvider> TextInput<T> {
    /// Instantiate a new text input control
    pub fn new(lines: Lines, initial: DOMString, clipboard_provider: T) -> TextInput<T> {
        let mut i = TextInput {
            lines: vec!(),
            edit_point: Default::default(),
            selection_begin: None,
            multiline: lines == Lines::Multiple,
            clipboard_provider: clipboard_provider
        };
        i.set_content(initial);
        i
    }

    /// Remove a character at the current editing point
    pub fn delete_char(&mut self, dir: DeleteDir) {
        if self.selection_begin.is_none() {
            self.adjust_horizontal(if dir == DeleteDir::Forward {
                1
            } else {
                -1
            }, Selection::Selected);
        }
        self.replace_selection("".to_owned());
    }

    /// Insert a character at the current editing point
    pub fn insert_char(&mut self, ch: char) {
        self.insert_string(ch.to_string());
    }

    /// Insert a string at the current editing point
    fn insert_string<S: Into<String>>(&mut self, s: S) {
        if self.selection_begin.is_none() {
            self.selection_begin = Some(self.edit_point);
        }
        self.replace_selection(s.into());
    }

    pub fn get_sorted_selection(&self) -> Option<(TextPoint, TextPoint)> {
        self.selection_begin.map(|begin| {
            let end = self.edit_point;

            if begin.line < end.line || (begin.line == end.line && begin.index < end.index) {
                (begin, end)
            } else {
                (end, begin)
            }
        })
    }

    pub fn get_selection_text(&self) -> Option<String> {
        self.get_sorted_selection().map(|(begin, end)| {
            if begin.line != end.line {
                let mut s = String::new();
                s.push_str(slice_chars(&self.lines[begin.line], begin.index, self.lines[begin.line].len()));
                for (_, line) in self.lines.iter().enumerate().filter(|&(i,_)| begin.line < i && i < end.line) {
                    s.push_str("\n");
                    s.push_str(line);
                }
                s.push_str("\n");
                s.push_str(slice_chars(&self.lines[end.line], 0, end.index));
                s
            } else {
                slice_chars(&self.lines[begin.line], begin.index, end.index).to_owned()
            }
        })
    }

    pub fn replace_selection(&mut self, insert: String) {
        if let Some((begin, end)) = self.get_sorted_selection() {
            self.clear_selection();

            let new_lines = {
                let prefix = slice_chars(&self.lines[begin.line], 0, begin.index);
                let suffix = slice_chars(&self.lines[end.line], end.index, self.lines[end.line].chars().count());
                let lines_prefix = &self.lines[..begin.line];
                let lines_suffix = &self.lines[end.line + 1..];

                let mut insert_lines = if self.multiline {
                    insert.split('\n').map(|s| s.to_owned()).collect()
                } else {
                    vec!(insert)
                };

                let mut new_line = prefix.to_owned();
                new_line.push_str(&insert_lines[0]);
                insert_lines[0] = new_line;

                let last_insert_lines_index = insert_lines.len() - 1;
                self.edit_point.index = insert_lines[last_insert_lines_index].chars().count();
                self.edit_point.line = begin.line + last_insert_lines_index;

                insert_lines[last_insert_lines_index].push_str(suffix);

                let mut new_lines = vec!();
                new_lines.push_all(lines_prefix);
                new_lines.push_all(&insert_lines);
                new_lines.push_all(lines_suffix);
                new_lines
            };

            self.lines = new_lines;
        }
    }

    /// Return the length of the current line under the editing point.
    pub fn current_line_length(&self) -> usize {
        self.lines[self.edit_point.line].chars().count()
    }

    /// Adjust the editing point position by a given of lines. The resulting column is
    /// as close to the original column position as possible.
    pub fn adjust_vertical(&mut self, adjust: isize, select: Selection) {
        if !self.multiline {
            return;
        }

        if select == Selection::Selected {
            if self.selection_begin.is_none() {
                self.selection_begin = Some(self.edit_point);
            }
        } else {
            self.clear_selection();
        }

        assert!(self.edit_point.line < self.lines.len());

        let target_line: isize = self.edit_point.line as isize + adjust;

        if target_line < 0 {
            self.edit_point.index = 0;
            self.edit_point.line = 0;
            return;
        } else if target_line as usize >= self.lines.len() {
            self.edit_point.line = self.lines.len() - 1;
            self.edit_point.index = self.current_line_length();
            return;
        }

        self.edit_point.line = target_line as usize;
        self.edit_point.index = min(self.current_line_length(), self.edit_point.index);
    }

    /// Adjust the editing point position by a given number of columns. If the adjustment
    /// requested is larger than is available in the current line, the editing point is
    /// adjusted vertically and the process repeats with the remaining adjustment requested.
    pub fn adjust_horizontal(&mut self, adjust: isize, select: Selection) {
        if select == Selection::Selected {
            if self.selection_begin.is_none() {
                self.selection_begin = Some(self.edit_point);
            }
        } else {
            if let Some((begin, end)) = self.get_sorted_selection() {
                self.edit_point = if adjust < 0 {begin} else {end};
                self.clear_selection();
                return
            }
        }

        if adjust < 0 {
            let remaining = self.edit_point.index;
            if adjust.abs() as usize > remaining && self.edit_point.line > 0 {
                self.adjust_vertical(-1, select);
                self.edit_point.index = self.current_line_length();
                self.adjust_horizontal(adjust + remaining as isize + 1, select);
            } else {
                self.edit_point.index = max(0, self.edit_point.index as isize + adjust) as usize;
            }
        } else {
            let remaining = self.current_line_length() - self.edit_point.index;
            if adjust as usize > remaining && self.lines.len() > self.edit_point.line + 1 {
                self.adjust_vertical(1, select);
                self.edit_point.index = 0;
                // one shift is consumed by the change of line, hence the -1
                self.adjust_horizontal(adjust - remaining as isize - 1, select);
            } else {
                self.edit_point.index = min(self.current_line_length(),
                                            self.edit_point.index + adjust as usize);
            }
        }
    }

    /// Deal with a newline input.
    pub fn handle_return(&mut self) -> KeyReaction {
        if !self.multiline {
            return KeyReaction::TriggerDefaultAction;
        }
        self.insert_char('\n');
        return KeyReaction::DispatchInput;
    }

    /// Select all text in the input control.
    pub fn select_all(&mut self) {
        self.selection_begin = Some(TextPoint {
            line: 0,
            index: 0,
        });
        let last_line = self.lines.len() - 1;
        self.edit_point.line = last_line;
        self.edit_point.index = self.lines[last_line].chars().count();
    }

    /// Remove the current selection.
    pub fn clear_selection(&mut self) {
        self.selection_begin = None;
    }

    /// Process a given `KeyboardEvent` and return an action for the caller to execute.
    pub fn handle_keydown(&mut self, event: &KeyboardEvent) -> KeyReaction {
        if let Some(key) = event.get_key() {
            self.handle_keydown_aux(key, event.get_key_modifiers())
        } else {
            KeyReaction::Nothing
        }
    }
    pub fn handle_keydown_aux(&mut self, key: Key, mods: KeyModifiers) -> KeyReaction {
        let maybe_select = if mods.contains(SHIFT) { Selection::Selected } else { Selection::NotSelected };
        match key {
            Key::A if is_control_key(mods) => {
                self.select_all();
                KeyReaction::Nothing
            },
            Key::C if is_control_key(mods) => {
                if let Some(text) = self.get_selection_text() {
                    self.clipboard_provider.set_clipboard_contents(text);
                }
                KeyReaction::DispatchInput
            },
            Key::V if is_control_key(mods) => {
                let contents = self.clipboard_provider.clipboard_contents();
                self.insert_string(contents);
                KeyReaction::DispatchInput
            },
            _ if is_printable_key(key) => {
                self.insert_string(key_value(key, mods));
                KeyReaction::DispatchInput
            }
            Key::Space => {
                self.insert_char(' ');
                KeyReaction::DispatchInput
            }
            Key::Delete => {
                self.delete_char(DeleteDir::Forward);
                KeyReaction::DispatchInput
            }
            Key::Backspace => {
                self.delete_char(DeleteDir::Backward);
                KeyReaction::DispatchInput
            }
            Key::Left => {
                self.adjust_horizontal(-1, maybe_select);
                KeyReaction::Nothing
            }
            Key::Right => {
                self.adjust_horizontal(1, maybe_select);
                KeyReaction::Nothing
            }
            Key::Up => {
                self.adjust_vertical(-1, maybe_select);
                KeyReaction::Nothing
            }
            Key::Down => {
                self.adjust_vertical(1, maybe_select);
                KeyReaction::Nothing
            }
            Key::Enter | Key::KpEnter => self.handle_return(),
            Key::Home => {
                self.edit_point.index = 0;
                KeyReaction::Nothing
            }
            Key::End => {
                self.edit_point.index = self.current_line_length();
                KeyReaction::Nothing
            }
            Key::PageUp => {
                self.adjust_vertical(-28, maybe_select);
                KeyReaction::Nothing
            }
            Key::PageDown => {
                self.adjust_vertical(28, maybe_select);
                KeyReaction::Nothing
            }
            Key::Tab => KeyReaction::TriggerDefaultAction,
            _ => KeyReaction::Nothing,
        }
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
        content
    }

    /// Set the current contents of the text input. If this is control supports multiple lines,
    /// any \n encountered will be stripped and force a new logical line.
    pub fn set_content(&mut self, content: DOMString) {
        self.lines = if self.multiline {
            content.split('\n').map(|s| s.to_owned()).collect()
        } else {
            vec!(content)
        };
        self.edit_point.line = min(self.edit_point.line, self.lines.len() - 1);
        self.edit_point.index = min(self.edit_point.index, self.current_line_length());
    }
}
