use newcss::stylesheet::Stylesheet;
use dom::node::{NodeScope, Node};
use std::arc::ARC;

pub struct Document {
    root: Node,
    scope: NodeScope,
}

pub fn Document(root: Node, scope: NodeScope) -> Document {
    Document {
        root : root,
        scope : scope,
    }
}
