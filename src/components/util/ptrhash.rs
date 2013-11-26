/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

//! A hash set for pointers. Assumes that addresses are 16-byte aligned. Loosely based on the
//! hash set of the Boehm GC described here:
//!
//!     http://www.hpl.hp.com/personal/Hans_Boehm/gc/tree.html
//!
//! We use the top 55 bits as the hash key, then use a 64-bit bitset to store the next 6 bits. The
//! bottom 3 bits are ignored as we assume 16-byte alignment.
//!
//! This data structure is quite fast; accesses are around 90 ns on a 2.7 GHz Core i7, and most of
//! that time is spent in SipHash. A more optimized hash table could probably reduce this by an
//! order of magnitude.

use std::hashmap::{HashMap, HashMapIterator};

#[cfg(test)]
use extra::test::BenchHarness;

/// The size of a bit set.
type BitSet = u64;

/// The number of bits in the "lower section" (bitfield of the hash table).
static LOWER_BITS: u64 = 6;

/// The number of bits ignored.
static ALIGN_BITS: u64 = 3;

/// The hash key.
///
/// FIXME(pcwalton): This should have a more efficient hash implementation, but that doesn't work
/// at the moment because of the coherence rules.
#[deriving(Clone, IterBytes)]
struct HashKey(u64);

impl HashKey {
    #[inline]
    fn from_address(address: u64) -> HashKey {
        HashKey(address >> (ALIGN_BITS + LOWER_BITS))
    }
}

impl Eq for HashKey {
    #[inline]
    fn eq(&self, other: &HashKey) -> bool {
        **self == **other
    }
}

/// A hash set for 64-bit pointers.
#[deriving(Clone)]
pub struct PtrHashSet {
    priv map: HashMap<HashKey,BitSet>,
}

impl PtrHashSet {
    pub fn init() -> PtrHashSet {
        PtrHashSet {
            map: HashMap::new(),
        }
    }

    #[inline]
    fn mask_for_address(address: u64) -> u64 {
        0x8000_0000_0000_0000 >> ((address >> ALIGN_BITS) & ((1 << LOWER_BITS) - 1))
    }

    #[inline]
    pub fn iter<'a>(&'a self) -> PtrHashSetIterator<'a> {
        let mut iter = PtrHashSetIterator {
            hash_iter: self.map.iter(),
            value: Some((HashKey(0), 0, 0)),
        };
        iter.pump();
        iter
    }
}

impl Container for PtrHashSet {
    fn len(&self) -> uint {
        fail!("len() unimplemented")
    }
}

impl Mutable for PtrHashSet {
    fn clear(&mut self) {
        fail!("clear() unimplemented")
    }
}

impl Set<u64> for PtrHashSet {
    #[inline]
    fn contains(&self, address: &u64) -> bool {
        match self.map.find(&HashKey::from_address(*address)) {
            None => false,
            Some(&bitset) => bitset & PtrHashSet::mask_for_address(*address) != 0,
        }
    }

    fn is_disjoint(&self, _: &PtrHashSet) -> bool {
        fail!("is_disjoint() unimplemented")
    }

    fn is_subset(&self, _: &PtrHashSet) -> bool {
        fail!("is_subset() unimplemented")
    }

    fn is_superset(&self, _: &PtrHashSet) -> bool {
        fail!("is_superset() unimplemented")
    }
}

impl MutableSet<u64> for PtrHashSet {
    fn insert(&mut self, address: u64) -> bool {
        let key = HashKey::from_address(address);
        let mask = PtrHashSet::mask_for_address(address);
        match self.map.find_mut(&HashKey::from_address(address)) {
            Some(ptr) => {
                if *ptr & mask != 0 {
                    return true
                } else {
                    *ptr = *ptr | mask;
                    return false
                }
            }
            None => {}
        }
        self.map.insert(key, mask)
    }

    fn remove(&mut self, address: &u64) -> bool {
        let key = HashKey::from_address(*address);
        let mask = PtrHashSet::mask_for_address(*address);
        match self.map.find_mut(&key) {
            Some(ptr) => {
                // TODO(pcwalton): Maybe remove the key here if possible? Not sure if it's worth
                // it.
                if *ptr & mask != 0 {
                    *ptr = *ptr & !mask;
                    return true
                } else {
                    return false
                }
            }
            None => return false
        }
    }
}

/// An iterator for a pointer hash set.

pub struct PtrHashSetIterator<'a> {
    hash_iter: HashMapIterator<'a,HashKey,BitSet>,
    priv value: Option<(HashKey, BitSet, u64)>,
}

impl<'a> PtrHashSetIterator<'a> {
    #[inline]
    fn pump(&mut self) {
        let (mut key, mut bitset, _) = self.value.unwrap();

        // The next index will be the number of leading zeroes in the bitset.
        loop {
            let index = bitset.leading_zeros();
            if index == 64 {
                // We're at the end of this bitset. Fetch the next from the iterator.
                match self.hash_iter.next() {
                    None => {
                        // Done!
                        self.value = None;
                        break
                    }
                    Some((&new_key, &new_bitset)) => {
                        key = new_key;
                        bitset = new_bitset;
                        continue
                    }
                }
            }

            bitset &= !(0x8000_0000_0000_0000 >> index);
            self.value = Some((key, bitset, index));
            break
        }
    }
}

impl<'a> Iterator<u64> for PtrHashSetIterator<'a> {
    #[inline]
    fn next(&mut self) -> Option<u64> {
        match self.value {
            Some((key, _, index)) => {
                let addr = (*key << (ALIGN_BITS + LOWER_BITS)) | (index << ALIGN_BITS);
                self.pump();
                Some(addr)
            }
            None => None
        }
    }
}

#[bench]
fn test_insert_ascending(harness: &mut BenchHarness) {
    let mut table = PtrHashSet::init();
    let mut addr: u64 = 0x7fff_ffff_0000_0000;
    harness.iter(|| {
        table.insert(addr);
        addr += 32;
    })
}

#[bench]
fn test_lookup_ascending(harness: &mut BenchHarness) {
    let mut table = PtrHashSet::init();
    let base: u64 = 0x7fff_ffff_0000_0000;
    for i in range(0, 1000000) {
        table.insert(base + (i as u64) * 32);
    }

    let mut addr = base;
    harness.iter(|| {
        let _ = table.contains(&addr);
        addr += 32;
    })
}

#[test]
fn test_insert() {
    let mut table = PtrHashSet::init();
    assert!(!table.contains(&0x12345678));
    table.insert(0x12345678);
    assert!(table.contains(&0x12345678));
    assert!(!table.contains(&0x12345680));
    table.insert(0x12345680);
    assert!(table.contains(&0x12345680));
}

#[test]
fn test_iter() {
    let mut table = PtrHashSet::init();
    let mut addrs: ~[u64] = table.iter().collect();
    assert_eq!(addrs.len(), 0);

    table.insert(0x12345678);
    addrs = table.iter().collect();
    assert_eq!(addrs, ~[0x12345678]);

    table.insert(0x87654320);
    addrs = table.iter().collect();
    assert_eq!(addrs.len(), 2);
    assert!(addrs.contains(&0x12345678));
    assert!(addrs.contains(&0x87654320));

    table.remove(&0x12345678);
    addrs = table.iter().collect();
    assert_eq!(addrs, ~[0x87654320]);
}

