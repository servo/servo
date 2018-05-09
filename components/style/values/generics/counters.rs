/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

//! Generic types for counters-related CSS values.

use std::ops::Deref;
use values::CustomIdent;

/// A name / value pair for counters.
#[derive(Clone, Debug, MallocSizeOf, PartialEq, SpecifiedValueInfo,
         ToComputedValue, ToCss)]
pub struct CounterPair<Integer> {
    /// The name of the counter.
    pub name: CustomIdent,
    /// The value of the counter / increment / etc.
    pub value: Integer,
}

/// A generic value for the `counter-increment` property.
#[derive(Clone, Debug, Default, MallocSizeOf, PartialEq, SpecifiedValueInfo,
         ToComputedValue, ToCss)]
pub struct CounterIncrement<I>(Counters<I>);

impl<I> CounterIncrement<I> {
    /// Returns a new value for `counter-increment`.
    #[inline]
    pub fn new(counters: Vec<CounterPair<I>>) -> Self {
        CounterIncrement(Counters(counters.into_boxed_slice()))
    }
}

impl<I> Deref for CounterIncrement<I> {
    type Target = [CounterPair<I>];

    #[inline]
    fn deref(&self) -> &Self::Target {
        &(self.0).0
    }
}

/// A generic value for the `counter-reset` property.
#[derive(Clone, Debug, Default, MallocSizeOf, PartialEq, SpecifiedValueInfo,
         ToComputedValue, ToCss)]
pub struct CounterReset<I>(Counters<I>);

impl<I> CounterReset<I> {
    /// Returns a new value for `counter-reset`.
    #[inline]
    pub fn new(counters: Vec<CounterPair<I>>) -> Self {
        CounterReset(Counters(counters.into_boxed_slice()))
    }
}

impl<I> Deref for CounterReset<I> {
    type Target = [CounterPair<I>];

    #[inline]
    fn deref(&self) -> &Self::Target {
        &(self.0).0
    }
}

/// A generic value for lists of counters.
///
/// Keyword `none` is represented by an empty vector.
#[derive(Clone, Debug, MallocSizeOf, PartialEq, SpecifiedValueInfo,
         ToComputedValue, ToCss)]
pub struct Counters<I>(#[css(iterable, if_empty = "none")] Box<[CounterPair<I>]>);

impl<I> Default for Counters<I> {
    #[inline]
    fn default() -> Self {
        Counters(vec![].into_boxed_slice())
    }
}
