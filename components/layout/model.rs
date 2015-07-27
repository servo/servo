/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

//! Borders, padding, and margins.

#![deny(unsafe_code)]

use fragment::Fragment;

use euclid::{Matrix4, SideOffsets2D};
use std::cmp::{max, min};
use std::fmt;
use style::computed_values::transform::ComputedMatrix;
use style::properties::ComputedValues;
use style::values::computed::LengthOrPercentageOrAuto;
use style::values::computed::{LengthOrPercentageOrNone, LengthOrPercentage};
use util::geometry::Au;
use util::logical_geometry::LogicalMargin;

/// A collapsible margin. See CSS 2.1 § 8.3.1.
#[derive(Copy, Clone)]
pub struct AdjoiningMargins {
    /// The value of the greatest positive margin.
    pub most_positive: Au,

    /// The actual value (not the absolute value) of the negative margin with the largest absolute
    /// value. Since this is not the absolute value, this is always zero or negative.
    pub most_negative: Au,
}

impl AdjoiningMargins {
    pub fn new() -> AdjoiningMargins {
        AdjoiningMargins {
            most_positive: Au(0),
            most_negative: Au(0),
        }
    }

    pub fn from_margin(margin_value: Au) -> AdjoiningMargins {
        if margin_value >= Au(0) {
            AdjoiningMargins {
                most_positive: margin_value,
                most_negative: Au(0),
            }
        } else {
            AdjoiningMargins {
                most_positive: Au(0),
                most_negative: margin_value,
            }
        }
    }

    pub fn union(&mut self, other: AdjoiningMargins) {
        self.most_positive = max(self.most_positive, other.most_positive);
        self.most_negative = min(self.most_negative, other.most_negative)
    }

    pub fn collapse(&self) -> Au {
        self.most_positive + self.most_negative
    }
}

/// Represents the block-start and block-end margins of a flow with collapsible margins. See CSS 2.1 § 8.3.1.
#[derive(Copy, Clone)]
pub enum CollapsibleMargins {
    /// Margins may not collapse with this flow.
    None(Au, Au),

    /// Both the block-start and block-end margins (specified here in that order) may collapse, but the
    /// margins do not collapse through this flow.
    Collapse(AdjoiningMargins, AdjoiningMargins),

    /// Margins collapse *through* this flow. This means, essentially, that the flow doesn’t
    /// have any border, padding, or out-of-flow (floating or positioned) content
    CollapseThrough(AdjoiningMargins),
}

impl CollapsibleMargins {
    pub fn new() -> CollapsibleMargins {
        CollapsibleMargins::None(Au(0), Au(0))
    }

    /// Returns the amount of margin that should be applied in a noncollapsible context. This is
    /// currently used to apply block-start margin for hypothetical boxes, since we do not collapse
    /// margins of hypothetical boxes.
    pub fn block_start_margin_for_noncollapsible_context(&self) -> Au {
        match *self {
            CollapsibleMargins::None(block_start, _) => block_start,
            CollapsibleMargins::Collapse(ref block_start, _) |
            CollapsibleMargins::CollapseThrough(ref block_start) => block_start.collapse(),
        }
    }
}

enum FinalMarginState {
    MarginsCollapseThrough,
    BottomMarginCollapses,
}

pub struct MarginCollapseInfo {
    pub state: MarginCollapseState,
    pub block_start_margin: AdjoiningMargins,
    pub margin_in: AdjoiningMargins,
}

impl MarginCollapseInfo {
    /// TODO(#2012, pcwalton): Remove this method once `fragment` is not an `Option`.
    pub fn new() -> MarginCollapseInfo {
        MarginCollapseInfo {
            state: MarginCollapseState::AccumulatingCollapsibleTopMargin,
            block_start_margin: AdjoiningMargins::new(),
            margin_in: AdjoiningMargins::new(),
        }
    }

    pub fn initialize_block_start_margin(&mut self,
                                 fragment: &Fragment,
                                 can_collapse_block_start_margin_with_kids: bool) {
        if !can_collapse_block_start_margin_with_kids {
            self.state = MarginCollapseState::AccumulatingMarginIn
        }

        self.block_start_margin = AdjoiningMargins::from_margin(fragment.margin.block_start)
    }

    pub fn finish_and_compute_collapsible_margins(mut self,
                                                  fragment: &Fragment,
                                                  can_collapse_block_end_margin_with_kids: bool)
                                                  -> (CollapsibleMargins, Au) {
        let state = match self.state {
            MarginCollapseState::AccumulatingCollapsibleTopMargin => {
                match fragment.style().content_block_size() {
                    LengthOrPercentageOrAuto::Auto | LengthOrPercentageOrAuto::Length(Au(0)) |
                    LengthOrPercentageOrAuto::Percentage(0.) => {
                        match fragment.style().min_block_size() {
                            LengthOrPercentage::Length(Au(0)) | LengthOrPercentage::Percentage(0.) => {
                                FinalMarginState::MarginsCollapseThrough
                            },
                            _ => {
                                // If the fragment has non-zero min-block-size, margins may not
                                // collapse through it.
                                FinalMarginState::BottomMarginCollapses
                            }
                        }
                    },
                    _ => {
                        // If the fragment has an explicitly specified block-size, margins may not
                        // collapse through it.
                        FinalMarginState::BottomMarginCollapses
                    }
                }
            }
            MarginCollapseState::AccumulatingMarginIn => FinalMarginState::BottomMarginCollapses,
        };

        // Different logic is needed here depending on whether this flow can collapse its block-end
        // margin with its children.
        let block_end_margin = fragment.margin.block_end;
        if !can_collapse_block_end_margin_with_kids {
            match state {
                FinalMarginState::MarginsCollapseThrough => {
                    let advance = self.block_start_margin.collapse();
                    self.margin_in.union(AdjoiningMargins::from_margin(block_end_margin));
                    (CollapsibleMargins::Collapse(self.block_start_margin, self.margin_in), advance)
                }
                FinalMarginState::BottomMarginCollapses => {
                    let advance = self.margin_in.collapse();
                    self.margin_in.union(AdjoiningMargins::from_margin(block_end_margin));
                    (CollapsibleMargins::Collapse(self.block_start_margin, self.margin_in), advance)
                }
            }
        } else {
            match state {
                FinalMarginState::MarginsCollapseThrough => {
                    self.block_start_margin.union(AdjoiningMargins::from_margin(block_end_margin));
                    (CollapsibleMargins::CollapseThrough(self.block_start_margin), Au(0))
                }
                FinalMarginState::BottomMarginCollapses => {
                    self.margin_in.union(AdjoiningMargins::from_margin(block_end_margin));
                    (CollapsibleMargins::Collapse(self.block_start_margin, self.margin_in), Au(0))
                }
            }
        }
    }

    pub fn current_float_ceiling(&mut self) -> Au {
        match self.state {
            MarginCollapseState::AccumulatingCollapsibleTopMargin => {
                // We do not include the top margin in the float ceiling, because the float flow
                // needs to be positioned relative to our *border box*, not our margin box. See
                // `tests/ref/float_under_top_margin_a.html`.
                Au(0)
            }
            MarginCollapseState::AccumulatingMarginIn => self.margin_in.collapse(),
        }
    }

    /// Adds the child's potentially collapsible block-start margin to the current margin state and
    /// advances the Y offset by the appropriate amount to handle that margin. Returns the amount
    /// that should be added to the Y offset during block layout.
    pub fn advance_block_start_margin(&mut self, child_collapsible_margins: &CollapsibleMargins)
                                      -> Au {
        match (self.state, *child_collapsible_margins) {
            (MarginCollapseState::AccumulatingCollapsibleTopMargin,
             CollapsibleMargins::None(block_start, _)) => {
                self.state = MarginCollapseState::AccumulatingMarginIn;
                block_start
            }
            (MarginCollapseState::AccumulatingCollapsibleTopMargin,
             CollapsibleMargins::Collapse(block_start, _)) => {
                self.block_start_margin.union(block_start);
                self.state = MarginCollapseState::AccumulatingMarginIn;
                Au(0)
            }
            (MarginCollapseState::AccumulatingMarginIn,
             CollapsibleMargins::None(block_start, _)) => {
                let previous_margin_value = self.margin_in.collapse();
                self.margin_in = AdjoiningMargins::new();
                previous_margin_value + block_start
            }
            (MarginCollapseState::AccumulatingMarginIn, CollapsibleMargins::Collapse(block_start, _)) => {
                self.margin_in.union(block_start);
                let margin_value = self.margin_in.collapse();
                self.margin_in = AdjoiningMargins::new();
                margin_value
            }
            (_, CollapsibleMargins::CollapseThrough(_)) => {
                // For now, we ignore this; this will be handled by `advance_block-end_margin` below.
                Au(0)
            }
        }
    }

    /// Adds the child's potentially collapsible block-end margin to the current margin state and
    /// advances the Y offset by the appropriate amount to handle that margin. Returns the amount
    /// that should be added to the Y offset during block layout.
    pub fn advance_block_end_margin(&mut self, child_collapsible_margins: &CollapsibleMargins) -> Au {
        match (self.state, *child_collapsible_margins) {
            (MarginCollapseState::AccumulatingCollapsibleTopMargin, CollapsibleMargins::None(..)) |
            (MarginCollapseState::AccumulatingCollapsibleTopMargin, CollapsibleMargins::Collapse(..)) => {
                // Can't happen because the state will have been replaced with
                // `MarginCollapseState::AccumulatingMarginIn` above.
                panic!("should not be accumulating collapsible block_start margins anymore!")
            }
            (MarginCollapseState::AccumulatingCollapsibleTopMargin, CollapsibleMargins::CollapseThrough(margin)) => {
                self.block_start_margin.union(margin);
                Au(0)
            }
            (MarginCollapseState::AccumulatingMarginIn, CollapsibleMargins::None(_, block_end)) => {
                assert_eq!(self.margin_in.most_positive, Au(0));
                assert_eq!(self.margin_in.most_negative, Au(0));
                block_end
            }
            (MarginCollapseState::AccumulatingMarginIn, CollapsibleMargins::Collapse(_, block_end)) |
            (MarginCollapseState::AccumulatingMarginIn, CollapsibleMargins::CollapseThrough(block_end)) => {
                self.margin_in.union(block_end);
                Au(0)
            }
        }
    }
}

#[derive(Copy, Clone)]
pub enum MarginCollapseState {
    AccumulatingCollapsibleTopMargin,
    AccumulatingMarginIn,
}

/// Intrinsic inline-sizes, which consist of minimum and preferred.
#[derive(RustcEncodable)]
pub struct IntrinsicISizes {
    /// The *minimum inline-size* of the content.
    pub minimum_inline_size: Au,
    /// The *preferred inline-size* of the content.
    pub preferred_inline_size: Au,
}

impl fmt::Debug for IntrinsicISizes {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "min={:?}, pref={:?}", self.minimum_inline_size, self.preferred_inline_size)
    }
}

impl IntrinsicISizes {
    pub fn new() -> IntrinsicISizes {
        IntrinsicISizes {
            minimum_inline_size: Au(0),
            preferred_inline_size: Au(0),
        }
    }
}

/// The temporary result of the computation of intrinsic inline-sizes.
pub struct IntrinsicISizesContribution {
    /// Intrinsic sizes for the content only (not counting borders, padding, or margins).
    pub content_intrinsic_sizes: IntrinsicISizes,
    /// The inline size of borders and padding, as well as margins if appropriate.
    pub surrounding_size: Au,
}

impl IntrinsicISizesContribution {
    /// Creates and initializes an inline size computation with all sizes set to zero.
    pub fn new() -> IntrinsicISizesContribution {
        IntrinsicISizesContribution {
            content_intrinsic_sizes: IntrinsicISizes::new(),
            surrounding_size: Au(0),
        }
    }

    /// Adds the content intrinsic sizes and the surrounding size together to yield the final
    /// intrinsic size computation.
    pub fn finish(self) -> IntrinsicISizes {
        IntrinsicISizes {
            minimum_inline_size: self.content_intrinsic_sizes.minimum_inline_size +
                                     self.surrounding_size,
            preferred_inline_size: self.content_intrinsic_sizes.preferred_inline_size +
                                     self.surrounding_size,
        }
    }

    /// Updates the computation so that the minimum is the maximum of the current minimum and the
    /// given minimum and the preferred is the sum of the current preferred and the given
    /// preferred. This is used when laying out fragments in the inline direction.
    ///
    /// FIXME(pcwalton): This is incorrect when the inline fragment contains forced line breaks
    /// (e.g. `<br>` or `white-space: pre`).
    pub fn union_inline(&mut self, sizes: &IntrinsicISizes) {
        self.content_intrinsic_sizes.minimum_inline_size =
            max(self.content_intrinsic_sizes.minimum_inline_size, sizes.minimum_inline_size);
        self.content_intrinsic_sizes.preferred_inline_size =
            self.content_intrinsic_sizes.preferred_inline_size + sizes.preferred_inline_size
    }

    /// Updates the computation so that the minimum is the sum of the current minimum and the
    /// given minimum and the preferred is the sum of the current preferred and the given
    /// preferred. This is used when laying out fragments in the inline direction when
    /// `white-space` is `pre` or `nowrap`.
    pub fn union_nonbreaking_inline(&mut self, sizes: &IntrinsicISizes) {
        self.content_intrinsic_sizes.minimum_inline_size =
            self.content_intrinsic_sizes.minimum_inline_size + sizes.minimum_inline_size;
        self.content_intrinsic_sizes.preferred_inline_size =
            self.content_intrinsic_sizes.preferred_inline_size + sizes.preferred_inline_size
    }

    /// Updates the computation so that the minimum is the maximum of the current minimum and the
    /// given minimum and the preferred is the maximum of the current preferred and the given
    /// preferred. This can be useful when laying out fragments in the block direction (but note
    /// that it does not take floats into account, so `BlockFlow` does not use it).
    ///
    /// This is used when contributing the intrinsic sizes for individual fragments.
    pub fn union_block(&mut self, sizes: &IntrinsicISizes) {
        self.content_intrinsic_sizes.minimum_inline_size =
            max(self.content_intrinsic_sizes.minimum_inline_size, sizes.minimum_inline_size);
        self.content_intrinsic_sizes.preferred_inline_size =
            max(self.content_intrinsic_sizes.preferred_inline_size, sizes.preferred_inline_size)
    }
}

/// Useful helper data type when computing values for blocks and positioned elements.
#[derive(Copy, Clone, PartialEq, Debug)]
pub enum MaybeAuto {
    Auto,
    Specified(Au),
}

impl MaybeAuto {
    #[inline]
    pub fn from_style(length: LengthOrPercentageOrAuto, containing_length: Au)
                      -> MaybeAuto {
        match length {
            LengthOrPercentageOrAuto::Auto => MaybeAuto::Auto,
            LengthOrPercentageOrAuto::Percentage(percent) => {
                MaybeAuto::Specified(containing_length.scale_by(percent))
            }
            LengthOrPercentageOrAuto::Length(length) => MaybeAuto::Specified(length)
        }
    }

    #[inline]
    pub fn specified_or_default(&self, default: Au) -> Au {
        match *self {
            MaybeAuto::Auto => default,
            MaybeAuto::Specified(value) => value,
        }
    }

    #[inline]
    pub fn specified_or_zero(&self) -> Au {
        self.specified_or_default(Au::new(0))
    }

    #[inline]
    pub fn map<F>(&self, mapper: F) -> MaybeAuto where F: FnOnce(Au) -> Au {
        match *self {
            MaybeAuto::Auto => MaybeAuto::Auto,
            MaybeAuto::Specified(value) => MaybeAuto::Specified(mapper(value)),
        }
    }
}

pub fn specified_or_none(length: LengthOrPercentageOrNone, containing_length: Au) -> Option<Au> {
    match length {
        LengthOrPercentageOrNone::None => None,
        LengthOrPercentageOrNone::Percentage(percent) => Some(containing_length.scale_by(percent)),
        LengthOrPercentageOrNone::Length(length) => Some(length),
    }
}

pub fn specified(length: LengthOrPercentage, containing_length: Au) -> Au {
    match length {
        LengthOrPercentage::Length(length) => length,
        LengthOrPercentage::Percentage(p) => containing_length.scale_by(p)
    }
}

#[inline]
pub fn padding_from_style(style: &ComputedValues, containing_block_inline_size: Au)
                          -> LogicalMargin<Au> {
    let padding_style = style.get_padding();
    LogicalMargin::from_physical(style.writing_mode, SideOffsets2D::new(
        specified(padding_style.padding_top, containing_block_inline_size),
        specified(padding_style.padding_right, containing_block_inline_size),
        specified(padding_style.padding_bottom, containing_block_inline_size),
        specified(padding_style.padding_left, containing_block_inline_size)))
}

pub trait ToGfxMatrix {
    fn to_gfx_matrix(&self) -> Matrix4;
}

impl ToGfxMatrix for ComputedMatrix {
    fn to_gfx_matrix(&self) -> Matrix4 {
        Matrix4 {
            m11: self.m11 as f32, m12: self.m12 as f32, m13: self.m13 as f32, m14: self.m14 as f32,
            m21: self.m21 as f32, m22: self.m22 as f32, m23: self.m23 as f32, m24: self.m24 as f32,
            m31: self.m31 as f32, m32: self.m32 as f32, m33: self.m33 as f32, m34: self.m34 as f32,
            m41: self.m41 as f32, m42: self.m42 as f32, m43: self.m43 as f32, m44: self.m44 as f32
        }
    }
}
