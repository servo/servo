/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use std::io::Write;
use std::str;
use termcolor::{Buffer, BufferWriter, Color, ColorSpec, WriteColor};

/// A struct that makes it easier to print out a pretty tree of data, which
/// can be visually scanned more easily.
pub struct PrintTree {
    /// The writer.
    writer: BufferWriter,

    /// The tab stop positions.
    tab_stops: Vec<u32>,

    /// The current level of recursion.
    level: u32,

    /// An item which is queued up, so that we can determine if we need
    /// a mid-tree prefix or a branch ending prefix.
    queued_item: Option<Vec<Buffer>>,
}

impl PrintTree {
    pub fn new(writer: BufferWriter, header: &[Buffer], tab_stops: Vec<u32>) -> PrintTree {
        let mut this = PrintTree {
            writer: writer,
            tab_stops: tab_stops,
            level: 0,
            queued_item: None,
        };
        this.print("\u{250c}  ", &header);
        this
    }

    /// Descend one level in the tree with the given row.
    pub fn new_level(&mut self, fields: &[Buffer]) {
        self.flush_queued_item("\u{251C}\u{2500}  ");

        self.print("\u{251C}\u{2500}  ", &fields);

        self.level = self.level + 1;
    }

    /// Ascend one level in the tree.
    pub fn end_level(&mut self) {
        self.flush_queued_item("\u{2514}\u{2500}  ");
        self.level = self.level - 1;
    }

    /// Add an item to the current level in the tree.
    pub fn add_item(&mut self, fields: Vec<Buffer>) {
        self.flush_queued_item("\u{251C}\u{2500}  ");
        self.queued_item = Some(fields);
    }

    /// Returns an appropriately formatted `Buffer`.
    pub fn buffer(&self, string: &str) -> Buffer {
        let mut buffer = self.writer.buffer();
        drop(buffer.write_all(string.as_bytes()));
        buffer
    }

    fn flush_queued_item(&mut self, prefix: &str) {
        if let Some(queued_item) = self.queued_item.take() {
            self.print(prefix, &queued_item);
        }
    }

    fn print(&mut self, prefix: &str, fields: &[Buffer]) {
        let mut column = 0;

        let mut prefix_buffer = self.writer.buffer();
        drop(prefix_buffer.set_color(ColorSpec::new().set_fg(Some(Color::White))));
        for _ in 0..self.level {
            drop(prefix_buffer.write_all("\u{2502}  ".as_bytes()));
        }
        drop(prefix_buffer.write_all(prefix.as_bytes()));
        drop(prefix_buffer.reset());
        print_internal(&mut self.writer, &mut column, &prefix_buffer);

        if !fields.is_empty() {
            print_internal(&mut self.writer, &mut column, &fields[0]);
            for (field, &tab_stop) in fields[1..].iter().zip(self.tab_stops.iter()) {
                while column < tab_stop {
                    print_string(&mut self.writer, &mut column, " ")
                }
                print_internal(&mut self.writer, &mut column, field)
            }
        }
        print_string(&mut self.writer, &mut column, "\n")
    }
}

impl Drop for PrintTree {
    fn drop(&mut self) {
        self.flush_queued_item("\u{9492}\u{9472}  ");
    }
}

fn print_internal(writer: &mut BufferWriter, column: &mut u32, field: &Buffer) {
    // Strip ANSI escape sequences.
    let mut state = State::Normal;
    if let Ok(string) = str::from_utf8(field.as_slice()) {
        for ch in string.chars() {
            match state {
                State::Normal if ch == '\x1b' => state = State::SawEsc,
                State::Normal => *column += 1,
                State::SawEsc if ch == '[' => state = State::SawLeftBracket,
                State::SawEsc => state = State::Normal,
                State::SawLeftBracket if ch >= '\x40' && ch <= '\x7e' => state = State::Normal,
                State::SawLeftBracket => {}
            }
        }
    }

    drop(writer.print(field));

    enum State {
        Normal,
        SawEsc,
        SawLeftBracket,
    }
}

fn print_string(writer: &mut BufferWriter, column: &mut u32, string: &str) {
    let mut buffer = writer.buffer();
    drop(buffer.write_all(string.as_bytes()));
    print_internal(writer, column, &buffer)
}
