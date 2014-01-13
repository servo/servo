/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

// Style retrieval from DOM elements.

use css::node_util::NodeUtil;
use layout::incremental::RestyleDamage;
use layout::wrapper::LayoutNode;

use extra::arc::Arc;
use style::ComputedValues;

/// Node mixin providing `style` method that returns a `NodeStyle`
pub trait StyledNode {
    fn style<'a>(&'a self) -> &'a Arc<ComputedValues>;
    fn restyle_damage(&self) -> RestyleDamage;
}

impl<'ln> StyledNode for LayoutNode<'ln> {
    #[inline]
    fn style<'a>(&'a self) -> &'a Arc<ComputedValues> {
        self.get_css_select_results()
    }

    fn restyle_damage(&self) -> RestyleDamage {
        self.get_restyle_damage()
    }
}
