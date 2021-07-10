/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

//! Flags that control the traversal process.
//!
//! We CamelCase rather than UPPER_CASING so that we can grep for the same
//! strings across gecko and servo.
#![allow(non_upper_case_globals)]

bitflags! {
    /// Flags that control the traversal process.
    pub struct TraversalFlags: u32 {
        /// Traverse only elements for animation restyles.
        const AnimationOnly = 1 << 0;
        /// Traverse and update all elements with CSS animations since
        /// @keyframes rules may have changed. Triggered by CSS rule changes.
        const ForCSSRuleChanges = 1 << 1;
        /// The final animation-only traversal, which shouldn't really care about other
        /// style changes anymore.
        const FinalAnimationTraversal = 1 << 2;
        /// Allows the traversal to run in parallel if there are sufficient cores on
        /// the machine.
        const ParallelTraversal = 1 << 7;
        /// Flush throttled animations. By default, we only update throttled animations
        /// when we have other non-throttled work to do. With this flag, we
        /// unconditionally tick and process them.
        const FlushThrottledAnimations = 1 << 8;

    }
}

/// Asserts that all TraversalFlags flags have a matching ServoTraversalFlags value in gecko.
#[cfg(feature = "gecko")]
#[inline]
pub fn assert_traversal_flags_match() {
    use crate::gecko_bindings::structs;

    macro_rules! check_traversal_flags {
        ( $( $a:ident => $b:path ),*, ) => {
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
        ServoTraversalFlags_AnimationOnly => TraversalFlags::AnimationOnly,
        ServoTraversalFlags_ForCSSRuleChanges => TraversalFlags::ForCSSRuleChanges,
        ServoTraversalFlags_FinalAnimationTraversal => TraversalFlags::FinalAnimationTraversal,
        ServoTraversalFlags_ParallelTraversal => TraversalFlags::ParallelTraversal,
        ServoTraversalFlags_FlushThrottledAnimations => TraversalFlags::FlushThrottledAnimations,
    }
}

impl TraversalFlags {
    /// Returns true if the traversal is for animation-only restyles.
    #[inline]
    pub fn for_animation_only(&self) -> bool {
        self.contains(TraversalFlags::AnimationOnly)
    }
}
