/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

//! Code for managing the layout data in the DOM.

use layout::util::{DisplayBoxes, LayoutData, LayoutDataAccess};

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
        let layout_data_handle = self.mutate_layout_data();
        match *layout_data_handle.ptr {
            None => *layout_data_handle.ptr = Some(~LayoutData::new()),
            Some(ref mut layout_data) => layout_data.boxes = DisplayBoxes::init(),
        }
    }

    /// Resets layout data and styles for a Node tree.
    ///
    /// FIXME(pcwalton): Do this as part of box building instead of in a traversal.
    fn initialize_style_for_subtree(self) {
        for n in self.traverse_preorder() {
            n.initialize_layout_data();
        }
    }
}
