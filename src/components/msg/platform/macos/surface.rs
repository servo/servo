/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

//! Mac OS-specific implementation of cross-process surfaces. This uses `IOSurface`, introduced
//! in Mac OS X 10.6 Snow Leopard.

use platform::surface::NativeSurfaceAzureMethods;

use azure::AzSkiaGrGLSharedSurfaceRef;
use io_surface::IOSurface;
use layers::platform::surface::NativeSurface;
use std::cast;

impl NativeSurfaceAzureMethods for NativeSurface {
    fn from_azure_surface(surface: AzSkiaGrGLSharedSurfaceRef) -> NativeSurface {
        unsafe {
            let io_surface = IOSurface {
                obj: cast::transmute(surface),
            };
            NativeSurface::from_io_surface(io_surface)
        }
    }
}

