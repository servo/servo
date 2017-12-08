/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

//! Computed values for counter properties

use std::fmt;
use style_traits::ToCss;
use values::CustomIdent;

/// A computed value for the `counter-reset` property.
#[derive(Clone, Debug, MallocSizeOf, PartialEq)]
pub struct CounterReset(pub Vec<(CustomIdent, i32)>);

impl CounterReset {
    /// Returns the `none` value.
    pub fn none() -> CounterReset {
        CounterReset(Vec::new())
    }
}

impl ToCss for CounterReset {
    fn to_css<W>(&self, dest: &mut W) -> fmt::Result
        where W: fmt::Write,
    {
        if self.0.is_empty() {
            return dest.write_str("none")
        }

        let mut first = true;
        for &(ref name, value) in &self.0 {
            if !first {
                dest.write_str(" ")?;
            }
            first = false;
            name.to_css(dest)?;
            dest.write_str(" ")?;
            value.to_css(dest)?;
        }
        Ok(())
    }
}
