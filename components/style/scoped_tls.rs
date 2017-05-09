/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

//! Stack-scoped thread-local storage for rayon thread pools.

#![allow(unsafe_code)]
#![deny(missing_docs)]

use rayon;
use std::cell::{Ref, RefCell, RefMut};

/// A scoped TLS set, that is alive during the `'scope` lifetime.
///
/// We use this on Servo to construct thread-local contexts, but clear them once
/// we're done with restyling.
pub struct ScopedTLS<'scope, T: Send> {
    pool: &'scope rayon::ThreadPool,
    slots: Box<[RefCell<Option<T>>]>,
}

/// The scoped TLS is `Sync` because no more than one worker thread can access a
/// given slot.
unsafe impl<'scope, T: Send> Sync for ScopedTLS<'scope, T> {}

impl<'scope, T: Send> ScopedTLS<'scope, T> {
    /// Create a new scoped TLS that will last as long as this rayon threadpool
    /// reference.
    pub fn new(p: &'scope rayon::ThreadPool) -> Self {
        let count = p.current_num_threads();
        let mut v = Vec::with_capacity(count);
        for _ in 0..count {
            v.push(RefCell::new(None));
        }

        ScopedTLS {
            pool: p,
            slots: v.into_boxed_slice(),
        }
    }

    /// Return an immutable reference to the `Option<T>` that this thread owns.
    pub fn borrow(&self) -> Ref<Option<T>> {
        let idx = self.pool.current_thread_index().unwrap();
        self.slots[idx].borrow()
    }

    /// Return a mutable reference to the `Option<T>` that this thread owns.
    pub fn borrow_mut(&self) -> RefMut<Option<T>> {
        let idx = self.pool.current_thread_index().unwrap();
        self.slots[idx].borrow_mut()
    }

    /// Ensure that the current data this thread owns is initialized, or
    /// initialize it using `f`.
    pub fn ensure<F: FnOnce() -> T>(&self, f: F) -> RefMut<T> {
        let mut opt = self.borrow_mut();
        if opt.is_none() {
            *opt = Some(f());
        }

        RefMut::map(opt, |x| x.as_mut().unwrap())
    }

    /// Unsafe access to the slots. This can be used to access the TLS when
    /// the caller knows that the pool does not have access to the TLS.
    pub unsafe fn unsafe_get(&self) -> &[RefCell<Option<T>>] {
        &self.slots
    }
}
