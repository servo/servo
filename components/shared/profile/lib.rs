/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

//! This module contains APIs for the `profile` crate used generically in the
//! rest of Servo. These APIs are here instead of in `profile` so that these
//! modules won't have to depend on `profile`.

#![deny(unsafe_code)]

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
