/**
Code for managing the DOM aux pointer
*/

use dom::node::{AbstractNode, LayoutData};

pub trait LayoutAuxMethods {
    fn initialize_layout_data(self) -> Option<@mut LayoutData>;
    fn initialize_style_for_subtree(self, refs: &mut ~[@mut LayoutData]);
}

impl LayoutAuxMethods for AbstractNode {
    /// If none exists, creates empty layout data for the node (the reader-auxiliary
    /// box in the COW model) and populates it with an empty style object.
    fn initialize_layout_data(self) -> Option<@mut LayoutData> {
        if self.has_layout_data() {
            None
        } else {
            let data = @mut LayoutData::new();
            self.set_layout_data(data);
            Some(data)
        }
    }

    /// Initializes layout data and styles for a Node tree, if any nodes do not have
    /// this data already. Append created layout data to the task's GC roots.
    fn initialize_style_for_subtree(self, refs: &mut ~[@mut LayoutData]) {
        let _ = for self.traverse_preorder |n| {
            match n.initialize_layout_data() {
                Some(r) => refs.push(r),
                None => {}
            }
        };
    }

}
