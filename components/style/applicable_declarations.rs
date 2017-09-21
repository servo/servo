/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

//! Applicable declarations management.

use properties::PropertyDeclarationBlock;
use rule_tree::{CascadeLevel, StyleSource};
use servo_arc::Arc;
use shared_lock::Locked;
use smallvec::SmallVec;
use std::fmt::{Debug, self};
use std::mem;

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
/// Note that the value of 24 is also hard-coded into the level() accessor,
/// which does a byte-aligned load of the 4th byte. If you change this value
/// you'll need to change that as well.
///
/// [1] https://cs.chromium.org/chromium/src/third_party/WebKit/Source/core/css/
///     RuleSet.h?l=128&rcl=90140ab80b84d0f889abc253410f44ed54ae04f3
const SOURCE_ORDER_BITS: usize = 24;
const SOURCE_ORDER_MASK: u32 = (1 << SOURCE_ORDER_BITS) - 1;
const SOURCE_ORDER_MAX: u32 = SOURCE_ORDER_MASK;

/// Stores the source order of a block and the cascade level it belongs to.
#[cfg_attr(feature = "gecko", derive(MallocSizeOf))]
#[cfg_attr(feature = "servo", derive(HeapSizeOf))]
#[derive(Clone, Copy, Eq, PartialEq)]
struct SourceOrderAndCascadeLevel(u32);

impl SourceOrderAndCascadeLevel {
    fn new(source_order: u32, cascade_level: CascadeLevel) -> SourceOrderAndCascadeLevel {
        let mut bits = ::std::cmp::min(source_order, SOURCE_ORDER_MAX);
        bits |= (cascade_level as u8 as u32) << SOURCE_ORDER_BITS;
        SourceOrderAndCascadeLevel(bits)
    }

    fn order(&self) -> u32 {
        self.0 & SOURCE_ORDER_MASK
    }

    fn level(&self) -> CascadeLevel {
        unsafe {
            // Transmute rather than shifting so that we're sure the compiler
            // emits a simple byte-aligned load.
            let as_bytes: [u8; 4] = mem::transmute(self.0);
            CascadeLevel::from_byte(as_bytes[3])
        }
    }
}

impl Debug for SourceOrderAndCascadeLevel {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_struct("SourceOrderAndCascadeLevel")
            .field("order", &self.order())
            .field("level", &self.level())
            .finish()
    }
}

/// A property declaration together with its precedence among rules of equal
/// specificity so that we can sort them.
///
/// This represents the declarations in a given declaration block for a given
/// importance.
#[cfg_attr(feature = "gecko", derive(MallocSizeOf))]
#[cfg_attr(feature = "servo", derive(HeapSizeOf))]
#[derive(Clone, Debug, PartialEq)]
pub struct ApplicableDeclarationBlock {
    /// The style source, either a style rule, or a property declaration block.
    #[cfg_attr(feature = "gecko", ignore_malloc_size_of = "contains Arcs")]
    #[cfg_attr(feature = "servo", ignore_heap_size_of = "Arc")]
    pub source: StyleSource,
    /// The source order of the block, and the cascade level it belongs to.
    order_and_level: SourceOrderAndCascadeLevel,
    /// The specificity of the selector this block is represented by.
    pub specificity: u32,
}

impl ApplicableDeclarationBlock {
    /// Constructs an applicable declaration block from a given property
    /// declaration block and importance.
    #[inline]
    pub fn from_declarations(declarations: Arc<Locked<PropertyDeclarationBlock>>,
                             level: CascadeLevel)
                             -> Self {
        ApplicableDeclarationBlock {
            source: StyleSource::Declarations(declarations),
            order_and_level: SourceOrderAndCascadeLevel::new(0, level),
            specificity: 0,
        }
    }

    /// Constructs an applicable declaration block from the given components
    #[inline]
    pub fn new(source: StyleSource,
               order: u32,
               level: CascadeLevel,
               specificity: u32) -> Self {
        ApplicableDeclarationBlock {
            source: source,
            order_and_level: SourceOrderAndCascadeLevel::new(order, level),
            specificity: specificity,
        }

    }

    /// Returns the source order of the block.
    #[inline]
    pub fn source_order(&self) -> u32 {
        self.order_and_level.order()
    }

    /// Returns the cascade level of the block.
    #[inline]
    pub fn level(&self) -> CascadeLevel {
        self.order_and_level.level()
    }

    /// Convenience method to consume self and return the source alongside the
    /// level.
    #[inline]
    pub fn order_and_level(self) -> (StyleSource, CascadeLevel) {
        let level = self.level();
        (self.source, level)
    }
}
