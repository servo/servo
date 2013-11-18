/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

//! Borders, padding, and margins.

use servo_util::geometry::Au;
use computed = style::computed_values;

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
