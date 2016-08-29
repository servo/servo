/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */
bitflags! {
    /// Set of flags that are set on the parent element depending on whether a
    /// child matches a selector.
    ///
    /// These setters, in the case of Servo, must be atomic, due to the parallel
    /// traversal.
    pub flags ElementFlags: u8 {
        /// When a child is added or removed from this element, all the children
        /// must be restyled, because they may match :nth-last-child,
        /// :last-of-type, :nth-last-of-type, or :only-of-type.
        const HAS_SLOW_SELECTOR = 1 << 0,
        /// When a child is added or removed from this element, any later
        /// children must be restyled, because they may match :nth-child,
        /// :first-of-type, or :nth-of-type.
        const HAS_SLOW_SELECTOR_LATER_SIBLINGS = 1 << 1,
        /// When a child is added or removed from this element, the first and
        /// last children must be restyled, because they may match :first-child,
        /// :last-child, or :only-child.
        const HAS_EDGE_CHILD_SELECTOR = 1 << 2,
    }
}
