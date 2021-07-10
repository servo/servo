/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

//! https://drafts.csswg.org/css-sizing/

use crate::style_ext::ComputedValuesExt;
use style::logical_geometry::WritingMode;
use style::properties::longhands::box_sizing::computed_value::T as BoxSizing;
use style::properties::ComputedValues;
use style::values::computed::{Length, LengthPercentage, Percentage};
use style::Zero;

#[derive(Clone, Debug, Serialize)]
pub(crate) struct ContentSizes {
    pub min_content: Length,
    pub max_content: Length,
}

/// https://drafts.csswg.org/css-sizing/#intrinsic-sizes
impl ContentSizes {
    pub fn zero() -> Self {
        Self {
            min_content: Length::zero(),
            max_content: Length::zero(),
        }
    }

    pub fn map(&self, f: impl Fn(Length) -> Length) -> Self {
        Self {
            min_content: f(self.min_content),
            max_content: f(self.max_content),
        }
    }

    pub fn max(self, other: Self) -> Self {
        Self {
            min_content: self.min_content.max(other.min_content),
            max_content: self.max_content.max(other.max_content),
        }
    }

    /// Relevant to outer intrinsic inline sizes, for percentages from padding and margin.
    pub fn adjust_for_pbm_percentages(&mut self, percentages: Percentage) {
        // " Note that this may yield an infinite result, but undefined results
        //   (zero divided by zero) must be treated as zero. "
        if self.max_content.px() == 0. {
            // Avoid a potential `NaN`.
            // Zero is already the result we want regardless of `denominator`.
        } else {
            let denominator = (1. - percentages.0).max(0.);
            self.max_content = Length::new(self.max_content.px() / denominator);
        }
    }
}

impl ContentSizes {
    /// https://drafts.csswg.org/css2/visudet.html#shrink-to-fit-float
    pub fn shrink_to_fit(&self, available_size: Length) -> Length {
        available_size.max(self.min_content).min(self.max_content)
    }
}

pub(crate) fn outer_inline(
    style: &ComputedValues,
    containing_block_writing_mode: WritingMode,
    get_content_size: impl FnOnce() -> ContentSizes,
) -> ContentSizes {
    let (mut outer, percentages) =
        outer_inline_and_percentages(style, containing_block_writing_mode, get_content_size);
    outer.adjust_for_pbm_percentages(percentages);
    outer
}

pub(crate) fn outer_inline_and_percentages(
    style: &ComputedValues,
    containing_block_writing_mode: WritingMode,
    get_content_size: impl FnOnce() -> ContentSizes,
) -> (ContentSizes, Percentage) {
    let padding = style.padding(containing_block_writing_mode);
    let border = style.border_width(containing_block_writing_mode);
    let margin = style.margin(containing_block_writing_mode);

    let mut pbm_percentages = Percentage::zero();
    let mut decompose = |x: &LengthPercentage| {
        pbm_percentages += x.to_percentage().unwrap_or_else(Zero::zero);
        x.to_length().unwrap_or_else(Zero::zero)
    };
    let pb_lengths =
        border.inline_sum() + decompose(padding.inline_start) + decompose(padding.inline_end);
    let mut m_lengths = Length::zero();
    if let Some(m) = margin.inline_start.non_auto() {
        m_lengths += decompose(m)
    }
    if let Some(m) = margin.inline_end.non_auto() {
        m_lengths += decompose(m)
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
        .percentage_relative_to(Length::zero())
        // FIXME: 'auto' is not zero in Flexbox
        .auto_is(Length::zero);
    let max_inline_size = style
        .max_box_size(containing_block_writing_mode)
        .inline
        // Percentages for 'max-width' are treated as 'none'
        .and_then(|lp| lp.to_length());
    let clamp = |l: Length| l.clamp_between_extremums(min_inline_size, max_inline_size);

    let border_box_sizes = match inline_size {
        Some(non_auto) => {
            let clamped = clamp(non_auto);
            let border_box_size = match box_sizing {
                BoxSizing::ContentBox => clamped + pb_lengths,
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
                BoxSizing::ContentBox => clamp(content_box_size) + pb_lengths,
                BoxSizing::BorderBox => clamp(content_box_size + pb_lengths),
            }
        }),
    };

    let outer = border_box_sizes.map(|s| s + m_lengths);
    (outer, pbm_percentages)
}
