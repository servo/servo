use crate::dom_wrapper::ServoLayoutElement;
use crate::dom_wrapper::ServoLayoutNode;
use crate::dom_wrapper::ServoThreadSafeLayoutNode;
use layout::traversal::{RecalcStyle, SequentialDomTraversal};
use style::dom::{TElement, TNode};
use script_layout_interface::wrapper_traits::ThreadSafeLayoutNode;

pub fn traverse_dom<'le, D>(
    traversal: &D,
    root: ServoLayoutElement<'le>,
    recalc_style: &RecalcStyle<ServoLayoutNode<'le>>,
) -> ServoLayoutElement<'le>
where
    D: SequentialDomTraversal<ServoLayoutElement<'le>>,
{
    let nodes = std::mem::take(&mut *recalc_style.nodes.lock().unwrap());

    for (node, children_count) in nodes {
        if children_count != 0 {
            continue;
        }
        let node = ServoThreadSafeLayoutNode::new(*node);
        traversal.handle_postorder_traversal(
            root.as_node().opaque(),
            unsafe { node.unsafe_get() },
        );
    }

    root
}
