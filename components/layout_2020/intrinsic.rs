use crate::style_ext::ComputedValuesExt;
use style::properties::ComputedValues;
use style::values::computed::{Length, LengthPercentage, Percentage};
use style::Zero;

#[derive(Debug, Clone)]
pub(crate) struct IntrinsicSizes {
    pub min_content: Length,
    pub max_content: Length,
}

impl IntrinsicSizes {
    pub fn zero() -> Self {
        Self {
            min_content: Length::zero(),
            max_content: Length::zero(),
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

/// https://dbaron.org/css/intrinsic/#outer-intrinsic
pub(crate) fn outer_intrinsic_inline_sizes(
    style: &ComputedValues,
    get_inner_intrinsic_sizes: &dyn Fn() -> IntrinsicSizes,
) -> (IntrinsicSizes, Percentage) {
    // FIXME: account for 'min-width', 'max-width', 'box-sizing'

    let specified = style.box_size().inline;
    // Percentages for 'width' are treated as 'auto'
    let specified = specified.map(|lp| lp.as_length());
    // The (inner) min/max-content are only used for 'auto'
    let mut outer = match specified.non_auto().flatten() {
        None => get_inner_intrinsic_sizes(),
        Some(length) => IntrinsicSizes {
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
