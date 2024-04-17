/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

//! <https://drafts.csswg.org/css-sizing/>

use std::ops::{Add, AddAssign};

use app_units::Au;
use serde::Serialize;
use style::logical_geometry::WritingMode;
use style::properties::longhands::box_sizing::computed_value::T as BoxSizing;
use style::properties::ComputedValues;
use style::values::computed::Length;
use style::Zero;

use crate::style_ext::{Clamp, ComputedValuesExt};

#[derive(Clone, Copy, Debug, Serialize)]
pub(crate) struct ContentSizes {
    pub min_content: Au,
    pub max_content: Au,
}

/// <https://drafts.csswg.org/css-sizing/#intrinsic-sizes>
impl ContentSizes {
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
        Self {
            min_content: Au::zero(),
            max_content: Au::zero(),
        }
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

pub(crate) fn outer_inline(
    style: &ComputedValues,
    containing_block_writing_mode: WritingMode,
    is_replaced: bool,
    get_content_size: impl FnOnce() -> ContentSizes,
) -> ContentSizes {
    let padding = style.padding(containing_block_writing_mode);
    let border = style.border_width(containing_block_writing_mode);
    let margin = style.margin(containing_block_writing_mode);
    let box_sizing = style.get_position().box_sizing;

    // For margins and paddings, a cyclic percentage is resolved against zero
    // for determining intrinsic size contributions.
    // https://drafts.csswg.org/css-sizing-3/#min-percentage-contribution
    let zero = Length::zero();
    let pb = (border.inline_sum() +
        padding.inline_start.percentage_relative_to(zero) +
        padding.inline_end.percentage_relative_to(zero))
    .into();
    let mut margins = Au::zero();
    if let Some(m) = margin.inline_start.non_auto() {
        margins += m.percentage_relative_to(zero).into()
    }
    if let Some(m) = margin.inline_end.non_auto() {
        margins += m.percentage_relative_to(zero).into()
    }

    // Percentages for 'width' are treated as 'auto', except for the min-content
    // contribution of replaced elements, then they resolve against zero instead.
    // https://drafts.csswg.org/css-sizing-3/#non-replaced-percentage-contribution
    // https://drafts.csswg.org/css-sizing-3/#replaced-percentage-max-contribution
    // https://drafts.csswg.org/css-sizing-3/#replaced-percentage-min-contribution
    let inline_size = style
        .box_size(containing_block_writing_mode)
        .inline
        .non_auto();
    let inline_size_for_max_content = inline_size.and_then(|lp| lp.to_length()).map(Au::from);
    let inline_size_for_min_content = if is_replaced {
        inline_size.map(|lp| lp.percentage_relative_to(zero).into())
    } else {
        inline_size_for_max_content
    };

    // Percentages for 'max-width' are treated as 'none', except for the min-content
    // contribution of replaced elements, then they resolve against zero instead.
    // https://drafts.csswg.org/css-sizing-3/#non-replaced-percentage-contribution
    // https://drafts.csswg.org/css-sizing-3/#replaced-percentage-max-contribution
    // https://drafts.csswg.org/css-sizing-3/#replaced-percentage-min-contribution
    let max_inline_size = style.max_box_size(containing_block_writing_mode).inline;
    let max_inline_size_for_max_content =
        max_inline_size.and_then(|lp| lp.to_length()).map(Au::from);
    let max_inline_size_for_min_content = if is_replaced {
        max_inline_size.map(|lp| lp.percentage_relative_to(zero).into())
    } else {
        max_inline_size_for_max_content
    };

    // Percentages for 'min-width' are resolved against zero.
    // https://drafts.csswg.org/css-sizing-3/#min-percentage-contribution
    let min_inline_size = style
        .min_box_size(containing_block_writing_mode)
        .inline
        .percentage_relative_to(zero)
        .map(Au::from)
        // FIXME: 'auto' is not zero in Flexbox
        .auto_is(Au::zero);

    let border_box_size = |min, preferred: Option<Au>, max, content: Au| {
        let clamp = |size: Au| size.clamp_between_extremums(min, max);
        match box_sizing {
            BoxSizing::ContentBox => clamp(preferred.unwrap_or(content)) + pb,
            BoxSizing::BorderBox => clamp(preferred.unwrap_or(content + pb)),
        }
    };

    let content_box_sizes =
        if inline_size_for_max_content.is_none() || inline_size_for_min_content.is_none() {
            get_content_size()
        } else {
            // The value is irrelevant.
            ContentSizes::zero()
        };
    ContentSizes {
        max_content: border_box_size(
            min_inline_size,
            inline_size_for_max_content,
            max_inline_size_for_max_content,
            content_box_sizes.max_content,
        ) + margins,
        min_content: border_box_size(
            min_inline_size,
            inline_size_for_min_content,
            max_inline_size_for_min_content,
            content_box_sizes.min_content,
        ) + margins,
    }
}
