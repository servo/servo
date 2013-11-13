/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use layout::box::{RenderBox, RenderBoxUtils};

use extra::arc::Arc;
use gfx::display_list::DisplayList;
use script::dom::node::{AbstractNode, LayoutView};
use servo_util::range::Range;
use servo_util::slot::Slot;
use servo_util::tree::TreeNodeRef;
use std::any::AnyRefExt;
use std::iter::Enumerate;
use std::vec::VecIterator;
use style::{ComputedValues, PropertyDeclaration};

/// The boxes associated with a node.
pub struct DisplayBoxes {
    display_list: Option<Arc<DisplayList<AbstractNode<()>>>>,
    range: Option<Range>,
}

impl DisplayBoxes {
    pub fn init() -> DisplayBoxes {
        DisplayBoxes {
            display_list: None,
            range: None,
        }
    }
}

/// A range of nodes.
pub struct NodeRange {
    node: AbstractNode<LayoutView>,
    range: Range,
}

impl NodeRange {
    pub fn new(node: AbstractNode<LayoutView>, range: &Range) -> NodeRange {
        NodeRange {
            node: node,
            range: (*range).clone()
        }
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

    pub fn repair_for_box_changes(&mut self, old_boxes: &[@RenderBox], new_boxes: &[@RenderBox]) {
        let entries = &mut self.entries;

        debug!("--- Old boxes: ---");
        for (i, box) in old_boxes.iter().enumerate() {
            debug!("{:u} --> {:s}", i, box.debug_str());
        }
        debug!("------------------");

        debug!("--- New boxes: ---");
        for (i, box) in new_boxes.iter().enumerate() {
            debug!("{:u} --> {:s}", i, box.debug_str());
        }
        debug!("------------------");

        debug!("--- Elem ranges before repair: ---");
        for (i, nr) in entries.iter().enumerate() {
            debug!("{:u}: {} --> {:s}", i, nr.range, nr.node.debug_str());
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
                debug!("repair_for_box_changes: Considering old box {:u}", old_i);
                // possibly push several items
                while entries_k < entries.len() && old_i == entries[entries_k].range.begin() {
                    let item = WorkItem {begin_idx: new_j, entry_idx: entries_k};
                    debug!("repair_for_box_changes: Push work item for elem {:u}: {:?}", entries_k, item);
                    repair_stack.push(item);
                    entries_k += 1;
                }
                while new_j < new_boxes.len() &&
                        old_boxes[old_i].base().node != new_boxes[new_j].base().node {
                    debug!("repair_for_box_changes: Slide through new box {:u}", new_j);
                    new_j += 1;
                }

                old_i += 1;

                // possibly pop several items
                while repair_stack.len() > 0 && old_i == entries[repair_stack.last().entry_idx].range.end() {
                    let item = repair_stack.pop();
                    debug!("repair_for_box_changes: Set range for {:u} to {}",
                           item.entry_idx, Range::new(item.begin_idx, new_j - item.begin_idx));
                    entries[item.entry_idx].range = Range::new(item.begin_idx, new_j - item.begin_idx);
                }
            }
        debug!("--- Elem ranges after repair: ---");
        for (i, nr) in entries.iter().enumerate() {
            debug!("{:u}: {} --> {:s}", i, nr.range, nr.node.debug_str());
        }
        debug!("----------------------------------");
    }
}

/// Data that layout associates with a node.
pub struct LayoutData {
    /// The results of CSS matching for this node.
    applicable_declarations: Slot<~[Arc<~[PropertyDeclaration]>]>,

    /// The results of CSS styling for this node.
    style: Slot<Option<ComputedValues>>,

    /// Description of how to account for recent style changes.
    restyle_damage: Slot<Option<int>>,

    /// The boxes assosiated with this flow.
    /// Used for getBoundingClientRect and friends.
    boxes: Slot<DisplayBoxes>,
}

impl LayoutData {
    /// Creates new layout data.
    pub fn new() -> LayoutData {
        LayoutData {
            applicable_declarations: Slot::init(~[]),
            style: Slot::init(None),
            restyle_damage: Slot::init(None),
            boxes: Slot::init(DisplayBoxes::init()),
        }
    }
}

// This serves as a static assertion that layout data remains sendable. If this is not done, then
// we can have memory unsafety, which usually manifests as shutdown crashes.
fn assert_is_sendable<T:Send>(_: T) {}
fn assert_layout_data_is_sendable() {
    assert_is_sendable(LayoutData::new())
}

/// A trait that allows access to the layout data of a DOM node.
pub trait LayoutDataAccess {
    fn layout_data<'a>(&'a self) -> &'a LayoutData;
}

impl LayoutDataAccess for AbstractNode<LayoutView> {
    #[inline(always)]
    fn layout_data<'a>(&'a self) -> &'a LayoutData {
        self.node().layout_data.as_ref().unwrap().as_ref().unwrap()
    }
}

