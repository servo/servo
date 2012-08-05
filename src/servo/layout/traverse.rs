#[doc = "Interface for running tree-based traversals over layout boxes"]

import base::{Box, BTree, NodeMethods};
import intrinsic::tydesc;

export full_traversal;
export top_down_traversal;
export bottom_up_traversal;

// The underlying representation of an @T.  We don't actually care
// what it is, just that we can transform to and from this
// representation to send boxes across task boundaries.
type shared_box<T> = {
    mut refcount : uint,
    // These are generic unsafe pointers, not just *ints
    foo : *int,
    bar : *int,
    baz : *int,
    payload : T
};

#[doc="Transform and @ into its underlying representation.  The reference count stays constant."]
fn unwrap_box(-b : @Box) -> *shared_box<Box> unsafe {
    let new_box : *shared_box<Box> = unsafe::transmute(b);
    return new_box;
}

#[doc="Transform an underlying representation back to an @.  The reference count stays constant."]
fn rewrap_box(-b : *shared_box<Box>) -> @Box unsafe {
    let new_box : @Box = unsafe::transmute(b);
    return new_box;
}

#[doc="

Iterate down and then up a tree of layout boxes in parallel and apply
the given functions to each box.  Each box applies the first function,
spawns a task to complete all of its children in parallel, waits for
them to finish, and then applies the second function.

# Arguments 

* `root` - The current top of the tree, the functions will be applied to it and its children.
* `top-down` - A function that is applied to each node after it is applied to that node's parent.
* `bottom-up` - A function that is applied to each node after it is applied to that node's 
                children
"]
fn traverse_helper(-root : @Box, -top_down : fn~(@Box), -bottom_up : fn~(@Box)) {
    top_down(root);

    do listen |ack_chan| { 
        let mut count = 0;
        
        // For each child we will send it off to another task and then
        // recurse.  It is safe to send these boxes across tasks
        // because root still holds a reference to the children so
        // they will not be destroyed from the other task.  Also the
        // current task will block until all of it's children return,
        // so the original owner of the @-box will not exit while the
        // children are still live.
        for BTree.each_child(root) |kid| {
            count += 1;

            // Unwrap the box so we can send it out of this task
            let unwrapped = unwrap_box(copy kid);
            // Hide the box in an option so we can get it across the
            // task boundary without copying it
            let swappable : ~mut option<*shared_box<Box>> = ~mut some(unwrapped);

            do task::spawn || {
                // Get the box out of the option and into the new task
                let mut swapped_in = none;
                swapped_in <-> *swappable;

                // Retrieve the original @Box and recurse
                let new_kid = rewrap_box(option::unwrap(swapped_in));
                traverse_helper(new_kid, copy top_down, copy bottom_up);

                ack_chan.send(());
            }
        }

        // wait for all the children to finish before preceding
        for count.times() { ack_chan.recv(); }
    }

    bottom_up(root);
}

#[doc="A noneffectful function to be used if only one pass is required."]
fn nop(box : @Box) {
    return;
}

#[doc="
   Iterate in parallel over the boxes in a tree, applying one function
   to a parent before recursing on its children and one after.
"]

fn full_traversal(+root : @Box, -top_down : fn~(@Box), -bottom_up : fn~(@Box)) {
    traverse_helper(root, top_down, bottom_up);
}

#[doc="
   Iterate in parallel over the boxes in a tree, applying the given
   function to a parent before its children.
"]
fn top_down_traversal(+root : @Box, -top_down : fn~(@Box)) {
    traverse_helper(root, top_down, nop);
}

#[doc="
   Iterate in parallel over the boxes in a tree, applying the given
   function to a parent after its children.
"]
fn bottom_up_traversal(+root : @Box, -bottom_up : fn~(@Box)) {
    traverse_helper(root, nop, bottom_up);
}
