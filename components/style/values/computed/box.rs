/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

//! Computed types for box properties.

use values::computed::length::LengthOrPercentage;
use values::generics::box_::VerticalAlign as GenericVerticalAlign;

/// A computed value for the `vertical-align` property.
pub type VerticalAlign = GenericVerticalAlign<LengthOrPercentage>;
