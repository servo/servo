/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */
use std::sync::OnceLock;
use std::time::Duration;

use futures::Future;
use net_traits::AsyncRuntime;
use tokio::runtime::{Handle, Runtime};


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
pub static HANDLE: OnceLock<Handle> = OnceLock::new();

/// Spawn a task using the handle to the runtime.
pub fn spawn_task<F>(task: F)
where
    F: Future + 'static + std::marker::Send,
    F::Output: Send + 'static,
{
    HANDLE
        .get()
        .expect("Runtime handle should be initialized on start-up")
        .spawn(task);
}
