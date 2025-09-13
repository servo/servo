/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

//! This module contains APIs for the `profile` crate used generically in the
//! rest of Servo. These APIs are here instead of in `profile` so that these
//! modules won't have to depend on `profile`.

#![deny(unsafe_code)]

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
        let span = tracing::info_span!(
            $category.variant_name(),
            servo_profiling = true,
            url = meta.as_ref().map(|m| m.url.clone()),
        );
        #[cfg(not(feature = "tracing"))]
        let span = ();
        $crate::time::profile($category, meta, $profiler_chan, span, $($callback)+)
    }};
}

/// Constructs a span at the trace level and immediately enters it.
///
/// Attention: This macro requires the user crate to have a `tracing` feature,
/// which can be used to disable the effects of this macro.
/// This instruments code to measure the time between here and the end of
/// the current scope.
/// This macro is intended for performance measurement purposes and may
/// use a different mechanism to record spans in the future.
#[macro_export]
macro_rules! trace_span {
    ($span_name:literal, $($field:tt)*) => {
        #[cfg(feature = "tracing")]
        let _servo_profile_span = ::profile_traits::servo_tracing::trace_span!(
            $span_name,
            servo_profiling = true,
            $($field)*
        ).entered();
    };
    ($span_name:literal) => {
        #[cfg(feature = "tracing")]
        let _servo_profile_span = ::profile_traits::servo_tracing::trace_span!(
            $span_name,
            servo_profiling = true,
        ).entered();
    };
}

#[cfg(feature = "tracing")]
pub use tracing as servo_tracing;
