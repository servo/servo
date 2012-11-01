/**
Code for managing the DOM aux pointer
*/

use dom::node::{Node, LayoutData};
use core::dvec::DVec;

pub trait LayoutAuxMethods {
    fn initialize_layout_data() -> Option<@LayoutData>;
    fn initialize_style_for_subtree(refs: &DVec<@LayoutData>);
}

impl Node : LayoutAuxMethods {
    /** If none exists, creates empty layout data for the node (the reader-auxiliary
     * box in the COW model) and populates it with an empty style object.
     */
    fn initialize_layout_data() -> Option<@LayoutData> {
        match self.has_aux() {
            false => {
                let data = @LayoutData({
                    mut style : None,
                    mut flow  : None
                });
                self.set_aux(data); Some(data)
            },
            true => None
        }
    }

    /**
     * Initializes layout data and styles for a Node tree, if any nodes do not have
     * this data already. Append created layout data to the task's GC roots.
     */
    fn initialize_style_for_subtree(refs: &DVec<@LayoutData>) {
        do self.traverse_preorder |n| {
            match n.initialize_layout_data() {
                Some(r) => refs.push(r),
                None => {}
            }
        }
    }

}