/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

//! https://drafts.csswg.org/css-sizing/

use crate::style_ext::ComputedValuesExt;
use style::properties::ComputedValues;
use style::values::computed::{Length, LengthPercentage, Percentage};
use style::Zero;

#[derive(Clone, Debug)]
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

    pub fn max_assign(&mut self, other: &Self) {
        self.min_content.max_assign(other.min_content);
        self.max_content.max_assign(other.max_content);
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

/// https://dbaron.org/css/intrinsic/#outer-intrinsic
pub(crate) fn outer_inline_content_sizes(
    style: &ComputedValues,
    inner_content_sizes: &Option<ContentSizes>,
) -> ContentSizes {
    let (mut outer, percentages) =
        outer_inline_content_sizes_and_percentages(style, inner_content_sizes);
    outer.adjust_for_pbm_percentages(percentages);
    outer
}

pub(crate) fn outer_inline_content_sizes_and_percentages(
    style: &ComputedValues,
    inner_content_sizes: &Option<ContentSizes>,
) -> (ContentSizes, Percentage) {
    // FIXME: account for 'min-width', 'max-width', 'box-sizing'

    let specified = style.box_size().inline;
    // Percentages for 'width' are treated as 'auto'
    let specified = specified.map(|lp| lp.as_length());
    // The (inner) min/max-content are only used for 'auto'
    let mut outer = match specified.non_auto().flatten() {
        None => inner_content_sizes
            .as_ref()
            .expect("Accessing content size that was not requested")
            .clone(),
        Some(length) => ContentSizes {
            min_content: length,
            max_content: length,
        },
    };

    let mut pbm_lengths = Length::zero();
    let mut pbm_percentages = Percentage::zero();
    let padding = style.padding();
    let border = style.border_width();
    let margin = style.margin();
    pbm_lengths += border.inline_start;
    pbm_lengths += border.inline_end;
    let mut add = |x: LengthPercentage| {
        pbm_lengths += x.length_component();
        pbm_percentages += x.percentage_component();
    };
    add(padding.inline_start);
    add(padding.inline_end);
    margin.inline_start.non_auto().map(&mut add);
    margin.inline_end.non_auto().map(&mut add);

    outer.min_content += pbm_lengths;
    outer.max_content += pbm_lengths;

    (outer, pbm_percentages)
}
