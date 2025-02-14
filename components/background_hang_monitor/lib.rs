/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

#![deny(unsafe_code)]

pub mod background_hang_monitor;
mod sampler;
#[cfg(all(
    feature = "sampler",
    target_os = "linux",
    not(any(
        target_arch = "arm",
        target_arch = "aarch64",
        target_env = "ohos",
        target_env = "musl"
    ))
))]
mod sampler_linux;
#[cfg(all(feature = "sampler", target_os = "macos"))]
mod sampler_mac;
#[cfg(all(feature = "sampler", target_os = "windows"))]
mod sampler_windows;

pub use self::background_hang_monitor::*;
