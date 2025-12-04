/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

//! This module contains APIs for the `profile` crate used generically in the
//! rest of Servo. These APIs are here instead of in `profile` so that these
//! modules won't have to depend on `profile`.

#![deny(unsafe_code)]

pub mod generic_callback;
pub mod generic_channel;
pub mod ipc;
pub mod mem;
pub mod time;

/// Measure the given callback with the time profiler and (if enabled) tracing.
///
/// `$category` must be const, because we use it to derive the span name.
#[macro_export]
macro_rules! time_profile {
    ($category:expr, $meta:expr, $profiler_chan:expr, $($callback:tt)+) => {{
        let meta: Option<$crate::time::TimerMetadata> = $meta;
        #[cfg(feature = "tracing")]
        let span = $crate::servo_tracing::info_span!(
            $category.variant_name(),
            servo_profiling = true,
            url = meta.as_ref().map(|m| m.url.clone()),
        );
        #[cfg(not(feature = "tracing"))]
        let span = ();
        $crate::time::profile($category, meta, $profiler_chan, span, $($callback)+)
    }};
}

/// Provides API compatible dummies for the tracing-rs APIs we use
/// if tracing is disabled. Hence, nothing will be traced
pub mod dummy_tracing {
    use std::fmt::Display;
    use std::marker::PhantomData;

    pub struct Span();

    pub struct EnteredSpan(Span);
    pub struct Entered<'a>(PhantomData<&'a Span>);

    impl Span {
        pub fn enter(&self) -> Entered<'_> {
            Entered(PhantomData)
        }

        pub fn entered(self) -> EnteredSpan {
            EnteredSpan(self)
        }

        pub fn in_scope<F: FnOnce() -> T, T>(&self, f: F) -> T {
            // We still need to execute the function, even if tracing is disabled.
            f()
        }
    }

    impl EnteredSpan {}

    impl Entered<'_> {}

    #[derive(Debug)]
    struct Level();

    #[expect(dead_code)]
    impl Level {
        /// The "error" level.
        ///
        /// Designates very serious errors.
        pub const ERROR: Level = Level();
        /// The "warn" level.
        ///
        /// Designates hazardous situations.
        pub const WARN: Level = Level();
        /// The "info" level.
        ///
        /// Designates useful information.
        pub const INFO: Level = Level();
        /// The "debug" level.
        ///
        /// Designates lower priority information.
        pub const DEBUG: Level = Level();
        /// The "trace" level.
        ///
        /// Designates very low priority, often extremely verbose, information.
        pub const TRACE: Level = Level();
        /// Returns the string representation of the `Level`.
        ///
        /// This returns the same string as the `fmt::Display` implementation.
        pub fn as_str(&self) -> &'static str {
            "disabled"
        }
    }
    impl Display for Level {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            write!(f, "{}", self.as_str())
        }
    }
}

/// Constructs a span at the trace level
///
/// This macro creates a Span for the purpose of instrumenting code to measure
/// the execution time of the span.
/// If the `tracing` feature (of the crate using this macro) is disabled, then
/// the Span implementation will be replaced with a dummy, that does not record
/// anything.
///
/// Attention: This macro requires the user crate to have a `tracing` feature,
/// which can be used to disable the effects of this macro.
#[macro_export]
macro_rules! trace_span {
    ($span_name:literal, $($field:tt)*) => {
        {
        #[cfg(feature = "tracing")]
        {
            $crate::servo_tracing::trace_span!(
                $span_name,
                servo_profiling = true,
                $($field)*
            )
        }
        #[cfg(not(feature = "tracing"))]
        { $crate::dummy_tracing::Span() }
        }
    };
    ($span_name:literal) => {
        {
         #[cfg(feature = "tracing")]
         {
             $crate::servo_tracing::trace_span!(
                $span_name,
                servo_profiling = true,
             )
         }
        #[cfg(not(feature = "tracing"))]
        { $crate::dummy_tracing::Span() }
        }
    };
}

/// Constructs a span at the info level
///
/// This macro creates a Span for the purpose of instrumenting code to measure
/// the execution time of the span.
/// If the `tracing` feature (of the crate using this macro) is disabled, then
/// the Span implementation will be replaced with a dummy, that does not record
/// anything.
///
/// Attention: This macro requires the user crate to have a `tracing` feature,
/// which can be used to disable the effects of this macro.
#[macro_export]
macro_rules! info_span {
    ($span_name:literal, $($field:tt)*) => {
        {
        #[cfg(feature = "tracing")]
        {
            $crate::servo_tracing::info_span!(
                $span_name,
                servo_profiling = true,
                $($field)*
            )
        }
        #[cfg(not(feature = "tracing"))]
        { $crate::dummy_tracing::Span() }
        }
    };
    ($span_name:literal) => {
        {
         #[cfg(feature = "tracing")]
         {
             $crate::servo_tracing::info_span!(
                $span_name,
                servo_profiling = true,
             )
         }
        #[cfg(not(feature = "tracing"))]
        { $crate::dummy_tracing::Span() }
        }
    };
}

#[cfg(feature = "tracing")]
pub use tracing as servo_tracing;
