/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

//! <https://drafts.csswg.org/css-sizing/>

use std::cell::LazyCell;
use std::ops::{Add, AddAssign};

use app_units::Au;
use serde::Serialize;
use style::properties::ComputedValues;
use style::Zero;

use crate::geom::Size;
use crate::style_ext::{Clamp, ComputedValuesExt, ContentBoxSizesAndPBM};
use crate::{AuOrAuto, IndefiniteContainingBlock, LogicalVec2};

#[derive(PartialEq)]
pub(crate) enum IntrinsicSizingMode {
    /// Used to refer to a min-content contribution or max-content contribution.
    /// This is the size that a box contributes to its containing block’s min-content
    /// or max-content size. Note this is based on the outer size of the box,
    /// and takes into account the relevant sizing properties of the element.
    /// <https://drafts.csswg.org/css-sizing-3/#contributions>
    Contribution,
    /// Used to refer to a min-content size or max-content size.
    /// This is the size based on the contents of an element, without regard for its context.
    /// Note this is usually based on the inner (content-box) size of the box,
    /// and ignores the relevant sizing properties of the element.
    /// <https://drafts.csswg.org/css-sizing-3/#intrinsic>
    Size,
}

#[derive(Clone, Copy, Debug, Default, Serialize)]
pub(crate) struct ContentSizes {
    pub min_content: Au,
    pub max_content: Au,
}

/// <https://drafts.csswg.org/css-sizing/#intrinsic-sizes>
impl ContentSizes {
    pub fn map(&self, f: impl Fn(Au) -> Au) -> Self {
        Self {
            min_content: f(self.min_content),
            max_content: f(self.max_content),
        }
    }

    pub fn max(&self, other: Self) -> Self {
        Self {
            min_content: self.min_content.max(other.min_content),
            max_content: self.max_content.max(other.max_content),
        }
    }

    pub fn max_assign(&mut self, other: Self) {
        *self = self.max(other);
    }

    pub fn union(&self, other: &Self) -> Self {
        Self {
            min_content: self.min_content.max(other.min_content),
            max_content: self.max_content + other.max_content,
        }
    }
}

impl Zero for ContentSizes {
    fn zero() -> Self {
        Au::zero().into()
    }

    fn is_zero(&self) -> bool {
        self.min_content.is_zero() && self.max_content.is_zero()
    }
}

impl Add for ContentSizes {
    type Output = Self;

    fn add(self, rhs: Self) -> Self {
        Self {
            min_content: self.min_content + rhs.min_content,
            max_content: self.max_content + rhs.max_content,
        }
    }
}

impl AddAssign for ContentSizes {
    fn add_assign(&mut self, rhs: Self) {
        *self = self.add(rhs)
    }
}

impl ContentSizes {
    /// Clamps the provided amount to be between the min-content and the max-content.
    /// This is called "shrink-to-fit" in CSS2, and "fit-content" in CSS Sizing.
    /// <https://drafts.csswg.org/css2/visudet.html#shrink-to-fit-float>
    /// <https://drafts.csswg.org/css-sizing/#funcdef-width-fit-content>
    pub fn shrink_to_fit(&self, available_size: Au) -> Au {
        // This formula is slightly different than what the spec says,
        // to ensure that the minimum wins for a malformed ContentSize
        // whose min_content is larger than its max_content.
        available_size.min(self.max_content).max(self.min_content)
    }
}

impl From<Au> for ContentSizes {
    fn from(size: Au) -> Self {
        Self {
            min_content: size,
            max_content: size,
        }
    }
}

pub(crate) fn outer_inline(
    style: &ComputedValues,
    containing_block: &IndefiniteContainingBlock,
    auto_minimum: &LogicalVec2<Au>,
    auto_block_size_stretches_to_containing_block: bool,
    get_content_size: impl FnOnce(&IndefiniteContainingBlock) -> InlineContentSizesResult,
) -> InlineContentSizesResult {
    let ContentBoxSizesAndPBM {
        content_box_size,
        content_min_box_size,
        content_max_box_size,
        pbm,
        mut depends_on_block_constraints,
    } = style.content_box_sizes_and_padding_border_margin(containing_block);
    let margin = pbm.margin.map(|v| v.auto_is(Au::zero));
    let pbm_sums = LogicalVec2 {
        block: pbm.padding_border_sums.block + margin.block_sum(),
        inline: pbm.padding_border_sums.inline + margin.inline_sum(),
    };
    let content_size = LazyCell::new(|| {
        let available_block_size = containing_block
            .size
            .block
            .non_auto()
            .map(|v| Au::zero().max(v - pbm_sums.block));
        let block_size = if content_box_size.block.is_initial() &&
            auto_block_size_stretches_to_containing_block
        {
            depends_on_block_constraints = true;
            available_block_size
        } else {
            content_box_size
                .block
                .maybe_resolve_extrinsic(available_block_size)
        }
        .map(|block_size| {
            let min_block_size = content_min_box_size
                .block
                .maybe_resolve_extrinsic(available_block_size)
                .unwrap_or(auto_minimum.block);
            let max_block_size = content_max_box_size
                .block
                .maybe_resolve_extrinsic(available_block_size);
            block_size.clamp_between_extremums(min_block_size, max_block_size)
        })
        .map_or(AuOrAuto::Auto, AuOrAuto::LengthPercentage);
        let containing_block_for_children =
            IndefiniteContainingBlock::new_for_writing_mode_and_block_size(
                style.writing_mode,
                block_size,
            );
        get_content_size(&containing_block_for_children)
    });
    let resolve_non_initial = |inline_size| {
        Some(match inline_size {
            Size::Initial => return None,
            Size::Numeric(numeric) => (numeric, numeric, false),
            Size::MinContent => (
                content_size.sizes.min_content,
                content_size.sizes.min_content,
                content_size.depends_on_block_constraints,
            ),
            Size::MaxContent => (
                content_size.sizes.max_content,
                content_size.sizes.max_content,
                content_size.depends_on_block_constraints,
            ),
            Size::Stretch | Size::FitContent => (
                content_size.sizes.min_content,
                content_size.sizes.max_content,
                content_size.depends_on_block_constraints,
            ),
        })
    };
    let (preferred_min_content, preferred_max_content, preferred_depends_on_block_constraints) =
        resolve_non_initial(content_box_size.inline)
            .unwrap_or_else(|| resolve_non_initial(Size::FitContent).unwrap());
    let (min_min_content, min_max_content, min_depends_on_block_constraints) = resolve_non_initial(
        content_min_box_size.inline,
    )
    .unwrap_or((auto_minimum.inline, auto_minimum.inline, false));
    let (max_min_content, max_max_content, max_depends_on_block_constraints) =
        resolve_non_initial(content_max_box_size.inline)
            .map(|(min_content, max_content, depends_on_block_constraints)| {
                (
                    Some(min_content),
                    Some(max_content),
                    depends_on_block_constraints,
                )
            })
            .unwrap_or_default();
    InlineContentSizesResult {
        sizes: ContentSizes {
            min_content: preferred_min_content
                .clamp_between_extremums(min_min_content, max_min_content) +
                pbm_sums.inline,
            max_content: preferred_max_content
                .clamp_between_extremums(min_max_content, max_max_content) +
                pbm_sums.inline,
        },
        depends_on_block_constraints: depends_on_block_constraints &&
            (preferred_depends_on_block_constraints ||
                min_depends_on_block_constraints ||
                max_depends_on_block_constraints),
    }
}

#[derive(Clone, Copy, Debug, Serialize)]
pub(crate) struct InlineContentSizesResult {
    pub sizes: ContentSizes,
    pub depends_on_block_constraints: bool,
}
