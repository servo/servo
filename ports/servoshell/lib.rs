/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

#[cfg(any(target_os = "macos", target_os = "linux"))]
#[macro_use]
extern crate sig;

#[cfg(test)]
mod test;

#[cfg(not(target_os = "android"))]
mod backtrace;
mod crash_handler;
#[cfg(not(any(target_os = "android", target_env = "ohos")))]
pub(crate) mod desktop;
#[cfg(any(target_os = "android", target_env = "ohos"))]
mod egl;
#[cfg(not(target_os = "android"))]
mod panic_hook;
mod parser;
mod prefs;
#[cfg(not(any(target_os = "android", target_env = "ohos")))]
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
    desktop::cli::main()
}

pub fn init_tracing() {
    #[cfg(feature = "tracing")]
    {
        let subscriber = tracing_subscriber::registry();

        #[cfg(feature = "tracing-perfetto")]
        let subscriber = {
            use tracing_subscriber::layer::SubscriberExt;
            // Set up a PerfettoLayer for performance tracing.
            // The servo.pftrace file can be uploaded to https://ui.perfetto.dev for analysis.
            let file = std::fs::File::create("servo.pftrace").unwrap();
            let perfetto_layer = tracing_perfetto::PerfettoLayer::new(std::sync::Mutex::new(file));
            subscriber.with(perfetto_layer)
        };

        // Same as SubscriberInitExt::init, but avoids initialising the tracing-log compat layer,
        // since it would break Servo’s FromScriptLogger and FromCompositorLogger.
        // <https://docs.rs/tracing-subscriber/0.3.18/tracing_subscriber/util/trait.SubscriberInitExt.html#method.init>
        // <https://docs.rs/tracing/0.1.40/tracing/#consuming-log-records>
        tracing::subscriber::set_global_default(subscriber)
            .expect("Failed to set tracing subscriber");
    }
}

pub fn servo_version() -> String {
    format!(
        "Servo {}-{}",
        env!("CARGO_PKG_VERSION"),
        env!("VERGEN_GIT_SHA")
    )
}
