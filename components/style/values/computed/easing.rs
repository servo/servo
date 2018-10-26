/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

//! Computed types for CSS Easing functions.

use values::computed::Number;
use values::generics::easing::TimingFunction as GenericTimingFunction;

/// A computed timing function.
pub type TimingFunction = GenericTimingFunction<u32, Number>;
