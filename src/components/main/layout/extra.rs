/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

//! Code for managing the layout data in the DOM.

use layout::util::{PrivateLayoutData, LayoutDataAccess, LayoutDataWrapper};
use layout::wrapper::LayoutNode;
use script::layout_interface::LayoutChan;

/// Functionality useful for querying the layout-specific data on DOM nodes.
pub trait LayoutAuxMethods {
    fn initialize_layout_data(self, chan: LayoutChan);
    fn initialize_style_for_subtree(self, chan: LayoutChan);
}

impl<'ln> LayoutAuxMethods for LayoutNode<'ln> {
    /// Resets layout data and styles for the node.
    ///
    /// FIXME(pcwalton): Do this as part of box building instead of in a traversal.
    fn initialize_layout_data(self, chan: LayoutChan) {
        let mut layout_data_ref = self.mutate_layout_data();
        match *layout_data_ref.get() {
            None => {
                *layout_data_ref.get() = Some(LayoutDataWrapper {
                    chan: Some(chan),
                    data: ~PrivateLayoutData::new(),
                });
            }
            Some(_) => {}
        }
    }

    /// Resets layout data and styles for a Node tree.
        ///
    /// FIXME(pcwalton): Do this as part of box building instead of in a traversal.
    fn initialize_style_for_subtree(self, chan: LayoutChan) {
        for n in self.traverse_preorder() {
            n.initialize_layout_data(chan.clone());
        }
    }
}
