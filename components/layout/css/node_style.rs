/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

//! Style retrieval from DOM elements.

use wrapper::{PseudoElementType, ThreadSafeLayoutNode};

use std::mem;
use style::properties::ComputedValues;
use std::sync::Arc;

/// Node mixin providing `style` method that returns a `NodeStyle`
pub trait StyledNode {
    /// Returns the style results for the given node. If CSS selector matching has not yet been
    /// performed, fails.
    fn style<'a>(&'a self) -> &'a Arc<ComputedValues>;
    /// Does this node have a computed style yet?
    fn has_style(&self) -> bool;
    /// Removes the style from this node.
    fn unstyle(self);
}

impl<'ln> StyledNode for ThreadSafeLayoutNode<'ln> {
    #[inline]
    #[allow(unsafe_blocks)]
    fn style<'a>(&'a self) -> &'a Arc<ComputedValues> {
        unsafe {
            let layout_data_ref = self.borrow_layout_data();
            match self.get_pseudo_element_type() {
                PseudoElementType::Before(_) => {
                     mem::transmute(layout_data_ref.as_ref()
                                                   .unwrap()
                                                   .data
                                                   .before_style
                                                   .as_ref()
                                                   .unwrap())
                }
                PseudoElementType::After(_) => {
                    mem::transmute(layout_data_ref.as_ref()
                                                  .unwrap()
                                                  .data
                                                  .after_style
                                                  .as_ref()
                                                  .unwrap())
                }
                PseudoElementType::Normal => {
                    mem::transmute(layout_data_ref.as_ref()
                                                  .unwrap()
                                                  .shared_data
                                                  .style
                                                  .as_ref()
                                                  .unwrap())
                }
            }
        }
    }

    fn has_style(&self) -> bool {
        let layout_data_ref = self.borrow_layout_data();
        layout_data_ref.as_ref().unwrap().shared_data.style.is_some()
    }

    fn unstyle(self) {
        let mut layout_data_ref = self.mutate_layout_data();
        let layout_data = layout_data_ref.as_mut().expect("no layout data");

        let style =
            match self.get_pseudo_element_type() {
                PseudoElementType::Before(_) => &mut layout_data.data.before_style,
                PseudoElementType::After (_) => &mut layout_data.data.after_style,
                PseudoElementType::Normal    => &mut layout_data.shared_data.style,
            };

        *style = None;
    }
}
