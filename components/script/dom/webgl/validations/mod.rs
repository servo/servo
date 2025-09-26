/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

pub(crate) trait WebGLValidator {
    type ValidatedOutput;
    type Error: ::std::error::Error;

    fn validate(self) -> Result<Self::ValidatedOutput, Self::Error>;
}

pub(crate) mod tex_image_2d;
pub(crate) mod tex_image_3d;
pub(crate) mod types;
