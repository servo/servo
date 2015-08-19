// Copyright 2013 The Rust Project Developers. See the COPYRIGHT
// file at the top-level directory of this distribution and at
// http://rust-lang.org/COPYRIGHT.
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

//! A (mostly) lock-free concurrent work-stealing deque
//!
//! This module contains an implementation of the Chase-Lev work stealing deque
//! described in "Dynamic Circular Work-Stealing Deque". The implementation is
//! heavily based on the pseudocode found in the paper.
//!
//! This implementation does not want to have the restriction of a garbage
//! collector for reclamation of buffers, and instead it uses a shared pool of
//! buffers. This shared pool is required for correctness in this
//! implementation.
//!
//! The only lock-synchronized portions of this deque are the buffer allocation
//! and deallocation portions. Otherwise all operations are lock-free.
//!
//! # Example
//!
//!     use util::deque::BufferPool;
//!
//!     let mut pool = BufferPool::new();
//!     let (mut worker, mut stealer) = pool.deque();
//!
//!     // Only the worker may push/pop
//!     worker.push(1i);
//!     worker.pop();
//!
//!     // Stealers take data from the other end of the deque
//!     worker.push(1i);
//!     stealer.steal();
//!
//!     // Stealers can be cloned to have many stealers stealing in parallel
//!     worker.push(1i);
//!     let mut stealer2 = stealer.clone();
//!     stealer2.steal();

// NB: the "buffer pool" strategy is not done for speed, but rather for
//     correctness. For more info, see the comment on `swap_buffer`

// FIXME: more than likely, more atomic operations than necessary use SeqCst

pub use self::Stolen::{Empty, Abort, Data};

use std::marker;
use std::mem::{forget, align_of, size_of, transmute};
use std::ptr;
use std::rt::heap::{allocate, deallocate};
use std::sync::Arc;

use std::sync::Mutex;
use std::sync::atomic::Ordering::{Relaxed, SeqCst};
use std::sync::atomic::{AtomicIsize, AtomicPtr};

// Once the queue is less than 1/K full, then it will be downsized. Note that
// the deque requires that this number be less than 2.
static K: isize = 4;

// Minimum number of bits that a buffer size should be. No buffer will resize to
// under this value, and all deques will initially contain a buffer of this
// size.
//
// The size in question is 1 << MIN_BITS
static MIN_BITS: usize = 7;

struct Deque<T> {
    bottom: AtomicIsize,
    top: AtomicIsize,
    array: AtomicPtr<Buffer<T>>,
    pool: BufferPool<T>,
}

unsafe impl<T> Send for Deque<T> {}

/// Worker half of the work-stealing deque. This worker has exclusive access to
/// one side of the deque, and uses `push` and `pop` method to manipulate it.
///
/// There may only be one worker per deque.
pub struct Worker<T> {
    deque: Arc<Deque<T>>,
}

impl<T> !marker::Sync for Worker<T> {}

/// The stealing half of the work-stealing deque. Stealers have access to the
/// opposite end of the deque from the worker, and they only have access to the
/// `steal` method.
pub struct Stealer<T> {
    deque: Arc<Deque<T>>,
}

impl<T> !marker::Sync for Stealer<T> {}

/// When stealing some data, this is an enumeration of the possible outcomes.
#[derive(PartialEq, Debug)]
pub enum Stolen<T> {
    /// The deque was empty at the time of stealing
    Empty,
    /// The stealer lost the race for stealing data, and a retry may return more
    /// data.
    Abort,
    /// The stealer has successfully stolen some data.
    Data(T),
}

/// The allocation pool for buffers used by work-stealing deques. Right now this
/// structure is used for reclamation of memory after it is no longer in use by
/// deques.
///
/// This data structure is protected by a mutex, but it is rarely used. Deques
/// will only use this structure when allocating a new buffer or deallocating a
/// previous one.
pub struct BufferPool<T> {
    // FIXME: This entire file was copied from std::sync::deque before it was removed,
    // I converted `Exclusive` to `Mutex` here, but that might not be ideal
    pool: Arc<Mutex<Vec<Box<Buffer<T>>>>>,
}

/// An internal buffer used by the chase-lev deque. This structure is actually
/// implemented as a circular buffer, and is used as the intermediate storage of
/// the data in the deque.
///
/// This type is implemented with *T instead of Vec<T> for two reasons:
///
///   1. There is nothing safe about using this buffer. This easily allows the
///      same value to be read twice in to rust, and there is nothing to
///      prevent this. The usage by the deque must ensure that one of the
///      values is forgotten. Furthermore, we only ever want to manually run
///      destructors for values in this buffer (on drop) because the bounds
///      are defined by the deque it's owned by.
///
///   2. We can certainly avoid bounds checks using *T instead of Vec<T>, although
///      LLVM is probably pretty good at doing this already.
struct Buffer<T> {
    storage: *const T,
    log_size: usize,
}

unsafe impl<T: 'static> Send for Buffer<T> { }

impl<T: Send + 'static> BufferPool<T> {
    /// Allocates a new buffer pool which in turn can be used to allocate new
    /// deques.
    pub fn new() -> BufferPool<T> {
        BufferPool { pool: Arc::new(Mutex::new(Vec::new())) }
    }

    /// Allocates a new work-stealing deque which will send/receiving memory to
    /// and from this buffer pool.
    pub fn deque(&self) -> (Worker<T>, Stealer<T>) {
        let a = Arc::new(Deque::new(self.clone()));
        let b = a.clone();
        (Worker { deque: a }, Stealer { deque: b })
    }

    fn alloc(&mut self, bits: usize) -> Box<Buffer<T>> {
        unsafe {
            let mut pool = self.pool.lock().unwrap();
            match pool.iter().position(|x| x.size() >= (1 << bits)) {
                Some(i) => pool.remove(i),
                None => box Buffer::new(bits)
            }
        }
    }
}

impl<T> BufferPool<T> {
    fn free(&self, buf: Box<Buffer<T>>) {
        let mut pool = self.pool.lock().unwrap();
        match pool.iter().position(|v| v.size() > buf.size()) {
            Some(i) => pool.insert(i, buf),
            None => pool.push(buf),
        }
    }
}

impl<T: Send> Clone for BufferPool<T> {
    fn clone(&self) -> BufferPool<T> { BufferPool { pool: self.pool.clone() } }
}

impl<T: Send + 'static> Worker<T> {
    /// Pushes data onto the front of this work queue.
    pub fn push(&self, t: T) {
        unsafe { self.deque.push(t) }
    }
    /// Pops data off the front of the work queue, returning `None` on an empty
    /// queue.
    pub fn pop(&self) -> Option<T> {
        unsafe { self.deque.pop() }
    }

    /// Gets access to the buffer pool that this worker is attached to. This can
    /// be used to create more deques which share the same buffer pool as this
    /// deque.
    pub fn pool<'a>(&'a self) -> &'a BufferPool<T> {
        &self.deque.pool
    }
}

impl<T: Send + 'static> Stealer<T> {
    /// Steals work off the end of the queue (opposite of the worker's end)
    pub fn steal(&self) -> Stolen<T> {
        unsafe { self.deque.steal() }
    }

    /// Gets access to the buffer pool that this stealer is attached to. This
    /// can be used to create more deques which share the same buffer pool as
    /// this deque.
    pub fn pool<'a>(&'a self) -> &'a BufferPool<T> {
        &self.deque.pool
    }
}

impl<T: Send> Clone for Stealer<T> {
    fn clone(&self) -> Stealer<T> {
        Stealer { deque: self.deque.clone() }
    }
}

// Almost all of this code can be found directly in the paper so I'm not
// personally going to heavily comment what's going on here.

impl<T: Send + 'static> Deque<T> {
    fn new(mut pool: BufferPool<T>) -> Deque<T> {
        let buf = pool.alloc(MIN_BITS);
        Deque {
            bottom: AtomicIsize::new(0),
            top: AtomicIsize::new(0),
            array: AtomicPtr::new(unsafe { transmute(buf) }),
            pool: pool,
        }
    }

    unsafe fn push(&self, data: T) {
        let mut b = self.bottom.load(Relaxed);
        let t = self.top.load(Relaxed);
        let mut a = self.array.load(SeqCst);
        let size = b - t;
        if size >= (*a).size() - 1 {
            // You won't find this code in the chase-lev deque paper. This is
            // alluded to in a small footnote, however. We always free a buffer
            // when growing in order to prevent leaks.
            a = self.swap_buffer(b, a, (*a).resize(b, t, 1));
            b = self.bottom.load(SeqCst);
        }
        (*a).put(b, data);
        self.bottom.store(b + 1, SeqCst);
    }

    unsafe fn pop(&self) -> Option<T> {
        let b = self.bottom.load(Relaxed);
        let a = self.array.load(SeqCst);
        let b = b - 1;
        self.bottom.store(b, SeqCst);
        let t = self.top.load(Relaxed);
        let size = b - t;
        if size < 0 {
            self.bottom.store(t, SeqCst);
            return None;
        }
        let data = (*a).get(b);
        if size > 0 {
            self.maybe_shrink(b, t);
            return Some(data);
        }
        if self.top.compare_and_swap(t, t + 1, SeqCst) == t {
            self.bottom.store(t + 1, SeqCst);
            return Some(data);
        } else {
            self.bottom.store(t + 1, SeqCst);
            forget(data); // someone else stole this value
            return None;
        }
    }

    unsafe fn steal(&self) -> Stolen<T> {
        let t = self.top.load(Relaxed);
        let old = self.array.load(Relaxed);
        let b = self.bottom.load(Relaxed);
        let size = b - t;
        if size <= 0 { return Empty }
        let a = self.array.load(SeqCst);
        if size % (*a).size() == 0 {
            if a == old && t == self.top.load(SeqCst) {
                return Empty
            }
            return Abort
        }
        let data = (*a).get(t);
        if self.top.compare_and_swap(t, t + 1, SeqCst) == t {
            Data(data)
        } else {
            forget(data); // someone else stole this value
            Abort
        }
    }

    unsafe fn maybe_shrink(&self, b: isize, t: isize) {
        let a = self.array.load(SeqCst);
        if b - t < (*a).size() / K && b - t > (1 << MIN_BITS) {
            self.swap_buffer(b, a, (*a).resize(b, t, -1));
        }
    }

    // Helper routine not mentioned in the paper which is used in growing and
    // shrinking buffers to swap in a new buffer into place. As a bit of a
    // recap, the whole point that we need a buffer pool rather than just
    // calling malloc/free directly is that stealers can continue using buffers
    // after this method has called 'free' on it. The continued usage is simply
    // a read followed by a forget, but we must make sure that the memory can
    // continue to be read after we flag this buffer for reclamation.
    unsafe fn swap_buffer(&self, b: isize, old: *mut Buffer<T>,
                          buf: Buffer<T>) -> *mut Buffer<T> {
        let newbuf: *mut Buffer<T> = transmute(box buf);
        self.array.store(newbuf, SeqCst);
        let ss = (*newbuf).size();
        self.bottom.store(b + ss, SeqCst);
        let t = self.top.load(SeqCst);
        if self.top.compare_and_swap(t, t + ss, SeqCst) != t {
            self.bottom.store(b, SeqCst);
        }
        self.pool.free(transmute(old));
        return newbuf;
    }
}


impl<T> Drop for Deque<T> {
    fn drop(&mut self) {
        let t = self.top.load(SeqCst);
        let b = self.bottom.load(SeqCst);
        let a = self.array.load(SeqCst);
        // Free whatever is leftover in the dequeue, and then move the buffer
        // back into the pool.
        for i in t..b {
            let _: T = unsafe { (*a).get(i) };
        }
        self.pool.free(unsafe { transmute(a) });
    }
}

#[inline]
fn buffer_alloc_size<T>(log_size: usize) -> usize {
    (1 << log_size) * size_of::<T>()
}

impl<T> Buffer<T> {
    unsafe fn new(log_size: usize) -> Buffer<T> {
        let size = buffer_alloc_size::<T>(log_size);
        let buffer = allocate(size, align_of::<T>());
        if buffer.is_null() { ::alloc::oom() }
        Buffer {
            storage: buffer as *const T,
            log_size: log_size,
        }
    }

    fn size(&self) -> isize { 1 << self.log_size }

    // Apparently LLVM cannot optimize (foo % (1 << bar)) into this implicitly
    fn mask(&self) -> isize { (1 << self.log_size) - 1 }

    unsafe fn elem(&self, i: isize) -> *const T {
        self.storage.offset(i & self.mask())
    }

    // This does not protect against loading duplicate values of the same cell,
    // nor does this clear out the contents contained within. Hence, this is a
    // very unsafe method which the caller needs to treat specially in case a
    // race is lost.
    unsafe fn get(&self, i: isize) -> T {
        ptr::read(self.elem(i))
    }

    // Unsafe because this unsafely overwrites possibly uninitialized or
    // initialized data.
    unsafe fn put(&self, i: isize, t: T) {
        ptr::write(self.elem(i) as *mut T, t);
    }

    // Again, unsafe because this has incredibly dubious ownership violations.
    // It is assumed that this buffer is immediately dropped.
    unsafe fn resize(&self, b: isize, t: isize, delta: isize) -> Buffer<T> {
        // NB: not entirely obvious, but thanks to 2's complement,
        // casting delta to usize and then adding gives the desired
        // effect.
        let buf = Buffer::new(self.log_size.wrapping_add(delta as usize));
        for i in t..b {
            buf.put(i, self.get(i));
        }
        return buf;
    }
}

impl<T> Drop for Buffer<T> {
    fn drop(&mut self) {
        // It is assumed that all buffers are empty on drop.
        let size = buffer_alloc_size::<T>(self.log_size);
        unsafe { deallocate(self.storage as *mut u8, size, align_of::<T>()) }
    }
}
