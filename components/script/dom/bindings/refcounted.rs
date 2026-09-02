/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

//! A generic, safe mechanism by which Promise objects can be pinned and transferred
//! between threads (or intra-thread for asynchronous events). See more information in
//! script_bindings::refcounted

use std::cell::RefCell;
use std::collections::hash_map::Entry::{Occupied, Vacant};
use std::rc::Rc;

use js::conversions::ToJSValConvertible;
use js::jsapi::JSTracer;
use rustc_hash::FxHashMap;
use script_bindings::error::Error;
pub(crate) use script_bindings::refcounted::Trusted;
use script_bindings::reflector::DomObject;
use script_bindings::trace::trace_reflector;

use crate::dom::promise::Promise;
use crate::tasks::task::TaskOnce;

thread_local!(pub(super) static LIVE_REFERENCES: Rc<RefCell<LivePromiseReferences>> =
    Rc::new(RefCell::new(
    LivePromiseReferences {
        promise_table: RefCell::new(FxHashMap::default()),
    }
)));

/// The set of live, pinned DOM objects that are currently prevented
/// from being garbage collected due to outstanding references.
pub(crate) struct LivePromiseReferences {
    // keyed on pointer to Rust DOM object
    promise_table: RefCell<FxHashMap<*const Promise, Vec<Rc<Promise>>>>,
}

impl LivePromiseReferences {
    pub(crate) fn destruct() {
        LIVE_REFERENCES.with(|r| {
            let live_references = r.borrow_mut();
            let _ = live_references.promise_table.take();
        });
    }

    fn addref_promise(&self, promise: Rc<Promise>) {
        let mut table = self.promise_table.borrow_mut();
        table.entry(&*promise).or_default().push(promise)
    }
}

/// A safe wrapper around a DOM Promise object that can be shared among threads for use
/// in asynchronous operations. The underlying DOM object is guaranteed to live at least
/// as long as the last outstanding `TrustedPromise` instance. These values cannot be cloned,
/// only created from existing `Rc<Promise>` values.
pub struct TrustedPromise {
    dom_object: *const Promise,
    owner_thread: *const libc::c_void,
}

unsafe impl Send for TrustedPromise {}

impl TrustedPromise {
    /// Create a new `TrustedPromise` instance from an existing DOM object. The object will
    /// be prevented from being GCed for the duration of the resulting `TrustedPromise` object's
    /// lifetime.
    pub(crate) fn new(promise: Rc<Promise>) -> TrustedPromise {
        LIVE_REFERENCES.with(|r| {
            let live_references = &*r.borrow();
            let ptr = &raw const *promise;
            live_references.addref_promise(promise);
            TrustedPromise {
                dom_object: ptr,
                owner_thread: (live_references) as *const _ as *const libc::c_void,
            }
        })
    }

    /// Obtain a usable DOM Promise from a pinned `TrustedPromise` value. Fails if used on
    /// a different thread than the original value from which this `TrustedPromise` was
    /// obtained.
    pub(crate) fn root(self) -> Rc<Promise> {
        LIVE_REFERENCES.with(|r| {
            let live_references = &*r.borrow();
            assert_eq!(
                self.owner_thread,
                live_references as *const _ as *const libc::c_void
            );
            match live_references
                .promise_table
                .borrow_mut()
                .entry(self.dom_object)
            {
                Occupied(mut entry) => {
                    let promise = {
                        let promises = entry.get_mut();
                        promises
                            .pop()
                            .expect("rooted promise list unexpectedly empty")
                    };
                    if entry.get().is_empty() {
                        entry.remove();
                    }
                    promise
                },
                Vacant(_) => unreachable!(),
            }
        })
    }

    /// A task which will reject the promise.
    pub(crate) fn reject_task(self, error: Error) -> impl TaskOnce {
        let this = self;
        task!(reject_promise: move |cx| {
            debug!("Rejecting promise.");
            this.root().reject_error(cx, error);
        })
    }

    /// A task which will resolve the promise.
    pub(crate) fn resolve_task<T>(self, value: T) -> impl TaskOnce
    where
        T: ToJSValConvertible + Send,
    {
        let this = self;
        task!(resolve_promise: move |cx| {
            debug!("Resolving promise.");
            this.root().resolve_native(cx, &value);
        })
    }
}

/// A JSTraceDataOp for tracing reflectors held in LIVE_REFERENCES
pub(crate) unsafe fn trace_refcounted_objects(tracer: *mut JSTracer) {
    trace!("tracing live refcounted references");
    LIVE_REFERENCES.with(|r| {
        let live_references = &*r.borrow();
        {
            let table = live_references.promise_table.borrow_mut();
            for promise in table.keys() {
                unsafe {
                    trace_reflector(tracer, "refcounted", (**promise).reflector());
                }
            }
        }
    });
    unsafe {
        script_bindings::refcounted::trace_live_domreferences(tracer);
    }
}
