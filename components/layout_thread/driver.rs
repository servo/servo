use crate::dom_wrapper::ServoLayoutElement;
use crate::dom_wrapper::ServoLayoutNode;
use crate::dom_wrapper::ServoThreadSafeLayoutNode;
use layout::traversal::{RecalcStyle, SequentialDomTraversal};
use script_layout_interface::wrapper_traits::ThreadSafeLayoutNode;

pub fn traverse_dom<'le, D>(
    traversal: &mut D,
    root: ServoLayoutElement<'le>,
    recalc_style: &RecalcStyle<ServoLayoutNode<'le>>,
) -> ServoLayoutElement<'le>
where
    D: SequentialDomTraversal<ServoLayoutElement<'le>>,
{
    let nodes = std::mem::take(&mut *recalc_style.nodes.lock().unwrap());

    for (node, _children_count) in nodes {
        // Process the nodes in the same order that the original traversal processed them.
        let node = ServoThreadSafeLayoutNode::new(*node);
        traversal.process_postorder(unsafe { node.unsafe_get() });
    }

    root
}
