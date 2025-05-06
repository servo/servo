/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */
use std::cmp::Ord;
use std::sync::LazyLock;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::thread;

use tokio::runtime::{Builder, Runtime};

pub static HANDLE: LazyLock<Runtime> = LazyLock::new(|| {
    Builder::new_multi_thread()
        .thread_name_fn(|| {
            static ATOMIC_ID: AtomicUsize = AtomicUsize::new(0);
            let id = ATOMIC_ID.fetch_add(1, Ordering::Relaxed);
            format!("tokio-runtime-{}", id)
        })
        .worker_threads(
            thread::available_parallelism()
                .map(|i| i.get())
                .unwrap_or(servo_config::pref!(threadpools_fallback_worker_num) as usize)
                .min(servo_config::pref!(threadpools_async_runtime_workers_max).max(1) as usize),
        )
        .enable_io()
        .enable_time()
        .build()
        .expect("Unable to build tokio-runtime runtime")
});
