/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

//! Stack-scoped thread-local storage for rayon thread pools.

#![allow(unsafe_code)]
#![deny(missing_docs)]

use rayon;
use std::cell::{Ref, RefCell, RefMut};
use std::ops::DerefMut;

/// A scoped TLS set, that is alive during the `'scope` lifetime.
///
/// We use this on Servo to construct thread-local contexts, but clear them once
/// we're done with restyling.
///
/// Note that the cleanup is done on the thread that owns the scoped TLS, thus
/// the Send bound.
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
    /// initialize it using `f`.  We want ensure() to be fast and inline, and we
    /// want to inline the memmove that initializes the Option<T>.  But we don't
    /// want to inline space for the entire large T struct in our stack frame.
    /// That's why we hand `f` a mutable borrow to write to instead of just
    /// having it return a T.
    #[inline(always)]
    pub fn ensure<F: FnOnce(&mut Option<T>)>(&self, f: F) -> RefMut<T> {
        let mut opt = self.borrow_mut();
        if opt.is_none() {
            f(opt.deref_mut());
        }

        RefMut::map(opt, |x| x.as_mut().unwrap())
    }

    /// Returns the slots, consuming the scope.
    pub fn into_slots(self) -> Box<[RefCell<Option<T>>]> {
        self.slots
    }
}
