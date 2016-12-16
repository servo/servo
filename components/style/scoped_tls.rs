/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

#![allow(unsafe_code)]

use rayon;
use std::cell::{Ref, RefCell, RefMut};

/// Stack-scoped thread-local storage for rayon thread pools.

pub struct ScopedTLS<'a, T: Send> {
    pool: &'a rayon::ThreadPool,
    slots: Box<[RefCell<Option<T>>]>,
}

unsafe impl<'a, T: Send> Sync for ScopedTLS<'a, T> {}

impl<'a, T: Send> ScopedTLS<'a, T> {
    pub fn new(p: &'a rayon::ThreadPool) -> Self {
        let count = p.num_threads();
        let mut v = Vec::with_capacity(count);
        for _ in 0..count {
            v.push(RefCell::new(None));
        }

        ScopedTLS {
            pool: p,
            slots: v.into_boxed_slice(),
        }
    }

    pub fn borrow(&self) -> Ref<Option<T>> {
        let idx = self.pool.current_thread_index().unwrap();
        self.slots[idx].borrow()
    }

    pub fn borrow_mut(&self) -> RefMut<Option<T>> {
        let idx = self.pool.current_thread_index().unwrap();
        self.slots[idx].borrow_mut()
    }

    pub fn ensure<F: FnOnce() -> T>(&self, f: F) -> RefMut<T> {
        let mut opt = self.borrow_mut();
        if opt.is_none() {
            *opt = Some(f());
        }

        RefMut::map(opt, |x| x.as_mut().unwrap())
    }
}
