/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use crate::sampler::{NativeStack, Sampler};

type MonitoredThreadId = usize; // TODO: use the `windows` crate to do this.

#[allow(dead_code)]
pub struct WindowsSampler {
    thread_id: MonitoredThreadId,
}

impl WindowsSampler {
    #[allow(unsafe_code, dead_code)]
    pub fn new_boxed() -> Box<dyn Sampler> {
        let thread_id = 0; // TODO: use windows::Win32::System::Threading::GetThreadId
        Box::new(WindowsSampler { thread_id })
    }
}

impl Sampler for WindowsSampler {
    fn suspend_and_sample_thread(&self) -> Result<NativeStack, ()> {
        // Warning: The "critical section" begins here.
        // In the critical section:
        // we must not do any dynamic memory allocation,
        // nor try to acquire any lock
        // or any other unshareable resource.

        // TODO:
        // 1: use windows::Win32::Threading::SuspendThread
        // 2: use windows::Win32::Threading::GetThreadContext
        // 3: populate registers using the context, see
        // https://dxr.mozilla.org/mozilla-central/source/tools/profiler/core/platform-win32.cpp#129
        // 4: use windows::Win32::Threading::ResumeThread

        // NOTE: End of "critical section".
        Err(())
    }
}
