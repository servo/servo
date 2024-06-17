/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

//! Data and main loop of WGPU poll thread.
//!
//! This is roughly based on <https://github.com/LucentFlux/wgpu-async/blob/1322c7e3fcdfc1865a472c7bbbf0e2e06dcf4da8/src/wgpu_future.rs>

use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
use std::sync::{Arc, Mutex, MutexGuard};
use std::thread::JoinHandle;

use log::warn;

use crate::wgc::global::Global;

/// Polls devices while there is something to poll.
///
/// This objects corresponds to a thread that parks itself when there is no work,
/// waiting on it, and then calls `poll_all_devices` repeatedly to block.
///
/// The thread dies when this object is dropped, and all work in submission is done.
///
/// ## Example
/// ```no_run
/// let token = self.poller.token(); // create a new token
/// let callback = SubmittedWorkDoneClosure::from_rust(Box::from(move || {
///    drop(token); // drop token as closure has been fired
///    // ...
/// }));
/// let result = gfx_select!(queue_id => global.queue_on_submitted_work_done(queue_id, callback));
/// self.poller.wake(); // wake poller thread to actually poll
/// ```
#[derive(Debug)]
pub(crate) struct Poller {
    /// The number of closures that still needs to be fired.
    /// When this is 0, the thread can park itself.
    work_count: Arc<AtomicUsize>,
    /// True if thread should die after all work in submission is done
    is_done: Arc<AtomicBool>,
    /// Handle to the WGPU poller thread (to be used for unparking the thread)
    handle: Option<JoinHandle<()>>,
    /// Lock for device maintain calls (in poll_all_devices and queue_submit)
    ///
    /// This is workaround for wgpu deadlocks: https://github.com/gfx-rs/wgpu/issues/5572
    lock: Arc<Mutex<()>>,
}

#[inline]
fn poll_all_devices(
    global: &Arc<Global>,
    more_work: &mut bool,
    force_wait: bool,
    lock: &Mutex<()>,
) {
    let _guard = lock.lock().unwrap();
    match global.poll_all_devices(force_wait) {
        Ok(all_queue_empty) => *more_work = !all_queue_empty,
        Err(e) => warn!("Poller thread got `{e}` on poll_all_devices."),
    }
    // drop guard
}

impl Poller {
    pub(crate) fn new(global: Arc<Global>) -> Self {
        let work_count = Arc::new(AtomicUsize::new(0));
        let is_done = Arc::new(AtomicBool::new(false));
        let work = work_count.clone();
        let done = is_done.clone();
        let lock = Arc::new(Mutex::new(()));
        Self {
            work_count,
            is_done,
            lock: Arc::clone(&lock),
            handle: Some(
                std::thread::Builder::new()
                    .name("WGPU poller".into())
                    .spawn(move || {
                        while !done.load(Ordering::Acquire) {
                            let mut more_work = false;
                            // Do non-blocking poll unconditionally
                            // so every `ẁake` (even spurious) will do at least one poll.
                            // this is mostly useful for stuff that is deferred
                            // to maintain calls in wgpu (device resource destruction)
                            poll_all_devices(&global, &mut more_work, false, &lock);
                            while more_work || work.load(Ordering::Acquire) != 0 {
                                poll_all_devices(&global, &mut more_work, true, &lock);
                            }
                            std::thread::park(); //TODO: should we use timeout here
                        }
                    })
                    .expect("Spawning thread should not fail"),
            ),
        }
    }

    /// Creates a token of work
    pub(crate) fn token(&self) -> WorkToken {
        let prev = self.work_count.fetch_add(1, Ordering::AcqRel);
        debug_assert!(
            prev < usize::MAX,
            "cannot have more than `usize::MAX` outstanding operations on the GPU"
        );
        WorkToken {
            work_count: Arc::clone(&self.work_count),
        }
    }

    /// Wakes the poller thread to start polling.
    pub(crate) fn wake(&self) {
        self.handle
            .as_ref()
            .expect("Poller thread does not exist!")
            .thread()
            .unpark();
    }

    /// Lock for device maintain calls (in poll_all_devices and queue_submit)
    pub(crate) fn lock(&self) -> MutexGuard<()> {
        self.lock.lock().unwrap()
    }
}

impl Drop for Poller {
    fn drop(&mut self) {
        self.is_done.store(true, Ordering::Release);

        let handle = self.handle.take().expect("Poller dropped twice");
        handle.thread().unpark();
        handle.join().expect("Poller thread panicked");
    }
}

/// RAII indicating that there is some work enqueued (closure to be fired),
/// while this token is held.
pub(crate) struct WorkToken {
    work_count: Arc<AtomicUsize>,
}

impl Drop for WorkToken {
    fn drop(&mut self) {
        self.work_count.fetch_sub(1, Ordering::AcqRel);
    }
}
