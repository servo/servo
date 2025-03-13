/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

//! Machinery for [tasks](https://html.spec.whatwg.org/multipage/#concept-task).

use std::fmt;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};

macro_rules! task {
    ($name:ident: move || $body:tt) => {{
        #[allow(non_camel_case_types)]
        struct $name<F>(F);
        impl<F> crate::task::TaskOnce for $name<F>
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
pub(crate) trait TaskOnce: Send {
    #[allow(unsafe_code)]
    fn name(&self) -> &'static str {
        ::std::any::type_name::<Self>()
    }

    fn run_once(self);
}

/// A boxed version of `TaskOnce`.
pub(crate) trait TaskBox: Send {
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

impl fmt::Debug for dyn TaskBox {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        fmt.debug_tuple(self.name())
            .field(&format_args!("..."))
            .finish()
    }
}

/// Encapsulated state required to create cancellable tasks from non-script threads.
#[derive(Clone, Default, JSTraceable, MallocSizeOf)]
pub(crate) struct TaskCanceller {
    #[ignore_malloc_size_of = "This is difficult, because only one of them should be measured"]
    pub(crate) cancelled: Arc<AtomicBool>,
}

impl TaskCanceller {
    /// Returns a wrapped `task` that will be cancelled if the `TaskCanceller` says so.
    pub(crate) fn wrap_task<T>(&self, task: T) -> impl TaskOnce + use<T>
    where
        T: TaskOnce,
    {
        CancellableTask {
            canceller: self.clone(),
            inner: task,
        }
    }

    pub(crate) fn cancelled(&self) -> bool {
        self.cancelled.load(Ordering::SeqCst)
    }
}

/// A task that can be cancelled by toggling a shared flag.
pub(crate) struct CancellableTask<T: TaskOnce> {
    canceller: TaskCanceller,
    inner: T,
}

impl<T: TaskOnce> TaskOnce for CancellableTask<T> {
    fn name(&self) -> &'static str {
        self.inner.name()
    }

    fn run_once(self) {
        if !self.canceller.cancelled() {
            self.inner.run_once()
        }
    }
}
