/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use incremental::RestyleDamage;
use util::LayoutDataAccess;
use wrapper::{TLayoutNode, ThreadSafeLayoutNode};
use wrapper::{After, AfterBlock, Before, BeforeBlock, Normal};
use std::mem;
use style::ComputedValues;
use sync::Arc;

pub trait NodeUtil {
    fn get_css_select_results<'a>(&'a self) -> &'a Arc<ComputedValues>;
    fn have_css_select_results(&self) -> bool;

    fn get_restyle_damage(&self) -> RestyleDamage;
    fn set_restyle_damage(&self, damage: RestyleDamage);
}

impl<'ln> NodeUtil for ThreadSafeLayoutNode<'ln> {
    /// Returns the style results for the given node. If CSS selector
    /// matching has not yet been performed, fails.
    #[inline]
    fn get_css_select_results<'a>(&'a self) -> &'a Arc<ComputedValues> {
        unsafe {
            let layout_data_ref = self.borrow_layout_data();
            match self.get_pseudo_element_type() {
                Before | BeforeBlock => {
                     mem::transmute(layout_data_ref.as_ref()
                                                   .unwrap()
                                                   .data
                                                   .before_style
                                                   .as_ref()
                                                   .unwrap())
                }
                After | AfterBlock => {
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
        layout_data_ref.get_ref().shared_data.style.is_some()
    }

    /// Get the description of how to account for recent style changes.
    /// This is a simple bitfield and fine to copy by value.
    fn get_restyle_damage(&self) -> RestyleDamage {
        // For DOM elements, if we haven't computed damage yet, assume the worst.
        // Other nodes don't have styles.
        let default = if self.node_is_element() {
            RestyleDamage::all()
        } else {
            RestyleDamage::empty()
        };

        let layout_data_ref = self.borrow_layout_data();
        layout_data_ref
            .get_ref()
            .data
            .restyle_damage
            .unwrap_or(default)
    }

    /// Set the restyle damage field.
    fn set_restyle_damage(&self, damage: RestyleDamage) {
        let mut layout_data_ref = self.mutate_layout_data();
        match &mut *layout_data_ref {
            &Some(ref mut layout_data) => layout_data.data.restyle_damage = Some(damage),
            _ => fail!("no layout data for this node"),
        }
    }
}
