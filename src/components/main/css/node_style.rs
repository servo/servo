/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

// Style retrieval from DOM elements.

use css::node_util::NodeUtil;
use layout::incremental::RestyleDamage;

use style::ComputedValues;
use script::dom::node::{AbstractNode, LayoutView};

/// Node mixin providing `style` method that returns a `NodeStyle`
pub trait StyledNode {
    fn style(&self) -> &ComputedValues;
    fn restyle_damage(&self) -> RestyleDamage;
}

impl StyledNode for AbstractNode<LayoutView> {
    #[inline]
    fn style(&self) -> &ComputedValues {
        // FIXME(pcwalton): Is this assertion needed for memory safety? It's slow.
        //assert!(self.is_element()); // Only elements can have styles
        let results = self.get_css_select_results();
        results
    }

    fn restyle_damage(&self) -> RestyleDamage {
        self.get_restyle_damage()
    }
}
