/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

//! Computed types for CSS borders.

use values::computed::length::LengthOrNumber;
use values::generics::rect::Rect;

/// A specified rectangle made of four `<length-or-number>` values.
pub type LengthOrNumberRect = Rect<LengthOrNumber>;
