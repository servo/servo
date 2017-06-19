/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

//! Generic types for counters-related CSS values.

use std::fmt;
use style_traits::{HasViewportPercentage, ToCss};
use values::CustomIdent;

/// A generic value for the `counter-increment` property.
///
/// Keyword `none` is represented by an empty slice.
#[derive(Clone, Debug, PartialEq, ToComputedValue)]
pub struct CounterIncrement<Integer>(Box<[(CustomIdent, Integer)]);

impl<I> CounterIncrement<I> {
    /// Returns `none`.
    #[inline]
    pub fn none() -> Self {
        CounterIncrement(vec![].into_boxed_slice())
    }
}

impl<I> HasViewportPercentage for CounterIncrement<I> {
    #[inline] fn has_viewport_percentage(&self) -> bool { false }
}

impl<I> ToCss for CounterIncrement<I>
where
    I: ToCss,
{
    #[inline]
    fn to_css<W>(&self, dest: &mut W) -> fmt::Result
    where
        W: fmt::Write,
    {
        if self.0.is_empty() {
            return dest.write_str("none");
        }
        for (&(ref name, ref value), i) in self.0.iter().enumerate() {
            if i != 0 {
                dest.write_str(" ")?;
            }
            name.to_css(dest)?;
            dest.write_str(" ")?;
            value.to_css(dest)?;
        }
        Ok(())
    }
}
