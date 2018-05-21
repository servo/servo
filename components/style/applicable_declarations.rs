/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

//! Applicable declarations management.

use properties::PropertyDeclarationBlock;
use rule_tree::{CascadeLevel, ShadowCascadeOrder, StyleSource};
use servo_arc::Arc;
use shared_lock::Locked;
use smallvec::SmallVec;
use std::fmt::{self, Debug};

/// List of applicable declarations. This is a transient structure that shuttles
/// declarations between selector matching and inserting into the rule tree, and
/// therefore we want to avoid heap-allocation where possible.
///
/// In measurements on wikipedia, we pretty much never have more than 8 applicable
/// declarations, so we could consider making this 8 entries instead of 16.
/// However, it may depend a lot on workload, and stack space is cheap.
pub type ApplicableDeclarationList = SmallVec<[ApplicableDeclarationBlock; 16]>;

/// Blink uses 18 bits to store source order, and does not check overflow [1].
/// That's a limit that could be reached in realistic webpages, so we use
/// 24 bits and enforce defined behavior in the overflow case.
///
/// [1] https://cs.chromium.org/chromium/src/third_party/WebKit/Source/core/css/
///     RuleSet.h?l=128&rcl=90140ab80b84d0f889abc253410f44ed54ae04f3
const SOURCE_ORDER_SHIFT: usize = 0;
const SOURCE_ORDER_BITS: usize = 24;
const SOURCE_ORDER_MAX: u32 = (1 << SOURCE_ORDER_BITS) - 1;
const SOURCE_ORDER_MASK: u32 = SOURCE_ORDER_MAX << SOURCE_ORDER_SHIFT;

/// We store up-to-15 shadow order levels.
///
/// You'd need an element slotted across 16 components with ::slotted rules to
/// trigger this as of this writing, which looks... Unlikely.
const SHADOW_CASCADE_ORDER_SHIFT: usize = SOURCE_ORDER_BITS;
const SHADOW_CASCADE_ORDER_BITS: usize = 4;
const SHADOW_CASCADE_ORDER_MAX: u8 = (1 << SHADOW_CASCADE_ORDER_BITS) - 1;
const SHADOW_CASCADE_ORDER_MASK: u32 = (SHADOW_CASCADE_ORDER_MAX as u32) << SHADOW_CASCADE_ORDER_SHIFT;

const CASCADE_LEVEL_SHIFT: usize = SOURCE_ORDER_BITS + SHADOW_CASCADE_ORDER_BITS;
const CASCADE_LEVEL_BITS: usize = 4;
const CASCADE_LEVEL_MAX: u8 = (1 << CASCADE_LEVEL_BITS) - 1;
const CASCADE_LEVEL_MASK: u32 = (CASCADE_LEVEL_MAX as u32) << CASCADE_LEVEL_SHIFT;

/// Stores the source order of a block, the cascade level it belongs to, and the
/// counter needed to handle Shadow DOM cascade order properly.
#[derive(Clone, Copy, Eq, MallocSizeOf, PartialEq)]
struct ApplicableDeclarationBits(u32);

impl ApplicableDeclarationBits {
    fn new(
        source_order: u32,
        cascade_level: CascadeLevel,
        shadow_cascade_order: ShadowCascadeOrder,
    ) -> Self {
        debug_assert!(
            cascade_level as u8 <= CASCADE_LEVEL_MAX,
            "Gotta find more bits!"
        );
        let mut bits = ::std::cmp::min(source_order, SOURCE_ORDER_MAX);
        bits |= ((shadow_cascade_order & SHADOW_CASCADE_ORDER_MAX) as u32) << SHADOW_CASCADE_ORDER_SHIFT;
        bits |= (cascade_level as u8 as u32) << CASCADE_LEVEL_SHIFT;
        ApplicableDeclarationBits(bits)
    }

    fn source_order(&self) -> u32 {
        (self.0 & SOURCE_ORDER_MASK) >> SOURCE_ORDER_SHIFT
    }

    fn shadow_cascade_order(&self) -> ShadowCascadeOrder {
        ((self.0 & SHADOW_CASCADE_ORDER_MASK) >> SHADOW_CASCADE_ORDER_SHIFT) as ShadowCascadeOrder
    }

    fn level(&self) -> CascadeLevel {
        let byte = ((self.0 & CASCADE_LEVEL_MASK) >> CASCADE_LEVEL_SHIFT) as u8;
        unsafe { CascadeLevel::from_byte(byte) }
    }
}

impl Debug for ApplicableDeclarationBits {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_struct("ApplicableDeclarationBits")
            .field("source_order", &self.source_order())
            .field("shadow_cascade_order", &self.shadow_cascade_order())
            .field("level", &self.level())
            .finish()
    }
}

/// A property declaration together with its precedence among rules of equal
/// specificity so that we can sort them.
///
/// This represents the declarations in a given declaration block for a given
/// importance.
#[derive(Clone, Debug, MallocSizeOf, PartialEq)]
pub struct ApplicableDeclarationBlock {
    /// The style source, either a style rule, or a property declaration block.
    #[ignore_malloc_size_of = "Arc"]
    pub source: StyleSource,
    /// The bits containing the source order, cascade level, and shadow cascade
    /// order.
    bits: ApplicableDeclarationBits,
    /// The specificity of the selector this block is represented by.
    pub specificity: u32,
}

impl ApplicableDeclarationBlock {
    /// Constructs an applicable declaration block from a given property
    /// declaration block and importance.
    #[inline]
    pub fn from_declarations(
        declarations: Arc<Locked<PropertyDeclarationBlock>>,
        level: CascadeLevel,
    ) -> Self {
        ApplicableDeclarationBlock {
            source: StyleSource::from_declarations(declarations),
            bits: ApplicableDeclarationBits::new(0, level, 0),
            specificity: 0,
        }
    }

    /// Constructs an applicable declaration block from the given components
    #[inline]
    pub fn new(
        source: StyleSource,
        order: u32,
        level: CascadeLevel,
        specificity: u32,
        shadow_cascade_order: ShadowCascadeOrder,
    ) -> Self {
        ApplicableDeclarationBlock {
            source,
            bits: ApplicableDeclarationBits::new(order, level, shadow_cascade_order),
            specificity,
        }
    }

    /// Returns the source order of the block.
    #[inline]
    pub fn source_order(&self) -> u32 {
        self.bits.source_order()
    }

    /// Returns the cascade level of the block.
    #[inline]
    pub fn level(&self) -> CascadeLevel {
        self.bits.level()
    }

    /// Convenience method to consume self and return the right thing for the
    /// rule tree to iterate over.
    #[inline]
    pub fn for_rule_tree(self) -> (StyleSource, CascadeLevel, ShadowCascadeOrder) {
        let level = self.level();
        let cascade_order = self.bits.shadow_cascade_order();
        (self.source, level, cascade_order)
    }
}
