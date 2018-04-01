/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

//! Common handling for the specified value CSS url() values.

use values::{Either, None_};

#[cfg(feature = "servo")]
pub use ::servo::url::{SpecifiedUrl, SpecifiedImageUrl};
#[cfg(feature = "gecko")]
pub use ::gecko::url::{SpecifiedUrl, SpecifiedImageUrl};

#[allow(missing_docs)]
pub type UrlOrNone = Either<SpecifiedUrl, None_>;
