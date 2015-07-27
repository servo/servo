/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

//! Style retrieval from DOM elements.

use data::LayoutDataWrapper;
use wrapper::{PseudoElementType, ThreadSafeLayoutNode};

use style::properties::ComputedValues;

use std::cell::Ref;
use std::sync::Arc;

/// Node mixin providing `style` method that returns a `NodeStyle`
pub trait StyledNode {
    fn get_style<'a>(&self, layout_data_ref: &'a LayoutDataWrapper) -> &'a Arc<ComputedValues>;
    /// Returns the style results for the given node. If CSS selector matching has not yet been
    /// performed, fails.
    fn style<'a>(&'a self) -> Ref<'a, Arc<ComputedValues>>;
    /// Removes the style from this node.
    fn unstyle(self);
}

impl<'ln> StyledNode for ThreadSafeLayoutNode<'ln> {
    #[inline]
    fn get_style<'a>(&self, layout_data_ref: &'a LayoutDataWrapper) -> &'a Arc<ComputedValues> {
        match self.get_pseudo_element_type() {
            PseudoElementType::Before(_) => layout_data_ref.data.before_style.as_ref().unwrap(),
            PseudoElementType::After(_) => layout_data_ref.data.after_style.as_ref().unwrap(),
            PseudoElementType::Normal => layout_data_ref.shared_data.style.as_ref().unwrap(),
        }
    }

    #[inline]
    fn style<'a>(&'a self) -> Ref<'a, Arc<ComputedValues>> {
        Ref::map(self.borrow_layout_data(), |layout_data_ref| {
            let layout_data = layout_data_ref.as_ref().expect("no layout data");
            self.get_style(layout_data)
        })
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
