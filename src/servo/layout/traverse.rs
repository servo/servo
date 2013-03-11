use layout::flow::{FlowContext, FlowTree};

/** Trait for running tree-based traversals over layout contexts */
pub trait FlowContextTraversals {
    fn traverse_preorder(preorder_cb: &fn(@mut FlowContext));
    fn traverse_postorder(postorder_cb: &fn(@mut FlowContext));
}

impl FlowContextTraversals for @mut FlowContext {
    fn traverse_preorder(preorder_cb: &fn(@mut FlowContext)) {
        preorder_cb(self);
        do FlowTree.each_child(self) |child| { child.traverse_preorder(preorder_cb); true }
    }

    fn traverse_postorder(postorder_cb: &fn(@mut FlowContext)) {
        do FlowTree.each_child(self) |child| { child.traverse_postorder(postorder_cb); true }
        postorder_cb(self);
    }
}
