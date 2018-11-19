/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

//! Common handling for the computed value CSS url() values.

use crate::values::generics::url::UrlOrNone as GenericUrlOrNone;

#[cfg(feature = "gecko")]
pub use crate::gecko::url::{ComputedImageUrl, ComputedUrl};
#[cfg(feature = "servo")]
pub use crate::servo::url::{ComputedImageUrl, ComputedUrl};

/// Computed <url> | <none>
pub type UrlOrNone = GenericUrlOrNone<ComputedUrl>;

/// Computed image <url> | <none>
pub type ImageUrlOrNone = GenericUrlOrNone<ComputedImageUrl>;
