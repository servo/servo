/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use layout::flow::{FlowContext, FlowTree};

/** Trait for running tree-based traversals over layout contexts */
pub trait FlowContextTraversals {
    fn traverse_preorder(@mut self, preorder_cb: &fn(@mut FlowContext));
    fn traverse_postorder(@mut self, postorder_cb: &fn(@mut FlowContext));
}

impl FlowContextTraversals for FlowContext {
    fn traverse_preorder(@mut self, preorder_cb: &fn(@mut FlowContext)) {
        preorder_cb(self);
        do FlowTree.each_child(self) |child| { child.traverse_preorder(preorder_cb); true }
    }

    fn traverse_postorder(@mut self, postorder_cb: &fn(@mut FlowContext)) {
        do FlowTree.each_child(self) |child| { child.traverse_postorder(postorder_cb); true }
        postorder_cb(self);
    }
}
