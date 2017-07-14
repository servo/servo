/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

//! Misc information about a given computed style.

bitflags! {
    /// Misc information about a given computed style.
    ///
    /// All flags are currently inherited for text, pseudo elements, and
    /// anonymous boxes, see StyleBuilder::for_inheritance and its callsites.
    /// If we ever want to add some flags that shouldn't inherit for them,
    /// we might want to add a function to handle this.
    pub flags ComputedValueFlags: u8 {
        /// Whether the style or any of the ancestors has a text-decoration-line
        /// property that should get propagated to descendants.
        ///
        /// text-decoration-line is a reset property, but gets propagated in the
        /// frame/box tree.
        const HAS_TEXT_DECORATION_LINES = 1 << 0,

        /// Whether line break inside should be suppressed.
        ///
        /// If this flag is set, the line should not be broken inside,
        /// which means inlines act as if nowrap is set, <br> element is
        /// suppressed, and blocks are inlinized.
        ///
        /// This bit is propagated to all children of line participants.
        /// It is currently used by ruby to make its content unbreakable.
        const SHOULD_SUPPRESS_LINEBREAK = 1 << 1,
    }
}
