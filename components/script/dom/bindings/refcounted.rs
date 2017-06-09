/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

//! A generic, safe mechanism by which DOM objects can be pinned and transferred
//! between threads (or intra-thread for asynchronous events). Akin to Gecko's
//! nsMainThreadPtrHandle, this uses thread-safe reference counting and ensures
//! that the actual SpiderMonkey GC integration occurs on the script thread via
//! weak refcounts. Ownership of a `Trusted<T>` object means the DOM object of
//! type T to which it points remains alive. Any other behaviour is undefined.
//! To guarantee the lifetime of a DOM object when performing asynchronous operations,
//! obtain a `Trusted<T>` from that object and pass it along with each operation.
//! A usable pointer to the original DOM object can be obtained on the script thread
//! from a `Trusted<T>` via the `to_temporary` method.
//!
//! The implementation of `Trusted<T>` is as follows:
//! The `Trusted<T>` object contains an atomic reference counted pointer to the Rust DOM object.
//! A hashtable resides in the script thread, keyed on the pointer.
//! The values in this hashtable are weak reference counts. When a `Trusted<T>` object is
//! created or cloned, the reference count is increased. When a `Trusted<T>` is dropped, the count
//! decreases. If the count hits zero, the weak reference is emptied, and is removed from
//! its hash table during the next GC. During GC, the entries of the hash table are counted
//! as JS roots.

use core::nonzero::NonZero;
use dom::bindings::conversions::ToJSValConvertible;
use dom::bindings::error::Error;
use dom::bindings::js::Root;
use dom::bindings::reflector::{DomObject, Reflector};
use dom::bindings::trace::trace_reflector;
use dom::promise::Promise;
use js::jsapi::JSAutoCompartment;
use js::jsapi::JSTracer;
use libc;
use script_thread::Runnable;
use script_thread::ScriptThread;
use std::cell::RefCell;
use std::collections::hash_map::Entry::{Occupied, Vacant};
use std::collections::hash_map::HashMap;
use std::hash::Hash;
use std::marker::PhantomData;
use std::os;
use std::rc::Rc;
use std::sync::{Arc, Weak};


#[allow(missing_docs)]  // FIXME
mod dummy {  // Attributes don’t apply through the macro.
    use std::cell::RefCell;
    use std::rc::Rc;
    use super::LiveDOMReferences;
    thread_local!(pub static LIVE_REFERENCES: Rc<RefCell<Option<LiveDOMReferences>>> =
            Rc::new(RefCell::new(None)));
}
pub use self::dummy::LIVE_REFERENCES;


/// A pointer to a Rust DOM object that needs to be destroyed.
pub struct TrustedReference(*const libc::c_void);
unsafe impl Send for TrustedReference {}

impl TrustedReference {
    fn new<T: DomObject>(ptr: *const T) -> TrustedReference {
        TrustedReference(ptr as *const libc::c_void)
    }
}

/// A safe wrapper around a DOM Promise object that can be shared among threads for use
/// in asynchronous operations. The underlying DOM object is guaranteed to live at least
/// as long as the last outstanding `TrustedPromise` instance. These values cannot be cloned,
/// only created from existing Rc<Promise> values.
pub struct TrustedPromise {
    dom_object: *const Promise,
    owner_thread: *const libc::c_void,
}

unsafe impl Send for TrustedPromise {}

impl TrustedPromise {
    /// Create a new `TrustedPromise` instance from an existing DOM object. The object will
    /// be prevented from being GCed for the duration of the resulting `TrustedPromise` object's
    /// lifetime.
    #[allow(unrooted_must_root)]
    pub fn new(promise: Rc<Promise>) -> TrustedPromise {
        LIVE_REFERENCES.with(|ref r| {
            let r = r.borrow();
            let live_references = r.as_ref().unwrap();
            let ptr = &*promise as *const Promise;
            live_references.addref_promise(promise);
            TrustedPromise {
                dom_object: ptr,
                owner_thread: (&*live_references) as *const _ as *const libc::c_void,
            }
        })
    }

    /// Obtain a usable DOM Promise from a pinned `TrustedPromise` value. Fails if used on
    /// a different thread than the original value from which this `TrustedPromise` was
    /// obtained.
    #[allow(unrooted_must_root)]
    pub fn root(self) -> Rc<Promise> {
        LIVE_REFERENCES.with(|ref r| {
            let r = r.borrow();
            let live_references = r.as_ref().unwrap();
            assert!(self.owner_thread == (&*live_references) as *const _ as *const libc::c_void);
            // Borrow-check error requires the redundant `let promise = ...; promise` here.
            let promise = match live_references.promise_table.borrow_mut().entry(self.dom_object) {
                Occupied(mut entry) => {
                    let promise = {
                        let promises = entry.get_mut();
                        promises.pop().expect("rooted promise list unexpectedly empty")
                    };
                    if entry.get().is_empty() {
                        entry.remove();
                    }
                    promise
                }
                Vacant(_) => unreachable!(),
            };
            promise
        })
    }

    /// A runnable which will reject the promise.
    #[allow(unrooted_must_root)]
    pub fn reject_runnable(self, error: Error) -> impl Runnable + Send {
        struct RejectPromise(TrustedPromise, Error);
        impl Runnable for RejectPromise {
            fn main_thread_handler(self: Box<Self>, script_thread: &ScriptThread) {
                debug!("Rejecting promise.");
                let this = *self;
                let cx = script_thread.get_cx();
                let promise = this.0.root();
                let _ac = JSAutoCompartment::new(cx, promise.reflector().get_jsobject().get());
                promise.reject_error(cx, this.1);
            }
        }
        RejectPromise(self, error)
    }

    /// A runnable which will resolve the promise.
    #[allow(unrooted_must_root)]
    pub fn resolve_runnable<T>(self, value: T) -> impl Runnable + Send where
        T: ToJSValConvertible + Send
    {
        struct ResolvePromise<T>(TrustedPromise, T);
        impl<T: ToJSValConvertible> Runnable for ResolvePromise<T> {
            fn main_thread_handler(self: Box<Self>, script_thread: &ScriptThread) {
                debug!("Resolving promise.");
                let this = *self;
                let cx = script_thread.get_cx();
                let promise = this.0.root();
                let _ac = JSAutoCompartment::new(cx, promise.reflector().get_jsobject().get());
                promise.resolve_native(cx, &this.1);
            }
        }
        ResolvePromise(self, value)
    }
}

/// A safe wrapper around a raw pointer to a DOM object that can be
/// shared among threads for use in asynchronous operations. The underlying
/// DOM object is guaranteed to live at least as long as the last outstanding
/// `Trusted<T>` instance.
#[allow_unrooted_interior]
pub struct Trusted<T: DomObject> {
    /// A pointer to the Rust DOM object of type T, but void to allow
    /// sending `Trusted<T>` between threads, regardless of T's sendability.
    refcount: Arc<TrustedReference>,
    owner_thread: *const libc::c_void,
    phantom: PhantomData<T>,
}

unsafe impl<T: DomObject> Send for Trusted<T> {}

impl<T: DomObject> Trusted<T> {
    /// Create a new `Trusted<T>` instance from an existing DOM pointer. The DOM object will
    /// be prevented from being GCed for the duration of the resulting `Trusted<T>` object's
    /// lifetime.
    pub fn new(ptr: &T) -> Trusted<T> {
        LIVE_REFERENCES.with(|ref r| {
            let r = r.borrow();
            let live_references = r.as_ref().unwrap();
            let refcount = live_references.addref(&*ptr as *const T);
            Trusted {
                refcount: refcount,
                owner_thread: (&*live_references) as *const _ as *const libc::c_void,
                phantom: PhantomData,
            }
        })
    }

    /// Obtain a usable DOM pointer from a pinned `Trusted<T>` value. Fails if used on
    /// a different thread than the original value from which this `Trusted<T>` was
    /// obtained.
    pub fn root(&self) -> Root<T> {
        assert!(LIVE_REFERENCES.with(|ref r| {
            let r = r.borrow();
            let live_references = r.as_ref().unwrap();
            self.owner_thread == (&*live_references) as *const _ as *const libc::c_void
        }));
        unsafe {
            Root::new(NonZero::new(self.refcount.0 as *const T))
        }
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
#[allow(unrooted_must_root)]
pub struct LiveDOMReferences {
    // keyed on pointer to Rust DOM object
    reflectable_table: RefCell<HashMap<*const libc::c_void, Weak<TrustedReference>>>,
    promise_table: RefCell<HashMap<*const Promise, Vec<Rc<Promise>>>>,
}

impl LiveDOMReferences {
    /// Set up the thread-local data required for storing the outstanding DOM references.
    pub fn initialize() {
        LIVE_REFERENCES.with(|ref r| {
            *r.borrow_mut() = Some(LiveDOMReferences {
                reflectable_table: RefCell::new(HashMap::new()),
                promise_table: RefCell::new(HashMap::new()),
            })
        });
    }

    #[allow(unrooted_must_root)]
    fn addref_promise(&self, promise: Rc<Promise>) {
        let mut table = self.promise_table.borrow_mut();
        table.entry(&*promise).or_insert(vec![]).push(promise)
    }

    fn addref<T: DomObject>(&self, ptr: *const T) -> Arc<TrustedReference> {
        let mut table = self.reflectable_table.borrow_mut();
        let capacity = table.capacity();
        let len = table.len();
        if (0 < capacity) && (capacity <= len) {
            info!("growing refcounted references by {}", len);
            remove_nulls(&mut table);
            table.reserve(len);
        }
        match table.entry(ptr as *const libc::c_void) {
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
            }
        }
    }
}

/// Remove null entries from the live references table
fn remove_nulls<K: Eq + Hash + Clone, V> (table: &mut HashMap<K, Weak<V>>) {
    let to_remove: Vec<K> =
        table.iter()
        .filter(|&(_, value)| Weak::upgrade(value).is_none())
        .map(|(key, _)| key.clone())
        .collect();
    info!("removing {} refcounted references", to_remove.len());
    for key in to_remove {
        table.remove(&key);
    }
}

/// A JSTraceDataOp for tracing reflectors held in LIVE_REFERENCES
#[allow(unrooted_must_root)]
pub unsafe extern "C" fn trace_refcounted_objects(tracer: *mut JSTracer,
                                                  _data: *mut os::raw::c_void) {
    info!("tracing live refcounted references");
    LIVE_REFERENCES.with(|ref r| {
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
