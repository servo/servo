/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

//! Animation implementations for various SVG-related types.

use super::{Animate, Procedure};
use crate::properties::animated_properties::ListAnimation;
use crate::values::distance::{ComputeSquaredDistance, SquaredDistance};
use crate::values::generics::svg::SVGStrokeDashArray;

/// <https://www.w3.org/TR/SVG11/painting.html#StrokeDasharrayProperty>
impl<L> Animate for SVGStrokeDashArray<L>
where
    L: Clone + Animate,
{
    #[inline]
    fn animate(&self, other: &Self, procedure: Procedure) -> Result<Self, ()> {
        if matches!(procedure, Procedure::Add | Procedure::Accumulate { .. }) {
            // Non-additive.
            return Err(());
        }
        match (self, other) {
            (&SVGStrokeDashArray::Values(ref this), &SVGStrokeDashArray::Values(ref other)) => Ok(
                SVGStrokeDashArray::Values(this.animate_repeatable_list(other, procedure)?),
            ),
            _ => Err(()),
        }
    }
}

impl<L> ComputeSquaredDistance for SVGStrokeDashArray<L>
where
    L: ComputeSquaredDistance,
{
    #[inline]
    fn compute_squared_distance(&self, other: &Self) -> Result<SquaredDistance, ()> {
        match (self, other) {
            (&SVGStrokeDashArray::Values(ref this), &SVGStrokeDashArray::Values(ref other)) => {
                this.squared_distance_repeatable_list(other)
            },
            _ => Err(()),
        }
    }
}
