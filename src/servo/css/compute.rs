/*!
Calculate styles for nodes based on SelectResults
*/

use dom::node::Node;
use newcss::color::{Color, rgba};

pub trait ComputeStyles {
    fn compute_background_color() -> Color;
}

impl Node: ComputeStyles {
    fn compute_background_color() -> Color {
        rgba(255, 0, 255, 1.0)
    }
}