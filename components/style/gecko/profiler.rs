/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

//! Gecko profiler support.
//!
//! Use the `profiler_label!` macro from macros.rs.

use crate::gecko_bindings::structs;

/// A label describing a category of work that style threads can perform.
pub enum ProfilerLabel {
    /// Style computation.
    Style,
    /// Style sheet parsing.
    Parse,
}

/// RAII object that constructs and destroys a C++ AutoProfilerLabel object
/// pointed to be the specified reference.
#[cfg(feature = "gecko_profiler")]
pub struct AutoProfilerLabel<'a>(&'a mut structs::AutoProfilerLabel);

#[cfg(feature = "gecko_profiler")]
impl<'a> AutoProfilerLabel<'a> {
    /// Creates a new AutoProfilerLabel with the specified label type.
    ///
    /// unsafe since the caller must ensure that `label` is allocated on the
    /// stack.
    #[inline]
    pub unsafe fn new(
        label: &mut structs::AutoProfilerLabel,
        label_type: ProfilerLabel,
    ) -> AutoProfilerLabel {
        let category_pair = match label_type {
            ProfilerLabel::Style => structs::JS::ProfilingCategoryPair_LAYOUT_StyleComputation,
            ProfilerLabel::Parse => structs::JS::ProfilingCategoryPair_LAYOUT_CSSParsing,
        };
        structs::Gecko_Construct_AutoProfilerLabel(label, category_pair);
        AutoProfilerLabel(label)
    }
}

#[cfg(feature = "gecko_profiler")]
impl<'a> Drop for AutoProfilerLabel<'a> {
    #[inline]
    fn drop(&mut self) {
        unsafe {
            structs::Gecko_Destroy_AutoProfilerLabel(self.0);
        }
    }
}

/// Whether the Gecko profiler is currently active.
///
/// This implementation must be kept in sync with
/// `mozilla::profiler::detail::RacyFeatures::IsActive`.
#[cfg(feature = "gecko_profiler")]
#[inline]
pub fn profiler_is_active() -> bool {
    use self::structs::profiler::detail;
    use std::mem;
    use std::sync::atomic::{AtomicU32, Ordering};

    let active_and_features: &AtomicU32 = unsafe {
        mem::transmute(&detail::RacyFeatures_sActiveAndFeatures)
    };
    (active_and_features.load(Ordering::Relaxed) & detail::RacyFeatures_Active) != 0
}

/// Always false when the Gecko profiler is disabled.
#[cfg(not(feature = "gecko_profiler"))]
#[inline]
pub fn profiler_is_active() -> bool {
    false
}
