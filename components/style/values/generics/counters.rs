/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

//! Generic types for counters-related CSS values.

use std::fmt;
use std::fmt::Write;
use style_traits::{CssWriter, ToCss};
use values::CustomIdent;

/// A generic value for both the `counter-increment` and `counter-reset` property.
///
/// Keyword `none` is represented by an empty vector.
#[derive(Clone, Debug, MallocSizeOf, PartialEq)]
pub struct CounterIntegerList<I>(Vec<(CustomIdent, I)>);

impl<I> CounterIntegerList<I> {
    /// Returns the `none` value.
    #[inline]
    pub fn none() -> CounterIntegerList<I> {
        CounterIntegerList(Vec::new())
    }

    /// Returns a clone of the values of the CounterIntegerList object.
    pub fn new(vec: Vec<(CustomIdent, I)>) -> CounterIntegerList<I> {
        CounterIntegerList(vec)
    }

    /// Returns a clone of the values of the CounterIntegerList object.
    pub fn get_values(&self) -> Vec<(CustomIdent, I)> where I: Copy {
        self.0.clone()
    }
}

impl<I> ToCss for CounterIntegerList<I>
where
    I: ToCss
{
    #[inline]
    fn to_css<W>(&self, dest: &mut CssWriter<W>) -> fmt::Result
    where
        W: fmt::Write,
    {
        if self.0.is_empty() {
            return dest.write_str("none")
        }

        let mut first = true;
        for &(ref name, ref value) in &self.0 {
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
