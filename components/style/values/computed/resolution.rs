/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

//! Resolution values:
//!
//! https://drafts.csswg.org/css-values/#resolution

use std::fmt::{self, Write};
use style_traits::{CssWriter, ToCss};
use values::CSSFloat;
use values::computed::{Context, ToComputedValue};
use values::specified;

/// A computed `<resolution>`.
pub struct Resolution(CSSFloat);

impl Resolution {
    /// Returns this resolution value as dppx.
    #[inline]
    pub fn dppx(&self) -> CSSFloat {
        self.0
    }
}

impl ToComputedValue for specified::Resolution {
    type ComputedValue = Resolution;

    #[inline]
    fn to_computed_value(&self, _: &Context) -> Self::ComputedValue {
        Resolution(self.to_dppx())
    }

    #[inline]
    fn from_computed_value(computed: &Self::ComputedValue) -> Self {
        specified::Resolution::Dppx(computed.dppx())
    }
}

impl ToCss for Resolution {
    fn to_css<W>(&self, dest: &mut CssWriter<W>) -> fmt::Result
    where
        W: fmt::Write,
    {
        self.dppx().to_css(dest)?;
        dest.write_str("dppx")
    }
}
