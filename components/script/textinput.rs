/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

//! Common handling of keyboard input and state management for text input controls

use dom::bindings::codegen::Bindings::KeyboardEventBinding::KeyboardEventMethods;
use dom::bindings::js::JSRef;
use dom::keyboardevent::KeyboardEvent;
use servo_util::str::DOMString;

use std::cmp::{min, max};
use std::default::Default;

#[jstraceable]
struct TextPoint {
    /// 0-based line number
    line: uint,
    /// 0-based column number
    index: uint,
}

/// Encapsulated state for handling keyboard input in a single or multiline text input control.
#[jstraceable]
pub struct TextInput {
    /// Current text input content, split across lines without trailing '\n'
    lines: Vec<DOMString>,
    /// Current cursor input point
    edit_point: TextPoint,
    /// Selection range, beginning and end point that can span multiple lines.
    _selection: Option<(TextPoint, TextPoint)>,
    /// Is this a multiline input?
    multiline: bool,
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
#[deriving(PartialEq)]
pub enum Lines {
    Single,
    Multiple,
}

/// The direction in which to delete a character.
#[deriving(PartialEq)]
enum DeleteDir {
    Forward,
    Backward
}

impl TextInput {
    /// Instantiate a new text input control
    pub fn new(lines: Lines, initial: DOMString) -> TextInput {
        let mut i = TextInput {
            lines: vec!(),
            edit_point: Default::default(),
            _selection: None,
            multiline: lines == Multiple,
        };
        i.set_content(initial);
        i
    }

    /// Return the current line under the editing point
    fn get_current_line(&self) -> &DOMString {
        &self.lines[self.edit_point.line]
    }

    /// Insert a character at the current editing point
    fn insert_char(&mut self, ch: char) {
        //TODO: handle replacing selection with character
        let new_line = {
            let prefix = self.get_current_line().as_slice().slice_chars(0, self.edit_point.index);
            let suffix = self.get_current_line().as_slice().slice_chars(self.edit_point.index,
                                                                        self.current_line_length());
            let mut new_line = prefix.to_string();
            new_line.push(ch);
            new_line.push_str(suffix.as_slice());
            new_line
        };

        self.lines[self.edit_point.line] = new_line;
        self.edit_point.index += 1;
    }

    /// Remove a character at the current editing point
    fn delete_char(&mut self, dir: DeleteDir) {
        let forward = dir == Forward;

        //TODO: handle deleting selection
        let prefix_end = if forward {
            self.edit_point.index
        } else {
            if self.multiline {
                //TODO: handle backspacing from position 0 of current line
                if self.edit_point.index == 0 {
                    return;
                }
            } else if self.edit_point.index == 0 {
                return;
            }
            self.edit_point.index - 1
        };
        let suffix_start = if forward {
            let is_eol = self.edit_point.index == self.current_line_length();
            if self.multiline {
                //TODO: handle deleting from end position of current line
                if is_eol {
                    return;
                }
            } else if is_eol {
                return;
            }
            self.edit_point.index + 1
        } else {
            self.edit_point.index
        };

        let new_line = {
            let prefix = self.get_current_line().as_slice().slice_chars(0, prefix_end);
            let suffix = self.get_current_line().as_slice().slice_chars(suffix_start,
                                                                        self.current_line_length());
            let mut new_line = prefix.to_string();
            new_line.push_str(suffix);
            new_line
        };

        self.lines[self.edit_point.line] = new_line;

        if !forward {
            self.adjust_horizontal(-1);
        }
    }

    /// Return the length of the current line under the editing point.
    fn current_line_length(&self) -> uint {
        self.lines[self.edit_point.line].char_len()
    }

    /// Adjust the editing point position by a given of lines. The resulting column is
    /// as close to the original column position as possible.
    fn adjust_vertical(&mut self, adjust: int) {
        if !self.multiline {
            return;
        }

        if adjust < 0 && self.edit_point.line as int + adjust < 0 {
            self.edit_point.index = 0;
            self.edit_point.line = 0;
            return;
        } else if adjust > 0 && self.edit_point.line + adjust as uint >= self.lines.len() {
            self.edit_point.line = self.lines.len() - 1;
            self.edit_point.index = self.current_line_length();
            return;
        }

        self.edit_point.line = (self.edit_point.line as int + adjust) as uint;
        self.edit_point.index = min(self.current_line_length(), self.edit_point.index);
    }

    /// Adjust the editing point position by a given number of columns. If the adjustment
    /// requested is larger than is available in the current line, the editing point is
    /// adjusted vertically and the process repeats with the remaining adjustment requested.
    fn adjust_horizontal(&mut self, adjust: int) {
        if adjust < 0 {
            let remaining = self.edit_point.index;
            if adjust.abs() as uint > remaining && self.edit_point.line > 0 {
                self.adjust_vertical(-1);
                self.edit_point.index = self.current_line_length();
                self.adjust_horizontal(adjust + remaining as int);
            } else {
                self.edit_point.index = max(0, self.edit_point.index as int + adjust) as uint;
            }
        } else {
            let remaining = self.current_line_length() - self.edit_point.index;
            if adjust as uint > remaining && self.edit_point.line < self.lines.len() - 1 {
                self.edit_point.index = 0;
                self.adjust_vertical(1);
                self.adjust_horizontal(adjust - remaining as int);
            } else {
                self.edit_point.index = min(self.current_line_length(),
                                            self.edit_point.index + adjust as uint);
            }
        }
    }

    /// Deal with a newline input.
    fn handle_return(&mut self) -> KeyReaction {
        if !self.multiline {
            return TriggerDefaultAction;
        }

        //TODO: support replacing selection with newline
        let prefix = self.get_current_line().as_slice().slice_chars(0, self.edit_point.index).to_string();
        let suffix = self.get_current_line().as_slice().slice_chars(self.edit_point.index,
                                                                    self.current_line_length()).to_string();
        self.lines[self.edit_point.line] = prefix;
        self.lines.insert(self.edit_point.line + 1, suffix);
        return DispatchInput;
    }

    /// Process a given `KeyboardEvent` and return an action for the caller to execute.
    pub fn handle_keydown(&mut self, event: JSRef<KeyboardEvent>) -> KeyReaction {
        match event.Key().as_slice() {
            // printable characters have single-character key values
            c if c.len() == 1 => {
                self.insert_char(c.char_at(0));
                return DispatchInput;
            }
            "Space" => {
                self.insert_char(' ');
                DispatchInput
            }
            "Delete" => {
                self.delete_char(Forward);
                DispatchInput
            }
            "Backspace" => {
                self.delete_char(Backward);
                DispatchInput
            }
            "ArrowLeft" => {
                self.adjust_horizontal(-1);
                Nothing
            }
            "ArrowRight" => {
                self.adjust_horizontal(1);
                Nothing
            }
            "ArrowUp" => {
                self.adjust_vertical(-1);
                Nothing
            }
            "ArrowDown" => {
                self.adjust_vertical(1);
                Nothing
            }
            "Enter" => self.handle_return(),
            "Home" => {
                self.edit_point.index = 0;
                Nothing
            }
            "End" => {
                self.edit_point.index = self.current_line_length();
                Nothing
            }
            "PageUp" => {
                self.adjust_vertical(-28);
                Nothing
            }
            "PageDown" => {
                self.adjust_vertical(28);
                Nothing
            }
            "Tab" => TriggerDefaultAction,
            _ => Nothing,
        }
    }

    /// Get the current contents of the text input. Multiple lines are joined by \n.
    pub fn get_content(&self) -> DOMString {
        let mut content = "".to_string();
        for (i, line) in self.lines.iter().enumerate() {
            content.push_str(line.as_slice());
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
            content.as_slice().split('\n').map(|s| s.to_string()).collect()
        } else {
            vec!(content)
        };
        self.edit_point.line = min(self.edit_point.line, self.lines.len() - 1);
        self.edit_point.index = min(self.edit_point.index, self.current_line_length() - 1);
    }
}
