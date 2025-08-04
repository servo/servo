/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */
use std::sync::OnceLock;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::thread;
use std::time::Duration;

use futures::Future;
use net_traits::AsyncRuntime;
use tokio::runtime::{Builder, Handle, Runtime};

/// The actual runtime,
/// to be used as part of shut-down.
pub struct AsyncRuntimeHolder {
    runtime: Option<Runtime>,
}

impl AsyncRuntimeHolder {
    pub(crate) fn new(runtime: Runtime) -> Self {
        Self {
            runtime: Some(runtime),
        }
    }
}

impl AsyncRuntime for AsyncRuntimeHolder {
    fn shutdown(&mut self) {
        self.runtime
            .take()
            .expect("Runtime should have been initialized on start-up.")
            .shutdown_timeout(Duration::from_millis(100))
    }
}

/// A shared handle to the runtime,
/// to be initialized on start-up.
static ASYNC_RUNTIME_HANDLE: OnceLock<Handle> = OnceLock::new();

pub fn init_async_runtime() -> Box<dyn AsyncRuntime> {
    // Initialize a tokio runtime.
    let runtime = Builder::new_multi_thread()
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
        .expect("Unable to build tokio-runtime runtime");

    // Make the runtime available to users inside this crate.
    ASYNC_RUNTIME_HANDLE
        .set(runtime.handle().clone())
        .expect("Runtime handle should be initialized once on start-up");

    // Return an async runtime for use in shutdown.
    Box::new(AsyncRuntimeHolder::new(runtime))
}

/// Spawn a task using the handle to the runtime.
pub fn spawn_task<F>(task: F)
where
    F: Future + 'static + std::marker::Send,
    F::Output: Send + 'static,
{
    ASYNC_RUNTIME_HANDLE
        .get()
        .expect("Runtime handle should be initialized on start-up")
        .spawn(task);
}

/// Spawn a blocking task using the handle to the runtime.
pub fn spawn_blocking_task<F, R>(task: F) -> F::Output
where
    F: Future,
{
    ASYNC_RUNTIME_HANDLE
        .get()
        .expect("Runtime handle should be initialized on start-up")
        .block_on(task)
}
