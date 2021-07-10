/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

//! Generic types for CSS values related to <ratio>.
//! https://drafts.csswg.org/css-values/#ratios

use std::fmt::{self, Write};
use style_traits::{CssWriter, ToCss};

/// A generic value for the `<ratio>` value.
#[derive(
    Clone,
    Copy,
    Debug,
    MallocSizeOf,
    PartialEq,
    SpecifiedValueInfo,
    ToComputedValue,
    ToResolvedValue,
    ToShmem,
)]
#[repr(C)]
pub struct Ratio<N>(pub N, pub N);

impl<N> ToCss for Ratio<N>
where
    N: ToCss,
{
    fn to_css<W>(&self, dest: &mut CssWriter<W>) -> fmt::Result
    where
        W: Write,
    {
        self.0.to_css(dest)?;
        // Even though 1 could be omitted, we don't per
        // https://drafts.csswg.org/css-values-4/#ratio-value:
        //
        //     The second <number> is optional, defaulting to 1. However,
        //     <ratio> is always serialized with both components.
        //
        // And for compat reasons, see bug 1669742.
        //
        // We serialize with spaces for consistency with all other
        // slash-delimited things, see
        // https://github.com/w3c/csswg-drafts/issues/4282
        dest.write_str(" / ")?;
        self.1.to_css(dest)?;
        Ok(())
    }
}
