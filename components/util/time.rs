/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use std::time::Duration;
use std::{u32, u64};

pub const NANOS_PER_SEC: u32 = 1_000_000_000;

pub fn duration_from_seconds(secs: f64) -> Duration {

    // Get number of seconds and check that it fits in a u64.
    let whole_secs = secs.trunc();
    assert!(whole_secs >= 0.0 && whole_secs <= u64::MAX as f64);

    // Get number of nanoseconds. This should always fit in a u32, but check anyway.
    let nanos = (secs.fract() * (NANOS_PER_SEC as f64)).trunc();
    assert!(nanos >= 0.0 && nanos <= u32::MAX as f64);

    Duration::new(whole_secs as u64, nanos as u32)
}

pub fn duration_from_nanoseconds(nanos: u64) -> Duration {
    // Get number of seconds.
    let secs = nanos / NANOS_PER_SEC as u64;

    // Get number of extra nanoseconds. This should always fit in a u32, but check anyway.
    let subsec_nanos = nanos % NANOS_PER_SEC as u64;
    assert!(subsec_nanos <= u32::MAX as u64);

    Duration::new(secs, subsec_nanos as u32)
}
