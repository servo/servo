/** Interface for running tree-based traversals over layout boxes and contextsg */

use layout::base::{Box, BoxTree};
use layout::base::{FlowContext, FlowTree};

trait BoxTraversals {
    fn traverse_preorder(preorder_cb: ~fn(@Box));
}

impl @Box : BoxTraversals {
    fn traverse_preorder(preorder_cb: ~fn(@Box)) {
        preorder_cb(self);
        do BoxTree.each_child(self) |child| { child.traverse_preorder(preorder_cb); true }
    }
}

trait FlowContextTraversals {
    fn traverse_preorder(preorder_cb: ~fn(@FlowContext));
    fn traverse_postorder(postorder_cb: ~fn(@FlowContext));
}

impl @FlowContext : FlowContextTraversals {
    fn traverse_preorder(preorder_cb: ~fn(@FlowContext)) {
        preorder_cb(self);
        do FlowTree.each_child(self) |child| { child.traverse_preorder(preorder_cb); true }
    }

    fn traverse_postorder(postorder_cb: ~fn(@FlowContext)) {
        do FlowTree.each_child(self) |child| { child.traverse_postorder(postorder_cb); true }
        postorder_cb(self);
    }
}
