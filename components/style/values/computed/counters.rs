/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

//! Computed values for counter properties

use values::CustomIdent;
use values::computed::{Context, ToComputedValue};
use values::generics::counters::CounterIntegerList;
use values::specified::{CounterIncrement as SpecifiedCounterIncrement, CounterReset as SpecifiedCounterReset};

type ComputedIntegerList = CounterIntegerList<i32>;

/// A computed value for the `counter-increment` property.
#[derive(Clone, Debug, MallocSizeOf, PartialEq, ToCss)]
pub struct CounterIncrement(pub ComputedIntegerList);

impl CounterIncrement {
    /// Returns the `none` value.
    #[inline]
    pub fn none() -> CounterIncrement {
        CounterIncrement(ComputedIntegerList::new(Vec::new()))
    }

    /// Returns a new computed `counter-increment` object with the given values.
    pub fn new(vec: Vec<(CustomIdent, i32)>) -> CounterIncrement {
        CounterIncrement(ComputedIntegerList::new(vec))
    }

    /// Returns the values of the computed `counter-increment` object.
    pub fn get_values(&self) -> &[(CustomIdent, i32)] {
        self.0.get_values()
    }
}

impl ToComputedValue for SpecifiedCounterIncrement {
    type ComputedValue = CounterIncrement;

    fn to_computed_value(&self, context: &Context) -> Self::ComputedValue {
        CounterIncrement(self.0.to_computed_value(context))
    }

    fn from_computed_value(computed: &Self::ComputedValue) -> Self {
        SpecifiedCounterIncrement(ToComputedValue::from_computed_value(&computed.0))
    }
}

/// A computed value for the `counter-reset` property.
#[derive(Clone, Debug, MallocSizeOf, PartialEq, ToCss)]
pub struct CounterReset(pub ComputedIntegerList);

impl CounterReset {
    /// Returns the `none` value.
    #[inline]
    pub fn none() -> CounterReset {
        CounterReset(ComputedIntegerList::new(Vec::new()))
    }

    /// Returns a new computed `counter-reset` object with the given values.
    pub fn new(vec: Vec<(CustomIdent, i32)>) -> CounterReset {
        CounterReset(ComputedIntegerList::new(vec))
    }

    /// Returns the values of the computed `counter-reset` object.
    pub fn get_values(&self) -> &[(CustomIdent, i32)] {
        self.0.get_values()
    }
}

impl ToComputedValue for SpecifiedCounterReset {
    type ComputedValue = CounterReset;

    fn to_computed_value(&self, context: &Context) -> Self::ComputedValue {
        CounterReset(self.0.to_computed_value(context))
    }

    fn from_computed_value(computed: &Self::ComputedValue) -> Self {
        SpecifiedCounterReset(ToComputedValue::from_computed_value(&computed.0))
    }
}
