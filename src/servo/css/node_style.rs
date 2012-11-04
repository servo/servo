/*!
Calculates computed Node styles, based on CSS SelectResults.

These methods mostly defer to the Node's ComputedStyle object.
The only exception is that this is where inheritance is resolved.
*/

use dom::node::Node;
use newcss::color::{Color, rgba};
use newcss::units::{Length, Px};
use newcss::values::{CSSValue, Specified, Inherit};
use newcss::values::{CSSMargin, CSSMarginLength};
use newcss::values::{CSSBorderWidth, CSSBorderWidthLength};
use newcss::computed::ComputedStyle;

/// Node mixin providing `style` method that returns a `NodeStyle`
trait StyledNode {
    fn style(&self) -> NodeStyle/&self;
}

impl Node: StyledNode {
    fn style(&self) -> NodeStyle/&self {
        NodeStyle::new(self)
    }
}

/// Provides getters for calculated node styles
pub struct NodeStyle {
    priv node: &Node
}

impl NodeStyle {

    static fn new(node: &r/Node) -> NodeStyle/&r {
        NodeStyle {
            node: node
        }
    }

    fn background_color(&self) -> Color {
        resolve(self, rgba(0, 0, 0, 0.0), |cs| cs.background_color() )
    }

    fn margin_top(&self) -> CSSMargin {
        resolve(self, CSSMarginLength(Px(0.0)), |cs| cs.margin_top() )
    }

    fn margin_right(&self) -> CSSMargin {
        resolve(self, CSSMarginLength(Px(0.0)), |cs| cs.margin_right() )
    }

    fn margin_bottom(&self) -> CSSMargin {
        resolve(self, CSSMarginLength(Px(0.0)), |cs| cs.margin_bottom() )
    }

    fn margin_left(&self) -> CSSMargin {
        resolve(self, CSSMarginLength(Px(0.0)), |cs| cs.margin_left() )
    }

    fn border_top_width(&self) -> CSSBorderWidth {
        resolve(self, CSSBorderWidthLength(Px(0.0)), |cs| cs.border_top_width() )
    }

    fn border_right_width(&self) -> CSSBorderWidth {
        resolve(self, CSSBorderWidthLength(Px(0.0)), |cs| cs.border_right_width() )
    }

    fn border_bottom_width(&self) -> CSSBorderWidth {
        resolve(self, CSSBorderWidthLength(Px(0.0)), |cs| cs.border_bottom_width() )
    }

    fn border_left_width(&self) -> CSSBorderWidth {
        resolve(self, CSSBorderWidthLength(Px(0.0)), |cs| cs.border_left_width() )
    }

    fn border_top_color(&self) -> Color {
        resolve(self, rgba(255, 255, 255, 0.0), |cs| cs.border_top_color() )
    }

    fn border_right_color(&self) -> Color {
        resolve(self, rgba(255, 255, 255, 0.0), |cs| cs.border_right_color() )
    }

    fn border_bottom_color(&self) -> Color {
        resolve(self, rgba(255, 255, 255, 0.0), |cs| cs.border_bottom_color() )
    }

    fn border_left_color(&self) -> Color {
        resolve(self, rgba(255, 255, 255, 0.0), |cs| cs.border_left_color() )
    }

}

fn resolve<T>(node_style: &NodeStyle, default: T, get: &fn(cs: ComputedStyle) -> CSSValue<T>) -> T {
    let node = node_style.node;
    let select_res = node.get_css_select_results();
    let computed = select_res.computed_style();
    let value = get(computed);
    match move value {
        Inherit => /* FIXME: need inheritance */ move default,
        Specified(move value) => move value,
    }
}