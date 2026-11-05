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

/// Whether the block size of a containing block was affected by the percentage quirk.
/// If true, it means the actual block size is indefinite, but it has been overriden
/// with a definite amount, in order to resolve percentages in quirks mode.
/// <https://quirks.spec.whatwg.org/#the-percentage-height-calculation-quirk>
#[derive(Clone, Copy, Debug, Default, MallocSizeOf, PartialEq)]
pub(crate) struct HasQuirkyBlockSize(pub bool);

/// Represents the set of constraints that we use when computing the min-content
/// and max-content inline sizes of an element.
pub(crate) struct ConstraintSpace<'a> {
    pub block_size: SizeConstraint,
    pub has_quirky_block_size: HasQuirkyBlockSize,
    pub style: &'a ComputedValues,
    pub preferred_aspect_ratio: Option<AspectRatio>,
}

impl<'a> ConstraintSpace<'a> {
    pub fn new(
        block_size: SizeConstraint,
        has_quirky_block_size: HasQuirkyBlockSize,
        style: &'a ComputedValues,
        preferred_aspect_ratio: Option<AspectRatio>,
    ) -> Self {
        Self {
            block_size,
            has_quirky_block_size,
            style,
            preferred_aspect_ratio,
        }
    }
}

#[inline]
fn inline_stretch_size(inline_size: Option<Au>, inline_pbm_sum: Au) -> Option<Au> {
    inline_size.map(|size| Au::zero().max(size - inline_pbm_sum))
}

#[inline]
fn block_stretch_size(
    block_size: Option<Au>,
    has_quirky_block_size: HasQuirkyBlockSize,
    block_pbm_sum: Au,
) -> Option<Au> {
    if has_quirky_block_size.0 {
        // The actual block size of the containing block is indefinite,
        // but the value was overridden in order to resolve percentages.
        // However, now we are resolving `stretch` instead of a percentage.
        return None;
    }
    block_size.map(|size| Au::zero().max(size - block_pbm_sum))
}

#[derive(Default)]
pub(crate) struct IndefiniteContainingBlockSize {
    pub inline: Option<Au>,
    pub block: Option<Au>,
    pub has_quirky_block_size: HasQuirkyBlockSize,
}

impl IndefiniteContainingBlockSize {
    #[inline]
    pub fn as_logical_vec2(&self) -> LogicalVec2<Option<Au>> {
        LogicalVec2 {
            inline: self.inline,
            block: self.block,
        }
    }
}

/// A variant of [`ContainingBlock`] that allows an indefinite inline size.
/// Useful for code that is shared for both layout (where we know the inline size
/// of the containing block) and intrinsic sizing (where we don't know it).
pub(crate) struct IndefiniteContainingBlock<'a> {
    pub size: IndefiniteContainingBlockSize,
    pub style: &'a ComputedValues,
}

impl IndefiniteContainingBlock<'_> {
    #[inline]
    pub fn block_stretch_size(&self, block_pbm_sum: Au) -> Option<Au> {
        block_stretch_size(
            self.size.block,
            self.size.has_quirky_block_size,
            block_pbm_sum,
        )
    }

    #[inline]
    pub fn has_quirky_block_size(&self) -> bool {
        self.size.has_quirky_block_size.0
    }
}

impl From<&ContainingBlockSize> for IndefiniteContainingBlockSize {
    fn from(size: &ContainingBlockSize) -> Self {
        Self {
            inline: Some(size.inline),
            block: size.block.to_definite(),
            has_quirky_block_size: size.has_quirky_block_size,
        }
    }
}

impl<'a> From<&ConstraintSpace<'a>> for IndefiniteContainingBlock<'a> {
    fn from(constraint_space: &ConstraintSpace<'a>) -> Self {
        Self {
            size: IndefiniteContainingBlockSize {
                inline: None,
                block: constraint_space.block_size.to_definite(),
                has_quirky_block_size: constraint_space.has_quirky_block_size,
            },
            style: constraint_space.style,
        }
    }
}

impl<'a> From<&'_ ContainingBlock<'a>> for IndefiniteContainingBlock<'a> {
    fn from(containing_block: &ContainingBlock<'a>) -> Self {
        Self {
            size: (&containing_block.size).into(),
            style: containing_block.style,
        }
    }
}

impl<'a> From<&'_ DefiniteContainingBlock<'a>> for IndefiniteContainingBlock<'a> {
    fn from(containing_block: &DefiniteContainingBlock<'a>) -> Self {
        Self {
            size: IndefiniteContainingBlockSize {
                inline: Some(containing_block.size.inline),
                block: Some(containing_block.size.block),
                has_quirky_block_size: HasQuirkyBlockSize(false),
            },
            style: containing_block.style,
        }
    }
}

#[derive(Clone, Debug, MallocSizeOf, PartialEq)]
pub(crate) struct ContainingBlockSize {
    pub inline: Au,
    pub block: SizeConstraint,
    pub has_quirky_block_size: HasQuirkyBlockSize,
}

pub(crate) struct ContainingBlock<'a> {
    pub size: ContainingBlockSize,
    pub style: &'a ComputedValues,
}

impl ContainingBlock<'_> {
    #[inline]
    pub fn inline_stretch_size(&self, inline_pbm_sum: Au) -> Option<Au> {
        inline_stretch_size(Some(self.size.inline), inline_pbm_sum)
    }

    #[inline]
    pub fn block_stretch_size(&self, block_pbm_sum: Au) -> Option<Au> {
        block_stretch_size(
            self.size.block.to_definite(),
            self.size.has_quirky_block_size,
            block_pbm_sum,
        )
    }

    #[inline]
    pub fn has_quirky_block_size(&self) -> bool {
        self.size.has_quirky_block_size.0
    }
}

pub(crate) struct DefiniteContainingBlock<'a> {
    pub size: LogicalVec2<Au>,
    pub style: &'a ServoArc<ComputedValues>,
}

impl<'a> From<&'_ DefiniteContainingBlock<'a>> for ContainingBlock<'a> {
    fn from(definite: &DefiniteContainingBlock<'a>) -> Self {
        ContainingBlock {
            size: ContainingBlockSize {
                inline: definite.size.inline,
                block: SizeConstraint::Definite(definite.size.block),
                has_quirky_block_size: HasQuirkyBlockSize(false),
            },
            style: definite.style,
        }
    }
}
