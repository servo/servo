use dom::node::{Node, NodeData, NodeTree, Doctype, Comment, Element, Text};
use newcss::select::SelectHandler;
use util::tree;

pub struct NodeSelectHandler {
    node: Node
}

/// Placeholder names
fn unnamed_node(name: &str) -> ~str {
    fmt!("unnamed-%s", name)
}

fn node_name(data: &NodeData) -> ~str {
    match *data.kind {
        Doctype(*) => unnamed_node("doctype"),
        Comment(*) => unnamed_node("comment"),
        Element(ref data) => copy data.tag_name,
        Text(*) => unnamed_node("text")
    }
}

impl NodeSelectHandler: SelectHandler<Node> {
    fn node_name(node: &Node) -> ~str {
        do node.read |data| {
            node_name(data)
        }
    }

    fn named_parent_node(node: &Node, name: &str) -> Option<Node> {
        let parent = tree::parent(&NodeTree, node);
        match parent {
            Some(parent) => {
                do parent.read |data| {
                    if name == node_name(data) {
                        Some(parent)
                    } else {
                        None
                    }
                }
            }
            None => None
        }
    }

    fn parent_node(node: &Node) -> Option<Node> {
        tree::parent(&NodeTree, node)
    }
}