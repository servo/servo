/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use layout::aux::LayoutAuxMethods;
use layout::incremental::RestyleDamage;

use std::cast::transmute;
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
        if !self.has_layout_data() {
            fail!(~"style() called on a node without aux data!");
        }

        match self.layout_data().style {
            None => fail!(~"style() called on node without a style!"),
            Some(ref style) => unsafe { transmute(style) }
        }
    }

    /// Does this node have a computed style yet?
    fn have_css_select_results(self) -> bool {
        self.has_layout_data() && self.layout_data().style.is_some()
    }

    /// Update the computed style of an HTML element with a style specified by CSS.
    fn set_css_select_results(self, decl: CompleteSelectResults) {
        if !self.has_layout_data() {
            fail!(~"set_css_select_results() called on a node without aux data!");
        }

        self.layout_data().style = Some(decl);
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

        if !self.has_layout_data() {
            return default;
        }
        self.layout_data().restyle_damage.unwrap_or_default(default)
    }

    /// Set the restyle damage field.
    fn set_restyle_damage(self, damage: RestyleDamage) {
        if !self.has_layout_data() {
            fail!(~"set_restyle_damage() called on a node without aux data!");
        }

        self.layout_data().restyle_damage = Some(damage);
    }
}
