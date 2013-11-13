/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

//! Code for managing the layout data in the DOM.

use layout::util::{DisplayBoxes, LayoutData, LayoutDataAccess};

use script::dom::node::{AbstractNode, LayoutView};
use servo_util::tree::TreeNodeRef;
use std::cast;

/// Functionality useful for querying the layout-specific data on DOM nodes.
pub trait LayoutAuxMethods {
    fn initialize_layout_data(self);
    fn initialize_style_for_subtree(self);
}

impl LayoutAuxMethods for AbstractNode<LayoutView> {
    /// Resets layout data and styles for the node.
    ///
    /// FIXME(pcwalton): Do this as part of box building instead of in a traversal.
    fn initialize_layout_data(self) {
        unsafe {
            let node = cast::transmute_mut(self.node());
            if node.layout_data.is_none() {
                node.layout_data = Some(~LayoutData::new() as ~Any)
            } else {
                self.layout_data().boxes.set(DisplayBoxes::init());
            }
        }
    }

    /// Resets layout data and styles for a Node tree.
    fn initialize_style_for_subtree(self) {
        for n in self.traverse_preorder() {
            n.initialize_layout_data();
        }
    }
}
