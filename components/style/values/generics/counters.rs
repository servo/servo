/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

//! Generic types for counters-related CSS values.

use std::fmt;
use std::fmt::Write;
use std::ops::Deref;
use style_traits::{CssWriter, ToCss};
use values::CustomIdent;

/// A generic value for the `counter-increment` property.
#[derive(Clone, Debug, Default, MallocSizeOf, PartialEq, ToComputedValue, ToCss)]
pub struct CounterIncrement<I>(Counters<I>);

impl<I> CounterIncrement<I> {
    /// Returns a new value for `counter-increment`.
    #[inline]
    pub fn new(counters: Vec<(CustomIdent, I)>) -> Self {
        CounterIncrement(Counters(counters.into_boxed_slice()))
    }
}

impl<I> Deref for CounterIncrement<I> {
    type Target = [(CustomIdent, I)];

    #[inline]
    fn deref(&self) -> &Self::Target {
        &(self.0).0
    }
}

/// A generic value for the `counter-reset` property.
#[derive(Clone, Debug, Default, MallocSizeOf, PartialEq, ToComputedValue, ToCss)]
pub struct CounterReset<I>(Counters<I>);

impl<I> CounterReset<I> {
    /// Returns a new value for `counter-reset`.
    #[inline]
    pub fn new(counters: Vec<(CustomIdent, I)>) -> Self {
        CounterReset(Counters(counters.into_boxed_slice()))
    }
}

impl<I> Deref for CounterReset<I> {
    type Target = [(CustomIdent, I)];

    #[inline]
    fn deref(&self) -> &Self::Target {
        &(self.0).0
    }
}

/// A generic value for lists of counters.
///
/// Keyword `none` is represented by an empty vector.
#[derive(Clone, Debug, MallocSizeOf, PartialEq, ToComputedValue)]
pub struct Counters<I>(Box<[(CustomIdent, I)]>);

impl<I> Default for Counters<I> {
    #[inline]
    fn default() -> Self {
        Counters(vec![].into_boxed_slice())
    }
}

impl<I> ToCss for Counters<I>
where
    I: ToCss,
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
        for &(ref name, ref value) in &*self.0 {
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
