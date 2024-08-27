/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

//! <https://drafts.csswg.org/css-sizing/>

use std::ops::{Add, AddAssign};

use app_units::Au;
use serde::Serialize;
use style::properties::ComputedValues;
use style::Zero;

use crate::style_ext::{Clamp, ComputedValuesExt};
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
    /// <https://drafts.csswg.org/css2/visudet.html#shrink-to-fit-float>
    pub fn shrink_to_fit(&self, available_size: Au) -> Au {
        available_size.max(self.min_content).min(self.max_content)
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
    get_content_size: impl FnOnce(&IndefiniteContainingBlock) -> ContentSizes,
) -> ContentSizes {
    let (content_box_size, content_min_size, content_max_size, pbm) =
        style.content_box_sizes_and_padding_border_margin(containing_block);
    let content_min_size = LogicalVec2 {
        inline: content_min_size.inline.auto_is(|| auto_minimum.inline),
        block: content_min_size.block.auto_is(|| auto_minimum.block),
    };
    let pbm_inline_sum = pbm.padding_border_sums.inline +
        pbm.margin.inline_start.auto_is(Au::zero) +
        pbm.margin.inline_end.auto_is(Au::zero);
    let adjust = |v: Au| {
        v.clamp_between_extremums(content_min_size.inline, content_max_size.inline) + pbm_inline_sum
    };
    match content_box_size.inline {
        AuOrAuto::LengthPercentage(inline_size) => adjust(inline_size).into(),
        AuOrAuto::Auto => {
            let block_size = content_box_size
                .block
                .map(|v| v.clamp_between_extremums(content_min_size.block, content_max_size.block));
            let containing_block_for_children =
                IndefiniteContainingBlock::new_for_style_and_block_size(style, block_size);
            get_content_size(&containing_block_for_children).map(adjust)
        },
    }
}
