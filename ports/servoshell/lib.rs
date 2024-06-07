/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

// For Android, see /support/android/apk/ + /ports/jniapi/.
#![cfg(not(target_os = "android"))]

#[cfg(any(target_os = "macos", target_os = "linux"))]
#[macro_use]
extern crate sig;

#[cfg(test)]
mod test;

mod backtrace;
mod crash_handler;
#[cfg(not(any(target_os = "android", target_env = "ohos")))]
pub(crate) mod desktop;
mod panic_hook;
mod parser;
mod prefs;
mod resources;

pub mod platform {
    #[cfg(target_os = "macos")]
    pub use crate::platform::macos::deinit;

    #[cfg(target_os = "macos")]
    pub mod macos;

    #[cfg(not(target_os = "macos"))]
    pub fn deinit(_clean_shutdown: bool) {}
}

#[cfg(not(any(target_os = "android", target_env = "ohos")))]
pub fn main() {
    desktop::desktop_main::main()
}

#[cfg(target_os = "android")]
pub fn main() {
    println!(
        "Cannot start /ports/servoshell/ on Android. \
                Use /support/android/apk/ + /ports/jniapi/ instead"
    );
}

#[cfg(target_env = "ohos")]
pub fn main() {
    println!("You shouldn't start /ports/servoshell/ on OpenHarmony.");
}

pub fn servo_version() -> String {
    format!(
        "Servo {}-{}",
        env!("CARGO_PKG_VERSION"),
        env!("VERGEN_GIT_SHA")
    )
}
