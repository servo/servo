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
use newcss::values::{CSSDisplay, CSSDisplayBlock};
use newcss::values::{CSSPosition, CSSPositionRelative};
use newcss::values::{CSSFloat, CSSFloatNone};
use newcss::values::{CSSWidth, CSSWidthLength};
use newcss::values::{CSSHeight, CSSHeightLength};
use newcss::computed::ComputedStyle;

/// Node mixin providing `style` method that returns a `NodeStyle`
trait StyledNode {
    fn style(&self) -> NodeStyle/&self;
}

impl Node: StyledNode {
    fn style(&self) -> NodeStyle/&self {
        assert self.is_element(); // Only elements can have styles
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

    // CSS 2.1, Section 8 - Box model

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

    // CSS 2.1, Section 9 - Visual formatting model

    fn display(&self) -> CSSDisplay {
        // FIXME: Shouldn't be passing false here
        resolve(self, CSSDisplayBlock, |cs| cs.display(false) )
    }

    fn position(&self) -> CSSPosition {
        resolve(self, CSSPositionRelative, |cs| cs.position() )
    }

    fn float(&self) -> CSSFloat {
        resolve(self, CSSFloatNone, |cs| cs.float() )
    }

    // CSS 2.1, Section 10 - Visual formatting model details

    fn width(&self) -> CSSWidth {
        resolve(self, CSSWidthLength(Px(0.0)), |cs| cs.width() )
    }

    fn height(&self) -> CSSHeight {
        resolve(self, CSSHeightLength(Px(0.0)), |cs| cs.height() )
    }

    // CSS 2.1, Section 11 - Visual effects

    // CSS 2.1, Section 12 - Generated content, automatic numbering, and lists

    // CSS 2.1, Section 13 - Paged media

    // CSS 2.1, Section 14 - Colors and Backgrounds

    fn background_color(&self) -> Color {
        resolve(self, rgba(0, 0, 0, 0.0), |cs| cs.background_color() )
    }

    // CSS 2.1, Section 15 - Fonts

    // CSS 2.1, Section 16 - Text

    // CSS 2.1, Section 17 - Tables

    // CSS 2.1, Section 18 - User interface

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