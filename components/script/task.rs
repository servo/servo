/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

//! Machinery for [tasks](https://html.spec.whatwg.org/multipage/#concept-task).

use std::fmt;
use std::intrinsics;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};

macro_rules! task {
    ($name:ident: move || $body:tt) => {{
        #[allow(non_camel_case_types)]
        struct $name<F>(F);
        impl<F> ::task::Task for $name<F>
        where
            F: ::std::ops::FnOnce(),
        {
            fn name(&self) -> &'static str {
                stringify!($name)
            }

            fn run(self: Box<Self>) {
                (self.0)();
            }
        }
        $name(move || $body)
    }};
}

/// A task that can be run. The name method is for profiling purposes.
pub trait Task {
    #[allow(unsafe_code)]
    fn name(&self) -> &'static str { unsafe { intrinsics::type_name::<Self>() } }
    fn run(self: Box<Self>);
}

impl fmt::Debug for Task + Send {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        fmt.debug_tuple(self.name()).field(&format_args!("...")).finish()
    }
}

/// Encapsulated state required to create cancellable tasks from non-script threads.
pub struct TaskCanceller {
    pub cancelled: Option<Arc<AtomicBool>>,
}

impl TaskCanceller {
    /// Returns a wrapped `task` that will be cancelled if the `TaskCanceller`
    /// says so.
    pub fn wrap_task<T>(&self, task: Box<T>) -> Box<Task + Send>
    where
        T: Send + Task + 'static,
    {
        box CancellableTask {
            cancelled: self.cancelled.clone(),
            inner: task,
        }
    }
}

/// A task that can be cancelled by toggling a shared flag.
pub struct CancellableTask<T: Send + Task> {
    cancelled: Option<Arc<AtomicBool>>,
    inner: Box<T>,
}

impl<T> CancellableTask<T>
where
    T: Send + Task,
{
    fn is_cancelled(&self) -> bool {
        self.cancelled.as_ref().map_or(false, |cancelled| {
            cancelled.load(Ordering::SeqCst)
        })
    }
}

impl<T> Task for CancellableTask<T>
where
    T: Send + Task,
{
    fn name(&self) -> &'static str {
        self.inner.name()
    }

    fn run(self: Box<Self>) {
        if !self.is_cancelled() {
            self.inner.run()
        }
    }
}
