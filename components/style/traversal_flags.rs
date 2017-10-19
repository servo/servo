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
        /// Styles unstyled elements, but does not handle invalidations on
        /// already-styled elements.
        const UnstyledOnly = 1 << 2,
        /// A forgetful traversal ignores the previous state of the frame tree, and
        /// thus does not compute damage or maintain other state describing the styles
        /// pre-traversal. A forgetful traversal is usually the right thing if you
        /// aren't going to do a post-traversal.
        const Forgetful = 1 << 3,
        /// Clears all the dirty bits on the elements traversed.
        const ClearDirtyBits = 1 << 5,
        /// Clears the animation-only dirty descendants bit in the subtree.
        const ClearAnimationOnlyDirtyDescendants = 1 << 6,
        /// Allows the traversal to run in parallel if there are sufficient cores on
        /// the machine.
        const ParallelTraversal = 1 << 7,
        /// Flush throttled animations. By default, we only update throttled animations
        /// when we have other non-throttled work to do. With this flag, we
        /// unconditionally tick and process them.
        const FlushThrottledAnimations = 1 << 8,

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
        ServoTraversalFlags_UnstyledOnly => UnstyledOnly,
        ServoTraversalFlags_Forgetful => Forgetful,
        ServoTraversalFlags_ClearDirtyBits => ClearDirtyBits,
        ServoTraversalFlags_ClearAnimationOnlyDirtyDescendants =>
            ClearAnimationOnlyDirtyDescendants,
        ServoTraversalFlags_ParallelTraversal => ParallelTraversal,
        ServoTraversalFlags_FlushThrottledAnimations => FlushThrottledAnimations,
    }
}

impl TraversalFlags {
    /// Returns true if the traversal is for animation-only restyles.
    #[inline]
    pub fn for_animation_only(&self) -> bool {
        self.contains(AnimationOnly)
    }
}
