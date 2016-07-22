/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

/// A struct that makes it easier to print out a pretty tree of data, which
/// can be visually scanned more easily.
pub struct PrintTree {
    /// The current level of recursion.
    level: u32,

    /// An item which is queued up, so that we can determine if we need
    /// a mid-tree prefix or a branch ending prefix.
    queued_item: Option<String>,
}

impl PrintTree {
    pub fn new(title: String) -> PrintTree {
        println!("┌ {}", title);
        PrintTree {
            level: 1,
            queued_item: None,
        }
    }

    /// Descend one level in the tree with the given title.
    pub fn new_level(&mut self, title: String) {
        self.flush_queued_item("├─");

        self.print_level_prefix();
        println!("├─ {}", title);

        self.level = self.level + 1;
    }

    /// Ascend one level in the tree.
    pub fn end_level(&mut self) {
        self.flush_queued_item("└─");
        self.level = self.level - 1;
    }

    /// Add an item to the current level in the tree.
    pub fn add_item(&mut self, text: String) {
        self.flush_queued_item("├─");
        self.queued_item = Some(text);
    }

    fn print_level_prefix(&self) {
        for _ in 0..self.level {
            print!("│  ");
        }
    }

    fn flush_queued_item(&mut self, prefix: &str) {
        if let Some(queued_item) = self.queued_item.take() {
            self.print_level_prefix();
            println!("{} {}", prefix, queued_item);
        }
    }
}

impl Drop for PrintTree {
    fn drop(&mut self) {
        self.flush_queued_item("└─");
    }
}
