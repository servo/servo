use css::values::Stylesheet;
use dom::node::{NodeScope, Node};
use std::arc::ARC;

struct Document {
    root: Node,
    scope: NodeScope,
    css_rules: ARC<Stylesheet>,
}

fn Document(root: Node, scope: NodeScope, css_rules: Stylesheet) -> Document {
    Document {
        root : root,
        scope : scope,
        css_rules : ARC(css_rules),
    }
}
