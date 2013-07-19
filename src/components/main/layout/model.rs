/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

//! Borders, padding, and margins.

use std::num::Zero;
use geom::side_offsets::SideOffsets2D;
use gfx::geometry::Au;
use newcss::complete::CompleteStyle;
use newcss::units::{Em, Pt, Px};
use newcss::values::{CSSBorderWidth, CSSBorderWidthLength, CSSBorderWidthMedium};
use newcss::values::{CSSBorderWidthThick, CSSBorderWidthThin, CSSFontSize, CSSFontSizeLength};
use newcss::values::{CSSWidth, CSSWidthLength, CSSWidthPercentage, CSSWidthAuto};
use newcss::values::{CSSHeight, CSSHeightLength, CSSHeightPercentage, CSSHeightAuto};
use newcss::values::{CSSMargin, CSSMarginLength, CSSMarginPercentage, CSSMarginAuto};
use newcss::values::{CSSPadding, CSSPaddingLength, CSSPaddingPercentage};
/// Encapsulates the borders, padding, and margins, which we collectively call the "box model".
pub struct BoxModel {
    border: SideOffsets2D<Au>,
    padding: SideOffsets2D<Au>,
    margin: SideOffsets2D<Au>,
    /// The width of the content box.
    content_box_width: Au,
}

/// Useful helper data type when computing values for blocks and positioned elements.
pub enum MaybeAuto {
    Auto,
    Specified(Au),
}

impl MaybeAuto {
    pub fn from_margin(margin: CSSMargin, containing_width: Au, font_size: CSSFontSize) -> MaybeAuto {
        match margin {
            CSSMarginAuto => Auto,
            CSSMarginPercentage(percent) => Specified(containing_width.scale_by(percent/100.0)),
            CSSMarginLength(Px(v)) => Specified(Au::from_frac_px(v)),
            CSSMarginLength(Pt(v)) => Specified(Au::from_pt(v)),
            CSSMarginLength(Em(em)) => {
                match font_size {
                    CSSFontSizeLength(Px(v)) => Specified(Au::from_frac_px(em * v)),
                    CSSFontSizeLength(Pt(v)) => Specified(Au::from_pt(em * v)),
                    _ => fail!(~"expected non-relative font size"),
                }
            }
        }
    }

    pub fn from_width(width: CSSWidth, containing_width: Au, font_size: CSSFontSize) -> MaybeAuto {
        match width {
            CSSWidthAuto => Auto,
            CSSWidthPercentage(percent) => Specified(containing_width.scale_by(percent/100.0)),
            CSSWidthLength(Px(v)) => Specified(Au::from_frac_px(v)),
            CSSWidthLength(Pt(v)) => Specified(Au::from_pt(v)),
            CSSWidthLength(Em(em)) => {
                match font_size {
                    CSSFontSizeLength(Px(v)) => Specified(Au::from_frac_px(em * v)),
                    CSSFontSizeLength(Pt(v)) => Specified(Au::from_pt(em * v)),
                    _ => fail!(~"expected non-relative font size"),
                }
            }
        }
    }

    pub fn from_height(height: CSSHeight, cb_height: Au, font_size: CSSFontSize) -> MaybeAuto {
        match height {
            CSSHeightAuto => Auto,
            CSSHeightPercentage(percent) => Specified(cb_height.scale_by(percent/100.0)),
            CSSHeightLength(Px(v)) => Specified(Au::from_frac_px(v)),
            CSSHeightLength(Pt(v)) => Specified(Au::from_pt(v)),
            CSSHeightLength(Em(em)) => {
                match font_size {
                    CSSFontSizeLength(Px(v)) => Specified(Au::from_frac_px(em * v)),
                    CSSFontSizeLength(Pt(v)) => Specified(Au::from_pt(em * v)),
                    _ => fail!(~"expected non-relative font size"),
                }
            }

        }
    }

    pub fn specified_or_default(&self, default: Au) -> Au {
        match *self {
            Auto => default,
            Specified(value) => value,
        }
    }

    pub fn specified_or_zero(&self) -> Au {
        self.specified_or_default(Au(0))
    }
}

impl Zero for BoxModel {
    fn zero() -> BoxModel {
        BoxModel {
            border: Zero::zero(),
            padding: Zero::zero(),
            margin: Zero::zero(),
            content_box_width: Zero::zero(),
        }
    }

    fn is_zero(&self) -> bool {
        self.padding.is_zero() && self.border.is_zero() && self.margin.is_zero()
    }
}

impl BoxModel {
    /// Populates the box model parameters from the given computed style.
    pub fn compute_borders(&mut self, style: CompleteStyle) {
        // Compute the borders.
        self.border.top = self.compute_border_width(style.border_top_width(), style.font_size());
        self.border.right = self.compute_border_width(style.border_right_width(), style.font_size());
        self.border.bottom = self.compute_border_width(style.border_bottom_width(), style.font_size());
        self.border.left = self.compute_border_width(style.border_left_width(), style.font_size());
    }

    pub fn compute_padding(&mut self, style: CompleteStyle, containing_width: Au) {
        self.padding.top = self.compute_padding_length(style.padding_top(), containing_width, style.font_size());
        self.padding.right = self.compute_padding_length(style.padding_right(), containing_width, style.font_size());
        self.padding.bottom = self.compute_padding_length(style.padding_bottom(), containing_width, style.font_size());
        self.padding.left = self.compute_padding_length(style.padding_left(), containing_width, style.font_size());
    }

    pub fn noncontent_width(&self) -> Au {
        let left = self.margin.left + self.border.left + self.padding.left;
        let right = self.margin.right + self.border.right + self.padding.right;
        left + right
    }

    pub fn offset(&self) -> Au {
        self.margin.left + self.border.left + self.padding.left
    }

    /// Helper function to compute the border width in app units from the CSS border width.
    pub fn compute_border_width(&self, width: CSSBorderWidth, font_size: CSSFontSize) -> Au {
        match width {
            CSSBorderWidthLength(Px(v)) => Au::from_frac_px(v),
            CSSBorderWidthLength(Pt(v)) => Au::from_pt(v),
            CSSBorderWidthLength(Em(em)) => {
                match font_size {
                    CSSFontSizeLength(Px(v)) => Au::from_frac_px(em * v),
                    CSSFontSizeLength(Pt(v)) => Au::from_pt(em * v),
                    _ => fail!(~"expected non-relative font size"),
                }
            },
            CSSBorderWidthThin => Au::from_px(1),
            CSSBorderWidthMedium => Au::from_px(5),
            CSSBorderWidthThick => Au::from_px(10),
        }
    }

    pub fn compute_padding_length(&self, padding: CSSPadding, content_box_width: Au, font_size: CSSFontSize) -> Au {
        match padding {
            CSSPaddingLength(Px(v)) => Au::from_frac_px(v),
            CSSPaddingLength(Pt(v)) => Au::from_pt(v),
            CSSPaddingLength(Em(em)) => {
                match font_size {
                    CSSFontSizeLength(Px(v)) => Au::from_frac_px(em * v),
                    CSSFontSizeLength(Pt(v)) => Au::from_pt(em * v),
                    _ => fail!(~"expected non-relative font size"),
                }
            },
            CSSPaddingPercentage(p) => content_box_width.scale_by(p/100.0)
        }
    }
}
