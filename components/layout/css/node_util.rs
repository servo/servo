/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use util::LayoutDataAccess;
use wrapper::ThreadSafeLayoutNode;
use wrapper::{After, Before, Normal};

use std::mem;
use style::ComputedValues;
use sync::Arc;

pub trait NodeUtil {
    fn get_css_select_results<'a>(&'a self) -> &'a Arc<ComputedValues>;
    fn have_css_select_results(&self) -> bool;
    fn remove_css_select_results(self);
}

impl<'ln> NodeUtil for ThreadSafeLayoutNode<'ln> {
    /// Returns the style results for the given node. If CSS selector
    /// matching has not yet been performed, fails.
    #[inline]
    fn get_css_select_results<'a>(&'a self) -> &'a Arc<ComputedValues> {
        unsafe {
            let layout_data_ref = self.borrow_layout_data();
            match self.get_pseudo_element_type() {
                Before(_) => {
                     mem::transmute(layout_data_ref.as_ref()
                                                   .unwrap()
                                                   .data
                                                   .before_style
                                                   .as_ref()
                                                   .unwrap())
                }
                After(_) => {
                    mem::transmute(layout_data_ref.as_ref()
                                                  .unwrap()
                                                  .data
                                                  .after_style
                                                  .as_ref()
                                                  .unwrap())
                }
                Normal => {
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

    /// Does this node have a computed style yet?
    fn have_css_select_results(&self) -> bool {
        let layout_data_ref = self.borrow_layout_data();
        layout_data_ref.as_ref().unwrap().shared_data.style.is_some()
    }

    fn remove_css_select_results(self) {
        let mut layout_data_ref = self.mutate_layout_data();
        let layout_data = layout_data_ref.as_mut().expect("no layout data");

        let style =
            match self.get_pseudo_element_type() {
                Before(_) => &mut layout_data.data.before_style,
                After (_) => &mut layout_data.data.after_style,
                Normal    => &mut layout_data.shared_data.style,
            };

        *style = None;
    }
}

