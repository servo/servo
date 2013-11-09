/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use layout::incremental::RestyleDamage;
use layout::util::LayoutDataAccess;

use std::cast;
use style::ComputedValues;
use script::dom::node::{AbstractNode, LayoutView};
use servo_util::tree::TreeNodeRef;

pub trait NodeUtil<'self> {
    fn get_css_select_results(self) -> &'self ComputedValues;
    fn set_css_select_results(self, decl: ComputedValues);
    fn have_css_select_results(self) -> bool;

    fn get_restyle_damage(self) -> RestyleDamage;
    fn set_restyle_damage(self, damage: RestyleDamage);
}

impl<'self> NodeUtil<'self> for AbstractNode<LayoutView> {
    /** 
     * Provides the computed style for the given node. If CSS selector
     * Returns the style results for the given node. If CSS selector
     * matching has not yet been performed, fails.
     * FIXME: This isn't completely memory safe since the style is
     * stored in a box that can be overwritten
     */
    fn get_css_select_results(self) -> &'self ComputedValues {
        let layout_data = self.layout_data();
        match *layout_data.style.borrow().ptr {
            None => fail!(~"style() called on node without a style!"),
            Some(ref style) => unsafe { cast::transmute_region(style) }
        }
    }

    /// Does this node have a computed style yet?
    fn have_css_select_results(self) -> bool {
        self.layout_data().style.borrow().ptr.is_some()
    }

    /// Update the computed style of an HTML element with a style specified by CSS.
    fn set_css_select_results(self, decl: ComputedValues) {
        *self.layout_data().style.mutate().ptr = Some(decl)
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

        self.layout_data()
            .restyle_damage
            .borrow()
            .ptr
            .map(|x| RestyleDamage::from_int(x))
            .unwrap_or(default)
    }

    /// Set the restyle damage field.
    fn set_restyle_damage(self, damage: RestyleDamage) {
        *self.layout_data().restyle_damage.mutate().ptr = Some(damage.to_int())
    }
}

