/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

//! An implementation of re-entrant mutexes.
//!
//! Re-entrant mutexes are like mutexes, but where it is expected
//! that a single thread may own a lock more than once.

//! It provides the same interface as https://github.com/rust-lang/rust/blob/master/src/libstd/sys/common/remutex.rs
//! so if those types are ever exported, we should be able to replace this implemtation.

#![feature(nonzero)]

extern crate core;
#[macro_use] extern crate lazy_static;
#[macro_use] extern crate log;

use core::nonzero::NonZero;
use std::cell::{Cell, UnsafeCell};
use std::ops::Deref;
use std::sync::{LockResult, Mutex, MutexGuard, PoisonError, TryLockError, TryLockResult};
use std::sync::atomic::{AtomicUsize, Ordering};

/// A type for thread ids.

// TODO: can we use the thread-id crate for this?

#[derive(Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug)]
pub struct ThreadId(NonZero<usize>);

lazy_static!{ static ref THREAD_COUNT: AtomicUsize = AtomicUsize::new(1); }

impl ThreadId {
    #[allow(unsafe_code)]
    fn new() -> ThreadId {
        let number = THREAD_COUNT.fetch_add(1, Ordering::SeqCst);
        ThreadId(NonZero::new(number).unwrap())
    }
    pub fn current() -> ThreadId {
        THREAD_ID.with(|tls| tls.clone())
    }
}

thread_local!{ static THREAD_ID: ThreadId = ThreadId::new() }

/// A type for atomic storage of thread ids.
#[derive(Debug)]
pub struct AtomicOptThreadId(AtomicUsize);

impl AtomicOptThreadId {
    pub fn new() -> AtomicOptThreadId {
        AtomicOptThreadId(AtomicUsize::new(0))
    }
    pub fn store(&self, value: Option<ThreadId>, ordering: Ordering) {
        let number = value.map(|id| id.0.get()).unwrap_or(0);
        self.0.store(number, ordering);
    }
    #[allow(unsafe_code)]
    pub fn load(&self, ordering: Ordering) -> Option<ThreadId> {
        let number = self.0.load(ordering);
        NonZero::new(number).map(ThreadId)
    }
    #[allow(unsafe_code)]
    pub fn swap(&self, value: Option<ThreadId>, ordering: Ordering) -> Option<ThreadId> {
        let number = value.map(|id| id.0.get()).unwrap_or(0);
        let number = self.0.swap(number, ordering);
        NonZero::new(number).map(ThreadId)
    }
}

/// A type for hand-over-hand mutexes.
///
/// These support `lock` and `unlock` functions. `lock` blocks waiting to become the
/// mutex owner. `unlock` can only be called by the lock owner, and panics otherwise.
/// They have the same happens-before and poisoning semantics as `Mutex`.

// TODO: Can we use `raw_lock` and `raw_unlock` from `parking_lot`'s `Mutex` for this?

pub struct HandOverHandMutex {
    mutex: Mutex<()>,
    owner: AtomicOptThreadId,
    guard: UnsafeCell<Option<MutexGuard<'static, ()>>>,
}

impl HandOverHandMutex {
    pub fn new() -> HandOverHandMutex {
        HandOverHandMutex {
            mutex: Mutex::new(()),
            owner: AtomicOptThreadId::new(),
            guard: UnsafeCell::new(None),
        }
    }
    #[allow(unsafe_code)]
    unsafe fn set_guard_and_owner<'a>(&'a self, guard: MutexGuard<'a, ()>) {
        // The following two lines allow us to unsafely store
        // Some(guard): Option<MutexGuard<'a, ()>
        // in self.guard, even though its contents are Option<MutexGuard<'static, ()>>,
        // that is the lifetime is 'a not 'static.
        let guard_ptr = &mut *(self.guard.get() as *mut u8 as *mut Option<MutexGuard<'a, ()>>);
        *guard_ptr = Some(guard);
        self.owner.store(Some(ThreadId::current()), Ordering::Relaxed);
    }
    #[allow(unsafe_code)]
    unsafe fn unset_guard_and_owner(&self) {
        let guard_ptr = &mut *self.guard.get();
        let old_owner = self.owner();
        self.owner.store(None, Ordering::Relaxed);
        // Make sure we release the lock before checking the assertions.
        // We protect logging by a re-entrant lock, so we don't want
        // to do any incidental logging while we the lock is held.
        drop(guard_ptr.take());
        // Now we have released the lock, it's okay to use logging.
        assert_eq!(old_owner, Some(ThreadId::current()));
    }
    #[allow(unsafe_code)]
    pub fn lock<'a>(&'a self) -> LockResult<()> {
        let (guard, result) = match self.mutex.lock() {
            Ok(guard) => (guard, Ok(())),
            Err(err) => (err.into_inner(), Err(PoisonError::new(()))),
        };
        unsafe { self.set_guard_and_owner(guard); }
        result
    }
    #[allow(unsafe_code)]
    pub fn try_lock(&self) -> TryLockResult<()> {
        let (guard, result) = match self.mutex.try_lock() {
            Ok(guard) => (guard, Ok(())),
            Err(TryLockError::WouldBlock) => return Err(TryLockError::WouldBlock),
            Err(TryLockError::Poisoned(err)) => (err.into_inner(), Err(TryLockError::Poisoned(PoisonError::new(())))),
        };
        unsafe { self.set_guard_and_owner(guard); }
        result
    }
    #[allow(unsafe_code)]
    pub fn unlock(&self) {
        unsafe { self.unset_guard_and_owner(); }
    }
    pub fn owner(&self) -> Option<ThreadId> {
        self.owner.load(Ordering::Relaxed)
    }
}

#[allow(unsafe_code)]
unsafe impl Send for HandOverHandMutex {}

/// A type for re-entrant mutexes.
///
/// It provides the same interface as https://github.com/rust-lang/rust/blob/master/src/libstd/sys/common/remutex.rs

pub struct ReentrantMutex<T> {
    mutex: HandOverHandMutex,
    count: Cell<usize>,
    data: T,
}

#[allow(unsafe_code)]
unsafe impl<T> Sync for ReentrantMutex<T> where T: Send {}

impl<T> ReentrantMutex<T> {
    pub fn new(data: T) -> ReentrantMutex<T> {
        trace!("{:?} Creating new lock.", ThreadId::current());
        ReentrantMutex {
            mutex: HandOverHandMutex::new(),
            count: Cell::new(0),
            data: data,
        }
    }

    pub fn lock(&self) -> LockResult<ReentrantMutexGuard<T>> {
        trace!("{:?} Locking.", ThreadId::current());
        if self.mutex.owner() != Some(ThreadId::current()) {
            trace!("{:?} Becoming owner.", ThreadId::current());
            if let Err(_) = self.mutex.lock() {
                trace!("{:?} Poison!", ThreadId::current());
                return Err(PoisonError::new(self.mk_guard()));
            }
            trace!("{:?} Became owner.", ThreadId::current());
        }
        Ok(self.mk_guard())
    }

    pub fn try_lock(&self) -> TryLockResult<ReentrantMutexGuard<T>> {
        trace!("{:?} Try locking.", ThreadId::current());
        if self.mutex.owner() != Some(ThreadId::current()) {
            trace!("{:?} Becoming owner?", ThreadId::current());
            if let Err(err) = self.mutex.try_lock() {
                match err {
                    TryLockError::WouldBlock => {
                        trace!("{:?} Would block.", ThreadId::current());
                        return Err(TryLockError::WouldBlock)
                    },
                    TryLockError::Poisoned(_) => {
                        trace!("{:?} Poison!", ThreadId::current());
                        return Err(TryLockError::Poisoned(PoisonError::new(self.mk_guard())));
                    },
                }
            }
            trace!("{:?} Became owner.", ThreadId::current());
        }
        Ok(self.mk_guard())
    }

    fn unlock(&self) {
        trace!("{:?} Unlocking.", ThreadId::current());
        let count = self.count.get().checked_sub(1).expect("Underflowed lock count.");
        trace!("{:?} Decrementing count to {}.", ThreadId::current(), count);
        self.count.set(count);
        if count == 0 {
            trace!("{:?} Releasing mutex.", ThreadId::current());
            self.mutex.unlock();
        }
    }

    fn mk_guard(&self) -> ReentrantMutexGuard<T> {
        let count = self.count.get().checked_add(1).expect("Overflowed lock count.");
        trace!("{:?} Incrementing count to {}.", ThreadId::current(), count);
        self.count.set(count);
        ReentrantMutexGuard { mutex: self }
    }
}

#[must_use]
pub struct ReentrantMutexGuard<'a, T> where T: 'static {
    mutex: &'a ReentrantMutex<T>,
}

impl<'a, T> Drop for ReentrantMutexGuard<'a, T> {
    #[allow(unsafe_code)]
    fn drop(&mut self) {
        self.mutex.unlock()
    }
}

impl<'a, T> Deref for ReentrantMutexGuard<'a, T> {
    type Target = T;
    fn deref(&self) -> &T {
        &self.mutex.data
    }
}
