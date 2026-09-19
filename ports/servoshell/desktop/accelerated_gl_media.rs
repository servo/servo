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
    use surfman::multi::connection::Connection;
    use surfman::multi::context::Context;
    use surfman::multi::device::Device;

    let api = api(&device, &context);
    let media_context = match (&*device, &*context) {
        (Device::Default(Device::Default(device)), Context::Default(Context::Default(context))) => {
            MediaGlContext::Egl(device.native_context(context).egl_context as usize)
        },
        (
            Device::Default(Device::Alternate(device)),
            Context::Default(Context::Alternate(context)),
        ) => MediaGlContext::Egl(device.native_context(context).egl_context as usize),
        _ => MediaGlContext::Unknown,
    };

    let media_display = match device.connection() {
        Connection::Default(Connection::Default(connection)) => {
            MediaNativeDisplay::Egl(connection.native_connection().0 as usize)
        },
        Connection::Default(Connection::Alternate(connection)) => {
            MediaNativeDisplay::X11(connection.native_connection().x11_display as usize)
        },
        _ => MediaNativeDisplay::Unknown,
    };

    Servo::initialize_gl_accelerated_media(media_display, api, media_context);
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
