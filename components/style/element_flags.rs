/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */
use selectors::matching::StyleRelations;

bitflags! {
    /// Set of flags that are set on an element depending on what selectors does
    /// the element or its children match.
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
        /// When this element is affected by the empty selector.
        const HAS_EMPTY_SELECTOR = 1 << 3,
    }
}

impl ElementFlags {
    /// Returns the two sets of flags (one for the child, one for the parent)
    /// generated for this set of relations.
    #[inline]
    pub fn from_relations(relations: StyleRelations) -> (Self, Self) {
        use selectors::matching::*;
        let mut child = Self::empty();
        let mut parent = Self::empty();
        if relations.intersects(AFFECTED_BY_EMPTY) {
            // FIXME(emilio): This doesn't need to be atomic.
            child.insert(HAS_EMPTY_SELECTOR);
        }

        if relations.intersects(AFFECTED_BY_NTH_LAST_CHILD |
                                AFFECTED_BY_LAST_OF_TYPE |
                                AFFECTED_BY_NTH_LAST_OF_TYPE |
                                AFFECTED_BY_ONLY_OF_TYPE) {
            parent.insert(HAS_SLOW_SELECTOR)
        }

        if relations.intersects(AFFECTED_BY_FIRST_CHILD |
                                AFFECTED_BY_LAST_CHILD |
                                AFFECTED_BY_ONLY_CHILD) {
            parent.insert(HAS_EDGE_CHILD_SELECTOR)
        }

        if relations.intersects(AFFECTED_BY_NTH_CHILD |
                                AFFECTED_BY_FIRST_OF_TYPE |
                                AFFECTED_BY_NTH_OF_TYPE) {
            parent.insert(HAS_SLOW_SELECTOR_LATER_SIBLINGS)
        }

        (child, parent)
    }
}
