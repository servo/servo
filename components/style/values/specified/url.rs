/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

//! Common handling for the specified value CSS url() values.

use values::generics::url::UrlOrNone as GenericUrlOrNone;

#[cfg(feature = "servo")]
pub use ::servo::url::{SpecifiedUrl, SpecifiedImageUrl};
#[cfg(feature = "gecko")]
pub use ::gecko::url::{SpecifiedUrl, SpecifiedImageUrl};

/// Specified <url> | <none>
pub type UrlOrNone = GenericUrlOrNone<SpecifiedUrl>;

/// Specified image <url> | <none>
pub type ImageUrlOrNone = GenericUrlOrNone<SpecifiedImageUrl>;
