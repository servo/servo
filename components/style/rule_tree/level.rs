/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

#![forbid(unsafe_code)]

use crate::properties::Importance;
use crate::shared_lock::{SharedRwLockReadGuard, StylesheetGuards};
use crate::stylesheets::Origin;

/// The cascade level these rules are relevant at, as per[1][2][3].
///
/// Presentational hints for SVG and HTML are in the "author-level
/// zero-specificity" level, that is, right after user rules, and before author
/// rules.
///
/// The order of variants declared here is significant, and must be in
/// _ascending_ order of precedence.
///
/// See also [4] for the Shadow DOM bits. We rely on the invariant that rules
/// from outside the tree the element is in can't affect the element.
///
/// The opposite is not true (i.e., :host and ::slotted) from an "inner" shadow
/// tree may affect an element connected to the document or an "outer" shadow
/// tree.
///
/// [1]: https://drafts.csswg.org/css-cascade/#cascade-origin
/// [2]: https://drafts.csswg.org/css-cascade/#preshint
/// [3]: https://html.spec.whatwg.org/multipage/#presentational-hints
/// [4]: https://drafts.csswg.org/css-scoping/#shadow-cascading
#[repr(u8)]
#[derive(Clone, Copy, Debug, Eq, Hash, MallocSizeOf, Ord, PartialEq, PartialOrd)]
pub enum CascadeLevel {
    /// Normal User-Agent rules.
    UANormal,
    /// User normal rules.
    UserNormal,
    /// Presentational hints.
    PresHints,
    /// Shadow DOM styles from author styles.
    AuthorNormal {
        /// The order in the shadow tree hierarchy. This number is relative to
        /// the tree of the element, and thus the only invariants that need to
        /// be preserved is:
        ///
        ///  * Zero is the same tree as the element that matched the rule. This
        ///    is important so that we can optimize style attribute insertions.
        ///
        ///  * The levels are ordered in accordance with
        ///    https://drafts.csswg.org/css-scoping/#shadow-cascading
        shadow_cascade_order: ShadowCascadeOrder,
    },
    /// SVG SMIL animations.
    SMILOverride,
    /// CSS animations and script-generated animations.
    Animations,
    /// Author-supplied important rules.
    AuthorImportant {
        /// The order in the shadow tree hierarchy, inverted, so that PartialOrd
        /// does the right thing.
        shadow_cascade_order: ShadowCascadeOrder,
    },
    /// User important rules.
    UserImportant,
    /// User-agent important rules.
    UAImportant,
    /// Transitions
    Transitions,
}

impl CascadeLevel {
    /// Convert this level from "unimportant" to "important".
    pub fn important(&self) -> Self {
        match *self {
            Self::UANormal => Self::UAImportant,
            Self::UserNormal => Self::UserImportant,
            Self::AuthorNormal {
                shadow_cascade_order,
            } => Self::AuthorImportant {
                shadow_cascade_order: -shadow_cascade_order,
            },
            Self::PresHints |
            Self::SMILOverride |
            Self::Animations |
            Self::AuthorImportant { .. } |
            Self::UserImportant |
            Self::UAImportant |
            Self::Transitions => *self,
        }
    }

    /// Convert this level from "important" to "non-important".
    pub fn unimportant(&self) -> Self {
        match *self {
            Self::UAImportant => Self::UANormal,
            Self::UserImportant => Self::UserNormal,
            Self::AuthorImportant {
                shadow_cascade_order,
            } => Self::AuthorNormal {
                shadow_cascade_order: -shadow_cascade_order,
            },
            Self::PresHints |
            Self::SMILOverride |
            Self::Animations |
            Self::AuthorNormal { .. } |
            Self::UserNormal |
            Self::UANormal |
            Self::Transitions => *self,
        }
    }

    /// Select a lock guard for this level
    pub fn guard<'a>(&self, guards: &'a StylesheetGuards<'a>) -> &'a SharedRwLockReadGuard<'a> {
        match *self {
            Self::UANormal | Self::UserNormal | Self::UserImportant | Self::UAImportant => {
                guards.ua_or_user
            },
            _ => guards.author,
        }
    }

    /// Returns the cascade level for author important declarations from the
    /// same tree as the element.
    #[inline]
    pub fn same_tree_author_important() -> Self {
        Self::AuthorImportant {
            shadow_cascade_order: ShadowCascadeOrder::for_same_tree(),
        }
    }

    /// Returns the cascade level for author normal declarations from the same
    /// tree as the element.
    #[inline]
    pub fn same_tree_author_normal() -> Self {
        Self::AuthorNormal {
            shadow_cascade_order: ShadowCascadeOrder::for_same_tree(),
        }
    }

    /// Returns whether this cascade level represents important rules of some
    /// sort.
    #[inline]
    pub fn is_important(&self) -> bool {
        match *self {
            Self::AuthorImportant { .. } | Self::UserImportant | Self::UAImportant => true,
            _ => false,
        }
    }

    /// Returns the importance relevant for this rule. Pretty similar to
    /// `is_important`.
    #[inline]
    pub fn importance(&self) -> Importance {
        if self.is_important() {
            Importance::Important
        } else {
            Importance::Normal
        }
    }

    /// Returns the cascade origin of the rule.
    #[inline]
    pub fn origin(&self) -> Origin {
        match *self {
            Self::UAImportant | Self::UANormal => Origin::UserAgent,
            Self::UserImportant | Self::UserNormal => Origin::User,
            Self::PresHints |
            Self::AuthorNormal { .. } |
            Self::AuthorImportant { .. } |
            Self::SMILOverride |
            Self::Animations |
            Self::Transitions => Origin::Author,
        }
    }

    /// Returns whether this cascade level represents an animation rules.
    #[inline]
    pub fn is_animation(&self) -> bool {
        match *self {
            Self::SMILOverride | Self::Animations | Self::Transitions => true,
            _ => false,
        }
    }
}

/// A counter to track how many shadow root rules deep we are. This is used to
/// handle:
///
/// https://drafts.csswg.org/css-scoping/#shadow-cascading
///
/// See the static functions for the meaning of different values.
#[derive(Clone, Copy, Debug, Eq, Hash, MallocSizeOf, Ord, PartialEq, PartialOrd)]
pub struct ShadowCascadeOrder(i8);

impl ShadowCascadeOrder {
    /// We keep a maximum of 3 bits of order as a limit so that we can pack
    /// CascadeLevel in one byte by using half of it for the order, if that ends
    /// up being necessary.
    const MAX: i8 = 0b111;
    const MIN: i8 = -Self::MAX;

    /// A level for the outermost shadow tree (the shadow tree we own, and the
    /// ones from the slots we're slotted in).
    #[inline]
    pub fn for_outermost_shadow_tree() -> Self {
        Self(-1)
    }

    /// A level for the element's tree.
    #[inline]
    fn for_same_tree() -> Self {
        Self(0)
    }

    /// A level for the innermost containing tree (the one closest to the
    /// element).
    #[inline]
    pub fn for_innermost_containing_tree() -> Self {
        Self(1)
    }

    /// Decrement the level, moving inwards. We should only move inwards if
    /// we're traversing slots.
    #[inline]
    pub fn dec(&mut self) {
        debug_assert!(self.0 < 0);
        if self.0 != Self::MIN {
            self.0 -= 1;
        }
    }

    /// The level, moving inwards. We should only move inwards if we're
    /// traversing slots.
    #[inline]
    pub fn inc(&mut self) {
        debug_assert_ne!(self.0, -1);
        if self.0 != Self::MAX {
            self.0 += 1;
        }
    }
}

impl std::ops::Neg for ShadowCascadeOrder {
    type Output = Self;
    #[inline]
    fn neg(self) -> Self {
        Self(self.0.neg())
    }
}
