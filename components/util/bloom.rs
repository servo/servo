/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

//! Simple counting bloom filters.

extern crate rand;

use fnv::{FnvState, hash};
use rand::Rng;
use std::hash::Hash;
use std::iter;
use std::num;
use std::uint;

// Just a quick and dirty xxhash embedding.

/// A counting bloom filter.
///
/// A bloom filter is a probabilistic data structure which allows you to add and
/// remove elements from a set, query the set for whether it may contain an
/// element or definitely exclude it, and uses much less ram than an equivalent
/// hashtable.
#[deriving(Clone)]
pub struct BloomFilter {
    buf: Vec<uint>,
    number_of_insertions: uint,
}

// Here's where some of the magic numbers came from:
//
// m = number of elements in the filter
// n = size of the filter
// k = number of hash functions
//
// p = Pr[false positive] = 0.01 false positive rate
//
// if we have an estimation of the number of elements in the bloom filter, we
// know m.
//
// p = (1 - exp(-kn/m))^k
// k = (m/n)ln2
// lnp = -(m/n)(ln2)^2
// m = -nlnp/(ln2)^2
// => n = -m(ln2)^2/lnp
//     ~= 10*m
//
// k = (m/n)ln2 = 10ln2 ~= 7

static NUMBER_OF_HASHES: uint = 7;

static BITS_PER_BUCKET: uint = 4;
static BUCKETS_PER_WORD: uint = uint::BITS / BITS_PER_BUCKET;

/// Returns a tuple of (array index, lsr shift amount) to get to the bits you
/// need. Don't forget to mask with 0xF!
fn bucket_index_to_array_index(bucket_index: uint) -> (uint, uint) {
    let arr_index = bucket_index / BUCKETS_PER_WORD;
    let shift_amount = (bucket_index % BUCKETS_PER_WORD) * BITS_PER_BUCKET;
    (arr_index, shift_amount)
}

// Key Stretching
// ==============
//
// Siphash is expensive. Instead of running it `NUMBER_OF_HASHES`, which would
// be a pretty big hit on performance, we just use it to see a non-cryptographic
// random number generator. This stretches the hash to get us our
// `NUMBER_OF_HASHES` array indicies.
//
// A hash is a `u64` and comes from SipHash.
// A shash is a `uint` stretched hash which comes from the XorShiftRng.

fn to_rng(hash: u64) -> rand::XorShiftRng {
    let bottom = (hash & 0xFFFFFFFF) as u32;
    let top    = ((hash >> 32) & 0xFFFFFFFF) as u32;
    rand::SeedableRng::from_seed([ 0x97830e05, 0x113ba7bb, bottom, top ])
}

fn stretch<'a>(r: &'a mut rand::XorShiftRng)
  -> iter::Take<rand::Generator<'a, uint, rand::XorShiftRng>> {
    r.gen_iter().take(NUMBER_OF_HASHES)
}

impl BloomFilter {
    /// This bloom filter is tuned to have ~1% false positive rate. In exchange
    /// for this guarantee, you need to have a reasonable upper bound on the
    /// number of elements that will ever be inserted into it. If you guess too
    /// low, your false positive rate will suffer. If you guess too high, you'll
    /// use more memory than is really necessary.
    pub fn new(expected_number_of_insertions: uint) -> BloomFilter {
        let size_in_buckets = 10 * expected_number_of_insertions;

        let size_in_words = size_in_buckets / BUCKETS_PER_WORD;

        let nonzero_size = if size_in_words == 0 { 1 } else { size_in_words };

        let num_words =
            num::checked_next_power_of_two(nonzero_size)
            .unwrap();

        BloomFilter {
            buf: Vec::from_elem(num_words, 0),
            number_of_insertions: 0,
        }
    }

    /// Since the array length must be a power of two, this will return a
    /// bitmask that can be `&`ed with a number to bring it into the range of
    /// the array.
    fn mask(&self) -> uint {
        (self.buf.len()*BUCKETS_PER_WORD) - 1 // guaranteed to be a power of two
    }

    /// Converts a stretched hash into a bucket index.
    fn shash_to_bucket_index(&self, shash: uint) -> uint {
        shash & self.mask()
    }

    /// Converts a stretched hash into an array and bit index. See the comment
    /// on `bucket_index_to_array_index` for details about the return value.
    fn shash_to_array_index(&self, shash: uint) -> (uint, uint) {
        bucket_index_to_array_index(self.shash_to_bucket_index(shash))
    }

    /// Gets the value at a given bucket.
    fn bucket_get(&self, a_idx: uint, shift_amount: uint) -> uint {
        let array_val = self.buf[a_idx];
        (array_val >> shift_amount) & 0xF
    }

    /// Sets the value at a given bucket. This will not bounds check, but that's
    /// ok because you've called `bucket_get` first, anyhow.
    fn bucket_set(&mut self, a_idx: uint, shift_amount: uint, new_val: uint) {
        // We can avoid bounds checking here since in order to do a bucket_set
        // we have to had done a `bucket_get` at the same index for it to make
        // sense.
        let old_val = self.buf.as_mut_slice().get_mut(a_idx).unwrap();
        let mask = (1 << BITS_PER_BUCKET) - 1;                // selects the right-most bucket
        let select_in_bucket = mask << shift_amount;          // selects the correct bucket
        let select_out_of_bucket = !select_in_bucket;         // selects everything except the correct bucket
        let new_array_val = (new_val << shift_amount)         // move the new_val into the right spot
                          | (*old_val & select_out_of_bucket); // mask out the old value, and or it with the new one
        *old_val = new_array_val;
    }

    /// Insert a stretched hash into the bloom filter, remembering to saturate
    /// the counter instead of overflowing.
    fn insert_shash(&mut self, shash: uint) {
        let (a_idx, shift_amount) = self.shash_to_array_index(shash);
        let b_val = self.bucket_get(a_idx, shift_amount);


        // saturate the count.
        if b_val == 0xF {
            return;
        }

        let new_val = b_val + 1;

        self.bucket_set(a_idx, shift_amount, new_val);
    }

    /// Insert a hashed value into the bloom filter.
    fn insert_hashed(&mut self, hash: u64) {
        self.number_of_insertions += 1;
        for h in stretch(&mut to_rng(hash)) {
            self.insert_shash(h);
        }
    }

    /// Inserts a value into the bloom filter. Note that the bloom filter isn't
    /// parameterized over the values it holds. That's because it can hold
    /// values of different types, as long as it can get a hash out of them.
    pub fn insert<H: Hash<FnvState>>(&mut self, h: &H) {
        self.insert_hashed(hash(h))
    }

    /// Removes a stretched hash from the bloom filter, taking care not to
    /// decrememnt saturated counters.
    ///
    /// It is an error to remove never-inserted elements.
    fn remove_shash(&mut self, shash: uint) {
        let (a_idx, shift_amount) = self.shash_to_array_index(shash);
        let b_val = self.bucket_get(a_idx, shift_amount);
        assert!(b_val != 0, "Removing an element that was never inserted.");

        // can't do anything if the counter saturated.
        if b_val == 0xF { return; }

        self.bucket_set(a_idx, shift_amount, b_val - 1);
    }

    /// Removes a hashed value from the bloom filter.
    fn remove_hashed(&mut self, hash: u64) {
        self.number_of_insertions -= 1;
        for h in stretch(&mut to_rng(hash)) {
            self.remove_shash(h);
        }
    }

    /// Removes a value from the bloom filter.
    ///
    /// Be careful of adding and removing lots of elements, especially for
    /// long-lived bloom filters. The counters in each bucket will saturate if
    /// 16 or more elements hash to it, and then stick there. This will hurt
    /// your false positive rate. To fix this, you might consider refreshing the
    /// bloom filter by `clear`ing it, and then reinserting elements at regular,
    /// long intervals.
    ///
    /// It is an error to remove never-inserted elements.
    pub fn remove<H: Hash<FnvState>>(&mut self, h: &H) {
        self.remove_hashed(hash(h))
    }

    /// Returns `true` if the bloom filter cannot possibly contain the given
    /// stretched hash.
    fn definitely_excludes_shash(&self, shash: uint) -> bool {
        let (a_idx, shift_amount) = self.shash_to_array_index(shash);
        self.bucket_get(a_idx, shift_amount) == 0
    }

    /// A hash is definitely excluded iff none of the stretched hashes are in
    /// the bloom filter.
    fn definitely_excludes_hashed(&self, hash: u64) -> bool {
        let mut ret = false;

        // Doing `.any` is slower than this branch-free version.
        for shash in stretch(&mut to_rng(hash)) {
            ret |= self.definitely_excludes_shash(shash);
        }

        ret
    }

    /// A bloom filter can tell you whether or not a value has definitely never
    /// been inserted. Note that bloom filters can give false positives.
    pub fn definitely_excludes<H: Hash<FnvState>>(&self, h: &H) -> bool {
        self.definitely_excludes_hashed(hash(h))
    }

    /// A bloom filter can tell you if an element /may/ be in it. It cannot be
    /// certain. But, assuming correct usage, this query will have a low false
    /// positive rate.
    pub fn may_include<H: Hash<FnvState>>(&self, h: &H) -> bool {
        !self.definitely_excludes(h)
    }

    /// Returns the number of elements ever inserted into the bloom filter - the
    /// number of elements removed.
    pub fn number_of_insertions(&self) -> uint {
        self.number_of_insertions
    }

    /// Returns the number of bytes of memory the bloom filter uses.
    pub fn size(&self) -> uint {
        self.buf.len() * uint::BYTES
    }

    /// Removes all elements from the bloom filter. This is both more efficient
    /// and has better false-positive properties than repeatedly calling `remove`
    /// on every element.
    pub fn clear(&mut self) {
        self.number_of_insertions = 0;
        for x in self.buf.as_mut_slice().iter_mut() {
            *x = 0u;
        }
    }
}

#[test]
fn create_and_insert_some_stuff() {
    use std::iter::range;

    let mut bf = BloomFilter::new(1000);

    for i in range(0u, 1000) {
        bf.insert(&i);
    }

    assert_eq!(bf.number_of_insertions(), 1000);

    for i in range(0u, 1000) {
        assert!(bf.may_include(&i));
    }

    let false_positives =
        range(1001u, 2000).filter(|i| bf.may_include(&i)).count();

    assert!(false_positives < 10) // 1%.

    for i in range(0u, 100) {
        bf.remove(&i);
    }

    assert_eq!(bf.number_of_insertions(), 900);

    for i in range(100u, 1000) {
        assert!(bf.may_include(&i));
    }

    let false_positives = range(0u, 100).filter(|i| bf.may_include(&i)).count();

    assert!(false_positives < 2); // 2%.

    bf.clear();

    assert_eq!(bf.number_of_insertions(), 0);

    for i in range(0u, 2000) {
        assert!(bf.definitely_excludes(&i));
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
            let mut bf = BloomFilter::new(1000);
            for i in iter::range(0u, 1000) {
                bf.insert(&i);
            }
            for i in iter::range(0u, 100) {
                bf.remove(&i);
            }
            for i in iter::range(100u, 200) {
                test::black_box(bf.may_include(&i));
            }
        });
    }

    #[bench]
    fn may_include(b: &mut test::Bencher) {
        let mut bf = BloomFilter::new(1000);

        for i in iter::range(0u, 1000) {
            bf.insert(&i);
        }

        let mut i = 0u;

        b.bench_n(1000, |b| {
            b.iter(|| {
                test::black_box(bf.may_include(&i));
                i += 1;
            });
        });
    }

    #[bench]
    fn insert(b: &mut test::Bencher) {
        let mut bf = BloomFilter::new(1000);

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
        let mut bf = BloomFilter::new(1000);
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

        test::black_box(bf.may_include(&0u));
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
