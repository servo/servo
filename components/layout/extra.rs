/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

//! Code for managing the layout data in the DOM.

use util::{PrivateLayoutData, LayoutDataAccess, LayoutDataWrapper};
use wrapper::LayoutNode;
use script::dom::node::SharedLayoutData;
use script::layout_interface::LayoutChan;

/// Functionality useful for querying the layout-specific data on DOM nodes.
pub trait LayoutAuxMethods {
    fn initialize_layout_data(&self, chan: LayoutChan);
    fn initialize_style_for_subtree(&self, chan: LayoutChan);
}

impl<'ln> LayoutAuxMethods for LayoutNode<'ln> {
    /// Resets layout data and styles for the node.
    ///
    /// FIXME(pcwalton): Do this as part of fragment building instead of in a traversal.
    fn initialize_layout_data(&self, chan: LayoutChan) {
        let mut layout_data_ref = self.mutate_layout_data();
        match *layout_data_ref {
            None => {
                *layout_data_ref = Some(LayoutDataWrapper {
                    chan: Some(chan),
                    shared_data: SharedLayoutData { style: None },
                    data: box PrivateLayoutData::new(),
                });
            }
            Some(_) => {}
        }
    }

    /// Resets layout data and styles for a Node tree.
        ///
    /// FIXME(pcwalton): Do this as part of fragment building instead of in a traversal.
    fn initialize_style_for_subtree(&self, chan: LayoutChan) {
        for n in self.traverse_preorder() {
            n.initialize_layout_data(chan.clone());
        }
    }
}
