/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use bitflags::bitflags;
use style::selector_parser::RestyleDamage;

bitflags! {
    /// Individual layout actions that may be necessary after restyling. This is an extension
    /// of `RestyleDamage` from stylo, which only uses the 4 lower bits.
    #[derive(Clone, Copy, Default, Eq, PartialEq)]
    pub struct LayoutDamage: u16 {
        /// Clear the cached inline content sizes and recompute them during the next layout.
        const RECOMPUTE_INLINE_CONTENT_SIZES = 0b1000_0000_0000 << 4;
        /// Rebuild this box and all of its ancestors. Do not rebuild any children. This
        /// is used when a box's content (such as text content) changes or a descendant
        /// has box damage ([`Self::BOX_DAMAGE`]).
        const DESCENDANT_HAS_BOX_DAMAGE = 0b0111_1111_1111 << 4;
        /// Rebuild this box, all of its ancestors and all of its descendants. This is the
        /// most a box can be damaged.
        const BOX_DAMAGE = 0b1111_1111_1111 << 4;
    }
}

impl LayoutDamage {
    pub fn descendant_has_box_damage() -> RestyleDamage {
        RestyleDamage::from_bits_retain(LayoutDamage::DESCENDANT_HAS_BOX_DAMAGE.bits())
    }

    pub fn box_damage() -> RestyleDamage {
        RestyleDamage::from_bits_retain(LayoutDamage::BOX_DAMAGE.bits())
    }

    pub fn needs_new_box(&self) -> bool {
        self.contains(Self::DESCENDANT_HAS_BOX_DAMAGE)
    }

    pub fn recompute_inline_content_sizes() -> RestyleDamage {
        RestyleDamage::from_bits_retain(LayoutDamage::RECOMPUTE_INLINE_CONTENT_SIZES.bits())
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

impl std::fmt::Debug for LayoutDamage {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if self.contains(Self::BOX_DAMAGE) {
            f.write_str("REBUILD_BOX")
        } else if self.contains(Self::DESCENDANT_HAS_BOX_DAMAGE) {
            f.write_str("RECOLLECT_BOX_TREE_CHILDREN")
        } else {
            f.write_str("EMPTY")
        }
    }
}
