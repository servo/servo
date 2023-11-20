/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

//! Restyle hints: an optimization to avoid unnecessarily matching selectors.

use crate::traversal_flags::TraversalFlags;

bitflags! {
    /// The kind of restyle we need to do for a given element.
    #[repr(C)]
    pub struct RestyleHint: u16 {
        /// Do a selector match of the element.
        const RESTYLE_SELF = 1 << 0;

        /// Do a selector match of the element's pseudo-elements. Always to be combined with
        /// RESTYLE_SELF.
        const RESTYLE_PSEUDOS = 1 << 1;

        /// Do a selector match if the element is a pseudo-element.
        const RESTYLE_SELF_IF_PSEUDO = 1 << 2;

        /// Do a selector match of the element's descendants.
        const RESTYLE_DESCENDANTS = 1 << 3;

        /// Recascade the current element.
        const RECASCADE_SELF = 1 << 4;

        /// Recascade the current element if it inherits any reset style.
        const RECASCADE_SELF_IF_INHERIT_RESET_STYLE = 1 << 5;

        /// Recascade all descendant elements.
        const RECASCADE_DESCENDANTS = 1 << 6;

        /// Replace the style data coming from CSS transitions without updating
        /// any other style data. This hint is only processed in animation-only
        /// traversal which is prior to normal traversal.
        const RESTYLE_CSS_TRANSITIONS = 1 << 7;

        /// Replace the style data coming from CSS animations without updating
        /// any other style data. This hint is only processed in animation-only
        /// traversal which is prior to normal traversal.
        const RESTYLE_CSS_ANIMATIONS = 1 << 8;

        /// Don't re-run selector-matching on the element, only the style
        /// attribute has changed, and this change didn't have any other
        /// dependencies.
        const RESTYLE_STYLE_ATTRIBUTE = 1 << 9;

        /// Replace the style data coming from SMIL animations without updating
        /// any other style data. This hint is only processed in animation-only
        /// traversal which is prior to normal traversal.
        const RESTYLE_SMIL = 1 << 10;
    }
}

impl RestyleHint {
    /// Creates a new `RestyleHint` indicating that the current element and all
    /// its descendants must be fully restyled.
    pub fn restyle_subtree() -> Self {
        RestyleHint::RESTYLE_SELF | RestyleHint::RESTYLE_DESCENDANTS
    }

    /// Creates a new `RestyleHint` indicating that the current element and all
    /// its descendants must be recascaded.
    pub fn recascade_subtree() -> Self {
        RestyleHint::RECASCADE_SELF | RestyleHint::RECASCADE_DESCENDANTS
    }

    /// Returns whether this hint invalidates the element and all its
    /// descendants.
    pub fn contains_subtree(&self) -> bool {
        self.contains(Self::restyle_subtree())
    }

    /// Returns whether we'll recascade all of the descendants.
    pub fn will_recascade_subtree(&self) -> bool {
        self.contains_subtree() || self.contains(Self::recascade_subtree())
    }

    /// Returns whether we need to restyle this element.
    pub fn has_non_animation_invalidations(&self) -> bool {
        !(*self & !Self::for_animations()).is_empty()
    }

    /// Propagates this restyle hint to a child element.
    pub fn propagate(&mut self, traversal_flags: &TraversalFlags) -> Self {
        use std::mem;

        // In the middle of an animation only restyle, we don't need to
        // propagate any restyle hints, and we need to remove ourselves.
        if traversal_flags.for_animation_only() {
            self.remove_animation_hints();
            return Self::empty();
        }

        debug_assert!(
            !self.has_animation_hint(),
            "There should not be any animation restyle hints \
             during normal traversal"
        );

        // Else we should clear ourselves, and return the propagated hint.
        mem::replace(self, Self::empty()).propagate_for_non_animation_restyle()
    }

    /// Returns a new `RestyleHint` appropriate for children of the current element.
    fn propagate_for_non_animation_restyle(&self) -> Self {
        if self.contains(RestyleHint::RESTYLE_DESCENDANTS) {
            return Self::restyle_subtree();
        }
        let mut result = Self::empty();
        if self.contains(RestyleHint::RESTYLE_PSEUDOS) {
            result |= Self::RESTYLE_SELF_IF_PSEUDO;
        }
        if self.contains(RestyleHint::RECASCADE_DESCENDANTS) {
            result |= Self::recascade_subtree();
        }
        result
    }

    /// Returns a hint that contains all the replacement hints.
    pub fn replacements() -> Self {
        RestyleHint::RESTYLE_STYLE_ATTRIBUTE | Self::for_animations()
    }

    /// The replacements for the animation cascade levels.
    #[inline]
    pub fn for_animations() -> Self {
        RestyleHint::RESTYLE_SMIL |
            RestyleHint::RESTYLE_CSS_ANIMATIONS |
            RestyleHint::RESTYLE_CSS_TRANSITIONS
    }

    /// Returns whether the hint specifies that an animation cascade level must
    /// be replaced.
    #[inline]
    pub fn has_animation_hint(&self) -> bool {
        self.intersects(Self::for_animations())
    }

    /// Returns whether the hint specifies that an animation cascade level must
    /// be replaced.
    #[inline]
    pub fn has_animation_hint_or_recascade(&self) -> bool {
        self.intersects(Self::for_animations() | Self::RECASCADE_SELF | Self::RECASCADE_SELF_IF_INHERIT_RESET_STYLE)
    }

    /// Returns whether the hint specifies some restyle work other than an
    /// animation cascade level replacement.
    #[inline]
    pub fn has_non_animation_hint(&self) -> bool {
        !(*self & !Self::for_animations()).is_empty()
    }

    /// Returns whether the hint specifies that some cascade levels must be
    /// replaced.
    #[inline]
    pub fn has_replacements(&self) -> bool {
        self.intersects(Self::replacements())
    }

    /// Removes all of the animation-related hints.
    #[inline]
    pub fn remove_animation_hints(&mut self) {
        self.remove(Self::for_animations());

        // While RECASCADE_SELF is not animation-specific, we only ever add and process it during
        // traversal.  If we are here, removing animation hints, then we are in an animation-only
        // traversal, and we know that any RECASCADE_SELF flag must have been set due to changes in
        // inherited values after restyling for animations, and thus we want to remove it so that
        // we don't later try to restyle the element during a normal restyle.
        // (We could have separate RECASCADE_SELF_NORMAL and RECASCADE_SELF_ANIMATIONS flags to
        // make it clear, but this isn't currently necessary.)
        self.remove(Self::RECASCADE_SELF | Self::RECASCADE_SELF_IF_INHERIT_RESET_STYLE);
    }
}

impl Default for RestyleHint {
    fn default() -> Self {
        Self::empty()
    }
}

#[cfg(feature = "servo")]
malloc_size_of_is_0!(RestyleHint);
