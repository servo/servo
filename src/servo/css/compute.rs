/*!
Calculates computed Node styles, based on CSS SelectResults.

These methods mostly defer to the Node's ComputedStyle object.
The only exception is that this is where inheritance is resolved.
*/

use dom::node::Node;
use newcss::color::{Color, rgba};
use newcss::values::{CSSValue, Specified, Inherit, Length, Px, CSSBorderWidth, BdrWidthLength};
use newcss::ComputedStyle;

pub trait ComputeStyles {
    fn compute_background_color(&self) -> Color;
    fn compute_border_top_width(&self) -> CSSBorderWidth;
    fn compute_border_right_width(&self) -> CSSBorderWidth;
    fn compute_border_bottom_width(&self) -> CSSBorderWidth;
    fn compute_border_left_width(&self) -> CSSBorderWidth;
}

impl Node: ComputeStyles {
    fn compute_background_color(&self) -> Color {
        resolve(self, rgba(0, 0, 0, 0.0), |cs| cs.background_color() )
    }

    fn compute_border_top_width(&self) -> CSSBorderWidth {
        resolve(self, BdrWidthLength(Px(0.0)), |cs| cs.border_top_width() )
    }

    fn compute_border_right_width(&self) -> CSSBorderWidth {
        resolve(self, BdrWidthLength(Px(0.0)), |cs| cs.border_right_width() )
    }

    fn compute_border_bottom_width(&self) -> CSSBorderWidth {
        resolve(self, BdrWidthLength(Px(0.0)), |cs| cs.border_bottom_width() )
    }

    fn compute_border_left_width(&self) -> CSSBorderWidth {
        resolve(self, BdrWidthLength(Px(0.0)), |cs| cs.border_left_width() )
    }
}

fn resolve<T>(node: &Node, default: T, get: &fn(cs: ComputedStyle) -> CSSValue<T>) -> T {
    let style = node.get_style();
    let computed = style.computed_style();
    let value = get(computed);
    match move value {
        Inherit => /* FIXME: need inheritance */ move default,
        Specified(move value) => move value,
        _ => fail
    }
}