/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

//! Restyle hints: an optimization to avoid unnecessarily matching selectors.

#[cfg(feature = "gecko")]
use gecko_bindings::structs::nsRestyleHint;
use traversal_flags::TraversalFlags;

bitflags! {
    /// The kind of restyle we need to do for a given element.
    pub struct RestyleHint: u8 {
        /// Do a selector match of the element.
        const RESTYLE_SELF = 1 << 0;

        /// Do a selector match of the element's descendants.
        const RESTYLE_DESCENDANTS = 1 << 1;

        /// Recascade the current element.
        const RECASCADE_SELF = 1 << 2;

        /// Recascade all descendant elements.
        const RECASCADE_DESCENDANTS = 1 << 3;

        /// Replace the style data coming from CSS transitions without updating
        /// any other style data. This hint is only processed in animation-only
        /// traversal which is prior to normal traversal.
        const RESTYLE_CSS_TRANSITIONS = 1 << 4;

        /// Replace the style data coming from CSS animations without updating
        /// any other style data. This hint is only processed in animation-only
        /// traversal which is prior to normal traversal.
        const RESTYLE_CSS_ANIMATIONS = 1 << 5;

        /// Don't re-run selector-matching on the element, only the style
        /// attribute has changed, and this change didn't have any other
        /// dependencies.
        const RESTYLE_STYLE_ATTRIBUTE = 1 << 6;

        /// Replace the style data coming from SMIL animations without updating
        /// any other style data. This hint is only processed in animation-only
        /// traversal which is prior to normal traversal.
        const RESTYLE_SMIL = 1 << 7;
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
        self.contains(RestyleHint::RESTYLE_SELF | RestyleHint::RESTYLE_DESCENDANTS)
    }

    /// Returns whether we need to restyle this element.
    pub fn has_non_animation_invalidations(&self) -> bool {
        self.intersects(
            RestyleHint::RESTYLE_SELF |
                RestyleHint::RECASCADE_SELF |
                (Self::replacements() & !Self::for_animations()),
        )
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

    /// Returns a new `CascadeHint` appropriate for children of the current
    /// element.
    fn propagate_for_non_animation_restyle(&self) -> Self {
        if self.contains(RestyleHint::RESTYLE_DESCENDANTS) {
            return Self::restyle_subtree();
        }
        if self.contains(RestyleHint::RECASCADE_DESCENDANTS) {
            return Self::recascade_subtree();
        }
        Self::empty()
    }

    /// Creates a new `RestyleHint` that indicates the element must be
    /// recascaded.
    pub fn recascade_self() -> Self {
        RestyleHint::RECASCADE_SELF
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

    /// Returns whether the hint specifies that the currently element must be
    /// recascaded.
    pub fn has_recascade_self(&self) -> bool {
        self.contains(RestyleHint::RECASCADE_SELF)
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
        self.intersects(Self::for_animations() | RestyleHint::RECASCADE_SELF)
    }

    /// Returns whether the hint specifies some restyle work other than an
    /// animation cascade level replacement.
    #[inline]
    pub fn has_non_animation_hint(&self) -> bool {
        !(*self & !Self::for_animations()).is_empty()
    }

    /// Returns whether the hint specifies that selector matching must be re-run
    /// for the element.
    #[inline]
    pub fn match_self(&self) -> bool {
        self.intersects(RestyleHint::RESTYLE_SELF)
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

        // While RECASCADE_SELF is not animation-specific, we only ever add and
        // process it during traversal.  If we are here, removing animation
        // hints, then we are in an animation-only traversal, and we know that
        // any RECASCADE_SELF flag must have been set due to changes in
        // inherited values after restyling for animations, and thus we want to
        // remove it so that we don't later try to restyle the element during a
        // normal restyle.  (We could have separate RECASCADE_SELF_NORMAL and
        // RECASCADE_SELF_ANIMATIONS flags to make it clear, but this isn't
        // currently necessary.)
        self.remove(RestyleHint::RECASCADE_SELF);
    }
}

impl Default for RestyleHint {
    fn default() -> Self {
        Self::empty()
    }
}

#[cfg(feature = "gecko")]
impl From<nsRestyleHint> for RestyleHint {
    fn from(mut raw: nsRestyleHint) -> Self {
        use gecko_bindings::structs::nsRestyleHint_eRestyle_Force as eRestyle_Force;
        use gecko_bindings::structs::nsRestyleHint_eRestyle_ForceDescendants as eRestyle_ForceDescendants;
        use gecko_bindings::structs::nsRestyleHint_eRestyle_LaterSiblings as eRestyle_LaterSiblings;
        use gecko_bindings::structs::nsRestyleHint_eRestyle_Self as eRestyle_Self;
        use gecko_bindings::structs::nsRestyleHint_eRestyle_SomeDescendants as eRestyle_SomeDescendants;
        use gecko_bindings::structs::nsRestyleHint_eRestyle_Subtree as eRestyle_Subtree;

        let mut hint = RestyleHint::empty();

        debug_assert!(
            raw.0 & eRestyle_LaterSiblings.0 == 0,
            "Handle later siblings manually if necessary plz."
        );

        if (raw.0 & (eRestyle_Self.0 | eRestyle_Subtree.0)) != 0 {
            raw.0 &= !eRestyle_Self.0;
            hint.insert(RestyleHint::RESTYLE_SELF);
        }

        if (raw.0 & (eRestyle_Subtree.0 | eRestyle_SomeDescendants.0)) != 0 {
            raw.0 &= !eRestyle_Subtree.0;
            raw.0 &= !eRestyle_SomeDescendants.0;
            hint.insert(RestyleHint::RESTYLE_DESCENDANTS);
        }

        if (raw.0 & (eRestyle_ForceDescendants.0 | eRestyle_Force.0)) != 0 {
            raw.0 &= !eRestyle_Force.0;
            hint.insert(RestyleHint::RECASCADE_SELF);
        }

        if (raw.0 & eRestyle_ForceDescendants.0) != 0 {
            raw.0 &= !eRestyle_ForceDescendants.0;
            hint.insert(RestyleHint::RECASCADE_DESCENDANTS);
        }

        hint.insert(RestyleHint::from_bits_truncate(raw.0 as u8));

        hint
    }
}

#[cfg(feature = "servo")]
malloc_size_of_is_0!(RestyleHint);

/// Asserts that all replacement hints have a matching nsRestyleHint value.
#[cfg(feature = "gecko")]
#[inline]
pub fn assert_restyle_hints_match() {
    use gecko_bindings::structs;

    macro_rules! check_restyle_hints {
        ( $( $a:ident => $b:path),*, ) => {
            if cfg!(debug_assertions) {
                let mut replacements = RestyleHint::replacements();
                $(
                    assert_eq!(structs::$a.0 as usize, $b.bits() as usize, stringify!($b));
                    replacements.remove($b);
                )*
                assert_eq!(replacements, RestyleHint::empty(),
                           "all RestyleHint replacement bits should have an \
                            assertion");
            }
        }
    }

    check_restyle_hints! {
        nsRestyleHint_eRestyle_CSSTransitions => RestyleHint::RESTYLE_CSS_TRANSITIONS,
        nsRestyleHint_eRestyle_CSSAnimations => RestyleHint::RESTYLE_CSS_ANIMATIONS,
        nsRestyleHint_eRestyle_StyleAttribute => RestyleHint::RESTYLE_STYLE_ATTRIBUTE,
        nsRestyleHint_eRestyle_StyleAttribute_Animations => RestyleHint::RESTYLE_SMIL,
    }
}
