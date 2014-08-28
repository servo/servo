/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

//! Declarations of types for cross-process surfaces.

use azure::AzSkiaGrGLSharedSurfaceRef;

pub trait NativeSurfaceAzureMethods {
    fn from_azure_surface(surface: AzSkiaGrGLSharedSurfaceRef) -> Self;
}

