/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use cfg_if::cfg_if;

#[cfg(test)]
mod test;

#[cfg(not(target_os = "android"))]
mod backtrace;
#[cfg(not(target_env = "ohos"))]
mod crash_handler;
#[cfg(not(any(target_os = "android", target_env = "ohos")))]
pub(crate) mod desktop;
#[cfg(any(target_os = "android", target_env = "ohos"))]
mod egl;
#[cfg(not(any(target_os = "android", target_env = "ohos")))]
mod panic_hook;
mod parser;
mod prefs;
#[cfg(not(any(target_os = "android", target_env = "ohos")))]
mod resources;
mod running_app_state;
mod webdriver;
mod window;

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

pub fn init_crypto() {
    rustls::crypto::aws_lc_rs::default_provider()
        .install_default()
        .expect("Error initializing crypto provider");
}

pub fn init_tracing(filter_directives: Option<&str>) {
    #[cfg(not(feature = "tracing"))]
    {
        if filter_directives.is_some() {
            log::debug!("The tracing feature was not selected - ignoring trace filter directives");
        }
    }
    #[cfg(feature = "tracing")]
    {
        use tracing_subscriber::layer::SubscriberExt;
        let subscriber = tracing_subscriber::registry();

        #[cfg(feature = "tracing-perfetto")]
        let subscriber = {
            // Set up a PerfettoLayer for performance tracing.
            // The servo.pftrace file can be uploaded to https://ui.perfetto.dev for analysis.
            let file = std::fs::File::create("servo.pftrace").unwrap();
            let perfetto_layer = tracing_perfetto::PerfettoLayer::new(std::sync::Mutex::new(file))
                .with_filter_by_marker(|field_name| field_name == "servo_profiling")
                .with_debug_annotations(true);
            subscriber.with(perfetto_layer)
        };

        #[cfg(feature = "tracing-hitrace")]
        let subscriber = {
            // Set up a HitraceLayer for performance tracing.
            subscriber.with(HitraceLayer::default())
        };

        // Filter events and spans by the directives in SERVO_TRACING, using EnvFilter as a global filter.
        // <https://docs.rs/tracing-subscriber/0.3.18/tracing_subscriber/layer/index.html#global-filtering>
        let filter_builder = tracing_subscriber::EnvFilter::builder()
            .with_default_directive(tracing::level_filters::LevelFilter::OFF.into());
        let filter = if let Some(filters) = &filter_directives {
            filter_builder.parse_lossy(filters)
        } else {
            filter_builder
                .with_env_var("SERVO_TRACING")
                .from_env_lossy()
        };

        let subscriber = subscriber.with(filter);

        // Same as SubscriberInitExt::init, but avoids initialising the tracing-log compat layer,
        // since it would break Servo’s FromScriptLogger and FromEmbederLogger.
        // <https://docs.rs/tracing-subscriber/0.3.18/tracing_subscriber/util/trait.SubscriberInitExt.html#method.init>
        // <https://docs.rs/tracing/0.1.40/tracing/#consuming-log-records>
        tracing::subscriber::set_global_default(subscriber)
            .expect("Failed to set tracing subscriber");

        // Capture a first event, including the explicit wallclock time.
        // The event itself is useful when investigating startup time.
        // `wallclock_ns` allows us to ground the time, so we can compare
        // against an external timestamp from before starting servoshell.
        // In practice, the perfetto timestamp seems to be the same, but
        // we shouldn't assume this, since different backends may behave differently.
        servo::profile_traits::info_event!(
            "servoshell::startup_tracing_initialized",
            wallclock_ns = std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .map(|duration| duration.as_nanos() as u64)
                .unwrap_or(0)
        );
    }
}

pub const VERSION: &str = concat!("Servo ", env!("CARGO_PKG_VERSION"), "-", env!("GIT_SHA"));

/// Plumbs tracing spans into HiTrace, with the following caveats:
///
/// - We ignore spans unless they have a `servo_profiling` field.
/// - We map span entry ([`Layer::on_enter`]) to `OH_HiTrace_StartTraceEx(metadata.name(), fields)`.
/// - We map span exit ([`Layer::on_exit`]) to `OH_HiTrace_FinishTraceEx()`.
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
/// [synchronous API]: https://docs.rs/hitrace-sys/0.1.9/hitrace_sys/fn.OH_HiTrace_StartTraceEx.html
/// [asynchronous API]: https://docs.rs/hitrace-sys/0.1.9/hitrace_sys/fn.OH_HiTrace_StartAsyncTraceEx.html
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
        use std::fmt;
        use std::fmt::Write;

        use tracing::field::{Field, Visit};
        use tracing::Level;
        use tracing::span::Id;
        use tracing::Subscriber;
        use tracing_subscriber::Layer;

        #[derive(Default)]
        struct HitraceFields(String);

        impl HitraceFields {
            fn record_value(&mut self, field: &Field, value: &dyn fmt::Debug) {
                if field.name() == "servo_profiling" {
                    return;
                }

                if !self.0.is_empty() {
                    self.0.push(',');
                }
                write!(&mut self.0, "{}={value:?}", field.name())
                    .expect("Writing to a String should never fail");
            }
        }

        impl Visit for HitraceFields {
            fn record_debug(&mut self, field: &Field, value: &dyn fmt::Debug) {
                self.record_value(field, value);
            }
        }

        #[cfg(debug_assertions)]
        thread_local! {
            /// Stack of span names, to ensure the HiTrace synchronous API is not misused.
            static HITRACE_NAME_STACK: RefCell<Vec<String>> = RefCell::default();
        }

        /// Map a tracing level to a HiTrace level.
        ///
        /// The log-level of hitrace itself can only be configured globally,
        /// which drastically increases the amount of data that needs to be saved.
        /// For our purposes, the tracing-rs internal filtering is sufficient,
        /// and we don't need additional `debug` log level on the hitrace side,
        /// so we just map anything below INFO to INFO.
        fn convert_level(tracing_level: Level) -> hitrace::api_19::HiTraceOutputLevel {
            if tracing_level < Level::INFO {
                Level::INFO.into()
            } else {
                tracing_level.into()
            }
        }

        impl<S: Subscriber + for<'lookup> tracing_subscriber::registry::LookupSpan<'lookup>>
            Layer<S> for HitraceLayer
        {
            fn on_new_span(
                &self,
                attrs: &tracing::span::Attributes<'_>,
                id: &Id,
                ctx: tracing_subscriber::layer::Context<'_, S>,
            ) {
                if attrs.metadata().fields().field("servo_profiling").is_none() {
                    return;
                }

                let Some(span) = ctx.span(id) else {
                    return;
                };

                let mut fields = HitraceFields::default();
                attrs.record(&mut fields);
                span.extensions_mut().insert(fields);
            }

            fn on_record(
                &self,
                id: &Id,
                values: &tracing::span::Record<'_>,
                ctx: tracing_subscriber::layer::Context<'_, S>,
            ) {
                let Some(span) = ctx.span(id) else {
                    return;
                };

                let mut extensions = span.extensions_mut();
                let Some(fields) = extensions.get_mut::<HitraceFields>() else {
                    return;
                };

                values.record(fields);
            }

            fn on_enter(&self, id: &Id, ctx: tracing_subscriber::layer::Context<'_, S>) {
                let Some(span) = ctx.span(id) else {
                    return;
                };
                let extensions = span.extensions();
                // The HitraceFields extension being present implies `servo_profiling` was set.
                let Some(fields) = extensions.get::<HitraceFields>() else {
                    return;
                };
                // We can't take the String out of the extensions, so
                //  unfortunately, this won't reuse the allocation.
                let custom_args = std::ffi::CString::new(fields.0.as_str())
                        .expect("Failed to convert to CString");

                let metadata = span.metadata();
                let level = convert_level(*metadata.level());
                let name = metadata.name();

                #[cfg(debug_assertions)]
                HITRACE_NAME_STACK.with_borrow_mut(|stack|
                    stack.push(name.to_owned()));

                hitrace::start_trace_ex(
                    level,
                    &std::ffi::CString::new(name)
                        .expect("Failed to convert str to CString"),
                    &custom_args,
                );

            }

            fn on_event(&self, event: &tracing::Event<'_>, _ctx: tracing_subscriber::layer::Context<'_, S>) {
                let mut fields = HitraceFields::default();
                event.record(&mut fields);
                let metadata = event.metadata();
                let level = convert_level(*metadata.level());

                hitrace::start_trace_ex(
                    level,
                    &std::ffi::CString::new(metadata.name())
                        .expect("Failed to convert str to CString"),
                    &std::ffi::CString::new(fields.0)
                        .expect("Failed to convert str to CString"),
                );

                hitrace::finish_trace_ex(level);
            }


            fn on_exit(&self, id: &Id, ctx: tracing_subscriber::layer::Context<'_, S>) {
                let Some(span) = ctx.span(id) else {
                    return;
                };
                if span.extensions().get::<HitraceFields>().is_none() {
                    return;
                }

                let level = convert_level(*span.metadata().level());
                hitrace::finish_trace_ex(level);

                #[cfg(debug_assertions)]
                HITRACE_NAME_STACK.with_borrow_mut(|stack| {
                    let metadata = span.metadata();
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
