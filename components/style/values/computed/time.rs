/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

//! Computed time values.

use std::fmt;
use style_traits::ToCss;
use values::CSSFloat;

/// A computed `<time>` value.
#[derive(Clone, PartialEq, PartialOrd, Copy, Debug)]
#[cfg_attr(feature = "servo", derive(HeapSizeOf, Deserialize, Serialize))]
pub struct Time {
    seconds: CSSFloat,
}

impl Time {
    /// Creates a time value from a seconds amount.
    pub fn from_seconds(seconds: CSSFloat) -> Self {
        Time {
            seconds: seconds,
        }
    }

    /// Returns `0s`.
    pub fn zero() -> Self {
        Self::from_seconds(0.0)
    }

    /// Returns the amount of seconds this time represents.
    #[inline]
    pub fn seconds(&self) -> CSSFloat {
        self.seconds
    }
}

impl ToCss for Time {
    fn to_css<W>(&self, dest: &mut W) -> fmt::Result
        where W: fmt::Write,
    {
        write!(dest, "{}s", self.seconds())
    }
}
