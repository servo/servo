/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

//! A generic, safe mechanism by which Promise objects can be pinned and transferred
//! between threads (or intra-thread for asynchronous events). See more information in
//! script_bindings::refcounted

use std::cell::RefCell;
use std::sync::{Arc, Weak};
use std::thread::{self, ThreadId};

use js::context::JSContext;
use js::conversions::ToJSValConvertible;
use js::jsapi::JSTracer;
use js::rust::Trace;
use rustc_hash::FxHashMap;
use script_bindings::error::Error;
use script_bindings::interfaces::ThreadTrustedPromiseHelpers;
pub(crate) use script_bindings::refcounted::Trusted;
use script_bindings::reflector::DomObject;

use crate::dom::promise::{Promise, RootedPromise, TracedPromise};
use crate::tasks::task::TaskOnce;

thread_local!(pub(super) static LIVE_PROMISE_REFERENCES: LivePromiseReferences =
    LivePromiseReferences {
        promise_table: RefCell::new(FxHashMap::default()),
    }
);

#[derive(Clone, Copy, Eq, Hash, PartialEq)]
struct PromiseKey(*const Promise);

/// # Safety
/// PromiseKey is only ever used for comparisons between keys.
/// Its value is never read.
unsafe impl Send for PromiseKey {}
unsafe impl Sync for PromiseKey {}

/// The set of live, pinned DOM objects that are currently prevented
/// from being garbage collected due to outstanding references.
#[cfg_attr(crown, crown::unrooted_must_root_lint::must_root)]
pub(crate) struct LivePromiseReferences {
    // The table of promises and weak keys keyed on the address of the `Rc<Promise>` owned by each
    // entry. The weak key tracks whether there are any live references to this entry remaining.
    promise_table: RefCell<FxHashMap<PromiseKey, (TracedPromise, Weak<PromiseKey>)>>,
}

impl LivePromiseReferences {
    /// Prune all `TrustedPromise` entries that no longer have live references. Since this walks
    /// through every promise, it should not be called from hot code.
    pub(crate) fn expensive_prune_dropped_promises() {
        LIVE_PROMISE_REFERENCES.with(|live_promise_references| {
            live_promise_references
                .promise_table
                .borrow_mut()
                .retain(|_, (_, key)| key.strong_count() > 0);
        })
    }

    pub(crate) fn destruct() {
        LIVE_PROMISE_REFERENCES.with(|live_references| {
            let _ = live_references.promise_table.take();
        });
    }

    #[cfg_attr(crown, expect(crown::unrooted_must_root))]
    fn store(&self, promise: &RootedPromise) -> TrustedPromise {
        // Since converting a RootedPromise to a TracedPromise allocates a new
        // underlying Rc<Promise>, we are guaranteed that there is no prior entry
        // in the hashtable for this particular promise object.
        let traced_promise = promise.to_traced();

        let mut promise_table = self.promise_table.borrow_mut();

        // Prune any promises that have been dropped on other threads. This is
        // done here so that they can be dropped on the right thread. This prunes
        // using the same heuristic as `Trusted<...>`.
        if promise_table.len() == promise_table.capacity() {
            promise_table.retain(|_, (_, key)| key.strong_count() > 0);
        }

        let key = Arc::new(PromiseKey(&raw const *traced_promise));
        let _exists = promise_table
            .insert(*key, (traced_promise, Arc::downgrade(&key)))
            .is_some();
        debug_assert!(!_exists);

        TrustedPromise {
            key,
            owner_thread: thread::current().id(),
        }
    }

    fn remove(&self, key: &PromiseKey) -> Option<TracedPromise> {
        self.promise_table
            .borrow_mut()
            .remove(key)
            .map(|(promise, _)| promise)
    }
}

/// A safe wrapper around a DOM Promise object that can be shared among threads for use
/// in asynchronous operations. The underlying DOM object is guaranteed to live at least
/// as long as the last outstanding `TrustedPromise` instance. These values cannot be cloned,
/// only created from existing `Rc<Promise>` values.
pub(crate) struct TrustedPromise {
    key: Arc<PromiseKey>,
    owner_thread: ThreadId,
}

impl TrustedPromise {
    /// Create a new `TrustedPromise` instance from an existing DOM object. The object will
    /// be prevented from being GCed for the duration of the resulting `TrustedPromise` object's
    /// lifetime.
    pub(crate) fn new(promise: &RootedPromise) -> TrustedPromise {
        LIVE_PROMISE_REFERENCES
            .with(|live_promise_references| live_promise_references.store(promise))
    }

    /// Obtain a usable DOM Promise from a pinned `TrustedPromise` value. Fails if used on
    /// a different thread than the original value from which this `TrustedPromise` was
    /// obtained.
    pub(crate) fn root(self, cx: &JSContext) -> RootedPromise {
        LIVE_PROMISE_REFERENCES.with(|live_references| {
            debug_assert_eq!(self.owner_thread, thread::current().id());
            live_references
                .promise_table
                .borrow()
                .get(&self.key)
                .expect("Must always have an entry matching a TrustedPromise")
                .0
                .root(cx)
        })
    }

    /// A task which will reject the promise.
    pub(crate) fn reject_task(self, error: Error) -> impl TaskOnce {
        let this = self;
        task!(reject_promise: move |cx| {
            debug!("Rejecting promise.");
            this.root(cx).reject_error(cx, error);
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
            this.root(cx).resolve_native(cx, &value);
        })
    }
}

impl Drop for TrustedPromise {
    fn drop(&mut self) {
        // When dropped on a different thread, removal happens lazily.
        if self.owner_thread != thread::current().id() {
            return;
        }
        // This needs `try_with` because it can be called during teardown.
        let _ = LIVE_PROMISE_REFERENCES
            .try_with(|live_promise_references| live_promise_references.remove(&self.key));
    }
}

impl ThreadTrustedPromiseHelpers<crate::DomTypeHolder> for TrustedPromise {
    type StackRoot = RootedPromise;

    fn root(self, cx: &JSContext) -> RootedPromise {
        TrustedPromise::root(self, cx)
    }
}

/// A JSTraceDataOp for tracing reflectors held in LIVE_REFERENCES
pub(crate) unsafe fn trace_refcounted_objects(tracer: *mut JSTracer) {
    trace!("tracing live refcounted promise references");
    LIVE_PROMISE_REFERENCES.with(|live_references| {
        let table = live_references.promise_table.borrow();
        for (promise, _) in table.values() {
            unsafe {
                promise.reflector().trace(tracer);
            }
        }
    });
    trace!("tracing live refcounted references");
    unsafe {
        script_bindings::refcounted::trace_live_domreferences(tracer);
    }
}
