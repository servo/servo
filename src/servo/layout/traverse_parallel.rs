/* Interface for running tree-based traversals over layout boxes */
use layout::box::{RenderBox, RenderBoxTree};
use intrinsic::TyDesc;

export full_traversal;
export top_down_traversal;
export bottom_up_traversal;
export extended_full_traversal;
export extended_top_down_traversal;

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

/**
Transform and @ into its underlying representation.  The reference count stays constant.
*/
fn unwrap_box(-b : @RenderBox) -> *shared_box<RenderBox> unsafe {
    let new_box : *shared_box<RenderBox> = cast::transmute(b);
    return new_box;
}

/**
Transform an underlying representation back to an @.  The reference count stays constant.
*/
fn rewrap_box(-b : *shared_box<RenderBox>) -> @RenderBox unsafe {
    let new_box : @RenderBox = cast::transmute(b);
    return new_box;
}

/**

Iterate down and then up a tree of layout boxes in parallel and apply
the given functions to each box.  Each box applies the first function,
spawns a task to complete all of its children in parallel, passing
each child the result of the ifrst funciton. It waits for them to
finish, and then applies the second function to the current box.

# Arguments 

* `root` - The current top of the tree, the functions will be applied
           to it and its children.
* `returned` - The value returned by applying top_down to the parent
               of the current box, or a passed in default
* `top_down` - A function that is applied to each node after it is
              applied to that node's parent.
* `bottom_up` - A function that is applied to each node after it is
                applied to that node's children

*/
fn traverse_helper<T : Copy Send>(-root : @RenderBox, returned : T, -top_down : fn~(+T, @RenderBox) -> T,
                      -bottom_up : fn~(@RenderBox)) {
    let returned = top_down(returned, root);

    do listen |ack_chan| { 
        let mut count = 0;
        
        // For each child we will send it off to another task and then
        // recurse.  It is safe to send these boxes across tasks
        // because root still holds a reference to the children so
        // they will not be destroyed from the other task.  Also the
        // current task will block until all of it's children return,
        // so the original owner of the @-box will not exit while the
        // children are still live.
        for RenderBoxTree.each_child(root) |kid| {
            count += 1;

            // Unwrap the box so we can send it out of this task
            let unwrapped = unwrap_box(copy kid);
            // Hide the box in an option so we can get it across the
            // task boundary without copying it
            let swappable : ~mut Option<*shared_box<RenderBox>> = ~mut Some(unwrapped);

            do task::spawn |copy top_down, copy bottom_up| {
                // Get the box out of the option and into the new task
                let mut swapped_in = None;
                swapped_in <-> *swappable;

                // Retrieve the original @RenderBox and recurse
                let new_kid = rewrap_box(option::unwrap(swapped_in));
                traverse_helper(new_kid, copy returned, copy top_down, copy bottom_up);

                ack_chan.send(());
            }
        }

        // wait for all the children to finish before preceding
        for count.times() { ack_chan.recv(); }
    }

    bottom_up(root);
}

/**
A noneffectful function to be used if only one pass is required.
*/
fn nop(_box : @RenderBox) {
    return;
}

/**
   A wrapper to change a function that only acts on a box to one that
   threasds a unit through to match travserse_helper
*/
fn unit_wrapper(-fun : fn~(@RenderBox)) -> fn~(+(), @RenderBox) {
    fn~(+_u : (), box : @RenderBox) { fun(box); }
}

/**
   Iterate in parallel over the boxes in a tree, applying one function
   to a parent before recursing on its children and one after.
*/
fn full_traversal(+root : @RenderBox, -top_down : fn~(@RenderBox), -bottom_up : fn~(@RenderBox)) {
    traverse_helper(root, (), unit_wrapper(top_down), bottom_up);
}

/**
   Iterate in parallel over the boxes in a tree, applying the given
   function to a parent before its children.
*/
fn top_down_traversal(+root : @RenderBox, -top_down : fn~(@RenderBox)) {
    traverse_helper(root, (), unit_wrapper(top_down), nop);
}

/**
   Iterate in parallel over the boxes in a tree, applying the given
   function to a parent after its children.
*/
fn bottom_up_traversal(+root : @RenderBox, -bottom_up : fn~(@RenderBox)) {
    traverse_helper(root, (), unit_wrapper(nop), bottom_up);
}

/**
   Iterate in parallel over the boxes in a tree, applying the given
   function to a parent before its children, the value returned by the
   function is passed to each child when they are recursed upon.  As
   the recursion unwinds, the second function is applied to first the
   children in parallel, and then the parent.
*/
fn extended_full_traversal<T : Copy Send>(+root : @RenderBox, first_val : T, 
                                          -top_down : fn~(+T, @RenderBox) -> T,
                                          -bottom_up : fn~(@RenderBox)) {
    traverse_helper(root, first_val, top_down, bottom_up);
}

/**
   Iterate in parallel over the boxes in a tree, applying the given
   function to a parent before its children, the value returned by the
   function is passed to each child when they are recursed upon.
*/
fn extended_top_down_traversal<T : Copy Send>(+root : @RenderBox, first_val : T,
                                              -top_down : fn~(+T, @RenderBox) -> T) {
    traverse_helper(root, first_val, top_down, nop);
}
