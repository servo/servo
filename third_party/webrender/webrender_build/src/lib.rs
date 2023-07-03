/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

#[macro_use]
extern crate bitflags;

#[macro_use]
extern crate lazy_static;

#[cfg(any(feature = "serde"))]
#[macro_use]
extern crate serde;

pub mod shader;
pub mod shader_features;

/// This must be known at build-time as the shaders depend on it.
pub const MAX_VERTEX_TEXTURE_WIDTH: usize = 1024;
