/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

#[cfg(any(
    all(
        target_os = "android",
        any(target_arch = "arm", target_arch = "aarch64")
    ),
    target_arch = "x86_64",
    target_arch = "aarch64",
))]
mod platform {
    pub use servo_media_gstreamer::GStreamerBackend as Backend;
}

#[cfg(not(any(
    all(
        target_os = "android",
        any(target_arch = "arm", target_arch = "aarch64")
    ),
    target_arch = "x86_64",
    target_arch = "aarch64",
)))]
mod platform {
    pub use servo_media_dummy::DummyBackend as Backend;
}

pub type Backend = platform::Backend;
