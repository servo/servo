/*!
Calculate styles for Nodes based on SelectResults
*/

use dom::node::Node;
use newcss::color::{Color, rgba};
use newcss::values::{CSSValue, Specified, Inherit};
use newcss::ComputedStyle;

pub trait ComputeStyles {
    fn compute_background_color(&self) -> Color;
}

impl Node: ComputeStyles {
    fn compute_background_color(&self) -> Color {
        compute(self, |cs| cs.background_color(), rgba(0, 0, 0, 0.0))
    }
}

fn compute<T>(node: &Node, get: &fn(cs: ComputedStyle) -> CSSValue<T>, default: T) -> T {
    let style = node.get_style();
    let computed = style.computed_style();
    let value = get(computed);
    match move value {
        Inherit => /* FIXME */ move default,
        Specified(move value) => move value,
        _ => fail
    }
}