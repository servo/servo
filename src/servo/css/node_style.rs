use dom::node::Node;
use newcss::complete::CompleteStyle;

/// Node mixin providing `style` method that returns a `NodeStyle`
trait StyledNode {
    fn style(&self) -> CompleteStyle/&self;
}

impl Node: StyledNode {
    fn style(&self) -> CompleteStyle/&self {
        assert self.is_element(); // Only elements can have styles
        let results = self.get_css_select_results();
        results.computed_style()
    }
}
