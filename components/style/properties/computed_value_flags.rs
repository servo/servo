/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

//! Misc information about a given computed style.

use properties::{ComputedValues, StyleBuilder};

bitflags! {
    /// Misc information about a given computed style.
    pub flags ComputedValueFlags: u8 {
        /// Whether the style or any of the ancestors has a text-decoration
        /// property that should get propagated to descendants.
        ///
        /// text-decoration is a reset property, but gets propagated in the
        /// frame/box tree.
        const HAS_TEXT_DECORATION_LINE = 1 << 0,
    }
}

impl ComputedValueFlags {
    /// Get the computed value flags for the initial style.
    pub fn initial() -> Self {
        Self::empty()
    }

    /// Compute the flags for this style, given the parent style.
    pub fn compute(
        this_style: &StyleBuilder,
        parent_style: &ComputedValues,
    ) -> Self {
        let mut ret = Self::empty();

        // FIXME(emilio): This feels like it wants to look at the
        // layout_parent_style, but the way it works in Gecko means it's not
        // needed (we'd recascade a bit more when it changes, but that's fine),
        // and this way it simplifies the code for text styles and similar.
        if parent_style.flags.contains(HAS_TEXT_DECORATION_LINE) ||
           !this_style.get_text().clone_text_decoration_line().is_empty() {
            ret.insert(HAS_TEXT_DECORATION_LINE);
        }

        ret
    }
}
