/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

//! Computed types for CSS Easing functions.

use crate::piecewise_linear::PiecewiseLinearFunctionBuildParameters;
use crate::values::computed::{Integer, Number, Percentage};
use crate::values::generics::easing;

/// A computed timing function.
pub type ComputedTimingFunction = easing::TimingFunction<Integer, Number, Percentage>;

/// An alias of the computed timing function.
pub type TimingFunction = ComputedTimingFunction;

/// A computed linear easing entry.
pub type ComputedLinearStop = easing::LinearStop<Number, Percentage>;

impl ComputedLinearStop {
    /// Convert this type to entries that can be used to build PiecewiseLinearFunction.
    pub fn to_piecewise_linear_build_parameters(
        x: &ComputedLinearStop,
    ) -> PiecewiseLinearFunctionBuildParameters {
        (
            x.output,
            x.input_start.into_rust().map(|x| x.0),
            x.input_end.into_rust().map(|x| x.0),
        )
    }
}
