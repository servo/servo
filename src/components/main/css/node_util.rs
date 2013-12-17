/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use layout::incremental::RestyleDamage;
use layout::util::LayoutDataAccess;
use layout::wrapper::LayoutNode;

use extra::arc::Arc;
use std::cast;
use style::{ComputedValues, TNode};

pub trait NodeUtil {
    fn get_css_select_results<'a>(&'a self) -> &'a Arc<ComputedValues>;
    fn have_css_select_results(self) -> bool;

    fn get_restyle_damage(self) -> RestyleDamage;
    fn set_restyle_damage(self, damage: RestyleDamage);
}

impl<'self> NodeUtil for LayoutNode<'self> {
    /** 
     * Provides the computed style for the given node. If CSS selector
     * Returns the style results for the given node. If CSS selector
     * matching has not yet been performed, fails.
     */
    #[inline]
    fn get_css_select_results<'a>(&'a self) -> &'a Arc<ComputedValues> {
        unsafe {
            cast::transmute_region(self.borrow_layout_data_unchecked()
                                       .as_ref()
                                       .unwrap()
                                       .style
                                       .as_ref()
                                       .unwrap())
        }
    }

    /// Does this node have a computed style yet?
    fn have_css_select_results(self) -> bool {
        self.borrow_layout_data().ptr.as_ref().unwrap().style.is_some()
    }

    /// Get the description of how to account for recent style changes.
    /// This is a simple bitfield and fine to copy by value.
    fn get_restyle_damage(self) -> RestyleDamage {
        // For DOM elements, if we haven't computed damage yet, assume the worst.
        // Other nodes don't have styles.
        let default = if self.is_element() {
            RestyleDamage::all()
        } else {
            RestyleDamage::none()
        };

        self.borrow_layout_data()
            .ptr
            .as_ref()
            .unwrap()
            .restyle_damage
            .map(|x| RestyleDamage::from_int(x))
            .unwrap_or(default)
    }

    /// Set the restyle damage field.
    fn set_restyle_damage(self, damage: RestyleDamage) {
        match *self.mutate_layout_data().ptr {
            Some(ref mut data) => data.restyle_damage = Some(damage.to_int()),
            _ => fail!("no layout data for this node"),
        }
    }
}

