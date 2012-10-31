use dom::node::{Node, Doctype, Comment, Element, Text};
use newcss::SelectHandler;

pub struct NodeSelectHandler {
    node: Node
}

/// Placeholder names
fn unnamed_node(name: &str) -> ~str {
    fmt!("unnamed-%s", name)
}

impl NodeSelectHandler: SelectHandler<Node> {
    fn node_name(node: &Node) -> ~str {
        do node.read |data| {
            match *data.kind {
                Doctype(*) => unnamed_node("doctype"),
                Comment(*) => unnamed_node("comment"),
                Element(ref data) => copy data.tag_name,
                Text(*) => unnamed_node("text")
            }
        }
    }
}