/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

//! Common handling of keyboard input and state management for text input controls

use dom::bindings::codegen::Bindings::KeyboardEventBinding::KeyboardEventMethods;
use dom::bindings::js::JSRef;
use msg::constellation_msg::ConstellationChan;
use msg::constellation_msg::Msg as ConstellationMsg;
use dom::keyboardevent::KeyboardEvent;
use util::str::DOMString;

use std::borrow::ToOwned;
use std::cmp::{min, max};
use std::default::Default;
use std::sync::mpsc::channel;

#[derive(Copy, Clone, PartialEq)]
pub enum Selection {
    Selected,
    NotSelected
}

#[jstraceable]
#[derive(Copy, Clone)]
pub struct TextPoint {
    /// 0-based line number
    pub line: usize,
    /// 0-based column number
    pub index: usize,
}

/// Encapsulated state for handling keyboard input in a single or multiline text input control.
#[jstraceable]
pub struct TextInput {
    /// Current text input content, split across lines without trailing '\n'
    lines: Vec<DOMString>,
    /// Current cursor input point
    pub edit_point: TextPoint,
    /// Beginning of selection range with edit_point as end that can span multiple lines.
    selection_begin: Option<TextPoint>,
    /// Is this a multiline input?
    multiline: bool,
    constellation_channel: Option<ConstellationChan>
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
fn is_control_key(event: JSRef<KeyboardEvent>) -> bool {
    event.MetaKey() && !event.CtrlKey() && !event.AltKey()
}

#[cfg(not(target_os="macos"))]
fn is_control_key(event: JSRef<KeyboardEvent>) -> bool {
    event.CtrlKey() && !event.MetaKey() && !event.AltKey()
}

impl TextInput {
    /// Instantiate a new text input control
    pub fn new(lines: Lines, initial: DOMString, cc: Option<ConstellationChan>) -> TextInput {
        let mut i = TextInput {
            lines: vec!(),
            edit_point: Default::default(),
            selection_begin: None,
            multiline: lines == Lines::Multiple,
            constellation_channel: cc,
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
        if self.selection_begin.is_none() {
            self.selection_begin = Some(self.edit_point);
        }
        self.replace_selection(ch.to_string());
    }

    /// Insert a string at the current editing point
    fn insert_string(&mut self, s: &str) {
        // it looks like this could be made performant by avoiding some redundant
        //  selection-related checks, but use the simple implementation for now
        for ch in s.chars() {
            self.insert_char(ch);
        }
    }

    pub fn get_sorted_selection(&self) -> (TextPoint, TextPoint) {
        let begin = self.selection_begin.unwrap();
        let end = self.edit_point;

        if begin.line < end.line || (begin.line == end.line && begin.index < end.index) {
            (begin, end)
        } else {
            (end, begin)
        }
    }

    pub fn replace_selection(&mut self, insert: String) {
        let (begin, end) = self.get_sorted_selection();
        self.clear_selection();

        let new_lines = {
            let prefix = self.lines[begin.line].slice_chars(0, begin.index);
            let suffix = self.lines[end.line].slice_chars(end.index, self.lines[end.line].chars().count());
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
            if self.selection_begin.is_some() {
                let (begin, end) = self.get_sorted_selection();
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
    pub fn handle_keydown(&mut self, event: JSRef<KeyboardEvent>) -> KeyReaction {
        //A simple way to convert an event to a selection
        fn maybe_select(event: JSRef<KeyboardEvent>) -> Selection {
            if event.ShiftKey() {
                return Selection::Selected
            }
            return Selection::NotSelected
        }
        match &*event.Key() {
           "a" if is_control_key(event) => {
                self.select_all();
                KeyReaction::Nothing
            },
            "v" if is_control_key(event) => {
                let (tx, rx) = channel();
                let mut contents = None;
                if let Some(ref cc) = self.constellation_channel {
                    cc.0.send(ConstellationMsg::GetClipboardContents(tx)).unwrap();
                    contents = Some(rx.recv().unwrap());
                }
                if let Some(contents) = contents {
                    self.insert_string(&contents);
                }
                KeyReaction::DispatchInput
            },
            // printable characters have single-character key values
            c if c.len() == 1 => {
                self.insert_char(c.chars().next().unwrap());
                KeyReaction::DispatchInput
            }
            "Space" => {
                self.insert_char(' ');
                KeyReaction::DispatchInput
            }
            "Delete" => {
                self.delete_char(DeleteDir::Forward);
                KeyReaction::DispatchInput
            }
            "Backspace" => {
                self.delete_char(DeleteDir::Backward);
                KeyReaction::DispatchInput
            }
            "ArrowLeft" => {
                self.adjust_horizontal(-1, maybe_select(event));
                KeyReaction::Nothing
            }
            "ArrowRight" => {
                self.adjust_horizontal(1, maybe_select(event));
                KeyReaction::Nothing
            }
            "ArrowUp" => {
                self.adjust_vertical(-1, maybe_select(event));
                KeyReaction::Nothing
            }
            "ArrowDown" => {
                self.adjust_vertical(1, maybe_select(event));
                KeyReaction::Nothing
            }
            "Enter" => self.handle_return(),
            "Home" => {
                self.edit_point.index = 0;
                KeyReaction::Nothing
            }
            "End" => {
                self.edit_point.index = self.current_line_length();
                KeyReaction::Nothing
            }
            "PageUp" => {
                self.adjust_vertical(-28, maybe_select(event));
                KeyReaction::Nothing
            }
            "PageDown" => {
                self.adjust_vertical(28, maybe_select(event));
                KeyReaction::Nothing
            }
            "Tab" => KeyReaction::TriggerDefaultAction,
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
