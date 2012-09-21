/** Interface for running tree-based traversals over layout boxes and contextsg */

use layout::box::{RenderBox, RenderBoxTree};
use layout::flow::{FlowContext, FlowTree};

/* TODO: we shouldn't need render box traversals  */
trait RenderBoxTraversals {
    fn traverse_preorder(preorder_cb: &fn(@RenderBox));
}

impl @RenderBox : RenderBoxTraversals {
    fn traverse_preorder(preorder_cb: &fn(@RenderBox)) {
        preorder_cb(self);
        do RenderBoxTree.each_child(self) |child| { child.traverse_preorder(preorder_cb); true }
    }
}

trait FlowContextTraversals {
    fn traverse_preorder(preorder_cb: &fn(@FlowContext));
    fn traverse_postorder(postorder_cb: &fn(@FlowContext));
}

impl @FlowContext : FlowContextTraversals {
    fn traverse_preorder(preorder_cb: &fn(@FlowContext)) {
        preorder_cb(self);
        do FlowTree.each_child(self) |child| { child.traverse_preorder(preorder_cb); true }
    }

    fn traverse_postorder(postorder_cb: &fn(@FlowContext)) {
        do FlowTree.each_child(self) |child| { child.traverse_postorder(postorder_cb); true }
        postorder_cb(self);
    }
}
