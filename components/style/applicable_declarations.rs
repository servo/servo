/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

//! Applicable declarations management.

use crate::properties::PropertyDeclarationBlock;
use crate::rule_tree::{CascadeLevel, StyleSource};
use crate::shared_lock::Locked;
use crate::stylesheets::layer_rule::LayerOrder;
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

/// Blink uses 18 bits to store source order, and does not check overflow [1].
/// That's a limit that could be reached in realistic webpages, so we use
/// 24 bits and enforce defined behavior in the overflow case.
///
/// Note that right now this restriction could be lifted if wanted (because we
/// no longer stash the cascade level in the remaining bits), but we keep it in
/// place in case we come up with a use-case for them, lacking reports of the
/// current limit being too small.
///
/// [1] https://cs.chromium.org/chromium/src/third_party/WebKit/Source/core/css/
///     RuleSet.h?l=128&rcl=90140ab80b84d0f889abc253410f44ed54ae04f3
const SOURCE_ORDER_BITS: usize = 24;
const SOURCE_ORDER_MAX: u32 = (1 << SOURCE_ORDER_BITS) - 1;
const SOURCE_ORDER_MASK: u32 = SOURCE_ORDER_MAX;

/// The cascade-level+layer order of this declaration.
#[derive(Clone, Copy, Debug, Eq, Hash, MallocSizeOf, PartialEq)]
pub struct CascadePriority {
    cascade_level: CascadeLevel,
    layer_order: LayerOrder,
}

const_assert_eq!(
    std::mem::size_of::<CascadePriority>(),
    std::mem::size_of::<u32>()
);

impl PartialOrd for CascadePriority {
    #[inline]
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for CascadePriority {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.cascade_level.cmp(&other.cascade_level).then_with(|| {
            let ordering = self.layer_order.cmp(&other.layer_order);
            if ordering == std::cmp::Ordering::Equal {
                return ordering;
            }
            // https://drafts.csswg.org/css-cascade-5/#cascade-layering
            //
            //     Cascade layers (like declarations) are ordered by order
            //     of appearance. When comparing declarations that belong to
            //     different layers, then for normal rules the declaration
            //     whose cascade layer is last wins, and for important rules
            //     the declaration whose cascade layer is first wins.
            //
            // But the style attribute layer for some reason is special.
            if self.cascade_level.is_important() &&
                !self.layer_order.is_style_attribute_layer() &&
                !other.layer_order.is_style_attribute_layer()
            {
                ordering.reverse()
            } else {
                ordering
            }
        })
    }
}

impl CascadePriority {
    /// Construct a new CascadePriority for a given (level, order) pair.
    pub fn new(cascade_level: CascadeLevel, layer_order: LayerOrder) -> Self {
        Self {
            cascade_level,
            layer_order,
        }
    }

    /// Returns the layer order.
    #[inline]
    pub fn layer_order(&self) -> LayerOrder {
        self.layer_order
    }

    /// Returns the cascade level.
    #[inline]
    pub fn cascade_level(&self) -> CascadeLevel {
        self.cascade_level
    }

    /// Whether this declaration should be allowed if `revert` or `revert-layer`
    /// have been specified on a given origin.
    ///
    /// `self` is the priority at which the `revert` or `revert-layer` keyword
    /// have been specified.
    pub fn allows_when_reverted(&self, other: &Self, origin_revert: bool) -> bool {
        if origin_revert {
            other.cascade_level.origin() < self.cascade_level.origin()
        } else {
            other.unimportant() < self.unimportant()
        }
    }

    /// Convert this priority from "important" to "non-important", if needed.
    pub fn unimportant(&self) -> Self {
        Self::new(self.cascade_level().unimportant(), self.layer_order())
    }

    /// Convert this priority from "non-important" to "important", if needed.
    pub fn important(&self) -> Self {
        Self::new(self.cascade_level().important(), self.layer_order())
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
    source_order: u32,
    /// The specificity of the selector.
    pub specificity: u32,
    /// The cascade priority of the rule.
    pub cascade_priority: CascadePriority,
}

impl ApplicableDeclarationBlock {
    /// Constructs an applicable declaration block from a given property
    /// declaration block and importance.
    #[inline]
    pub fn from_declarations(
        declarations: Arc<Locked<PropertyDeclarationBlock>>,
        level: CascadeLevel,
        layer_order: LayerOrder,
    ) -> Self {
        ApplicableDeclarationBlock {
            source: StyleSource::from_declarations(declarations),
            source_order: 0,
            specificity: 0,
            cascade_priority: CascadePriority::new(level, layer_order),
        }
    }

    /// Constructs an applicable declaration block from the given components.
    #[inline]
    pub fn new(
        source: StyleSource,
        source_order: u32,
        level: CascadeLevel,
        specificity: u32,
        layer_order: LayerOrder,
    ) -> Self {
        ApplicableDeclarationBlock {
            source,
            source_order: source_order & SOURCE_ORDER_MASK,
            specificity,
            cascade_priority: CascadePriority::new(level, layer_order),
        }
    }

    /// Returns the source order of the block.
    #[inline]
    pub fn source_order(&self) -> u32 {
        self.source_order
    }

    /// Returns the cascade level of the block.
    #[inline]
    pub fn level(&self) -> CascadeLevel {
        self.cascade_priority.cascade_level()
    }

    /// Returns the cascade level of the block.
    #[inline]
    pub fn layer_order(&self) -> LayerOrder {
        self.cascade_priority.layer_order()
    }

    /// Convenience method to consume self and return the right thing for the
    /// rule tree to iterate over.
    #[inline]
    pub fn for_rule_tree(self) -> (StyleSource, CascadePriority) {
        (self.source, self.cascade_priority)
    }
}

// Size of this struct determines sorting and selector-matching performance.
size_of_test!(ApplicableDeclarationBlock, 24);
