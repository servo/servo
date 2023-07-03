/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use std::io::{stdout, Stdout, Write};

/// A struct that makes it easier to print out a pretty tree of data, which
/// can be visually scanned more easily.
pub struct PrintTree<W>
where
    W: Write
{
    /// The current level of recursion.
    level: u32,

    /// An item which is queued up, so that we can determine if we need
    /// a mid-tree prefix or a branch ending prefix.
    queued_item: Option<String>,

    /// The sink to print to.
    sink: W,
}

/// A trait that makes it easy to describe a pretty tree of data,
/// regardless of the printing destination, to either print it
/// directly to stdout, or serialize it as in the debugger
pub trait PrintTreePrinter {
    fn new_level(&mut self, title: String);
    fn end_level(&mut self);
    fn add_item(&mut self, text: String);
}

impl PrintTree<Stdout> {
    pub fn new(title: &str) -> Self {
        PrintTree::new_with_sink(title, stdout())
    }
}

impl<W> PrintTree<W>
where
    W: Write
{
    pub fn new_with_sink(title: &str, mut sink: W) -> Self {
        writeln!(sink, "\u{250c} {}", title).unwrap();
        PrintTree {
            level: 1,
            queued_item: None,
            sink,
        }
    }

    fn print_level_prefix(&mut self) {
        for _ in 0 .. self.level {
            write!(self.sink, "\u{2502}  ").unwrap();
        }
    }

    fn flush_queued_item(&mut self, prefix: &str) {
        if let Some(queued_item) = self.queued_item.take() {
            self.print_level_prefix();
            writeln!(self.sink, "{} {}", prefix, queued_item).unwrap();
        }
    }
}

// The default `println!` based printer
impl<W> PrintTreePrinter for PrintTree<W>
where
    W: Write
{
    /// Descend one level in the tree with the given title.
    fn new_level(&mut self, title: String) {
        self.flush_queued_item("\u{251C}\u{2500}");

        self.print_level_prefix();
        writeln!(self.sink, "\u{251C}\u{2500} {}", title).unwrap();

        self.level = self.level + 1;
    }

    /// Ascend one level in the tree.
    fn end_level(&mut self) {
        self.flush_queued_item("\u{2514}\u{2500}");
        self.level = self.level - 1;
    }

    /// Add an item to the current level in the tree.
    fn add_item(&mut self, text: String) {
        self.flush_queued_item("\u{251C}\u{2500}");
        self.queued_item = Some(text);
    }
}

impl<W> Drop for PrintTree<W>
where
    W: Write
{
    fn drop(&mut self) {
        self.flush_queued_item("\u{9492}\u{9472}");
    }
}

pub trait PrintableTree {
    fn print_with<T: PrintTreePrinter>(&self, pt: &mut T);
}
