/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use layout::box::{RenderBox};
use script::dom::node::{AbstractNode, LayoutView};
use servo_util::range::Range;

use std::iterator::Enumerate;
use std::vec::VecIterator;

pub struct NodeRange {
    node: AbstractNode<LayoutView>,
    range: Range,
}

impl NodeRange {
    pub fn new(node: AbstractNode<LayoutView>, range: &Range) -> NodeRange {
        NodeRange { node: node, range: (*range).clone() }
    }
}

struct ElementMapping {
    priv entries: ~[NodeRange],
}

impl ElementMapping {
    pub fn new() -> ElementMapping {
        ElementMapping { entries: ~[] }
    }

    pub fn add_mapping(&mut self, node: AbstractNode<LayoutView>, range: &Range) {
        self.entries.push(NodeRange::new(node, range))
    }

    pub fn each(&self, callback: &fn(nr: &NodeRange) -> bool) -> bool {
        for nr in self.entries.iter() {
            if !callback(nr) {
                break
            }
        }
        true
    }

    pub fn eachi<'a>(&'a self) -> Enumerate<VecIterator<'a, NodeRange>> {
        self.entries.iter().enumerate()
    }

    pub fn repair_for_box_changes(&mut self, old_boxes: &[RenderBox], new_boxes: &[RenderBox]) {
        let entries = &mut self.entries;

        debug!("--- Old boxes: ---");
        for (i, box) in old_boxes.iter().enumerate() {
            debug!("%u --> %s", i, box.debug_str());
        }
        debug!("------------------");

        debug!("--- New boxes: ---");
        for (i, box) in new_boxes.iter().enumerate() {
            debug!("%u --> %s", i, box.debug_str());
        }
        debug!("------------------");

        debug!("--- Elem ranges before repair: ---");
        for (i, nr) in entries.iter().enumerate() {
            debug!("%u: %? --> %s", i, nr.range, nr.node.debug_str());
        }
        debug!("----------------------------------");

        let mut old_i = 0;
        let mut new_j = 0;

        struct WorkItem {
            begin_idx: uint,
            entry_idx: uint,
        };
        let mut repair_stack : ~[WorkItem] = ~[];

            // index into entries
            let mut entries_k = 0;

            while old_i < old_boxes.len() {
                debug!("repair_for_box_changes: Considering old box %u", old_i);
                // possibly push several items
                while entries_k < entries.len() && old_i == entries[entries_k].range.begin() {
                    let item = WorkItem {begin_idx: new_j, entry_idx: entries_k};
                    debug!("repair_for_box_changes: Push work item for elem %u: %?", entries_k, item);
                    repair_stack.push(item);
                    entries_k += 1;
                }
                // XXX: the following loop form causes segfaults; assigning to locals doesn't.
                // while new_j < new_boxes.len() && old_boxes[old_i].d().node != new_boxes[new_j].d().node {
                while new_j < new_boxes.len() {
                    let should_leave = do old_boxes[old_i].with_base |old_box_base| {
                        do new_boxes[new_j].with_base |new_box_base| {
                            old_box_base.node != new_box_base.node
                        }
                    };
                    if should_leave {
                        break
                    }

                    debug!("repair_for_box_changes: Slide through new box %u", new_j);
                    new_j += 1;
                }

                old_i += 1;

                // possibly pop several items
                while repair_stack.len() > 0 && old_i == entries[repair_stack.last().entry_idx].range.end() {
                    let item = repair_stack.pop();
                    debug!("repair_for_box_changes: Set range for %u to %?",
                           item.entry_idx, Range::new(item.begin_idx, new_j - item.begin_idx));
                    entries[item.entry_idx].range = Range::new(item.begin_idx, new_j - item.begin_idx);
                }
            }
        debug!("--- Elem ranges after repair: ---");
        for (i, nr) in entries.iter().enumerate() {
            debug!("%u: %? --> %s", i, nr.range, nr.node.debug_str());
        }
        debug!("----------------------------------");
    }
}
