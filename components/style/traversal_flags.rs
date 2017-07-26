/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

//! Flags that control the traversal process.
//!
//! We CamelCase rather than UPPER_CASING so that we can grep for the same
//! strings across gecko and servo.
#![allow(non_upper_case_globals)]

bitflags! {
    /// Flags that control the traversal process.
    pub flags TraversalFlags: u32 {
        /// Traverse only elements for animation restyles.
        const AnimationOnly = 1 << 0,
        /// Traverse and update all elements with CSS animations since
        /// @keyframes rules may have changed. Triggered by CSS rule changes.
        const ForCSSRuleChanges = 1 << 1,
        /// Traverse only unstyled children of the root and their descendants.
        const UnstyledChildrenOnly = 1 << 2,
        /// FIXME(bholley): This will go away.
        const ForReconstruct = 1 << 3,
        /// FIXME(bholley): This will go away.
        const ForNewlyBoundElement = 1 << 4,
    }
}

/// Asserts that all TraversalFlags flags have a matching ServoTraversalFlags value in gecko.
#[cfg(feature = "gecko")]
#[inline]
pub fn assert_traversal_flags_match() {
    use gecko_bindings::structs;

    macro_rules! check_traversal_flags {
        ( $( $a:ident => $b:ident ),*, ) => {
            if cfg!(debug_assertions) {
                let mut modes = TraversalFlags::all();
                $(
                    assert_eq!(structs::$a as usize, $b.bits() as usize, stringify!($b));
                    modes.remove($b);
                )*
                assert_eq!(modes, TraversalFlags::empty(), "all TraversalFlags bits should have an assertion");
            }
        }
    }

    check_traversal_flags! {
        ServoTraversalFlags_AnimationOnly => AnimationOnly,
        ServoTraversalFlags_ForCSSRuleChanges => ForCSSRuleChanges,
        ServoTraversalFlags_UnstyledChildrenOnly => UnstyledChildrenOnly,
        ServoTraversalFlags_ForReconstruct => ForReconstruct,
        ServoTraversalFlags_ForNewlyBoundElement => ForNewlyBoundElement,
    }
}

impl TraversalFlags {
    /// Returns true if the traversal is for animation-only restyles.
    pub fn for_animation_only(&self) -> bool {
        self.contains(AnimationOnly)
    }
}
