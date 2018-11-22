/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use crate::sampler::{NativeStack, Sampler};
use libc;

type MonitoredThreadId = libc::pthread_t;

#[allow(dead_code)]
pub struct LinuxSampler {
    thread_id: MonitoredThreadId,
}

impl LinuxSampler {
    #[allow(unsafe_code, dead_code)]
    pub fn new() -> Box<Sampler> {
        let thread_id = unsafe { libc::pthread_self() };
        Box::new(LinuxSampler { thread_id })
    }
}

impl Sampler for LinuxSampler {
    #[allow(unsafe_code)]
    fn suspend_and_sample_thread(&self) -> Result<NativeStack, ()> {
        // Warning: The "critical section" begins here.
        // In the critical section:
        // we must not do any dynamic memory allocation,
        // nor try to acquire any lock
        // or any other unshareable resource.

        // NOTE: End of "critical section".
        Err(())
    }
}
