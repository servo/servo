/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

#![deny(missing_docs)]

//! A timeline module, used to specify an `AnimationTimeline` which determines
//! the time used for synchronizing animations in the script thread.

use std::time::{SystemTime, UNIX_EPOCH};

use jstraceable_derive::JSTraceable;

/// A `AnimationTimeline` which is used to synchronize animations during the script
/// event loop.
#[derive(Clone, Copy, Debug, JSTraceable, MallocSizeOf)]
pub(crate) struct AnimationTimeline {
    current_value: f64,
}

impl AnimationTimeline {
    /// Creates a new "normal" timeline, i.e., a "Current" mode timer.
    #[inline]
    pub fn new() -> Self {
        Self {
            current_value: SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs_f64(),
        }
    }

    /// Creates a new "test mode" timeline, with initial time 0.
    #[inline]
    pub fn new_for_testing() -> Self {
        Self { current_value: 0. }
    }

    /// Returns the current value of the timeline in seconds.
    pub fn current_value(&self) -> f64 {
        self.current_value
    }

    /// Updates the value of the `AnimationTimeline` to the current clock time.
    pub fn update(&mut self) {
        self.current_value = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs_f64();
    }

    /// Increments the current value of the timeline by a specific number of seconds.
    /// This is used for testing.
    pub fn advance_specific(&mut self, by: f64) {
        self.current_value += by;
    }
}
