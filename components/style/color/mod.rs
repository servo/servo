/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

//! Color support functions.

/// cbindgen:ignore
pub mod convert;

/// The 3 components that make up a color.  (Does not include the alpha component)
#[derive(Copy, Clone, Debug, MallocSizeOf, PartialEq, ToShmem)]
pub struct ColorComponents(pub f32, pub f32, pub f32);

impl ColorComponents {
    /// Apply a function to each of the 3 components of the color.
    pub fn map(self, f: impl Fn(f32) -> f32) -> Self {
        Self(f(self.0), f(self.1), f(self.2))
    }
}
