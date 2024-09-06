/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use cfg_if::cfg_if;

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

        #[cfg(feature = "tracing-hitrace")]
        let subscriber = {
            use tracing_subscriber::layer::SubscriberExt;
            // Set up a HitraceLayer for performance tracing.
            subscriber.with(HitraceLayer::default())
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

/// Plumbs tracing spans into HiTrace, with the following caveats:
///
/// - We ignore spans unless they have a `servo_profiling` field.
/// - We map span entry ([`Layer::on_enter`]) to `OH_HiTrace_StartTrace(metadata.name())`.
/// - We map span exit ([`Layer::on_exit`]) to `OH_HiTrace_FinishTrace()`.
///
/// As a result, within each thread, spans must exit in reverse order of their entry, otherwise the
/// resultant profiling data will be incorrect (see the section below). This is not necessarily the
/// case for tracing spans, since there can be multiple [trace trees], so we check that this
/// invariant is upheld when debug assertions are enabled, logging errors if it is violated.
///
/// [trace trees]: https://docs.rs/tracing/0.1.40/tracing/span/index.html#span-relationships
///
/// # Uniquely identifying spans
///
/// We need to ensure that the start and end points of one span are not mixed up with other spans.
/// For now, we use the HiTrace [synchronous API], which restricts how spans must behave.
///
/// In the HiTrace [synchronous API], spans must have stack-like behaviour, because spans are keyed
/// entirely on their *name* string, and OH_HiTrace_FinishTrace always ends the most recent span.
/// While synchronous API spans are thread-local, callers could still violate this invariant with
/// reentrant or asynchronous code.
///
/// In the [asynchronous API], spans are keyed on a (*name*,*taskId*) pair, where *name* is again
/// a string, and *taskId* is an arbitrary [`i32`]. This makes *taskId* a good place for a unique
/// identifier, but asynchronous spans can cross thread boundaries, so the identifier needs to be
/// temporally unique in the whole process.
///
/// Tracing spans have such an identifier ([`Id`]), but they’re [`u64`]-based, and their format
/// is an internal implementation detail of the [`Subscriber`]. For [`Registry`], those values
/// [come from] a [packed representation] of a generation number, thread number, page number, and
/// variable-length index. This makes them hard to compress robustly into an [`i32`].
///
/// If we move to the asynchronous API, we will need to generate our own *taskId* values, perhaps
/// by combining some sort of thread id with a thread-local atomic counter. [`ThreadId`] is opaque
/// in stable Rust, and converts to a [`u64`] in unstable Rust, so we would also need to make our
/// own thread ids, perhaps by having a global atomic counter cached in a thread-local.
///
/// [synchronous API]: https://docs.rs/hitrace-sys/0.1.2/hitrace_sys/fn.OH_HiTrace_StartTrace.html
/// [asynchronous API]: https://docs.rs/hitrace-sys/0.1.2/hitrace_sys/fn.OH_HiTrace_StartAsyncTrace.html
/// [`Registry`]: tracing_subscriber::Registry
/// [come from]: https://docs.rs/tracing-subscriber/0.3.18/src/tracing_subscriber/registry/sharded.rs.html#237-269
/// [packed representation]: https://docs.rs/sharded-slab/0.1.7/sharded_slab/trait.Config.html
/// [`ThreadId`]: std::thread::ThreadId
#[cfg(feature = "tracing-hitrace")]
#[derive(Default)]
struct HitraceLayer {}

cfg_if! {
    if #[cfg(feature = "tracing-hitrace")] {
        use std::cell::RefCell;

        use tracing::span::Id;
        use tracing::Subscriber;
        use tracing_subscriber::Layer;

        #[cfg(debug_assertions)]
        thread_local! {
            /// Stack of span names, to ensure the HiTrace synchronous API is not misused.
            static HITRACE_NAME_STACK: RefCell<Vec<String>> = RefCell::default();
        }

        impl<S: Subscriber + for<'lookup> tracing_subscriber::registry::LookupSpan<'lookup>>
            Layer<S> for HitraceLayer
        {
            fn on_enter(&self, id: &Id, ctx: tracing_subscriber::layer::Context<'_, S>) {
                if let Some(metadata) = ctx.metadata(id) {
                    // TODO: is this expensive? Would extensions be faster?
                    // <https://docs.rs/tracing-subscriber/0.3.18/tracing_subscriber/registry/struct.ExtensionsMut.html>
                    if metadata.fields().field("servo_profiling").is_some() {
                        #[cfg(debug_assertions)]
                        HITRACE_NAME_STACK.with_borrow_mut(|stack|
                            stack.push(metadata.name().to_owned()));

                        hitrace::start_trace(
                            &std::ffi::CString::new(metadata.name())
                                .expect("Failed to convert str to CString"),
                        );
                    }
                }
            }

            fn on_exit(&self, id: &Id, ctx: tracing_subscriber::layer::Context<'_, S>) {
                if let Some(metadata) = ctx.metadata(id) {
                    if metadata.fields().field("servo_profiling").is_some() {
                        hitrace::finish_trace();

                        #[cfg(debug_assertions)]
                        HITRACE_NAME_STACK.with_borrow_mut(|stack| {
                            if stack.last().map(|name| &**name) != Some(metadata.name()) {
                                log::error!(
                                    "Tracing span out of order: {} (stack: {:?})",
                                    metadata.name(),
                                    stack
                                );
                            }
                            if !stack.is_empty() {
                                stack.pop();
                            }
                        });
                    }
                }
            }
        }
    }
}
