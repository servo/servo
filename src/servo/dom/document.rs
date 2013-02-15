use dom::node::AbstractNode;
use newcss::stylesheet::Stylesheet;

use std::arc::ARC;

pub struct Document {
    root: AbstractNode,
}

pub fn Document(root: AbstractNode) -> Document {
    Document {
        root: root,
    }
}
