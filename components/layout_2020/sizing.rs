/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

//! <https://drafts.csswg.org/css-sizing/>

use std::cell::LazyCell;
use std::ops::{Add, AddAssign};

use app_units::Au;
use style::Zero;
use style::values::computed::LengthPercentage;

use crate::context::LayoutContext;
use crate::geom::Size;
use crate::style_ext::{AspectRatio, Clamp, ComputedValuesExt, ContentBoxSizesAndPBM, LayoutStyle};
use crate::{ConstraintSpace, IndefiniteContainingBlock, LogicalVec2};

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

#[derive(Clone, Copy, Debug, Default)]
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

    pub fn map(&self, f: impl Fn(Au) -> Au) -> Self {
        Self {
            min_content: f(self.min_content),
            max_content: f(self.max_content),
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

#[allow(clippy::too_many_arguments)]
pub(crate) fn outer_inline(
    layout_style: &LayoutStyle,
    containing_block: &IndefiniteContainingBlock,
    auto_minimum: &LogicalVec2<Au>,
    auto_block_size_stretches_to_containing_block: bool,
    is_replaced: bool,
    establishes_containing_block: bool,
    get_preferred_aspect_ratio: impl FnOnce(&LogicalVec2<Au>) -> Option<AspectRatio>,
    get_content_size: impl FnOnce(&ConstraintSpace) -> InlineContentSizesResult,
) -> InlineContentSizesResult {
    let ContentBoxSizesAndPBM {
        content_box_sizes,
        pbm,
        mut depends_on_block_constraints,
    } = layout_style.content_box_sizes_and_padding_border_margin(containing_block);
    let margin = pbm.margin.map(|v| v.auto_is(Au::zero));
    let pbm_sums = LogicalVec2 {
        block: pbm.padding_border_sums.block + margin.block_sum(),
        inline: pbm.padding_border_sums.inline + margin.inline_sum(),
    };
    let style = layout_style.style();
    let content_size = LazyCell::new(|| {
        let constraint_space = if establishes_containing_block {
            let available_block_size = containing_block
                .size
                .block
                .map(|v| Au::zero().max(v - pbm_sums.block));
            let automatic_size = if content_box_sizes.block.preferred.is_initial() &&
                auto_block_size_stretches_to_containing_block
            {
                depends_on_block_constraints = true;
                Size::Stretch
            } else {
                Size::FitContent
            };
            ConstraintSpace::new(
                content_box_sizes.block.resolve_extrinsic(
                    automatic_size,
                    auto_minimum.block,
                    available_block_size,
                ),
                style.writing_mode,
                get_preferred_aspect_ratio(&pbm.padding_border_sums),
            )
        } else {
            // This assumes that there is no preferred aspect ratio, or that there is no
            // block size constraint to be transferred so the ratio is irrelevant.
            // We only get into here for anonymous blocks, for which the assumption holds.
            ConstraintSpace::new(
                containing_block.size.block.into(),
                containing_block.writing_mode,
                None,
            )
        };
        get_content_size(&constraint_space)
    });
    let resolve_non_initial = |inline_size, stretch_values| {
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
            Size::FitContent => (
                content_size.sizes.min_content,
                content_size.sizes.max_content,
                content_size.depends_on_block_constraints,
            ),
            Size::FitContentFunction(size) => {
                let size = content_size.sizes.shrink_to_fit(size);
                (size, size, content_size.depends_on_block_constraints)
            },
            Size::Stretch => return stretch_values,
        })
    };
    let (mut preferred_min_content, preferred_max_content, preferred_depends_on_block_constraints) =
        resolve_non_initial(content_box_sizes.inline.preferred, None)
            .unwrap_or_else(|| resolve_non_initial(Size::FitContent, None).unwrap());
    let (mut min_min_content, mut min_max_content, mut min_depends_on_block_constraints) =
        resolve_non_initial(
            content_box_sizes.inline.min,
            Some((Au::zero(), Au::zero(), false)),
        )
        .unwrap_or((auto_minimum.inline, auto_minimum.inline, false));
    let (mut max_min_content, max_max_content, max_depends_on_block_constraints) =
        resolve_non_initial(content_box_sizes.inline.max, None)
            .map(|(min_content, max_content, depends_on_block_constraints)| {
                (
                    Some(min_content),
                    Some(max_content),
                    depends_on_block_constraints,
                )
            })
            .unwrap_or_default();

    // https://drafts.csswg.org/css-sizing-3/#replaced-percentage-min-contribution
    // > If the box is replaced, a cyclic percentage in the value of any max size property
    // > or preferred size property (width/max-width/height/max-height), is resolved against
    // > zero when calculating the min-content contribution in the corresponding axis.
    //
    // This means that e.g. the min-content contribution of `width: calc(100% + 100px)`
    // should be 100px, but it's just zero on other browsers, so we do the same.
    if is_replaced {
        let has_percentage = |size: Size<LengthPercentage>| {
            // We need a comment here to avoid breaking `./mach test-tidy`.
            matches!(size, Size::Numeric(numeric) if numeric.has_percentage())
        };
        if content_box_sizes.inline.preferred.is_initial() &&
            has_percentage(style.box_size(containing_block.writing_mode).inline)
        {
            preferred_min_content = Au::zero();
        }
        if content_box_sizes.inline.max.is_initial() &&
            has_percentage(style.max_box_size(containing_block.writing_mode).inline)
        {
            max_min_content = Some(Au::zero());
        }
    }

    // Regardless of their sizing properties, tables are always forced to be at least
    // as big as their min-content size, so floor the minimums.
    if layout_style.is_table() {
        min_min_content.max_assign(content_size.sizes.min_content);
        min_max_content.max_assign(content_size.sizes.min_content);
        min_depends_on_block_constraints |= content_size.depends_on_block_constraints;
    }

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

#[derive(Clone, Copy, Debug)]
pub(crate) struct InlineContentSizesResult {
    pub sizes: ContentSizes,
    pub depends_on_block_constraints: bool,
}

pub(crate) trait ComputeInlineContentSizes {
    fn compute_inline_content_sizes(
        &self,
        layout_context: &LayoutContext,
        constraint_space: &ConstraintSpace,
    ) -> InlineContentSizesResult;
}
