/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

//! A generic, safe mechanism by which DOM objects can be pinned and transferred
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

use dom::bindings::js::Root;
use dom::bindings::utils::{Reflector, Reflectable};
use dom::bindings::trace::trace_reflector;
use script_task::{ScriptChan, CommonScriptMsg};

use js::jsapi::{JSContext, JSTracer};

use libc;
use std::cell::RefCell;
use std::collections::hash_map::HashMap;
use std::collections::hash_map::Entry::{Vacant, Occupied};
use std::marker::PhantomData;
use std::sync::{Arc, Mutex};
use core::nonzero::NonZero;


#[allow(missing_docs)]  // FIXME
mod dummy {  // Attributes don’t apply through the macro.
    use std::rc::Rc;
    use std::cell::RefCell;
    use super::LiveDOMReferences;
    thread_local!(pub static LIVE_REFERENCES: Rc<RefCell<Option<LiveDOMReferences>>> =
            Rc::new(RefCell::new(None)));
}
pub use self::dummy::LIVE_REFERENCES;


/// A pointer to a Rust DOM object that needs to be destroyed.
pub struct TrustedReference(*const libc::c_void);
unsafe impl Send for TrustedReference {}

/// A safe wrapper around a raw pointer to a DOM object that can be
/// shared among tasks for use in asynchronous operations. The underlying
/// DOM object is guaranteed to live at least as long as the last outstanding
/// `Trusted<T>` instance.
#[allow_unrooted_interior]
pub struct Trusted<T: Reflectable> {
    /// A pointer to the Rust DOM object of type T, but void to allow
    /// sending `Trusted<T>` between tasks, regardless of T's sendability.
    ptr: *const libc::c_void,
    refcount: Arc<Mutex<usize>>,
    script_chan: Box<ScriptChan + Send>,
    owner_thread: *const libc::c_void,
    phantom: PhantomData<T>,
}

unsafe impl<T: Reflectable> Send for Trusted<T> {}

impl<T: Reflectable> Trusted<T> {
    /// Create a new `Trusted<T>` instance from an existing DOM pointer. The DOM object will
    /// be prevented from being GCed for the duration of the resulting `Trusted<T>` object's
    /// lifetime.
    pub fn new(_cx: *mut JSContext, ptr: &T, script_chan: Box<ScriptChan + Send>) -> Trusted<T> {
        LIVE_REFERENCES.with(|ref r| {
            let r = r.borrow();
            let live_references = r.as_ref().unwrap();
            let refcount = live_references.addref(&*ptr as *const T);
            Trusted {
                ptr: &*ptr as *const T as *const libc::c_void,
                refcount: refcount,
                script_chan: script_chan.clone(),
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
            Root::new(NonZero::new(self.ptr as *const T))
        }
    }
}

impl<T: Reflectable> Clone for Trusted<T> {
    fn clone(&self) -> Trusted<T> {
        {
            let mut refcount = self.refcount.lock().unwrap();
            *refcount += 1;
        }

        Trusted {
            ptr: self.ptr,
            refcount: self.refcount.clone(),
            script_chan: self.script_chan.clone(),
            owner_thread: self.owner_thread,
            phantom: PhantomData,
        }
    }
}

impl<T: Reflectable> Drop for Trusted<T> {
    fn drop(&mut self) {
        let mut refcount = self.refcount.lock().unwrap();
        assert!(*refcount > 0);
        *refcount -= 1;
        if *refcount == 0 {
            // It's possible this send will fail if the script task
            // has already exited. There's not much we can do at this
            // point though.
            let msg = CommonScriptMsg::RefcountCleanup(TrustedReference(self.ptr));
            let _ = self.script_chan.send(msg);
        }
    }
}

/// The set of live, pinned DOM objects that are currently prevented
/// from being garbage collected due to outstanding references.
pub struct LiveDOMReferences {
    // keyed on pointer to Rust DOM object
    table: RefCell<HashMap<*const libc::c_void, Arc<Mutex<usize>>>>
}

impl LiveDOMReferences {
    /// Set up the task-local data required for storing the outstanding DOM references.
    pub fn initialize() {
        LIVE_REFERENCES.with(|ref r| {
            *r.borrow_mut() = Some(LiveDOMReferences {
                table: RefCell::new(HashMap::new()),
            })
        });
    }

    fn addref<T: Reflectable>(&self, ptr: *const T) -> Arc<Mutex<usize>> {
        let mut table = self.table.borrow_mut();
        match table.entry(ptr as *const libc::c_void) {
            Occupied(mut entry) => {
                let refcount = entry.get_mut();
                *refcount.lock().unwrap() += 1;
                refcount.clone()
            }
            Vacant(entry) => {
                let refcount = Arc::new(Mutex::new(1));
                entry.insert(refcount.clone());
                refcount
            }
        }
    }

    /// Unpin the given DOM object if its refcount is 0.
    pub fn cleanup(raw_reflectable: TrustedReference) {
        let TrustedReference(raw_reflectable) = raw_reflectable;
        LIVE_REFERENCES.with(|ref r| {
            let r = r.borrow();
            let live_references = r.as_ref().unwrap();
            let mut table = live_references.table.borrow_mut();
            match table.entry(raw_reflectable) {
                Occupied(entry) => {
                    if *entry.get().lock().unwrap() != 0 {
                        // there could have been a new reference taken since
                        // this message was dispatched.
                        return;
                    }

                    let _ = entry.remove();
                }
                Vacant(_) => {
                    // there could be a cleanup message dispatched, then a new
                    // pinned reference obtained and released before the message
                    // is processed, at which point there would be no matching
                    // hashtable entry.
                    info!("attempt to cleanup an unrecognized reflector");
                }
            }
        })
    }
}

/// A JSTraceDataOp for tracing reflectors held in LIVE_REFERENCES
pub unsafe extern fn trace_refcounted_objects(tracer: *mut JSTracer, _data: *mut libc::c_void) {
    LIVE_REFERENCES.with(|ref r| {
        let r = r.borrow();
        let live_references = r.as_ref().unwrap();
        let table = live_references.table.borrow();
        for obj in table.keys() {
            let reflectable = &*(*obj as *const Reflector);
            trace_reflector(tracer, "LIVE_REFERENCES", reflectable);
        }
    });
}
