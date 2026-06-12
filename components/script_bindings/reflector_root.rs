/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

//! A permanently rooted Reflector, that can be unrooted on demand.
//!
//! We currently still need perma-roots in some areas of servo. However, this can lead to
//! memory leaks if the perma-rooted Object transitively references an object that holds the perma-root.
//! To allow breaking such cycles, we add the [`ReflectorRoot`] type.
//! It allows manually unrooting via [`ReflectorRoot::release`].
//!
//! [`ReflectorRoot`] internally uses an `Rc`, so besides an owned Root, which can be released on drop,
//! we can also have a [`WeakReflectorRoot`], which can be used to manually unroot and break cycles.

use std::cell::Cell;
use std::ffi::CStr;
use std::rc::{Rc, Weak};

use js::jsapi::{AddRawValueRoot, Heap, RemoveRawValueRoot};
use js::jsval::JSVal;
use js::rust::Runtime;
use malloc_size_of::{MallocSizeOf, MallocSizeOfOps};

use crate::script_runtime::JSContext;

#[derive(JSTraceable)]
#[cfg_attr(crown, crown::unrooted_must_root_lint::allow_unrooted_interior)]
struct ReflectorRootInternal {
    /// The rooted reflector value. While `rooted` is true an `AddRawValueRoot` registration is
    /// keyed on this cell's address, which is stable because the cell lives in a heap (`Rc`)
    /// allocation in `ReflectorRoot`.
    /// `value` can be assumed to be rooted for the life-time, since manual unrooting is unsafe,
    /// and requires the caller to promise that `value` will not be used after this point anymore.
    value: Heap<JSVal>,
    /// Whether `value` is rooted or not. Avoids double unroots.
    rooted: Cell<bool>,
}

#[derive(JSTraceable)]
pub struct ReflectorRoot {
    inner: Rc<ReflectorRootInternal>,
}

pub struct WeakReflectorRoot(Weak<ReflectorRootInternal>);

impl ReflectorRootInternal {
    /// Remove the root.
    ///
    /// # Safety
    ///
    /// Since this makes the Object eligible for GC, this may only be called when the caller
    /// is sure the object is not going to be used anymore.
    #[expect(unsafe_code)]
    unsafe fn release(&self) {
        if !self.rooted.replace(false) {
            return;
        }
        if let Some(cx) = Runtime::get() {
            unsafe { RemoveRawValueRoot(cx.as_ptr(), self.value.get_unsafe()) };
        }
    }
}

impl ReflectorRoot {
    /// Root `value` under `name`.
    #[expect(unsafe_code)]
    pub fn new(cx: &mut JSContext, value: JSVal, name: &CStr) -> Self {
        let inner = Rc::new(ReflectorRootInternal {
            value: Heap::default(),
            rooted: Cell::new(false),
        });
        inner.value.set(value);
        let root = ReflectorRoot { inner };
        unsafe {
            assert!(AddRawValueRoot(
                cx.raw_cx(),
                root.inner.value.get_unsafe(),
                name.as_ptr()
            ));
        }
        root.inner.rooted.set(true);
        root
    }

    /// Returns a weak reference to this ReflectorRoot.
    pub fn get_weak(&self) -> WeakReflectorRoot {
        WeakReflectorRoot(Rc::downgrade(&self.inner))
    }
}

impl Drop for ReflectorRoot {
    fn drop(&mut self) {
        // SAFETY: If the ReflectorRoot itself is dropped, then we trivially know
        // it's safe to release the root.
        unsafe { self.inner.release() };
    }
}

impl WeakReflectorRoot {
    /// Remove the root if the Reflector is still alive.
    ///
    /// # Safety
    ///
    /// Since this makes the Object eligible for GC, this may only be called when the caller
    /// is sure the object is not going to be used anymore.
    pub unsafe fn release(&self) {
        if let Some(reflector_root) = self.0.upgrade() {
            // SAFETY: The safety precondition was delegated to the caller of this method.
            unsafe { reflector_root.release() }
        }
    }

    /// Returns true if the weakly referenced `ReflectorRoot` is still live.
    pub fn live(&self) -> bool {
        self.0.strong_count() > 0
    }
}

impl MallocSizeOf for ReflectorRoot {
    fn size_of(&self, _ops: &mut MallocSizeOfOps) -> usize {
        // JS Heap is tracked in `mozjs`, so only count our owned internal struct.
        // Our `Rc` is not clonable, so we treat this as an owned heap value.
        size_of::<ReflectorRootInternal>()
    }
}

impl MallocSizeOf for WeakReflectorRoot {
    fn size_of(&self, _ops: &mut MallocSizeOfOps) -> usize {
        // Weak reference, nothing to count here.
        0
    }
}
