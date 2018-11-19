/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

//! Computed types for CSS values related to backgrounds.

use crate::values::computed::length::NonNegativeLengthOrPercentageOrAuto;
use crate::values::generics::background::BackgroundSize as GenericBackgroundSize;

pub use crate::values::specified::background::BackgroundRepeat;

/// A computed value for the `background-size` property.
pub type BackgroundSize = GenericBackgroundSize<NonNegativeLengthOrPercentageOrAuto>;

impl BackgroundSize {
    /// Returns `auto auto`.
    pub fn auto() -> Self {
        GenericBackgroundSize::Explicit {
            width: NonNegativeLengthOrPercentageOrAuto::auto(),
            height: NonNegativeLengthOrPercentageOrAuto::auto(),
        }
    }
}
