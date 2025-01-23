/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

//! A generic, safe mechanism by which DOM objects can be pinned and transferred
//! between threads (or intra-thread for asynchronous events). Akin to Gecko's
//! nsMainThreadPtrHandle, this uses thread-safe reference counting and ensures
//! that the actual SpiderMonkey GC integration occurs on the script thread via
//! weak refcounts. Ownership of a `Trusted<T>` object means the DOM object of
//! type T to which it points remains alive. Any other behaviour is undefined.
//! To guarantee the lifetime of a DOM object when performing asynchronous operations,
//! obtain a `Trusted<T>` from that object and pass it along with each operation.
//! A usable pointer to the original DOM object can be obtained on the script thread
//! from a `Trusted<T>` via the `root` method.
//!
//! The implementation of `Trusted<T>` is as follows:
//! The `Trusted<T>` object contains an atomic reference counted pointer to the Rust DOM object.
//! A hashtable resides in the script thread, keyed on the pointer.
//! The values in this hashtable are weak reference counts. When a `Trusted<T>` object is
//! created or cloned, the reference count is increased. When a `Trusted<T>` is dropped, the count
//! decreases. If the count hits zero, the weak reference is emptied, and is removed from
//! its hash table during the next GC. During GC, the entries of the hash table are counted
//! as JS roots.

use std::cell::RefCell;
use std::collections::hash_map::Entry::{Occupied, Vacant};
use std::collections::hash_map::HashMap;
use std::hash::Hash;
use std::marker::PhantomData;
use std::rc::Rc;
use std::sync::{Arc, Weak};

use js::jsapi::JSTracer;

use crate::dom::bindings::conversions::ToJSValConvertible;
use crate::dom::bindings::error::Error;
use crate::dom::bindings::reflector::{DomObject, Reflector};
use crate::dom::bindings::root::DomRoot;
use crate::dom::bindings::trace::trace_reflector;
use crate::dom::promise::Promise;
use crate::task::TaskOnce;

#[allow(missing_docs)] // FIXME
mod dummy {
    // Attributes don’t apply through the macro.
    use std::cell::RefCell;
    use std::rc::Rc;

    use super::LiveDOMReferences;
    thread_local!(pub(crate) static LIVE_REFERENCES: Rc<RefCell<Option<LiveDOMReferences>>> =
            Rc::new(RefCell::new(None)));
}
pub(crate) use self::dummy::LIVE_REFERENCES;

/// A pointer to a Rust DOM object that needs to be destroyed.
#[derive(MallocSizeOf)]
struct TrustedReference(
    #[ignore_malloc_size_of = "This is a shared reference."] *const libc::c_void,
);
unsafe impl Send for TrustedReference {}

impl TrustedReference {
    /// Creates a new TrustedReference from a pointer to a value that impements DOMObject.
    /// This is not enforced by the type system to reduce duplicated generic code,
    /// which is acceptable since this method is internal to this module.
    unsafe fn new(ptr: *const libc::c_void) -> TrustedReference {
        TrustedReference(ptr)
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
    #[cfg_attr(crown, allow(crown::unrooted_must_root))]
    pub(crate) fn new(promise: Rc<Promise>) -> TrustedPromise {
        LIVE_REFERENCES.with(|r| {
            let r = r.borrow();
            let live_references = r.as_ref().unwrap();
            let ptr = &*promise as *const Promise;
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
            let r = r.borrow();
            let live_references = r.as_ref().unwrap();
            assert_eq!(
                self.owner_thread,
                (live_references) as *const _ as *const libc::c_void
            );
            // Borrow-check error requires the redundant `let promise = ...; promise` here.
            let promise = match live_references
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
            };
            promise
        })
    }

    /// A task which will reject the promise.
    #[cfg_attr(crown, allow(crown::unrooted_must_root))]
    pub(crate) fn reject_task(self, error: Error) -> impl TaskOnce {
        let this = self;
        task!(reject_promise: move || {
            debug!("Rejecting promise.");
            this.root().reject_error(error);
        })
    }

    /// A task which will resolve the promise.
    #[cfg_attr(crown, allow(crown::unrooted_must_root))]
    pub(crate) fn resolve_task<T>(self, value: T) -> impl TaskOnce
    where
        T: ToJSValConvertible + Send,
    {
        let this = self;
        task!(resolve_promise: move || {
            debug!("Resolving promise.");
            this.root().resolve_native(&value);
        })
    }
}

/// A safe wrapper around a raw pointer to a DOM object that can be
/// shared among threads for use in asynchronous operations. The underlying
/// DOM object is guaranteed to live at least as long as the last outstanding
/// `Trusted<T>` instance.
#[cfg_attr(crown, crown::unrooted_must_root_lint::allow_unrooted_interior)]
#[derive(MallocSizeOf)]
pub(crate) struct Trusted<T: DomObject> {
    /// A pointer to the Rust DOM object of type T, but void to allow
    /// sending `Trusted<T>` between threads, regardless of T's sendability.
    #[conditional_malloc_size_of]
    refcount: Arc<TrustedReference>,
    #[ignore_malloc_size_of = "These are shared by all `Trusted` types."]
    owner_thread: *const LiveDOMReferences,
    phantom: PhantomData<T>,
}

unsafe impl<T: DomObject> Send for Trusted<T> {}

impl<T: DomObject> Trusted<T> {
    /// Create a new `Trusted<T>` instance from an existing DOM pointer. The DOM object will
    /// be prevented from being GCed for the duration of the resulting `Trusted<T>` object's
    /// lifetime.
    pub(crate) fn new(ptr: &T) -> Trusted<T> {
        fn add_live_reference(
            ptr: *const libc::c_void,
        ) -> (Arc<TrustedReference>, *const LiveDOMReferences) {
            LIVE_REFERENCES.with(|r| {
                let r = r.borrow();
                let live_references = r.as_ref().unwrap();
                let refcount = unsafe { live_references.addref(ptr) };
                (refcount, live_references as *const _)
            })
        }

        let (refcount, owner_thread) = add_live_reference(ptr as *const T as *const _);
        Trusted {
            refcount,
            owner_thread,
            phantom: PhantomData,
        }
    }

    /// Obtain a usable DOM pointer from a pinned `Trusted<T>` value. Fails if used on
    /// a different thread than the original value from which this `Trusted<T>` was
    /// obtained.
    pub(crate) fn root(&self) -> DomRoot<T> {
        fn validate(owner_thread: *const LiveDOMReferences) {
            assert!(LIVE_REFERENCES.with(|r| {
                let r = r.borrow();
                let live_references = r.as_ref().unwrap();
                owner_thread == live_references
            }));
        }
        validate(self.owner_thread);
        unsafe { DomRoot::from_ref(&*(self.refcount.0 as *const T)) }
    }
}

impl<T: DomObject> Clone for Trusted<T> {
    fn clone(&self) -> Trusted<T> {
        Trusted {
            refcount: self.refcount.clone(),
            owner_thread: self.owner_thread,
            phantom: PhantomData,
        }
    }
}

/// The set of live, pinned DOM objects that are currently prevented
/// from being garbage collected due to outstanding references.
#[cfg_attr(crown, allow(crown::unrooted_must_root))]
pub(crate) struct LiveDOMReferences {
    // keyed on pointer to Rust DOM object
    reflectable_table: RefCell<HashMap<*const libc::c_void, Weak<TrustedReference>>>,
    promise_table: RefCell<HashMap<*const Promise, Vec<Rc<Promise>>>>,
}

impl LiveDOMReferences {
    /// Set up the thread-local data required for storing the outstanding DOM references.
    pub(crate) fn initialize() {
        LIVE_REFERENCES.with(|r| {
            *r.borrow_mut() = Some(LiveDOMReferences {
                reflectable_table: RefCell::new(HashMap::new()),
                promise_table: RefCell::new(HashMap::new()),
            })
        });
    }

    pub(crate) fn destruct() {
        LIVE_REFERENCES.with(|r| {
            *r.borrow_mut() = None;
        });
    }

    #[cfg_attr(crown, allow(crown::unrooted_must_root))]
    fn addref_promise(&self, promise: Rc<Promise>) {
        let mut table = self.promise_table.borrow_mut();
        table.entry(&*promise).or_default().push(promise)
    }

    /// ptr must be a pointer to a type that implements DOMObject.
    /// This is not enforced by the type system to reduce duplicated generic code,
    /// which is acceptable since this method is internal to this module.
    #[allow(clippy::arc_with_non_send_sync)]
    unsafe fn addref(&self, ptr: *const libc::c_void) -> Arc<TrustedReference> {
        let mut table = self.reflectable_table.borrow_mut();
        let capacity = table.capacity();
        let len = table.len();
        if (0 < capacity) && (capacity <= len) {
            trace!("growing refcounted references by {}", len);
            remove_nulls(&mut table);
            table.reserve(len);
        }
        match table.entry(ptr) {
            Occupied(mut entry) => match entry.get().upgrade() {
                Some(refcount) => refcount,
                None => {
                    let refcount = Arc::new(TrustedReference::new(ptr));
                    entry.insert(Arc::downgrade(&refcount));
                    refcount
                },
            },
            Vacant(entry) => {
                let refcount = Arc::new(TrustedReference::new(ptr));
                entry.insert(Arc::downgrade(&refcount));
                refcount
            },
        }
    }
}

/// Remove null entries from the live references table
fn remove_nulls<K: Eq + Hash + Clone, V>(table: &mut HashMap<K, Weak<V>>) {
    let to_remove: Vec<K> = table
        .iter()
        .filter(|&(_, value)| Weak::upgrade(value).is_none())
        .map(|(key, _)| key.clone())
        .collect();
    trace!("removing {} refcounted references", to_remove.len());
    for key in to_remove {
        table.remove(&key);
    }
}

/// A JSTraceDataOp for tracing reflectors held in LIVE_REFERENCES
#[cfg_attr(crown, allow(crown::unrooted_must_root))]
pub(crate) unsafe fn trace_refcounted_objects(tracer: *mut JSTracer) {
    trace!("tracing live refcounted references");
    LIVE_REFERENCES.with(|r| {
        let r = r.borrow();
        let live_references = r.as_ref().unwrap();
        {
            let mut table = live_references.reflectable_table.borrow_mut();
            remove_nulls(&mut table);
            for obj in table.keys() {
                let reflectable = &*(*obj as *const Reflector);
                trace_reflector(tracer, "refcounted", reflectable);
            }
        }

        {
            let table = live_references.promise_table.borrow_mut();
            for promise in table.keys() {
                trace_reflector(tracer, "refcounted", (**promise).reflector());
            }
        }
    });
}
