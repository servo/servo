/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

//! Computed types for legacy Gecko-only properties.

use values::computed::length::LengthOrPercentage;
use values::generics::gecko::ScrollSnapPoint as GenericScrollSnapPoint;

/// A computed type for scroll snap points.
pub type ScrollSnapPoint = GenericScrollSnapPoint<LengthOrPercentage>;
