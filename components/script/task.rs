/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

//! Machinery for [tasks](https://html.spec.whatwg.org/multipage/#concept-task).

use std::fmt;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};

macro_rules! task {
    ($name:ident: |$($field:ident: $field_type:ty$(,)*)*| $body:tt) => {{
        #[allow(non_camel_case_types)]
        struct $name<F> {
            $($field: $field_type,)*
            task: F,
        }
        #[expect(unsafe_code)]
        unsafe impl<F> crate::JSTraceable for $name<F> {
            #[expect(unsafe_code)]
            unsafe fn trace(&self, tracer: *mut ::js::jsapi::JSTracer) {
                unsafe { $(self.$field.trace(tracer);)* }
                // We cannot trace the actual task closure. This is safe because
                // all referenced values from within the closure are either borrowed
                // or moved into fields in the struct (and therefore traced).
            }
        }
        impl<F> crate::task::NonSendTaskOnce for $name<F>
        where
            F: ::std::ops::FnOnce($($field_type,)*),
        {
            fn run_once(self, _cx: &mut js::context::JSContext) {
                (self.task)($(self.$field,)*);
            }
        }
        $name {
            $($field,)*
            task: |$($field: $field_type,)*| $body,
        }
    }};

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

            fn run_once(self, _cx: &mut js::context::JSContext) {
                (self.0)();
            }
        }
        $name(move || $body)
    }};
}

/// A task that can be sent between threads and run.
/// The name method is for profiling purposes.
pub(crate) trait TaskOnce: Send {
    fn name(&self) -> &'static str {
        ::std::any::type_name::<Self>()
    }

    fn run_once(self, cx: &mut js::context::JSContext);
}

/// A task that must be run on the same thread it originated in.
pub(crate) trait NonSendTaskOnce: crate::JSTraceable {
    fn run_once(self, cx: &mut js::context::JSContext);
}

/// A boxed version of `TaskOnce`.
pub(crate) trait TaskBox: Send {
    fn name(&self) -> &'static str;

    fn run_box(self: Box<Self>, cx: &mut js::context::JSContext);
}

/// A boxed version of `NonSendTaskOnce`.
pub(crate) trait NonSendTaskBox: crate::JSTraceable {
    fn run_box(self: Box<Self>, cx: &mut js::context::JSContext);
}

impl<T> NonSendTaskBox for T
where
    T: NonSendTaskOnce,
{
    fn run_box(self: Box<Self>, cx: &mut js::context::JSContext) {
        self.run_once(cx)
    }
}

impl<T> TaskBox for T
where
    T: TaskOnce,
{
    fn name(&self) -> &'static str {
        TaskOnce::name(self)
    }

    fn run_box(self: Box<Self>, cx: &mut js::context::JSContext) {
        self.run_once(cx)
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
    #[conditional_malloc_size_of]
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

    fn run_once(self, cx: &mut js::context::JSContext) {
        if !self.canceller.cancelled() {
            self.inner.run_once(cx)
        }
    }
}
