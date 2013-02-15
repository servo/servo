// Style retrieval from DOM elements.

use css::node_util::NodeUtil;
use dom::node::AbstractNode;
use newcss::complete::CompleteStyle;

/// Node mixin providing `style` method that returns a `NodeStyle`
pub trait StyledNode {
    fn style(&self) -> CompleteStyle/&self;
}

impl StyledNode for AbstractNode {
    fn style(&self) -> CompleteStyle/&self {
        assert self.is_element(); // Only elements can have styles
        let results = self.get_css_select_results();
        results.computed_style()
    }
}
