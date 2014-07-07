/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

//! Borders, padding, and margins.

#![deny(unsafe_block)]

use fragment::Fragment;

use computed = style::computed_values;
use geom::SideOffsets2D;
use style::computed_values::{LPA_Auto, LPA_Length, LPA_Percentage, LP_Length, LP_Percentage};
use style::ComputedValues;
use servo_util::geometry::Au;
use servo_util::geometry;
use servo_util::logical_geometry::LogicalMargin;
use std::fmt;

/// A collapsible margin. See CSS 2.1 § 8.3.1.
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
        self.most_positive = geometry::max(self.most_positive, other.most_positive);
        self.most_negative = geometry::min(self.most_negative, other.most_negative)
    }

    pub fn collapse(&self) -> Au {
        self.most_positive + self.most_negative
    }
}

/// Represents the bstart and bend margins of a flow with collapsible margins. See CSS 2.1 § 8.3.1.
pub enum CollapsibleMargins {
    /// Margins may not collapse with this flow.
    NoCollapsibleMargins(Au, Au),

    /// Both the bstart and bend margins (specified here in that order) may collapse, but the
    /// margins do not collapse through this flow.
    MarginsCollapse(AdjoiningMargins, AdjoiningMargins),

    /// Margins collapse *through* this flow. This means, essentially, that the flow doesn’t
    /// have any border, padding, or out-of-flow (floating or positioned) content
    MarginsCollapseThrough(AdjoiningMargins),
}

impl CollapsibleMargins {
    pub fn new() -> CollapsibleMargins {
        NoCollapsibleMargins(Au(0), Au(0))
    }
}

enum FinalMarginState {
    MarginsCollapseThroughFinalMarginState,
    BottomMarginCollapsesFinalMarginState,
}

pub struct MarginCollapseInfo {
    pub state: MarginCollapseState,
    pub bstart_margin: AdjoiningMargins,
    pub margin_in: AdjoiningMargins,
}

impl MarginCollapseInfo {
    /// TODO(#2012, pcwalton): Remove this method once `fragment` is not an `Option`.
    pub fn new() -> MarginCollapseInfo {
        MarginCollapseInfo {
            state: AccumulatingCollapsibleTopMargin,
            bstart_margin: AdjoiningMargins::new(),
            margin_in: AdjoiningMargins::new(),
        }
    }

    pub fn initialize_bstart_margin(&mut self,
                                 fragment: &Fragment,
                                 can_collapse_bstart_margin_with_kids: bool) {
        if !can_collapse_bstart_margin_with_kids {
            self.state = AccumulatingMarginIn
        }

        self.bstart_margin = AdjoiningMargins::from_margin(fragment.margin.bstart)
    }

    pub fn finish_and_compute_collapsible_margins(mut self,
                                                  fragment: &Fragment,
                                                  can_collapse_bend_margin_with_kids: bool)
                                                  -> (CollapsibleMargins, Au) {
        let state = match self.state {
            AccumulatingCollapsibleTopMargin => {
                match fragment.style().content_bsize() {
                    LPA_Auto | LPA_Length(Au(0)) | LPA_Percentage(0.) => {
                        match fragment.style().min_bsize() {
                            LP_Length(Au(0)) | LP_Percentage(0.) => {
                                MarginsCollapseThroughFinalMarginState
                            },
                            _ => {
                                // If the fragment has non-zero min-bsize, margins may not
                                // collapse through it.
                                BottomMarginCollapsesFinalMarginState
                            }
                        }
                    },
                    _ => {
                        // If the fragment has an explicitly specified bsize, margins may not
                        // collapse through it.
                        BottomMarginCollapsesFinalMarginState
                    }
                }
            }
            AccumulatingMarginIn => BottomMarginCollapsesFinalMarginState,
        };

        // Different logic is needed here depending on whether this flow can collapse its bend
        // margin with its children.
        let bend_margin = fragment.margin.bend;
        if !can_collapse_bend_margin_with_kids {
            match state {
                MarginsCollapseThroughFinalMarginState => {
                    let advance = self.bstart_margin.collapse();
                    self.margin_in.union(AdjoiningMargins::from_margin(bend_margin));
                    (MarginsCollapse(self.bstart_margin, self.margin_in), advance)
                }
                BottomMarginCollapsesFinalMarginState => {
                    let advance = self.margin_in.collapse();
                    self.margin_in.union(AdjoiningMargins::from_margin(bend_margin));
                    (MarginsCollapse(self.bstart_margin, self.margin_in), advance)
                }
            }
        } else {
            match state {
                MarginsCollapseThroughFinalMarginState => {
                    self.bstart_margin.union(AdjoiningMargins::from_margin(bend_margin));
                    (MarginsCollapseThrough(self.bstart_margin), Au(0))
                }
                BottomMarginCollapsesFinalMarginState => {
                    self.margin_in.union(AdjoiningMargins::from_margin(bend_margin));
                    (MarginsCollapse(self.bstart_margin, self.margin_in), Au(0))
                }
            }
        }
    }

    pub fn current_float_ceiling(&mut self) -> Au {
        match self.state {
            AccumulatingCollapsibleTopMargin => self.bstart_margin.collapse(),
            AccumulatingMarginIn => self.margin_in.collapse(),
        }
    }

    /// Adds the child's potentially collapsible bstart margin to the current margin state and
    /// advances the Y offset by the appropriate amount to handle that margin. Returns the amount
    /// that should be added to the Y offset during block layout.
    pub fn advance_bstart_margin(&mut self, child_collapsible_margins: &CollapsibleMargins) -> Au {
        match (self.state, *child_collapsible_margins) {
            (AccumulatingCollapsibleTopMargin, NoCollapsibleMargins(bstart, _)) => {
                self.state = AccumulatingMarginIn;
                bstart
            }
            (AccumulatingCollapsibleTopMargin, MarginsCollapse(bstart, _)) => {
                self.bstart_margin.union(bstart);
                self.state = AccumulatingMarginIn;
                Au(0)
            }
            (AccumulatingMarginIn, NoCollapsibleMargins(bstart, _)) => {
                let previous_margin_value = self.margin_in.collapse();
                self.margin_in = AdjoiningMargins::new();
                previous_margin_value + bstart
            }
            (AccumulatingMarginIn, MarginsCollapse(bstart, _)) => {
                self.margin_in.union(bstart);
                let margin_value = self.margin_in.collapse();
                self.margin_in = AdjoiningMargins::new();
                margin_value
            }
            (_, MarginsCollapseThrough(_)) => {
                // For now, we ignore this; this will be handled by `advance_bend_margin` below.
                Au(0)
            }
        }
    }

    /// Adds the child's potentially collapsible bend margin to the current margin state and
    /// advances the Y offset by the appropriate amount to handle that margin. Returns the amount
    /// that should be added to the Y offset during block layout.
    pub fn advance_bend_margin(&mut self, child_collapsible_margins: &CollapsibleMargins) -> Au {
        match (self.state, *child_collapsible_margins) {
            (AccumulatingCollapsibleTopMargin, NoCollapsibleMargins(..)) |
            (AccumulatingCollapsibleTopMargin, MarginsCollapse(..)) => {
                // Can't happen because the state will have been replaced with
                // `AccumulatingMarginIn` above.
                fail!("should not be accumulating collapsible bstart margins anymore!")
            }
            (AccumulatingCollapsibleTopMargin, MarginsCollapseThrough(margin)) => {
                self.bstart_margin.union(margin);
                Au(0)
            }
            (AccumulatingMarginIn, NoCollapsibleMargins(_, bend)) => {
                assert_eq!(self.margin_in.most_positive, Au(0));
                assert_eq!(self.margin_in.most_negative, Au(0));
                bend
            }
            (AccumulatingMarginIn, MarginsCollapse(_, bend)) |
            (AccumulatingMarginIn, MarginsCollapseThrough(bend)) => {
                self.margin_in.union(bend);
                Au(0)
            }
        }
    }
}

pub enum MarginCollapseState {
    AccumulatingCollapsibleTopMargin,
    AccumulatingMarginIn,
}

/// Intrinsic isizes, which consist of minimum and preferred.
pub struct IntrinsicISizes {
    /// The *minimum isize* of the content.
    pub minimum_isize: Au,
    /// The *preferred isize* of the content.
    pub preferred_isize: Au,
    /// The estimated sum of borders, padding, and margins. Some calculations use this information
    /// when computing intrinsic isizes.
    pub surround_isize: Au,
}

impl fmt::Show for IntrinsicISizes {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "min={}, pref={}, surr={}", self.minimum_isize, self.preferred_isize, self.surround_isize)
    }
}

impl IntrinsicISizes {
    pub fn new() -> IntrinsicISizes {
        IntrinsicISizes {
            minimum_isize: Au(0),
            preferred_isize: Au(0),
            surround_isize: Au(0),
        }
    }

    pub fn total_minimum_isize(&self) -> Au {
        self.minimum_isize + self.surround_isize
    }

    pub fn total_preferred_isize(&self) -> Au {
        self.preferred_isize + self.surround_isize
    }
}

/// Useful helper data type when computing values for blocks and positioned elements.
pub enum MaybeAuto {
    Auto,
    Specified(Au),
}

impl MaybeAuto {
    #[inline]
    pub fn from_style(length: computed::LengthOrPercentageOrAuto, containing_length: Au)
                      -> MaybeAuto {
        match length {
            computed::LPA_Auto => Auto,
            computed::LPA_Percentage(percent) => Specified(containing_length.scale_by(percent)),
            computed::LPA_Length(length) => Specified(length)
        }
    }

    #[inline]
    pub fn specified_or_default(&self, default: Au) -> Au {
        match *self {
            Auto => default,
            Specified(value) => value,
        }
    }

    #[inline]
    pub fn specified_or_zero(&self) -> Au {
        self.specified_or_default(Au::new(0))
    }
}

pub fn specified_or_none(length: computed::LengthOrPercentageOrNone, containing_length: Au) -> Option<Au> {
    match length {
        computed::LPN_None => None,
        computed::LPN_Percentage(percent) => Some(containing_length.scale_by(percent)),
        computed::LPN_Length(length) => Some(length),
    }
}

pub fn specified(length: computed::LengthOrPercentage, containing_length: Au) -> Au {
    match length {
        computed::LP_Length(length) => length,
        computed::LP_Percentage(p) => containing_length.scale_by(p)
    }
}

#[inline]
pub fn padding_from_style(style: &ComputedValues, containing_block_isize: Au)
                          -> LogicalMargin<Au> {
    let padding_style = style.get_padding();
    LogicalMargin::from_physical(style.writing_mode, SideOffsets2D::new(
        specified(padding_style.padding_top, containing_block_isize),
        specified(padding_style.padding_right, containing_block_isize),
        specified(padding_style.padding_bottom, containing_block_isize),
        specified(padding_style.padding_left, containing_block_isize)))
}

