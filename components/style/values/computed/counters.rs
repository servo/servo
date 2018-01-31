/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

//! Computed values for counter properties

use std::fmt;
use style_traits::{CssWriter, ToCss};
use values::CustomIdent;
use values::computed::{Context, ToComputedValue};
use values::generics::counters::CounterIntegerList;
use values::specified::{CounterIncrement as SpecifiedCounterIncrement, CounterReset as SpecifiedCounterReset, Integer};

type ComputedIntegerList = CounterIntegerList<i32>;

/// A computed value for the `counter-increment` property.
#[derive(Clone, Debug, MallocSizeOf, PartialEq)]
pub struct CounterIncrement(ComputedIntegerList);

impl CounterIncrement {
    /// Returns the `none` value.
    #[inline]
    pub fn none() -> CounterIncrement {
        CounterIncrement(CounterIntegerList::new(Vec::new()))
    }

    /// Returns a new computed `counter-increment` object with the given values.
    pub fn new(vec: Vec<(CustomIdent, i32)>) -> CounterIncrement {
        CounterIncrement(ComputedIntegerList::new(vec))
    }

    /// Returns a clone of the values of the computed `counter-increment` object.
    pub fn get_values(&self) -> Vec<(CustomIdent, i32)> {
        self.0.get_values()
    }
}

impl ToCss for CounterIncrement {
    fn to_css<W>(&self, dest: &mut CssWriter<W>) -> fmt::Result where W: fmt::Write,
    {
        self.0.to_css(dest)
    }
}

impl ToComputedValue for SpecifiedCounterIncrement {
    type ComputedValue = CounterIncrement;

    fn to_computed_value(&self, context: &Context) -> Self::ComputedValue {
        CounterIncrement::new(self.get_values().iter().map(|&(ref name, ref value)| {
            (name.clone(), value.to_computed_value(context))
        }).collect::<Vec<_>>())
    }

    fn from_computed_value(computed: &Self::ComputedValue) -> Self {
        SpecifiedCounterIncrement::new(computed.get_values().iter().map(|&(ref name, ref value)| {
            (name.clone(), Integer::from_computed_value(&value))
        }).collect::<Vec<_>>())
    }
}

/// A computed value for the `counter-reset` property.
#[derive(Clone, Debug, MallocSizeOf, PartialEq)]
pub struct CounterReset(ComputedIntegerList);

impl ToCss for CounterReset {
    fn to_css<W>(&self, dest: &mut CssWriter<W>) -> fmt::Result where W: fmt::Write,
    {
        self.0.to_css(dest)
    }
}

impl CounterReset {
    /// Returns the `none` value.
    #[inline]
    pub fn none() -> CounterReset {
        CounterReset(CounterIntegerList::new(Vec::new()))
    }

    /// Returns a new computed `counter-reset` object with the given values.
    pub fn new(vec: Vec<(CustomIdent, i32)>) -> CounterReset {
        CounterReset(ComputedIntegerList::new(vec))
    }

    /// Returns a clone of the values of the computed `counter-reset` object.
    pub fn get_values(&self) -> Vec<(CustomIdent, i32)> {
        self.0.get_values()
    }
}

impl ToComputedValue for SpecifiedCounterReset {
    type ComputedValue = CounterReset;

    fn to_computed_value(&self, context: &Context) -> Self::ComputedValue {
        CounterReset::new(self.get_values().iter().map(|&(ref name, ref value)| {
            (name.clone(), value.to_computed_value(context))
        }).collect::<Vec<_>>())
    }

    fn from_computed_value(computed: &Self::ComputedValue) -> Self {
        SpecifiedCounterReset::new(computed.get_values().iter().map(|&(ref name, ref value)| {
            (name.clone(), Integer::from_computed_value(&value))
        }).collect::<Vec<_>>())
    }
}
