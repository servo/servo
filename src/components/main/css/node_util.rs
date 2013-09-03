/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use layout::incremental::RestyleDamage;

use std::cast;
use std::cell::Cell;
use newcss::complete::CompleteSelectResults;
use script::dom::node::{AbstractNode, LayoutView};

pub trait NodeUtil<'self> {
    fn get_css_select_results(self) -> &'self CompleteSelectResults;
    fn set_css_select_results(self, decl: CompleteSelectResults);
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
    fn get_css_select_results(self) -> &'self CompleteSelectResults {
        do self.read_layout_data |layout_data| {
            match layout_data.style {
                None => fail!(~"style() called on node without a style!"),
                Some(ref style) => unsafe { cast::transmute_region(style) }
            }
        }
    }

    /// Does this node have a computed style yet?
    fn have_css_select_results(self) -> bool {
        self.read_layout_data(|data| data.style.is_some())
    }

    /// Update the computed style of an HTML element with a style specified by CSS.
    fn set_css_select_results(self, decl: CompleteSelectResults) {
        let cell = Cell::new(decl);
        self.write_layout_data(|data| data.style = Some(cell.take()));
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

        do self.read_layout_data |layout_data| {
            layout_data.restyle_damage
                .map(|&x| RestyleDamage::from_int(x))
                .unwrap_or_default(default)
        }
    }

    /// Set the restyle damage field.
    fn set_restyle_damage(self, damage: RestyleDamage) {
        self.write_layout_data(|data| data.restyle_damage = Some(damage.to_int()));
    }
}
