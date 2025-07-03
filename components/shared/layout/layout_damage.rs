/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use bitflags::bitflags;
use style::selector_parser::RestyleDamage;

bitflags! {
    /// Individual layout actions that may be necessary after restyling. This is an extension
    /// of `RestyleDamage` from stylo, which only uses the 4 lower bits.
    #[derive(Clone, Copy, Default, Debug, Eq, PartialEq)]
    pub struct LayoutDamage: u16 {
        /// Recollect the box children for this element, because some of the them will be
        /// rebuilt.
        const RECOLLECT_BOX_TREE_CHILDREN = 0b011111111111 << 4;
        /// Rebuild the entire box for this element, which means that every part of layout
        /// needs to happena again.
        const REBUILD_BOX = 0b111111111111 << 4;
    }
}

impl LayoutDamage {
    pub fn recollect_box_tree_children() -> RestyleDamage {
        RestyleDamage::from_bits_retain(LayoutDamage::RECOLLECT_BOX_TREE_CHILDREN.bits()) |
            RestyleDamage::RELAYOUT
    }

    pub fn has_box_damage(&self) -> bool {
        self.intersects(Self::REBUILD_BOX)
    }
}

impl std::fmt::Display for LayoutDamage {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> Result<(), std::fmt::Error> {
        let mut first_elem = true;

        let to_iter = [
            (LayoutDamage::RECOLLECT_BOX_TREE_CHILDREN, "Recollect box tree childern"),
            (LayoutDamage::REBUILD_BOX, "Rebuild box"),
        ];

        for &(damage, damage_str) in &to_iter {
            if self.contains(damage) {
                if !first_elem {
                    write!(f, " | ")?;
                }
                write!(f, "{}", damage_str)?;
                first_elem = false;
            }
        }

        if first_elem {
            write!(f, "NoDamage")?;
        }

        Ok(())
    }
}
