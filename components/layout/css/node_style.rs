/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

// Style retrieval from DOM elements.

use css::node_util::NodeUtil;
use wrapper::ThreadSafeLayoutNode;

use style::ComputedValues;
use sync::Arc;

/// Node mixin providing `style` method that returns a `NodeStyle`
pub trait StyledNode {
    fn style<'a>(&'a self) -> &'a Arc<ComputedValues>;
    fn unstyle(self);
}

impl<'ln> StyledNode for ThreadSafeLayoutNode<'ln> {
    #[inline]
    fn style<'a>(&'a self) -> &'a Arc<ComputedValues> {
        self.get_css_select_results()
    }

    fn unstyle(self) {
        self.remove_css_select_results()
    }
}
