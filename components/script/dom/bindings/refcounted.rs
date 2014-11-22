/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

#![deny(missing_docs)]

//! A generic, safe mechnanism by which DOM objects can be pinned and transferred
//! between tasks (or intra-task for asynchronous events). Akin to Gecko's
//! nsMainThreadPtrHandle, this uses thread-safe reference counting and ensures
//! that the actual SpiderMonkey GC integration occurs on the script task via
//! message passing. Ownership of a `Trusted<T>` object means the DOM object of
//! type T to which it points remains alive. Any other behaviour is undefined.
//! To guarantee the lifetime of a DOM object when performing asynchronous operations,
//! obtain a `Trusted<T>` from that object and pass it along with each operation.
//! A usable pointer to the original DOM object can be obtained on the script task
//! from a `Trusted<T>` via the `to_temporary` method.
//!
//! The implementation of Trusted<T> is as follows:
//! A hashtable resides in the script task, keyed on the pointer to the Rust DOM object.
//! The values in this hashtable are atomic reference counts. When a Trusted<T> object is
//! created or cloned, this count is increased. When a Trusted<T> is dropped, the count
//! decreases. If the count hits zero, a message is dispatched to the script task to remove
//! the entry from the hashmap if the count is still zero. The JS reflector for the DOM object
//! is rooted when a hashmap entry is first created, and unrooted when the hashmap entry
//! is removed.

use dom::bindings::js::{Temporary, JS, JSRef};
use dom::bindings::utils::{Reflector, Reflectable};
use script_task::{ScriptMsg, ScriptChan};

use js::jsapi::{JS_AddObjectRoot, JS_RemoveObjectRoot, JSContext};

use libc;
use std::cell::RefCell;
use std::collections::hash_map::{HashMap, Vacant, Occupied};
use std::mem::transmute;
use std::sync::atomic::{AtomicUint, Relaxed};

local_data_key!(pub LiveReferences: LiveDOMReferences)

struct TrustedData {
    ptr: *const libc::c_void,
    refcount: AtomicUint,
    script_chan: Box<ScriptChan + Send>,
}

/// A safe wrapper around a raw pointer to a DOM object that can be
/// shared among tasks for use in asynchronous operations. The underlying
/// DOM object is guaranteed to live at least as long as the last outstanding
/// `Trusted<T>` instance.
pub struct Trusted<T> {
    _ptr: *mut TrustedData
}

impl<T: Reflectable> Trusted<T> {
    /// Create a new `Trusted<T>` instance from an existing DOM pointer. The DOM object will
    /// be prevented from being GCed for the duration of the resulting `Trusted<T>` object's
    /// lifetime.
    pub fn new(cx: *mut JSContext, ptr: JSRef<T>, script_chan: Box<ScriptChan + Send>) -> Trusted<T> {
        let live_references = LiveReferences.get().unwrap();
        live_references.addref(cx, &*ptr as *const T, script_chan)
    }

    /// Obtain a usable DOM pointer from a pinned `Trusted<T>` value. Fails if used on
    /// a different thread than the original value from which this `Trusted<T>` was
    /// obtained.
    pub fn to_temporary(&self) -> Temporary<T> {
        assert!(LiveReferences.get().unwrap().exists(&self.inner().ptr))
        unsafe {
            Temporary::new(JS::from_raw(self.inner().ptr as *const T))
        }
    }

    fn inner(&self) -> &TrustedData {
        unsafe { &*self._ptr }
    }
}

impl<T: Reflectable> Clone for Trusted<T> {
    fn clone(&self) -> Trusted<T> {
        self.inner().refcount.fetch_add(1, Relaxed);
        Trusted {
            _ptr: self._ptr
        }
    }
}

#[unsafe_destructor]
impl<T: Reflectable> Drop for Trusted<T> {
    fn drop(&mut self) {
        // Relaxed ordering is sufficient since no other shared data
        // is accessible through Trusted<T>
        let refcount = self.inner().refcount.fetch_sub(1, Relaxed);
        if refcount == 1 {
            self.inner().script_chan.send(ScriptMsg::RefcountCleanup(self.inner().ptr));
        }
    }
}

/// The set of live, pinned DOM objects that are currently prevented
/// from being garbage collected due to outstanding references.
pub struct LiveDOMReferences {
    // keyed on pointer to Rust DOM object
    table: RefCell<HashMap<*const libc::c_void, *mut TrustedData>>
}

impl LiveDOMReferences {
    /// Set up the task-local data required for storing the outstanding DOM references.
    pub fn initialize() {
        LiveReferences.replace(Some(LiveDOMReferences {
            table: RefCell::new(HashMap::new()),
        }));
    }

    fn addref<T: Reflectable>(&self, cx: *mut JSContext, ptr: *const T, script_chan: Box<ScriptChan + Send>) -> Trusted<T> {
        let mut table = self.table.borrow_mut();
        match table.entry(ptr as *const libc::c_void) {
            Occupied(entry) => {
                unsafe {
                    (**entry.get()).refcount.fetch_add(1, Relaxed);
                }
                Trusted { _ptr: *entry.get() }
            }
            Vacant(entry) => {
                unsafe {
                    let rootable = (*ptr).reflector().rootable();
                    JS_AddObjectRoot(cx, rootable);
                }
                let data = box TrustedData {
                    ptr: ptr as *const libc::c_void,
                    refcount: AtomicUint::new(1),
                    script_chan: script_chan,
                };
                let data = unsafe { transmute(data) };
                entry.set(data);
                Trusted { _ptr: data }
            }
        }
    }

    /// Unpin the given DOM object if its refcount is 0.
    pub fn cleanup(cx: *mut JSContext, raw_reflectable: *const libc::c_void) {
        let live_references = LiveReferences.get().unwrap();
        let reflectable = raw_reflectable as *const Reflector;
        let mut table = live_references.table.borrow_mut();
        match table.entry(raw_reflectable) {
            Occupied(entry) => unsafe {
                // there could have been a new reference taken since
                // this message was dispatched.
                if (**entry.get()).refcount.load(Relaxed) == 0 {
                    JS_RemoveObjectRoot(cx, (*reflectable).rootable());
                    let _ : Box<TrustedData> = transmute(entry.take());
                }

            },
            Vacant(_) => {
                // there could be a cleanup message dispatched, then a new
                // pinned reference obtained and released before the message
                // is processed, at which point there would be no matching
                // hashtable entry.
                info!("attempt to cleanup an unrecognized reflector");
            }
        }
    }

    fn exists(&self, raw_reflectable: &*const libc::c_void) -> bool {
        self.table.borrow().contains_key(raw_reflectable)
    }
}

impl Drop for LiveDOMReferences {
    fn drop(&mut self) {
        assert!(self.table.borrow().keys().count() == 0);
    }
}
