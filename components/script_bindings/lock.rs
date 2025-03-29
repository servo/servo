/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use std::sync::OnceLock;

/// A OnceLock wrapping a type that is not considered threadsafe by the Rust compiler, but
/// will be used in a threadsafe manner (it will not be mutated, after being initialized).
///
/// This is needed to allow using JS API types (which usually involve raw pointers) in static initializers,
/// when Servo guarantees through the use of OnceLock that only one thread will ever initialize
/// the value.
pub struct ThreadUnsafeOnceLock<T>(OnceLock<T>);

impl<T> ThreadUnsafeOnceLock<T> {
    #[allow(clippy::new_without_default)]
    pub const fn new() -> Self {
        Self(OnceLock::new())
    }

    /// Initialize the value inside this lock. Panics if the lock has been previously initialized.
    pub fn set(&self, val: T) {
        assert!(self.0.set(val).is_ok());
    }

    /// Get a reference to the value inside this lock. Panics if the lock has not been initialized.
    ///
    /// # Safety
    ///   The caller must ensure that it does not mutate value contained inside this lock
    ///   (using interior mutability).
    pub unsafe fn get(&self) -> &T {
        self.0.get().unwrap()
    }
}

unsafe impl<T> Sync for ThreadUnsafeOnceLock<T> {}
unsafe impl<T> Send for ThreadUnsafeOnceLock<T> {}
