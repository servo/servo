/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use std::cell::RefMut;

use surfman::{Context, Device};

#[cfg(not(any(
    target_os = "windows",
    all(target_os = "linux", not(target_env = "ohos"))
)))]
pub(crate) fn setup_gl_accelerated_media(_: RefMut<'_, Device>, _: RefMut<'_, Context>) {}

#[cfg(all(target_os = "linux", not(target_env = "ohos")))]
pub(crate) fn setup_gl_accelerated_media(device: RefMut<'_, Device>, context: RefMut<'_, Context>) {
    use servo::{MediaGlContext, MediaNativeDisplay, Servo};
    use surfman::platform::generic::multi::connection::NativeConnection;
    use surfman::platform::generic::multi::context::NativeContext;

    let api = api(&device, &context);
    let context = match device.native_context(&context) {
        NativeContext::Default(NativeContext::Default(native_context)) => {
            MediaGlContext::Egl(native_context.egl_context as usize)
        },
        NativeContext::Default(NativeContext::Alternate(native_context)) => {
            MediaGlContext::Egl(native_context.egl_context as usize)
        },
        NativeContext::Alternate(_) => MediaGlContext::Unknown,
    };
    let display = match device.connection().native_connection() {
        surfman::NativeConnection::Default(NativeConnection::Default(connection)) => {
            MediaNativeDisplay::Egl(connection.0 as usize)
        },
        surfman::NativeConnection::Default(NativeConnection::Alternate(connection)) => {
            MediaNativeDisplay::X11(connection.x11_display as usize)
        },
        surfman::NativeConnection::Alternate(_) => MediaNativeDisplay::Unknown,
    };
    Servo::initialize_gl_accelerated_media(display, api, context);
}

#[cfg(target_os = "windows")]
pub(crate) fn setup_gl_accelerated_media(device: RefMut<'_, Device>, context: RefMut<'_, Context>) {
    use servo::{MediaGlContext, MediaNativeDisplay, Servo};

    let api = api(&device, &context);
    let context = MediaGlContext::Egl(device.native_context(&context).egl_context as usize);
    let display = MediaNativeDisplay::Egl(device.native_device().egl_display as usize);
    Servo::initialize_gl_accelerated_media(display, api, context);
}

#[cfg(any(
    all(target_os = "linux", not(target_env = "ohos")),
    target_os = "windows"
))]
fn api(device: &RefMut<Device>, context: &RefMut<Context>) -> servo::MediaGlApi {
    use servo::MediaGlApi;
    use surfman::GLApi;

    let descriptor = device.context_descriptor(context);
    let attributes = device.context_descriptor_attributes(&descriptor);
    let major = attributes.version.major;
    let minor = attributes.version.minor;
    match device.connection().gl_api() {
        GLApi::GL if major >= 3 && minor >= 2 => MediaGlApi::OpenGL3,
        GLApi::GL => MediaGlApi::OpenGL,
        GLApi::GLES if major > 1 => MediaGlApi::Gles2,
        GLApi::GLES => MediaGlApi::Gles1,
    }
}
