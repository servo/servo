/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

//! https://drafts.csswg.org/css-sizing/

use crate::style_ext::ComputedValuesExt;
use style::properties::ComputedValues;
use style::values::computed::{Length, LengthPercentage, Percentage};
use style::values::generics::length::MaxSize;
use style::Zero;

/// Which min/max-content values should be computed during box construction
#[derive(Clone, Copy, Debug)]
pub(crate) enum ContentSizesRequest {
    Inline,
    None,
}

impl ContentSizesRequest {
    pub fn inline_if(condition: bool) -> Self {
        if condition {
            Self::Inline
        } else {
            Self::None
        }
    }

    pub fn requests_inline(self) -> bool {
        match self {
            Self::Inline => true,
            Self::None => false,
        }
    }

    pub fn if_requests_inline<T>(self, f: impl FnOnce() -> T) -> Option<T> {
        match self {
            Self::Inline => Some(f()),
            Self::None => None,
        }
    }

    pub fn compute(self, compute_inline: impl FnOnce() -> ContentSizes) -> BoxContentSizes {
        match self {
            Self::Inline => BoxContentSizes::Inline(compute_inline()),
            Self::None => BoxContentSizes::NoneWereRequested,
        }
    }
}

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

/// Optional min/max-content for storage in the box tree
#[derive(Debug, Serialize)]
pub(crate) enum BoxContentSizes {
    NoneWereRequested, // … during box construction
    Inline(ContentSizes),
}

impl BoxContentSizes {
    fn expect_inline(&self) -> &ContentSizes {
        match self {
            Self::NoneWereRequested => panic!("Accessing content size that was not requested"),
            Self::Inline(s) => s,
        }
    }

    /// https://dbaron.org/css/intrinsic/#outer-intrinsic
    pub fn outer_inline(&self, style: &ComputedValues) -> ContentSizes {
        let (mut outer, percentages) = self.outer_inline_and_percentages(style);
        outer.adjust_for_pbm_percentages(percentages);
        outer
    }

    pub(crate) fn outer_inline_and_percentages(
        &self,
        style: &ComputedValues,
    ) -> (ContentSizes, Percentage) {
        // FIXME: account for 'box-sizing'
        let inline_size = style.box_size().inline;
        let min_inline_size = style
            .min_box_size()
            .inline
            .percentage_relative_to(Length::zero())
            .auto_is(Length::zero);
        let max_inline_size = match style.max_box_size().inline {
            MaxSize::None => None,
            MaxSize::LengthPercentage(ref lp) => lp.to_length(),
        };
        let clamp = |l: Length| l.clamp_between_extremums(min_inline_size, max_inline_size);

        // Percentages for 'width' are treated as 'auto'
        let inline_size = inline_size.map(|lp| lp.to_length());
        // The (inner) min/max-content are only used for 'auto'
        let mut outer = match inline_size.non_auto().flatten() {
            None => {
                let inner = self.expect_inline().clone();
                ContentSizes {
                    min_content: clamp(inner.min_content),
                    max_content: clamp(inner.max_content),
                }
            },
            Some(length) => {
                let length = clamp(length);
                ContentSizes {
                    min_content: length,
                    max_content: length,
                }
            },
        };

        let mut pbm_lengths = Length::zero();
        let mut pbm_percentages = Percentage::zero();
        let padding = style.padding();
        let border = style.border_width();
        let margin = style.margin();
        pbm_lengths += border.inline_sum();
        let mut add = |x: LengthPercentage| {
            if let Some(l) = x.to_length() {
                pbm_lengths += l;
            }
            if let Some(p) = x.to_percentage() {
                pbm_percentages += p;
            }
        };
        add(padding.inline_start);
        add(padding.inline_end);
        margin.inline_start.non_auto().map(&mut add);
        margin.inline_end.non_auto().map(&mut add);

        outer.min_content += pbm_lengths;
        outer.max_content += pbm_lengths;

        (outer, pbm_percentages)
    }

    /// https://drafts.csswg.org/css2/visudet.html#shrink-to-fit-float
    pub(crate) fn shrink_to_fit(&self, available_size: Length) -> Length {
        let inline = self.expect_inline();
        available_size
            .max(inline.min_content)
            .min(inline.max_content)
    }
}
