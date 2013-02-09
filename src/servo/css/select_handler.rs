use dom::node::{Node, NodeData, NodeTree, Doctype, Comment, Element, Text};
use newcss::select::SelectHandler;
use util::tree;

pub struct NodeSelectHandler {
    node: Node
}

fn with_node_name<R>(data: &NodeData, f: &fn(&str) -> R) -> R {
    match *data.kind {
        Element(ref data) => f(data.tag_name),
        _ => fail!(~"attempting to style non-element node")
    }
}

impl NodeSelectHandler: SelectHandler<Node> {
    fn with_node_name<R>(node: &Node, f: &fn(&str) -> R) -> R {
        do node.read |data| {
            with_node_name(data, f)
        }
    }

    fn named_parent_node(node: &Node, name: &str) -> Option<Node> {
        let parent = tree::parent(&NodeTree, node);
        match parent {
            Some(parent) => {
                do parent.read |data| {
                    do with_node_name(data) |node_name| {
                        if name == node_name {
                            Some(parent)
                        } else {
                            None
                        }
                    }
                }
            }
            None => None
        }
    }

    fn parent_node(node: &Node) -> Option<Node> {
        tree::parent(&NodeTree, node)
    }

    // TODO: Use a Bloom filter.
    fn named_ancestor_node(node: &Node, name: &str) -> Option<Node> {
        let mut node = *node;
        loop {
            let parent = tree::parent(&NodeTree, &node);
            match parent {
                Some(parent) => {
                    let mut found = false;
                    do parent.read |data| {
                        do with_node_name(data) |node_name| {
                            if name == node_name {
                                found = true;
                            }
                        }
                    }
                    if found {
                        return Some(parent);
                    }
                    node = parent;
                }
                None => return None
            }
        }
    }

    fn node_is_root(node: &Node) -> bool {
        self.parent_node(node).is_none()
    }

    fn with_node_id<R>(node: &Node, f: &fn(Option<&str>) -> R) -> R {
        do node.read |data| {
            match *data.kind {
                Element(ref data) => data.with_attr("id", f),
                _ => fail!(~"attempting to style non-element node")
            }
        }
    }

    fn node_has_id(node: &Node, id: &str) -> bool {
        do node.read |data| {
            match *data.kind {
                Element(ref data) => {
                    do data.with_attr("id") |existing_id_opt| {
                        match existing_id_opt {
                            None => false,
                            Some(existing_id) => str::eq_slice(id, existing_id)
                        }
                    }
                }
                _ => fail!(~"attempting to style non-element node")
            }
        }
    }
}
