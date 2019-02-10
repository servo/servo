/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

//! Computed types for CSS values related to backgrounds.

use crate::values::computed::length::NonNegativeLengthPercentage;
use crate::values::generics::background::BackgroundSize as GenericBackgroundSize;
use crate::values::generics::length::LengthPercentageOrAuto;

pub use crate::values::specified::background::BackgroundRepeat;

/// A computed value for the `background-size` property.
pub type BackgroundSize = GenericBackgroundSize<NonNegativeLengthPercentage>;

impl BackgroundSize {
    /// Returns `auto auto`.
    pub fn auto() -> Self {
        GenericBackgroundSize::Explicit {
            width: LengthPercentageOrAuto::auto(),
            height: LengthPercentageOrAuto::auto(),
        }
    }
}
