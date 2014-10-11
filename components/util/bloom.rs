/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

//! Simple counting bloom filters.

use string_cache::{Atom, Namespace};

static KEY_SIZE: uint = 12;
static ARRAY_SIZE: uint = 1 << KEY_SIZE;
static KEY_MASK: u32 = (1 << KEY_SIZE) - 1;
static KEY_SHIFT: uint = 16;

/// A counting Bloom filter with 8-bit counters.  For now we assume
/// that having two hash functions is enough, but we may revisit that
/// decision later.
///
/// The filter uses an array with 2**KeySize entries.
///
/// Assuming a well-distributed hash function, a Bloom filter with
/// array size M containing N elements and
/// using k hash function has expected false positive rate exactly
///
/// $  (1 - (1 - 1/M)^{kN})^k  $
///
/// because each array slot has a
///
/// $  (1 - 1/M)^{kN}  $
///
/// chance of being 0, and the expected false positive rate is the
/// probability that all of the k hash functions will hit a nonzero
/// slot.
///
/// For reasonable assumptions (M large, kN large, which should both
/// hold if we're worried about false positives) about M and kN this
/// becomes approximately
///
/// $$  (1 - \exp(-kN/M))^k   $$
///
/// For our special case of k == 2, that's $(1 - \exp(-2N/M))^2$,
/// or in other words
///
/// $$    N/M = -0.5 * \ln(1 - \sqrt(r))   $$
///
/// where r is the false positive rate.  This can be used to compute
/// the desired KeySize for a given load N and false positive rate r.
///
/// If N/M is assumed small, then the false positive rate can
/// further be approximated as 4*N^2/M^2.  So increasing KeySize by
/// 1, which doubles M, reduces the false positive rate by about a
/// factor of 4, and a false positive rate of 1% corresponds to
/// about M/N == 20.
///
/// What this means in practice is that for a few hundred keys using a
/// KeySize of 12 gives false positive rates on the order of 0.25-4%.
///
/// Similarly, using a KeySize of 10 would lead to a 4% false
/// positive rate for N == 100 and to quite bad false positive
/// rates for larger N.
pub struct BloomFilter {
    counters: [u8, ..ARRAY_SIZE],
}

impl Clone for BloomFilter {
    #[inline]
    fn clone(&self) -> BloomFilter {
        BloomFilter {
            counters: self.counters,
        }
    }
}

impl BloomFilter {
    /// Creates a new bloom filter.
    #[inline]
    pub fn new() -> BloomFilter {
        BloomFilter {
            counters: [0, ..ARRAY_SIZE],
        }
    }

    #[inline]
    fn first_slot(&self, hash: u32) -> &u8 {
        &self.counters[hash1(hash) as uint]
    }

    #[inline]
    fn first_mut_slot(&mut self, hash: u32) -> &mut u8 {
        &mut self.counters[hash1(hash) as uint]
    }

    #[inline]
    fn second_slot(&self, hash: u32) -> &u8 {
        &self.counters[hash2(hash) as uint]
    }

    #[inline]
    fn second_mut_slot(&mut self, hash: u32) -> &mut u8 {
        &mut self.counters[hash2(hash) as uint]
    }

    #[inline]
    pub fn clear(&mut self) {
        self.counters = [0, ..ARRAY_SIZE]
    }

    #[inline]
    fn insert_hash(&mut self, hash: u32) {
        {
            let slot1 = self.first_mut_slot(hash);
            if !full(slot1) {
                *slot1 += 1
            }
        }
        {
            let slot2 = self.second_mut_slot(hash);
            if !full(slot2) {
                *slot2 += 1
            }
        }
    }

    /// Inserts an item into the bloom filter.
    #[inline]
    pub fn insert<T:BloomHash>(&mut self, elem: &T) {
        self.insert_hash(elem.bloom_hash())

    }

    #[inline]
    fn remove_hash(&mut self, hash: u32) {
        {
            let slot1 = self.first_mut_slot(hash);
            if !full(slot1) {
                *slot1 -= 1
            }
        }
        {
            let slot2 = self.second_mut_slot(hash);
            if !full(slot2) {
                *slot2 -= 1
            }
        }
    }

    /// Removes an item from the bloom filter.
    #[inline]
    pub fn remove<T:BloomHash>(&mut self, elem: &T) {
        self.remove_hash(elem.bloom_hash())
    }

    #[inline]
    fn might_contain_hash(&self, hash: u32) -> bool {
        *self.first_slot(hash) != 0 && *self.second_slot(hash) != 0
    }

    /// Check whether the filter might contain an item.  This can
    /// sometimes return true even if the item is not in the filter,
    /// but will never return false for items that are actually in the
    /// filter.
    #[inline]
    pub fn might_contain<T:BloomHash>(&self, elem: &T) -> bool {
        self.might_contain_hash(elem.bloom_hash())
    }
}

pub trait BloomHash {
    fn bloom_hash(&self) -> u32;
}

impl BloomHash for int {
    #[inline]
    fn bloom_hash(&self) -> u32 {
        ((*self >> 32) ^ *self) as u32
    }
}

impl BloomHash for uint {
    #[inline]
    fn bloom_hash(&self) -> u32 {
        ((*self >> 32) ^ *self) as u32
    }
}

impl BloomHash for Atom {
    #[inline]
    fn bloom_hash(&self) -> u32 {
        ((self.data >> 32) ^ self.data) as u32
    }
}

impl BloomHash for Namespace {
    #[inline]
    fn bloom_hash(&self) -> u32 {
        let Namespace(ref atom) = *self;
        atom.bloom_hash()
    }
}

#[inline]
fn full(slot: &u8) -> bool {
    *slot == 0xff
}

#[inline]
fn hash1(hash: u32) -> u32 {
    hash & KEY_MASK
}

#[inline]
fn hash2(hash: u32) -> u32 {
    (hash >> KEY_SHIFT) & KEY_MASK
}

#[test]
fn create_and_insert_some_stuff() {
    use std::iter::range;

    let mut bf = BloomFilter::new();

    for i in range(0u, 1000) {
        bf.insert(&i);
    }

    for i in range(0u, 1000) {
        assert!(bf.might_contain(&i));
    }

    let false_positives =
        range(1001u, 2000).filter(|i| bf.might_contain(i)).count();

    assert!(false_positives < 10) // 1%.

    for i in range(0u, 100) {
        bf.remove(&i);
    }

    for i in range(100u, 1000) {
        assert!(bf.might_contain(&i));
    }

    let false_positives = range(0u, 100).filter(|i| bf.might_contain(i)).count();

    assert!(false_positives < 2); // 2%.

    bf.clear();

    for i in range(0u, 2000) {
        assert!(!bf.might_contain(&i));
    }
}

#[cfg(test)]
mod bench {
    extern crate test;

    use std::hash::hash;
    use std::iter;
    use super::BloomFilter;

    #[bench]
    fn create_insert_1000_remove_100_lookup_100(b: &mut test::Bencher) {
        b.iter(|| {
            let mut bf = BloomFilter::new();
            for i in iter::range(0u, 1000) {
                bf.insert(&i);
            }
            for i in iter::range(0u, 100) {
                bf.remove(&i);
            }
            for i in iter::range(100u, 200) {
                test::black_box(bf.might_contain(&i));
            }
        });
    }

    #[bench]
    fn might_contain(b: &mut test::Bencher) {
        let mut bf = BloomFilter::new();

        for i in iter::range(0u, 1000) {
            bf.insert(&i);
        }

        let mut i = 0u;

        b.bench_n(1000, |b| {
            b.iter(|| {
                test::black_box(bf.might_contain(&i));
                i += 1;
            });
        });
    }

    #[bench]
    fn insert(b: &mut test::Bencher) {
        let mut bf = BloomFilter::new();

        b.bench_n(1000, |b| {
            let mut i = 0u;

            b.iter(|| {
                test::black_box(bf.insert(&i));
                i += 1;
            });
        });
    }

    #[bench]
    fn remove(b: &mut test::Bencher) {
        let mut bf = BloomFilter::new();
        for i in range(0u, 1000) {
            bf.insert(&i);
        }

        b.bench_n(1000, |b| {
            let mut i = 0u;

            b.iter(|| {
                bf.remove(&i);
                i += 1;
            });
        });

        test::black_box(bf.might_contain(&0u));
    }

    #[bench]
    fn hash_a_uint(b: &mut test::Bencher) {
        let mut i = 0u;
        b.iter(|| {
            test::black_box(hash(&i));
            i += 1;
        })
    }
}

