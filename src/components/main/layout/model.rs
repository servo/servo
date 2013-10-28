/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

//! Borders, padding, and margins.

use std::num::Zero;
use geom::side_offsets::SideOffsets2D;
use servo_util::geometry::Au;
use style::ComputedValues;
use style::properties::common_types::computed;

/// Encapsulates the borders, padding, and margins, which we collectively call the "box model".
#[deriving(Clone)]
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
    /// Populates the box model border parameters from the given computed style.
    pub fn compute_borders(&mut self, style: &ComputedValues) {
        self.border.top = style.Border.border_top_width;
        self.border.right = style.Border.border_right_width;
        self.border.bottom = style.Border.border_bottom_width;
        self.border.left = style.Border.border_left_width;
    }

    /// Populates the box model padding parameters from the given computed style.
    pub fn compute_padding(&mut self, style: &ComputedValues, containing_width: Au) {
        self.padding.top = self.compute_padding_length(style.Padding.padding_top,
                                                       containing_width);
        self.padding.right = self.compute_padding_length(style.Padding.padding_right,
                                                         containing_width);
        self.padding.bottom = self.compute_padding_length(style.Padding.padding_bottom,
                                                          containing_width);
        self.padding.left = self.compute_padding_length(style.Padding.padding_left,
                                                        containing_width);
    }

    pub fn noncontent_width(&self) -> Au {
        let left = self.margin.left + self.border.left + self.padding.left;
        let right = self.margin.right + self.border.right + self.padding.right;
        left + right
    }

    pub fn noncontent_height(&self) -> Au {
        let top = self.margin.top + self.border.top + self.padding.top;
        let bottom = self.margin.bottom + self.border.bottom + self.padding.bottom;
        top + bottom
    }

    pub fn offset(&self) -> Au {
        self.margin.left + self.border.left + self.padding.left
    }

    pub fn compute_padding_length(&self, padding: computed::LengthOrPercentage, content_box_width: Au) -> Au {
        match padding {
            computed::LP_Length(length) => length,
            computed::LP_Percentage(p) => content_box_width.scale_by(p)
        }
    }
}
