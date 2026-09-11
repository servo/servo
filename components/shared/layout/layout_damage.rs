/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use bitflags::bitflags;
use style::selector_parser::RestyleDamage;

bitflags! {
    /// Individual layout actions that may be necessary after restyling. This is an extension
    /// of `RestyleDamage` from stylo, which only uses the 4 lower bits.
    #[derive(Clone, Copy, Debug,Default, Eq, PartialEq)]
    pub struct LayoutDamage: u16 {
        // Layout Modes
        //
        // These should be kept in sync with the layout modes defined in Stylo's `RestyleDamage`.
        // The entire damage machinery depends on `LayoutDamage` being a superset of `RestyleDamage`.
        /// Repaint the node itself.
        const Repaint = 0b0001;
        /// Rebuilds the stacking contexts.
        const RebuildStackingContextTree = 0b0011;
        /// Recalculates the scrollable overflow.
        const RecalculateOverflow = 0b0111;
        /// Any other type of damage, which requires running layout again.
        const Relayout = 0b1111;

        // Layout-specific damage
        /// Clear the cached inline content sizes and recompute them during the next layout.
        const RecomputeInlineContentSizes = 0b1000_0000_0000_0000;
        /// A descendant was collected as a layout root for fragment tree layout.
        const DescendantCollectedAsLayoutRoot = 0b0100_0000_0000_0000;
        /// Rebuild this box and all of its ancestors. Do not rebuild any children. This
        /// is used when a box's content (such as text content) changes or a descendant
        /// has box damage ([`Self::BOX_DAMAGE`]).
        const DescendantHasBoxDamage = 0b0011_0000_0000_0000;

        /// Rebuild this box, all of its ancestors and all of its descendants. This is the
        /// most a box can be damaged.
        const BoxDamage = 0b1111_1111_0000_0000;

        // Accessibility-specific damage
        //
        // These should be kept in sync with the layout-related values in `AccessibilityDamage`

        /// Corresponds to [`AccessibilityDamage::Layout`].
        const HasAccessibilityDamage = 0b0000_0000_0001_0000;
        /// Corresponds to [`AccessibilityDamage::DescendantHasDamageFromLayout`].
        const DescendantHasAccessibilityDamage = 0b0000_0000_1000_0000;
    }
}

impl LayoutDamage {
    pub fn only_layout_modes(&self) -> LayoutDamage {
        self.intersection(LayoutDamage::Relayout)
    }
}

impl From<RestyleDamage> for LayoutDamage {
    fn from(restyle_damage: RestyleDamage) -> Self {
        LayoutDamage::from_bits_retain(restyle_damage.bits())
    }
}

impl From<LayoutDamage> for RestyleDamage {
    fn from(layout_damage: LayoutDamage) -> Self {
        RestyleDamage::from_bits_retain(layout_damage.bits())
    }
}

bitflags! {
    #[derive(Clone, Copy, Default, Debug, Eq, PartialEq)]
    pub struct AccessibilityDamage: u16 {
        /// The properties of this node (other than children) have changed.
        const Node = 0b0001;
        /// Children have been added to or removed from this node.
        const Children = 0b0010;

        // Layout-related values: keep in sync with accessibility-related values in `LayoutDamage`.

        /// This node's box(es) was recomputed during layout.
        const Layout = 0b0000_0000_0001_0000;
        /// A descendent of this node has damage from layout.
        const DescendantHasDamageFromLayout = 0b0000_0000_1000_0000;

        /// All properties of this node need to be recomputed.
        const Rebuild = 0b0000_0000_0001_0011;
    }
}
malloc_size_of::malloc_size_of_is_0!(AccessibilityDamage);

impl From<RestyleDamage> for AccessibilityDamage {
    fn from(restyle_damage: RestyleDamage) -> Self {
        AccessibilityDamage::from_bits_retain(restyle_damage.bits())
    }
}

impl From<AccessibilityDamage> for RestyleDamage {
    fn from(accessibility_damage: AccessibilityDamage) -> Self {
        RestyleDamage::from_bits_retain(accessibility_damage.bits())
    }
}
