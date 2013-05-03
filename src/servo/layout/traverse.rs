/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use layout::flow::FlowContext;

/// A trait for running tree-based traversals over flows contexts.
pub trait FlowContextTraversals {
    fn traverse_preorder(@mut self, preorder_cb: &fn(@mut FlowContext));
    fn traverse_postorder(@mut self, postorder_cb: &fn(@mut FlowContext));
}

impl FlowContextTraversals for FlowContext {
    fn traverse_preorder(@mut self, preorder_cb: &fn(@mut FlowContext)) {
        preorder_cb(self);
        for self.each_child |child| {
            child.traverse_preorder(preorder_cb);
        }
    }

    fn traverse_postorder(@mut self, postorder_cb: &fn(@mut FlowContext)) {
        for self.each_child |child| {
            child.traverse_postorder(postorder_cb);
        }
        postorder_cb(self);
    }
}
