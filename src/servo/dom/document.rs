use newcss::values::Stylesheet;
use dom::node::{NodeScope, Node};
use std::arc::ARC;

struct Document {
    root: Node,
    scope: NodeScope,
}

fn Document(root: Node, scope: NodeScope) -> Document {
    Document {
        root : root,
        scope : scope,
    }
}
