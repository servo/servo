/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

//! Simple counting bloom filters.

use fnv::FnvHasher;
use std::hash::{Hash, Hasher};

// The top 12 bits of the 32-bit hash value are not used by the bloom filter.
// Consumers may rely on this to pack hashes more efficiently.
pub const BLOOM_HASH_MASK: u32 = 0x00ffffff;
const KEY_SIZE: usize = 12;

const ARRAY_SIZE: usize = 1 << KEY_SIZE;
const KEY_MASK: u32 = (1 << KEY_SIZE) - 1;

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
    counters: [u8; ARRAY_SIZE],
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
            counters: [0; ARRAY_SIZE],
        }
    }

    #[inline]
    fn first_slot(&self, hash: u32) -> &u8 {
        &self.counters[hash1(hash) as usize]
    }

    #[inline]
    fn first_mut_slot(&mut self, hash: u32) -> &mut u8 {
        &mut self.counters[hash1(hash) as usize]
    }

    #[inline]
    fn second_slot(&self, hash: u32) -> &u8 {
        &self.counters[hash2(hash) as usize]
    }

    #[inline]
    fn second_mut_slot(&mut self, hash: u32) -> &mut u8 {
        &mut self.counters[hash2(hash) as usize]
    }

    #[inline]
    pub fn clear(&mut self) {
        self.counters = [0; ARRAY_SIZE]
    }

    // Slow linear accessor to make sure the bloom filter is zeroed. This should
    // never be used in release builds.
    #[cfg(debug_assertions)]
    pub fn is_zeroed(&self) -> bool {
        self.counters.iter().all(|x| *x == 0)
    }

    #[cfg(not(debug_assertions))]
    pub fn is_zeroed(&self) -> bool {
        unreachable!()
    }

    #[inline]
    pub fn insert_hash(&mut self, hash: u32) {
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
    pub fn insert<T: Hash>(&mut self, elem: &T) {
        self.insert_hash(hash(elem))
    }

    #[inline]
    pub fn remove_hash(&mut self, hash: u32) {
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
    pub fn remove<T: Hash>(&mut self, elem: &T) {
        self.remove_hash(hash(elem))
    }

    #[inline]
    pub fn might_contain_hash(&self, hash: u32) -> bool {
        *self.first_slot(hash) != 0 && *self.second_slot(hash) != 0
    }

    /// Check whether the filter might contain an item.  This can
    /// sometimes return true even if the item is not in the filter,
    /// but will never return false for items that are actually in the
    /// filter.
    #[inline]
    pub fn might_contain<T: Hash>(&self, elem: &T) -> bool {
        self.might_contain_hash(hash(elem))
    }
}

#[inline]
fn full(slot: &u8) -> bool {
    *slot == 0xff
}

fn hash<T: Hash>(elem: &T) -> u32 {
    let mut hasher = FnvHasher::default();
    elem.hash(&mut hasher);
    let hash: u64 = hasher.finish();
    (hash >> 32) as u32 ^ (hash as u32)
}

#[inline]
fn hash1(hash: u32) -> u32 {
    hash & KEY_MASK
}

#[inline]
fn hash2(hash: u32) -> u32 {
    (hash >> KEY_SIZE) & KEY_MASK
}

#[test]
fn create_and_insert_some_stuff() {
    let mut bf = BloomFilter::new();

    for i in 0_usize .. 1000 {
        bf.insert(&i);
    }

    for i in 0_usize .. 1000 {
        assert!(bf.might_contain(&i));
    }

    let false_positives =
        (1001_usize .. 2000).filter(|i| bf.might_contain(i)).count();

    assert!(false_positives < 150, "{} is not < 150", false_positives); // 15%.

    for i in 0_usize .. 100 {
        bf.remove(&i);
    }

    for i in 100_usize .. 1000 {
        assert!(bf.might_contain(&i));
    }

    let false_positives = (0_usize .. 100).filter(|i| bf.might_contain(i)).count();

    assert!(false_positives < 20, "{} is not < 20", false_positives); // 20%.

    bf.clear();

    for i in 0_usize .. 2000 {
        assert!(!bf.might_contain(&i));
    }
}

#[cfg(feature = "unstable")]
#[cfg(test)]
mod bench {
    extern crate test;
    use super::BloomFilter;

    #[derive(Default)]
    struct HashGenerator(u32);

    impl HashGenerator {
        fn next(&mut self) -> u32 {
            // Each hash is split into two twelve-bit segments, which are used
            // as an index into an array. We increment each by 64 so that we
            // hit the next cache line, and then another 1 so that our wrapping
            // behavior leads us to different entries each time.
            //
            // Trying to simulate cold caches is rather difficult with the cargo
            // benchmarking setup, so it may all be moot depending on the number
            // of iterations that end up being run. But we might as well.
            self.0 += (65) + (65 << super::KEY_SIZE);
            self.0
        }
    }

    #[bench]
    fn create_insert_1000_remove_100_lookup_100(b: &mut test::Bencher) {
        b.iter(|| {
            let mut gen1 = HashGenerator::default();
            let mut gen2 = HashGenerator::default();
            let mut bf = BloomFilter::new();
            for _ in 0_usize .. 1000 {
                bf.insert_hash(gen1.next());
            }
            for _ in 0_usize .. 100 {
                bf.remove_hash(gen2.next());
            }
            for _ in 100_usize .. 200 {
                test::black_box(bf.might_contain_hash(gen2.next()));
            }
        });
    }

    #[bench]
    fn might_contain_10(b: &mut test::Bencher) {
        let bf = BloomFilter::new();
        let mut gen = HashGenerator::default();
        b.iter(|| for _ in 0..10 {
            test::black_box(bf.might_contain_hash(gen.next()));
        });
    }

    #[bench]
    fn clear(b: &mut test::Bencher) {
        let mut bf = Box::new(BloomFilter::new());
        b.iter(|| test::black_box(&mut bf).clear());
    }

    #[bench]
    fn insert_10(b: &mut test::Bencher) {
        let mut bf = BloomFilter::new();
        let mut gen = HashGenerator::default();
        b.iter(|| for _ in 0..10 {
            test::black_box(bf.insert_hash(gen.next()));
        });
    }

    #[bench]
    fn remove_10(b: &mut test::Bencher) {
        let mut bf = BloomFilter::new();
        let mut gen = HashGenerator::default();
        // Note: this will underflow, and that's ok.
        b.iter(|| for _ in 0..10 {
            bf.remove_hash(gen.next())
        });
    }
}
