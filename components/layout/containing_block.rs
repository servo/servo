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

    #[inline]
    pub fn has_quirky_block_size(&self) -> bool {
        self.size.has_quirky_block_size()
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
        if self.has_quirky_block_size() {
            // The actual block size of the containing block is indefinite,
            // but the value was overridden in order to resolve percentages.
            // However, now we are resolving `stretch` instead of a percentage.
            return None;
        }
        self.block_size()
            .map(|size| Au::zero().max(size - block_pbm_sum))
    }

    fn has_quirky_block_size(&self) -> bool;

    fn as_logical_vec2(&self) -> LogicalVec2<Option<Au>> {
        LogicalVec2 {
            inline: self.inline_size(),
            block: self.block_size(),
        }
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
    pub has_quirky_block_size: HasQuirkyBlockSize,
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

    #[inline]
    fn has_quirky_block_size(&self) -> bool {
        self.has_quirky_block_size.0
    }
}

/// A variant of [`ContainingBlock`] that allows an indefinite inline size.
/// Useful for code that is shared for both layout (where we know the inline size
/// of the containing block) and intrinsic sizing (where we don't know it).
pub(crate) type IndefiniteContainingBlock<'a> =
    GenericContainingBlock<IndefiniteContainingBlockSize, &'a ComputedValues>;

#[derive(Default)]
pub(crate) struct IndefiniteContainingBlockSize {
    pub inline: Option<Au>,
    pub block: Option<Au>,
    pub has_quirky_block_size: HasQuirkyBlockSize,
}

impl ContainingBlockSizeTrait for IndefiniteContainingBlockSize {
    #[inline]
    fn inline_size(&self) -> Option<Au> {
        self.inline
    }

    #[inline]
    fn block_size(&self) -> Option<Au> {
        self.block
    }

    #[inline]
    fn has_quirky_block_size(&self) -> bool {
        self.has_quirky_block_size.0
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

    #[inline]
    fn has_quirky_block_size(&self) -> bool {
        false
    }
}

/// Represents the set of constraints that we use when computing the min-content
/// and max-content inline sizes of an element.
pub(crate) struct ConstraintSpace<'a> {
    pub block_size: SizeConstraint,
    pub has_quirky_block_size: HasQuirkyBlockSize,
    pub style: &'a ComputedValues,
    pub preferred_aspect_ratio: Option<AspectRatio>,
}

impl<'a> ConstraintSpace<'a> {
    #[inline]
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

/* Size conversions */

impl From<&LogicalVec2<Au>> for ContainingBlockSize {
    #[inline]
    fn from(size: &LogicalVec2<Au>) -> Self {
        Self {
            inline: size.inline,
            block: SizeConstraint::Definite(size.block),
            has_quirky_block_size: HasQuirkyBlockSize(false),
        }
    }
}

impl From<&ContainingBlockSize> for IndefiniteContainingBlockSize {
    #[inline]
    fn from(size: &ContainingBlockSize) -> Self {
        Self {
            inline: Some(size.inline),
            block: size.block_size(),
            has_quirky_block_size: size.has_quirky_block_size,
        }
    }
}

impl From<&LogicalVec2<Au>> for IndefiniteContainingBlockSize {
    #[inline]
    fn from(size: &LogicalVec2<Au>) -> Self {
        Self {
            inline: Some(size.inline),
            block: Some(size.block),
            has_quirky_block_size: HasQuirkyBlockSize(false),
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
            size: IndefiniteContainingBlockSize {
                inline: None,
                block: constraint_space.block_size.to_definite(),
                has_quirky_block_size: constraint_space.has_quirky_block_size,
            },
            style: constraint_space.style,
        }
    }
}
