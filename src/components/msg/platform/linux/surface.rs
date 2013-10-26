/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

//! X11-specific implementation of cross-process surfaces. This uses X pixmaps.

use platform::surface::NativeSurfaceAzureMethods;

use azure::AzSkiaGrGLSharedSurfaceRef;
use layers::platform::surface::NativeSurface;
use std::cast;

impl NativeSurfaceAzureMethods for NativeSurface {
    fn from_azure_surface(surface: AzSkiaGrGLSharedSurfaceRef) -> NativeSurface {
        unsafe {
            NativeSurface::from_pixmap(cast::transmute(surface))
        }
    }
}

