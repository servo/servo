/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

pub mod ext;
mod extension;
mod extensions;
mod wrapper;

pub use self::extension::{WebGLExtension, WebGLExtensionSpec};
pub use self::extensions::WebGLExtensions;
