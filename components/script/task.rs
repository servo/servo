/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

//! Machinery for [tasks](https://html.spec.whatwg.org/multipage/#concept-task).

use std::fmt;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};

macro_rules! task {
    ($name:ident: move || $body:tt) => {{
        #[allow(non_camel_case_types)]
        struct $name<F>(F);
        impl<F> ::task::TaskOnce for $name<F>
        where
            F: ::std::ops::FnOnce() + Send,
        {
            fn name(&self) -> &'static str {
                stringify!($name)
            }

            fn run_once(self) {
                (self.0)();
            }
        }
        $name(move || $body)
    }};
}

/// A task that can be run. The name method is for profiling purposes.
pub trait TaskOnce: Send {
    #[allow(unsafe_code)]
    fn name(&self) -> &'static str {
        #[cfg(feature = "unstable")]
        unsafe { ::std::intrinsics::type_name::<Self>() }
        #[cfg(not(feature = "unstable"))]
        { "(task name unknown)" }
    }

    fn run_once(self);
}

/// A boxed version of `TaskOnce`.
pub trait TaskBox: Send {
    fn name(&self) -> &'static str;

    fn run_box(self: Box<Self>);
}

impl<T> TaskBox for T
where
    T: TaskOnce,
{
    fn name(&self) -> &'static str {
        TaskOnce::name(self)
    }

    fn run_box(self: Box<Self>) {
        self.run_once()
    }
}

impl fmt::Debug for TaskBox {
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
    pub fn wrap_task<T>(&self, task: T) -> impl TaskOnce
    where
        T: TaskOnce,
    {
        CancellableTask {
            cancelled: self.cancelled.clone(),
            inner: task,
        }
    }
}

/// A task that can be cancelled by toggling a shared flag.
pub struct CancellableTask<T: TaskOnce> {
    cancelled: Option<Arc<AtomicBool>>,
    inner: T,
}

impl<T> CancellableTask<T>
where
    T: TaskOnce,
{
    fn is_cancelled(&self) -> bool {
        self.cancelled.as_ref().map_or(false, |cancelled| {
            cancelled.load(Ordering::SeqCst)
        })
    }
}

impl<T> TaskOnce for CancellableTask<T>
where
    T: TaskOnce,
{
    fn name(&self) -> &'static str {
        self.inner.name()
    }

    fn run_once(self) {
        if !self.is_cancelled() {
            self.inner.run_once()
        }
    }
}
