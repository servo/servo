/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

//! <https://drafts.csswg.org/css-sizing/>

use std::cell::{LazyCell, OnceCell};
use std::ops::{Add, AddAssign};

use app_units::{Au, MAX_AU};
use malloc_size_of_derive::MallocSizeOf;
use style::Zero;
use style::logical_geometry::Direction;
use style::values::computed::{
    LengthPercentage, MaxSize as StyleMaxSize, Percentage, Size as StyleSize,
};

use crate::context::LayoutContext;
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

#[derive(Clone, Copy, Debug, Default, MallocSizeOf)]
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
    get_inline_content_size: impl FnOnce(&ConstraintSpace) -> InlineContentSizesResult,
    get_tentative_block_content_size: impl FnOnce(Option<AspectRatio>) -> Option<ContentSizes>,
) -> InlineContentSizesResult {
    let ContentBoxSizesAndPBM {
        content_box_sizes,
        pbm,
        mut depends_on_block_constraints,
        preferred_size_computes_to_auto,
    } = layout_style.content_box_sizes_and_padding_border_margin(containing_block);
    let margin = pbm.margin.map(|v| v.auto_is(Au::zero));
    let pbm_sums = LogicalVec2 {
        block: pbm.padding_border_sums.block + margin.block_sum(),
        inline: pbm.padding_border_sums.inline + margin.inline_sum(),
    };
    let style = layout_style.style();
    let is_table = layout_style.is_table();
    let content_size = LazyCell::new(|| {
        let constraint_space = if establishes_containing_block {
            let available_block_size = containing_block
                .size
                .block
                .map(|v| Au::zero().max(v - pbm_sums.block));
            let automatic_size = if preferred_size_computes_to_auto.block &&
                auto_block_size_stretches_to_containing_block
            {
                depends_on_block_constraints = true;
                Size::Stretch
            } else {
                Size::FitContent
            };
            let aspect_ratio = get_preferred_aspect_ratio(&pbm.padding_border_sums);
            let block_size =
                if let Some(block_content_size) = get_tentative_block_content_size(aspect_ratio) {
                    SizeConstraint::Definite(content_box_sizes.block.resolve(
                        Direction::Block,
                        automatic_size,
                        || auto_minimum.block,
                        available_block_size,
                        || block_content_size,
                        is_table,
                    ))
                } else {
                    content_box_sizes.block.resolve_extrinsic(
                        automatic_size,
                        auto_minimum.block,
                        available_block_size,
                    )
                };
            ConstraintSpace::new(block_size, style.writing_mode, aspect_ratio)
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
        get_inline_content_size(&constraint_space)
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
    if is_table {
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

#[derive(Clone, Copy, Debug, MallocSizeOf)]
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

    /// Returns the same result as [`Self::compute_inline_content_sizes()`], but adjusted
    /// to floor the max-content size by the min-content size.
    /// This is being discussed in <https://github.com/w3c/csswg-drafts/issues/12076>.
    fn compute_inline_content_sizes_with_fixup(
        &self,
        layout_context: &LayoutContext,
        constraint_space: &ConstraintSpace,
    ) -> InlineContentSizesResult {
        let mut result = self.compute_inline_content_sizes(layout_context, constraint_space);
        let sizes = &mut result.sizes;
        sizes.max_content.max_assign(sizes.min_content);
        result
    }
}

/// The possible values accepted by the sizing properties.
/// <https://drafts.csswg.org/css-sizing/#sizing-properties>
#[derive(Clone, Debug, PartialEq)]
pub(crate) enum Size<T> {
    /// Represents an `auto` value for the preferred and minimum size properties,
    /// or `none` for the maximum size properties.
    /// <https://drafts.csswg.org/css-sizing/#valdef-width-auto>
    /// <https://drafts.csswg.org/css-sizing/#valdef-max-width-none>
    Initial,
    /// <https://drafts.csswg.org/css-sizing/#valdef-width-min-content>
    MinContent,
    /// <https://drafts.csswg.org/css-sizing/#valdef-width-max-content>
    MaxContent,
    /// <https://drafts.csswg.org/css-sizing-4/#valdef-width-fit-content>
    FitContent,
    /// <https://drafts.csswg.org/css-sizing-3/#funcdef-width-fit-content>
    FitContentFunction(T),
    /// <https://drafts.csswg.org/css-sizing-4/#valdef-width-stretch>
    Stretch,
    /// Represents a numeric `<length-percentage>`, but resolved as a `T`.
    /// <https://drafts.csswg.org/css-sizing/#valdef-width-length-percentage-0>
    Numeric(T),
}

impl<T: Copy> Copy for Size<T> {}

impl<T> Default for Size<T> {
    #[inline]
    fn default() -> Self {
        Self::Initial
    }
}

impl<T> Size<T> {
    #[inline]
    pub(crate) fn is_initial(&self) -> bool {
        matches!(self, Self::Initial)
    }
}

impl<T: Clone> Size<T> {
    #[inline]
    pub(crate) fn to_numeric(&self) -> Option<T> {
        match self {
            Self::Numeric(numeric) => Some(numeric).cloned(),
            _ => None,
        }
    }

    #[inline]
    pub(crate) fn map<U>(&self, f: impl FnOnce(T) -> U) -> Size<U> {
        match self {
            Size::Initial => Size::Initial,
            Size::MinContent => Size::MinContent,
            Size::MaxContent => Size::MaxContent,
            Size::FitContent => Size::FitContent,
            Size::FitContentFunction(size) => Size::FitContentFunction(f(size.clone())),
            Size::Stretch => Size::Stretch,
            Size::Numeric(numeric) => Size::Numeric(f(numeric.clone())),
        }
    }
}

impl From<StyleSize> for Size<LengthPercentage> {
    fn from(size: StyleSize) -> Self {
        match size {
            StyleSize::LengthPercentage(lp) => Size::Numeric(lp.0),
            StyleSize::Auto => Size::Initial,
            StyleSize::MinContent => Size::MinContent,
            StyleSize::MaxContent => Size::MaxContent,
            StyleSize::FitContent => Size::FitContent,
            StyleSize::FitContentFunction(lp) => Size::FitContentFunction(lp.0),
            StyleSize::Stretch => Size::Stretch,
            StyleSize::AnchorSizeFunction(_) | StyleSize::AnchorContainingCalcFunction(_) => {
                unreachable!("anchor-size() should be disabled")
            },
        }
    }
}

impl From<StyleMaxSize> for Size<LengthPercentage> {
    fn from(max_size: StyleMaxSize) -> Self {
        match max_size {
            StyleMaxSize::LengthPercentage(lp) => Size::Numeric(lp.0),
            StyleMaxSize::None => Size::Initial,
            StyleMaxSize::MinContent => Size::MinContent,
            StyleMaxSize::MaxContent => Size::MaxContent,
            StyleMaxSize::FitContent => Size::FitContent,
            StyleMaxSize::FitContentFunction(lp) => Size::FitContentFunction(lp.0),
            StyleMaxSize::Stretch => Size::Stretch,
            StyleMaxSize::AnchorSizeFunction(_) | StyleMaxSize::AnchorContainingCalcFunction(_) => {
                unreachable!("anchor-size() should be disabled")
            },
        }
    }
}

impl Size<LengthPercentage> {
    #[inline]
    pub(crate) fn to_percentage(&self) -> Option<Percentage> {
        self.to_numeric()
            .and_then(|length_percentage| length_percentage.to_percentage())
    }

    /// Resolves percentages in a preferred size, against the provided basis.
    /// If the basis is missing, percentages are considered cyclic.
    /// <https://www.w3.org/TR/css-sizing-3/#preferred-size-properties>
    /// <https://www.w3.org/TR/css-sizing-3/#cyclic-percentage-size>
    #[inline]
    pub(crate) fn resolve_percentages_for_preferred(&self, basis: Option<Au>) -> Size<Au> {
        match self {
            Size::Numeric(numeric) => numeric
                .maybe_to_used_value(basis)
                .map_or(Size::Initial, Size::Numeric),
            Size::FitContentFunction(numeric) => {
                // Under discussion in https://github.com/w3c/csswg-drafts/issues/11805
                numeric
                    .maybe_to_used_value(basis)
                    .map_or(Size::FitContent, Size::FitContentFunction)
            },
            _ => self.map(|_| unreachable!("This shouldn't be called for keywords")),
        }
    }

    /// Resolves percentages in a maximum size, against the provided basis.
    /// If the basis is missing, percentages are considered cyclic.
    /// <https://www.w3.org/TR/css-sizing-3/#preferred-size-properties>
    /// <https://www.w3.org/TR/css-sizing-3/#cyclic-percentage-size>
    #[inline]
    pub(crate) fn resolve_percentages_for_max(&self, basis: Option<Au>) -> Size<Au> {
        match self {
            Size::Numeric(numeric) => numeric
                .maybe_to_used_value(basis)
                .map_or(Size::Initial, Size::Numeric),
            Size::FitContentFunction(numeric) => {
                // Under discussion in https://github.com/w3c/csswg-drafts/issues/11805
                numeric
                    .maybe_to_used_value(basis)
                    .map_or(Size::MaxContent, Size::FitContentFunction)
            },
            _ => self.map(|_| unreachable!("This shouldn't be called for keywords")),
        }
    }
}

impl LogicalVec2<Size<LengthPercentage>> {
    pub(crate) fn percentages_relative_to_basis(
        &self,
        basis: &LogicalVec2<Au>,
    ) -> LogicalVec2<Size<Au>> {
        LogicalVec2 {
            inline: self.inline.map(|value| value.to_used_value(basis.inline)),
            block: self.block.map(|value| value.to_used_value(basis.block)),
        }
    }
}

impl Size<Au> {
    /// Resolves a preferred size into a numerical value.
    /// <https://www.w3.org/TR/css-sizing-3/#preferred-size-properties>
    #[inline]
    pub(crate) fn resolve_for_preferred<F: FnOnce() -> ContentSizes>(
        &self,
        automatic_size: Size<Au>,
        stretch_size: Option<Au>,
        content_size: &LazyCell<ContentSizes, F>,
    ) -> Au {
        match self {
            Self::Initial => {
                assert!(!automatic_size.is_initial());
                automatic_size.resolve_for_preferred(automatic_size, stretch_size, content_size)
            },
            Self::MinContent => content_size.min_content,
            Self::MaxContent => content_size.max_content,
            Self::FitContentFunction(size) => content_size.shrink_to_fit(*size),
            Self::FitContent => {
                content_size.shrink_to_fit(stretch_size.unwrap_or_else(|| content_size.max_content))
            },
            Self::Stretch => stretch_size.unwrap_or_else(|| content_size.max_content),
            Self::Numeric(numeric) => *numeric,
        }
    }

    /// Resolves a minimum size into a numerical value.
    /// <https://www.w3.org/TR/css-sizing-3/#min-size-properties>
    #[inline]
    pub(crate) fn resolve_for_min<F: FnOnce() -> ContentSizes>(
        &self,
        get_automatic_minimum_size: impl FnOnce() -> Au,
        stretch_size: Option<Au>,
        content_size: &LazyCell<ContentSizes, F>,
        is_table: bool,
    ) -> Au {
        let result = match self {
            Self::Initial => get_automatic_minimum_size(),
            Self::MinContent => content_size.min_content,
            Self::MaxContent => content_size.max_content,
            Self::FitContentFunction(size) => content_size.shrink_to_fit(*size),
            Self::FitContent => content_size.shrink_to_fit(stretch_size.unwrap_or_default()),
            Self::Stretch => stretch_size.unwrap_or_default(),
            Self::Numeric(numeric) => *numeric,
        };
        if is_table {
            // In addition to the specified minimum, the inline size of a table is forced to be
            // at least as big as its min-content size.
            //
            // Note that if there are collapsed columns, only the inline size of the table grid will
            // shrink, while the size of the table wrapper (being computed here) won't be affected.
            // However, collapsed rows should typically affect the block size of the table wrapper,
            // so it might be wrong to use this function for that case.
            // This is being discussed in https://github.com/w3c/csswg-drafts/issues/11408
            result.max(content_size.min_content)
        } else {
            result
        }
    }

    /// Resolves a maximum size into a numerical value.
    /// <https://www.w3.org/TR/css-sizing-3/#max-size-properties>
    #[inline]
    pub(crate) fn resolve_for_max<F: FnOnce() -> ContentSizes>(
        &self,
        stretch_size: Option<Au>,
        content_size: &LazyCell<ContentSizes, F>,
    ) -> Option<Au> {
        Some(match self {
            Self::Initial => return None,
            Self::MinContent => content_size.min_content,
            Self::MaxContent => content_size.max_content,
            Self::FitContentFunction(size) => content_size.shrink_to_fit(*size),
            Self::FitContent => content_size.shrink_to_fit(stretch_size.unwrap_or(MAX_AU)),
            Self::Stretch => return stretch_size,
            Self::Numeric(numeric) => *numeric,
        })
    }

    /// Tries to resolve an extrinsic size into a numerical value.
    /// Extrinsic sizes are those based on the context of an element, without regard for its contents.
    /// <https://drafts.csswg.org/css-sizing-3/#extrinsic>
    ///
    /// Returns `None` if either:
    /// - The size is intrinsic.
    /// - The size is the initial one.
    ///   TODO: should we allow it to behave as `stretch` instead of assuming it's intrinsic?
    /// - The provided `stretch_size` is `None` but we need its value.
    #[inline]
    pub(crate) fn maybe_resolve_extrinsic(&self, stretch_size: Option<Au>) -> Option<Au> {
        match self {
            Self::Initial |
            Self::MinContent |
            Self::MaxContent |
            Self::FitContent |
            Self::FitContentFunction(_) => None,
            Self::Stretch => stretch_size,
            Self::Numeric(numeric) => Some(*numeric),
        }
    }
}

/// Represents the sizing constraint that the preferred, min and max sizing properties
/// impose on one axis.
#[derive(Clone, Copy, Debug, MallocSizeOf, PartialEq)]
pub(crate) enum SizeConstraint {
    /// Represents a definite preferred size, clamped by minimum and maximum sizes (if any).
    Definite(Au),
    /// Represents an indefinite preferred size that allows a range of values between
    /// the first argument (minimum size) and the second one (maximum size).
    MinMax(Au, Option<Au>),
}

impl Default for SizeConstraint {
    #[inline]
    fn default() -> Self {
        Self::MinMax(Au::default(), None)
    }
}

impl SizeConstraint {
    #[inline]
    pub(crate) fn new(preferred_size: Option<Au>, min_size: Au, max_size: Option<Au>) -> Self {
        preferred_size.map_or_else(
            || Self::MinMax(min_size, max_size),
            |size| Self::Definite(size.clamp_between_extremums(min_size, max_size)),
        )
    }

    #[inline]
    pub(crate) fn is_definite(self) -> bool {
        matches!(self, Self::Definite(_))
    }

    #[inline]
    pub(crate) fn to_definite(self) -> Option<Au> {
        match self {
            Self::Definite(size) => Some(size),
            _ => None,
        }
    }
}

impl From<Option<Au>> for SizeConstraint {
    fn from(size: Option<Au>) -> Self {
        size.map(SizeConstraint::Definite).unwrap_or_default()
    }
}

#[derive(Clone, Debug, Default)]
pub(crate) struct Sizes {
    /// <https://drafts.csswg.org/css-sizing-3/#preferred-size-properties>
    pub preferred: Size<Au>,
    /// <https://drafts.csswg.org/css-sizing-3/#min-size-properties>
    pub min: Size<Au>,
    /// <https://drafts.csswg.org/css-sizing-3/#max-size-properties>
    pub max: Size<Au>,
}

impl Sizes {
    #[inline]
    pub(crate) fn new(preferred: Size<Au>, min: Size<Au>, max: Size<Au>) -> Self {
        Self {
            preferred,
            min,
            max,
        }
    }

    /// Resolves the three sizes into a single numerical value.
    #[inline]
    pub(crate) fn resolve(
        &self,
        axis: Direction,
        automatic_size: Size<Au>,
        get_automatic_minimum_size: impl FnOnce() -> Au,
        stretch_size: Option<Au>,
        get_content_size: impl FnOnce() -> ContentSizes,
        is_table: bool,
    ) -> Au {
        if is_table && axis == Direction::Block {
            // The intrinsic block size of a table already takes sizing properties into account,
            // but it can be a smaller amount if there are collapsed rows.
            // Therefore, disregard sizing properties and just defer to the intrinsic size.
            // This is being discussed in https://github.com/w3c/csswg-drafts/issues/11408
            return get_content_size().max_content;
        }
        let (preferred, min, max) = self.resolve_each(
            automatic_size,
            get_automatic_minimum_size,
            stretch_size,
            get_content_size,
            is_table,
        );
        preferred.clamp_between_extremums(min, max)
    }

    /// Resolves each of the three sizes into a numerical value, separately.
    /// - The 1st returned value is the resolved preferred size.
    /// - The 2nd returned value is the resolved minimum size.
    /// - The 3rd returned value is the resolved maximum size. `None` means no maximum.
    #[inline]
    pub(crate) fn resolve_each(
        &self,
        automatic_size: Size<Au>,
        get_automatic_minimum_size: impl FnOnce() -> Au,
        stretch_size: Option<Au>,
        get_content_size: impl FnOnce() -> ContentSizes,
        is_table: bool,
    ) -> (Au, Au, Option<Au>) {
        // The provided `get_content_size` is a FnOnce but we may need its result multiple times.
        // A LazyCell will only invoke it once if needed, and then reuse the result.
        let content_size = LazyCell::new(get_content_size);
        (
            self.preferred
                .resolve_for_preferred(automatic_size, stretch_size, &content_size),
            self.min.resolve_for_min(
                get_automatic_minimum_size,
                stretch_size,
                &content_size,
                is_table,
            ),
            self.max.resolve_for_max(stretch_size, &content_size),
        )
    }

    /// Tries to extrinsically resolve the three sizes into a single [`SizeConstraint`].
    /// Values that are intrinsic or need `stretch_size` when it's `None` are handled as such:
    /// - On the preferred size, they make the returned value be an indefinite [`SizeConstraint::MinMax`].
    /// - On the min size, they are treated as `auto`, enforcing the automatic minimum size.
    /// - On the max size, they are treated as `none`, enforcing no maximum.
    #[inline]
    pub(crate) fn resolve_extrinsic(
        &self,
        automatic_size: Size<Au>,
        automatic_minimum_size: Au,
        stretch_size: Option<Au>,
    ) -> SizeConstraint {
        let (preferred, min, max) =
            self.resolve_each_extrinsic(automatic_size, automatic_minimum_size, stretch_size);
        SizeConstraint::new(preferred, min, max)
    }

    /// Tries to extrinsically resolve each of the three sizes into a numerical value, separately.
    /// This can't resolve values that are intrinsic or need `stretch_size` but it's `None`.
    /// - The 1st returned value is the resolved preferred size. If it can't be resolved then
    ///   the returned value is `None`. Note that this is different than treating it as `auto`.
    ///   TODO: This needs to be discussed in <https://github.com/w3c/csswg-drafts/issues/11387>.
    /// - The 2nd returned value is the resolved minimum size. If it can't be resolved then we
    ///   treat it as the initial `auto`, returning the automatic minimum size.
    /// - The 3rd returned value is the resolved maximum size. If it can't be resolved then we
    ///   treat it as the initial `none`, returning `None`.
    #[inline]
    pub(crate) fn resolve_each_extrinsic(
        &self,
        automatic_size: Size<Au>,
        automatic_minimum_size: Au,
        stretch_size: Option<Au>,
    ) -> (Option<Au>, Au, Option<Au>) {
        (
            if self.preferred.is_initial() {
                automatic_size.maybe_resolve_extrinsic(stretch_size)
            } else {
                self.preferred.maybe_resolve_extrinsic(stretch_size)
            },
            self.min
                .maybe_resolve_extrinsic(stretch_size)
                .unwrap_or(automatic_minimum_size),
            self.max.maybe_resolve_extrinsic(stretch_size),
        )
    }
}

struct LazySizeData<'a> {
    sizes: &'a Sizes,
    axis: Direction,
    automatic_size: Size<Au>,
    get_automatic_minimum_size: fn() -> Au,
    stretch_size: Option<Au>,
    is_table: bool,
}

/// Represents a size that can't be fully resolved until the intrinsic size
/// is known. This is useful in the block axis, since the intrinsic size
/// depends on layout, but the other inputs are known beforehand.
pub(crate) struct LazySize<'a> {
    result: OnceCell<Au>,
    data: Option<LazySizeData<'a>>,
}

impl<'a> LazySize<'a> {
    pub(crate) fn new(
        sizes: &'a Sizes,
        axis: Direction,
        automatic_size: Size<Au>,
        get_automatic_minimum_size: fn() -> Au,
        stretch_size: Option<Au>,
        is_table: bool,
    ) -> Self {
        Self {
            result: OnceCell::new(),
            data: Some(LazySizeData {
                sizes,
                axis,
                automatic_size,
                get_automatic_minimum_size,
                stretch_size,
                is_table,
            }),
        }
    }

    /// Creates a [`LazySize`] that will resolve to the intrinsic size.
    /// Should be equivalent to [`LazySize::new()`] with default parameters,
    /// but avoiding the trouble of getting a reference to a [`Sizes::default()`]
    /// which lives long enough.
    ///
    /// TODO: It's not clear what this should do if/when [`LazySize::resolve()`]
    /// is changed to accept a [`ContentSizes`] as the intrinsic size.
    pub(crate) fn intrinsic() -> Self {
        Self {
            result: OnceCell::new(),
            data: None,
        }
    }

    /// Resolves the [`LazySize`] into [`Au`], caching the result.
    /// The argument is a callback that computes the intrinsic size lazily.
    ///
    /// TODO: The intrinsic size should probably be a [`ContentSizes`] instead of [`Au`].
    pub(crate) fn resolve(&self, get_content_size: impl FnOnce() -> Au) -> Au {
        *self.result.get_or_init(|| {
            let Some(ref data) = self.data else {
                return get_content_size();
            };
            data.sizes.resolve(
                data.axis,
                data.automatic_size,
                data.get_automatic_minimum_size,
                data.stretch_size,
                || get_content_size().into(),
                data.is_table,
            )
        })
    }
}

impl From<Au> for LazySize<'_> {
    /// Creates a [`LazySize`] that will resolve to the given [`Au`],
    /// ignoring the intrinsic size.
    fn from(value: Au) -> Self {
        let result = OnceCell::new();
        result.set(value).unwrap();
        LazySize { result, data: None }
    }
}
