/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

//! Restyle hints: an optimization to avoid unnecessarily matching selectors.

#[cfg(feature = "gecko")]
use gecko_bindings::structs::nsRestyleHint;

bitflags! {
    /// The kind of restyle we need to do for a given element.
    pub flags RestyleHint: u8 {
        /// Do a selector match of the element.
        const RESTYLE_SELF = 1 << 0,

        /// Do a selector match of the element's descendants.
        const RESTYLE_DESCENDANTS = 1 << 1,

        /// Recascade the current element.
        const RECASCADE_SELF = 1 << 2,

        /// Recascade all descendant elements.
        const RECASCADE_DESCENDANTS = 1 << 3,

        /// Replace the style data coming from CSS transitions without updating
        /// any other style data. This hint is only processed in animation-only
        /// traversal which is prior to normal traversal.
        const RESTYLE_CSS_TRANSITIONS = 1 << 4,

        /// Replace the style data coming from CSS animations without updating
        /// any other style data. This hint is only processed in animation-only
        /// traversal which is prior to normal traversal.
        const RESTYLE_CSS_ANIMATIONS = 1 << 5,

        /// Don't re-run selector-matching on the element, only the style
        /// attribute has changed, and this change didn't have any other
        /// dependencies.
        const RESTYLE_STYLE_ATTRIBUTE = 1 << 6,

        /// Replace the style data coming from SMIL animations without updating
        /// any other style data. This hint is only processed in animation-only
        /// traversal which is prior to normal traversal.
        const RESTYLE_SMIL = 1 << 7,
    }
}

impl RestyleHint {
    /// Creates a new `RestyleHint` indicating that the current element and all
    /// its descendants must be fully restyled.
    pub fn restyle_subtree() -> Self {
        RESTYLE_SELF | RESTYLE_DESCENDANTS
    }

    /// Creates a new `RestyleHint` indicating that the current element and all
    /// its descendants must be recascaded.
    pub fn recascade_subtree() -> Self {
        RECASCADE_SELF | RECASCADE_DESCENDANTS
    }

    /// Returns a new `CascadeHint` appropriate for children of the current
    /// element.
    pub fn propagate_for_non_animation_restyle(&self) -> Self {
        if self.contains(RESTYLE_DESCENDANTS) {
            return Self::restyle_subtree()
        }
        if self.contains(RECASCADE_DESCENDANTS) {
            return Self::recascade_subtree()
        }
        Self::empty()
    }

    /// Creates a new `RestyleHint` that indicates the element must be
    /// recascaded.
    pub fn recascade_self() -> Self {
        RECASCADE_SELF
    }

    /// Returns a hint that contains all the replacement hints.
    pub fn replacements() -> Self {
        RESTYLE_STYLE_ATTRIBUTE | Self::for_animations()
    }

    /// The replacements for the animation cascade levels.
    #[inline]
    pub fn for_animations() -> Self {
        RESTYLE_SMIL | RESTYLE_CSS_ANIMATIONS | RESTYLE_CSS_TRANSITIONS
    }

    /// Returns whether the hint specifies that some work must be performed on
    /// the current element.
    #[inline]
    pub fn affects_self(&self) -> bool {
        self.intersects(RESTYLE_SELF | RECASCADE_SELF | Self::replacements())
    }

    /// Returns whether the hint specifies that the currently element must be
    /// recascaded.
    pub fn has_recascade_self(&self) -> bool {
        self.contains(RECASCADE_SELF)
    }

    /// Returns whether the hint specifies that an animation cascade level must
    /// be replaced.
    #[inline]
    pub fn has_animation_hint(&self) -> bool {
        self.intersects(Self::for_animations())
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
        self.intersects(RESTYLE_SELF)
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
        self.remove(RECASCADE_SELF);
    }
}

#[cfg(feature = "gecko")]
impl From<nsRestyleHint> for RestyleHint {
    fn from(raw: nsRestyleHint) -> Self {
        use gecko_bindings::structs::nsRestyleHint_eRestyle_ForceDescendants as eRestyle_ForceDescendants;
        use gecko_bindings::structs::nsRestyleHint_eRestyle_LaterSiblings as eRestyle_LaterSiblings;
        use gecko_bindings::structs::nsRestyleHint_eRestyle_Self as eRestyle_Self;
        use gecko_bindings::structs::nsRestyleHint_eRestyle_SomeDescendants as eRestyle_SomeDescendants;
        use gecko_bindings::structs::nsRestyleHint_eRestyle_Subtree as eRestyle_Subtree;

        let mut hint = RestyleHint::empty();

        debug_assert!(raw.0 & eRestyle_LaterSiblings.0 == 0,
                      "Handle later siblings manually if necessary plz.");

        if (raw.0 & (eRestyle_Self.0 | eRestyle_Subtree.0)) != 0 {
            hint.insert(RESTYLE_SELF);
        }

        if (raw.0 & (eRestyle_Subtree.0 | eRestyle_SomeDescendants.0)) != 0 {
            hint.insert(RESTYLE_DESCENDANTS);
        }

        if (raw.0 & eRestyle_ForceDescendants.0) != 0 {
            hint.insert(RECASCADE_DESCENDANTS);
        }

        hint.insert(RestyleHint::from_bits_truncate(raw.0 as u8));

        hint
    }
}

#[cfg(feature = "servo")]
impl ::heapsize::HeapSizeOf for RestyleHint {
    fn heap_size_of_children(&self) -> usize { 0 }
}

/// Asserts that all replacement hints have a matching nsRestyleHint value.
#[cfg(feature = "gecko")]
#[inline]
pub fn assert_restyle_hints_match() {
    use gecko_bindings::structs;

    macro_rules! check_restyle_hints {
        ( $( $a:ident => $b:ident ),*, ) => {
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
        nsRestyleHint_eRestyle_CSSTransitions => RESTYLE_CSS_TRANSITIONS,
        nsRestyleHint_eRestyle_CSSAnimations => RESTYLE_CSS_ANIMATIONS,
        nsRestyleHint_eRestyle_StyleAttribute => RESTYLE_STYLE_ATTRIBUTE,
        nsRestyleHint_eRestyle_StyleAttribute_Animations => RESTYLE_SMIL,
    }
}
