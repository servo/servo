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
    get_content_size: impl FnOnce() -> ContentSizes,
) -> ContentSizes {
    let padding = style.padding(containing_block_writing_mode);
    let border = style.border_width(containing_block_writing_mode);
    let margin = style.margin(containing_block_writing_mode);

    // For margins and paddings, a cyclic percentage is resolved against zero
    // for determining intrinsic size contributions.
    // https://drafts.csswg.org/css-sizing-3/#min-percentage-contribution
    let zero = Length::zero();
    let pb_lengths = border.inline_sum() +
        padding.inline_start.percentage_relative_to(zero) +
        padding.inline_end.percentage_relative_to(zero);
    let mut m_lengths = zero;
    if let Some(m) = margin.inline_start.non_auto() {
        m_lengths += m.percentage_relative_to(zero)
    }
    if let Some(m) = margin.inline_end.non_auto() {
        m_lengths += m.percentage_relative_to(zero)
    }

    let box_sizing = style.get_position().box_sizing;
    let inline_size = style
        .box_size(containing_block_writing_mode)
        .inline
        .non_auto()
        // Percentages for 'width' are treated as 'auto'
        .and_then(|lp| lp.to_length());
    let min_inline_size = style
        .min_box_size(containing_block_writing_mode)
        .inline
        // Percentages for 'min-width' are treated as zero
        .percentage_relative_to(zero)
        // FIXME: 'auto' is not zero in Flexbox
        .auto_is(Length::zero);
    let max_inline_size = style
        .max_box_size(containing_block_writing_mode)
        .inline
        // Percentages for 'max-width' are treated as 'none'
        .and_then(|lp| lp.to_length());
    let clamp = |l: Au| {
        l.clamp_between_extremums(min_inline_size.into(), max_inline_size.map(|t| t.into()))
    };

    let border_box_sizes = match inline_size {
        Some(non_auto) => {
            let clamped = clamp(non_auto.into());
            let border_box_size = match box_sizing {
                BoxSizing::ContentBox => clamped + pb_lengths.into(),
                BoxSizing::BorderBox => clamped,
            };
            ContentSizes {
                min_content: border_box_size,
                max_content: border_box_size,
            }
        },
        None => get_content_size().map(|content_box_size| {
            match box_sizing {
                // Clamp to 'min-width' and 'max-width', which are sizing the…
                BoxSizing::ContentBox => clamp(content_box_size) + pb_lengths.into(),
                BoxSizing::BorderBox => clamp(content_box_size + pb_lengths.into()),
            }
        }),
    };

    border_box_sizes.map(|s| s + m_lengths.into())
}
