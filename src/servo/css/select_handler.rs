use dom::node::Node;
use newcss::SelectHandler;

pub struct NodeSelectHandler {
    node: Node
}

impl NodeSelectHandler: SelectHandler<Node> {
    
}