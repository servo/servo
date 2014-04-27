/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

//! Borders, padding, and margins.

use layout::box_::Box;

use computed = style::computed_values;
use style::computed_values::{LPA_Auto, LPA_Length, LPA_Percentage, LP_Length, LP_Percentage};
use servo_util::geometry::Au;
use servo_util::geometry;

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

/// Represents the top and bottom margins of a flow with collapsible margins. See CSS 2.1 § 8.3.1.
pub enum CollapsibleMargins {
    /// Margins may not collapse with this flow.
    NoCollapsibleMargins(Au, Au),

    /// Both the top and bottom margins (specified here in that order) may collapse, but the
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
    pub top_margin: AdjoiningMargins,
    pub margin_in: AdjoiningMargins,
}

impl MarginCollapseInfo {
    /// TODO(#2012, pcwalton): Remove this method once `box_` is not an `Option`.
    pub fn new() -> MarginCollapseInfo {
        MarginCollapseInfo {
            state: AccumulatingCollapsibleTopMargin,
            top_margin: AdjoiningMargins::new(),
            margin_in: AdjoiningMargins::new(),
        }
    }

    pub fn initialize_top_margin(&mut self,
                                 fragment: &Box,
                                 can_collapse_top_margin_with_kids: bool) {
        if !can_collapse_top_margin_with_kids {
            self.state = AccumulatingMarginIn
        }

        self.top_margin = AdjoiningMargins::from_margin(fragment.margin.borrow().top)
    }

    pub fn finish_and_compute_collapsible_margins(mut self,
                                                  fragment: &Box,
                                                  can_collapse_bottom_margin_with_kids: bool)
                                                  -> (CollapsibleMargins, Au) {
        let state = match self.state {
            AccumulatingCollapsibleTopMargin => {
                match fragment.style().Box.get().height {
                    LPA_Auto | LPA_Length(Au(0)) | LPA_Percentage(0.) => {
                        match fragment.style().Box.get().min_height {
                            LP_Length(Au(0)) | LP_Percentage(0.) => {
                                MarginsCollapseThroughFinalMarginState
                            },
                            _ => {
                                // If the box has non-zero min-height, margins may not collapse
                                // through it.
                                BottomMarginCollapsesFinalMarginState
                            }
                        }
                    },
                    _ => {
                        // If the box has an explicitly specified height, margins may not collapse
                        // through it.
                        BottomMarginCollapsesFinalMarginState
                    }
                }
            }
            AccumulatingMarginIn => BottomMarginCollapsesFinalMarginState,
        };

        // Different logic is needed here depending on whether this flow can collapse its bottom
        // margin with its children.
        let bottom_margin = fragment.margin.borrow().bottom;
        if !can_collapse_bottom_margin_with_kids {
            match state {
                MarginsCollapseThroughFinalMarginState => {
                    let advance = self.top_margin.collapse();
                    self.margin_in.union(AdjoiningMargins::from_margin(bottom_margin));
                    (MarginsCollapse(self.top_margin, self.margin_in), advance)
                }
                BottomMarginCollapsesFinalMarginState => {
                    let advance = self.margin_in.collapse();
                    self.margin_in.union(AdjoiningMargins::from_margin(bottom_margin));
                    (MarginsCollapse(self.top_margin, self.margin_in), advance)
                }
            }
        } else {
            match state {
                MarginsCollapseThroughFinalMarginState => {
                    self.top_margin.union(AdjoiningMargins::from_margin(bottom_margin));
                    (MarginsCollapseThrough(self.top_margin), Au(0))
                }
                BottomMarginCollapsesFinalMarginState => {
                    self.margin_in.union(AdjoiningMargins::from_margin(bottom_margin));
                    (MarginsCollapse(self.top_margin, self.margin_in), Au(0))
                }
            }
        }
    }

    pub fn current_float_ceiling(&mut self) -> Au {
        match self.state {
            AccumulatingCollapsibleTopMargin => self.top_margin.collapse(),
            AccumulatingMarginIn => self.margin_in.collapse(),
        }
    }

    /// Adds the child's potentially collapsible top margin to the current margin state and
    /// advances the Y offset by the appropriate amount to handle that margin. Returns the amount
    /// that should be added to the Y offset during block layout.
    pub fn advance_top_margin(&mut self, child_collapsible_margins: &CollapsibleMargins) -> Au {
        match (self.state, *child_collapsible_margins) {
            (AccumulatingCollapsibleTopMargin, NoCollapsibleMargins(top, _)) => {
                self.state = AccumulatingMarginIn;
                top
            }
            (AccumulatingCollapsibleTopMargin, MarginsCollapse(top, _)) => {
                self.top_margin.union(top);
                self.state = AccumulatingMarginIn;
                Au(0)
            }
            (AccumulatingMarginIn, NoCollapsibleMargins(top, _)) => {
                let previous_margin_value = self.margin_in.collapse();
                self.margin_in = AdjoiningMargins::new();
                previous_margin_value + top
            }
            (AccumulatingMarginIn, MarginsCollapse(top, _)) => {
                self.margin_in.union(top);
                let margin_value = self.margin_in.collapse();
                self.margin_in = AdjoiningMargins::new();
                margin_value
            }
            (_, MarginsCollapseThrough(_)) => {
                // For now, we ignore this; this will be handled by `advance_bottom_margin` below.
                Au(0)
            }
        }
    }

    /// Adds the child's potentially collapsible bottom margin to the current margin state and
    /// advances the Y offset by the appropriate amount to handle that margin. Returns the amount
    /// that should be added to the Y offset during block layout.
    pub fn advance_bottom_margin(&mut self, child_collapsible_margins: &CollapsibleMargins) -> Au {
        match (self.state, *child_collapsible_margins) {
            (AccumulatingCollapsibleTopMargin, NoCollapsibleMargins(..)) |
            (AccumulatingCollapsibleTopMargin, MarginsCollapse(..)) => {
                // Can't happen because the state will have been replaced with
                // `AccumulatingMarginIn` above.
                fail!("should not be accumulating collapsible top margins anymore!")
            }
            (AccumulatingCollapsibleTopMargin, MarginsCollapseThrough(margin)) => {
                self.top_margin.union(margin);
                Au(0)
            }
            (AccumulatingMarginIn, NoCollapsibleMargins(_, bottom)) => {
                assert_eq!(self.margin_in.most_positive, Au(0));
                assert_eq!(self.margin_in.most_negative, Au(0));
                bottom
            }
            (AccumulatingMarginIn, MarginsCollapse(_, bottom)) |
            (AccumulatingMarginIn, MarginsCollapseThrough(bottom)) => {
                self.margin_in.union(bottom);
                Au(0)
            }
        }
    }
}

pub enum MarginCollapseState {
    AccumulatingCollapsibleTopMargin,
    AccumulatingMarginIn,
}

/// Intrinsic widths, which consist of minimum and preferred.
pub struct IntrinsicWidths {
    /// The *minimum width* of the content.
    pub minimum_width: Au,
    /// The *preferred width* of the content.
    pub preferred_width: Au,
    /// The estimated sum of borders, padding, and margins. Some calculations use this information
    /// when computing intrinsic widths.
    pub surround_width: Au,
}

impl IntrinsicWidths {
    pub fn new() -> IntrinsicWidths {
        IntrinsicWidths {
            minimum_width: Au(0),
            preferred_width: Au(0),
            surround_width: Au(0),
        }
    }

    pub fn total_minimum_width(&self) -> Au {
        self.minimum_width + self.surround_width
    }

    pub fn total_preferred_width(&self) -> Au {
        self.preferred_width + self.surround_width
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
