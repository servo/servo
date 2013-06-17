/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

//! Code for managing the layout data in the DOM.

use layout::flow::FlowContext;

use newcss::complete::CompleteSelectResults;
use script::dom::node::{AbstractNode, LayoutView};
use servo_util::tree::TreeUtils;

/// Data that layout associates with a node.
pub struct LayoutData {
    /// The results of CSS styling for this node.
    style: Option<CompleteSelectResults>,

    /// The CSS flow that this node is associated with.
    flow: Option<FlowContext>,
}

impl LayoutData {
    /// Creates new layout data.
    pub fn new() -> LayoutData {
        LayoutData {
            style: None,
            flow: None,
        }
    }
}

/// Functionality useful for querying the layout-specific data on DOM nodes.
pub trait LayoutAuxMethods {
    fn layout_data(self) -> @mut LayoutData;
    pub fn has_layout_data(self) -> bool;
    fn set_layout_data(self, data: @mut LayoutData);

    fn initialize_layout_data(self) -> Option<@mut LayoutData>;
    fn initialize_style_for_subtree(self, refs: &mut ~[@mut LayoutData]);
}

impl LayoutAuxMethods for AbstractNode<LayoutView> {
    // FIXME (Rust #3080): These unsafe blocks are *not* unused!
    pub fn layout_data(self) -> @mut LayoutData {
        /*unsafe {*/
            self.unsafe_layout_data()
        /*}*/
    }
    pub fn has_layout_data(self) -> bool {
        /*unsafe {*/
            self.unsafe_has_layout_data()
        /*}*/
    }
    pub fn set_layout_data(self, data: @mut LayoutData) {
        /*unsafe {*/
            self.unsafe_set_layout_data(data)
        /*}*/
    }

    /// If none exists, creates empty layout data for the node (the reader-auxiliary
    /// box in the COW model) and populates it with an empty style object.
    fn initialize_layout_data(self) -> Option<@mut LayoutData> {
        if self.has_layout_data() {
            {
                let layout_data = &mut self.layout_data().flow;
                match *layout_data {
                  Some(ref flow) => flow.teardown(),
                  None => ()
                }
            }
            self.layout_data().flow = None;
            None
        } else {
            let data = @mut LayoutData::new();
            self.set_layout_data(data);
            Some(data)
        }
    }

    /// Initializes layout data and styles for a Node tree, if any nodes do not have
    /// this data already. Append created layout data to the task's GC roots.
    fn initialize_style_for_subtree(self, refs: &mut ~[@mut LayoutData]) {
        let _ = for self.traverse_preorder |n| {
            match n.initialize_layout_data() {
                Some(r) => refs.push(r),
                None => {}
            }
        };
    }
}

