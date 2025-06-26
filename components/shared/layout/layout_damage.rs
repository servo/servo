/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use bitflags::bitflags;

bitflags! {
    /// Individual layout actions that may be necessary after restyling. This is an extension
    /// of `RestyleDamage` from stylo, which only uses the 4 lower bits.
    #[derive(Clone, Copy, Debug, Eq, PartialEq)]
    pub struct LayoutDamage: u16 {
        /// Rebuild the entire box for this element, which means that every part of layout
        /// needs to happena again.
        const REBUILD_BOX = 0b111111111111 << 4;
    }
}
