/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

//! Computed types for CSS values related to backgrounds.

use properties::animated_properties::{Animatable, RepeatableListAnimatable};
use values::computed::length::LengthOrPercentageOrAuto;
use values::generics::background::BackgroundSize as GenericBackgroundSize;

/// A computed value for the `background-size` property.
pub type BackgroundSize = GenericBackgroundSize<LengthOrPercentageOrAuto>;

impl RepeatableListAnimatable for BackgroundSize {}

impl Animatable for BackgroundSize {
    fn add_weighted(&self, other: &Self, self_portion: f64, other_portion: f64) -> Result<Self, ()> {
        match (self, other) {
            (
                &GenericBackgroundSize::Explicit { width: self_width, height: self_height },
                &GenericBackgroundSize::Explicit { width: other_width, height: other_height },
            ) => {
                Ok(GenericBackgroundSize::Explicit {
                    width: self_width.add_weighted(&other_width, self_portion, other_portion)?,
                    height: self_height.add_weighted(&other_height, self_portion, other_portion)?,
                })
            }
            _ => Err(()),
        }
    }

    #[inline]
    fn compute_distance(&self, other: &Self) -> Result<f64, ()> {
        self.compute_squared_distance(other).map(|sd| sd.sqrt())
    }

    #[inline]
    fn compute_squared_distance(&self, other: &Self) -> Result<f64, ()> {
        match (self, other) {
            (
                &GenericBackgroundSize::Explicit { width: self_width, height: self_height },
                &GenericBackgroundSize::Explicit { width: other_width, height: other_height },
            ) => {
                Ok(
                    self_width.compute_squared_distance(&other_width)? +
                    self_height.compute_squared_distance(&other_height)?
                )
            }
            _ => Err(()),
        }
    }
}
