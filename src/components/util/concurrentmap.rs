/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

//! A Doug Lea-style concurrent hash map using striped locks.

use rand;
use rand::Rng;
use std::cast;
use std::hash::{Hash, sip};
use std::ptr;
use std::sync::atomics::{AtomicUint, Relaxed, SeqCst};
use std::unstable::mutex::StaticNativeMutex;
use std::mem;
use std::slice;

/// When the size exceeds (number of buckets * LOAD_NUMERATOR/LOAD_DENOMINATOR), the hash table
/// grows.
static LOAD_NUMERATOR: uint = 3;

/// When the size exceeds (number of buckets * LOAD_NUMERATOR/LOAD_DENOMINATOR), the hash table
/// grows.
static LOAD_DENOMINATOR: uint = 4;

/// One bucket in the hash table.
struct Bucket<K,V> {
    next: Option<~Bucket<K,V>>,
    key: K,
    value: V,
}

/// A concurrent hash map using striped locks.
pub struct ConcurrentHashMap<K,V> {
    /// The first secret key value.
    k0: u64,
    /// The second key value.
    k1: u64,
    /// The number of elements in this hash table.
    size: AtomicUint,
    /// The striped locks.
    locks: ~[StaticNativeMutex],
    /// The buckets.
    buckets: ~[Option<Bucket<K,V>>],
}

impl<K:Hash + Eq,V> ConcurrentHashMap<K,V> {
    /// Creates a hash map with 16 locks and 4 buckets per lock.
    pub fn new() -> ConcurrentHashMap<K,V> {
        ConcurrentHashMap::with_locks_and_buckets(16, 4)
    }

    /// Creates a hash map with the given number of locks and buckets per lock.
    pub fn with_locks_and_buckets(lock_count: uint, buckets_per_lock: uint)
                                  -> ConcurrentHashMap<K,V> {
        let mut rand = rand::task_rng();
        ConcurrentHashMap {
            k0: rand.gen(),
            k1: rand.gen(),
            size: AtomicUint::new(0),
            locks: slice::from_fn(lock_count, |_| {
                unsafe {
                    StaticNativeMutex::new()
                }
            }),
            buckets: slice::from_fn(lock_count * buckets_per_lock, |_| None),
        }
    }

    /// Inserts the given value into the hash table, replacing the value with the previous value
    /// if any.
    pub fn insert(&self, key: K, value: V) {
        unsafe {
            let this: &mut ConcurrentHashMap<K,V> = cast::transmute_mut(self);

            loop {
                let (bucket_index, lock_index) = self.bucket_and_lock_indices(&key);
                if this.overloaded() {
                    this.locks[lock_index].unlock_noguard();
                    this.try_resize(self.buckets_per_lock() * 2);

                    // Have to retry because the bucket and lock indices will have shifted.
                    continue
                }

                this.insert_unlocked(key, value, Some(bucket_index));
                this.locks[lock_index].unlock_noguard();
                break
            }
        }
    }

    #[inline(always)]
    unsafe fn insert_unlocked(&self, key: K, value: V, opt_bucket_index: Option<uint>) {
        let this: &mut ConcurrentHashMap<K,V> = cast::transmute_mut(self);

        let bucket_index = match opt_bucket_index {
            Some(bucket_index) => bucket_index,
            None => self.bucket_index_unlocked(&key),
        };

        match this.buckets[bucket_index] {
            None => {
                this.buckets[bucket_index] = Some(Bucket {
                    next: None,
                    key: key,
                    value: value,
                });
                drop(this.size.fetch_add(1, SeqCst));
            }
            Some(ref mut bucket) => {
                // Search to try to find a value.
                let mut bucket: *mut Bucket<K,V> = bucket;
                loop {
                    if (*bucket).key == key {
                        (*bucket).value = value;
                        break
                    }

                    match (*bucket).next {
                        None => {}
                        Some(ref mut next_bucket) => {
                            bucket = &mut **next_bucket as *mut Bucket<K,V>;
                            continue
                        }
                    }

                    (*bucket).next = Some(~Bucket {
                        next: None,
                        key: key,
                        value: value,
                    });
                    drop(this.size.fetch_add(1, SeqCst));
                    break
                }
            }
        }
    }

    /// Removes the given key from the hash table.
    pub fn remove(&self, key: &K) {
        let this: &mut ConcurrentHashMap<K,V> = unsafe {
            cast::transmute_mut(self)
        };

        let (bucket_index, lock_index) = self.bucket_and_lock_indices(key);

        // Rebuild the bucket.
        let mut nuke_bucket = false;
        match this.buckets[bucket_index] {
            None => {}
            Some(ref mut bucket) if bucket.key == *key => {
                // Common case (assuming a sparse table): If the key is the first one in the
                // chain, just copy the next fields over.
                let next_opt = mem::replace(&mut bucket.next, None);
                match next_opt {
                    None => nuke_bucket = true,
                    Some(~next) => *bucket = next,
                }
                drop(this.size.fetch_sub(1, SeqCst))
            }
            Some(ref mut bucket) => {
                // Rarer case: If the key is elsewhere in the chain (or nowhere), then search for
                // it and just stitch up pointers.
                let mut prev: *mut Bucket<K,V> = bucket;
                unsafe {
                    loop {
                        match (*prev).next {
                            None => break,  // Not found.
                            Some(ref mut bucket) => {
                                // Continue the search.
                                if bucket.key != *key {
                                    prev = &mut **bucket as *mut Bucket<K,V>;
                                    continue
                                }
                            }
                        }

                        // If we got here, then we found the key. Now do a pointer stitch.
                        let ~Bucket {
                            next: next_next,
                            ..
                        } = (*prev).next.take_unwrap();
                        (*prev).next = next_next;
                        drop(this.size.fetch_sub(1, SeqCst));
                        break
                    }
                }
            }
        }
        if nuke_bucket {
            this.buckets[bucket_index] = None
        }

        unsafe {
            this.locks[lock_index].unlock_noguard()
        }
    }

    /// Returns an iterator over this concurrent map.
    pub fn iter<'a>(&'a self) -> ConcurrentHashMapIterator<'a,K,V> {
        ConcurrentHashMapIterator {
            map: self,
            bucket_index: -1,
            current_bucket: ptr::null(),
        }
    }

    /// Returns true if the given key is in the map and false otherwise.
    pub fn contains_key(&self, key: &K) -> bool {
        let this: &mut ConcurrentHashMap<K,V> = unsafe {
            cast::transmute_mut(self)
        };

        let (bucket_index, lock_index) = this.bucket_and_lock_indices(key);

        let result;
        match this.buckets[bucket_index] {
            None => result = false,
            Some(ref bucket) => {
                // Search to try to find a value.
                let mut bucket = bucket;
                loop {
                    if bucket.key == *key {
                        result = true;
                        break
                    }
                    match bucket.next {
                        None => {
                            result = false;
                            break
                        }
                        Some(ref next_bucket) => bucket = &**next_bucket,
                    }
                }
            }
        }

        unsafe {
            this.locks[lock_index].unlock_noguard()
        }

        result
    }

    /// Removes all entries from the map.
    pub fn clear(&self) {
        let this: &mut ConcurrentHashMap<K,V> = unsafe {
            cast::transmute_mut(self)
        };

        let (bucket_count, lock_count) = (this.buckets.len(), this.locks.len());
        let buckets_per_lock = bucket_count / lock_count;

        let (mut lock_index, mut stripe_index) = (0, 0);
        for bucket in this.buckets.mut_iter() {
            stripe_index += 1;
            if stripe_index == buckets_per_lock {
                unsafe {
                    this.locks[lock_index].unlock_noguard();
                }

                stripe_index = 0;
                lock_index += 1
            }
            if stripe_index == 0 {
                unsafe {
                    this.locks[lock_index].lock_noguard()
                }
            }

            *bucket = None
        }
    }

    /// Resizes the map to a new size. Takes all the locks (i.e. acquires an exclusive lock on the
    /// entire table) as it does so.
    ///
    /// This has no problem with invalidating iterators because iterators always hold onto at least
    /// one lock.
    fn try_resize(&self, new_buckets_per_lock: uint) {
        let this: &mut ConcurrentHashMap<K,V> = unsafe {
            cast::transmute_mut(self)
        };

        // Take a lock on all buckets.
        for lock in this.locks.mut_iter() {
            unsafe {
                lock.lock_noguard()
            }
        }

        // Check to make sure we aren't already at the right size. Someone else could have already
        // resized.
        let lock_count = this.locks.len();
        let new_bucket_count = lock_count * new_buckets_per_lock;
        if new_bucket_count > this.buckets.len() {
            // Create a new set of buckets.
            let mut buckets = slice::from_fn(new_bucket_count, |_| None);
            mem::swap(&mut this.buckets, &mut buckets);
            this.size.store(0, Relaxed);

            // Go through all the old buckets and insert the new data.
            for bucket in buckets.move_iter() {
                match bucket {
                    None => continue,
                    Some(Bucket {
                        key: key,
                        value: value,
                        next: mut bucket
                    }) => {
                        unsafe {
                            this.insert_unlocked(key, value, None)
                        }

                        loop {
                            match bucket {
                                None => break,
                                Some(~Bucket {
                                    key: key,
                                    value: value,
                                    next: next
                                }) => {
                                    unsafe {
                                        this.insert_unlocked(key, value, None)
                                    }

                                    bucket = next
                                }
                            }
                        }
                    }
                }
            }
        }

        // Release all our locks.
        for lock in this.locks.mut_iter() {
            unsafe {
                lock.unlock_noguard()
            }
        }
    }

    /// Returns the index of the bucket and the lock for the given key, respectively, taking the
    /// appropriate lock before returning. This is subtle: it contains a loop to deal with race
    /// conditions in which the bucket array might have resized.
    #[inline]
    fn bucket_and_lock_indices(&self, key: &K) -> (uint, uint) {
        let this: &mut ConcurrentHashMap<K,V> = unsafe {
            cast::transmute_mut(cast::transmute_region(self))
        };

        let hash = sip::hash_with_keys(self.k0, self.k1, key);
        let lock_count = this.locks.len();
        let mut bucket_index;
        let mut lock_index;
        loop {
            let bucket_count = this.buckets.len();
            let buckets_per_lock = bucket_count / lock_count;
            bucket_index = hash as uint % bucket_count;
            lock_index = bucket_index / buckets_per_lock;
            unsafe {
                this.locks[lock_index].lock_noguard();
            }
            let new_bucket_count = this.buckets.len();
            if bucket_count == new_bucket_count {
                break
            }

            // If we got here, the hash table resized from under us: try again.
            unsafe {
                this.locks[lock_index].unlock_noguard()
            }
        }

        (bucket_index, lock_index)
    }

    /// Returns the index of the bucket. You must be holding at least one lock to call this
    /// function!
    #[inline]
    unsafe fn bucket_index_unlocked(&self, key: &K) -> uint {
        let hash = sip::hash_with_keys(self.k0, self.k1, key);
        hash as uint % self.buckets.len()
    }

    /// Returns true if this hash table is overloaded (at its current load factor, default 0.75)
    /// and false otherwise.
    #[inline]
    fn overloaded(&self) -> bool {
        self.size.load(SeqCst) >= (self.buckets.len() * LOAD_NUMERATOR / LOAD_DENOMINATOR)
    }

    /// Returns the number of buckets per lock.
    #[inline]
    fn buckets_per_lock(&self) -> uint {
        self.buckets.len() / self.locks.len()
    }

    /// Returns the number of elements in the hash table.
    #[inline]
    pub fn size(&self) -> uint {
        self.size.load(SeqCst)
    }
}

pub struct ConcurrentHashMapIterator<'a,K,V> {
    map: &'a ConcurrentHashMap<K,V>,
    bucket_index: int,
    current_bucket: *Bucket<K,V>,
}

impl<'a,K,V> Iterator<(&'a K, &'a V)> for ConcurrentHashMapIterator<'a,K,V> {
    fn next(&mut self) -> Option<(&'a K, &'a V)> {
        let map: &'a mut ConcurrentHashMap<K,V> = unsafe {
            cast::transmute_mut(self.map)
        };

        let (bucket_count, lock_count) = (map.buckets.len(), map.locks.len());
        let buckets_per_lock = bucket_count / lock_count;

        // Go to the next bucket in the chain, if necessary.
        if self.current_bucket != ptr::null() {
            unsafe {
                self.current_bucket = match (*self.current_bucket).next {
                    None => ptr::null(),
                    Some(ref bucket) => {
                        let bucket: *Bucket<K,V> = &**bucket;
                        bucket
                    }
                }
            }
        }

        // Advance buckets, taking locks along the way if necessary.
        while self.current_bucket == ptr::null() {
            let bucket_index = self.bucket_index;
            let lock_index = if bucket_index < 0 {
                -1
            } else {
                bucket_index / (buckets_per_lock as int)
            };

            if bucket_index < 0 ||
                    bucket_index % (buckets_per_lock as int) == (buckets_per_lock as int) - 1 {
                // We're at the boundary between one lock and another. Drop the old lock if
                // necessary and acquire the new one, if necessary.
                if bucket_index != -1 {
                    unsafe {
                        map.locks[lock_index as uint].unlock_noguard()
                    }
                }
                if bucket_index != (bucket_count as int) - 1 {
                    unsafe {
                        map.locks[(lock_index + 1) as uint].lock_noguard()
                    }
                }
            }

            // If at end, return None.
            if self.bucket_index == (bucket_count as int) - 1 {
                return None
            }

            self.bucket_index += 1;

            self.current_bucket = match map.buckets[self.bucket_index as uint] {
                None => ptr::null(),
                Some(ref bucket) => {
                    let bucket: *Bucket<K,V> = bucket;
                    bucket
                }
            }
        }

        unsafe {
            Some((cast::transmute(&(*self.current_bucket).key),
                  cast::transmute(&(*self.current_bucket).value)))
        }
    }
}

#[cfg(test)]
pub mod test {
    use sync::Arc;
    use native;
    use std::comm;

    use concurrentmap::ConcurrentHashMap;

    #[test]
    pub fn smoke() {
        let m = Arc::new(ConcurrentHashMap::new());
        let (chan, port) = comm::channel();

        // Big enough to make it resize once.
        for i in range(0, 5) {
            let m = m.clone();
            let chan = chan.clone();
            native::task::spawn(proc() {
                for j in range(i * 20, (i * 20) + 20) {
                    m.insert(j, j * j);
                }
                chan.send(());
            })
        }
        for _ in range(0, 5) {
            port.recv();
        }

        let mut count = 0;
        for (&k, &v) in m.iter() {
            assert_eq!(k * k, v)
            count += 1;
        }
        assert_eq!(count, 100)
    }
}

