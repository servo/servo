/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use app_units::Au;
use malloc_size_of_derive::MallocSizeOf;
use num_traits::Zero;
use servo_arc::Arc as ServoArc;
use style::properties::ComputedValues;

use crate::geom::LogicalVec2;
use crate::sizing::SizeConstraint;
use crate::style_ext::AspectRatio;

pub(crate) struct GenericContainingBlock<Size: ContainingBlockSizeTrait, Style> {
    pub size: Size,
    pub style: Style,
}

impl<Size: ContainingBlockSizeTrait, Style> GenericContainingBlock<Size, Style> {
    #[inline]
    pub fn inline_stretch_size(&self, inline_pbm_sum: Au) -> Option<Au> {
        self.size.inline_stretch_size(inline_pbm_sum)
    }

    #[inline]
    pub fn block_stretch_size(&self, block_pbm_sum: Au) -> Option<Au> {
        self.size.block_stretch_size(block_pbm_sum)
    }
}

pub(crate) trait ContainingBlockSizeTrait {
    fn inline_size(&self) -> Option<Au>;

    fn block_size(&self) -> Option<Au>;

    #[inline]
    fn inline_stretch_size(&self, inline_pbm_sum: Au) -> Option<Au> {
        self.inline_size()
            .map(|size| Au::zero().max(size - inline_pbm_sum))
    }

    #[inline]
    fn block_stretch_size(&self, block_pbm_sum: Au) -> Option<Au> {
        self.block_size()
            .map(|size| Au::zero().max(size - block_pbm_sum))
    }
}

/// The usual containing block. Its inline size is known, but the block size
/// may be an indefinite constraint.
pub(crate) type ContainingBlock<'a> =
    GenericContainingBlock<ContainingBlockSize, &'a ComputedValues>;

#[derive(Clone, Debug, MallocSizeOf, PartialEq)]
pub(crate) struct ContainingBlockSize {
    pub inline: Au,
    pub block: SizeConstraint,
}

impl ContainingBlockSizeTrait for ContainingBlockSize {
    #[inline]
    fn inline_size(&self) -> Option<Au> {
        Some(self.inline)
    }

    #[inline]
    fn block_size(&self) -> Option<Au> {
        self.block.to_definite()
    }
}

/// A variant of [`ContainingBlock`] that allows an indefinite inline size.
/// Useful for code that is shared for both layout (where we know the inline size
/// of the containing block) and intrinsic sizing (where we don't know it).
pub(crate) type IndefiniteContainingBlock<'a> =
    GenericContainingBlock<LogicalVec2<Option<Au>>, &'a ComputedValues>;

impl ContainingBlockSizeTrait for LogicalVec2<Option<Au>> {
    #[inline]
    fn inline_size(&self) -> Option<Au> {
        self.inline
    }

    #[inline]
    fn block_size(&self) -> Option<Au> {
        self.block
    }
}

/// A variant of [`ContainingBlock`] that has a known block size.
/// Useful for laying out absolutely positioned elements.
pub(crate) type DefiniteContainingBlock<'a> =
    GenericContainingBlock<LogicalVec2<Au>, &'a ServoArc<ComputedValues>>;

impl ContainingBlockSizeTrait for LogicalVec2<Au> {
    #[inline]
    fn inline_size(&self) -> Option<Au> {
        Some(self.block)
    }

    #[inline]
    fn block_size(&self) -> Option<Au> {
        Some(self.block)
    }
}

/// Represents the set of constraints that we use when computing the min-content
/// and max-content inline sizes of an element.
pub(crate) struct ConstraintSpace<'a> {
    pub block_size: SizeConstraint,
    pub style: &'a ComputedValues,
    pub preferred_aspect_ratio: Option<AspectRatio>,
}

impl<'a> ConstraintSpace<'a> {
    #[inline]
    pub fn new(
        block_size: SizeConstraint,
        style: &'a ComputedValues,
        preferred_aspect_ratio: Option<AspectRatio>,
    ) -> Self {
        Self {
            block_size,
            style,
            preferred_aspect_ratio,
        }
    }
}

/* Size conversions */

impl From<&LogicalVec2<Au>> for ContainingBlockSize {
    #[inline]
    fn from(size: &LogicalVec2<Au>) -> Self {
        Self {
            inline: size.inline,
            block: SizeConstraint::Definite(size.block),
        }
    }
}

impl From<&ContainingBlockSize> for LogicalVec2<Option<Au>> {
    #[inline]
    fn from(size: &ContainingBlockSize) -> Self {
        Self {
            inline: Some(size.inline),
            block: size.block_size(),
        }
    }
}

impl From<&LogicalVec2<Au>> for LogicalVec2<Option<Au>> {
    #[inline]
    fn from(size: &LogicalVec2<Au>) -> Self {
        Self {
            inline: Some(size.inline),
            block: Some(size.block),
        }
    }
}

/* Containing block conversions */

impl<'a> From<&'_ DefiniteContainingBlock<'a>> for ContainingBlock<'a> {
    #[inline]
    fn from(containing_block: &DefiniteContainingBlock<'a>) -> Self {
        ContainingBlock {
            size: (&containing_block.size).into(),
            style: containing_block.style,
        }
    }
}

impl<'a> From<&'_ ContainingBlock<'a>> for IndefiniteContainingBlock<'a> {
    #[inline]
    fn from(containing_block: &ContainingBlock<'a>) -> Self {
        Self {
            size: (&containing_block.size).into(),
            style: containing_block.style,
        }
    }
}

impl<'a> From<&'_ DefiniteContainingBlock<'a>> for IndefiniteContainingBlock<'a> {
    #[inline]
    fn from(containing_block: &DefiniteContainingBlock<'a>) -> Self {
        Self {
            size: (&containing_block.size).into(),
            style: containing_block.style,
        }
    }
}

impl<'a> From<&ConstraintSpace<'a>> for IndefiniteContainingBlock<'a> {
    #[inline]
    fn from(constraint_space: &ConstraintSpace<'a>) -> Self {
        Self {
            size: LogicalVec2 {
                inline: None,
                block: constraint_space.block_size.to_definite(),
            },
            style: constraint_space.style,
        }
    }
}
