/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

//! Computed types for CSS borders.

use crate::values::computed::length::NonNegativeLengthOrNumber;
use crate::values::generics::rect::Rect;

/// A specified rectangle made of four `<length-or-number>` values.
pub type NonNegativeLengthOrNumberRect = Rect<NonNegativeLengthOrNumber>;
