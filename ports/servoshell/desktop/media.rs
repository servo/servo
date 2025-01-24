/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use std::rc::Rc;

use servo::webrender_traits::SurfmanRenderingContext;
use servo::MediaGlSetup;
#[allow(unused_imports)]
use servo_media::player::context::GlContext;
#[cfg(all(target_os = "linux", not(target_env = "ohos")))]
use surfman::platform::generic::multi::connection::NativeConnection as LinuxNativeConnection;
#[cfg(all(target_os = "linux", not(target_env = "ohos")))]
use surfman::platform::generic::multi::context::NativeContext as LinuxNativeContext;
#[cfg(all(target_os = "linux", not(target_env = "ohos")))]
use surfman::{NativeConnection, NativeContext};

#[cfg(all(target_os = "linux", not(target_env = "ohos")))]
pub(crate) fn get_native_media_display_and_gl_context(
    rendering_context: &Rc<SurfmanRenderingContext>,
) -> Option<MediaGlSetup> {
    let gl_context = match rendering_context.context() {
        NativeContext::Default(LinuxNativeContext::Default(native_context)) => {
            GlContext::Egl(native_context.egl_context as usize)
        },
        NativeContext::Default(LinuxNativeContext::Alternate(native_context)) => {
            GlContext::Egl(native_context.egl_context as usize)
        },
        NativeContext::Alternate(_) => return None,
    };

    let native_display = match rendering_context.connection().native_connection() {
        NativeConnection::Default(LinuxNativeConnection::Default(connection)) => {
            NativeDisplay::Egl(connection.0 as usize)
        },
        NativeConnection::Default(LinuxNativeConnection::Alternate(connection)) => {
            NativeDisplay::X11(connection.x11_display as usize)
        },
        NativeConnection::Alternate(_) => return None,
    };
    Some(MediaGlSetup {
        native_display,
        gl_context,
    })
}

// @TODO(victor): https://github.com/servo/media/pull/315
#[cfg(target_os = "windows")]
pub(crate) fn get_native_media_display_and_gl_context(
    rendering_context: &Rc<SurfmanRenderingContext>,
) -> Option<MediaGlSetup> {
    #[cfg(feature = "no-wgl")]
    {
        let gl_context = GlContext::Egl(rendering_context.context().egl_context as usize);
        let native_display = NativeDisplay::Egl(rendering_context.device().egl_display as usize);
        Some(MediaGlSetup {
            native_display,
            gl_context,
        })
    }
    #[cfg(not(feature = "no-wgl"))]
    None
}

#[cfg(not(any(
    target_os = "windows",
    all(target_os = "linux", not(target_env = "ohos"))
)))]
pub(crate) fn get_native_media_display_and_gl_context(
    _rendering_context: &Rc<SurfmanRenderingContext>,
) -> Option<MediaGlSetup> {
    None
}
