/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

//! Applicable declarations management.

use crate::properties::PropertyDeclarationBlock;
use crate::rule_tree::{CascadeLevel, StyleSource};
use crate::shared_lock::Locked;
use servo_arc::Arc;
use smallvec::SmallVec;

/// List of applicable declarations. This is a transient structure that shuttles
/// declarations between selector matching and inserting into the rule tree, and
/// therefore we want to avoid heap-allocation where possible.
///
/// In measurements on wikipedia, we pretty much never have more than 8 applicable
/// declarations, so we could consider making this 8 entries instead of 16.
/// However, it may depend a lot on workload, and stack space is cheap.
pub type ApplicableDeclarationList = SmallVec<[ApplicableDeclarationBlock; 16]>;

/// Stores the source order of a block, the cascade level it belongs to, and the
/// counter needed to handle Shadow DOM cascade order properly.
///
/// FIXME(emilio): Optimize storage.
#[derive(Clone, Copy, Eq, MallocSizeOf, PartialEq, Debug)]
struct ApplicableDeclarationBits {
    source_order: u32,
    cascade_level: CascadeLevel,
}

impl ApplicableDeclarationBits {
    fn new(source_order: u32, cascade_level: CascadeLevel) -> Self {
        Self { source_order, cascade_level }
    }

    fn source_order(&self) -> u32 {
        self.source_order
    }

    fn level(&self) -> CascadeLevel {
        self.cascade_level
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
            bits: ApplicableDeclarationBits::new(0, level),
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
    ) -> Self {
        ApplicableDeclarationBlock {
            source,
            bits: ApplicableDeclarationBits::new(order, level),
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
    pub fn for_rule_tree(self) -> (StyleSource, CascadeLevel) {
        let level = self.level();
        (self.source, level)
    }
}
