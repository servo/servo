/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

//! Common handling for the computed value CSS url() values.

use super::{Either, None_};

#[cfg(feature = "servo")]
pub use ::servo::url::{ComputedUrl, ComputedImageUrl};
#[cfg(feature = "gecko")]
pub use ::gecko::url::{ComputedUrl, ComputedImageUrl};

/// <url> | <none>
pub type UrlOrNone = Either<ComputedUrl, None_>;
