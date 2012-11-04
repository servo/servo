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

pub trait ComputeStyles {
    fn compute_background_color(&self) -> Color;
    fn compute_margin_top(&self) -> CSSMargin;
    fn compute_margin_right(&self) -> CSSMargin;
    fn compute_margin_bottom(&self) -> CSSMargin;
    fn compute_margin_left(&self) -> CSSMargin;
    fn compute_border_top_width(&self) -> CSSBorderWidth;
    fn compute_border_right_width(&self) -> CSSBorderWidth;
    fn compute_border_bottom_width(&self) -> CSSBorderWidth;
    fn compute_border_left_width(&self) -> CSSBorderWidth;
    fn compute_border_top_color(&self) -> Color;
    fn compute_border_right_color(&self) -> Color;
    fn compute_border_bottom_color(&self) -> Color;
    fn compute_border_left_color(&self) -> Color;
}

impl Node: ComputeStyles {
    fn compute_background_color(&self) -> Color {
        resolve(self, rgba(0, 0, 0, 0.0), |cs| cs.background_color() )
    }

    fn compute_margin_top(&self) -> CSSMargin {
        resolve(self, CSSMarginLength(Px(0.0)), |cs| cs.margin_top() )
    }

    fn compute_margin_right(&self) -> CSSMargin {
        resolve(self, CSSMarginLength(Px(0.0)), |cs| cs.margin_right() )
    }

    fn compute_margin_bottom(&self) -> CSSMargin {
        resolve(self, CSSMarginLength(Px(0.0)), |cs| cs.margin_bottom() )
    }

    fn compute_margin_left(&self) -> CSSMargin {
        resolve(self, CSSMarginLength(Px(0.0)), |cs| cs.margin_left() )
    }

    fn compute_border_top_width(&self) -> CSSBorderWidth {
        resolve(self, CSSBorderWidthLength(Px(0.0)), |cs| cs.border_top_width() )
    }

    fn compute_border_right_width(&self) -> CSSBorderWidth {
        resolve(self, CSSBorderWidthLength(Px(0.0)), |cs| cs.border_right_width() )
    }

    fn compute_border_bottom_width(&self) -> CSSBorderWidth {
        resolve(self, CSSBorderWidthLength(Px(0.0)), |cs| cs.border_bottom_width() )
    }

    fn compute_border_left_width(&self) -> CSSBorderWidth {
        resolve(self, CSSBorderWidthLength(Px(0.0)), |cs| cs.border_left_width() )
    }

    fn compute_border_top_color(&self) -> Color {
        resolve(self, rgba(255, 255, 255, 0.0), |cs| cs.border_top_color() )
    }

    fn compute_border_right_color(&self) -> Color {
        resolve(self, rgba(255, 255, 255, 0.0), |cs| cs.border_right_color() )
    }

    fn compute_border_bottom_color(&self) -> Color {
        resolve(self, rgba(255, 255, 255, 0.0), |cs| cs.border_bottom_color() )
    }

    fn compute_border_left_color(&self) -> Color {
        resolve(self, rgba(255, 255, 255, 0.0), |cs| cs.border_left_color() )
    }

}

fn resolve<T>(node: &Node, default: T, get: &fn(cs: ComputedStyle) -> CSSValue<T>) -> T {
    let style = node.get_style();
    let computed = style.computed_style();
    let value = get(computed);
    match move value {
        Inherit => /* FIXME: need inheritance */ move default,
        Specified(move value) => move value,
    }
}