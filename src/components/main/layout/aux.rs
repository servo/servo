/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

//! Code for managing the layout data in the DOM.

use script::dom::node::{AbstractNode, LayoutView};
use servo_util::tree::TreeNodeRef;

/// Functionality useful for querying the layout-specific data on DOM nodes.
pub trait LayoutAuxMethods {
    fn initialize_layout_data(self);
    fn initialize_style_for_subtree(self);
}

impl LayoutAuxMethods for AbstractNode<LayoutView> {
    /// Resets layout data and styles for the node.
    fn initialize_layout_data(self) {
        do self.write_layout_data |data| {
            data.boxes.display_list = None;
            data.boxes.range = None;
        }
    }

    /// Resets layout data and styles for a Node tree.
    fn initialize_style_for_subtree(self) {
        for n in self.traverse_preorder() {
            n.initialize_layout_data();
        }
    }
}
