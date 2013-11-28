/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

//! Utility functions for concurrency-related synchronization.

use extra::arc::MutexArc;
use std::cast;

pub trait MutexArcUtils<T> {
    /// Accesses a `MutexArc` without the use of `unsafe`.
    ///
    /// I don't find the possibility of memory leaks so bad that they require `unsafe` personally
    /// (as you can always leak), and using `unsafe` would be so noisy that it would lose the
    /// benefit of `unsafe`.
    ///
    /// FIXME(pcwalton): Upstream to the Rust standard library.
    /// FIXME(pcwalton): Convert to RAII.
    fn force_access<U>(&self, f: &fn(&mut T) -> U) -> U;
}

impl<T:Send> MutexArcUtils<T> for MutexArc<T> {
    #[inline(always)]
    fn force_access<U>(&self, f: &fn(&mut T) -> U) -> U {
        unsafe {
            let this: &MutexArc<*()> = cast::transmute(self);
            let f: &fn(&mut *()) -> U = cast::transmute(f);
            this.access(f)
        }
    }
}

