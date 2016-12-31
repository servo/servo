/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

#![deny(missing_docs)]

//! A timer module, used to define a `Timer` type, that is controlled by script.

use time;

/// The `TimerMode` is used to determine what time should the `Timer` return.
#[derive(Debug, Clone)]
enum TimerMode {
    /// The timer should return a fixed value.
    Test(f64),
    /// The timer should return the actual time.
    Current,
}

/// A `Timer` struct that takes care of giving the current time for animations.
///
/// This is needed to be allowed to hook the time in the animations' test-mode.
#[derive(Debug, Clone)]
pub struct Timer {
    mode: TimerMode,
}

impl Timer {
    /// Creates a new "normal" timer, i.e., a "Current" mode timer.
    #[inline]
    pub fn new() -> Self {
        Timer {
            mode: TimerMode::Current,
        }
    }

    /// Creates a new "test mode" timer, with initial time 0.
    #[inline]
    pub fn test_mode() -> Self {
        Timer {
            mode: TimerMode::Test(0.),
        }
    }

    /// Returns the current time, at least from the caller's perspective. In
    /// test mode returns whatever the value is.
    pub fn seconds(&self) -> f64 {
        match self.mode {
            TimerMode::Test(test_value) => test_value,
            TimerMode::Current => time::precise_time_s(),
        }
    }

    /// Increments the current clock. Panics if the clock is not on test mode.
    pub fn increment(&mut self, by: f64) {
        match self.mode {
            TimerMode::Test(ref mut val)
                => *val += by,
            TimerMode::Current
                => panic!("Timer::increment called for a non-test mode timer. This is a bug."),
        }
    }
}
